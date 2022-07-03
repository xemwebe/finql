///! Example storing general calendars as JSON object in PostgreSQL
///! Please note: All existing content of the database will be deleted!
use chrono::Weekday;

use cal_calc::Holiday;
use finql::datatypes::ObjectHandler;
use finql::postgres::PostgresDB;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} <database connection string>]", args[0]);
        return;
    }
    let db = PostgresDB::new(args[1].as_str()).await.unwrap();
    db.clean().await.unwrap();

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
        },
    ];

    db.store_object("test", "calendar", &holidays)
        .await
        .unwrap();

    let new_holiday: Vec<Holiday> = db.get_object("test").await.unwrap();
    println!("New holiday struct: {:?}", new_holiday);
}
