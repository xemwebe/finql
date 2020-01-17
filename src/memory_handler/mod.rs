use crate::asset::Asset;
use crate::data_handler::{DataError, DataHandler};
use crate::transaction::Transaction;
///! Implementation of in-memory data handler
use std::collections::BTreeMap;

/// Struct to store data in memory
pub struct InMemoryDB {
    assets: BTreeMap<usize, Asset>,
    next_asset_id: usize,
    transactions: BTreeMap<usize, Transaction>,
    next_transaction_id: usize,
}

impl InMemoryDB {
    pub fn new() -> InMemoryDB {
        InMemoryDB {
            assets: BTreeMap::new(),
            next_asset_id: 0,
            transactions: BTreeMap::new(),
            next_transaction_id: 0,
        }
    }

    fn get_by_id<T: Clone>(id: usize, map: &BTreeMap<usize, T>) -> Result<T, DataError> {
        let object = map.get(&id);
        match object {
            Some(object) => Ok(object.clone()),
            None => Err(DataError::NotFound(
                "asset id not found in database".to_string(),
            )),
        }
    }

    fn get_all<T: Clone>(map: &BTreeMap<usize, T>) -> Result<Vec<T>, DataError> {
        let mut objects = Vec::new();
        for object in map.values() {
            objects.push(object.clone());
        }
        Ok(objects)
    }
}

/// Handler for globally available data
impl DataHandler for InMemoryDB {
    // insert, get, update and delete for assets
    fn insert_asset(&mut self, asset: &Asset) -> Result<usize, DataError> {
        let id = self.next_asset_id;
        let mut asset = asset.clone();
        asset.id = Some(id);
        self.assets.insert(id, asset);
        self.next_transaction_id += 1;
        Ok(id)
    }

    fn get_asset_by_id(&self, id: usize) -> Result<Asset, DataError> {
        Self::get_by_id(id, &self.assets)
    }

    fn get_all_assets(&self) -> Result<Vec<Asset>, DataError> {
        Self::get_all(&self.assets)
    }

    fn update_asset(&mut self, asset: &Asset) -> Result<(), DataError> {
        match asset.id {
            None => Err(DataError::UpdateFailed(
                "asset has no id yet, can't update new value".to_string(),
            )),
            Some(id) => {
                if !self.assets.contains_key(&id) {
                    return Err(DataError::NotFound(
                        "asset id not found in database".to_string(),
                    ));
                }
                self.assets.insert(id, asset.clone());
                Ok(())
            }
        }
    }

    fn delete_asset(&mut self, id: usize) -> Result<(), DataError> {
        if !self.assets.contains_key(&id) {
            return Err(DataError::NotFound(
                "asset id not found in database".to_string(),
            ));
        }
        self.assets.remove(&id);
        Ok(())
    }

    // insert, get, update and delete for transactions
    fn insert_transaction(&mut self, transaction: &Transaction) -> Result<usize, DataError> {
        let id = self.next_transaction_id;
        let mut transaction = transaction.clone();
        transaction.id = Some(id);
        self.transactions.insert(id, transaction);
        self.next_transaction_id += 1;
        Ok(id)
    }

    fn get_transaction_by_id(&self, id: usize) -> Result<Transaction, DataError> {
        Self::get_by_id(id, &self.transactions)
    }

    fn get_all_transactions(&self) -> Result<Vec<Transaction>, DataError> {
        Self::get_all(&self.transactions)
    }

    fn update_transaction(&mut self, transaction: &Transaction) -> Result<(), DataError> {
        match transaction.id {
            None => Err(DataError::UpdateFailed(
                "transaction has no id yet, can't update new value".to_string(),
            )),
            Some(id) => {
                if !self.transactions.contains_key(&id) {
                    return Err(DataError::NotFound(
                        "transaction id not found in database".to_string(),
                    ));
                }
                self.transactions.insert(id, transaction.clone());
                Ok(())
            }
        }
    }

    fn delete_transaction(&mut self, id: usize) -> Result<(), DataError> {
        if !self.transactions.contains_key(&id) {
            return Err(DataError::NotFound(
                "transaction id not found in database".to_string(),
            ));
        }
        self.transactions.remove(&id);
        Ok(())
    }
}
