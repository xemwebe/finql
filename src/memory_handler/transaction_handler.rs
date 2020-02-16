use super::InMemoryDB;
use crate::data_handler::{DataError, TransactionHandler};
use crate::transaction::Transaction;

/// Handler for globally available data
impl TransactionHandler for InMemoryDB {
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
