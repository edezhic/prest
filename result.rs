use crate::*;

/// Basic Result alias with [`enum@prest::Error`]`
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

/// Utility for type inference that allows using `?` operator in closure handlers
pub const OK: Result<(), Error> = Result::Ok(());

/// Utility for type inference that allows using `?` operator in closure handlers
pub const fn ok<T: IntoResponse>(resp: T) -> Result<T, Error> {
    Ok(resp)
}

use thiserror::Error;
/// Error type used across prest codebase
#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal")]
    Internal,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Not found")]
    NotFound,
    #[error(transparent)]
    Env(#[from] std::env::VarError),
    #[cfg(all(host, feature = "auth"))]
    #[error(transparent)]
    Session(#[from] tower_sessions::session_store::Error),
    #[cfg(all(host, feature = "auth"))]
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[cfg(all(host, feature = "auth"))]
    #[error(transparent)]
    OAuth(#[from] openidconnect::ClaimsVerificationError),
    #[cfg(feature = "db")]
    #[error(transparent)]
    GlueSQL(#[from] gluesql::core::error::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    FormRejection(#[from] axum::extract::rejection::FormRejection),
    #[error(transparent)]
    QueryRejection(#[from] axum::extract::rejection::QueryRejection),
    #[cfg(host)]
    #[error(transparent)]
    RuSSH(#[from] russh::Error),
    #[cfg(host)]
    #[error(transparent)]
    RuSFTP(#[from] russh_sftp::client::error::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::FormRejection(e) => e.into_response(),
            Error::QueryRejection(e) => e.into_response(),
            Error::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            #[cfg(all(host, feature = "auth"))]
            Error::Auth(_) | Error::Session(_) | Error::OAuth(_) => {
                StatusCode::UNAUTHORIZED.into_response()
            }
            Error::NotFound => StatusCode::NOT_FOUND.into_response(),
            _ => {
                error!("{self}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

