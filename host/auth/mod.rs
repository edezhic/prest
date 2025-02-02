pub(crate) mod authn;
mod google_openid;
mod permissions;
mod session;
pub(crate) use session::SessionRow;

use crate::*;
use axum::http::request::Parts;
use axum_login::{
    AuthManagerLayer, AuthManagerLayerBuilder, AuthSession, AuthUser, AuthnBackend, AuthzBackend,
};
pub use google_openid::{GOOGLE_CLIENT, WITH_GOOGLE_AUTH};
pub use openidconnect::{CsrfToken as OAuthCSRF, Nonce as OAuthNonce};
use password_auth::{generate_hash, verify_password};
use std::collections::HashSet;
pub use tower_sessions::Session;
use tower_sessions::{Expiry, SessionManagerLayer, SessionStore};

pub type UserId = Uuid;
pub type AuthLayer = AuthManagerLayer<Prest, Prest>;
pub type Auth = AuthSession<Prest>;
pub type OAuthCode = String;

#[derive(Storage, Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub permissions: Vec<String>,
    pub group: UserGroup,
    #[unique]
    pub username: Option<String>,
    #[unique]
    pub email: Option<String>,
    pub password_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum UserGroup {
    Default,
    Admin,
    Custom(String),
}

#[derive(Clone, Debug)]
pub enum Credentials {
    UsernamePassword { username: String, password: String },
    EmailPassword { email: String, password: String },
    GoogleOpenID { code: OAuthCode, nonce: OAuthNonce },
}

pub type OAuthQuery = axum::extract::Query<OAuthQueryParams>;
#[derive(Debug, Deserialize)]
pub struct OAuthQueryParams {
    pub code: OAuthCode,
    pub state: OAuthCSRF,
}

#[allow(dead_code)]
trait AuthBackend:
    AuthnBackend<User = User, Credentials = Credentials, Error = authn::AuthError>
    + SessionStore
    + Send
    + Sync
{
}
impl AuthBackend for Prest {}

pub const LOGIN_ROUTE: &str = "/auth/login";
pub const LOGOUT_ROUTE: &str = "/auth/logout";
pub const GOOGLE_LOGIN_ROUTE: &str = "/auth/google";
pub const GOOGLE_CALLBACK_ROUTE: &str = "/auth/google/callback";

pub fn init_auth_module() -> Result<(AuthLayer, Router)> {
    let mut session_layer = SessionManagerLayer::new(Prest)
        .with_name("prest_session")
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(30)));
    if let Some(domain) = APP_CONFIG.domain {
        session_layer = session_layer.with_domain(domain);
    }
    let layer = AuthManagerLayerBuilder::new(Prest, session_layer).build();

    let mut router = route(LOGIN_ROUTE, post(login)).route(LOGOUT_ROUTE, get(logout));

    if *WITH_GOOGLE_AUTH {
        router = router
            .route(GOOGLE_LOGIN_ROUTE, get(init_google_oauth))
            .route(GOOGLE_CALLBACK_ROUTE, get(google_oauth_callback));
    }

    Ok((layer, router))
}

impl User {
    pub fn from_email(email: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            permissions: vec![],
            group: UserGroup::Default,
            username: None,
            email: Some(email),
            password_hash: None,
        }
    }
    pub fn from_username_password(username: String, password: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            permissions: vec![],
            group: UserGroup::Default,
            username: Some(username),
            email: None,
            password_hash: Some(generate_hash(password)),
        }
    }
    pub fn from_email_password(email: String, password: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            permissions: vec![],
            group: UserGroup::Default,
            username: None,
            email: Some(email),
            password_hash: Some(generate_hash(password)),
        }
    }
    pub fn is_admin(&self) -> bool {
        self.group == UserGroup::Admin
    }
}

#[derive(Debug, Default, Deserialize)]
struct AuthForm {
    username: Option<String>,
    email: Option<String>,
    password: String,
    signup: bool,
    next: Option<String>,
}

async fn login(mut auth: Auth, Vals(form): Vals<AuthForm>) -> Result<Response> {
    let AuthForm {
        username,
        email,
        password,
        signup,
        next,
    } = form;

    let user = if signup {
        let new = if let Some(username) = username {
            if User::select_by_username(&username).await?.is_some() {
                return Ok(StatusCode::CONFLICT.into_response());
            }
            User::from_username_password(username, password)
        } else if let Some(email) = email {
            if User::select_by_email(&email).await?.is_some() {
                return Ok(StatusCode::CONFLICT.into_response());
            }
            User::from_email_password(email, password)
        } else {
            return Ok(StatusCode::BAD_REQUEST.into_response());
        };
        let Ok(_) = new.save().await else {
            return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        };
        new
    } else {
        if let Some(username) = username {
            let credentials = Credentials::UsernamePassword { username, password };
            let Ok(Some(user)) = auth.authenticate(credentials).await else {
                return Ok(StatusCode::UNAUTHORIZED.into_response());
            };
            user
        } else if let Some(email) = email {
            let credentials = Credentials::EmailPassword { email, password };
            let Ok(Some(user)) = auth.authenticate(credentials).await else {
                return Ok(StatusCode::UNAUTHORIZED.into_response());
            };
            user
        } else {
            return Ok(StatusCode::BAD_REQUEST.into_response());
        }
    };

    if auth.login(&user).await.is_err() {
        #[cfg(debug_assertions)]
        return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        #[cfg(not(debug_assertions))]
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }
    if let Some(next) = next {
        Ok(Redirect::to(&next).into_response())
    } else {
        Ok(Redirect::to("/").into_response())
    }
}

#[derive(Debug, Deserialize)]
struct NextUrl {
    next: Option<String>,
}

const CSRF_KEY: &str = "oauth_csrf";
const NONCE_KEY: &str = "oauth_nonce";
const REDIRECT_KEY: &str = "after_auth_redirect";

async fn init_google_oauth(
    session: Session,
    axum::extract::Query(NextUrl { next }): axum::extract::Query<NextUrl>,
) -> impl IntoResponse {
    let (authz_url, csrf_token, nonce) = GOOGLE_CLIENT.authz_request();
    let ins1 = session.insert(NONCE_KEY, nonce).await;
    let ins2 = session.insert(CSRF_KEY, csrf_token).await;
    let ins3 = if let Some(next) = next {
        session.insert(REDIRECT_KEY, next).await
    } else {
        Ok(())
    };
    if ins1.is_err() || ins2.is_err() || ins3.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Redirect::to(authz_url.as_str()).into_response()
}

async fn google_oauth_callback(
    session: Session,
    axum::extract::Query(query): OAuthQuery,
    mut auth: Auth,
) -> impl IntoResponse {
    let Ok(Some(initial_csrf)) = session.remove::<OAuthCSRF>(CSRF_KEY).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Ok(Some(nonce)) = session.remove::<OAuthNonce>(NONCE_KEY).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    if initial_csrf.secret() != query.state.secret() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let credentials = Credentials::GoogleOpenID {
        code: query.code,
        nonce,
    };
    let Ok(Some(user)) = auth.authenticate(credentials).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    if auth.login(&user).await.is_err() {
        #[cfg(debug_assertions)]
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        #[cfg(not(debug_assertions))]
        return StatusCode::UNAUTHORIZED.into_response();
    }

    if let Ok(Some(next)) = session.remove::<String>(REDIRECT_KEY).await {
        Redirect::to(&next).into_response()
    } else {
        Redirect::to("/").into_response()
    }
}

async fn logout(mut auth: Auth) -> impl IntoResponse {
    if let Some(_) = auth.user {
        auth.logout().await?;
    }
    ok(Redirect::to("/"))
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let Some(auth_session) = parts.extensions.get::<Auth>().cloned() else {
            #[cfg(debug_assertions)]
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
            #[cfg(not(debug_assertions))]
            return Err(StatusCode::UNAUTHORIZED);
        };
        let Some(user) = auth_session.user else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        if parts.uri.path().starts_with("/admin/") && !user.is_admin() {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(user)
    }
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        if let Some(password_hash) = &self.password_hash {
            password_hash.as_bytes()
        } else if let Some(email) = &self.email {
            email.as_bytes()
        } else if let Some(username) = &self.username {
            username.as_bytes()
        } else {
            self.id.as_bytes()
        }
    }
}
