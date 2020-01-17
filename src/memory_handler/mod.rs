///! Implementation of in-memory data handler

use crate::asset::Asset;
use crate::data_handler::DataError;
use crate::transaction::Transaction;
use std::collections::BTreeMap;

/// Struct to store data in memory
pub struct InMemoryDB {
    assets: BTreeMap<usize, Asset>,
    next_asset_id: usize,
    transactions: BTreeMap<usize, Transaction>,
    next_transaction_id: usize,
}

impl InMemoryDB {
    pub fn new() -> InMemoryDB {
        InMemoryDB {
            assets: BTreeMap::new(),
            next_asset_id: 0,
            transactions: BTreeMap::new(),
            next_transaction_id: 0,
        }
    }

    fn get_by_id<T: Clone>(id: usize, map: &BTreeMap<usize, T>) -> Result<T, DataError> {
        let object = map.get(&id);
        match object {
            Some(object) => Ok(object.clone()),
            None => Err(DataError::NotFound(
                "asset id not found in database".to_string(),
            )),
        }
    }

    fn get_all<T: Clone>(map: &BTreeMap<usize, T>) -> Result<Vec<T>, DataError> {
        let mut objects = Vec::new();
        for object in map.values() {
            objects.push(object.clone());
        }
        Ok(objects)
    }
}

pub mod transaction_handler;

