// forked from gluesql sled storage with additions from https://github.com/kanekoshoyu/gluesql_shared_sled_storage
mod alter_table;
mod error;
mod gc;
mod index;
mod index_mut;
mod index_sync;
mod key;
mod lock;
mod snapshot;
mod store;
mod store_mut;
mod transaction;

use {
    self::snapshot::Snapshot,
    crate::{host::await_blocking, warn, Arc, Result, RwLock},
    error::{err_into, tx_err_into},
    gluesql::core::{
        data::Schema,
        error::{Error as GlueError, Result as GlueResult},
        store::{CustomFunction, CustomFunctionMut, Metadata, Transaction},
    },
    sled::{
        transaction::{
            ConflictableTransactionError, ConflictableTransactionResult as ConflictTxResult,
            TransactionalTree,
        },
        Db,
    },
    std::{
        mem::replace,
        sync::atomic::{AtomicBool, Ordering},
    },
    tokio::sync::Notify,
};

/// default transaction timeout : 1 minute
const DEFAULT_TX_TIMEOUT: u128 = 60 * 1000;

const SCHEMA_PREFIX: &'static str = "schema/";

#[derive(Debug, Clone)]
enum State {
    Idle,
    Transaction {
        txid: u64,
        created_at: u128,
        autocommit: bool,
    },
}

#[derive(Debug, Clone)]
struct SledStorage {
    pub tree: Db,
    pub id_offset: u64,
    pub state: State,
    /// transaction timeout in milliseconds
    pub tx_timeout: Option<u128>,
}

type ExportData<T> = (u64, Vec<(Vec<u8>, Vec<u8>, T)>);

/// Lock and Notify
#[derive(Debug)]
struct RWSledStorage {
    pub db: RwLock<SledStorage>,
    in_progress: AtomicBool,
    notify: Notify,
}

#[derive(Clone, Debug)]
pub struct SharedSledStorage {
    #[allow(private_interfaces)]
    pub state: Arc<RWSledStorage>, // Combined Mutex for state and Notify for signaling
}

impl SharedSledStorage {
    pub fn new(db_path: std::path::PathBuf) -> crate::Result<Self> {
        let sled_config = sled::Config::default()
            .path(db_path)
            .cache_capacity(100_000_000)
            .flush_every_ms(Some(100));

        let tree = sled_config.open().map_err(err_into)?;
        let id_offset = get_id_offset(&tree)?;
        let state = State::Idle;
        let tx_timeout = Some(DEFAULT_TX_TIMEOUT);

        let mut database = SledStorage {
            tree,
            id_offset,
            state,
            tx_timeout,
        };

        match replace(&mut database.state, State::Idle) {
            State::Idle => {}
            ref tx @ State::Transaction { txid, .. } => {
                warn!(target: "storage", "recovering from unfinished transaction: {:?}", tx);
                match database.tree.transaction(
                    |tx| -> ConflictTxResult<ConflictTxResult<(), GlueError>> {
                        Ok(lock::release(&tx, txid))
                    },
                ) {
                    Err(err) => {
                        warn!(target: "storage", "error recovering from unfinished transaction: {:?}", err);
                    }
                    Ok(Err(err)) => {
                        warn!(target: "storage", "error recovering from unfinished transaction: {:?}", err);
                    }
                    Ok(Ok(_)) => {
                        warn!(target: "storage", "recovered from unfinished transaction");
                    }
                }
            }
        }

        let this = SharedSledStorage {
            state: Arc::new(RWSledStorage {
                db: RwLock::new(database),
                in_progress: AtomicBool::new(false),
                notify: Notify::new(),
            }),
        };
        Ok(this)
    }
    async fn open_transaction(&self) -> GlueResult<()> {
        let state = &self.state;

        while state
            .in_progress
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
            .is_err()
        {
            // Await notification that the transaction has completed.
            state.notify.notified().await;
        }

        Ok(())
    }
    async fn close_transaction(&self) {
        // Set the transaction as not in progress and notify all waiting.
        let state = &self.state;
        state.in_progress.store(false, Ordering::Release);
        state.notify.notify_one();
    }

    pub fn flush(&self) -> sled::Result<usize> {
        self.state.db.blocking_write().tree.flush()
    }

    pub fn export(&self) -> Result<ExportData<impl Iterator<Item = Vec<Vec<u8>>>>> {
        let storage = self.state.db.blocking_write();
        let id_offset = storage.id_offset + storage.tree.generate_id()?;
        let data = storage.tree.export();

        Ok((id_offset, data))
    }

    pub fn import(&mut self, export: ExportData<impl Iterator<Item = Vec<Vec<u8>>>>) -> Result<()> {
        let mut storage = self.state.db.blocking_write();
        let (new_id_offset, data) = export;
        let old_id_offset = get_id_offset(&storage.tree)?;

        storage.tree.import(data);

        if new_id_offset > old_id_offset {
            storage
                .tree
                .insert("id_offset", &new_id_offset.to_be_bytes())?;

            storage.id_offset = new_id_offset;
        }

        Ok(())
    }
}

fn get_id_offset(tree: &Db) -> GlueResult<u64> {
    tree.get("id_offset")
        .map_err(err_into)?
        .map(|id| {
            id.as_ref()
                .try_into()
                .map_err(err_into)
                .map(u64::from_be_bytes)
        })
        .unwrap_or(Ok(0))
}

fn fetch_schema(
    tree: &TransactionalTree,
    table_name: &str,
) -> ConflictTxResult<(String, Option<Snapshot<Schema>>), GlueError> {
    let key = format!("{SCHEMA_PREFIX}{}", table_name);
    let value = tree.get(key.as_bytes())?;
    let schema_snapshot = value
        .map(|v| bincode::deserialize(&v))
        .transpose()
        .map_err(err_into)
        .map_err(ConflictableTransactionError::Abort)?;

    Ok((key, schema_snapshot))
}

impl Metadata for SharedSledStorage {}
impl CustomFunction for SharedSledStorage {}
impl CustomFunctionMut for SharedSledStorage {}
impl Drop for SharedSledStorage {
    fn drop(&mut self) {
        // if there is an active transaction, rollback
        if self.state.in_progress.load(Ordering::Acquire) {
            if let Err(err) = await_blocking(async { self.rollback().await }) {
                warn!(target: "storage", "error rolling back transaction: {:?}", err);
            }
        }
    }
}
