use super::{DataError, DataItem};
///! Implementation of a container for basic asset data
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stock {
    pub id: Option<usize>,
    pub name: String,
    pub isin: Option<String>,
    pub wkn: Option<String>,
    pub note: Option<String>
}

impl Stock {
    pub fn new(
        id: Option<usize>,
        name: String,
        isin: Option<String>,
        wkn: Option<String>,
        note: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            isin,
            wkn,
            note,
        }
    }
}

impl DataItem for Stock {
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
