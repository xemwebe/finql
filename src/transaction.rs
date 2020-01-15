use crate::fixed_income::CashFlow;
///! Implementation of basic transaction types
use serde::{Deserialize, Serialize};

/// Type of transaction
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Cash,
    Asset { asset_id: usize, position: f64 },
    Dividend { asset_id: usize },
    Interest { asset_id: usize },
    Tax { transaction_ref: Option<usize> },
    Fee { transaction_ref: Option<usize> },
}

/// Basic transaction data
#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    // Before a transaction is stored to a database, the id maybe None
    pub id: Option<usize>,
    pub transaction_type: TransactionType,
    pub cash_flow: CashFlow,
    pub note: Option<String>,
}
