use crate::fixed_income::CashFlow;
///! Implementation of basic transaction types
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

/// Type of transaction
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Buy = 1,
    Sell = 2,
    CashIn = 3,
    CashOut = 4,
    Dividend = 5,
    Tax = 6,
    Interest = 7,
    Fee = 8,
}

#[derive(Debug)]
pub struct InvalidTransactionType;

impl std::error::Error for InvalidTransactionType {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl fmt::Display for InvalidTransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid transaction type")
    }
}

impl TryFrom<u8> for TransactionType {
    type Error = InvalidTransactionType;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            1 => Ok(TransactionType::Buy),
            2 => Ok(TransactionType::Sell),
            3 => Ok(TransactionType::CashIn),
            4 => Ok(TransactionType::CashOut),
            5 => Ok(TransactionType::Dividend),
            6 => Ok(TransactionType::Tax),
            7 => Ok(TransactionType::Interest),
            8 => Ok(TransactionType::Fee),
            _ => Err(InvalidTransactionType),
        }
    }
}

/// Basic transaction data
#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Option<usize>,
    pub trans_type: TransactionType,
    pub asset: Option<usize>,
    pub cash_flow: CashFlow,
    pub related_trans: Option<usize>,
    pub position: Option<f64>,
    pub note: Option<String>,
}
