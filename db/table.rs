use crate::*;

use gluesql::core::ast_builder::{DeleteNode, InsertNode, SelectNode, UpdateNode};

/// Describes [`Table`]-derived columns schema
#[doc(hidden)]
pub type ColumnSchemas = &'static [ColumnSchema];

/// Describes [`Table`]-derived column schema
#[doc(hidden)]
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

pub type TableSchema = &'static dyn TableSchemaTrait;

/// Derived interface to access schemas of derived [`Table`]s
#[async_trait]
pub trait TableSchemaTrait: Sync {
    fn name(&self) -> &'static str;
    fn columns(&self) -> ColumnSchemas;
    fn relative_path(&self) -> &'static str;
    fn full_path(&self) -> &'static str;
    async fn get_all(&self) -> Result<Vec<Vec<String>>>;
    async fn get_row_by_id(&self, id: String) -> Result<Vec<String>>;
    async fn save(&self, req: Request) -> Result<String>;
    async fn remove(&self, req: Request) -> Result;
}

/// Derived interface to interact with structs as tables of their values
#[async_trait]
pub trait Table: Sized + Send {
    const TABLE_NAME: &'static str;
    const COLUMN_SCHEMAS: ColumnSchemas;

    type Key: std::fmt::Display + Send + Clone;

    fn schema() -> TableSchema;

    fn into_row(&self) -> Result<sql::ExprList<'static>>;
    fn from_row(row: Vec<sql::Value>) -> Result<Self>;
    fn from_rows(rows: Vec<Vec<sql::Value>>) -> Result<Vec<Self>> {
        rows.into_iter().map(Self::from_row).collect()
    }

    fn get_pkey(&self) -> &Self::Key;
    fn pkey_filter<'a, 'b>(pkey: &'a Self::Key) -> sql::ExprNode<'b>;

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

    async fn save(&self) -> Result<&Self>;

    async fn select_all() -> Result<Vec<Self>> {
        Self::from_rows(Self::select().rows().await?)
    }

    async fn insert_self(&self) -> Result {
        Self::insert().values(vec![self.into_row()?]).exec().await?;
        OK
    }

    async fn select_by_pkey(pkey: Self::Key) -> Result<Option<Self>> {
        let mut rows = Self::select()
            .filter(Self::pkey_filter(&pkey))
            .rows()
            .await?;
        match rows.pop() {
            Some(row) => Ok(Some(Self::from_row(row)?)),
            None => Ok(None),
        }
    }

    async fn delete_by_pkey(pkey: Self::Key) -> Result {
        let payload = Self::delete()
            .filter(Self::pkey_filter(&pkey))
            .exec()
            .await?;
        match payload {
            sql::Payload::Delete(_) => OK,
            _ => return Err(e!("Couldn't delete item with pkey = {pkey}")),
        }
    }

    async fn remove(&self) -> Result {
        Self::delete_by_pkey(self.get_pkey().clone()).await
    }
}
