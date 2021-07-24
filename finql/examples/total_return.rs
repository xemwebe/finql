///! Demonstrate total retun calculation by single investment in dividend stock
use std::{sync::Arc, str::FromStr};

use std::time::{Duration, UNIX_EPOCH};
use chrono::{DateTime, Utc};

use finql_data::{Currency, Ticker, QuoteHandler, TransactionHandler, date_time_helper::naive_date_to_date_time};
use finql::{Market, time_period::TimePeriod, market_quotes::{yahoo::Yahoo, MarketQuoteProvider}};
use finql_sqlite::SqliteDB;

#[tokio::main]
async fn main() {
    println!("Calculate total return of single investment of 10'000 USD in Broadcom (AVGO) five years before today");
    let today = Utc::now().naive_local().date();
    let five_years_before = "-5Y".parse::<TimePeriod>().unwrap();
    let start = five_years_before.add_to(today, None);
    println!("The simulation will run from {} until {}.", start, today);

    println!("Get price history and dividends for AVGO");
    let usd = Currency::from_str("USD").unwrap();
    let quote_provider   = Yahoo{};
    let ticker = Ticker {
        id: Some(0),
        asset: 0,
        name: "AVGO".to_string(),
        currency: usd,
        source: "yahoo".to_string(),
        priority: 1,
        factor: 1.0,
    };
    let quotes = quote_provider.fetch_quote_history(&ticker, 
        naive_date_to_date_time(&start, 20), 
        naive_date_to_date_time(&today, 20)).await.unwrap();
    let dividends = quote_provider.fetch_dividend_history(&ticker, 
        naive_date_to_date_time(&start, 20), 
        naive_date_to_date_time(&today, 20)).await.unwrap();
    println!("Quotes: {:?}", dividends);
}

//     // put some cash into the account
//     print!("Store cash transaction...");
//     let eur = Currency::from_str("EUR").unwrap();
//     let cash_flow = CashFlow::new(10_000.0, eur, NaiveDate::from_ymd(2020, 01, 15));
//     let cash_in = Transaction {
//         id: None,
//         transaction_type: TransactionType::Cash,
//         cash_flow,
//         note: Some("start capital".to_string()),
//     };
//     let result = db.insert_transaction(&cash_in).await;
//     match result {
//         Ok(_) => println!("ok"),
//         Err(err) => println!("failed with error: {}", err),
//     };

//     // Lets buy the asset!
//     print!("Store buy asset transaction...");
//     let cash_flow = CashFlow::new(-9_000.0, eur, NaiveDate::from_ymd(2020, 01, 15));
//     let asset_buy = Transaction {
//         id: None,
//         transaction_type: TransactionType::Asset {
//             asset_id: 1,
//             position: 10.0,
//         },
//         cash_flow,
//         note: None,
//     };
//     let trans_id = db.insert_transaction(&asset_buy).await.unwrap();
//     println!("ok");
    
//     println!("Setup initial transactions (ash and buy initial position");
//     let db = Arc::new(SqliteDB::new("sqlite::memory:").await.unwrap());
//     db.init().await.unwrap();
//     let market = Market::new(db.clone());       

//     // Define the asset
//     let asset = Asset::new(
//         None,
//         "Broadcom Inc.",
//         None,
//         None,
//         None,
//     );
//     let asset_id = db.insert_asset(&asset).await.unwrap();

//     // put some cash into the account
//     let usd = Currency::from_str("USD").unwrap();
//     let cash_flow = CashFlow::new(10_000.0, usd, start);
//     let cash_in = Transaction {
//         id: None,
//         transaction_type: TransactionType::Cash,
//         cash_flow,
//         note: Some("start capital".to_string()),
//     };
//     let result = db.insert_transaction(&cash_in).await.unwrap();

//     // Lets buy the asset!
//     print!("Store buy asset transaction...");
//     let cash_flow = CashFlow::new(-9_000.0, eur, NaiveDate::from_ymd(2020, 01, 15));
//     let asset_buy = Transaction {
//         id: None,
//         transaction_type: TransactionType::Asset {
//             asset_id: 1,
//             position: 10.0,
//         },
//         cash_flow,
//         note: None,
//     };
//     let trans_id = db.insert_transaction(&asset_buy).await.unwrap();


// }
