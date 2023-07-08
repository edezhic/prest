use axum_login::{secrecy::SecretVec, AuthUser};

pub type UserId = u64;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
}

impl AuthUser<UserId, Role> for User {
    fn get_id(&self) -> UserId {
        self.id
    }
    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.clone().into())
    }
    fn get_role(&self) -> Option<Role> {
        Some(self.role.clone())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
