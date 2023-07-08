mod user;
pub use user::*;

#[cfg(feature = "oauth")]
mod oauth;

use axum::{
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use axum_login::{
    axum_sessions::{
        async_session::MemoryStore as SessionMemoryStore,
        SameSite, SessionLayer,
    },
    AuthLayer, RequireAuthorizationLayer,
};

pub type AuthContext = axum_login::extractors::AuthContext<UserId, User, UserStore, Role>;
pub type RequireAuthzLayer = RequireAuthorizationLayer<UserId, User, Role>;

pub async fn init() -> (
    Router,
    SessionLayer<SessionMemoryStore>,
    AuthLayer<UserStore, UserId, User, Role>,
) {
    let secret = rand::Rng::gen::<[u8; 64]>(&mut rand::thread_rng());
    let session_store = SessionMemoryStore::new();
    let user_store = init_user_store().await;

    let svc = Router::new().route("/auth/logout", get(logout));

    #[cfg(feature = "oauth")]
    let svc = svc
        .route("/auth/login", get(oauth::login))
        .route("/auth/callback", get(oauth::callback));

    #[cfg(not(feature = "oauth"))]
    let svc = svc.route("/auth/login", get(|| async { "custom login stuff" }));

    (
        svc,
        SessionLayer::new(session_store, &secret).with_same_site_policy(SameSite::Lax),
        AuthLayer::new(user_store, &secret),
    )
}

async fn logout(mut auth: AuthContext) -> impl IntoResponse {
    if let Some(_) = auth.current_user {
        tracing::info!("Logging out user: {:?}", &auth.current_user);
        auth.logout().await;
    }
    Redirect::to("/")
}
