use gluesql_core::{
    ast_builder::Build,
    prelude::Glue,
    store::{Store, StoreMut},
};

use crate::*;

mod alter_table;
mod index;
mod index_mut;
mod index_sync;
mod snapshot;
mod store;
mod store_mut;
mod transaction;

use std::{
    borrow::BorrowMut,
    cell::RefCell,
    sync::mpsc::{RecvError, TryRecvError},
};
use {
    self::snapshot::Snapshot,
    gluesql_core::store::{CustomFunction, CustomFunctionMut, Metadata},
    sled::InlineArray,
    std::sync::atomic::{AtomicBool, AtomicU64, Ordering},
    tokio::sync::Notify,
};

pub(crate) const WRITE_STATE_KEY: &[u8] = b"state/write";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct WriteState {
    pub tx_id: u64,
    pub in_progress: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct DbConn<'a> {
    pub state: WriteState,
    pub readonly: bool,
    pub tree: &'a sled::Db,
}

impl Db {
    pub(crate) fn init() -> Db {
        let mut db_path = APP_CONFIG.data_dir.clone();
        db_path.push(DB_DIRECTORY_NAME);
        if !APP_CONFIG.persistent {
            todo!("global statics aren't dropped => sled's temporary key doesn't work (doesn't Drop), need another way to deal with tmp storages")
        }

        let storage = sled::Config::default()
            .path(db_path.clone())
            .cache_capacity_bytes(256 * 1024 * 1024)
            .open::<1024>()
            .expect(&format!("DB path ({db_path:?}) should be available"));

        if let Some(state_bytes) = storage.get(WRITE_STATE_KEY).unwrap() {
            let state: WriteState = bitcode::deserialize(&state_bytes).unwrap();
            if state.in_progress {
                crate::warn!("detected unfinished write, rolling it back");
                TX_ID.store(state.tx_id, Ordering::SeqCst);
                // TODO
                // DbConn {
                //     lock: Some(lock.clone()),
                //     tree: storage.clone(),
                //     state,
                // }
                // .rollback_self()?;
            }
        }

        let (write_sender, writes) = std::sync::mpsc::sync_channel::<DbWriteMessage>(10);
        let (read_sender, reads) = std::sync::mpsc::sync_channel::<DbReadMessage>(100);

        let write_storage = storage.clone();
        let read_storage = storage.clone();

        let write_thread = std::thread::Builder::new()
            .name("DB writer".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();
                loop {
                    let Ok((tx, returner)) = writes.recv() else {
                        panic!("DB is disconnected (write senders are dropped)")
                    };
                    let result = rt.block_on(write(&write_storage, tx));
                    if let Err(e) = returner.send(result) {
                        warn!("failed to return write result: {e:?}");
                    }
                }
            })
            .expect("DB writer thread should spawn");
        let read_thread = std::thread::Builder::new()
            .name("DB reader".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();
                loop {
                    let Ok(msg) = reads.recv() else {
                        panic!("DB is disconnected (read senders are dropped)")
                    };
                    rt.block_on(read(&read_storage, msg));
                }
            })
            .expect("DB reader thread should spawn");

        use crate::host::analytics::RouteStat;
        #[allow(unused_mut)]
        let mut internal_schemas = vec![
            ScheduledJobRecord::schema(),
            RouteStat::schema(),
            SystemStat::schema(),
        ];
        #[cfg(feature = "auth")]
        {
            internal_schemas.push(crate::host::auth::SessionRow::schema());
            internal_schemas.push(crate::host::auth::User::schema());
        }

        Db {
            read: read_sender,
            write: write_sender,
            internal_schemas: Arc::new(internal_schemas),
            custom_schemas: Default::default(),
            handles: Arc::new(vec![write_thread, read_thread]),
        }
    }
}

async fn write<'a>(tree: &'a sled::Db, query: Transaction) -> Result<Payload> {
    let tx_id = TX_ID.fetch_add(1, Ordering::Relaxed);
    TX_IN_PROGRESS.store(true, Ordering::SeqCst);

    let mut conn = DbConn {
        state: WriteState {
            tx_id,
            in_progress: true,
        },
        readonly: false,
        tree,
    };

    let result = match query {
        Transaction::SqlString(sql) => {
            Ok(Glue::new(conn).execute(sql).await?.pop().unwrap().into())
        }
        Transaction::SqlStatement(stmt) => {
            let planned = gluesql_core::plan::plan(&conn, stmt).await?;
            Ok(Glue::new(conn).execute_stmt(&planned).await?.into())
        }
        Transaction::Insert { name, key, row } => {
            if let Some(_) = conn.fetch_data(name, &key).await? {
                return Err(e!("duplicate data insertion for {key:?}"));
            }
            conn.insert_data(name, vec![(key, sql::DataRow::Vec(row))])
                .await?;
            Ok(Payload::Success)
        }
        Transaction::Save { name, key, row } => {
            conn.insert_data(name, vec![(key, sql::DataRow::Vec(row))])
                .await?;
            Ok(Payload::Success)
        }
        Transaction::UpdateField {
            name,
            key,
            column,
            value,
        } => {
            conn.update_cell(name, key, column, value).await?;
            Ok(Payload::Success)
        }
        Transaction::Delete { name, key } => {
            if let None = conn.fetch_data(name, &key).await? {
                return Err(e!("deleting non-existent value {key:?}"));
            }
            conn.delete_data(name, vec![key]).await?;
            Ok(Payload::Success)
        }
        // Transaction::Flush => {
        //     conn.tree.flush()?;
        //     Ok(Payload::Success)
        // }
        #[cfg(feature = "experimental")]
        Transaction::Nuke => {
            conn.tree.clear()?;
            conn.tree.flush()?;
            Ok(Payload::Success)
        }
    };
    TX_IN_PROGRESS.store(false, Ordering::SeqCst);
    result
}

async fn read<'a>(tree: &'a sled::Db, (query, returner): DbReadMessage) -> Result {
    let tx_id = TX_ID.load(Ordering::Relaxed);
    let in_progress = TX_IN_PROGRESS.load(Ordering::Relaxed);

    let mut conn = DbConn {
        state: WriteState { tx_id, in_progress },
        readonly: false,
        tree,
    };

    let result = match query {
        Query::SqlString(sql) => Ok(Glue::new(conn).execute(sql).await?.pop().unwrap().into()),
        Query::SqlStatement(stmt) => {
            let planned = gluesql_core::plan::plan(&conn, stmt).await?;
            Ok(Glue::new(conn).execute_stmt(&planned).await?.into())
        }
        Query::GetByPKey { name, pkey } => {
            let rows = match conn.fetch_data(name, &pkey).await? {
                Some(row) => match row {
                    sql::DataRow::Vec(vec) => vec![vec],
                    sql::DataRow::Map(vec) => unimplemented!(),
                },
                None => vec![],
            };
            Ok(Payload::Rows(rows))
        }
        Query::PKRange {
            name,
            pkey_min,
            pkey_max,
        } => {
            let rows = conn.pk_range(name, pkey_min, pkey_max).await?;
            Ok(Payload::Rows(rows))
        }
    };
    returner.send(result);
    OK
}

impl<'a> Metadata for DbConn<'a> {}
impl<'a> CustomFunction for DbConn<'a> {}
impl<'a> CustomFunctionMut for DbConn<'a> {}

pub fn data_prefix(table_name: &str) -> String {
    format!("/{table_name}/")
}

pub fn sled_key(
    table_name: &str,
    key: sql::Key,
) -> Result<InlineArray, gluesql_core::error::Error> {
    let key = key
        .to_cmp_be_bytes()
        .map(|key| data_prefix(table_name).into_bytes().into_iter().chain(key))?;

    Ok(InlineArray::from_iter(key))
}

trait AsStorageError<T, E: std::fmt::Display> {
    fn as_storage_err(self) -> Result<T, gluesql_core::error::Error>;
}

impl<T, E: std::fmt::Display> AsStorageError<T, E> for Result<T, E> {
    fn as_storage_err(self) -> Result<T, gluesql_core::error::Error> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(gluesql_core::error::Error::StorageMsg(e.to_string())),
        }
    }
}
