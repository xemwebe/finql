///! Demonstration of storing quotes and related data in Sqlite3, PostgreSQL or in-memory database
///! Please note: The postgres example will delete all existing content of the database
use std::fs;
use std::io::{stdout, Write};
use std::str::FromStr;
use std::sync::Arc;

use finql_data::{Asset, Currency, CurrencyConverter, Quote, Ticker, QuoteHandler, date_time_helper::make_time};
use finql::fx_rates::insert_fx_quote;
use finql::market::Market;
use finql::market_quotes::MarketDataSource;
use finql_sqlite::SqliteDB;

fn log(s: &str) {
    print!("{}", s);
    stdout().flush().unwrap();
}

async fn quote_tests(market: &mut Market) {
    // We need some assets the quotes are related
    let basf_id = market
        .db()
        .insert_asset(&Asset {
            id: None,
            name: "BASF AG".to_string(),
            wkn: None,
            isin: None,
            note: None,
        })
        .await.unwrap();
    let siemens_id = market
        .db()
        .insert_asset(&Asset {
            id: None,
            name: "Siemens AG".to_string(),
            wkn: None,
            isin: None,
            note: None,
        })
        .await.unwrap();
    let bhp_id = market
        .db()
        .insert_asset(&Asset {
            id: None,
            name: "BHP Inc.".to_string(),
            wkn: None,
            isin: None,
            note: None,
        })
        .await.unwrap();

    // Create some market data sources
    let yahoo = MarketDataSource::Yahoo;

    // Dealing with ticker data
    let eur = Currency::from_str("EUR").unwrap();
    let aus = Currency::from_str("AUS").unwrap();
    // Insert ticker
    log("Insert quote ticker...");
    let basf = Ticker {
        id: None,
        name: "BAS.DE".to_string(),
        asset: basf_id,
        currency: eur,
        priority: 10,
        source: yahoo.to_string(),
        factor: 1.0,
    };
    let basf_id = market.db().insert_ticker(&basf).await.unwrap();
    // Get ticker back
    market.db().get_ticker_by_id(basf_id).await.unwrap();
    // Insert another ticker, same source
    let siemens = Ticker {
        id: None,
        name: "SIE.DE".to_string(),
        asset: siemens_id,
        priority: 10,
        currency: eur,
        source: yahoo.to_string(),
        factor: 1.0,
    };
    let siemens_id = market.db().insert_ticker(&siemens).await.unwrap();
    // Insert another ticker, with other source
    let mut bhp = Ticker {
        id: None,
        name: "BHP.AUS".to_string(),
        asset: bhp_id,
        priority: 10,
        currency: eur,
        source: MarketDataSource::Manual.to_string(),
        factor: 1.0,
    };
    let bhp_id = market.db().insert_ticker(&bhp).await.unwrap();
    println!("ok");
    // Ups, wrong currency, update ticker
    bhp.id = Some(bhp_id);
    bhp.currency = aus;
    // Show all ticker with yahoo as market source
    log("Update ticker...");
    market.db().update_ticker(&bhp).await.unwrap();
    println!("ok");
    log("Get all ticker by source...");
    let tickers = market.db().get_all_ticker_for_source(&yahoo.to_string()).await.unwrap();
    if tickers.len() == 2 {
        println!("ok");
    } else {
        println!("failed");
    }

    // Don't need this ticker anymore, delete
    log("Delete ticker...");
    market.db().delete_ticker(bhp_id).await.unwrap();
    println!("ok");

    // Dealing with quotes
    log("Insert market quotes...");
    let time = make_time(2019, 12, 30, 20, 0, 0).unwrap();
    let quote = Quote {
        id: None,
        ticker: basf_id,
        price: 67.35,
        time,
        volume: None,
    };
    market.db().insert_quote(&quote).await.unwrap();
    let time = make_time(2020, 1, 2, 20, 0, 0).unwrap();
    let quote = Quote {
        id: None,
        ticker: basf_id,
        price: 68.29,
        time,
        volume: None,
    };
    market.db().insert_quote(&quote).await.unwrap();
    let time = make_time(2020, 1, 3, 20, 0, 0).unwrap();
    let quote = Quote {
        id: None,
        ticker: basf_id,
        price: 67.27,
        time,
        volume: None,
    };
    market.db().insert_quote(&quote).await.unwrap();
    let time = make_time(2020, 1, 6, 20, 0, 0).unwrap();
    let quote = Quote {
        id: None,
        ticker: basf_id,
        price: 66.27,
        time,
        volume: None,
    };
    market.db().insert_quote(&quote).await.unwrap();
    let time = make_time(2020, 1, 7, 20, 0, 0).unwrap();
    let quote = Quote {
        id: None,
        ticker: basf_id,
        price: 66.30,
        time,
        volume: None,
    };
    market.db().insert_quote(&quote).await.unwrap();
    let time = make_time(2020, 1, 8, 20, 0, 0).unwrap();
    let mut wrong_quote = Quote {
        id: None,
        ticker: siemens_id,
        price: 65.73,
        time,
        volume: None,
    };
    let wrong_quote_id = market.db().insert_quote(&wrong_quote).await.unwrap();
    println!("ok");
    let time = make_time(2020, 1, 4, 0, 0, 0).unwrap();
    log("get last quote...");
    let (quote, currency) = market.db().get_last_quote_before("BASF AG", time).await.unwrap();
    if currency == eur && (quote.price - 67.27) < 1e-10 {
        println!("ok");
    } else {
        println!("failed");
    }
    log("get all quotes for ticker...");
    let quotes = market.db().get_all_quotes_for_ticker(basf_id).await.unwrap();
    if quotes.len() == 5 {
        println!("ok");
    } else {
        println!("failed");
    }
    println!("List of quotes in db: {:?}", quotes);
    
    // correct wrong quote
    log("update quote...");
    wrong_quote.id = Some(wrong_quote_id);
    wrong_quote.ticker = basf_id;
    market.db().update_quote(&wrong_quote).await.unwrap();
    println!("ok");
    // Maybe deleting strange quote is better...
    log("delete quote...");
    market.db().delete_quote(wrong_quote_id).await.unwrap();
    println!("ok");
    log("insert fx quote...");
    insert_fx_quote(0.9, aus, eur, time, market.db()).await.unwrap();
    println!("ok");
    log("read fx quote...");
    let fx1 = market.fx_rate(aus, eur, time).await.unwrap();
    println!("ok");
    log("read inverse fx quote...");
    let fx2 = market.fx_rate(eur, aus, time).await.unwrap();
    println!("ok");
    log("sanity check fx quotes...");
    if (fx1 * fx2).abs() > 1.0e-10 {
        println!("ok");
    } else {
        println!("not ok: fx1: {}, fx2: {}, fx1*fx2: {}", fx1, fx2, fx1 * fx2);
    }

    log("insert rounding convention...");
    let xxx = Currency::from_str("XXX").unwrap();
    market.db().set_rounding_digits(xxx, 3).await.unwrap();
    println!("ok");
    log("read rounding convention...");

    let digits = market.db().get_rounding_digits(xxx).await;
    if digits == 3 {
        println!("ok");
    } else {
        println!("not ok: got rounding digits {}, expected 3", digits);
    }

    log("Check default rounding convention...");
    let digits = market.db().get_rounding_digits(eur).await;
    if digits == 2 {
        println!("ok");
    } else {
        println!("not ok: default rounding digits is {} instead of 2", digits);
    }

    log("Check update of quotes...");
    market.add_provider(
        yahoo.to_string(),
        yahoo.get_provider(String::new()).unwrap(),
    );
    market.update_quotes().await.unwrap();
    println!("ok");
    println!("\nDone.");
}


#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() >= 2,
        "usage: quotes_db <sqlite3 database file path>\n"
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
        let qh: Arc<dyn QuoteHandler+Sync+Send> = Arc::new(db);
        let mut market = Market::new(qh);            
        quote_tests(&mut market).await;
        println!("You may have a look at the database for further inspection.");
    }
}
