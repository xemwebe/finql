use std::str::FromStr;
use async_trait::async_trait;

use crate::datatypes::asset::Asset;
use crate::datatypes::{AssetHandler, CurrencyISOCode, DataError, Resource};
use crate::datatypes::currency::Currency;

use super::PostgresDB;

/// helper struct
struct ID { id: i32, }


/// Handler for globally available Asset data
#[async_trait]
impl AssetHandler for PostgresDB {
    async fn insert_asset(&self, asset: &Asset) -> Result<usize, DataError> {
        let asset = asset.to_owned();

        let row = sqlx::query!(
                "INSERT INTO assets (name, note) VALUES ($1, $2) RETURNING id",
                asset.name, asset.note,
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = row.id;

        match asset.resource {
            Resource::Currency(c) => {
                sqlx::query!(
                    "INSERT INTO currencies (id, iso_code, rounding_digits) VALUES ($1, $2, $3)",
                    id, &c.iso_code.to_string(), &c.rounding_digits,
                );

                Ok(id as usize)
            },
            Resource::Stock(s) => {
                sqlx::query!(
                        "INSERT INTO stocks (id, isin, wkn) VALUES ($1, $2, $3)",
                        id, &s.isin, s.wkn
                );

                Ok(id as usize)
            }
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
                sqlx::query_as!(ID, "SELECT id FROM currencies WHERE iso_code = $1", &c.iso_code.to_string()).fetch_one(&self.pool).await.ok()
            },
            Resource::Stock(s) => {
                if let Some(wkn) = &s.wkn {
                    sqlx::query_as!(ID, "SELECT id FROM stocks WHERE wkn = $1", wkn).fetch_one(&self.pool).await.ok()
                } else {
                    sqlx::query_as!(ID, "SELECT id FROM stocks WHERE isin = $1", s.isin).fetch_one(&self.pool).await.ok()
                }
            }
        };

        if id.is_some() {
            return id.map(|x| x.id as usize);
        }

        sqlx::query_as!(ID, "SELECT id FROM assets WHERE name = $1", name).fetch_one(&self.pool).await.ok();
        id.map(|x| x.id as usize)
    }

    async fn get_asset_by_id(&self, id: usize) -> Result<Asset, DataError> {
        let row = sqlx::query!(
                r#"SELECT
                     a.id,
                     a.name,
                     a.note,
                     c.id AS "currency_id?",
                     c.iso_code AS "iso_code?",
                     c.rounding_digits AS "rounding_digits?",
                     s.id AS "stock_id?",
                     s.isin AS "isin?",
                     s.wkn AS "wkn?"
                 FROM assets a
                 LEFT JOIN currencies c ON c.id = a.id
                 LEFT JOIN stocks s ON s.id = a.id
                 WHERE a.id = $1"#,
                (id as i32),
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;

        let cid:Option<i32> = row.currency_id;
        let sid = row.stock_id;

        let name = row.name;
        let note = row.note;

        if cid.is_some() {
            let ic:Option<String> = row.iso_code;
            let rd:Option<i32> = row.rounding_digits;
            let rd = rd.expect("missing rounding_digits");

            let ic = CurrencyISOCode::new(&ic.unwrap()).expect("invalid hard_currency.iso_code arguments from db unexpected");

            return Ok(Asset::new_currency(
                Some(id),
                name,
                note,
                ic,
                rd,
            ));
        } else if sid.is_some() {
            let isin:Option<String> = row.isin;

            return Ok(Asset::new_stock(
                Some(id),
                name,
                note,
                isin.unwrap(),
                row.wkn,
            ));
        }
        else {
            //TODO: handle this more gracefully, wasn't able to find a way to return an Err here
            unreachable!("Invalid asset type!");
        }
    }

    async fn get_asset_by_isin(&self, isin: &str) -> Result<Asset, DataError> {
        let row = sqlx::query!(
                r#"SELECT
                   a.id,
                   a.name,
                   a.note,
                   s.id AS "stock_id!",
                   s.isin AS "isin!",
                   s.wkn AS "wkn?"
                 FROM assets a
                 JOIN stocks s ON s.id = a.id
                 WHERE s.isin = $1"#,
                isin.to_string(),
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        Ok(Asset::new_stock(
            Some(row.id as usize),
            row.name,
            row.note,
            row.isin,
            row.wkn,
        ))
    }

    async fn get_all_assets(&self) -> Result<Vec<Asset>, DataError> {
        let mut assets = Vec::new();
        for row in sqlx::query!(
            r#"SELECT
                 a.id,
                 a.name,
                 a.note,
                 c.id AS "currency_id?",
                 c.iso_code AS "iso_code?",
                 c.rounding_digits AS "rounding_digits?",
                 s.id AS "stock_id?",
                 s.isin AS "isin?",
                 s.wkn AS "wkn?"
             FROM assets a
             LEFT JOIN currencies c ON c.id = a.id
             LEFT JOIN stocks s ON s.id = a.id
             ORDER BY a.name"#)
            .fetch_all(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?
        {
            let id = row.id as usize;
            let cid:Option<usize> = row.currency_id.and_then(|i| Some(i as usize));
            let sid:Option<usize> = row.stock_id.and_then(|i| Some(i as usize));

            let name:String = row.name;
            let note = row.note;

            if cid.is_some() {
                let ic:Option<String> = row.iso_code;
                let rd:Option<i32> = row.rounding_digits;
                let rd = rd.expect("missing rounding_digits");

                let ic = CurrencyISOCode::new(&ic.unwrap()).expect("invalid hard_currency.iso_code arguments from db unexpected");

                assets.push(Asset::new_currency(
                    Some(id),
                    name,
                    note,
                    ic,
                    rd,
                ));
            } else if sid.is_some() {
                let isin:Option<String> = row.isin;

                assets.push(Asset::new_stock(
                    Some(id),
                    name,
                    note,
                    isin.unwrap(),
                    row.wkn,
                ));
            }
            else {
                //TODO: handle this more gracefully, wasn't able to find a way to return an Err here
                unreachable!("Invalid asset type!");
            }
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
                "UPDATE assets SET name=$2, note=$3
                WHERE id=$1;",
                id, asset.name, asset.note,
            ).execute(&self.pool).await
            .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    async fn delete_asset(&self, id: usize) -> Result<(), DataError> {
        sqlx::query!("DELETE FROM assets WHERE id=$1;", (id as i32))
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
