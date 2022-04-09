use async_trait::async_trait;

use crate::datatypes::{Asset, Currency, CurrencyISOCode, Stock, AssetHandler, DataError, DataItem};

use super::PostgresDB;

/// helper struct
struct ID { id: i32, }


/// Handler for globally available Asset data
#[async_trait]
impl AssetHandler for PostgresDB {
    async fn insert_asset(&self, asset: &Asset) -> Result<i32, DataError> {
        let asset = asset.to_owned();
        // begin transaction
        let tx = self.pool.begin().await?;
        let row = sqlx::query!(
                "INSERT INTO assets (asset_class) VALUES ($1) RETURNING id",
                asset.class(),
            ).fetch_one(&self.pool).await?;
        let id = row.id;

        match asset {
            Asset::Currency(c) => {
                sqlx::query!(
                    "INSERT INTO currencies (id, iso_code, rounding_digits) VALUES ($1, $2, $3)",
                    id, c.iso_code.to_string(), c.rounding_digits,
                ).execute(&self.pool).await?;
                tx.commit().await?;
                Ok(id)
            },
            Asset::Stock(s) => {
                sqlx::query!(
                        "INSERT INTO stocks (id, name, isin, wkn, note) VALUES ($1, $2, $3, $4, $5)",
                        id, s.name, s.isin, s.wkn, s.note
                ).execute(&self.pool).await?;
                tx.commit().await?;
                Ok(id)
            }
        }
    }
    
    async fn get_asset_id(&self, asset: &Asset) -> Option<i32> {
        let id = match asset {
            Asset::Currency(c) => {
                sqlx::query_as!(ID, "SELECT id FROM currencies WHERE iso_code = $1", &c.iso_code.to_string()).fetch_one(&self.pool).await.ok()
            },
            Asset::Stock(s) => {
                if let Some(wkn) = &s.wkn {
                    sqlx::query_as!(ID, "SELECT id FROM stocks WHERE wkn = $1", wkn).fetch_one(&self.pool).await.ok()
                } else {
                    sqlx::query_as!(ID, "SELECT id FROM stocks WHERE isin = $1", s.isin).fetch_one(&self.pool).await.ok()
                }
            }
        };

        id.map(|x| x.id)
    }

    async fn get_asset_by_id(&self, id: i32) -> Result<Asset, DataError> {
        let row = sqlx::query!(
            r#"SELECT
                asset_class
             FROM assets 
             WHERE id = $1"#,
            id,
        ).fetch_one(&self.pool).await?;

        match row.asset_class.as_str() {
            "currency" => {
                let row = sqlx::query!(
                    r#"SELECT
                         id,
                         iso_code,
                         rounding_digits
                     FROM currencies 
                     WHERE id = $1"#,
                    id,
                ).fetch_one(&self.pool).await?;

                Ok(Asset::Currency(Currency::new(
                    Some(row.id), 
                    CurrencyISOCode::new(&row.iso_code)?, 
                    Some(row.rounding_digits))))
            },
            "stock" => {
                let row = sqlx::query!(
                    r#"SELECT
                        id,
                        name,
                        isin,
                        wkn,
                        note
                     FROM stocks s
                     WHERE id = $1"#,
                   id,
                ).fetch_one(&self.pool).await?;

                Ok(Asset::Stock(
                    Stock::new(
                    Some(row.id),
                    row.name,
                    row.isin,
                    row.wkn,
                    row.note)))
            }
            _ => {
                Err(DataError::InvalidAsset(row.asset_class))
            }
        }
    }

    async fn get_asset_by_isin(&self, isin: &str) -> Result<Asset, DataError> {
        let row = sqlx::query!(
                r#"SELECT
                   id,
                   name,
                   isin,
                   wkn,
                   note
                 FROM stocks
                 WHERE isin = $1"#,
                isin.to_string(),
            ).fetch_one(&self.pool).await?;

        Ok(Asset::Stock(
            Stock::new(
            Some(row.id),
            row.name,
            row.isin,
            row.wkn,
            row.note)))
    }

    async fn get_all_assets(&self) -> Result<Vec<Asset>, DataError> {
        let mut assets = Vec::new();
        for row in sqlx::query!(
            r#"SELECT
                id
             FROM assets"#
        ).fetch_all(&self.pool).await?
        {
            assets.push(self.get_asset_by_id(row.id).await?
            );
        }
        Ok(assets)
    }

    async fn update_asset(&self, asset: &Asset) -> Result<(), DataError> {
        match asset {
            Asset::Currency(c) => {
                if let Some(id) = c.id  {
                    sqlx::query!(
                        "UPDATE currencies 
                        SET 
                            iso_code=$2,
                            rounding_digits=$3
                        WHERE id=$1;",
                        id as i32, c.iso_code.to_string(), c.rounding_digits
                    ).execute(&self.pool).await?;
                    Ok(())
                } else {
                    Err(DataError::NotFound(
                        "not yet stored to database".to_string(),
                    ))
                }
            },
            Asset::Stock(s) => {
                if let Some(id) = s.id  {
                    sqlx::query!(
                        "UPDATE stocks 
                        SET 
                            name=$2,
                            isin=$3,
                            wkn=$4,
                            note=$5
                        WHERE id=$1;",
                        id as i32, s.name, s.isin, s.wkn, s.note
                    ).execute(&self.pool).await?;
                    Ok(())
                } else {
                    Err(DataError::NotFound(
                        "not yet stored to database".to_string(),
                    ))
                }
            }
        }
    }

    async fn delete_asset(&self, id: i32) -> Result<(), DataError> {
        let row = sqlx::query!("SELECT asset_class FROM assets WHERE id=$1",
                id as i32,
            ).fetch_one(&self.pool).await?;
        match row.asset_class.as_str() {
            "currency" => {
                let tx = self.pool.begin().await?;
                sqlx::query!("DELETE FROM currencies WHERE id=$1;",id)
                    .execute(&self.pool).await?;
                sqlx::query!("DELETE FROM assets WHERE id=$1;",id)
                    .execute(&self.pool).await?;
                tx.commit().await?;
                Ok(())
            },
            "stock" => {
                let tx = self.pool.begin().await?;
                sqlx::query!("DELETE FROM stocks WHERE id=$1;",id)
                    .execute(&self.pool).await?;
                sqlx::query!("DELETE FROM assets WHERE id=$1;",id)
                    .execute(&self.pool).await?;      
                tx.commit().await?;
                Ok(())
            },
            _ => Err(DataError::InvalidAsset("Could not delete unknown asset".to_string()))
        }
    }

    async fn get_all_currencies(&self) -> Result<Vec<Currency>, DataError> {
        let mut currencies = Vec::new();
        for row in sqlx::query!(
            "SELECT
                id,
                iso_code,
                rounding_digits
            FROM currencies")
             .fetch_all(&self.pool).await?
        {
            currencies.push(Currency::new(
                Some(row.id),
                CurrencyISOCode::new(&row.iso_code)?,
                Some(row.rounding_digits)
            ));
        }
        Ok(currencies)
    }

    async fn get_or_new_currency(&self, iso_code: CurrencyISOCode) -> Result<Currency, DataError> {
        self.get_or_new_currency_with_digits(iso_code, 2).await
    }

    async fn get_or_new_currency_with_digits(&self, iso_code: CurrencyISOCode, rounding_digits: i32) -> Result<Currency, DataError>
    {
        let row = sqlx::query!(
            "SELECT
                id,
                rounding_digits
            FROM currencies
            WHERE iso_code=$1",
            iso_code.to_string())
            .fetch_one(&self.pool).await;
        
        if let Ok(row) = row {
            Ok(Currency::new(Some(row.id), iso_code, Some(row.rounding_digits)))
        } else {
            let mut currency = Currency::new(None, iso_code, Some(rounding_digits));
            let id = self.insert_asset(&Asset::Currency(currency)).await?;
            currency.set_id(id)?;
            Ok(currency)
        }
    }
}
