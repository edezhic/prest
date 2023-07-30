use axum_login::{secrecy::SecretVec, AuthUser};
use pbkdf2::{Pbkdf2, password_hash::{SaltString, PasswordHasher, PasswordHash, PasswordVerifier}};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
pub use email_address::EmailAddress as Email;
use std::sync::atomic::{AtomicU64, Ordering};
use anyhow::Result;

static NEW_USER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub pw_hash: PwHash,
    pub role: Role,
}

impl User {
    pub async fn signup(email: Email, password: Option<String>) -> Result<User> {
        let pw_hash = match password {
            Some(pw) => {
                let salt = SaltString::generate(&mut OsRng);
                let hash = Pbkdf2.hash_password(pw.as_bytes(), &salt).unwrap().to_string();
                PwHash(Some(hash))
            }
            None => PwHash(None)
        };
        let user = User {
            id: UserId::default(), 
            email, 
            pw_hash, 
            role: Role::default(),
        };
        crate::Storage::insert_user(&user).await?;
        Ok(user) 
    }
    
    pub async fn verify_password(user: &User, password: String) -> Result<()> {
        let parsed_hash = PasswordHash::new(&user.pw_hash.0.as_ref().unwrap()).unwrap();
        Pbkdf2.verify_password(password.as_bytes(), &parsed_hash).unwrap();
        Ok(())
    } 
}

impl AuthUser<UserId, Role> for User {
    fn get_id(&self) -> UserId {
        self.id
    }
    fn get_password_hash(&self) -> SecretVec<u8> {
        let value = match &self.pw_hash.0 {
            Some(hash) => hash.clone().into(),
            // generate a random one? What to do with OAuth users?
            None => vec![1, 2, 3]
        };
        SecretVec::new(value)
    }
    fn get_role(&self) -> Option<Role> {
        Some(self.role)
    }
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UserId(pub u64);
impl Default for UserId {
    fn default() -> Self {
        Self(NEW_USER_ID.fetch_add(1, Ordering::SeqCst))
    }
}
impl From<u64> for UserId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl Into<u64> for UserId {
    fn into(self) -> u64 {
        self.0
    }
}
impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PwHash(pub Option<String>);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Role {
    Default,
    Admin,
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Admin" | "admin" => Self::Admin,
            _ => Self::Default,
        }
    }
}

impl Into<String> for Role {
    fn into(self) -> String {
        match self {
            Self::Admin => "Admin".to_owned(),
            Self::Default => "Default".to_owned(),
        }
    }
}

impl Default for Role {
    fn default() -> Self {
        Role::Default
    }
}
