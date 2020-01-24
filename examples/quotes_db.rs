///! Demonstration of storing quotes and related data in Sqlite3, PostgreSQL or in-memory database
///! Please note: The postgres example will delete all existing content of the database

use finql::currency::Currency;
use finql::data_handler::QuoteHandler;
use finql::helpers::make_time;
use finql::memory_handler::InMemoryDB;
use finql::postgres_handler::PostgresDB;
use finql::quote::{MarketDataSource, Quote, Ticker};
use finql::sqlite_handler::SqliteDB;
use std::fs;
use std::str::FromStr;

fn quote_tests<DB: QuoteHandler>(db: &mut DB) {
    // Create some market data sources
    let yahoo = MarketDataSource {
        id: None,
        name: "yahoo!".to_string(),
    };
    let guru_focus = MarketDataSource {
        id: None,
        name: "gurufocus".to_string(),
    };
    let mut alpha_vantage = MarketDataSource {
        id: None,
        name: "alpha_vantage".to_string(),
    };

    // Store market data sources
    print!("Insert new market data source...");
    let yahoo_id = db.insert_md_source(&yahoo).unwrap();
    println!("ok");

    // get back source
    print!("Reading market data source from db...");
    db.get_md_source_by_id(yahoo_id).unwrap();
    let gf_id = db.insert_md_source(&guru_focus).unwrap();
    let av_id = db.insert_md_source(&alpha_vantage).unwrap();
    println!("ok");
    // update source
    print!("Updating market data source...");
    alpha_vantage.id = Some(av_id);
    alpha_vantage.name = "AlphaVantage".to_string();
    db.update_md_source(&alpha_vantage).unwrap();
    println!("ok");
    // delete source
    print!("Deleting market data source...");
    db.delete_md_source(gf_id).unwrap();
    println!("ok");
    // Get all sources
    print!("Get list of all market data sources...");
    let sources = db.get_all_md_sources().unwrap();
    if sources.len() == 2 {
        println!("ok");
    } else {
        println!("failed");
    }

    // Dealing with ticker data
    let eur = Currency::from_str("EUR").unwrap();
    let aus = Currency::from_str("AUS").unwrap();
    // Insert ticker
    print!("Insert quote ticker...");
    let basf = Ticker {
        id: None,
        name: "BAS.DE".to_string(),
        currency: eur,
        source: yahoo_id,
    };
    let basf_id = db.insert_ticker(&basf).unwrap();
    // Get ticker back
    db.get_ticker_by_id(basf_id).unwrap();
    // Insert another ticker, same source
    let siemens = Ticker {
        id: None,
        name: "SIE.DE".to_string(),
        currency: eur,
        source: yahoo_id,
    };
    let siemens_id = db.insert_ticker(&siemens).unwrap();
    // Insert another ticker, with other source
    let mut bhp = Ticker {
        id: None,
        name: "BHP.AUS".to_string(),
        currency: eur,
        source: av_id,
    };
    let bhp_id = db.insert_ticker(&bhp).unwrap();
    println!("ok");
    // Ups, wrong currency, update ticker
    bhp.id = Some(bhp_id);
    bhp.currency = aus;
    // Show all ticker with yahoo as market source
    print!("Update ticker...");
    db.update_ticker(&bhp).unwrap();
    println!("ok");
    print!("Get all ticker by source...");
    db.get_all_ticker_for_source(yahoo_id).unwrap();
    if sources.len() == 2 {
        println!("ok");
    } else {
        println!("failed");
    }
    // Don't need this ticker anymore, delete
    print!("Delete ticker...");
    db.delete_ticker(bhp_id).unwrap();
    println!("ok");

    // Dealing with quotes
    print!("Insert market quotes...");
    let time = make_time(2019, 12, 30, 20, 0, 0).unwrap();
    let quote = Quote {
        id: None,
        ticker: basf_id,
        price: 67.35,
        time,
        volume: None,
    };
    db.insert_quote(&quote).unwrap();
    let time = make_time(2020, 1, 2, 20, 0, 0).unwrap();
    let quote = Quote {
        id: None,
        ticker: basf_id,
        price: 68.29,
        time,
        volume: None,
    };
    db.insert_quote(&quote).unwrap();
    let time = make_time(2020, 1, 3, 20, 0, 0).unwrap();
    let quote = Quote {
        id: None,
        ticker: basf_id,
        price: 67.27,
        time,
        volume: None,
    };
    db.insert_quote(&quote).unwrap();
    let time = make_time(2020, 1, 6, 20, 0, 0).unwrap();
    let quote = Quote {
        id: None,
        ticker: basf_id,
        price: 66.27,
        time,
        volume: None,
    };
    db.insert_quote(&quote).unwrap();
    let time = make_time(2020, 1, 7, 20, 0, 0).unwrap();
    let quote = Quote {
        id: None,
        ticker: basf_id,
        price: 66.30,
        time,
        volume: None,
    };
    db.insert_quote(&quote).unwrap();
    let time = make_time(2020, 1, 8, 20, 0, 0).unwrap();
    let mut wrong_quote = Quote {
        id: None,
        ticker: siemens_id,
        price: 65.73,
        time,
        volume: None,
    };
    let wrong_quote_id = db.insert_quote(&wrong_quote).unwrap();
    println!("ok");
    let time = make_time(2020, 1, 4, 0, 0, 0).unwrap();
    print!("get last quote...");
    let (quote, currency) = db.get_last_quote_before(basf_id, time).unwrap();
    if currency == eur && (quote.price - 67.27) < 1e-10 {
        println!("ok");
    } else {
        println!("failed");
    }
    print!("get all quotes for ticker...");
    let quotes = db.get_all_quotes_for_ticker(basf_id).unwrap();
    if quotes.len() == 5 {
        println!("ok");
    } else {
        println!("failed");
    }
    // correct wrong quote
    print!("update quote...");
    wrong_quote.id = Some(wrong_quote_id);
    wrong_quote.ticker = basf_id;
    db.update_quote(&wrong_quote).unwrap();
    println!("ok");
    // Maybe deleting strange quote is better...
    print!("delete quote...");
    db.delete_quote(wrong_quote_id).unwrap();
    println!("ok");
    println!("\nDone. You may have a look at the database for further inspection.");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() >= 2,
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
            let mut db = InMemoryDB::new();
            quote_tests(&mut db);
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
                    let mut db = SqliteDB::create(path).unwrap();
                    quote_tests(&mut db);
                }
            }
        }
        "postgres" => {
            if args.len() < 3 {
                eprintln!("Please give the connection string to PostgreSQL as parameter, in the form of");
                eprintln!("'host=127.0.0.1 user=<username> password=<password> dbname=<database name> sslmode=disable'");
            } else {
                let connect_str = &args[2];
                let mut db = PostgresDB::connect(connect_str).unwrap();
                db.clean().unwrap();
                quote_tests(&mut db);
            }
        }
        other => println!("Unknown database type {}", other),
    }
}
