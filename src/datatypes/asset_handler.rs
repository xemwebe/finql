use async_trait::async_trait;

use super::{Asset, AssetSelector, Currency, CurrencyISOCode, DataError};

/// Handler for globally available data of transactions and related data
#[async_trait]
pub trait AssetHandler {
    // insert, get, update and delete for assets
    async fn insert_asset(&self, asset: &Asset) -> Result<i32, DataError>;
    async fn get_asset_id(&self, asset: &Asset) -> Option<i32>;
    async fn get_asset_by_id(&self, id: i32) -> Result<Asset, DataError>;
    async fn get_asset_by_isin(&self, id: &str) -> Result<Asset, DataError>;
    /// Return a list of all assets ordered by name
    async fn get_all_assets(&self) -> Result<Vec<Asset>, DataError>;
    /// Return AssetSelector for all assets
    async fn get_asset_list(&self) -> Result<Vec<AssetSelector>, DataError>;
    async fn update_asset(&self, asset: &Asset) -> Result<(), DataError>;
    async fn delete_asset(&self, id: i32) -> Result<(), DataError>;
    async fn get_all_currencies(&self) -> Result<Vec<Currency>, DataError>;
    /// Get a list of currencies as list of AssetSelectors
    async fn get_currency_list(&self) -> Result<Vec<AssetSelector>, DataError>;
    /// Either read currency from database or create new currency and store it in database with default rounding digits
    async fn get_or_new_currency(&self, iso_code: CurrencyISOCode) -> Result<Currency, DataError>;
    /// Either read currency from database or create new currency and store it in database.
    /// If currency exists already, ignore rounding errors.
    async fn get_or_new_currency_with_digits(
        &self,
        iso_code: CurrencyISOCode,
        rounding_digits: i32,
    ) -> Result<Currency, DataError>;
}
