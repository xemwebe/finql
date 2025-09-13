use std::cmp::min;
use std::error::Error;
///! Demonstrate total return calculation by single investment in dividend stock
///! Example storing general calendars as JSON object in PostgreSQL
///! Please note: All existing content of the database will be deleted!
use std::sync::Arc;

use log::debug;
use plotters::prelude::*;
use pretty_env_logger;
use time::{Date, Month};

use cal_calc::last_day_of_month;
use finql::datatypes::{
    date_time_helper::make_time, Asset, CashFlow, Currency, QuoteHandler, Stock, Ticker,
    Transaction, TransactionType,
};
use finql::postgres::PostgresDB;
use finql::{
    market::CachePolicy,
    market_quotes::MarketDataSource,
    portfolio::{calc_delta_position, PortfolioPosition},
    strategy::{
        ReInvestInSingleStock, StaticInSingleStock, StockTransactionCosts, StockTransactionFee,
        Strategy,
    },
    time_period::TimePeriod,
    time_series::{TimeSeries, TimeSeriesError, TimeValue},
    Market,
};

async fn calc_strategy(
    currency: Currency,
    start_transactions: &Vec<Transaction>,
    strategy: &dyn Strategy,
    start: Date,
    end: Date,
    market: Market,
) -> Vec<TimeValue> {
    let mut current_date = start;
    let mut total_return = Vec::new();
    let mut transactions = start_transactions.clone();

    let mut position = PortfolioPosition::new(currency);
    calc_delta_position(
        &mut position,
        &transactions,
        Some(start),
        Some(start),
        market.clone(),
    )
    .await
    .unwrap();

    position
        .add_quote(
            naive_date_to_date_time(&start, 20, None).unwrap(),
            market.clone(),
        )
        .await;
    //let totals = position.calc_totals();
    //total_return.push(TimeValue{ value: totals.value, date: current_date});

    while current_date < end {
        // Update list of transactions with new strategic transactions for the current day
        let mut new_transactions = strategy.apply(&position, current_date).await.unwrap();
        transactions.append(&mut new_transactions);

        // roll position forward to next day
        let next_date = min(end, strategy.next_day(current_date));

        // Calculate new position including new transactions
        debug!(
            "CalcStrategy: cash position before applying new transactions: {}",
            position.cash.position
        );
        calc_delta_position(
            &mut position,
            &transactions,
            Some(current_date),
            Some(next_date),
            market.clone(),
        )
        .await
        .unwrap();
        debug!(
            "CalcStrategy: cash position after applying new transactions: {}",
            position.cash.position
        );

        current_date = next_date;
        let current_time = naive_date_to_date_time(&current_date, 20, None).unwrap();
        position.add_quote(current_time, market.clone()).await;
        let totals = position.calc_totals();
        total_return.push(TimeValue {
            value: totals.value,
            time: current_time,
        });
    }
    total_return
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

    pretty_env_logger::init();

    println!("Calculate total return of single investment of 10'000 USD in Broadcom (AVGO) five years before today");
    let today = Local::now().naive_local().date();
    let five_years_before = "-5Y".parse::<TimePeriod>().unwrap();
    let start = five_years_before.add_to(today, None);
    println!("The simulation will run from {} until {}.", start, today);

    db.clean().await.unwrap();
    let db: Arc<dyn QuoteHandler + Send + Sync> = Arc::new(db);

    // Define the asset
    let asset = Asset::Stock(Stock::new(
        None,
        "Broadcom Inc.".to_string(),
        None,
        None,
        None,
    ));
    let asset_id = db.insert_asset(&asset).await.unwrap();

    println!("Get price history and dividends for AVGO");
    let market = Market::new_with_date_range(db.clone(), start, today).await;
    let yahoo = MarketDataSource::Yahoo;
    let quote_provider = yahoo.get_provider(String::new()).unwrap();
    market.add_provider(yahoo.to_string(), quote_provider.clone());
    let usd = market.get_currency_from_str("USD").await.unwrap();
    let ticker = Ticker {
        id: None,
        asset: asset_id,
        name: "AVGO".to_string(),
        currency: usd,
        source: yahoo.to_string(),
        priority: 1,
        factor: 1.0,
        tz: None,
        cal: None,
    };
    let ticker_id = db.insert_ticker(&ticker).await.unwrap();
    let price_offset_period = "7D".parse::<TimePeriod>().unwrap();
    let history_start_time = start - price_offset_period;
    let start_time = date_to_date_time(&start, 0, None).unwrap();
    let end_time = date_to_date_time(&today, 20, None).unwrap();
    market
        .update_quote_history(ticker_id, history_start_time, end_time)
        .await
        .unwrap();

    let dividends = quote_provider
        .fetch_dividend_history(&ticker, start_time, end_time)
        .await
        .unwrap();
    println!("Found {} dividends", dividends.len());

    let mut transactions = Vec::new();

    // put some cash into the account
    println!("Setup initial cash transaction");
    let cash_flow = CashFlow::new(10_000.0, usd, start);
    transactions.push(Transaction {
        id: None,
        transaction_type: TransactionType::Cash,
        cash_flow,
        note: Some("start capital".to_string()),
    });
    let asset_price = market
        .get_asset_price(asset_id, usd, start_time)
        .await
        .unwrap();

    println!("Buy transaction for initial stock position");
    transactions.push(Transaction {
        id: None,
        transaction_type: TransactionType::Asset {
            asset_id,
            position: cash_flow.amount.amount / asset_price,
        },
        cash_flow: CashFlow {
            amount: -cash_flow.amount,
            date: start,
        },
        note: Some("Initial asset buy transaction".to_string()),
    });

    let mut all_time_series = Vec::new();

    let reinvest_strategy_no_tax_no_fee = ReInvestInSingleStock::new(
        asset_id,
        ticker_id,
        market.clone(),
        dividends.clone(),
        Default::default(),
    );
    let reinvest_returns_no_tax_no_fee = calc_strategy(
        usd,
        &transactions,
        &reinvest_strategy_no_tax_no_fee,
        start,
        today,
        market.clone(),
    )
    .await;
    all_time_series.push(TimeSeries {
        series: reinvest_returns_no_tax_no_fee,
        title: "AVGO re-invest return, no fees and tax".to_string(),
    });

    let costs = StockTransactionCosts {
        fee: StockTransactionFee::new(5.0, Some(30.0), 0.0025),
        tax_rate: 0.25 * 1.07,
    };
    let reinvest_strategy = ReInvestInSingleStock::new(
        asset_id,
        ticker_id,
        market.clone(),
        dividends.clone(),
        costs.clone(),
    );
    let reinvest_returns = calc_strategy(
        usd,
        &transactions,
        &reinvest_strategy,
        start,
        today,
        market.clone(),
    )
    .await;
    all_time_series.push(TimeSeries {
        series: reinvest_returns,
        title: "AVGO re-invest return".to_string(),
    });

    let static_invest_strategy_no_tax =
        StaticInSingleStock::new(asset_id, dividends.clone(), Default::default());
    let static_invest_returns_no_tax = calc_strategy(
        usd,
        &transactions,
        &static_invest_strategy_no_tax,
        start,
        today,
        market.clone(),
    )
    .await;
    all_time_series.push(TimeSeries {
        series: static_invest_returns_no_tax,
        title: "AVGO static return, no tax".to_string(),
    });

    let static_invest_strategy = StaticInSingleStock::new(asset_id, dividends, costs);
    let static_invest_returns = calc_strategy(
        usd,
        &transactions,
        &static_invest_strategy,
        start,
        today,
        market.clone(),
    )
    .await;
    all_time_series.push(TimeSeries {
        series: static_invest_returns,
        title: "AVGO static return".to_string(),
    });

    let no_dividends_strategy = StaticInSingleStock::new(asset_id, Vec::new(), Default::default());
    let no_dividends_returns = calc_strategy(
        usd,
        &transactions,
        &no_dividends_strategy,
        start,
        today,
        market.clone(),
    )
    .await;
    all_time_series.push(TimeSeries {
        series: no_dividends_returns,
        title: "AVGO without dividends".to_string(),
    });

    // plot the graph
    make_plot("strategies.png", "Strategies Performance", &all_time_series).unwrap();
}

fn make_plot(
    file_name: &str,
    title: &str,
    all_time_series: &[TimeSeries],
) -> Result<(), Box<dyn Error>> {
    //let root = SVGBackend::new(file_name, (2048, 1024)).into_drawing_area();
    let root = BitMapBackend::new(file_name, (2048, 1024)).into_drawing_area();

    root.fill(&WHITE)?;

    if all_time_series.len() == 0 {
        return Err(Box::new(TimeSeriesError::IsEmpty));
    }
    let (mut min_date, mut max_date, mut min_val, mut max_val) = all_time_series[0].min_max()?;

    // Calculate max ranges over all time sereies
    for ts in &all_time_series[1..] {
        let (min_date_tmp, max_date_tmp, min_val_tmp, max_val_tmp) = ts.min_max()?;
        if min_date_tmp < min_date {
            min_date = min_date_tmp;
        }
        if max_date_tmp > max_date {
            max_date = max_date_tmp;
        }
        if min_val_tmp < min_val {
            min_val = min_val_tmp;
        }
        if max_val_tmp > max_val {
            max_val = max_val_tmp;
        }
    }

    let y_range = min_val..max_val;
    let min_time = make_time(min_date.year(), min_date.month(), 1, 0, 0, 0).unwrap();
    let max_year = max_date.year();
    let max_month = max_date.month();
    let max_time = make_time(
        max_year,
        max_month,
        last_day_of_month(max_year, max_month),
        23,
        59,
        59,
    )
    .unwrap();
    let x_range = (min_time..max_time).monthly();

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption(title, ("sans-serif", 40))
        .set_label_area_size(LabelAreaPosition::Left, 80)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .build_cartesian_2d(x_range, y_range)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(30)
        .y_desc("Total position value (€)")
        .x_desc("Date")
        .label_style(("sans-serif", 16))
        .axis_desc_style(("sans-serif", 20))
        .draw()?;

    static COLORS: [&'static RGBColor; 5] = [&BLUE, &GREEN, &RED, &CYAN, &MAGENTA];
    let mut color_index: usize = 0;
    for ts in all_time_series {
        chart
            .draw_series(LineSeries::new(
                ts.series.iter().map(|v| (v.time, v.value)),
                COLORS[color_index],
            ))?
            .label(&ts.title)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], COLORS[color_index]));
        color_index = (color_index + 1) % COLORS.len();
    }

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .position(SeriesLabelPosition::UpperLeft)
        .label_font(("sans-serif", 20))
        .draw()?;

    Ok(())
}
