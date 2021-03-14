///! Implementation of portfolio
use serde::{Deserialize, Serialize};

/// Type of transaction
#[derive(Debug, Serialize, Deserialize)]
pub struct Portfolio {
    id: u64,
    name: String,
}
