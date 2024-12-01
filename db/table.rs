use crate::*;

use gluesql::core::ast_builder::{DeleteNode, InsertNode, SelectNode, UpdateNode};

/// Describes [`Table`]-derived column schema
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

/// Describes [`Table`]-derived columns schema
pub type ColumnsSchema = &'static [ColumnSchema];

/// Describes [`Table`]-derived table schema
#[derive(Debug, Clone)]

pub struct TableSchema {
    pub name: &'static str,
    pub columns: ColumnsSchema,
}

/// Describes a collection of [`Table`]-derived schemas
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

/// Derived interface to access schemas of derived [`Table`]s
#[async_trait]
pub trait TableSchemaTrait: Sync {
    fn name(&self) -> &'static str;
    fn schema(&self) -> ColumnsSchema;
    fn relative_path(&self) -> &'static str;
    fn full_path(&self) -> &'static str;
    async fn get_all(&self) -> Vec<Vec<String>>;
    async fn get_row_by_id(&self, id: String) -> Result<Vec<String>>;
    async fn save(&self, req: Request) -> Result<String>;
    async fn remove(&self, req: Request) -> Result;
}

/// Derived interface to interact with structs as tables of their values
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

    fn update() -> UpdateNode {
        table(Self::TABLE_NAME).update()
    }

    fn find_all() -> Vec<Self> {
        Self::from_rows(Self::select().rows().unwrap())
    }

    fn insert_self(&self) -> Result {
        Self::insert().values(vec![self.into_row()]).exec()?;
        OK
    }

    fn find_by_key(key: &Self::Key) -> Option<Self> {
        let mut rows = Self::select().filter(Self::key_filter(key)).rows().unwrap();
        match rows.pop() {
            Some(row) => Some(Self::from_row(row)),
            None => None,
        }
    }

    // fn update_by_key(key: &Self::Key) -> UpdateFilterNode<'static> {
    //     Self::update().filter(Self::key_filter(key))
    // }

    fn delete_by_key(key: &Self::Key) -> Result {
        let payload = Self::delete().filter(Self::key_filter(key)).exec()?;
        match payload {
            Payload::Delete(_) => OK,
            _ => return Err(e!("Couldn't delete item with key = {key}")),
        }
    }

    fn remove(&self) -> Result {
        Self::delete_by_key(&self.get_key())
    }
}
