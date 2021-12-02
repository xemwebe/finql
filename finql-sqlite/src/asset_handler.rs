///! Implementation of sqlite3 data handler

use std::str::FromStr;
use async_trait::async_trait;

use finql_data::asset::Asset;
use finql_data::{AssetHandler, DataError};
use finql_data::currency::Currency;

use super::{SqliteDB, SQLiteError};
use deadpool_sqlite::rusqlite::params;

/// Handler for globally available Asset data
#[async_trait]
impl AssetHandler for SqliteDB {
    async fn insert_asset(&self, asset: &Asset) -> Result<usize, DataError> {
        let asset_name = asset.name.clone();
        let asset = asset.to_owned();
        let _ = self.conn.interact(move |conn| -> Result<(), SQLiteError> {
            conn.execute(
                "INSERT INTO assets (name, wkn, isin, note) VALUES (?1, ?2, ?3, ?4)",
                params![&asset.name, &asset.wkn, &asset.isin, &asset.note])?;
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()));

        self.conn.interact(move |conn| -> Result<usize, SQLiteError> {
            Ok(conn.query_row(
                "SELECT id FROM assets WHERE name=?",
                params![&asset_name],
                |row| row.get(0) )?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
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
        let asset = asset.to_owned();
        if let Some(isin) = &asset.isin {
            let isin = isin.to_owned();
            self.conn.interact(move |conn| -> Option<usize> {
                conn.query_row(
                    "SELECT id FROM assets WHERE isin=?",
                    params![&isin],
                    |row| row.get(0) ).ok()
            }).await.ok().flatten()
        } else if let Some(wkn) = &asset.wkn {
            let wkn = wkn.to_owned();
            self.conn.interact(move |conn| -> Option<usize> {
                conn.query_row(
                    "SELECT id FROM assets WHERE wkn=?",
                    params![&wkn],
                    |row| row.get(0) ).ok()
            }).await.ok().flatten()
        } else {
            self.conn.interact(move |conn| -> Option<usize>  {
                conn.query_row(
                    "SELECT id FROM assets WHERE name=?",
                    params![&asset.name],
                    |row| row.get(0) ).ok()
            }).await.ok().flatten()
        } 
    }

    async fn get_asset_by_id(&self, id: usize) -> Result<Asset, DataError> {
        self.conn.interact(move |conn| -> Result<Asset, SQLiteError> {
            Ok(conn.query_row(
                "SELECT name, wkn, isin, note FROM assets WHERE id=?",
                params![&id],
                |row| { Ok(Asset {
                    id: Some(id),
                    name: row.get(0)?,
                    wkn: row.get(1)?,
                    isin: row.get(2)?,
                    note: row.get(3)?,
                })
            })?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_asset_by_isin(&self, isin: &str) -> Result<Asset, DataError> {
        let isin = isin.to_owned();
        self.conn.interact(move |conn| -> Result<Asset, SQLiteError> {
            Ok(conn.query_row(
                "SELECT id, name, wkn, note FROM assets WHERE isin=?",
                params![&isin],
                |row| { Ok(Asset {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    wkn: row.get(2)?,
                    isin: Some(isin.clone()),
                    note: row.get(3)?,
                })
            })?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_all_assets(&self) -> Result<Vec<Asset>, DataError> {
        self.conn.interact(|conn| -> Result<Vec<Asset>, SQLiteError> {
            let mut stmt = conn.prepare("SELECT id, name, wkn, isin, note FROM assets ORDER BY name")?;
            let assets: Vec<Asset> = stmt.query_map([], |row| {
                Ok(Asset {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    wkn: row.get(2)?,
                    isin: row.get(3)?,
                    note: row.get(4)?,
                })
            })?.filter_map(|asset| asset.ok() ).collect();
            Ok(assets)
        })
        .await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn update_asset(&self, asset: &Asset) -> Result<(), DataError> {
        if let Some(id) = asset.id {
            let asset = asset.to_owned();
            self.conn.interact(move |conn| -> Result<(), SQLiteError> {
                conn.execute(
                    "UPDATE assets SET name=?2, wkn=?3, isin=?4, note=?5 
                    WHERE id=?1",
                    params![&id, &asset.name, &asset.wkn, &asset.isin, &asset.note])?;
                Ok(())
            }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
            .map_err(|e| DataError::DataAccessFailure(e.to_string()))
        } else {
            Err(DataError::NotFound(
                "not yet stored to database".to_string(),
            ))
        }
    }

    async fn delete_asset(&self, id: usize) -> Result<(), DataError> {
        self.conn.interact(move |conn| -> Result<(), SQLiteError> {
            conn.execute("DELETE FROM assets WHERE id=?;", params![&id])?;
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_all_currencies(&self) -> Result<Vec<Currency>, DataError> {
        self.conn.interact(|conn| -> Result<Vec<Currency>, SQLiteError> {
            let mut stmt = conn.prepare("SELECT name FROM assets WHERE isin IS NULL AND wkn IS NULL AND length(name)=3 ORDER BY name")?;
            let currencies: Vec<Currency> = stmt.query_map([], |row| {
                let c: String = row.get(0)?;
                Ok(c)
            })?.filter_map(|c| c.ok() ).filter_map(|c| Currency::from_str(&c).ok() ).collect();
            Ok(currencies)
        })
        .await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;
    use super::super::SqliteDBPool;
    
    #[tokio::test]
    async fn asset_handler_test() {
        let db_pool = Arc::new(SqliteDBPool::in_memory().await.unwrap());
        let db = db_pool.get_conection().await.unwrap();
        assert!(db.clean().await.is_ok());

        let asset1 = Asset{
            id: None,
            name: "A asset".to_string(),
            isin: Some("123456789012".to_string()),
            wkn: Some("A1B2C3".to_string()),
            note: Some("Just a simple asset for testing".to_string()),
        };

        let id = db.insert_asset(&asset1).await.unwrap();
        assert_eq!(id, 1);

        let id = db.get_asset_id(&asset1).await;
        assert_eq!(id, Some(1));

        let asset2 = db.get_asset_by_isin("123456789012").await.unwrap();
        assert_eq!(asset2.id, Some(1));
        assert_eq!(&asset2.name, "A asset");

        let asset2 = db.get_asset_by_id(1).await.unwrap();
        assert_eq!(asset2.id, Some(1));
        assert_eq!(&asset2.name, "A asset");

        let mut asset2 = Asset{
            id: None,
            name: "B asset".to_string(),
            isin: Some("210987654321".to_string()),
            wkn: Some("3c2b1a".to_string()),
            note: Some("Some other asset".to_string()),
        };

        let id2 = db.insert_asset(&asset2).await.unwrap();
        assert_eq!(id2, 2);
        asset2.id = Some(id2);
        asset2.name = "bb".to_string();
        assert!(db.update_asset(&asset2).await.is_ok());

        let assets = db.get_all_assets().await.unwrap();
        assert_eq!(assets.len(), 2);

        db.delete_asset(1).await.unwrap();
        let assets = db.get_all_assets().await.unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].id, Some(2));
        assert_eq!(&assets[0].name, "bb");
    }
}
