use super::InMemoryDB;
use crate::asset::Asset;
use crate::data_handler::{AssetHandler, DataError};
use crate::helpers::some_equal;

/// Handler for globally available data
impl AssetHandler for InMemoryDB {
    // insert, get, update and delete for assets
    fn insert_asset(&mut self, asset: &Asset) -> Result<usize, DataError> {
        self.assets.insert(asset)
    }

    fn get_asset_id(&mut self, asset: &Asset) -> Option<usize> {
        if let Some(isin) = &asset.isin {
            for (id, a) in &self.assets.items {
                if some_equal(&a.isin, &isin) {
                    return Some(*id);
                }
            }
        } else if let Some(wkn) = &asset.wkn {
            for (id, a) in &self.assets.items {
                if some_equal(&a.wkn, &wkn) {
                    return Some(*id);
                }
            }
        } else {
            for (id, a) in &self.assets.items {
                if a.name == asset.name {
                    return Some(*id);
                }
            }
        }
        None
    }

    fn get_asset_by_id(&mut self, id: usize) -> Result<Asset, DataError> {
        self.assets.get_by_id(id)
    }

    fn get_asset_by_isin(&mut self, isin: &String) -> Result<Asset, DataError> {
        for (_, a) in &self.assets.items {
            if some_equal(&a.isin, &isin) {
                return Ok(a.clone());
            }
        }
        Err(DataError::NotFound("no asset found in db with the given ISIN".to_string()))
    }


    fn get_all_assets(&mut self) -> Result<Vec<Asset>, DataError> {
        self.assets.get_all()
    }

    fn update_asset(&mut self, asset: &Asset) -> Result<(), DataError> {
        self.assets.update(asset)
    }

    fn delete_asset(&mut self, id: usize) -> Result<(), DataError> {
        self.assets.delete(id)
    }
}
