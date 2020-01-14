///! Struct to handle connections to sqlite3 databases

use rusqlite::{params,Connection, OpenFlags, NO_PARAMS};
use crate::asset::Asset;
use crate::data_handler::{DataHandler, DataError};

pub struct SqliteDB {
    conn: Connection,
}

impl SqliteDB {
    pub fn connect(file_path: &str) -> rusqlite::Result<SqliteDB> {
        let conn = Connection::open_with_flags(file_path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        Ok(SqliteDB{conn})
    }

    pub fn create(file_path: &str) -> rusqlite::Result<SqliteDB> {
        let conn = Connection::open(file_path)?;
        let db = SqliteDB{conn};
        db.init()?;
        Ok(db)
    }

    /// Initialize new database by creating table, fill
    fn init(&self) -> rusqlite::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS assets (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                wkn TEXT UNIQUE,
                isin TEXT UNIQUE,
                note TEXT
            )",
            NO_PARAMS,
        )?;
        Ok(())
    }

}

/// Handler for globally available data
impl DataHandler for SqliteDB {
    fn get_asset_by_id(&self, id: u64) -> Result<Asset, DataError> {
        let asset = self.conn.query_row("SELECT id, name, wkn, isin, note FROM assets
        WHERE id=?;", &[id as i64],
         |row| { 
                let id: i64 = row.get(0)?;
                let id = Some(id as u64);
                Ok( 
                    Asset {
                        id,
                        name: row.get(1)?,
                        wkn: row.get(2)?,
                        isin: row.get(3)?,
                        note: row.get(4)?,
                    }
                )
            }

            ).map_err(|e| DataError::NotFound(e.to_string()))?;	
        Ok(asset)
    }

    fn update_asset(&self, asset: &Asset) -> Result<(), DataError> {
        if asset.id.is_none() {
            return Err(DataError::NotFound("not yet stored to database".to_string()));
        }
        let id = asset.id.unwrap() as i64;
        self.conn.execute(
            "UPDATE assets SET name=?2, wkn=?3, isin=?4, note=?5 
                WHERE id=?1;",
            params![id, asset.name, asset.wkn, asset.isin, asset.note]
        ).map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }

    fn insert_asset(&self, asset: &Asset) -> Result<u64, DataError> {
        self.conn.execute(
            "INSERT INTO assets (name, wkn, isin, note) VALUES (?1, ?2, ?3, ?4)",
            params![asset.name, asset.wkn, asset.isin, asset.note]
        ).map_err(|e| DataError::InsertFailed(e.to_string()))?;
        let id = self.conn.query_row("SELECT id FROM assets
        WHERE name=?;", params![asset.name],
         |row| { 
                let id: i64 = row.get(0)?;
                Ok( id as u64)
            }

            ).map_err(|e| DataError::NotFound(e.to_string()))?;	
        Ok(id)
    }

    fn delete_asset(&self, id: u64) -> Result<(), DataError> {
        self.conn.execute(
            "DELETE FROM assets WHERE id=?1;", params![id as i64]
        ).map_err(|e| DataError::InsertFailed(e.to_string()))?;
        Ok(())
    }
}

