///! Implementation of sqlite3 object handler

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use serde_json;

use finql_data::{ObjectHandler, DataError};
use super::{SqliteDB, SQLiteError};

/// Handler for globally available Asset data
#[async_trait]
impl ObjectHandler for SqliteDB {
    async fn store_object<T: Serialize+Sync>(&self, name: &str, object_type: &str, object: &T) -> Result<(), DataError> {
        let object_json = serde_json::to_string(&object)
        .map_err(|e| DataError::InsertFailed(e.to_string()))?;
        
        let name = name.to_owned();
        let object_type = object_type.to_owned();
        self.conn.interact(move |conn| -> Result<(), SQLiteError> {
            conn.execute(
                "INSERT INTO objects (name, type, object) VALUES (?, ?, ?)",
                &[&name, &object_type, &object_json])?;
            Ok(())
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))
    }

    async fn get_object<T: DeserializeOwned>(&self,  name: &str) ->Result<T, DataError> {
        let name = name.to_owned();
        let object = self.conn.interact(move |conn| -> Result<String, SQLiteError> {
            Ok(conn.query_row(
                "SELECT object FROM objects WHERE name=?",
                &[&name],
                |row| row.get(0) )?)
        }).await.map_err(|e| DataError::DataAccessFailure(e.to_string()))?
        .map_err(|e| DataError::DataAccessFailure(e.to_string()))?;
        let object: T = serde_json::from_str(&object)
        .map_err(|_| DataError::DataAccessFailure("Failed to deserialize string to object".to_string()))?;
        Ok(object)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;
    use serde::Deserialize;
    use super::super::SqliteDBPool;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestData {
        text: String,
        number: i32,
    }

    #[tokio::test]
    async fn store_object() {
        let db_pool = Arc::new(SqliteDBPool::in_memory().await.unwrap());
        let db = db_pool.get_conection().await.unwrap();
        assert!(db.clean().await.is_ok());

        let test_data = TestData{
            text: "hello".to_string(),
            number: 10
        };

        db.store_object("first_struct", "testdata", &test_data).await.unwrap();
        let new_test_data = db.get_object::<TestData>("first_struct").await.unwrap();

        assert_eq!(new_test_data.text, test_data.text);
        assert_eq!(new_test_data.number, test_data.number);
    }
}
