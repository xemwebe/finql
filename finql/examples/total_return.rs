///! Demonstrate total return calculation by single investment in dividend stock
use std::{sync::Arc, str::FromStr};
use std::cmp::min;
use std::error::Error;
use std::ops::Range;

use chrono::{Utc, NaiveDate, Datelike};
use plotters::prelude::*;

use finql_data::{Asset, CashFlow, Currency, Ticker, QuoteHandler, Transaction, TransactionType, date_time_helper::naive_date_to_date_time};
use finql::{
    Market,
    time_period::TimePeriod, 
    market_quotes::{
        MarketDataSource},
    portfolio::{
        PortfolioPosition,
        calc_delta_position,
    },
    strategy::{
        Strategy, 
        InvestAllInSingleStock,
    },
    calendar::last_day_of_month,
};
use finql_sqlite::SqliteDB;

#[derive(Debug)]
struct TimeValue {
    date: NaiveDate,
    value: f64,
}

#[tokio::main]
async fn main() {
    println!("Calculate total return of single investment of 10'000 USD in Broadcom (AVGO) five years before today");
    let today = Utc::now().naive_local().date();
    let five_years_before = "-5Y".parse::<TimePeriod>().unwrap();
    let start = five_years_before.add_to(today, None);
    println!("The simulation will run from {} until {}.", start, today);

    // Prepare database
    let sqlite_db = SqliteDB::new("sqlite::memory:").await.unwrap();
    sqlite_db.init().await.unwrap();
    let db: Arc<dyn QuoteHandler+Send+Sync> = Arc::new(sqlite_db);

    // Define the asset
    let asset = Asset::new(
        None,
        "Broadcom Inc.",
        None,
        None,
        None,
    );
    let asset_id = db.insert_asset(&asset).await.unwrap();

    println!("Get price history and dividends for AVGO");
    let mut market = Market::new(db.clone());
    let yahoo = MarketDataSource::Yahoo;
    let quote_provider = yahoo.get_provider(String::new()).unwrap();
    market.add_provider(
        yahoo.to_string(),
        quote_provider.clone(),
    );
    let usd = Currency::from_str("USD").unwrap();
    let ticker = Ticker {
        id: None,
        asset: asset_id,
        name: "AVGO".to_string(),
        currency: usd,
        source: yahoo.to_string(),
        priority: 1,
        factor: 1.0,
    };
    let ticker_id = db.insert_ticker(&ticker).await.unwrap();
    let start_time = naive_date_to_date_time(&start, 0);
    let end_time = naive_date_to_date_time(&today, 20);
    market.update_quote_history(ticker_id, start_time, end_time).await.unwrap();
    
    let dividends = quote_provider.fetch_dividend_history(&ticker, 
        start_time, end_time).await.unwrap();
    println!("Found {} dividends", dividends.len());

    // put some cash into the account
    println!("Setup initial cash transaction...");
    let cash_flow = CashFlow::new(10_000.0, usd, start);
    let cash_in = Transaction {
        id: None,
        transaction_type: TransactionType::Cash,
        cash_flow,
        note: Some("start capital".to_string()),
    };
    let mut transactions = Vec::new();
    transactions.push(cash_in);

    let strategy = InvestAllInSingleStock::new(asset_id, ticker_id, market);

    let mut current_date = start;
    let mut total_return = Vec::new();

    total_return.push(TimeValue{ value: cash_flow.amount.amount, date: current_date});
    let mut position = PortfolioPosition::new(usd);
    //let (mut position, mut totals) = calculate_position_and_pnl(usd, &transactions, None, db.clone()).await.unwrap();
    let market = Market::new(db.clone());
    while current_date < today {
        // roll position forward to next day
        let next_date = min(today, strategy.next_day(current_date));
        let previous_position = position.clone();
        calc_delta_position(
            &mut position,
            &transactions,
            Some(current_date),
            Some(next_date)).unwrap();
        
        // Update list of transactions with new strategic transactions for the current day
        let mut new_transactions = strategy.apply(&position, next_date).await.unwrap();
        transactions.append(&mut new_transactions);
        
        // Calculate new position including new transactions
        position = previous_position;
        calc_delta_position(
            &mut position,
            &transactions,
            Some(current_date),
            Some(next_date)).unwrap();

        current_date = next_date;
        position.add_quote(naive_date_to_date_time(&current_date, 20), &market).await;
        let totals = position.calc_totals();
        total_return.push(TimeValue{ value: totals.value, date: current_date});
    }

    // plot the graph
    make_plot("total_return.png", "Total Return", &total_return).unwrap();
}

fn min_max(time_series: &[TimeValue]) -> (f64, f64) {
    if time_series.len() == 0 {
        return (0.0,0.0);
    }
    let mut min = time_series[0].value;
    let mut max = min;
    for v in time_series {
        if min>v.value {
            min = v.value;
        } 
        if max<v.value {
            max = v.value;
        }
   }
   (min,max)
}

fn calc_ranges(time_series: &[TimeValue]) -> (Range<NaiveDate>, Range<f64>) {
    let start_date = time_series[0].date;
    let end_date = time_series.last().unwrap().date;
    let (start_value, end_value) = min_max(time_series);

    let first_day = NaiveDate::from_ymd(start_date.year(), start_date.month(), 1);
    let last_year = end_date.year();
    let last_month = end_date.month();
    let last_day = NaiveDate::from_ymd(last_year, last_month, last_day_of_month(last_year, last_month));
    let date_range = first_day..last_day;
    ( date_range, start_value..end_value)
}

fn make_plot(file_name: &str, title: &str, time_series: &[TimeValue]) -> Result<(), Box<dyn Error>> {
    
    let root = BitMapBackend::new(file_name, (1024, 768)).into_drawing_area();

    root.fill(&WHITE)?;

    let (x_range, y_range) = calc_ranges(time_series);
    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption(title, ("sans-serif", 40))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(x_range.monthly(), y_range)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(30)
        .y_desc("Total position value (€)")
        .draw()?;

    chart.draw_series(LineSeries::new(
        time_series.iter().map(|v| (v.date, v.value) ),
        &RED,
    ))?;

    Ok(())
}

