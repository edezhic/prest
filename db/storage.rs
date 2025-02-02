use crate::*;

use db::key::IntoSqlKey;
use gluesql_core::{
    ast::{ColumnDef, ColumnUniqueOption, Expr},
    ast_builder::{DeleteNode, InsertNode, SelectNode, UpdateNode},
    data::{Schema as GlueSchema, SchemaIndex, SchemaIndexOrd},
};

/// Describes [`Storage`]-derived columns schema
#[doc(hidden)]
pub type FieldSchemas = &'static [FieldSchema];

/// Describes [`Storage`]-derived column schema
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub name: &'static str,
    pub rust_type: &'static str,
    pub sql_type: sql::DataType,
    pub unique: bool,
    pub pkey: bool,
    pub list: bool,
    pub optional: bool,
    pub serialized: bool,
    pub numeric: bool,
    pub comparable: bool,
}

pub type StructSchema = &'static dyn StructSchemaTrait;

pub fn into_glue_schema(schema: StructSchema) -> GlueSchema {
    let pkey = schema
        .fields()
        .iter()
        .find(|c| c.pkey)
        .map(|c| c.name)
        .expect("must have a PK");

    let columns = schema
        .fields()
        .iter()
        .map(|c| ColumnDef {
            name: c.name.to_owned(),
            data_type: c.sql_type.clone(),
            nullable: c.optional,
            default: None,
            unique: if c.unique {
                Some(ColumnUniqueOption { is_primary: c.pkey })
            } else {
                None
            },
            comment: None,
        })
        .collect::<Vec<_>>();

    GlueSchema {
        table_name: schema.name().to_owned(),
        column_defs: Some(columns),
        indexes: vec![],
        engine: None,
        foreign_keys: vec![],
        comment: None,
    }
}

/// Derived interface to access schemas of derived [`Storage`]s
#[async_trait]
pub trait StructSchemaTrait: Sync {
    fn name(&self) -> &'static str;
    fn fields(&self) -> FieldSchemas;
    fn relative_path(&self) -> &'static str;
    fn full_path(&self) -> &'static str;
    async fn get_all_as_strings(&self) -> Result<Vec<Vec<String>>>;
    async fn get_as_strings_by_id(&self, id: String) -> Result<Vec<String>>;
    async fn save(&self, req: Request) -> Result<String>;
    async fn remove(&self, req: Request) -> Result;
}

/// Derived interface to interact with structs as tables of their values
#[async_trait]
pub trait Storage: Sized + Send + serde::de::DeserializeOwned {
    const STRUCT_NAME: &'static str;
    const FIELD_SCHEMAS: FieldSchemas;
    const PK_INDEX: usize;

    type Key: std::fmt::Display + Send + Clone + IntoSqlKey;

    fn schema() -> StructSchema;

    fn into_expr_list(&self) -> Result<sql::ExprList<'static>>;
    fn into_row(&self) -> Result<Vec<sql::Value>>;
    fn from_row(row: Vec<sql::Value>) -> Result<Self>;
    fn from_rows(rows: Vec<Vec<sql::Value>>) -> Result<Vec<Self>> {
        rows.into_iter().map(Self::from_row).collect()
    }

    fn get_pkey(&self) -> &Self::Key;
    async fn get_by_pkey(pkey: Self::Key) -> Result<Option<Self>> {
        let pkey = pkey.into_sql_key();
        let prest::db::Payload::Rows(mut rows) = DB
            .read(prest::Query::GetByPKey {
                pkey,
                name: Self::STRUCT_NAME,
            })
            .await?
        else {
            panic!("unexpected DB get_by_pkey return")
        };
        Ok(rows.pop().map(|r| Self::from_row(r)).transpose()?)
    }

    fn pk_filter_sql_node<'a, 'b>(pkey: &'a Self::Key) -> sql::ExprNode<'b>;

    fn select() -> SelectNode<'static> {
        sql::table(Self::STRUCT_NAME).select()
    }

    fn insert() -> InsertNode {
        sql::table(Self::STRUCT_NAME).insert()
    }

    fn delete() -> DeleteNode<'static> {
        sql::table(Self::STRUCT_NAME).delete()
    }

    fn update() -> UpdateNode {
        sql::table(Self::STRUCT_NAME).update()
    }

    async fn get_all() -> Result<Vec<Self>> {
        Self::select().rows().await
    }

    async fn insert_self(&self) -> Result {
        let pkey = self.get_pkey().clone().into_sql_key();
        let row = self.into_row()?;
        let payload = DB
            .write(prest::Transaction::Insert {
                name: Self::STRUCT_NAME,
                key: pkey,
                row,
            })
            .await?;

        let prest::db::Payload::Success = payload else {
            panic!("unexpected DB insert_self return payload: {payload:?}")
        };
        OK
    }

    async fn save(&self) -> Result<&Self> {
        let pkey = self.get_pkey().clone().into_sql_key();
        let row = self.into_row()?;
        let payload = DB
            .write(prest::Transaction::Save {
                name: Self::STRUCT_NAME,
                key: pkey,
                row,
            })
            .await?;

        let prest::db::Payload::Success = payload else {
            panic!("unexpected DB save return payload: {payload:?}")
        };
        Ok(self)
    }

    async fn delete_by_pkey(pkey: Self::Key) -> Result {
        let payload = DB
            .write(prest::Transaction::Delete {
                name: Self::STRUCT_NAME,
                key: pkey.into_sql_key(),
            })
            .await?;

        let prest::db::Payload::Success = payload else {
            panic!("unexpected DB remove return payload: {payload:?}")
        };
        OK
    }

    async fn remove(&self) -> Result {
        Self::delete_by_pkey(self.get_pkey().clone()).await
    }
}
