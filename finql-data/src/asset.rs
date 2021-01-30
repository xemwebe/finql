use super::{DataError, DataItem};
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

impl DataItem for Asset {
    // get id or return error if id hasn't been set yet
    fn get_id(&self) -> Result<usize, DataError> {
        match self.id {
            Some(id) => Ok(id),
            None => Err(DataError::DataAccessFailure(
                "tried to get id of temporary asset".to_string(),
            )),
        }
    }
    // set id or return error if id has already been set
    fn set_id(&mut self, id: usize) -> Result<(), DataError> {
        match self.id {
            Some(_) => Err(DataError::DataAccessFailure(
                "tried to change valid asset id".to_string(),
            )),
            None => {
                self.id = Some(id);
                Ok(())
            }
        }
    }
}
