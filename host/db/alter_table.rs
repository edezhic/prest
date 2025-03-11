use {
    super::{AsStorageError, DbConn, Snapshot},
    crate::*,
    gluesql_core::{
        ast::ColumnDef,
        data::{schema::Schema, Value},
        error::{AlterTableError, Error, Result},
        executor::evaluate_stateless,
        store::{AlterTable, DataRow},
    },
    std::{iter::once, str},
};

#[async_trait(?Send)]
impl<'a> AlterTable for DbConn<'a> {
    async fn rename_schema(&mut self, _table_name: &str, _new_table_name: &str) -> Result<()> {
        unimplemented!("rename table method is not supported");
    }

    async fn rename_column(
        &mut self,
        _table_name: &str,
        _old_column_name: &str,
        _new_column_name: &str,
    ) -> Result<()> {
        unimplemented!("rename column method is not supported");
    }

    async fn add_column(&mut self, table_name: &str, column_def: &ColumnDef) -> Result<()> {
        let prefix = format!("data/{}/", table_name);
        let items = self
            .tree
            .scan_prefix(prefix.as_bytes())
            .map(|item| item.as_storage_err())
            .collect::<Result<Vec<_>>>()?;

        // let schema_snapshot = fetch_schema(&self.tree, table_name)?;
        // let mut schema_snapshot = schema_snapshot
        //     .ok_or(Error::AlterTable(AlterTableError::TableNotFound(table_name.to_owned())))?;

        let Schema {
            table_name,
            column_defs,
            indexes,
            engine,
            foreign_keys,
            comment,
        } = crate::DB
            .fetch_glue_schema(table_name)
            .ok_or(Error::AlterTable(AlterTableError::TableNotFound(
                table_name.to_owned(),
            )))?;

        let column_defs = column_defs.ok_or(Error::AlterTable(
            AlterTableError::SchemalessTableFound(table_name.to_owned()),
        ))?;

        if column_defs
            .iter()
            .any(|ColumnDef { name, .. }| name == &column_def.name)
        {
            let adding_column = column_def.name.to_owned();
            return Err(Error::AlterTable(AlterTableError::AlreadyExistingColumn(
                adding_column,
            )));
        }

        let ColumnDef {
            data_type,
            nullable,
            default,
            ..
        } = column_def;

        let value = match (default, nullable) {
            (Some(expr), _) => {
                let evaluated = evaluate_stateless(None, expr).await_blocking()?;

                evaluated.try_into_value(data_type, *nullable)?
            }
            (None, true) => Value::Null,
            (None, false) => {
                return Err(Error::AlterTable(AlterTableError::DefaultValueRequired(
                    column_def.clone(),
                )));
            }
        };

        // migrate data
        todo!("actual migrate data methods");
        for (key, snapshot) in items.iter() {
            let mut snapshot: Snapshot<DataRow> =
                bitcode::deserialize(snapshot).as_storage_err()?;
            let row = match snapshot.get(self.state).take() {
                Some(row) => row,
                None => {
                    continue;
                }
            };

            let values = match row {
                DataRow::Vec(values) => values,
                DataRow::Map(_) => {
                    return Err(Error::StorageMsg(
                        "conflict - add_column failed: schemaless row found".to_owned(),
                    ));
                }
            };
            let row = values
                .into_iter()
                .chain(once(value.clone()))
                .collect::<Vec<Value>>()
                .into();

            snapshot.update(self.state, row);
            let snapshot = bitcode::serialize(&snapshot).as_storage_err()?;

            self.tree.insert(key, snapshot).as_storage_err()?;
        }

        // update schema
        let column_defs = column_defs
            .into_iter()
            .chain(once(column_def.clone()))
            .collect::<Vec<ColumnDef>>();

        let schema = Schema {
            table_name: table_name.clone(),
            column_defs: Some(column_defs),
            indexes,
            engine,
            foreign_keys,
            comment,
        };
        // schema_snapshot.update(self.state, schema);
        // let schema_value = bitcode::serialize(&schema_snapshot).as_storage_err()?;
        // self.tree
        //     .insert(format!("{SCHEMA_PREFIX}{}", table_name).as_bytes(), schema_value)
        //     .as_storage_err()?;

        Ok(())
    }

    async fn drop_column(
        &mut self,
        _table_name: &str,
        _column_name: &str,
        _if_exists: bool,
    ) -> Result<()> {
        todo!("drop_column method");
        // let prefix = format!("data/{}/", table_name);
        // let items = self
        //     .tree
        //     .scan_prefix(prefix.as_bytes())
        //     .map(|item| item.as_storage_err())
        //     .collect::<Result<Vec<_>>>()?;

        // let txid = self.txid;

        // self.tree
        //     .transaction(move |tree| {
        //         let (schema_key, schema_snapshot) = fetch_schema(tree, table_name)?;
        //         let schema_snapshot = schema_snapshot
        //             .ok_or_else(|| AlterTableError::TableNotFound(table_name.to_owned()).into())
        //             .map_err(ConflictableTransactionError::Abort)?;

        //         let Schema {
        //             table_name,
        //             column_defs,
        //             indexes,
        //             engine,
        //             foreign_keys,
        //             comment,
        //         } = schema_snapshot
        //             .get(txid, None)
        //             .ok_or_else(|| AlterTableError::TableNotFound(table_name.to_owned()).into())
        //             .map_err(ConflictableTransactionError::Abort)?;

        //         let column_defs = column_defs
        //             .ok_or_else(|| {
        //                 AlterTableError::SchemalessTableFound(table_name.to_owned()).into()
        //             })
        //             .map_err(ConflictableTransactionError::Abort)?;

        //         let column_index = column_defs
        //             .iter()
        //             .position(|ColumnDef { name, .. }| name == column_name);
        //         let column_index = match (column_index, if_exists) {
        //             (Some(index), _) => index,
        //             (None, true) => {
        //                 return Ok(());
        //             }
        //             (None, false) => {
        //                 return Err(ConflictableTransactionError::Abort(
        //                     AlterTableError::DroppingColumnNotFound(column_name.to_owned()).into(),
        //                 ));
        //             }
        //         };

        //         // migrate data
        //         for (key, snapshot) in items.iter() {
        //             let snapshot: Snapshot<DataRow> = bitcode::deserialize(snapshot)
        //                 .as_storage_err()
        //                 .map_err(ConflictableTransactionError::Abort)?;
        //             let row = match snapshot.clone().extract(txid, None) {
        //                 Some(row) => row,
        //                 None => {
        //                     continue;
        //                 }
        //             };

        //             let values = match row {
        //                 DataRow::Vec(values) => values,
        //                 DataRow::Map(_) => {
        //                     return Err(ConflictableTransactionError::Abort(Error::StorageMsg(
        //                         "conflict - drop_column failed: schemaless row found".to_owned(),
        //                     )));
        //                 }
        //             };

        //             let row = values
        //                 .into_iter()
        //                 .enumerate()
        //                 .filter_map(|(i, v)| (i != column_index).then_some(v))
        //                 .collect::<Vec<_>>()
        //                 .into();

        //             let (snapshot, _) = snapshot.update(txid, row);
        //             let snapshot = bitcode::serialize(&snapshot)
        //                 .as_storage_err()
        //                 .map_err(ConflictableTransactionError::Abort)?;

        //             tree.insert(key, snapshot)?;
        //         }

        //         // update schema
        //         let column_defs = column_defs
        //             .into_iter()
        //             .enumerate()
        //             .filter_map(|(i, v)| (i != column_index).then_some(v))
        //             .collect::<Vec<ColumnDef>>();

        //         let schema = Schema {
        //             table_name,
        //             column_defs: Some(column_defs),
        //             indexes,
        //             engine,
        //             foreign_keys,
        //             comment,
        //         };
        //         let (schema_snapshot, _) = schema_snapshot.update(txid, schema);
        //         let schema_value = bitcode::serialize(&schema_snapshot)
        //             .as_storage_err()
        //             .map_err(ConflictableTransactionError::Abort)?;
        //         tree.insert(schema_key.as_bytes(), schema_value)?;

        //         Ok(())
        //     })
        //     .map_err(tx_err_into)?;

        // Ok(())
    }
}
