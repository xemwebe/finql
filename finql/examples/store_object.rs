
use std::fs;
use chrono::Weekday;

use finql::calendar::Holiday;
use finql_data::ObjectHandler;
use finql_sqlite::SqliteDB;

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
        let conn = format!("sqlite:{}", path);
        { let _= fs::File::create(path); }
        let db = SqliteDB::new(&conn).await.unwrap();
        db.init().await.unwrap();

        db.store_object("test", &holidays).await.unwrap();
        
        let new_holiday: Vec<Holiday> = db.get_object("test").await.unwrap();
        println!("New holiday struct: {:?}", new_holiday);
    }
}
