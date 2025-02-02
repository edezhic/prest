use crate::*;

/// Basic Result alias with [`enum@prest::Error`]
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

/// Utility for type inference that allows using `?` operator in closure handlers
pub const OK: prest::Result<(), Error> = prest::Result::Ok(());

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
    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),
    #[error(transparent)]
    UuidParse(#[from] uuid::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    HttpError(#[from] http::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[cfg(all(host, feature = "auth"))]
    #[error(transparent)]
    Session(#[from] tower_sessions::session_store::Error),
    #[cfg(all(host, feature = "auth"))]
    #[error(transparent)]
    Auth(#[from] crate::host::auth::authn::AuthError),
    #[cfg(all(host, feature = "auth"))]
    #[error(transparent)]
    Login(#[from] ::axum_login::Error<Prest>),
    #[cfg(all(host, feature = "auth"))]
    #[error(transparent)]
    OpenIDClaimVerification(#[from] openidconnect::ClaimsVerificationError),
    #[cfg(feature = "db")]
    #[error(transparent)]
    SQL(#[from] gluesql_core::error::Error),
    #[cfg(all(host, feature = "db"))]
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[cfg(all(host, feature = "db"))]
    #[error(transparent)]
    Bincode(#[from] bitcode::Error),
    #[error(transparent)]
    FormRejection(#[from] axum::extract::rejection::FormRejection),
    #[error(transparent)]
    QueryRejection(#[from] axum::extract::rejection::QueryRejection),
    #[error(transparent)]
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[cfg(host)]
    #[error(transparent)]
    RuSSH(#[from] russh::Error),
    #[cfg(host)]
    #[error(transparent)]
    RuSFTP(#[from] russh_sftp::client::error::Error),
    #[cfg(host)]
    #[error(transparent)]
    Channel(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("{0:?}")]
    Any(AnyError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::FormRejection(e) => e.into_response(),
            Error::QueryRejection(e) => e.into_response(),
            Error::JsonRejection(e) => e.into_response(),
            Error::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            #[cfg(all(host, feature = "auth"))]
            Error::Login(_)
            | Error::Auth(_)
            | Error::Session(_)
            | Error::OpenIDClaimVerification(_) => StatusCode::UNAUTHORIZED.into_response(),
            Error::NotFound => StatusCode::NOT_FOUND.into_response(),
            _ => {
                error!("{self}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

/// Provides shorthand to map errs into [`prest::Error`] using `.somehow()`
#[doc(hidden)]
pub trait _Somehow<T, E> {
    fn somehow(self) -> Result<T, Error>;
}

impl<T, E> _Somehow<T, E> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn somehow(self) -> Result<T, Error> {
        self.map_err(|e| Error::Any(AnyError(format!("{e}"))))
    }
}

#[derive(Debug)]
#[doc(hidden)]
pub struct AnyError(pub String);
impl<E: std::error::Error> From<E> for AnyError {
    fn from(value: E) -> Self {
        AnyError(format!("{value}"))
    }
}

/// Anyhow-like result which can be `?` from any error type
pub type Somehow<T = ()> = std::result::Result<T, AnyError>;

/// Shorthand to create formatted [`prest::Error`] values like `e!("{x:?}")`
#[macro_export]
macro_rules! e {
    ($($tokens:tt),+) => {
        prest::Error::Any(prest::AnyError(format!($($tokens),+)))
    };
}

// #[cfg(host)]
// impl From<::sled::transaction::ConflictableTransactionError<Box<bitcode::ErrorKind>>> for Error {
//     fn from(
//         value: ::sled::transaction::ConflictableTransactionError<Box<bitcode::ErrorKind>>,
//     ) -> Self {
//         e!("{value}")
//     }
// }
