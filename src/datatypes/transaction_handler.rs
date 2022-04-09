use async_trait::async_trait;

use super::AssetHandler;
use super::DataError;
use super::Transaction;

/// Handler for globally available data of transactions and related data
#[async_trait]
pub trait TransactionHandler: AssetHandler {
    // insert, get, update and delete for transactions
    async fn insert_transaction(&self, transaction: &Transaction) -> Result<i32, DataError>;
    async fn get_transaction_by_id(&self, id: i32) -> Result<Transaction, DataError>;
    async fn get_all_transactions(&self) -> Result<Vec<Transaction>, DataError>;
    async fn update_transaction(&self, transaction: &Transaction) -> Result<(), DataError>;
    async fn delete_transaction(&self, id: i32) -> Result<(), DataError>;
}
