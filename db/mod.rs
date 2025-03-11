use crate::*;

mod storage;
pub use storage::*;

mod key;
pub use key::IntoSqlKey;

use gluesql_core::{ast_builder::Build as BuildSQL, prelude::Glue};
use std::sync::{
    atomic::{AtomicBool, AtomicU64},
    mpsc::{Receiver, SyncSender},
};

/// re-export of GlueSQL core AST builder and other utils
pub mod sql {
    pub use gluesql_core::ast::*;
    pub use gluesql_core::ast_builder::*;
    pub use gluesql_core::data::{Key, Value};
    pub use gluesql_core::executor::Payload;
    pub use gluesql_core::store::DataRow;
    pub use ordered_float::OrderedFloat;
}
pub use prest_db_macro::Storage;

type Returner = async_oneshot_channel::Sender<Result<Payload>>;
pub(crate) type DbReadMessage = (Query, Returner);
pub(crate) type DbWriteMessage = (Transaction, Returner);

pub(crate) const DB_DIRECTORY_NAME: &str = "db";

pub(crate) static TX_ID: AtomicU64 = AtomicU64::new(0);
pub(crate) static TX_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

state!(DB: Db = { Db::init() });

pub struct Db {
    pub(crate) read: SyncSender<DbReadMessage>,
    pub(crate) write: SyncSender<DbWriteMessage>,
    // shutdown: Sender ? + fn shutdown() -> Result ?
    pub(crate) internal_schemas: Arc<Vec<StructSchema>>,
    pub(crate) custom_schemas: Arc<std::sync::RwLock<Vec<StructSchema>>>,
    pub(crate) handles: Arc<Vec<std::thread::JoinHandle<Result>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Query {
    SqlStatement(sql::Statement),
    SqlString(String),
    GetByPKey {
        name: &'static str,
        pkey: sql::Key,
    },
    PKRange {
        name: &'static str,
        pkey_min: sql::Key,
        pkey_max: sql::Key,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transaction {
    SqlStatement(sql::Statement),
    SqlString(String),
    Insert {
        name: &'static str,
        key: sql::Key,
        row: Vec<sql::Value>,
    },
    Save {
        name: &'static str,
        key: sql::Key,
        row: Vec<sql::Value>,
    },
    UpdateField {
        name: &'static str,
        key: sql::Key,
        column: usize,
        value: sql::Value,
    },
    Delete {
        name: &'static str,
        key: sql::Key,
    },
    #[cfg(feature = "experimental")]
    Nuke,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Payload {
    Success,
    Rows(Vec<Vec<sql::Value>>),
    Affected(usize),
    Sql(sql::Payload),
}

impl From<sql::Payload> for Payload {
    fn from(value: sql::Payload) -> Self {
        match value {
            sql::Payload::Select { rows, .. } => Self::Rows(rows),
            sql::Payload::Insert(usize) => Self::Affected(usize),
            sql::Payload::Delete(usize) => Self::Affected(usize),
            sql::Payload::Update(usize) => Self::Affected(usize),
            other => Self::Sql(other),
        }
    }
}

impl Db {
    pub async fn read(&self, query: Query) -> Result<Payload> {
        let (returner, result) = async_oneshot_channel::oneshot::<Result<Payload>>();
        self.read.send((query, returner));
        result.recv().await.ok_or(e!("missing db return"))?
    }
    pub async fn write(&self, tx: Transaction) -> Result<Payload> {
        let (returner, result) = async_oneshot_channel::oneshot::<Result<Payload>>();
        self.write.send((tx, returner));
        result.recv().await.ok_or(e!("missing db return"))?
    }

    pub async fn read_sql(&self, sql: &str) -> Result<Payload> {
        let (returner, result) = async_oneshot_channel::oneshot::<Result<Payload>>();
        self.read.send((Query::SqlString(sql.to_owned()), returner));
        result.recv().await.ok_or(e!("missing db return"))?
    }

    pub async fn read_sql_rows<T: Storage>(&self, sql: &str) -> Result<Vec<T>> {
        let (returner, result) = async_oneshot_channel::oneshot::<Result<Payload>>();
        self.read.send((Query::SqlString(sql.to_owned()), returner));
        match result.recv().await.ok_or(e!("missing value"))?? {
            Payload::Rows(rows) => rows.into_iter().map(T::from_row).collect(),
            p => Err(e!("Got {p:?} instead of rows")),
        }
    }

    pub async fn write_sql(&self, sql: &str) -> Result<Payload> {
        let (returner, result) = async_oneshot_channel::oneshot::<Result<Payload>>();
        self.write
            .send((Transaction::SqlString(sql.to_owned()), returner));
        result.recv().await.ok_or(e!("missing db return"))?
    }

    #[cfg(feature = "experimental")]
    pub async fn nuke(&self) -> Result<Payload> {
        let (returner, result) = async_oneshot_channel::oneshot::<Result<Payload>>();
        warn!("nuking the database");
        self.write.send((Transaction::Nuke, returner));
        result.recv().await.ok_or(e!("missing db return"))?
    }

    pub fn _register_schema(&self, schema: StructSchema) {
        self.custom_schemas.write().unwrap().push(schema);
    }
    pub(crate) fn custom_schemas(&self) -> Vec<StructSchema> {
        self.custom_schemas.read().unwrap().clone()
    }
    pub(crate) fn fetch_glue_schema(&self, table_name: &str) -> Option<gluesql_core::data::Schema> {
        if let Some(schema) = self
            .internal_schemas
            .iter()
            .find(|s| s.name() == table_name)
        {
            Some(into_glue_schema(*schema))
        } else if let Some(schema) = self
            .custom_schemas
            .read()
            .unwrap()
            .iter()
            .find(|s| s.name() == table_name)
        {
            Some(into_glue_schema(*schema))
        } else {
            None
        }
    }
    pub(crate) fn fetch_all_glue_schemas(&self) -> Vec<gluesql_core::data::Schema> {
        let mut schemas = self
            .internal_schemas
            .iter()
            .map(|s| into_glue_schema(*s))
            .collect::<Vec<_>>();
        schemas.extend(
            self.custom_schemas
                .read()
                .unwrap()
                .iter()
                .map(|s| into_glue_schema(*s)),
        );
        schemas
    }
    pub async fn migrate(&self) -> Result {
        let mut all_tables = (*self.internal_schemas).clone();
        all_tables.extend(self.custom_schemas().into_iter());
        Ok(())
    }
    pub fn shutdown(&self) -> Result {
        // TODO: stop receiving on the write thread and flush?
        OK
    }
}

/// Simplified interface for gluesql AST queries to run in the [`DB`]
#[async_trait]
pub trait SqlExecutable: BuildSQL + Sized {
    async fn read(self) -> Result<Payload> {
        Ok(DB.read(Query::SqlStatement(self.build()?)).await?)
    }
    async fn write(self) -> Result<Payload> {
        Ok(DB.write(Transaction::SqlStatement(self.build()?)).await?)
    }
    async fn rows<T: storage::Storage>(self) -> Result<Vec<T>> {
        match self.read().await {
            Ok(Payload::Rows(rows)) => rows.into_iter().map(T::from_row).collect(),
            Ok(p) => return Err(e!("rows method used on query which returned: {:?}", p)),
            Err(e) => Err(e),
        }
    }
}
impl<T> SqlExecutable for T where T: BuildSQL + Send + Sized {}
