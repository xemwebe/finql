///! Implementation of a container for basic asset data
use serde::{Deserialize, Serialize};

use super::{Currency, Stock, DataItem, DataError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Asset {
    Currency(Currency),
    Stock(Stock),
}

impl Asset {
    pub fn class(&self) -> String {
        match self {
            Self::Currency(_) => "currency".into(),
            Self::Stock(_) => "stock".into(),
        }
    }
}

impl DataItem for Asset {
    // get id or return error if id hasn't been set yet
    fn get_id(&self) -> Result<usize, DataError> {
        match self {
            Asset::Currency(c) => c.get_id(),
            Asset::Stock(s) => s.get_id(),
        }
    }

    // set id or return error if id has already been set
    fn set_id(&mut self, id: usize) -> Result<(), DataError> {
        *self = match &*self {
            Asset::Currency(c) => {
                let mut c = c.clone();
                c.set_id(id)?;
                Asset::Currency(c)
            }
            Asset::Stock(s) => {
                let mut s = s.clone();
                s.set_id(id)?;
                Asset::Stock(s)
            }
        };
        Ok(())
    }
}
