// Copied from https://github.com/kanekoshoyu/gluesql_shared_sled_storage
use crate::*;

use async_trait::async_trait;
use gluesql::core::ast::{ColumnDef, IndexOperator, OrderByExpr};
use gluesql::core::data::{Key, Schema, Value};
use gluesql::core::error::{Error as GlueError, Result as GlueResult};
use gluesql::core::store::{
    AlterTable, CustomFunction, CustomFunctionMut, DataRow, Index, IndexMut, Metadata, RowIter,
    Store, StoreMut, Transaction,
};
pub use gluesql_sled_storage::{error::err_into, SledStorage, State};
use sled::transaction::ConflictableTransactionResult;
pub use sled::*;
use std::mem::replace;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use stream::iter;
use tokio::sync::{Notify, RwLock};
use tracing::warn;

use lock::release;

/// Lock and Notify
#[derive(Debug)]
struct StorageInner {
    pub db: RwLock<SledStorage>,
    in_progress: AtomicBool,
    notify: Notify,
}

#[derive(Clone, Debug)]
pub struct SharedSledStorage {
    #[allow(private_interfaces)]
    pub state: Arc<StorageInner>, // Combined Mutex for state and Notify for signaling
}

impl SharedSledStorage {
    pub fn new(sled_config: Config) -> crate::Result<Self> {
        let mut database = gluesql_sled_storage::SledStorage::try_from(sled_config)?;

        match replace(&mut database.state, State::Idle) {
            State::Idle => {}
            ref tx @ State::Transaction { txid, .. } => {
                warn!("recovering from unfinished transaction: {:?}", tx);
                match database.tree.transaction(
                    |tx| -> ConflictableTransactionResult<crate::Result<()>> {
                        Ok(release(&tx, txid))
                    },
                ) {
                    Err(err) => {
                        warn!("error recovering from unfinished transaction: {:?}", err);
                    }
                    Ok(Err(err)) => {
                        warn!("error recovering from unfinished transaction: {:?}", err);
                    }
                    Ok(Ok(_)) => {
                        warn!("recovered from unfinished transaction");
                    }
                }
            }
        }

        let this = SharedSledStorage {
            state: Arc::new(StorageInner {
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

    /// Copy of gluesql_sled_storage::SledStorage.scan_data() because wrapping it messes with lifetimes
    async fn scan_inner_data(&self, table_name: &str) -> GlueResult<RowIter> {
        let store = self.state.db.read().await;
        let (txid, created_at) = match store.state {
            State::Transaction {
                txid, created_at, ..
            } => (txid, created_at),
            State::Idle => {
                return Err(GlueError::StorageMsg(
                    "conflict - scan_data failed, lock does not exist".to_owned(),
                ));
            }
        };
        let lock_txid =
            gluesql_sled_storage::lock::fetch(&store.tree, txid, created_at, store.tx_timeout)?;

        let prefix = gluesql_sled_storage::key::data_prefix(table_name);
        let prefix_len = prefix.len();
        let result_set = store
            .tree
            .scan_prefix(prefix.as_bytes())
            .map(move |item| {
                let (key, value) = item.map_err(err_into)?;
                let key = key.subslice(prefix_len, key.len() - prefix_len).to_vec();
                let snapshot: gluesql_sled_storage::snapshot::Snapshot<DataRow> =
                    bincode::deserialize(&value).map_err(err_into)?;
                let row = snapshot.extract(txid, lock_txid);
                let item = row.map(|row| (Key::Bytea(key), row));

                Ok(item)
            })
            .filter_map(|item| item.transpose());

        Ok(Box::pin(iter(result_set)))
    }
}

#[async_trait(?Send)]
impl AlterTable for SharedSledStorage {
    async fn rename_schema(&mut self, table_name: &str, new_table_name: &str) -> GlueResult<()> {
        let mut database = self.state.db.write().await;
        database.rename_schema(table_name, new_table_name).await
    }
    async fn rename_column(
        &mut self,
        table_name: &str,
        old_column_name: &str,
        new_column_name: &str,
    ) -> GlueResult<()> {
        let mut database = self.state.db.write().await;
        database
            .rename_column(table_name, old_column_name, new_column_name)
            .await
    }
    async fn add_column(&mut self, table_name: &str, column_def: &ColumnDef) -> GlueResult<()> {
        let mut database = self.state.db.write().await;
        database.add_column(table_name, column_def).await
    }
    async fn drop_column(
        &mut self,
        table_name: &str,
        column_name: &str,
        if_exists: bool,
    ) -> GlueResult<()> {
        let mut database = self.state.db.write().await;
        database
            .drop_column(table_name, column_name, if_exists)
            .await
    }
}
#[async_trait(?Send)]
impl Transaction for SharedSledStorage {
    async fn begin(&mut self, autocommit: bool) -> GlueResult<bool> {
        self.open_transaction().await?;
        self.state.db.write().await.begin(autocommit).await
    }
    async fn rollback(&mut self) -> GlueResult<()> {
        self.state.db.write().await.rollback().await?;
        self.close_transaction().await;
        Ok(())
    }
    async fn commit(&mut self) -> GlueResult<()> {
        self.state.db.write().await.commit().await?;
        self.close_transaction().await;
        Ok(())
    }
}
/// By implementing `Store` trait, you can run `SELECT` query.
#[async_trait(?Send)]
impl Store for SharedSledStorage {
    async fn fetch_schema(&self, table_name: &str) -> GlueResult<Option<Schema>> {
        self.state.db.read().await.fetch_schema(table_name).await
    }
    async fn fetch_all_schemas(&self) -> GlueResult<Vec<Schema>> {
        self.state.db.read().await.fetch_all_schemas().await
    }

    async fn fetch_data(&self, table_name: &str, key: &Key) -> GlueResult<Option<DataRow>> {
        self.state.db.read().await.fetch_data(table_name, key).await
    }

    async fn scan_data(&self, _table_name: &str) -> GlueResult<RowIter> {
        self.scan_inner_data(&_table_name).await
    }
}

#[async_trait(?Send)]
impl StoreMut for SharedSledStorage {
    async fn insert_schema(&mut self, schema: &Schema) -> GlueResult<()> {
        self.state.db.write().await.insert_schema(schema).await
    }

    async fn delete_schema(&mut self, table_name: &str) -> GlueResult<()> {
        self.state.db.write().await.delete_schema(table_name).await
    }

    async fn append_data(&mut self, table_name: &str, rows: Vec<DataRow>) -> GlueResult<()> {
        self.state
            .db
            .write()
            .await
            .append_data(table_name, rows)
            .await
    }

    async fn insert_data(&mut self, table_name: &str, rows: Vec<(Key, DataRow)>) -> GlueResult<()> {
        self.state
            .db
            .write()
            .await
            .insert_data(table_name, rows)
            .await
    }

    async fn delete_data(&mut self, table_name: &str, keys: Vec<Key>) -> GlueResult<()> {
        self.state
            .db
            .write()
            .await
            .delete_data(table_name, keys)
            .await
    }
}

#[async_trait(?Send)]
impl Index for SharedSledStorage {
    async fn scan_indexed_data(
        &self,
        _table_name: &str,
        _index_name: &str,
        _asc: Option<bool>,
        _cmp_value: Option<(&IndexOperator, Value)>,
    ) -> GlueResult<RowIter> {
        unimplemented!("scan_indexed_data is not implemented because 'cannot return value referencing temporary value: returns a value referencing data owned by the current function'");
        // self.state
        //     .db
        //     .read()
        //     .await
        //     .scan_indexed_data(table_name, index_name, asc, cmp_value)
        //     .await
    }
}
#[async_trait(?Send)]
impl IndexMut for SharedSledStorage {
    async fn create_index(
        &mut self,
        table_name: &str,
        index_name: &str,
        column: &OrderByExpr,
    ) -> GlueResult<()> {
        self.state
            .db
            .write()
            .await
            .create_index(table_name, index_name, column)
            .await
    }
    async fn drop_index(&mut self, table_name: &str, index_name: &str) -> GlueResult<()> {
        self.state
            .db
            .write()
            .await
            .drop_index(table_name, index_name)
            .await
    }
}
impl Metadata for SharedSledStorage {}
impl CustomFunction for SharedSledStorage {}
impl CustomFunctionMut for SharedSledStorage {}
impl Drop for StorageInner {
    fn drop(&mut self) {
        // if there is an active transaction, rollback
        if self.in_progress.load(Ordering::Acquire) {
            if let Err(err) =
                futures::executor::block_on(async { self.db.write().await.rollback().await })
            {
                warn!("error rolling back transaction: {:?}", err);
            }
        }
    }
}

mod lock {
    use crate::Result;

    use {
        gluesql::core::error::Error,
        serde::{Deserialize, Serialize},
        sled::{
            transaction::{ConflictableTransactionError, TransactionalTree},
            Db,
        },
    };

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TxData {
        pub txid: u64,
        pub alive: bool,
        pub created_at: u128,
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Lock {
        pub lock_txid: Option<u64>,
        pub lock_created_at: u128,
        pub gc_txid: Option<u64>,
        // TODO: support serializable transaction isolation level
        // - prev_done_at: u128,
    }

    pub fn get_txdata_key(txid: u64) -> Vec<u8> {
        "tx_data/"
            .to_owned()
            .into_bytes()
            .into_iter()
            .chain(txid.to_be_bytes().iter().copied())
            .collect::<Vec<_>>()
    }
    #[allow(dead_code)]
    pub fn unregister(tree: &Db, txid: u64) -> Result<()> {
        let key = get_txdata_key(txid);
        let mut tx_data: TxData = tree
            .get(&key)?
            .ok_or_else(|| Error::StorageMsg("conflict - lock does not exist".to_owned()))
            .map(|tx_data| bincode::deserialize(&tx_data))??;

        tx_data.alive = false;

        bincode::serialize(&tx_data).map(|tx_data| tree.insert(key, tx_data))??;

        Ok(())
    }

    pub fn release(tree: &TransactionalTree, txid: u64) -> Result<()> {
        let Lock {
            gc_txid, lock_txid, ..
        } = tree
            .get("lock/")?
            .map(|l| bincode::deserialize(&l))
            .transpose()
            .map_err(ConflictableTransactionError::Abort)?
            .unwrap_or_default();

        if Some(txid) == lock_txid {
            let lock = Lock {
                lock_txid: None,
                lock_created_at: 0,
                gc_txid,
            };

            bincode::serialize(&lock)
                .map_err(ConflictableTransactionError::Abort)
                .map(|lock| tree.insert("lock/", lock))??;
        }

        let key = get_txdata_key(txid);
        let tx_data: Option<TxData> = tree
            .get(&key)?
            .map(|tx_data| bincode::deserialize(&tx_data))
            .transpose()
            .map_err(ConflictableTransactionError::Abort)?;

        let mut tx_data = match tx_data {
            Some(tx_data) => tx_data,
            None => {
                return Ok(());
            }
        };

        tx_data.alive = false;

        bincode::serialize(&tx_data)
            .map_err(ConflictableTransactionError::Abort)
            .map(|tx_data| tree.insert(key, tx_data))??;

        Ok(())
    }
}
