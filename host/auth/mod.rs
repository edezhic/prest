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
#[derive(serde::Deserialize)]
struct Credentials {
    email: Email,
    password: String,
}

pub fn init() -> (
    Router,
    SessionLayer<MemoryStore>,
    AuthLayer<crate::Storage, UserId, User, Role>,
) {
    let secret = rand::Rng::gen::<[u8; 64]>(&mut rand::thread_rng());
    let session_store = MemoryStore::new();

    let svc = Router::new()
        .route("/signup", get(signup))
        .route("/login", get(login))
        .route("/logout", get(logout));

    #[cfg(feature = "oauth")]
    let svc = svc
        .route("/oauth/google", get(oauth::authorize))
        .route("/oauth/google/callback", get(oauth::callback));

    (
        svc,
        SessionLayer::new(session_store, &secret).with_same_site_policy(SameSite::Lax),
        AuthLayer::new(crate::Storage, &secret),
    )
}

async fn signup(mut auth: AuthContext, Form(creds): Form<Credentials>) -> impl IntoResponse {
    let user = User::signup(creds.email, Some(creds.password)).await.unwrap();
    auth.login(&user)
        .await
        .unwrap();
    Redirect::to("/authorized")
}

async fn login(mut auth: AuthContext, Form(creds): Form<Credentials>) -> impl IntoResponse {
    let user = crate::Storage::get_user_by_email(&creds.email).await.unwrap();
    auth.login(&user)
        .await
        .unwrap();
    Redirect::to("/authorized")
}

async fn logout(mut auth: AuthContext) -> impl IntoResponse {
    if let Some(_) = auth.current_user {
        tracing::info!("Logging out user: {:?}", &auth.current_user);
        auth.logout().await;
    }
    Redirect::to("/")
}
