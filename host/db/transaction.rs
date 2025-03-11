use {
    super::{AsStorageError, DbConn, Snapshot, WRITE_STATE_KEY},
    async_trait::async_trait,
    gluesql_core::{
        error::{Error, Result},
        store::{DataRow, Transaction},
    },
};

#[async_trait(?Send)]
impl<'a> Transaction for DbConn<'a> {
    async fn begin(&mut self, autocommit: bool) -> Result<bool> {
        if !autocommit {
            return Err(Error::StorageMsg(
                "nested and non-autocommitted transactions are not supported".to_owned(),
            ));
        }

        if !self.readonly {
            self.state.in_progress = true;
            if let Err(e) = bitcode::serialize(&self.state)
                .as_storage_err()
                .map(|state| self.tree.insert(WRITE_STATE_KEY, state))
            {
                crate::error!("failed to update latest transaction id: {e}");
                return Err(Error::StorageMsg(
                    "failed to record tx beginning".to_owned(),
                ));
            }
        }

        Ok(true)
    }

    async fn rollback(&mut self) -> Result<()> {
        if !self.readonly {
            self.rollback_self()?;
            // CURRENT_TXID.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    }

    async fn commit(&mut self) -> Result<()> {
        if !self.readonly {
            self.state.in_progress = false;
            if let Err(e) = bitcode::serialize(&self.state)
                .as_storage_err()
                .map(|state| self.tree.insert(WRITE_STATE_KEY, state))
            {
                crate::error!("failed to update latest transaction id: {e}")
            }
        }
        Ok(())
    }
}

impl<'a> DbConn<'a> {
    pub fn rollback_self(&self) -> Result<()> {
        let mut affected = 0;
        for item in self.tree.scan_prefix(b"data/") {
            let (key, value) = item.as_storage_err()?;
            let snapshot = bitcode::deserialize::<Snapshot<DataRow>>(&value).as_storage_err()?;
            if let Some(v) = snapshot.rollback(self.state) {
                affected += 1;
                if let Some(restored) = v {
                    let restored = bitcode::serialize(&restored).as_storage_err()?;
                    self.tree.insert(key, restored).as_storage_err()?;
                } else {
                    self.tree.remove(key).as_storage_err()?;
                }
            }
        }
        crate::warn!("rolled back {affected} affected rows");
        Ok(())
    }
}
