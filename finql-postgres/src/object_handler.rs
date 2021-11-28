///! Implementation of sqlite3 object handler

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use serde_json;

use finql_data::{ObjectHandler, DataError};
use super::PostgresDB;

/// Handler for globally available Asset data
#[async_trait]
impl ObjectHandler for PostgresDB {
    async fn store_object<T: Serialize+Sync>(&self, id: &str, _object_type: &str, object: &T) -> Result<(), DataError> {
        let object_json = serde_json::to_value(&object)
        .map_err(|_| DataError::InsertFailed("Could not serialize object".to_string()))?;

        sqlx::query!(
            "INSERT INTO objects (id, object) VALUES ($1, $2)",
            id, object_json
        ).execute(&self.pool).await
        .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    async fn get_object<T: DeserializeOwned>(&self,  id: &str) ->Result<T, DataError> {
        let row = sqlx::query!(
                "SELECT object FROM objects WHERE id=$1",
                id
            ).fetch_one(&self.pool).await
            .map_err(|e| DataError::NotFound(e.to_string()))?;
        let object: T = serde_json::from_value(row.object)
        .map_err(|_| DataError::DataAccessFailure("Failed to deserialize string to object".to_string()))?;
        Ok(object)
    }
}