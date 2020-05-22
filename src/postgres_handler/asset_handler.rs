use super::PostgresDB;
use crate::asset::Asset;
use crate::data_handler::{AssetHandler, DataError};
use crate::currency::Currency;
use std::str::FromStr;

/// Handler for globally available data
impl AssetHandler for PostgresDB<'_> {
    fn insert_asset(&mut self, asset: &Asset) -> Result<usize, DataError> {
        let row = self
            .conn
            .query_one(
                "INSERT INTO assets (name, wkn, isin, note) VALUES ($1, $2, $3, $4) RETURNING id",
                &[&asset.name, &asset.wkn, &asset.isin, &asset.note],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id: i32 = row.get(0);
        Ok(id as usize)
    }

    fn get_asset_id(&mut self, asset: &Asset) -> Option<usize> {
        let row = if let Some(isin) = &asset.isin {
            self.conn
                .query_one("SELECT id FROM assets WHERE isin=$1", &[&isin])
        } else if let Some(wkn) = &asset.wkn {
            self.conn
                .query_one("SELECT id FROM assets WHERE wkn=$1", &[&wkn])
        } else {
            self.conn
                .query_one("SELECT id FROM assets WHERE name=$1", &[&asset.name])
        };
        match row {
            Ok(row) => {
                let id: i32 = row.get(0);
                Some(id as usize)
            }
            _ => None,
        }
    }

    fn get_asset_by_id(&mut self, id: usize) -> Result<Asset, DataError> {
        let row = self
            .conn
            .query_one(
                "SELECT name, wkn, isin, note FROM assets WHERE id=$1",
                &[&(id as i32)],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(Asset {
            id: Some(id),
            name: row.get(0),
            wkn: row.get(1),
            isin: row.get(2),
            note: row.get(3),
        })
    }

    fn get_asset_by_isin(&mut self, isin: &String) -> Result<Asset, DataError> {
        let row = self
            .conn
            .query_one(
                "SELECT id, name, wkn, note FROM assets WHERE isin=$1",
                &[isin],
            )
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let id: i32 = row.get(0);
        Ok(Asset {
            id: Some(id as usize),
            name: row.get(1),
            wkn: row.get(2),
            isin: Some(isin.clone()),
            note: row.get(3),
        })
    }

    fn get_all_assets(&mut self) -> Result<Vec<Asset>, DataError> {
        let mut assets = Vec::new();
        for row in self
            .conn
            .query("SELECT id, name, wkn, isin, note FROM assets ORDER BY name", &[])
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id: i32 = row.get(0);
            let id = Some(id as usize);
            assets.push(Asset {
                id,
                name: row.get(1),
                wkn: row.get(2),
                isin: row.get(3),
                note: row.get(4),
            });
        }
        Ok(assets)
    }

    fn update_asset(&mut self, asset: &Asset) -> Result<(), DataError> {
        if asset.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = asset.id.unwrap() as i32;
        self.conn
            .execute(
                "UPDATE assets SET name=$2, wkn=$3, isin=$4, note=$5 
                WHERE id=$1;",
                &[&id, &asset.name, &asset.wkn, &asset.isin, &asset.note],
            )
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn delete_asset(&mut self, id: usize) -> Result<(), DataError> {
        self.conn
            .execute("DELETE FROM assets WHERE id=$1;", &[&(id as i32)])
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn get_all_currencies(&mut self) -> Result<Vec<Currency>, DataError> {
        let mut currencies = Vec::new();
        for row in self
            .conn
            .query("SELECT name FROM assets WHERE isin IS NULL AND wkn IS NULL AND length(name)=3", &[])
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let currency: String = row.get(0);
            let currency =
                Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
            currencies.push(currency);
        }
        Ok(currencies)
    }
}
