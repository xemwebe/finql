use super::{DataError, DataItem};
///! Implementation of a container for basic asset data
use serde::{Deserialize, Serialize};

use crate::{Currency, CurrencyISOCode, Stock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Resource {
    Currency(Currency),
    Stock(Stock),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Option<usize>,
    pub name: String,
    pub note: Option<String>,
    pub resource: Resource,
}

impl Asset {
    pub fn new_currency(
        id: Option<usize>,
        name: String,
        note: Option<String>,
        iso_code: CurrencyISOCode,
        rounding_digits: i32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            note,
            resource: Resource::Currency(Currency::new(
                id,
                iso_code,
                Some(rounding_digits),
            )),
        }
    }

    pub fn new_stock(
        id: Option<usize>,
        name: String,
        note: Option<String>,
        isin: String,
        wkn: Option<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            note,
            resource: Resource::Stock(Stock::new(
                id,
                isin,
                wkn
            ))
        }
    }

    pub fn currency(&self) -> Result<Currency, DataError> {
        if let Resource::Currency(c) = &self.resource {
            Ok(c.to_owned())
        }
        else {
            Err(DataError::InvalidResource)
        }
    }

    pub fn stock(&self) -> Result<Stock, DataError> {
        if let Resource::Stock(s) = &self.resource {
            Ok(s.to_owned())
        }
        else {
            Err(DataError::InvalidResource)
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

                match &mut self.resource {
                    Resource::Currency(c) => {
                        c.id = Some(id);
                    },
                    Resource::Stock(s) => {
                        s.id = Some(id);
                    }
                }

                Ok(())
            }
        }
    }
}
