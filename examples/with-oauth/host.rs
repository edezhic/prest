#![feature(lazy_cell)]

#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "./pub"]
struct Assets;

use pwrs::*;
use pwrs::host::auth::*;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};
use tokio::sync::RwLock;

static GCLIENT: LazyLock<GoogleClient> = LazyLock::new(|| {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(GoogleClient::init(
            "http://localhost",
            std::env::var("GOOGLE_CLIENT_ID").unwrap(),
            std::env::var("GOOGLE_CLIENT_SECRET").unwrap(),
        ))
    })
});

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub pw_hash: String,
}

impl AuthUser<u64> for User {
    fn get_id(&self) -> u64 {
        self.id
    }
    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.pw_hash.clone().into())
    }
}

pub type AuthCtx = AuthContext<u64, User, AuthMemoryStore<u64, User>>;
pub type RequireAuthzLayer = RequireAuthorizationLayer<u64, User>;

#[tokio::main]
async fn main() {
    pwrs::host::set_dot_env_variables();
    let (auth_svc, session, authn) = init_auth::<u64, User>();
    let service = pwrs::Router::new()
        .route("/protected", get(|| async {"Authorized!"}))
        .route_layer(RequireAuthzLayer::login())
        .merge(shared::service())
        .merge(auth_svc)
        .layer(authn)
        .layer(session)
        .layer(pwrs::host::embed(Assets));
    pwrs::host::serve(service, 80).await.unwrap();
}

use std::hash::Hash;
pub fn init_auth<Id: Hash + Eq + Clone + Send + Sync + 'static, User: AuthUser<Id>>() -> (
    Router,
    SessionLayer<SessionMemoryStore>,
    AuthLayer<AuthMemoryStore<Id, User>, Id, User>,
) {
    let secret = pwrs::host::generate_secret::<[u8; 64]>();
    let session_store = SessionMemoryStore::new();
    let auth_store = Arc::new(RwLock::new(HashMap::new()));

    let svc = Router::new()
        .route("/oauth/google", get(init_oauth_flow))
        .route("/oauth/google/callback", get(callback))
        .route("/logout", get(logout));

    (
        svc,
        SessionLayer::new(session_store, &secret).with_same_site_policy(SameSite::Lax),
        AuthLayer::new(AuthMemoryStore::new(&auth_store), &secret),
    )
}


pub async fn init_oauth_flow(mut session: WritableSession) -> impl IntoResponse {
    let (authz_url, csrf_token, nonce) = GCLIENT.authz_request(&["email"]);
    session.insert("nonce", nonce).unwrap();
    session.insert("csrf", csrf_token).unwrap();
    Redirect::to(authz_url.as_ref())
}

pub async fn callback(
    session: ReadableSession,
    Query(query): Query<OAuthQuery>,
    mut auth: AuthCtx,
) -> impl IntoResponse {
    let Some(initial_csrf) = session.get::<CsrfToken>("csrf") else { panic!("missing csrf!") };
    let Some(nonce) = session.get::<Nonce>("nonce") else { panic!("missing nonce!") };

    // remove existing/anonymous session
    drop(session);

    // validate CSRF
    if initial_csrf.secret() != query.state.secret() {
        return Redirect::to("/");
    }
    let (_token, claims) = GCLIENT.get_token_and_claims(query.code, nonce).await;
    let email = claims.email().unwrap().to_string();
    let user = User {
        id: 1,
        email,
        pw_hash: String::new()
    };
    /*
    let user = match crate::Storage::get_user_by_email(&email).await {
        Some(user) => user,
        None => User::signup(email, None).await.unwrap(),
    };
    */
    auth.login(&user).await.unwrap();

    Redirect::to("/")
}

async fn logout(mut auth: AuthCtx) -> impl IntoResponse {
    if let Some(_) = auth.current_user {
        auth.logout().await;
    }
    Redirect::to("/")
}
