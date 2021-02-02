use std::str::FromStr;
use async_trait::async_trait;

use finql_data::asset::Asset;
use finql_data::{AssetHandler, DataError};
use finql_data::currency::Currency;

use super::PostgresDB;

/// helper struct
struct ID { id: i32, }


/// Handler for globally available data
#[async_trait]
impl AssetHandler for PostgresDB {
    async fn insert_asset(&mut self, asset: &Asset) -> Result<usize, DataError> {
        let row = sqlx::query!(
                "INSERT INTO assets (name, wkn, isin, note) VALUES ($1, $2, $3, $4) RETURNING id",
                asset.name, asset.wkn, asset.isin, asset.note,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = row.id;
        Ok(id as usize)
    }
   
    async fn get_asset_id(&mut self, asset: &Asset) -> Option<usize> {
        let id = if let Some(isin) = &asset.isin {
            sqlx::query_as!(ID, "SELECT id FROM assets WHERE isin=$1", isin).fetch_one(&self.pool).await.ok()
        } else if let Some(wkn) = &asset.wkn {
            sqlx::query_as!(ID, "SELECT id FROM assets WHERE wkn=$1", wkn).fetch_one(&self.pool).await.ok()
        } else {
            sqlx::query_as!(ID, "SELECT id FROM assets WHERE name=$1", asset.name).fetch_one(&self.pool).await.ok()
        };
        id.map(|x| x.id as usize)
    }

    async fn get_asset_by_id(&mut self, id: usize) -> Result<Asset, DataError> {
        let row = sqlx::query!(
                "SELECT name, wkn, isin, note FROM assets WHERE id=$1",
                (id as i32),
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

    async fn get_asset_by_isin(&mut self, isin: &str) -> Result<Asset, DataError> {
        let row = sqlx::query!(
                "SELECT id, name, wkn, note FROM assets WHERE isin=$1",
                isin.to_string(),
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let id: i32 = row.id;
        Ok(Asset {
            id: Some(id as usize),
            name: row.name,
            wkn: row.wkn,
            isin: Some(isin.to_string()),
            note: row.note,
        })
    }

    async fn get_all_assets(&mut self) -> Result<Vec<Asset>, DataError> {
        let mut assets = Vec::new();
        for row in sqlx::query!("SELECT id, name, wkn, isin, note FROM assets ORDER BY name")
            .fetch_all(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id: i32 = row.id;
            let id = Some(id as usize);
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

    async fn update_asset(&mut self, asset: &Asset) -> Result<(), DataError> {
        if asset.id.is_none() {
            return Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ));
        }
        let id = asset.id.unwrap() as i32;
        sqlx::query!(
                "UPDATE assets SET name=$2, wkn=$3, isin=$4, note=$5 
                WHERE id=$1;",
                id, asset.name, asset.wkn, asset.isin, asset.note,
            ).execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    async fn delete_asset(&mut self, id: usize) -> Result<(), DataError> {
        sqlx::query!("DELETE FROM assets WHERE id=$1;", (id as i32))
            .execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    async fn get_all_currencies(&mut self) -> Result<Vec<Currency>, DataError> {
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
