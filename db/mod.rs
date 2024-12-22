mod gluesql_traits;

mod table;
pub use table::*;

use crate::*;

use gluesql::{
    core::ast_builder::Build as BuildSQL,
    gluesql_shared_memory_storage::SharedMemoryStorage as MemoryStorage, prelude::Glue,
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
#[derive(Clone, Debug)]
pub enum Db {
    Memory(MemoryStorage),
    Persistent(PersistentStorage),
}
use Db::*;

// Container for the [`Db`]
state!(DB: Db = {
    let AppConfig {
        persistent,
        #[cfg(host)]
        data_dir,
        ..
    } = APP_CONFIG.check();

    let db = if *persistent {
        #[cfg(host)]
        {
            let mut db_path = data_dir.clone();
            db_path.push(DB_DIRECTORY_NAME);
            let storage = PersistentStorage::new(db_path).expect("Database storage should initialize");
            Persistent(storage)
        }
        #[cfg(sw)]
        {
            Persistent(MemoryStorage::default())
        }
    } else {
        Memory(MemoryStorage::default())
    };

    db
});

/// Interface for the [`DB`]
pub trait DbAccess {
    fn copy(&self) -> Db;
    fn query(&self, query: &str) -> Result<Vec<sql::Payload>>;
    fn flush(&self);
}

impl DbAccess for Lazy<Db> {
    fn copy(&self) -> Db {
        use std::ops::Deref;
        DB.deref().clone()
    }

    fn query(&self, query: &str) -> Result<Vec<sql::Payload>> {
        // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1265
        let payload = await_blocking(Glue::new(DB.copy()).execute(query))?;
        Ok(payload)
    }

    fn flush(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        match DB.copy() {
            Memory(_) => (),
            Persistent(store) => {
                if let Err(e) = store.flush() {
                    error!(target:"db", "flushing DB failed with: {e}");
                }
            }
        }
    }
}

/// Simplified interface for queries to run with [`DB`]
pub trait DbExecutable {
    fn exec(self) -> Result<sql::Payload>;
    fn rows(self) -> Result<Vec<Vec<sql::Value>>>;
    fn values<T: Table>(self) -> Result<Vec<T>>;
}

impl<Q: BuildSQL> DbExecutable for Q {
    fn exec(self) -> Result<sql::Payload> {
        let statement = self.build()?;
        // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1265
        let payload = await_blocking(Glue::new(DB.copy()).execute_stmt(&statement))?;
        Ok(payload)
    }

    fn rows(self) -> Result<Vec<Vec<sql::Value>>> {
        match self.exec() {
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

    fn values<T: Table>(self) -> Result<Vec<T>> {
        let rows = self.rows()?;
        rows.into_iter().map(T::from_row).collect()
    }
}
