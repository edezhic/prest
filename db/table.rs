use crate::*;

use gluesql::core::ast_builder::{DeleteNode, InsertNode, SelectNode, UpdateNode};

/// Describes [`Table`]-derived column schema
#[derive(Debug, Clone, Copy)]
pub struct ColumnSchema {
    pub name: &'static str,
    pub rust_type: &'static str,
    pub sql_type: &'static str,
    pub unique: bool,
    pub pkey: bool,
    pub list: bool,
    pub optional: bool,
    pub serialized: bool,
    pub numeric: bool,
    pub comparable: bool,
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
    async fn get_all(&self) -> Result<Vec<Vec<String>>>;
    async fn get_row_by_id(&self, id: String) -> Result<Vec<String>>;
    async fn save(&self, req: Request) -> Result<String>;
    async fn remove(&self, req: Request) -> Result;
}

/// Derived interface to interact with structs as tables of their values
pub trait Table: Sized {
    const TABLE_NAME: &'static str;
    const TABLE_SCHEMA: ColumnsSchema;
    const KEY: &'static str;
    type Key: std::fmt::Display;

    fn migrate();
    fn prepare_table();
    fn into_row(&self) -> Result<sql::ExprList<'static>>;
    fn from_row(row: Vec<sql::Value>) -> Result<Self>;
    fn get_pkey(&self) -> &Self::Key;
    fn save(&self) -> Result<&Self>;

    fn pkey_filter<'a, 'b>(pkey: &'a Self::Key) -> sql::ExprNode<'b>;

    fn from_rows(rows: Vec<Vec<sql::Value>>) -> Result<Vec<Self>> {
        rows.into_iter().map(Self::from_row).collect()
    }

    fn select() -> SelectNode<'static> {
        sql::table(Self::TABLE_NAME).select()
    }

    fn insert() -> InsertNode {
        sql::table(Self::TABLE_NAME).insert()
    }

    fn delete() -> DeleteNode<'static> {
        sql::table(Self::TABLE_NAME).delete()
    }

    fn update() -> UpdateNode {
        sql::table(Self::TABLE_NAME).update()
    }

    fn find_all() -> Result<Vec<Self>> {
        Self::from_rows(Self::select().rows()?)
    }

    fn insert_self(&self) -> Result {
        Self::insert().values(vec![self.into_row()?]).exec()?;
        OK
    }

    fn find_by_pkey(pkey: &Self::Key) -> Result<Option<Self>> {
        let mut rows = Self::select().filter(Self::pkey_filter(pkey)).rows()?;
        match rows.pop() {
            Some(row) => Ok(Some(Self::from_row(row)?)),
            None => Ok(None),
        }
    }

    fn delete_by_pkey(pkey: &Self::Key) -> Result {
        let payload = Self::delete().filter(Self::pkey_filter(pkey)).exec()?;
        match payload {
            sql::Payload::Delete(_) => OK,
            _ => return Err(e!("Couldn't delete item with pkey = {pkey}")),
        }
    }

    fn remove(&self) -> Result {
        Self::delete_by_pkey(&self.get_pkey())
    }
}
