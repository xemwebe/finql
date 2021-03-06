use async_trait::async_trait;

use super::AssetHandler;
use super::DataError;
use crate::transaction::Transaction;

/// Handler for globally available data of transactions and related data
#[async_trait]
pub trait TransactionHandler: AssetHandler {
    // insert, get, update and delete for transactions
    async fn insert_transaction(&self, transaction: &Transaction) -> Result<usize, DataError>;
    async fn get_transaction_by_id(&self, id: usize) -> Result<Transaction, DataError>;
    async fn get_all_transactions(&self) -> Result<Vec<Transaction>, DataError>;
    async fn update_transaction(&self, transaction: &Transaction) -> Result<(), DataError>;
    async fn delete_transaction(&self, id: usize) -> Result<(), DataError>;
}
