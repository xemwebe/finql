///! Implementation of sqlite3 data handler

use std::str::FromStr;
use async_trait::async_trait;

use finql_data::{Asset, Stock, Resource};
use finql_data::{AssetHandler, DataError};
use finql_data::currency::{Currency, CurrencyISOCode};

use super::{SqliteDB, SQLiteError};
use deadpool_sqlite::rusqlite::params;
use serde_json::error::Category::Data;

/// Handler for globally available Asset data
#[async_trait]
impl AssetHandler for SqliteDB {
    async fn insert_asset(&self, asset: &Asset) -> Result<usize, DataError> {
        let asset = asset.to_owned();
     
        self.conn.interact(move |conn| -> Result<usize, SQLiteError> {
            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO assets (name, note) VALUES (?1, ?2)",
                params![&asset.name, &asset.note])
                ?;
            let id = tx.last_insert_rowid();

            match asset.resource {
                Resource::Currency(c) => {
                    tx.execute(
                        "INSERT INTO currencies (id, iso_code, rounding_digits) VALUES (?1, ?2, ?3)",
                        params![id, &c.iso_code.to_string(), &c.rounding_digits]
                    )?;
                },
                Resource::Stock(s) => {
                    tx.execute(
                        "INSERT INTO stocks (id, isin, wkn) VALUES (?1, ?2, ?3)",
                        params![id, &s.isin, &s.wkn])?;
                },
            }

            tx.commit()?;
            Ok(id as usize)
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
                        self.insert_asset(&mut Asset {
                            id: None,
                            name: new_name,
                            note: asset.note.clone(),
                            resource: asset.resource.clone(),
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
        let name = asset.name.clone();

        let id = match asset.resource {
            Resource::Currency(c) => {
                self.conn.interact(move |conn| -> Option<usize> {
                    conn.query_row(
                        "SELECT id FROM currencies WHERE iso_code=?",
                        params![&c.iso_code.to_string()],
                        |row| row.get(0)).ok()
                }).await
            },
            Resource::Stock(s) => {
                if let Some(wkn) = &s.wkn {
                    let wkn = wkn.to_owned();
                    self.conn.interact(move |conn| -> Option<usize> {
                        conn.query_row(
                            "SELECT id FROM stocks WHERE wkn = ?",
                            params![&wkn],
                            |row| {
                                let id:usize = row.get(0)?;
                                row.get(0)
                            }).ok()
                    }).await
                } else {
                    self.conn.interact(move |conn| -> Option<usize> {
                        conn.query_row(
                            "SELECT id FROM stocks WHERE isin=?",
                            params![s.isin],
                            |row| row.get(0)).ok()
                    }).await
                }
            }
        }.ok().flatten();

        if id.is_some() {
            return id;
        }

        self.conn.interact(move |conn| -> Option<usize>  {
            conn.query_row(
                "SELECT id FROM assets WHERE name=?",
                params![name],
                |row| row.get(0) ).ok()
        }).await.ok().flatten()
    }

    async fn get_asset_by_id(&self, id: usize) -> Result<Asset, DataError> {
        self.conn.interact(move |conn| -> Result<Asset, SQLiteError> {
            Ok(conn.query_row(
                "SELECT
                     a.id,
                     a.name,
                     a.note,
                     c.id AS currency_id,
                     c.iso_code,
                     c.rounding_digits,
                     s.id AS stock_id,
                     s.isin,
                     s.wkn
                 FROM assets a
                 LEFT JOIN currencies c ON c.id = a.id
                 LEFT JOIN stocks s ON s.id = a.id
                 WHERE a.id = ?",
                params![&id],
                |row| {
                    let cid:Option<usize> = row.get(3)?;
                    let sid:Option<usize> = row.get(6)?;

                    let name:String = row.get(1)?;
                    let note = row.get(2)?;

                    if let Some(cid) = cid {
                        let ic:Option<String> = row.get(4)?;
                        let rd:Option<i32> = row.get(5)?;
                        let rd = rd.expect("missing rounding_digits");

                        let ic = CurrencyISOCode::new(&ic.unwrap()).expect("invalid hard_currency.iso_code arguments from db unexpected");

                        return Ok(Asset::new_currency(
                            Some(id),
                            name,
                            note,
                            ic,
                            rd,
                        ));
                    } else if let Some(sid) = sid {
                        let isin:Option<String> = row.get(7)?;

                        return Ok(Asset::new_stock(
                            Some(id),
                            name,
                            note,
                            isin.unwrap(),
                            row.get(8)?,
                        ));
                    }
                    else {
                        //TODO: handle this more gracefully, wasn't able to find a way to return an Err here
                        unreachable!("Invalid asset type!");
                    }
                })?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
            .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_asset_by_isin(&self, isin: &str) -> Result<Asset, DataError> {
        let isin = isin.to_owned();
        self.conn.interact(move |conn| -> Result<Asset, SQLiteError> {
            Ok(conn.query_row(
                "SELECT a.id, a.name, a.note, s.id AS stock_id, s.isin, s.wkn
                 FROM assets a
                 JOIN stocks s ON s.id = a.id
                 WHERE s.isin = ?",
                params![&isin],
                |row| {
                    let name:String = row.get(1)?;

                    Ok(Asset::new_stock(
                        row.get(0)?,
                        name,
                        row.get(2)?,
                        row.get(4)?,
                        row.get(2)?,
                    ))
            })?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_all_assets(&self) -> Result<Vec<Asset>, DataError> {
        self.conn.interact(|conn| -> Result<Vec<Asset>, SQLiteError> {
            let mut stmt = conn.prepare(
                "SELECT
                     a.id,
                     a.name,
                     a.note,
                     c.id AS currency_id,
                     c.iso_code,
                     c.rounding_digits,
                     s.id AS stock_id,
                     s.isin,
                     s.wkn
                 FROM assets a
                 LEFT JOIN currencies c ON c.id = a.id
                 LEFT JOIN stocks s ON s.id = a.id
                 ORDER BY a.name")?;

            let assets: Vec<Asset> = stmt.query_map([], |row| {
                let id:Option<usize> = row.get(0)?;
                let cid:Option<usize> = row.get(3)?;
                let sid:Option<usize> = row.get(6)?;

                let id = id.expect("id should exist in db");
                let name:String = row.get(1)?;
                let note = row.get(2)?;

                if let Some(cid) = cid {
                    let ic:Option<String> = row.get(4)?;
                    let rd:Option<i32> = row.get(5)?;
                    let rd = rd.expect("missing rounding_digits");

                    let ic = CurrencyISOCode::new(&ic.unwrap()).expect("invalid hard_currency.iso_code arguments from db unexpected");

                    return Ok(Asset::new_currency(
                        Some(id),
                        name,
                        note,
                        ic,
                        rd,
                    ));
                } else if let Some(sid) = sid {
                    let isin:Option<String> = row.get(7)?;

                    return Ok(Asset::new_stock(
                        Some(id),
                        name,
                        note,
                        isin.unwrap(),
                        row.get(8)?,
                    ));
                }
                else {
                    //TODO: handle this more gracefully, wasn't able to find a way to return an Err here
                    unreachable!("Invalid asset type!");
                }
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
                    "UPDATE assets SET name = ?2, note = ?3
                    WHERE id = ?1",
                    params![&id, &asset.name, &asset.note])?;

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
    use finql_data::DataItem;
    use super::super::SqliteDBPool;
    
    #[tokio::test]
    async fn asset_handler_test() {
        let db_pool = Arc::new(SqliteDBPool::in_memory().await.unwrap());
        let db = db_pool.get_conection().await.unwrap();
        assert!(db.clean().await.is_ok());

        let mut asset1 = Asset{
            id: None,
            name: "A asset".to_string(),
            note: Some("Just a simple asset for testing".to_string()),
            resource: Resource::Stock(Stock {
                id: None,
                isin: "123456789012".to_string(),
                wkn: Some("A1B2C3".to_string()),
            }),
        };

        let mut id1 = db.insert_asset(&mut asset1).await.unwrap();
        asset1.set_id(id1);
        assert_eq!(asset1.id, Some(1), "insert id");

        let id = db.get_asset_id(&asset1).await;
        assert_eq!(id, Some(1), "id by get_asset_id");

        let asset2 = db.get_asset_by_isin("123456789012").await.unwrap();
        assert_eq!(asset2.id, Some(1), "id by get_asset_by_isin");
        assert_eq!(&asset2.name, "A asset", "name by get_asset_by_isin");

        let asset2 = db.get_asset_by_id(1).await.unwrap();
        assert_eq!(asset2.id, Some(1), "2nd id by get_asset_by_id");
        assert_eq!(&asset2.name, "A asset", "2nd name by get_asset_by_id");

        let mut asset2 = Asset{
            id: None,
            name: "B asset".to_string(),
            note: Some("Some other asset".to_string()),
            resource: Resource::Stock(Stock {
                id: None,
                isin: "210987654321".to_string(),
                wkn: Some("3c2b1a".to_string()),
            })
        };

        let id2 = db.insert_asset(&mut asset2).await.unwrap();
        asset2.set_id(id2);
        assert_eq!(asset2.id, Some(2));
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

    #[tokio::test]
    async fn unique_asset_test() {
        let db_pool = Arc::new(SqliteDBPool::in_memory().await.unwrap());
        let db = db_pool.get_conection().await.unwrap();
        assert!(db.clean().await.is_ok());

        let mut asset1 = Asset{
            id: None,
            name: "EUR".to_string(),
            note: None,
            resource: Resource::Currency(Currency {
                id: None,
                iso_code: CurrencyISOCode::new("EUR").unwrap(),
                rounding_digits: 4,
            }),
        };

        let id1 = db.insert_asset(&asset1).await.unwrap();
        asset1.set_id(id1).unwrap();
        assert_eq!(asset1.id, Some(1), "insert id");


        let asset2 = Asset{
            id: None,
            name: "EUR2".to_string(),
            note: None,
            resource: Resource::Currency(Currency {
                id: None,
                iso_code: CurrencyISOCode::new("EUR").unwrap(),
                rounding_digits: 4,
            }),
        };

        // Should return an error, since we tried to store same currency twice
        let res = db.insert_asset(&asset2).await;
        assert!(res.is_err());

        let assets = db.get_all_assets().await.unwrap();
        // Since second transaction failed, we still should have only one entry in database
        assert_eq!(assets.len(), 1);

    }
}
