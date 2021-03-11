///! Implementation of sqlite3 data handler

use std::str::FromStr;
use async_trait::async_trait;

use finql_data::asset::Asset;
use finql_data::{AssetHandler, DataError};
use finql_data::currency::Currency;

use super::SqliteDB;

/// helper struct
struct ID { id: Option<i64>, }

/// Handler for globally available Asset data
#[async_trait]
impl AssetHandler for SqliteDB {
    async fn insert_asset(&self, asset: &Asset) -> Result<usize, DataError> {
        sqlx::query!(
                "INSERT INTO assets (name, wkn, isin, note) VALUES (?1, ?2, ?3, ?4)",
                asset.name, asset.wkn, asset.isin, asset.note,
            ).execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let row = sqlx::query!(
                "SELECT id FROM assets WHERE name=?",
                asset.name,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        if let Some(id) = row.id {
            Ok(id as usize)
        } else {
            Err(DataError::NotFound("id after insert not found".to_string()))
        }
    }

    async fn insert_asset_if_new(
        &self,
        asset: &Asset,
        rename_asset: bool,
    ) -> Result<usize, DataError> {
        match self.get_asset_id(asset).await {
            Some(id) => Ok(id),
            None => match self.insert_asset(asset).await {
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
                        }).await
                    } else {
                        Err(err)
                    }
                }
            },
        }
    }

    async fn get_asset_id(&self, asset: &Asset) -> Option<usize> {
        let id = if let Some(isin) = &asset.isin {
            sqlx::query_as!(ID, "SELECT id FROM assets WHERE isin=?", isin).fetch_one(&self.pool).await
        } else if let Some(wkn) = &asset.wkn {
            sqlx::query_as!(ID, "SELECT id FROM assets WHERE wkn=?", wkn).fetch_one(&self.pool).await
        } else {
            sqlx::query_as!(ID, "SELECT id FROM assets WHERE name=?", asset.name).fetch_one(&self.pool).await
        }.ok();
        if let Some(id_opt) = id {
            id_opt.id.map(|x| x as usize)
        } else {
            None
        }
    }

    async fn get_asset_by_id(&self, id: usize) -> Result<Asset, DataError> {
        let id_param = id as i64;
        let row = sqlx::query!(
                "SELECT name, wkn, isin, note FROM assets WHERE id=?",
                id_param,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(Asset {
            id: Some(id),
            name: row.name,
            wkn: row.wkn,
            isin: row.isin,
            note: row.note,
        })
    }

    async fn get_asset_by_isin(&self, isin: &str) -> Result<Asset, DataError> {
        let row = sqlx::query!(
                "SELECT id, name, wkn, note FROM assets WHERE isin=?",
                isin,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let id = row.id.map(|x| x as usize);
        Ok(Asset {
            id,
            name: row.name,
            wkn: row.wkn,
            isin: Some(isin.to_string()),
            note: row.note,
        })
    }

    async fn get_all_assets(&self) -> Result<Vec<Asset>, DataError> {
        let mut assets = Vec::new();
        for row in sqlx::query!("SELECT id, name, wkn, isin, note FROM assets ORDER BY name")
            .fetch_all(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id = row.id.map(|x| x as usize);
            assets.push(Asset {
                id,
                name: row.name,
                wkn: row.wkn,
                isin: row.isin,
                note: row.note,
            });
        }
        Ok(assets)
    }

    async fn update_asset(&self, asset: &Asset) -> Result<(), DataError> {
        if asset.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = asset.id.unwrap() as i32;
        sqlx::query!(
                "UPDATE assets SET name=?2, wkn=?3, isin=?4, note=?5 
                WHERE id=?1",
                id, asset.name, asset.wkn, asset.isin, asset.note,
            ).execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    async fn delete_asset(&self, id: usize) -> Result<(), DataError> {
        let id = id as i32;
        sqlx::query!("DELETE FROM assets WHERE id=?;", id)
            .execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    async fn get_all_currencies(&self) -> Result<Vec<Currency>, DataError> {
        let mut currencies = Vec::new();
        for row in sqlx::query!("SELECT name FROM assets WHERE isin IS NULL AND wkn IS NULL AND length(name)=3")
            .fetch_all(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let currency = row.name;
            let currency =
                Currency::from_str(&currency).map_err(|e| DataError::NotFound(e.to_string()))?;
            currencies.push(currency);
        }
        Ok(currencies)
    }
}
