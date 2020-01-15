use chrono::NaiveDate;
///! Demonstration of storing Assets in Sqlite3 database
use finql::asset::Asset;
use finql::currency::Currency;
use finql::data_handler::DataHandler;
use finql::fixed_income::CashFlow;
use finql::sqlite_handler::SqliteDB;
use finql::transaction::{Transaction, TransactionType};
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() == 2,
        format!("usage: {} <name of sqlite3 db>", args[0])
    );
    let db = SqliteDB::create(&args[1]).unwrap();
    let asset = Asset::new(
        None,
        "Admiral Group plc",
        Some("AODJ58".to_string()),
        Some("GB00B02J6398".to_string()),
        Some("Here are my notes".to_string()),
    );
    let asset_id = db.insert_asset(&asset);
    match asset_id {
        Ok(id) => println!("Asset has been stored successfully with id {}", id),
        Err(err) => println!(
            "Could not insert asset! Did the database already contained the entry? Error was {}",
            err
        ),
    }

    // Show all assets in database
    let assets = db.get_all_assets().unwrap();
    println!("Here comes a list of all stored assets:\n{:?}", assets);

    // put some cash into the account
    let eur = Currency::from_str("EUR").unwrap();
    let cash_flow = CashFlow::new(10_000.0, eur, NaiveDate::from_ymd(2020, 01, 15));
    let cash_in = Transaction {
        id: None,
        transaction_type: TransactionType::Cash,
        cash_flow,
        note: Some("start capital".to_string()),
    };
    let result = db.insert_transaction(&cash_in);
    match result {
        Ok(_) => println!("Start capital has been stored successfully!"),
        Err(err) => println!("Could not insert start capital! Error was {}", err),
    };

    // Lets buy the asset!
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
    let trans_id = db.insert_transaction(&asset_buy).unwrap();

    // Associate some fees with the trade
    let fee = Transaction {
        id: None,
        transaction_type: TransactionType::Fee {
            transaction_ref: Some(trans_id),
        },
        cash_flow: CashFlow::new(-30.0, eur, NaiveDate::from_ymd(2020, 01, 15)),
        note: None,
    };
    let _ = db.insert_transaction(&fee).unwrap();

    // You got some dividends later
    let dividend = Transaction {
        id: None,
        transaction_type: TransactionType::Dividend { asset_id: 1 },
        cash_flow: CashFlow::new(90.0, eur, NaiveDate::from_ymd(2020, 01, 30)),
        note: None,
    };
    let dividend_id = db.insert_transaction(&dividend).unwrap();

    // But you get taxed, too!
    let tax = Transaction {
        id: None,
        transaction_type: TransactionType::Tax {
            transaction_ref: Some(dividend_id),
        },
        cash_flow: CashFlow::new(-40.0, eur, NaiveDate::from_ymd(2020, 01, 30)),
        note: None,
    };
    let _ = db.insert_transaction(&tax).unwrap();

    // Show all transactions in database
    let transactions = db.get_all_transactions().unwrap();
    println!("Here comes a list of all stored transactions:");
    for transaction in transactions {
        println!("{:?}", transaction);
    }
}
