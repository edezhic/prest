use crate::*;
use tower_sessions::{
    session::{Id, Record},
    session_store::{Error as SessionError, Result as SessionResult},
    Expiry, SessionManagerLayer, SessionStore,
};

#[derive(Storage, Debug, Serialize, Deserialize)]
pub struct SessionRow {
    pub id: i128,
    pub record: Vec<u8>,
}

#[async_trait]
impl SessionStore for Prest {
    async fn save(&self, record: &Record) -> SessionResult<()> {
        let id = record.id.0;
        let record = match bitcode::serialize(record) {
            Ok(s) => s,
            Err(e) => return Err(SessionError::Encode(format!("{e}"))),
        };
        match (SessionRow { id, record }).save().await {
            Ok(_) => Ok(()),
            Err(e) => Err(SessionError::Backend(format!("Session save error: {e}"))),
        }
    }

    async fn load(&self, session_id: &Id) -> SessionResult<Option<Record>> {
        let search = match SessionRow::get_by_pkey(session_id.0).await {
            Ok(v) => v,
            Err(e) => {
                return Err(SessionError::Backend(format!(
                    "Failed to load session: {e}"
                )))
            }
        };

        let Some(session_row) = search else {
            return Ok(None);
        };
        match bitcode::deserialize(&session_row.record) {
            Ok(record) => Ok(Some(record)),
            Err(e) => Err(SessionError::Decode(format!("Session load error: {e}"))),
        }
    }

    async fn delete(&self, session_id: &Id) -> SessionResult<()> {
        match SessionRow::delete_by_pkey(session_id.0).await {
            Ok(_) => Ok(()),
            Err(e) => Err(SessionError::Backend(format!(
                "Session deletion error: {e}"
            ))),
        }
    }
}
