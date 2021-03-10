///! Demonstration of storing Assets in Sqlite3 database

use std::fs;
use std::str::FromStr;
use chrono::NaiveDate;

use finql_data::{Asset, Currency, CashFlow, Transaction, TransactionHandler, TransactionType};
use finql_sqlite::SqliteDB;

async fn transaction_tests(db: &mut dyn TransactionHandler) {
    print!("Store asset...");
    let asset = Asset::new(
        None,
        "Admiral Group plc",
        Some("AODJ58".to_string()),
        Some("GB00B02J6398".to_string()),
        Some("Here are my notes".to_string()),
    );
    let asset_id = db.insert_asset(&asset).await;
    match asset_id {
        Ok(_) => println!("ok"),
        Err(err) => println!("failed with error: {}", err),
    }

    // Show all assets in database
    print!("Get list of assets...");
    let _assets = db.get_all_assets().await.unwrap();
    println!("ok");

    // put some cash into the account
    print!("Store cash transaction...");
    let eur = Currency::from_str("EUR").unwrap();
    let cash_flow = CashFlow::new(10_000.0, eur, NaiveDate::from_ymd(2020, 01, 15));
    let cash_in = Transaction {
        id: None,
        transaction_type: TransactionType::Cash,
        cash_flow,
        note: Some("start capital".to_string()),
    };
    let result = db.insert_transaction(&cash_in).await;
    match result {
        Ok(_) => println!("ok"),
        Err(err) => println!("failed with error: {}", err),
    };

    // Lets buy the asset!
    print!("Store buy asset transaction...");
    let cash_flow = CashFlow::new(-9_000.0, eur, NaiveDate::from_ymd(2020, 01, 15));
    let asset_buy = Transaction {
        id: None,
        transaction_type: TransactionType::Asset {
            asset_id: 1,
            position: 10.0,
        },
        cash_flow,
        note: None,
    };
    let trans_id = db.insert_transaction(&asset_buy).await.unwrap();
    println!("ok");

    print!("Store associated fee transaction...");
    // Associate some fees with the trade
    let fee = Transaction {
        id: None,
        transaction_type: TransactionType::Fee {
            transaction_ref: Some(trans_id),
        },
        cash_flow: CashFlow::new(-30.0, eur, NaiveDate::from_ymd(2020, 01, 15)),
        note: None,
    };
    let _ = db.insert_transaction(&fee).await.unwrap();
    println!("ok");

    // You got some dividends later
    print!("Store dividend transaction...");
    let dividend = Transaction {
        id: None,
        transaction_type: TransactionType::Dividend { asset_id: 1 },
        cash_flow: CashFlow::new(90.0, eur, NaiveDate::from_ymd(2020, 01, 30)),
        note: None,
    };
    let dividend_id = db.insert_transaction(&dividend).await.unwrap();
    println!("ok");

    // But you get taxed, too!
    print!("Insert related tax transaction...");
    let tax = Transaction {
        id: None,
        transaction_type: TransactionType::Tax {
            transaction_ref: Some(dividend_id),
        },
        cash_flow: CashFlow::new(-40.0, eur, NaiveDate::from_ymd(2020, 01, 30)),
        note: None,
    };
    let _ = db.insert_transaction(&tax).await.unwrap();
    println!("ok");

    // Show all transactions in database
    print!("Get list of all transactions...");
    let _transactions = db.get_all_transactions().await.unwrap();
    println!("ok");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() >= 2 && args.len() <= 3,
        format!(
            concat!(
                "usage: {} <db_type> [<database connection string>]\n",
                "where <db_type> is any of 'sqlite' or 'memory'"
            ),
            args[0]
        )
    );
    match args[1].as_str() {
        "memory" => {
            let mut db = SqliteDB::new("sqlite::memory:").await.unwrap();
            db.init().await.unwrap();
            transaction_tests(&mut db).await;
        }
        "sqlite" => {
            if args.len() < 3 {
                eprintln!("Please give the sqlite database path as parameter");
            } else {
                let path = &args[2];
                if fs::metadata(path).is_ok() {
                    eprintln!("Apparently there exists already a file with this path.");
                    eprintln!("Please provide another path or remove the file, since a new database will be created.");
                } else {
                    let conn = format!("sqlite:{}", path);
                    let mut db = SqliteDB::new(&conn).await.unwrap();
                    db.init().await.unwrap();
                    transaction_tests(&mut db).await;
                }
            }
        }
        other => println!("Unknown database type {}", other),
    }
}
