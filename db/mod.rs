use crate::*;
pub use gluesql::{
    core::ast_builder::{col, table},
    prelude::{Payload, Value as DbValue},
};
pub use prest_db_macro::Table;

use gluesql::{
    core::{
        ast::{ColumnDef, IndexOperator, OrderByExpr},
        ast_builder::{Build as BuildSQL, DeleteNode, InsertNode, SelectNode, UpdateNode},
        data::{CustomFunction as StructCustomFunction, Key, Schema},
        error::Error as GlueError,
        store::{
            AlterTable, CustomFunction, CustomFunctionMut, DataRow, Index, IndexMut, Metadata,
            RowIter, Store, StoreMut, Transaction,
        },
    },
    prelude::Glue,
    shared_memory_storage::SharedMemoryStorage as MemoryStorage,
};

type GResult<T> = std::result::Result<T, GlueError>;

#[derive(Clone, Debug)]
pub enum Db {
    Memory(MemoryStorage),
    Persistent(PersistentStorage),
}
use Db::*;

pub static DB: std::sync::OnceLock<Db> = std::sync::OnceLock::new();

pub trait DbAccess {
    fn init(&self);
    fn cloned(&self) -> Db;
    fn query(&self, query: &str) -> Result<Vec<Payload>>;
    fn flush(&self);
}

impl DbAccess for std::sync::OnceLock<Db> {
    fn init(&self) {
        let CrateConfig {
            project_dirs,
            persistent,
            ..
        } = CRATE_CONFIG.check();
        
        let db = if *persistent {
            let mut db_path = project_dirs.data_dir().to_path_buf();
            db_path.push("db");
            Db::Persistent(PersistentStorage::new(db_path.to_str().unwrap()).unwrap())
        } else {
            Db::Memory(MemoryStorage::new())
        };

        self.get_or_init(|| db);
    }

    fn cloned(&self) -> Db {
        DB.get()
            .expect("DB should be initialized before access")
            .clone()
    }

    fn query(&self, query: &str) -> Result<Vec<Payload>> {
        // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1265
        let payload = block_on(Glue::new(DB.cloned()).execute(query))?;
        Ok(payload)
    }

    fn flush(&self) {
        match DB.cloned() {
            Memory(_) => (),
            Persistent(sled) => {
                if let Err(e) = sled.tree.flush() {
                    tracing::error!("flushing DB failed with: {e}");
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnSchema {
    pub name: &'static str,
    pub rust_type: &'static str,
    pub glue_type: &'static str,
    pub unique: bool,
    pub key: bool,
    pub list: bool,
    pub optional: bool,
    pub custom_type: bool,
}

pub type ColumnsSchema = &'static [ColumnSchema];

#[derive(Debug, Clone)]

pub struct TableSchema {
    pub name: &'static str,
    pub columns: ColumnsSchema,
}

pub struct DbSchema(pub std::sync::RwLock<Vec<&'static dyn TableSchemaTrait>>);
impl DbSchema {
    fn init() -> Self {
        Self(std::sync::RwLock::new(vec![]))
    }
    pub fn add_table(&self, schema: &'static dyn TableSchemaTrait) {
        self.0.write().unwrap().push(schema);
    }
    pub fn tables(&self) -> Vec<&dyn TableSchemaTrait> {
        self.0.read().unwrap().clone()
    }
}
state!(DB_SCHEMA: DbSchema = { DbSchema::init() });

#[async_trait]
pub trait TableSchemaTrait: Sync {
    fn name(&self) -> &'static str;
    fn schema(&self) -> ColumnsSchema;
    fn path(&self) -> &'static str;
    fn get_all(&self) -> Vec<Vec<String>>;
    async fn save(&self, req: Request) -> Result<()>;
    async fn remove(&self, req: Request) -> Result<()>;
}

pub trait Executable {
    fn exec(self) -> Result<Payload>;
    fn rows(self) -> Result<Vec<Vec<DbValue>>>;
}

impl<Q: BuildSQL> Executable for Q {
    fn exec(self) -> Result<Payload> {
        let statement = self.build()?;
        // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1265
        let payload = block_on(Glue::new(DB.cloned()).execute_stmt(&statement))?;
        Ok(payload)
    }

    fn rows(self) -> Result<Vec<Vec<DbValue>>> {
        match self.exec() {
            Ok(Payload::Select { rows, .. }) => Ok(rows),
            Ok(p) => {
                return Err(anyhow!(
                    "rows method used on non-select query that returned: {:?}",
                    p
                )
                .into())
            }
            Err(e) => return Err(anyhow!("query execution failed with: {e:?}").into()),
        }
    }
}

pub trait Table: Sized {
    const TABLE_NAME: &'static str;
    const TABLE_SCHEMA: ColumnsSchema;
    const KEY: &'static str;
    const STRINGY_KEY: bool;
    type Key: std::fmt::Display;

    fn migrate();
    fn prepare_table();
    fn into_row(&self) -> String;
    fn from_row(row: Vec<DbValue>) -> Self;
    fn get_key(&self) -> &Self::Key;
    fn save(&self) -> Result<&Self>;

    fn key_filter(key: &Self::Key) -> String {
        match Self::STRINGY_KEY {
            true => format!("{} = '{key}'", Self::KEY),
            false => format!("{} = {key}", Self::KEY),
        }
    }

    fn from_rows(rows: Vec<Vec<DbValue>>) -> Vec<Self> {
        rows.into_iter().map(Self::from_row).collect::<Vec<Self>>()
    }

    fn select() -> SelectNode<'static> {
        table(Self::TABLE_NAME).select()
    }

    fn insert() -> InsertNode {
        table(Self::TABLE_NAME).insert()
    }

    fn delete() -> DeleteNode<'static> {
        table(Self::TABLE_NAME).delete()
    }

    fn update() -> UpdateNode<'static> {
        table(Self::TABLE_NAME).update()
    }

    fn find_all() -> Vec<Self> {
        Self::from_rows(Self::select().rows().unwrap())
    }

    fn insert_self(&self) -> Result<()> {
        Self::insert().values(vec![self.into_row()]).exec()?;
        Ok(())
    }

    fn find_by_key(key: &Self::Key) -> Option<Self> {
        let mut rows = Self::select().filter(Self::key_filter(key)).rows().unwrap();
        match rows.pop() {
            Some(row) => Some(Self::from_row(row)),
            None => None,
        }
    }

    fn update_by_key(key: &Self::Key) -> UpdateNode<'static> {
        Self::update().filter(Self::key_filter(key))
    }

    fn delete_by_key(key: &Self::Key) -> Result<()> {
        let payload = Self::delete().filter(Self::key_filter(key)).exec()?;
        match payload {
            Payload::Delete(_) => Ok(()),
            _ => return Err(anyhow!("Couldn't delete item with key = {key}").into()),
        }
    }

    fn remove(&self) -> Result<()> {
        Self::delete_by_key(&self.get_key())
    }
}

#[async_trait(?Send)]
impl Store for Db {
    async fn fetch_schema(&self, table_name: &str) -> GResult<Option<Schema>> {
        match self {
            Memory(s) => s.fetch_schema(table_name).await,
            Persistent(s) => s.fetch_schema(table_name).await,
        }
    }

    async fn fetch_all_schemas(&self) -> GResult<Vec<Schema>> {
        match self {
            Memory(s) => s.fetch_all_schemas().await,
            Persistent(s) => s.fetch_all_schemas().await,
        }
    }

    async fn fetch_data(&self, table_name: &str, key: &Key) -> GResult<Option<DataRow>> {
        match self {
            Memory(s) => s.fetch_data(table_name, key).await,
            Persistent(s) => s.fetch_data(table_name, key).await,
        }
    }

    async fn scan_data(&self, table_name: &str) -> GResult<RowIter> {
        match self {
            Memory(s) => s.scan_data(table_name).await,
            Persistent(s) => s.scan_data(table_name).await,
        }
    }
}

#[async_trait(?Send)]
impl StoreMut for Db {
    async fn insert_schema(&mut self, schema: &Schema) -> GResult<()> {
        match self {
            Memory(s) => s.insert_schema(schema).await,
            Persistent(s) => s.insert_schema(schema).await,
        }
    }

    async fn delete_schema(&mut self, table_name: &str) -> GResult<()> {
        match self {
            Memory(s) => s.delete_schema(table_name).await,
            Persistent(s) => s.delete_schema(table_name).await,
        }
    }

    async fn append_data(&mut self, table_name: &str, rows: Vec<DataRow>) -> GResult<()> {
        match self {
            Memory(s) => s.append_data(table_name, rows).await,
            Persistent(s) => s.append_data(table_name, rows).await,
        }
    }

    async fn insert_data(&mut self, table_name: &str, rows: Vec<(Key, DataRow)>) -> GResult<()> {
        match self {
            Memory(s) => s.insert_data(table_name, rows).await,
            Persistent(s) => s.insert_data(table_name, rows).await,
        }
    }

    async fn delete_data(&mut self, table_name: &str, keys: Vec<Key>) -> GResult<()> {
        match self {
            Memory(s) => s.delete_data(table_name, keys).await,
            Persistent(s) => s.delete_data(table_name, keys).await,
        }
    }
}

#[async_trait(?Send)]
impl AlterTable for Db {
    async fn rename_schema(&mut self, _table_name: &str, _new_table_name: &str) -> GResult<()> {
        match self {
            Memory(s) => s.rename_schema(_table_name, _new_table_name).await,
            Persistent(s) => s.rename_schema(_table_name, _new_table_name).await,
        }
    }

    async fn rename_column(
        &mut self,
        _table_name: &str,
        _old_column_name: &str,
        _new_column_name: &str,
    ) -> GResult<()> {
        match self {
            Memory(s) => {
                s.rename_column(_table_name, _old_column_name, _new_column_name)
                    .await
            }
            Persistent(s) => {
                s.rename_column(_table_name, _old_column_name, _new_column_name)
                    .await
            }
        }
    }

    async fn add_column(&mut self, _table_name: &str, _column_def: &ColumnDef) -> GResult<()> {
        match self {
            Memory(s) => s.add_column(_table_name, _column_def).await,
            Persistent(s) => s.add_column(_table_name, _column_def).await,
        }
    }

    async fn drop_column(
        &mut self,
        _table_name: &str,
        _column_name: &str,
        _if_exists: bool,
    ) -> GResult<()> {
        match self {
            Memory(s) => s.drop_column(_table_name, _column_name, _if_exists).await,
            Persistent(s) => s.drop_column(_table_name, _column_name, _if_exists).await,
        }
    }
}

#[async_trait(?Send)]
impl Transaction for Db {
    async fn begin(&mut self, autocommit: bool) -> GResult<bool> {
        match self {
            Memory(s) => s.begin(autocommit).await,
            Persistent(s) => s.begin(autocommit).await,
        }
    }

    async fn rollback(&mut self) -> GResult<()> {
        match self {
            Memory(s) => s.rollback().await,
            Persistent(s) => s.rollback().await,
        }
    }

    async fn commit(&mut self) -> GResult<()> {
        match self {
            Memory(s) => s.commit().await,
            Persistent(s) => s.commit().await,
        }
    }
}

#[async_trait(?Send)]
impl CustomFunction for Db {
    async fn fetch_function(&self, _func_name: &str) -> GResult<Option<&StructCustomFunction>> {
        match self {
            Memory(s) => s.fetch_function(_func_name).await,
            Persistent(s) => s.fetch_function(_func_name).await,
        }
    }

    async fn fetch_all_functions(&self) -> GResult<Vec<&StructCustomFunction>> {
        match self {
            Memory(s) => s.fetch_all_functions().await,
            Persistent(s) => s.fetch_all_functions().await,
        }
    }
}

#[async_trait(?Send)]
impl CustomFunctionMut for Db {
    async fn insert_function(&mut self, _func: StructCustomFunction) -> GResult<()> {
        match self {
            Memory(s) => s.insert_function(_func).await,
            Persistent(s) => s.insert_function(_func).await,
        }
    }

    async fn delete_function(&mut self, _func_name: &str) -> GResult<()> {
        match self {
            Memory(s) => s.delete_function(_func_name).await,
            Persistent(s) => s.delete_function(_func_name).await,
        }
    }
}

#[async_trait(?Send)]
impl Index for Db {
    async fn scan_indexed_data(
        &self,
        _table_name: &str,
        _index_name: &str,
        _asc: Option<bool>,
        _cmp_value: Option<(&IndexOperator, DbValue)>,
    ) -> GResult<RowIter> {
        match self {
            Memory(s) => {
                s.scan_indexed_data(_table_name, _index_name, _asc, _cmp_value)
                    .await
            }
            Persistent(s) => {
                s.scan_indexed_data(_table_name, _index_name, _asc, _cmp_value)
                    .await
            }
        }
    }
}

#[async_trait(?Send)]
impl IndexMut for Db {
    async fn create_index(
        &mut self,
        _table_name: &str,
        _index_name: &str,
        _column: &OrderByExpr,
    ) -> GResult<()> {
        match self {
            Memory(s) => s.create_index(_table_name, _index_name, _column).await,
            Persistent(s) => s.create_index(_table_name, _index_name, _column).await,
        }
    }

    async fn drop_index(&mut self, _table_name: &str, _index_name: &str) -> GResult<()> {
        match self {
            Memory(s) => s.drop_index(_table_name, _index_name).await,
            Persistent(s) => s.drop_index(_table_name, _index_name).await,
        }
    }
}

type ObjectName = String;
use std::collections::HashMap;
pub type MetaIter = Box<dyn Iterator<Item = GResult<(ObjectName, HashMap<String, DbValue>)>>>;

#[async_trait(?Send)]
impl Metadata for Db {
    async fn scan_table_meta(&self) -> GResult<MetaIter> {
        Ok(Box::new(std::iter::empty()))
    }
}
