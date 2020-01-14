///! Implementation of a data handler trait to deal with global data

use std::fmt;
use crate::asset::Asset;

#[derive(Debug)]
pub enum DataError {
    DataAccessFailure(String),
    NotFound(String),
    UpdateFailed(String),
    DeleteFailed(String),
    InsertFailed(String),
}

impl std::error::Error for DataError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(self)
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataAccessFailure(err) => write!(f, "connection to database failed: {}", err),
            Self::NotFound(err) => write!(f, "could found request object in database: {}", err),
            Self::UpdateFailed(err) => write!(f, "update of object in database failed: {}", err),
            Self::DeleteFailed(err) => write!(f, "removing object from database failed: {}", err),
            Self::InsertFailed(err) => write!(f, "inserting object to database failed: {}", err),
        }
    }
}

/// Handler for globally available data
pub trait DataHandler {
    fn get_asset_by_id(&self, id: u64) -> Result<Asset, DataError>;
    fn update_asset(&self, asset: &Asset) -> Result<(), DataError>;
    fn insert_asset(&self, asset: &Asset) -> Result<u64, DataError>;
    fn delete_asset(&self, id: u64) -> Result<(), DataError>;
}
