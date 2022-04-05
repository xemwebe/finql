use async_trait::async_trait;

use super::{DataError, Asset, Currency, CurrencyISOCode};

/// Handler for globally available data of transactions and related data
#[async_trait]
pub trait AssetHandler {
    // insert, get, update and delete for assets
    async fn insert_asset(&self, asset: &Asset) -> Result<usize, DataError>;
    async fn get_asset_id(&self, asset: &Asset) -> Option<usize>;
    async fn get_asset_by_id(&self, id: usize) -> Result<Asset, DataError>;
    async fn get_asset_by_isin(&self, id: &str) -> Result<Asset, DataError>;
    /// Return a list of all assets ordered by name
    async fn get_all_assets(&self) -> Result<Vec<Asset>, DataError>;
    async fn update_asset(&self, asset: &Asset) -> Result<(), DataError>;
    async fn delete_asset(&self, id: usize) -> Result<(), DataError>;
    /// We assume here that a currency is an Asset with a three letter name and no ISIN nor WKN
    async fn get_all_currencies(&self) -> Result<Vec<Currency>, DataError>;
    /// Either read currency from database or create new currency and store it in database with default rounding digits
    async fn get_or_new_currency(&self, iso_code: CurrencyISOCode) -> Result<Currency, DataError>;
    /// Either read currency from database or create new currency and store it in database.
    /// If currency exists already, ignore rounding errors.
    async fn get_or_new_currency_with_digits(&self, iso_code: CurrencyISOCode, rounding_digits: i32) -> Result<Currency, DataError>;
}
