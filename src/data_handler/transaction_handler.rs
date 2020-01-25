use super::DataError;
use crate::asset::Asset;
use crate::transaction::Transaction;

/// Handler for globally available data of transactions and related data
pub trait DataHandler {
    // insert, get, update and delete for assets
    fn insert_asset(&mut self, asset: &Asset) -> Result<usize, DataError>;
    fn get_asset_by_id(&mut self, id: usize) -> Result<Asset, DataError>;
    fn get_all_assets(&mut self) -> Result<Vec<Asset>, DataError>;
    fn update_asset(&mut self, asset: &Asset) -> Result<(), DataError>;
    fn delete_asset(&mut self, id: usize) -> Result<(), DataError>;

    // insert, get, update and delete for transactions
    fn insert_transaction(&mut self, transaction: &Transaction) -> Result<usize, DataError>;
    fn get_transaction_by_id(&mut self, id: usize) -> Result<Transaction, DataError>;
    fn get_all_transactions(&mut self) -> Result<Vec<Transaction>, DataError>;
    fn update_transaction(&mut self, transaction: &Transaction) -> Result<(), DataError>;
    fn delete_transaction(&mut self, id: usize) -> Result<(), DataError>;
}
