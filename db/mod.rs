mod gluesql_traits;

mod table;
pub use table::*;

use crate::*;

use gluesql::{
    core::{ast_builder::Build as BuildSQL, store::Transaction},
    gluesql_shared_memory_storage::SharedMemoryStorage as MemoryStorage,
    prelude::Glue,
};

/// re-export of GlueSQL core AST builder and other utils
pub mod sql {
    pub use gluesql::core::ast::*;
    pub use gluesql::core::ast_builder::*;
    pub use gluesql::core::data::Value;
    pub use gluesql::core::executor::Payload;
}
pub use prest_db_macro::Table;

pub(crate) const DB_DIRECTORY_NAME: &str = "db";

/// Embedded database
pub struct Db {
    storage: DbStorage,
    internal_schemas: Arc<Vec<TableSchema>>,
    custom_schemas: Arc<std::sync::RwLock<Vec<TableSchema>>>,
}

// Container for the [`Db`]
state!(DB: Db = {
    #[cfg(host)] {
        let storage = if APP_CONFIG.persistent {
            let mut db_path = APP_CONFIG.data_dir.clone();
            db_path.push(DB_DIRECTORY_NAME);
            let storage = PersistentStorage::new(db_path).expect("Database storage should initialize");
            Persistent(storage)
        } else {
            Memory(MemoryStorage::default())
        };

        use crate::host::analytics::RouteStat;
        #[allow(unused_mut)]
        let mut internal_schemas = vec![ScheduledJobRecord::schema(), RouteStat::schema(), SystemStat::schema()];
        #[cfg(feature = "auth")] {
            internal_schemas.push(crate::host::auth::SessionRow::schema());
            internal_schemas.push(crate::host::auth::User::schema());
        }

        Db {
            storage,
            internal_schemas: Arc::new(internal_schemas),
            custom_schemas: Default::default(),
        }
    }
    #[cfg(sw)] {
        Db {
            storage: Memory(MemoryStorage::default()),
            internal_schemas: Arc::new(vec![]),
            custom_schemas: Default::default(),
        }
    }
});

impl Db {
    pub(crate) fn storage(&self) -> DbStorage {
        self.storage.clone()
    }
    pub fn _register_table(&self, schema: TableSchema) {
        self.custom_schemas.write().unwrap().push(schema);
    }
    pub(crate) fn custom_tables(&self) -> Vec<TableSchema> {
        self.custom_schemas.read().unwrap().clone()
    }
    pub async fn migrate(&self) -> Result {
        let mut all_tables = (*self.internal_schemas).clone();
        all_tables.extend(self.custom_tables().into_iter());

        // let names: Vec<&'static str> = all_tables.iter().map(|t| t.name()).collect();

        // let DbStorage::Persistent(PersistentStorage { state }) = &DB.storage() else {
        //     return Ok(());
        // };

        // for name in names {
        //     let tree = state
        //         .db
        //         .read()
        //         .await
        //         .tree
        //         .transaction(move |tx_tree| sled::fetch_schema(tx_tree, name));
        //     info!("{:?}", tree);
        // }

        // naive migration
        for table in all_tables {
            Self::create_if_not_exists(table).await?;
        }

        Ok(())
    }

    async fn create_if_not_exists(table: TableSchema) -> Result {
        let mut stmt = sql::table(table.name()).create_table_if_not_exists();
        for ColumnSchema {
            name,
            sql_type,
            unique,
            pkey,
            list,
            optional,
            ..
        } in table.columns()
        {
            let col = if *list {
                format!("{name} LIST")
            } else {
                let unique = if !*pkey && *unique { " UNIQUE" } else { "" };
                let pkey = if *pkey { " PRIMARY KEY" } else { "" };
                let optional = if *optional { "" } else { " NOT NULL" };
                format!("{name} {sql_type}{pkey}{unique}{optional}")
            };
            stmt = stmt.add_column(col.as_str());
        }
        stmt.exec().await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
#[doc(hidden)]
pub enum DbStorage {
    Memory(MemoryStorage),
    Persistent(PersistentStorage),
}
use DbStorage::*;

/// Interface for the [`DB`]
#[doc(hidden)]
#[async_trait]
pub trait DbAccess {
    async fn query(&self, query: &str) -> Result<Vec<sql::Payload>>;
    async fn flush(&self);
}

#[async_trait]
impl DbAccess for Lazy<Db> {
    async fn query(&self, query: &str) -> Result<Vec<sql::Payload>> {
        // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1265
        let payload = await_blocking(async move { Glue::new(DB.storage()).execute(query).await })?;
        Ok(payload)
    }

    async fn flush(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        match DB.storage() {
            Memory(_) => (),
            Persistent(mut store) => {
                // if there is an active transaction, rollback
                if store
                    .state
                    .in_progress
                    .load(std::sync::atomic::Ordering::Acquire)
                {
                    if let Err(err) = crate::host::await_blocking(async { store.rollback().await })
                    {
                        warn!(target: "db", "error rolling back transaction: {:?}", err);
                    }
                }
                if let Err(e) = store.flush() {
                    error!(target:"db", "flushing DB failed with: {e}");
                }
            }
        }
    }
}

/// Simplified interface for queries to run with [`DB`]
#[async_trait]
pub trait DbExecutable {
    async fn exec(self) -> Result<sql::Payload>;
    async fn rows(self) -> Result<Vec<Vec<sql::Value>>>;
    async fn values<T: Table>(self) -> Result<Vec<T>>;
}

#[async_trait]
impl<Q: BuildSQL + Send> DbExecutable for Q {
    async fn exec(self) -> Result<sql::Payload> {
        let statement = self.build()?;
        // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1265
        let payload =
            await_blocking(async move { Glue::new(DB.storage()).execute_stmt(&statement).await })?;
        Ok(payload)
    }

    async fn rows(self) -> Result<Vec<Vec<sql::Value>>> {
        match self.exec().await {
            Ok(sql::Payload::Select { rows, .. }) => Ok(rows),
            Ok(p) => {
                return Err(e!(
                    "rows method used on non-select query which returned: {:?}",
                    p
                ))
            }
            Err(e) => return Err(e!("query execution failed: {e:?}")),
        }
    }

    async fn values<T: Table>(self) -> Result<Vec<T>> {
        let rows = self.rows().await?;
        rows.into_iter().map(T::from_row).collect()
    }
}
