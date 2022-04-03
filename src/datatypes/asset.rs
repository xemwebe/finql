///! Implementation of a container for basic asset data
use serde::{Deserialize, Serialize};

use super::{Currency, Stock};

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

// impl DataItem for Asset {
//     // get id or return error if id hasn't been set yet
//     fn get_id(&self) -> Result<usize, DataError> {
//         match self.id {
//             Some(id) => Ok(id),
//             None => Err(DataError::DataAccessFailure(
//                 "tried to get id of temporary asset".to_string(),
//             )),
//         }
//     }
//     // set id or return error if id has already been set
//     fn set_id(&mut self, id: usize) -> Result<(), DataError> {
//         match self.id {
//             Some(_) => Err(DataError::DataAccessFailure(
//                 "tried to change valid asset id".to_string(),
//             )),
//             None => {
//                 self.id = Some(id);

//                 match &mut self.resource {
//                     Resource::Currency(c) => {
//                         c.id = Some(id);
//                     },
//                     Resource::Stock(s) => {
//                         s.id = Some(id);
//                     }
//                 }

//                 Ok(())
//             }
//         }
//     }
// }
