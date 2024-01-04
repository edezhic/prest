use crate::*;
use tower_sessions::{
    session::{Id, Record},
    session_store::{Result, Error}, SessionStore,
};
use axum_login::{AuthUser, AuthnBackend, UserId};
pub use tower_sessions::Session;

#[derive(Table, Clone, Debug)]
pub struct User {
    id: Uuid,
    username: Option<String>,
    email: Option<String>,
    password_hash: Vec<u8>,
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.password_hash
    }
}

#[derive(Clone)]
pub struct Credentials {
    user_id: Uuid,
}

#[async_trait]
impl AuthnBackend for Db {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        Credentials { user_id }: Self::Credentials,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        todo!("{user_id}")
        //User::find_by_key(user_id)
        //Ok(self.users.get(&user_id).cloned())
    }

    async fn get_user(
        &self,
        user_id: &UserId<Self>,
    ) -> std::result::Result<Option<Self::User>, Self::Error> {
        todo!("{user_id}")
        //Ok(self.users.get(user_id).cloned())
    }
}

#[derive(Table, Debug)]
pub struct SessionRow {
    pub id: Uuid,
    pub record: String,
}

#[async_trait]
impl SessionStore for Db {
    async fn save(&self, record: &Record) -> Result<()> {
        let id = record.id.0;
        let record = match serde_json::to_string(record) {
            Ok(s) => s,
            Err(e) => return Err(Error::Decode(format!("{e}"))),
        };
        match (SessionRow { id, record }).save() {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Backend(format!("Session save error: {e}"))),
        }
    }

    async fn load(&self, session_id: &Id) -> Result<Option<Record>> {
        let search = SessionRow::find_by_key(&session_id.0);
        let Some(session_row) = search else {
            return Ok(None);
        };
        match serde_json::from_str(&session_row.record) {
            Ok(record) => Ok(Some(record)),
            Err(e) => Err(Error::Backend(format!("Session load error: {e}"))),
        }
    }

    async fn delete(&self, session_id: &Id) -> Result<()> {
        match SessionRow::delete_by_key(&session_id.0) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Backend(format!("Session deletion error: {e}")))
        }
    }
}
