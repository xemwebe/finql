///! Demonstration of storing Assets in Sqlite3 database
use finql::datatypes::{
    Asset, CashFlow, CurrencyISOCode, Stock, Transaction, TransactionHandler, TransactionType,
};
use finql::postgres::PostgresDB;

async fn transaction_tests(db: &dyn TransactionHandler) {
    print!("Store asset...");
    let asset = Asset::Stock(Stock::new(
        None,
        "Admiral Group plc".to_string(),
        Some("GB00B02J6398".to_string()),
        Some("AODJ58".to_string()),
        Some("Here are my notes".to_string()),
    ));
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
    let eur = db
        .get_or_new_currency(CurrencyISOCode::new("EUR").unwrap())
        .await
        .unwrap();
    let cash_flow = CashFlow::new(10_000.0, eur, NaiveDate::from_ymd_opt(2020, 01, 15))?;
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
    let cash_flow = CashFlow::new(-9_000.0, eur, NaiveDate::from_ymd_opt(2020, 01, 15))?;
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
        cash_flow: CashFlow::new(-30.0, eur, NaiveDate::from_ymd_opt(2020, 01, 15)?),
        note: None,
    };
    let _ = db.insert_transaction(&fee).await.unwrap();
    println!("ok");

    // You got some dividends later
    print!("Store dividend transaction...");
    let dividend = Transaction {
        id: None,
        transaction_type: TransactionType::Dividend { asset_id: 1 },
        cash_flow: CashFlow::new(90.0, eur, NaiveDate::from_ymd_opt(2020, 01, 30)?),
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
        cash_flow: CashFlow::new(-40.0, eur, NaiveDate::from_ymd_opt(2020, 01, 30)?),
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
    if args.len() != 2 {
        eprintln!("usage: {} <database connection string>]", args[0]);
        return;
    }
    let db = PostgresDB::new(args[1].as_str()).await.unwrap();
    db.clean().await.unwrap();

    transaction_tests(&db).await;
}
