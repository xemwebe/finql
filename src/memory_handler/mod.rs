///! Implementation of in-memory data handler
use crate::asset::Asset;
use crate::data_handler::{DataError, DataItem};
use crate::quote::{MarketDataSource, Quote, Ticker};
use crate::transaction::Transaction;
use std::collections::BTreeMap;

mod asset_handler;
mod quote_handler;
mod transaction_handler;

struct InMemoryContainer<T: DataItem + Clone> {
    next_id: usize,
    items: BTreeMap<usize, T>,
}

impl<T: DataItem + Clone> InMemoryContainer<T> {
    fn new() -> InMemoryContainer<T> {
        InMemoryContainer {
            next_id: 0,
            items: BTreeMap::new(),
        }
    }

    fn insert(&mut self, item: &T) -> Result<usize, DataError> {
        let id = self.next_id;
        let mut item = item.clone();
        item.set_id(id)?;
        self.items.insert(id, item);
        self.next_id += 1;
        Ok(id)
    }

    fn get_by_id(&self, id: usize) -> Result<T, DataError> {
        let item = self.items.get(&id);
        match item {
            Some(item) => Ok(item.clone()),
            None => Err(DataError::NotFound(
                "item id not found in database".to_string(),
            )),
        }
    }

    fn get_all(&self) -> Result<Vec<T>, DataError> {
        let mut items = Vec::new();
        for item in self.items.values() {
            items.push(item.clone());
        }
        Ok(items)
    }

    fn update(&mut self, item: &T) -> Result<(), DataError> {
        let id = item.get_id()?;
        if !self.items.contains_key(&id) {
            return Err(DataError::UpdateFailed(
                "item id not found in database".to_string(),
            ));
        }
        self.items.insert(id, item.clone());
        Ok(())
    }

    fn delete(&mut self, id: usize) -> Result<(), DataError> {
        if !self.items.contains_key(&id) {
            return Err(DataError::DeleteFailed(
                "item id not found in database".to_string(),
            ));
        }
        self.items.remove(&id);
        Ok(())
    }
}

/// Struct to store data in memory
pub struct InMemoryDB {
    assets: InMemoryContainer<Asset>,
    transactions: InMemoryContainer<Transaction>,
    md_sources: InMemoryContainer<MarketDataSource>,
    ticker_map: InMemoryContainer<Ticker>,
    quotes: InMemoryContainer<Quote>,
}

impl InMemoryDB {
    pub fn new() -> InMemoryDB {
        InMemoryDB {
            assets: InMemoryContainer::new(),
            transactions: InMemoryContainer::new(),
            md_sources: InMemoryContainer::new(),
            ticker_map: InMemoryContainer::new(),
            quotes: InMemoryContainer::new(),
        }
    }
}
