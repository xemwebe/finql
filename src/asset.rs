///! Implementation of a container for basic asset data
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCategory {
    id: usize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Option<usize>,
    pub name: String,
    pub wkn: Option<String>,
    pub isin: Option<String>,
    pub note: Option<String>,
}

impl Asset {
    pub fn new(
        id: Option<usize>,
        name: &str,
        wkn: Option<String>,
        isin: Option<String>,
        note: Option<String>,
    ) -> Asset {
        Asset {
            id,
            name: name.to_string(),
            wkn,
            isin,
            note,
        }
    }
}
