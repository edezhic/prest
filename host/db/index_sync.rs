use {
    super::{AsStorageError, Snapshot, WriteState},
    gluesql_core::{
        ast::Expr,
        data::schema::{Schema, SchemaIndex},
        error::{Error, IndexError, Result},
        executor::evaluate_stateless,
        prelude::Value,
        store::DataRow,
    },
    sled::{Db, InlineArray},
    std::borrow::Cow,
};

pub struct IndexSync<'a> {
    tree: &'a Db,
    state: WriteState,
    table_name: &'a str,
    columns: Option<Vec<String>>,
    indexes: Cow<'a, Vec<SchemaIndex>>,
}

impl<'a> IndexSync<'a> {
    pub fn from_schema(tree: &'a Db, state: WriteState, schema: &'a Schema) -> Self {
        let Schema {
            table_name,
            column_defs,
            indexes,
            ..
        } = schema;

        let columns = column_defs.as_ref().map(|column_defs| {
            column_defs
                .iter()
                .map(|column_def| column_def.name.to_owned())
                .collect::<Vec<_>>()
        });

        let indexes = Cow::Borrowed(indexes);

        Self {
            tree,
            state,
            table_name,
            columns,
            indexes,
        }
    }

    pub fn new(tree: &'a Db, state: WriteState, table_name: &'a str) -> Result<Self, Error> {
        // crate::info!("IndexSync::new table_name={table_name} txid={txid}");
        let Schema {
            column_defs,
            indexes,
            ..
        } = crate::DB
            .fetch_glue_schema(table_name)
            .ok_or_else(|| IndexError::ConflictTableNotFound(table_name.to_owned()))?;

        let columns = column_defs.map(|column_defs| {
            column_defs
                .into_iter()
                .map(|column_def| column_def.name)
                .collect::<Vec<_>>()
        });

        Ok(Self {
            tree,
            state,
            table_name,
            columns,
            indexes: Cow::Owned(indexes),
        })
    }

    pub async fn insert(&self, data_key: &InlineArray, row: &DataRow) -> Result<()> {
        for index in self.indexes.iter() {
            self.insert_index(index, data_key, row).await?;
        }

        Ok(())
    }

    pub async fn insert_index(
        &self,
        index: &SchemaIndex,
        data_key: &InlineArray,
        row: &DataRow,
    ) -> Result<()> {
        let SchemaIndex {
            name: index_name,
            expr: index_expr,
            ..
        } = index;

        let index_key = &evaluate_index_key(
            self.table_name,
            index_name,
            index_expr,
            self.columns.as_deref(),
            row,
        )
        .await?;

        self.insert_index_data(index_key, data_key)?;

        Ok(())
    }

    pub async fn update(
        &self,
        data_key: &InlineArray,
        old_row: &DataRow,
        new_row: &DataRow,
    ) -> Result<()> {
        for index in self.indexes.iter() {
            let SchemaIndex {
                name: index_name,
                expr: index_expr,
                ..
            } = index;

            let old_index_key = &evaluate_index_key(
                self.table_name,
                index_name,
                index_expr,
                self.columns.as_deref(),
                old_row,
            )
            .await?;

            let new_index_key = &evaluate_index_key(
                self.table_name,
                index_name,
                index_expr,
                self.columns.as_deref(),
                new_row,
            )
            .await?;

            self.delete_index_data(old_index_key, data_key)?;
            self.insert_index_data(new_index_key, data_key)?;
        }

        Ok(())
    }

    pub async fn delete(&self, data_key: &InlineArray, row: &DataRow) -> Result<()> {
        for index in self.indexes.iter() {
            self.delete_index(index, data_key, row).await?;
        }

        Ok(())
    }

    pub async fn delete_index(
        &self,
        index: &SchemaIndex,
        data_key: &InlineArray,
        row: &DataRow,
    ) -> Result<()> {
        let SchemaIndex {
            name: index_name,
            expr: index_expr,
            ..
        } = index;

        let index_key = &evaluate_index_key(
            self.table_name,
            index_name,
            index_expr,
            self.columns.as_deref(),
            row,
        )
        .await?;

        self.delete_index_data(index_key, data_key)?;

        Ok(())
    }

    fn insert_index_data(&self, index_key: &[u8], data_key: &InlineArray) -> Result<()> {
        let mut data_keys: Vec<Snapshot<Vec<u8>>> = self
            .tree
            .get(index_key)
            .as_storage_err()?
            .map(|v| bitcode::deserialize(&v))
            .transpose()
            .as_storage_err()?
            .unwrap_or_default();

        let key_snapshot = Snapshot::<Vec<u8>>::new(self.state.tx_id, data_key.to_vec());
        data_keys.push(key_snapshot);
        let data_keys = bitcode::serialize(&Vec::from(data_keys)).as_storage_err()?;

        // let temp_key = super::temp_index(self.state.txid, index_key);

        self.tree.insert(index_key, data_keys).as_storage_err()?;
        // self.tree.insert(temp_key, index_key).as_storage_err()?;

        Ok(())
    }

    fn delete_index_data(&self, index_key: &[u8], data_key: &InlineArray) -> Result<()> {
        let data_keys: Vec<Snapshot<Vec<u8>>> = self
            .tree
            .get(index_key)
            .as_storage_err()?
            .map(|v| bitcode::deserialize(&v))
            .transpose()
            .as_storage_err()?
            .ok_or(Error::StorageMsg("index not found".into()))?;
        // .as_storage_err()?;

        let data_keys = data_keys
            .into_iter()
            .map(|snapshot| {
                todo!("fix this stuff (state isn't here)");
                // let key = snapshot.get(state);

                // if Some(data_key) == key.map(InlineArray::from).as_ref() {
                //     snapshot.delete(self.state)
                // } else {
                //     Some(snapshot)
                // }
            })
            .collect::<Vec<_>>();

        let data_keys = bitcode::serialize(&data_keys).as_storage_err()?;

        // let temp_key = super::temp_index(self.state.txid, index_key);

        self.tree.insert(index_key, data_keys).as_storage_err()?;
        // self.tree.insert(temp_key, index_key).as_storage_err()?;

        Ok(())
    }
}

async fn evaluate_index_key(
    table_name: &str,
    index_name: &str,
    index_expr: &Expr,
    columns: Option<&[String]>,
    row: &DataRow,
) -> Result<Vec<u8>> {
    let context = Some(row.as_context(columns));
    let evaluated = evaluate_stateless(context, index_expr).await?;
    let value: Value = evaluated.try_into()?;

    build_index_key(table_name, index_name, value)
}

pub fn build_index_key_prefix(table_name: &str, index_name: &str) -> Vec<u8> {
    format!("index/{}/{}/", table_name, index_name).into_bytes()
}

pub fn build_index_key(table_name: &str, index_name: &str, value: Value) -> Result<Vec<u8>> {
    Ok(build_index_key_prefix(table_name, index_name)
        .into_iter()
        .chain(value.to_cmp_be_bytes()?)
        .collect::<Vec<_>>())
}
