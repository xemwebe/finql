use async_trait::async_trait;

use super::DataError;
use super::Asset;
use super::Currency;

/// Handler for globally available data of transactions and related data
#[async_trait]
pub trait AssetHandler {
    // insert, get, update and delete for assets
    async fn insert_asset(&self, asset: &Asset) -> Result<usize, DataError>;
    async fn insert_asset_if_new(
        &self,
        asset: &Asset,
        rename_asset: bool,
    ) -> Result<usize, DataError>;
    
    async fn get_asset_id(&self, asset: &Asset) -> Option<usize>;
    async fn get_asset_by_id(&self, id: usize) -> Result<Asset, DataError>;
    async fn get_asset_by_isin(&self, id: &str) -> Result<Asset, DataError>;
    /// Return a list of all assets ordered by name
    async fn get_all_assets(&self) -> Result<Vec<Asset>, DataError>;
    async fn update_asset(&self, asset: &Asset) -> Result<(), DataError>;
    async fn delete_asset(&self, id: usize) -> Result<(), DataError>;
    /// We assume here that a currency is an Asset with a three letter name and no ISIN nor WKN
    async fn get_all_currencies(&self) -> Result<Vec<Currency>, DataError>;
}
