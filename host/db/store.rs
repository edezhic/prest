use {
    super::{AsStorageError, DbConn, Snapshot},
    async_trait::async_trait,
    futures::stream::iter,
    gluesql_core::{
        data::{Key, Schema, Value},
        error::Result,
        store::{DataRow, RowIter, Store},
    },
    russh_sftp::protocol::Data,
    std::str,
};

#[async_trait(?Send)]
impl<'a> Store for DbConn<'a> {
    async fn fetch_all_schemas(&self) -> Result<Vec<Schema>> {
        Ok(crate::DB.fetch_all_glue_schemas())
    }

    async fn fetch_schema(&self, table_name: &str) -> Result<Option<Schema>> {
        Ok(crate::DB.fetch_glue_schema(table_name))
    }

    async fn fetch_data(&self, table_name: &str, key: &Key) -> Result<Option<DataRow>> {
        Ok(self
            .tree
            .get(super::sled_key(table_name, key.clone())?)
            .as_storage_err()?
            .map(|v| bitcode::deserialize(&v))
            .transpose()
            .as_storage_err()?
            .and_then(|snapshot: Snapshot<DataRow>| snapshot.take(self.state)))
    }

    async fn scan_data(&self, table_name: &str) -> Result<RowIter> {
        let prefix = super::data_prefix(table_name);
        let prefix_len = prefix.len();

        // Use range scanning instead of prefix scanning for much better performance
        // Create a range that starts with the table prefix and ends just before the next table
        let start_key = prefix.as_bytes();
        let mut end_key = prefix.as_bytes().to_vec();

        // Increment the last byte to create an exclusive upper bound
        // This ensures we only scan keys for this specific table
        if let Some(last_byte) = end_key.last_mut() {
            if *last_byte < 255 {
                *last_byte += 1;
            } else {
                // Handle edge case where last byte is 255
                end_key.push(0);
            }
        } else {
            // Fallback to prefix scanning if we can't create range bounds
            let result_set = self
                .tree
                .scan_prefix(prefix.as_bytes())
                .map(move |item| {
                    let (key, value) = item.as_storage_err()?;
                    let key = key[prefix_len..key.len()].to_owned();
                    let snapshot: Snapshot<DataRow> =
                        bitcode::deserialize(&value).as_storage_err()?;
                    let row = snapshot.take(self.state);
                    let item = row.map(|row| (Key::Bytea(key), row));

                    Ok(item)
                })
                .filter_map(|item| item.transpose());
            return Ok(Box::pin(iter(result_set)));
        }

        let result_set = self
            .tree
            .range(start_key..end_key.as_slice())
            .map(move |item| {
                let (key, value) = item.as_storage_err()?;
                let key = key[prefix_len..key.len()].to_owned();
                let snapshot: Snapshot<DataRow> = bitcode::deserialize(&value).as_storage_err()?;
                let row = snapshot.take(self.state);
                let item = row.map(|row| (Key::Bytea(key), row));

                Ok(item)
            })
            .filter_map(|item| item.transpose());
        Ok(Box::pin(iter(result_set)))
    }
}

use gluesql_core::error::Error;

impl<'a> DbConn<'a> {
    pub async fn pk_range(
        &self,
        table_name: &str,
        pkey_min: Key,
        pkey_max: Key,
    ) -> Result<Vec<Vec<Value>>> {
        let start = super::sled_key(table_name, pkey_min)?;
        let end = super::sled_key(table_name, pkey_max)?;
        self.tree
            .range(start..end)
            .map(move |item| {
                let (_, value) = item.as_storage_err()?;
                let snapshot: Snapshot<DataRow> = bitcode::deserialize(&value).as_storage_err()?;
                let Some(row) = snapshot.take(self.state) else {
                    return Err(Error::StorageMsg("unexpected DataRow variant".to_owned()));
                };
                let DataRow::Vec(values) = row else {
                    return Err(Error::StorageMsg("unexpected DataRow variant".to_owned()));
                };

                Ok(values)
            })
            .collect()
    }
}
