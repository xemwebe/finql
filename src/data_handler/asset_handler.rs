use super::DataError;
use crate::asset::Asset;
use crate::currency::Currency;

/// Handler for globally available data of transactions and related data
pub trait AssetHandler {
    // insert, get, update and delete for assets
    fn insert_asset(&mut self, asset: &Asset) -> Result<usize, DataError>;
    fn insert_asset_if_new(
        &mut self,
        asset: &Asset,
        rename_asset: bool,
    ) -> Result<usize, DataError> {
        match self.get_asset_id(asset) {
            Some(id) => Ok(id),
            None => match self.insert_asset(asset) {
                Ok(id) => Ok(id),
                Err(err) => {
                    if rename_asset {
                        let new_name = format!("{} (NEW)", asset.name);
                        self.insert_asset(&Asset {
                            id: None,
                            name: new_name,
                            wkn: asset.wkn.clone(),
                            isin: asset.isin.clone(),
                            note: asset.note.clone(),
                        })
                    } else {
                        Err(err)
                    }
                }
            },
        }
    }
    fn get_asset_id(&mut self, asset: &Asset) -> Option<usize>;
    fn get_asset_by_id(&mut self, id: usize) -> Result<Asset, DataError>;
    fn get_asset_by_isin(&mut self, id: &String) -> Result<Asset, DataError>;
    /// Return a list of all assets ordered by name 
    fn get_all_assets(&mut self) -> Result<Vec<Asset>, DataError>;
    fn update_asset(&mut self, asset: &Asset) -> Result<(), DataError>;
    fn delete_asset(&mut self, id: usize) -> Result<(), DataError>;
    /// We assume here that a currency is an Asset with a three letter name and no ISIN nor WKN
    fn get_all_currencies(&mut self) -> Result<Vec<Currency>, DataError>;
}
