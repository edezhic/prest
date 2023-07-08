use axum_login::{secrecy::SecretVec, AuthUser, memory_store::MemoryStore};
use std::{sync::Arc, collections::HashMap};
use tokio::sync::RwLock;

pub type UserId = u64;
pub type UserStore = MemoryStore<UserId, User>;

lazy_static! {
    pub static ref USER_STORE: Arc<RwLock<HashMap<UserId, User>>> = Arc::new(RwLock::new(HashMap::<UserId, User>::new()));
}

pub async fn init_user_store() -> UserStore {
    USER_STORE.write().await.insert(1, User {
        id: 1,
        email: "edezhic@gmail.com".to_owned(),
        password_hash: String::new(),
        role: Role::Admin,
    });
    MemoryStore::new(&USER_STORE)
}

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
    Visitor,
    Admin,
}

impl Default for Role {
    fn default() -> Self {
        Role::Visitor
    }
}
