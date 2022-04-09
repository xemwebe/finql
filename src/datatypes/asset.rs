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
    fn get_id(&self) -> Result<i32, DataError> {
        match self {
            Asset::Currency(c) => c.get_id(),
            Asset::Stock(s) => s.get_id(),
        }
    }

    // set id or return error if id has already been set
    fn set_id(&mut self, id: i32) -> Result<(), DataError> {
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



#[cfg(test)]
mod tests {
    use super::*;
    use crate::datatypes::CurrencyISOCode;

    #[test]
    fn set_asset_id() {
        let mut asset = Asset::Currency(Currency::new(None, CurrencyISOCode::new("EUR").unwrap(), Some(4)));
        asset.set_id(1).unwrap();
        assert_eq!(asset.get_id().unwrap(), 1);


        let aus = Currency::new(None, CurrencyISOCode::new("AUS").unwrap(), Some(2));
        let mut aus_asset = Asset::Currency(aus);
        aus_asset.set_id(1).unwrap();
        assert_eq!(aus_asset.get_id().unwrap(), 1);
        assert!(aus.get_id().is_err());
    }
}