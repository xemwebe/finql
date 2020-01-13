///! Implementation of portfolio 

use serde::{Serialize,Deserialize};


/// Type of transaction
#[derive(Debug,Serialize,Deserialize)]
pub struct Portfolio {
    id: u64,
    name: String,
}
