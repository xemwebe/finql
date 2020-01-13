///! Implementation of basic transaction types

use serde::{Serialize,Deserialize};
use crate::fixed_income::{CashFlow};

/// Type of transaction
#[derive(Debug,Serialize,Deserialize)]
pub enum TransactionType {
    Buy,
    Sell,
    CashIn,
    CashOut,
    Dividend,
    Tax,
    Interest,
    Fee,
}

/// Basic transaction data
#[derive(Debug,Serialize,Deserialize)]
pub struct Transaction {
    id: u64,
    pub trans_type: TransactionType,
    asset: u64,
    pub cash_flow: CashFlow,
    related_trans: Option<u64>,
    pub position: Option<f64>,
    pub portfolio: Option<u64>,
    pub note: Option<String>,
}
