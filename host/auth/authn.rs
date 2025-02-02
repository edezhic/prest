use crate::*;

use axum_login::AuthnBackend;
use password_auth::verify_password;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Failed to load: {0}")]
    DbError(String),
}

#[async_trait]
impl AuthnBackend for Prest {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        match creds {
            Credentials::GoogleOpenID { code, nonce } => {
                if !*WITH_GOOGLE_AUTH {
                    warn!("Attempted to authenticate with google credentials without google credentials!");
                    return Ok(None); // TODO an error here
                }
                let Ok(email) = GOOGLE_CLIENT.get_email(code, nonce).await else {
                    return Ok(None); // TODO an error here
                };
                let maybe_user = match User::select_by_email(&email).await {
                    Ok(v) => v,
                    Err(e) => return Err(AuthError::DbError(format!("User load error: {e}"))),
                };
                match maybe_user {
                    Some(user) => Ok(Some(user)),
                    None => {
                        let user = User::from_email(email);
                        user.save()
                            .await
                            .map_err(|e| AuthError::UserNotFound(e.to_string()))?;
                        Ok(Some(user))
                    }
                }
            }
            Credentials::UsernamePassword { username, password } => {
                let maybe_user = match User::select_by_username(&username).await {
                    Ok(v) => v,
                    Err(e) => return Err(AuthError::DbError(format!("User load error: {e}"))),
                };

                let Some(user) = maybe_user else {
                    return Ok(None); // TODO an error here
                };
                let Some(pw_hash) = &user.password_hash else {
                    return Ok(None); // TODO an error here
                };
                let Ok(()) = verify_password(password, pw_hash) else {
                    return Ok(None); // TODO an error here
                };
                Ok(Some(user))
            }
            Credentials::EmailPassword { email, password } => {
                let maybe_user = match User::select_by_email(&email).await {
                    Ok(v) => v,
                    Err(e) => return Err(AuthError::DbError(format!("User load error: {e}"))),
                };

                let Some(user) = maybe_user else {
                    return Ok(None); // TODO an error here
                };
                let Some(pw_hash) = &user.password_hash else {
                    return Ok(None); // TODO an error here
                };
                let Ok(()) = verify_password(password, pw_hash) else {
                    return Ok(None); // TODO an error here
                };
                Ok(Some(user))
            }
        }
    }

    async fn get_user(
        &self,
        user_id: &axum_login::UserId<Self>,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        let maybe_user = match User::select_by_id(user_id).await {
            Ok(v) => v,
            Err(e) => return Err(AuthError::DbError(format!("User load error: {e}"))),
        };
        Ok(maybe_user)
    }
}
