///! Implementation of sqlite3 object handler
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use serde_json;

use super::PostgresDB;
use crate::datatypes::{DataError, ObjectHandler};

/// Handler for globally available Asset data
#[async_trait]
impl ObjectHandler for PostgresDB {
    async fn store_object<T: Serialize + Sync>(
        &self,
        id: &str,
        _object_type: &str,
        object: &T,
    ) -> Result<(), DataError> {
        let object_json = serde_json::to_value(&object)?;

        sqlx::query!(
            "INSERT INTO objects (id, object) VALUES ($1, $2)",
            id,
            object_json
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_object<T: DeserializeOwned>(&self, id: &str) -> Result<T, DataError> {
        let row = sqlx::query!("SELECT object FROM objects WHERE id=$1", id)
            .fetch_one(&self.pool)
            .await?;
        let object: T = serde_json::from_value(row.object)?;
        Ok(object)
    }
}
