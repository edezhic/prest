mod user;
pub use user::*;

#[cfg(feature = "oauth")]
mod oauth;

use axum::{
    response::{IntoResponse, Redirect},
    routing::get,
    Form, Router,
};
use axum_login::{
    axum_sessions::{async_session::MemoryStore, SameSite, SessionLayer},
    AuthLayer, RequireAuthorizationLayer,
};

pub type AuthContext = axum_login::extractors::AuthContext<UserId, User, crate::Storage, Role>;
pub type RequireAuthzLayer = RequireAuthorizationLayer<UserId, User, Role>;

pub async fn init() -> (
    Router,
    SessionLayer<MemoryStore>,
    AuthLayer<crate::Storage, UserId, User, Role>,
) {
    let secret = rand::Rng::gen::<[u8; 64]>(&mut rand::thread_rng());
    let session_store = MemoryStore::new();
    let user_store = crate::Storage;

    let svc = Router::new().route("/auth/logout", get(logout));

    #[cfg(feature = "oauth")]
    let svc = svc
        .route("/auth/login", get(oauth::login))
        .route("/auth/signup", get(oauth::login))
        .route("/auth/callback", get(oauth::callback));

    #[cfg(not(feature = "oauth"))]
    let svc = svc.route("/auth/signup", get(signup));

    (
        svc,
        SessionLayer::new(session_store, &secret).with_same_site_policy(SameSite::Lax),
        AuthLayer::new(user_store, &secret),
    )
}

#[derive(serde::Deserialize)]
struct SignUp {
    email: String,
    password: String,
}
async fn signup(Form(sign_up): Form<SignUp>) {
    // ...
}

async fn logout(mut auth: AuthContext) -> impl IntoResponse {
    if let Some(_) = auth.current_user {
        tracing::info!("Logging out user: {:?}", &auth.current_user);
        auth.logout().await;
    }
    Redirect::to("/")
}
