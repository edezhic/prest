mod google;
use google::GoogleClient;

use axum_login::{AuthUser, AuthnBackend, UserId, AuthManagerLayerBuilder, login_required};
use prest::*;
use std::{collections::HashMap, env};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};
use openidconnect::{CsrfToken, Nonce};

static GCLIENT: OnceCell<GoogleClient> = OnceCell::const_new();
async fn gclient() -> &'static GoogleClient {
    GCLIENT.get_or_init(|| async {
        let client_id = env::var("GOOGLE_CLIENT_ID").unwrap();
        let client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap();
        GoogleClient::init(
            "http://localhost",
            client_id,
            client_secret,
        ).await
    }).await
}

type Email = String;

#[derive(Debug, Clone)]
pub struct User {
    pub email: Email,
}

impl AuthUser for User {
    type Id = Email;
    fn id(&self) -> Email {
        self.email.clone()
    }
    fn session_auth_hash(&self) -> &[u8] {
        self.email.as_bytes()
    }
}

#[derive(Clone, Default)]
struct Backend {
    users: HashMap<Email, User>,
}

type AuthSession = axum_login::AuthSession<Backend>;
type OAuthCode = String;

#[async_trait::async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = (OAuthCode, Nonce);
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        (code, nonce): Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let (_token, claims) = gclient().await.get_token_and_claims(code, nonce).await;
        let email = claims.email().unwrap().to_string();
        Ok(self.users.get(&email).cloned())
    }

    async fn get_user(
        &self,
        user_id: &UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        Ok(self.users.get(user_id).cloned())
    }
}

fn main() {
    dotenv::dotenv().unwrap();

    let admin_email = env::var("ADMIN_EMAIL").expect("Add an ADMIN_EMAIL env variable for a default user");
    let mut backend = Backend::default();
    backend.users.insert(admin_email.clone(), User { email: admin_email });

    let session_store = MemoryStore::default();
    let session_manager_layer = SessionManagerLayer::new(session_store);
    
    let auth_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(AuthManagerLayerBuilder::new(backend, session_manager_layer).build());
 
    Router::new()
        .route("/protected", get(html!(h1{"Authorized!"})))
        .route_layer(login_required!(Backend, login_url = "/oauth/google"))
        .route("/", get(homepage))
        .route("/oauth/google", get(init_oauth))
        .route("/oauth/google/callback", get(callback))
        .route("/logout", get(logout))
        .layer(auth_service)
        .serve(ServeOptions::default());
}

async fn homepage() -> Markup {
    html!(
        html {
            (Head::example("With Google OAuth"))
            body {
                h1{"With OAuth"}
                p{a href="/oauth/google" {"Click me to initiate Google OAuth flow"}}
                p{a href="/protected" {"Click me to go to the authorized route"}}
                (Scripts::default())
            }
        }
    )
}

async fn init_oauth(session: Session) -> impl IntoResponse {
    let (authz_url, csrf_token, nonce) = gclient().await.authz_request(&["email"]);
    session.insert("nonce", nonce).unwrap();
    session.insert("csrf", csrf_token).unwrap();
    Redirect::to(authz_url.as_ref())
}

#[derive(Debug, serde::Deserialize)]
pub struct OAuthQuery {
    pub code: OAuthCode,
    pub state: CsrfToken,
}

async fn callback(
    session: Session,
    Query(query): Query<OAuthQuery>,
    mut auth: AuthSession,
) -> impl IntoResponse {
    let Ok(Some(initial_csrf)) = session.get::<CsrfToken>("csrf") else {
        panic!("missing csrf!")
    };
    let Ok(Some(nonce)) = session.get::<Nonce>("nonce") else {
        panic!("missing nonce!")
    };

    // remove existing/anonymous session
    drop(session);

    // validate CSRF
    if initial_csrf.secret() != query.state.secret() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Ok(Some(user)) = auth.authenticate((query.code, nonce)).await else {
        panic!("no user!")
    };
    
    auth.login(&user).await.unwrap();

    Redirect::to("/protected").into_response()
}

async fn logout(mut auth: AuthSession) -> impl IntoResponse {
    if let Some(_) = auth.user {
        auth.logout().unwrap();
    }
    Redirect::to("/")
}

