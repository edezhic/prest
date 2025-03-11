use {
    super::{index_sync::IndexSync, sql, AsStorageError, DbConn, Snapshot},
    async_trait::async_trait,
    gluesql_core::{
        data::{Key, Schema},
        error::{Error, Result},
        store::{DataRow, StoreMut},
    },
};

#[async_trait(?Send)]
impl<'a> StoreMut for DbConn<'a> {
    async fn insert_schema(&mut self, _schema: &Schema) -> Result<()> {
        unimplemented!("Schemas are only defined by code")
    }

    async fn delete_schema(&mut self, _table_name: &str) -> Result<()> {
        unimplemented!("Schemas are only defined by code")
    }

    async fn append_data(&mut self, table_name: &str, _rows: Vec<DataRow>) -> Result<()> {
        panic!("append data (called for {table_name}) is only for unstructured data? shouldn't be used... or... for what?");
    }

    async fn insert_data(&mut self, table_name: &str, rows: Vec<(Key, DataRow)>) -> Result<()> {
        if self.readonly {
            return Err(Error::StorageMsg(
                "can't write inside of read statement".into(),
            ));
        }

        let tx_rows = &rows;

        // let index_sync = IndexSync::new(&self.tree, self.state, table_name)?;

        for (key, new_row) in tx_rows.iter() {
            let key = super::sled_key(table_name, key.clone())?;

            let snapshot = match self.tree.get(&key).as_storage_err()? {
                Some(snapshot) => {
                    let mut snapshot: Snapshot<DataRow> =
                        bitcode::deserialize(&snapshot).as_storage_err()?;

                    snapshot.update(self.state, new_row.clone());

                    // index_sync.update(&key, &old_row, new_row).await?;

                    snapshot
                }
                None => {
                    // index_sync.insert(&key, new_row).await?;

                    Snapshot::new(self.state.tx_id, new_row.clone())
                }
            };

            let snapshot = bitcode::serialize(&snapshot).as_storage_err()?;

            self.tree.insert(&key, snapshot).as_storage_err()?;
        }

        Ok(())
    }

    async fn delete_data(&mut self, table_name: &str, keys: Vec<Key>) -> Result<()> {
        if self.readonly {
            return Err(Error::StorageMsg(
                "can't write inside of read statement".into(),
            ));
        }

        let tx_keys = &keys;

        // let index_sync = IndexSync::new(&self.tree, self.state, table_name)?;

        for key in tx_keys.iter() {
            let key = super::sled_key(table_name, key.clone())?;

            let snapshot = self
                .tree
                .get(&key)
                .as_storage_err()?
                .ok_or(Error::StorageMsg("not found item to delete".into()))?;
            let snapshot: Snapshot<DataRow> = bitcode::deserialize(&snapshot).as_storage_err()?;

            let Some(updated) = snapshot.delete(self.state) else {
                continue;
            };

            bitcode::serialize(&updated)
                .as_storage_err()
                .map(|snapshot| self.tree.insert(&key, snapshot).as_storage_err())??;

            // index_sync.delete(&key, &updated).await?;
        }
        Ok(())
    }
}

impl DbConn<'_> {
    pub async fn update_cell(
        &mut self,
        table_name: &str,
        key: Key,
        field: usize,
        value: sql::Value,
    ) -> Result<()> {
        // let index_sync = IndexSync::new(&self.tree, self.state, table_name)?;

        let key = super::sled_key(table_name, key.clone())?;

        match self
            .tree
            .get(&key)
            .as_storage_err()?
            .map(|s| bitcode::deserialize::<Snapshot<DataRow>>(&s).as_storage_err())
            .transpose()?
        {
            Some(mut snapshot) => {
                if let Some(row) = snapshot.get_mut(self.state) {
                    if let DataRow::Vec(row) = row {
                        if let Some(cell) = row.get_mut(field) {
                            *cell = value;
                            let snapshot = bitcode::serialize(&snapshot).as_storage_err()?;
                            self.tree.insert(&key, snapshot).as_storage_err()?;
                            // index_sync.update(&key, &old_row, new_row).await?;
                        } else {
                            panic!("update_cell with non-existent value");
                        }
                    } else {
                        crate::error!("update_cell used with DataRow::Map");
                    }
                }
            }
            None => panic!("update_cell with non-existent row"),
        };

        Ok(())
    }
}
