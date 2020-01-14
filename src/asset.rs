///! Implementation of a container for basic asset data

use serde::{Serialize,Deserialize};

#[derive(Debug,Serialize,Deserialize)]
pub struct AssetCategory {
    id: u64,
    pub name: String,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Asset {
    pub id: Option<u64>,
    pub name: String,
    pub wkn: Option<String>,
    pub isin: Option<String>,
    pub note: Option<String>,
}

impl Asset {
    pub fn new(id: Option<u64>, name: &str, wkn: Option<String>, isin: Option<String>, note: Option<String>) -> Asset {
        Asset{ id, name: name.to_string(), wkn, isin, note }
    }
}