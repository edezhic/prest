mod google_openid;
use axum::http::request::Parts;
pub use google_openid::GOOGLE_CLIENT;
use tower_sessions::{Expiry, SessionManagerLayer};

use crate::*;
use axum_login::{
    AuthManagerLayer, AuthManagerLayerBuilder, AuthSession, AuthUser, AuthnBackend, UserId,
};
pub use openidconnect::{CsrfToken as OAuthCSRF, Nonce as OAuthNonce};
pub use tower_sessions::Session;
use tower_sessions::{
    session::{Id, Record},
    session_store::{Error, Result},
    SessionStore,
};
use password_auth::{generate_hash, verify_password};

pub type AuthLayer = AuthManagerLayer<Db, Db>;

pub fn init_auth() -> (AuthLayer, Router) {
    SessionRow::migrate();
    User::migrate();
    let mut session_layer = SessionManagerLayer::new(DB.clone())
        .with_name("prest_session")
        .with_same_site(tower_sessions::cookie::SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(7)));
    if let Ok(domain) = env::var("DOMAIN") {
        session_layer = session_layer.with_domain(domain);
    }
    let layer = AuthManagerLayerBuilder::new(DB.clone(), session_layer).build();

    let router = route("/auth/username_password/login", post(username_password_login))
        .route("/auth/username_password/signup", post(username_password_signup))
        .route("/auth/google", get(init_google_oauth))
        .route("/auth/google/callback", get(google_oauth_callback))
        .route("/auth/logout", get(logout));

    (layer, router)
}

#[derive(Debug, serde::Deserialize)]
struct UsernamePasswordForm {
    username: String,
    password: String,
    redirect: Option<String>,
}

async fn username_password_login(mut auth: Auth, Form(form): Form<UsernamePasswordForm>) -> impl IntoResponse {
    let UsernamePasswordForm { username, password, redirect } = form;

    let credentials = Credentials::UsernamePassword { username, password };
    let Ok(Some(user)) = auth.authenticate(credentials).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    if auth.login(&user).await.is_err() {
        #[cfg(debug_assertions)]
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        #[cfg(not(debug_assertions))]
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if let Some(next) = redirect {
        Redirect::to(&next).into_response()
    } else {
        Redirect::to("/").into_response()
    }
}

async fn username_password_signup(mut auth: Auth, Form(form): Form<UsernamePasswordForm>) -> impl IntoResponse {
    let UsernamePasswordForm { username, password, redirect } = form;

    if User::find_by_username(&username).is_some() {
        return StatusCode::CONFLICT.into_response()
    }

    let user = User::from_username_password(username, password);

    if auth.login(&user).await.is_err() {
        #[cfg(debug_assertions)]
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        #[cfg(not(debug_assertions))]
        return StatusCode::UNAUTHORIZED.into_response();
    }

    if let Some(next) = redirect {
        Redirect::to(&next).into_response()
    } else {
        Redirect::to("/").into_response()
    }
}

#[derive(Debug, serde::Deserialize)]
struct NextUrl {
    next: Option<String>,
}

const CSRF_KEY: &str = "oauth_csrf";
const NONCE_KEY: &str = "oauth_nonce";
const REDIRECT_KEY: &str = "after_auth_redirect";

async fn init_google_oauth(session: Session, Query(NextUrl { next }): Query<NextUrl>) -> impl IntoResponse {
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

async fn google_oauth_callback(session: Session, Query(query): OAuthQuery, mut auth: Auth) -> impl IntoResponse {
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
        auth.logout().await.unwrap();
    }
    Redirect::to("/")
}

pub type Auth = AuthSession<Db>;

pub type OAuthQuery = Query<OAuthQueryParams>;
pub type OAuthCode = String;

#[derive(Debug, serde::Deserialize)]
pub struct OAuthQueryParams {
    pub code: OAuthCode,
    pub state: OAuthCSRF,
}

#[derive(Table, Clone, Debug)]
pub struct User {
    pub id: Uuid,
    #[unique_column]
    pub username: Option<String>,
    #[unique_column]
    pub email: Option<String>,
    pub password_hash: Option<String>,
}

impl User {
    pub fn from_email(email: String) -> Self {
        Self {
            id: generate_uuid(),
            username: None,
            email: Some(email),
            password_hash: None,
        }
    }
    pub fn from_username_password(username: String, password: String) -> Self {
        Self {
            id: generate_uuid(),
            username: Some(username),
            email: None,
            password_hash: Some(generate_hash(password)),
        }
    }
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
        auth_session.user.ok_or(StatusCode::UNAUTHORIZED)
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

#[derive(Clone, Debug)]
pub enum Credentials {
    UsernamePassword { username: String, password: String },
    EmailPassword { email: String, password: String },
    GoogleOpenID { code: OAuthCode, nonce: OAuthNonce },
}

#[async_trait]
impl AuthnBackend for Db {
    type User = User;
    type Credentials = Credentials;
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        match creds {
            Credentials::GoogleOpenID { code, nonce } => {
                let Ok(email) = GOOGLE_CLIENT.get_email(code, nonce).await else {
                    return Ok(None); // TODO an error here
                };
                match User::find_by_email(&email) {
                    Some(user) => Ok(Some(user)),
                    None => {
                        let user = User::from_email(email);
                        user.save().unwrap();
                        Ok(Some(user))
                    }
                }
            }
            Credentials::UsernamePassword { username, password } => {
                let Some(user) = User::find_by_username(&username) else {
                    return Ok(None) // TODO an error here
                };
                let Some(pw_hash) = &user.password_hash else {
                    return Ok(None) // TODO an error here
                };
                let Ok(()) = verify_password(password, pw_hash) else {
                    return Ok(None) // TODO an error here
                };
                Ok(Some(user))
            }
            _ => todo!("auth with other {creds:?}"),
        }
    }

    async fn get_user(
        &self,
        user_id: &UserId<Self>,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        Ok(User::find_by_id(user_id))
    }
}

#[derive(Table, Debug)]
pub struct SessionRow {
    pub id: Uuid,
    pub record: String,
}

#[async_trait]
impl SessionStore for Db {
    async fn save(&self, record: &Record) -> Result<()> {
        let id = record.id.0;
        let record = match serde_json::to_string(record) {
            Ok(s) => s,
            Err(e) => return Err(Error::Encode(format!("{e}"))),
        };
        match (SessionRow { id, record }).save() {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Backend(format!("Session save error: {e}"))),
        }
    }

    async fn load(&self, session_id: &Id) -> Result<Option<Record>> {
        let search = SessionRow::find_by_id(&session_id.0);
        let Some(session_row) = search else {
            return Ok(None);
        };
        match serde_json::from_str(&session_row.record) {
            Ok(record) => Ok(Some(record)),
            Err(e) => Err(Error::Decode(format!("Session load error: {e}"))),
        }
    }

    async fn delete(&self, session_id: &Id) -> Result<()> {
        match SessionRow::delete_by_key(&session_id.0) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Backend(format!("Session deletion error: {e}"))),
        }
    }
}
