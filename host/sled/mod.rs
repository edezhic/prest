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
    super::SYSTEM_INFO,
    crate::{warn, Arc, Result, RwLock},
    error::{err_into, tx_err_into},
    gluesql::core::{
        data::Schema,
        error::{Error as GlueError, Result as GlueResult},
        store::{CustomFunction, CustomFunctionMut, Metadata},
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

#[derive(Clone, Debug)]
pub struct SharedSledStorage {
    #[allow(private_interfaces)]
    pub state: Arc<RWSledStorage>, // Combined Mutex for state and Notify for signaling
}

/// Lock and Notify
#[derive(Debug)]
pub(crate) struct RWSledStorage {
    pub db: RwLock<SledStorage>,
    pub in_progress: AtomicBool,
    notify: Notify,
}

#[derive(Debug, Clone)]
pub(crate) struct SledStorage {
    pub tree: Db,
    pub id_offset: u64,
    pub state: State,
    /// transaction timeout in milliseconds
    pub tx_timeout: Option<u128>,
}

#[derive(Debug, Clone)]
pub(crate) enum State {
    Idle,
    Transaction {
        txid: u64,
        created_at: u128,
        autocommit: bool,
    },
}

type ExportData<T> = (u64, Vec<(Vec<u8>, Vec<u8>, T)>);

impl SharedSledStorage {
    pub fn new(db_path: std::path::PathBuf) -> crate::Result<Self> {
        let total_ram_mbs = SYSTEM_INFO.ram;
        let sled_config = sled::Config::default()
            .path(db_path)
            // use up to 20% ram for cache
            .cache_capacity(total_ram_mbs.div_ceil(5));

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

    #[allow(dead_code)]
    pub fn export(&self) -> Result<ExportData<impl Iterator<Item = Vec<Vec<u8>>>>> {
        let storage = self.state.db.blocking_write();
        let id_offset = storage.id_offset + storage.tree.generate_id()?;
        let data = storage.tree.export();

        Ok((id_offset, data))
    }

    #[allow(dead_code)]
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

pub(crate) fn fetch_schema(
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
