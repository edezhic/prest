use {
    super::{index_sync::IndexSync, AsStorageError, DbConn},
    crate::*,
    async_trait::async_trait,
    futures::stream::TryStreamExt,
    gluesql_core::{
        ast::OrderByExpr,
        chrono::Utc,
        data::{Schema, SchemaIndex, SchemaIndexOrd},
        error::{Error, IndexError, Result},
        store::{IndexMut, Store},
    },
    std::iter::once,
};

#[async_trait(?Send)]
impl<'a> IndexMut for DbConn<'a> {
    async fn create_index(
        &mut self,
        _table_name: &str,
        _index_name: &str,
        _column: &OrderByExpr,
    ) -> Result<()> {
        // TODO

        // let rows = self
        //     .scan_data(table_name)
        //     .await?
        //     .try_collect::<Vec<_>>()
        //     .await?;

        // let txid = self.txid;

        // self.tree
        //     .transaction(move |tree| {
        //         let index_expr = &column.expr;

        //         let (schema_key, schema_snapshot) = fetch_schema(tree, table_name)?;
        //         let schema_snapshot = schema_snapshot
        //             .ok_or_else(|| IndexError::TableNotFound(table_name.to_owned()).into())
        //             .map_err(ConflictableTransactionError::Abort)?;

        //         let (schema_snapshot, schema) = schema_snapshot.delete(txid);
        //         let Schema {
        //             column_defs,
        //             indexes,
        //             engine,
        //             foreign_keys,
        //             comment,
        //             ..
        //         } = schema
        //             .ok_or_else(|| IndexError::ConflictTableNotFound(table_name.to_owned()).into())
        //             .map_err(ConflictableTransactionError::Abort)?;

        //         if indexes.iter().any(|index| index.name == index_name) {
        //             return Err(ConflictableTransactionError::Abort(
        //                 IndexError::IndexNameAlreadyExists(index_name.to_owned()).into(),
        //             ));
        //         }

        //         let index = SchemaIndex {
        //             name: index_name.to_owned(),
        //             expr: index_expr.clone(),
        //             order: SchemaIndexOrd::Both,
        //             created: Utc::now().naive_utc(),
        //         };

        //         let indexes = indexes
        //             .into_iter()
        //             .chain(once(index.clone()))
        //             .collect::<Vec<_>>();

        //         let schema = Schema {
        //             table_name: table_name.to_owned(),
        //             column_defs,
        //             indexes,
        //             engine,
        //             foreign_keys,
        //             comment,
        //         };

        //         let index_sync = IndexSync::from_schema(tree, txid, &schema);

        //         let schema_snapshot = schema_snapshot.update(txid, schema.clone());
        //         let schema_snapshot = bitcode::serialize(&schema_snapshot)
        //             .as_storage_err()
        //             .map_err(ConflictableTransactionError::Abort)?;

        //         async {
        //             for (data_key, row) in rows.iter() {
        //                 let data_key = data_key
        //                     .to_cmp_be_bytes()
        //                     .map_err(ConflictableTransactionError::Abort)
        //                     .map(|key| key::sled_key(table_name, key))?;

        //                 index_sync.insert_index(&index, &data_key, row).await?;
        //             }

        //             Ok(()) as ConflictableTransactionResult<(), Error>
        //         }
        //         .await_blocking()?;

        //         tree.insert(schema_key.as_bytes(), schema_snapshot)?;
        //         Ok(())
        //     })
        //     .map_err(tx_err_into)?;

        Ok(())
    }

    async fn drop_index(&mut self, _table_name: &str, _index_name: &str) -> Result<()> {
        // let rows = self
        //     .scan_data(table_name)
        //     .await?
        //     .try_collect::<Vec<_>>()
        //     .await?;

        // let txid = self.txid;

        // self.tree
        //     .transaction(move |tree| {
        //         let (schema_key, schema_snapshot) = fetch_schema(tree, table_name)?;
        //         let schema_snapshot = schema_snapshot
        //             .ok_or_else(|| IndexError::TableNotFound(table_name.to_owned()).into())
        //             .map_err(ConflictableTransactionError::Abort)?;

        //         let (schema_snapshot, schema) = schema_snapshot.delete(txid);
        //         let Schema {
        //             column_defs,
        //             indexes,
        //             engine,
        //             foreign_keys,
        //             comment,
        //             ..
        //         } = schema
        //             .ok_or_else(|| IndexError::ConflictTableNotFound(table_name.to_owned()).into())
        //             .map_err(ConflictableTransactionError::Abort)?;

        //         let (index, indexes): (Vec<_>, _) = indexes
        //             .into_iter()
        //             .partition(|index| index.name == index_name);

        //         let index = match index.into_iter().next() {
        //             Some(index) => index,
        //             None => {
        //                 return Err(ConflictableTransactionError::Abort(
        //                     IndexError::IndexNameDoesNotExist(index_name.to_owned()).into(),
        //                 ));
        //             }
        //         };

        //         let schema = Schema {
        //             table_name: table_name.to_owned(),
        //             column_defs,
        //             indexes,
        //             engine,
        //             foreign_keys,
        //             comment,
        //         };

        //         let index_sync = IndexSync::from_schema(tree, txid, &schema);

        //         let schema_snapshot = schema_snapshot.update(txid, schema.clone());
        //         let schema_snapshot = bitcode::serialize(&schema_snapshot)
        //             .as_storage_err()
        //             .map_err(ConflictableTransactionError::Abort)?;

        //         async {
        //             for (data_key, row) in rows.iter() {
        //                 let data_key = data_key
        //                     .to_cmp_be_bytes()
        //                     .map_err(ConflictableTransactionError::Abort)
        //                     .map(|key| key::sled_key(table_name, key))?;

        //                 index_sync.delete_index(&index, &data_key, row).await?;
        //             }

        //             Ok(()) as ConflictableTransactionResult<(), Error>
        //         }
        //         .await_blocking()?;

        //         tree.insert(schema_key.as_bytes(), schema_snapshot)?;
        //         Ok(())
        //     })
        //     .map_err(tx_err_into)?;

        Ok(())
    }
}
