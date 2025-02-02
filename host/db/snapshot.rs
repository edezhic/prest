use {
    super::WriteState,
    crate::{Deserialize, Serialize},
    std::fmt::Debug,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot<T> {
    pub data_txid: u64,
    pub data: Option<T>,
    pub backup_txid: Option<u64>,
    pub backup: Option<T>,
}

impl<T: Clone + Debug> Snapshot<T> {
    pub fn new(txid: u64, data: T) -> Self {
        Self {
            data: Some(data),
            data_txid: txid,
            backup: None,
            backup_txid: None,
        }
    }

    pub fn update(&mut self, state: WriteState, data: T) {
        // if last modification was in a previous tx then backup current data
        if self.data_txid < state.tx_id {
            self.backup = self.data.take();
            self.backup_txid = Some(self.data_txid);
        }
        self.data = Some(data);
        self.data_txid = state.tx_id;
    }

    // pub fn update_cell(&mut self, state: WriteState, value: sql::Value) {
    //     // if last modification was in a previous tx then backup current data
    //     if self.data_txid < state.tx_id {
    //         self.backup = self.data.take();
    //         self.backup_txid = Some(self.data_txid);
    //     }
    //     self.data = Some(data);
    //     self.data_txid = state.tx_id;
    // }

    pub fn delete(mut self, state: WriteState) -> Option<Self> {
        // if last modification was in a previous tx
        if self.data_txid < state.tx_id {
            // and it has some data - back it up
            if let Some(data) = self.data.take() {
                self.backup = Some(data);
                self.backup_txid = Some(self.data_txid);
                return Some(self);
            }
            // and there is no data - remove
            else {
                return None;
            }
        }
        // if last modification was by current tx
        else {
            // and it has a backup
            if self.backup.is_some() {
                self.data = None;
                self.data_txid = state.tx_id;
                return Some(self);
            }
            // and no backup (just created)
            else {
                return None;
            }
        }
    }

    // -> Option(should update)<Option(new value)<Self>>
    pub fn rollback(mut self, state: WriteState) -> Option<Option<Self>> {
        // nothing if modified by previous tx
        if self.data_txid < state.tx_id {
            return None;
        }

        // if there is a backup
        if let Some(backup) = self.backup.take() {
            self.data = Some(backup);
            self.data_txid = self
                .backup_txid
                .take()
                .expect("backups must be stored with txid");
            Some(Some(self))
        }
        // just created - remove
        else {
            Some(None)
        }
    }

    pub fn take(mut self, state: WriteState) -> Option<T> {
        if state.in_progress && state.tx_id == self.data_txid {
            self.backup.take()
        } else {
            self.data.take()
        }
    }

    pub fn get(&self, state: WriteState) -> Option<T> {
        if state.in_progress && state.tx_id == self.data_txid {
            self.backup.clone().take()
        } else {
            self.data.clone().take()
        }
    }

    pub fn get_mut(&mut self, state: WriteState) -> Option<&mut T> {
        if let Some(data) = &mut self.data {
            Some(data)
        } else {
            None
        }
    }
}
