use super::InMemoryDB;
use crate::asset::Asset;
use crate::data_handler::{DataError, DataHandler};
use crate::transaction::Transaction;

fn some_equal(opt: &Option<String>, s: &str) -> bool {
    match opt {
        None => false,
        Some(opt_s) => &opt_s == &s,
    }
}

/// Handler for globally available data
impl DataHandler for InMemoryDB {
    // insert, get, update and delete for assets
    fn insert_asset(&mut self, asset: &Asset) -> Result<usize, DataError> {
        self.assets.insert(asset)
    }

    fn get_asset_id(&mut self, asset: &Asset) -> Option<usize> {
        if let Some(isin) = &asset.isin {
            for (id, a) in &self.assets.items {
                if some_equal(&a.isin, &isin) {
                    return Some(*id);
                }
            }
        } else if let Some(wkn) = &asset.wkn {
            for (id, a) in &self.assets.items {
                if some_equal(&a.wkn, &wkn) {
                    return Some(*id);
                }
            }
        } else {
            for (id, a) in &self.assets.items {
                if a.name == asset.name {
                    return Some(*id);
                }
            }
        }
        None
    }

    fn get_asset_by_id(&mut self, id: usize) -> Result<Asset, DataError> {
        self.assets.get_by_id(id)
    }

    fn get_all_assets(&mut self) -> Result<Vec<Asset>, DataError> {
        self.assets.get_all()
    }

    fn update_asset(&mut self, asset: &Asset) -> Result<(), DataError> {
        self.assets.update(asset)
    }

    fn delete_asset(&mut self, id: usize) -> Result<(), DataError> {
        self.assets.delete(id)
    }

    // insert, get, update and delete for transactions
    fn insert_transaction(&mut self, transaction: &Transaction) -> Result<usize, DataError> {
        self.transactions.insert(transaction)
    }

    fn get_transaction_by_id(&mut self, id: usize) -> Result<Transaction, DataError> {
        self.transactions.get_by_id(id)
    }

    fn get_all_transactions(&mut self) -> Result<Vec<Transaction>, DataError> {
        self.transactions.get_all()
    }

    fn update_transaction(&mut self, transaction: &Transaction) -> Result<(), DataError> {
        self.transactions.update(transaction)
    }

    fn delete_transaction(&mut self, id: usize) -> Result<(), DataError> {
        self.transactions.delete(id)
    }
}
