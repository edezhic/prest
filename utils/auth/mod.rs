mod google;
pub use google::GoogleClient;
pub use axum_login::{
    axum_sessions::{
        async_session::{MemoryStore as SessionMemoryStore},
        extractors::{ReadableSession, WritableSession},
        SameSite, SessionLayer,
    },
    extractors::AuthContext,
    memory_store::MemoryStore as AuthMemoryStore,
    secrecy::SecretVec,
    AuthLayer, AuthUser, RequireAuthorizationLayer,
};
pub use openidconnect::{reqwest::async_http_client as oauth_http_client, CsrfToken, Nonce};

#[derive(Debug, serde::Deserialize)]
pub struct OAuthQuery {
    pub code: String,
    pub state: openidconnect::CsrfToken,
}
