
use std::fs;
use chrono::Weekday;

use finql::calendar::Holiday;
use finql_data::ObjectHandler;
use finql_sqlite::SqliteDBPool;

#[tokio::main]
async fn main() {
// we use a calendar as sample object
    let holidays = vec![
        // Saturdays
        Holiday::WeekDay(Weekday::Sat),
        // Sundays
        Holiday::WeekDay(Weekday::Sun),
        // New Year's day
        Holiday::MovableYearlyDay {
            month: 1,
            day: 1,
            first: None,
            last: None,
        },
        // Good Friday
        Holiday::EasterOffset {
            offset: -2,
            first: None,
            last: None,
        }    
    ];

    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() >= 2,
        "usage: store_object <sqlite3 database file path>\n"
    );
    let path = &args[1];
    if fs::metadata(path).is_ok() {
        eprintln!("Apparently there exists already a file with this path.");
        eprintln!("Please provide another path or remove the file, since a new database will be created.");
    } else {
        let db_pool = SqliteDBPool::open(&std::path::Path::new(path)).await.unwrap();
        let db = db_pool.get_conection().await.unwrap();
        db.init().await.unwrap();

        db.store_object("test", "calendar", &holidays).await.unwrap();
        
        let new_holiday: Vec<Holiday> = db.get_object("test").await.unwrap();
        println!("New holiday struct: {:?}", new_holiday);
    }
}
