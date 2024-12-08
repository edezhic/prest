mod gluesql_traits;

mod table;
pub use table::*;

use crate::*;
pub use gluesql::{
    core::ast_builder::{col, table},
    prelude::{Payload, Value as DbValue},
};
pub use prest_db_macro::Table;

use gluesql::{
    core::ast_builder::Build as BuildSQL,
    gluesql_shared_memory_storage::SharedMemoryStorage as MemoryStorage, prelude::Glue,
};

pub const DB_DIRECTORY_NAME: &str = "db";

/// Storage of the embedded database
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
            let config = sled::Config::default()
                .path(db_path.to_str().unwrap())
                .cache_capacity(100_000_000)
                .flush_every_ms(Some(100));

            let storage = PersistentStorage::try_from(config).unwrap();
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
    fn query(&self, query: &str) -> Result<Vec<Payload>>;
    fn flush(&self);
}

impl DbAccess for Lazy<Db> {
    fn copy(&self) -> Db {
        use std::ops::Deref;
        DB.deref().clone()
    }

    fn query(&self, query: &str) -> Result<Vec<Payload>> {
        // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1265
        let payload = await_fut(Glue::new(DB.copy()).execute(query))?;
        Ok(payload)
    }

    fn flush(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        match DB.copy() {
            Memory(_) => (),
            Persistent(sled) => {
                if let Err(e) = sled.tree.flush() {
                    error!(target:"db", "flushing DB failed with: {e}");
                }
            }
        }
    }
}

/// Simplified interface for queries to run with [`DB`]
pub trait DbExecutable {
    fn exec(self) -> Result<Payload>;
    fn rows(self) -> Result<Vec<Vec<DbValue>>>;
    fn values<T: Table>(self) -> Result<Vec<T>>;
}

impl<Q: BuildSQL> DbExecutable for Q {
    fn exec(self) -> Result<Payload> {
        let statement = self.build()?;
        // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1265
        let payload = await_fut(Glue::new(DB.copy()).execute_stmt(&statement))?;
        Ok(payload)
    }

    fn rows(self) -> Result<Vec<Vec<DbValue>>> {
        match self.exec() {
            Ok(Payload::Select { rows, .. }) => Ok(rows),
            Ok(p) => {
                return Err(e!(
                    "rows method used on non-select query which returned: {:?}",
                    p
                ))
            }
            Err(e) => return Err(e!("query execution failed with: {e:?}")),
        }
    }

    fn values<T: Table>(self) -> Result<Vec<T>> {
        let rows = self.rows()?;
        Ok(rows.into_iter().map(T::from_row).collect::<Vec<T>>())
    }
}
