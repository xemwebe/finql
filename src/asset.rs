///! Implementation of a container for basic asset data

use serde::{Serialize,Deserialize};

#[derive(Debug,Serialize,Deserialize)]
pub enum AssetType {
    Equity,
    ETF,
    Fonds,
    FixedIncome,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Asset {
    id: u64,
    pub asset_type: AssetType,
    pub name: String,
    pub wkn: String,
    pub isin: String,
    pub note: String,
}

