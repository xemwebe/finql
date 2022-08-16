use futures::future::join_all;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::vec::Vec;
use thiserror::Error;

use chrono::offset::TimeZone;
use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::datatypes::{
    currency::CurrencyConverter,
    date_time_helper::{naive_date_to_date_time, DateTimeError},
    Asset, AssetHandler, Currency, CurrencyError, DataError, Transaction,
    TransactionType,
};

use crate::period_date::PeriodDateError;
use crate::Market;

/// Errors related to position calculation
#[derive(Error, Debug)]
pub enum PositionError {
    #[error("Failed to fetch position data")]
    PositionDataError(#[from] DataError),
    #[error("Invalid start or end date")]
    DateError(#[from] PeriodDateError),
    #[error("Invalid date or time")]
    DateTimeError(#[from] DateTimeError),
    #[error("Failed to convert currency")]
    CurrencyError(#[from] CurrencyError),
}

/// Calculate the total position as of a given date by applying a specified set of filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub asset_id: Option<i32>,
    pub name: String,
    pub position: f64,
    pub purchase_value: f64,
    // realized p&l from buying/selling assets
    pub trading_pnl: f64,
    pub interest: f64,
    pub dividend: f64,
    pub fees: f64,
    pub tax: f64,
    pub currency: Currency,
    pub last_quote: Option<f64>,
    pub last_quote_time: Option<DateTime<Local>>,
}

/// Calculate the total position as of a given date by applying a specified set of filters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PositionTotals {
    pub value: f64,
    trading_pnl: f64,
    unrealized_pnl: f64,
    dividend: f64,
    interest: f64,
    tax: f64,
    fees: f64,
}

impl Position {
    pub fn new(asset_id: Option<i32>, currency: Currency) -> Position {
        Position {
            asset_id,
            name: String::new(),
            position: 0.0,
            purchase_value: 0.0,
            trading_pnl: 0.0,
            currency,
            interest: 0.0,
            dividend: 0.0,
            fees: 0.0,
            tax: 0.0,
            last_quote: None,
            last_quote_time: None,
        }
    }

    fn quote_from_purchase(&self) -> Option<f64> {
        if self.position == 0.0 {
            None
        } else {
            Some(-self.purchase_value / self.position)
        }
    }

    /// Add quote information to position
    /// If no quote is available (or no conversion to position currency), calculate
    /// from purchase value.
    pub async fn add_quote(&mut self, time: DateTime<Local>, market: &Market) {
        if let Some(asset_id) = self.asset_id {
            if let Ok(price) = market.get_asset_price(asset_id, self.currency, time).await {
                self.last_quote = Some(price);
                self.last_quote_time = Some(time);
            } else {
                // No price found
                self.last_quote = self.quote_from_purchase();
                self.last_quote_time = None;
            }
        } else {
            // No asset ID, must be some technical account, set price to 1.0
            self.last_quote = Some(1.0);
            self.last_quote_time = Some(Local::now());
        };
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioPosition {
    pub cash: Position,
    pub assets: BTreeMap<i32, Position>,
}

impl PortfolioPosition {
    pub fn new(base_currency: Currency) -> PortfolioPosition {
        PortfolioPosition {
            cash: Position::new(None, base_currency),
            assets: BTreeMap::new(),
        }
    }

    pub async fn get_asset_names(
        &mut self,
        db: Arc<dyn AssetHandler + Send + Sync>,
    ) -> Result<(), DataError> {
        for (id, mut pos) in &mut self.assets {
            let asset = db.get_asset_by_id(*id).await?;
            pos.name = match asset {
                Asset::Currency(c) => c.iso_code.to_string(),
                Asset::Stock(s) => s.name.clone(),
            };
        }
        Ok(())
    }

    pub async fn add_quote(&mut self, time: DateTime<Local>, market: &Market) {
        let mut get_quote_futures = Vec::new();
        for pos in self.assets.values_mut() {
            get_quote_futures.push(pos.add_quote(time, market));
        }
        let _ = join_all(get_quote_futures).await;
    }

    pub fn calc_totals(&mut self) -> PositionTotals {
        let mut totals = PositionTotals {
            value: self.cash.position,
            trading_pnl: self.cash.trading_pnl,
            unrealized_pnl: 0.0,
            dividend: self.cash.dividend,
            interest: self.cash.interest,
            tax: self.cash.tax,
            fees: self.cash.fees,
        };
        for pos in self.assets.values() {
            let pos_value = if let Some(quote) = pos.last_quote {
                pos.position * quote
            } else {
                -pos.purchase_value
            };
            totals.value += pos_value;
            totals.trading_pnl += pos.trading_pnl;
            totals.unrealized_pnl += pos_value + pos.purchase_value;
            totals.dividend += pos.dividend;
            totals.interest += pos.interest;
            totals.tax += pos.tax;
            totals.fees += pos.fees;
        }
        totals
    }

    /// Reset all pnl relevant figures, i.e. set purchase value to position * price and
    /// realized p&l, dividends, interest, tax, fee to 0 and eliminate 0 positions
    fn reset_pnl(&mut self) {
        self.remove_zero_positions();
        self.cash.trading_pnl = 0.0;
        self.cash.dividend = 0.0;
        self.cash.interest = 0.0;
        self.cash.fees = 0.0;
        self.cash.tax = 0.0;
        for mut pos in self.assets.iter_mut() {
            pos.1.trading_pnl = 0.0;
            pos.1.dividend = 0.0;
            pos.1.interest = 0.0;
            pos.1.fees = 0.0;
            pos.1.tax = 0.0;
            pos.1.purchase_value = -pos.1.position * pos.1.last_quote.unwrap_or(0.0);
        }
    }

    fn remove_zero_positions(&mut self) {
        let mut zero_positions = Vec::new();
        for pos in self.assets.iter() {
            if pos.1.position == 0.0 {
                zero_positions.push(*pos.0);
            }
        }
        for key in zero_positions {
            self.assets.remove(&key);
        }
    }
}

/// Search for transaction referred to by transaction_ref and return associated asset_id
fn get_asset_id(transactions: &[Transaction], trans_ref: Option<i32>) -> Option<i32> {
    trans_ref?;
    for trans in transactions {
        if trans.id == trans_ref {
            return match trans.transaction_type {
                TransactionType::Asset {
                    asset_id,
                    position: _,
                } => Some(asset_id),
                TransactionType::Dividend { asset_id } => Some(asset_id),
                TransactionType::Interest { asset_id } => Some(asset_id),
                _ => None,
            };
        }
    }
    None
}

/// Calculate the total position since inception caused by a given set of transactions.
pub async fn calc_position(
    base_currency: Currency,
    transactions: &[Transaction],
    date: Option<NaiveDate>,
    market: Arc<Market>,
) -> Result<PortfolioPosition, PositionError> {
    let mut positions = PortfolioPosition::new(base_currency);
    calc_delta_position(&mut positions, transactions, None, date, market).await?;
    Ok(positions)
}

/// Given a PortfolioPosition, calculate changes to position by a given set of transactions.
pub async fn calc_delta_position(
    positions: &mut PortfolioPosition,
    transactions: &[Transaction],
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    market: Arc<Market>,
) -> Result<(), PositionError> {
    let base_currency = positions.cash.currency;
    for trans in transactions {
        if start.is_some() && trans.cash_flow.date < start.unwrap() {
            continue;
        }
        if end.is_some() && trans.cash_flow.date >= end.unwrap() {
            continue;
        }
        let curr_factor = if trans.cash_flow.amount.currency != base_currency {
            market
                .fx_rate(
                    trans.cash_flow.amount.currency,
                    base_currency,
                    naive_date_to_date_time(&trans.cash_flow.date, 20, None)?,
                )
                .await?
        } else {
            1.0
        };
        // adjust cash balance
        positions.cash.position += trans.cash_flow.amount.amount * curr_factor;

        match trans.transaction_type {
            TransactionType::Cash => {
                // Do nothing, cash position has already been updated
            }
            TransactionType::Asset { asset_id, position } => {
                match positions.assets.get_mut(&asset_id) {
                    None => {
                        let mut new_pos = Position::new(Some(asset_id), base_currency);
                        new_pos.position = position;
                        new_pos.purchase_value = trans.cash_flow.amount.amount;
                        positions.assets.insert(asset_id, new_pos);
                    }
                    Some(pos) => {
                        let amount = trans.cash_flow.amount.amount;
                        if pos.position * position >= 0.0 {
                            // Increase position
                            pos.position += position;
                            pos.purchase_value += amount;
                        } else {
                            // Reduce position, calculate realized p&l part
                            let eff_price = -pos.purchase_value / pos.position;
                            let sell_price = -amount / position;
                            let pnl = -position * (sell_price - eff_price);
                            pos.trading_pnl += pnl;
                            pos.position += position;
                            pos.purchase_value += amount - pnl;
                        }
                    }
                };
            }
            TransactionType::Interest { asset_id } => {
                match positions.assets.get_mut(&asset_id) {
                    None => {
                        let mut new_pos = Position::new(Some(asset_id), base_currency);
                        new_pos.interest = trans.cash_flow.amount.amount;
                        positions.assets.insert(asset_id, new_pos);
                    }
                    Some(pos) => {
                        pos.interest += trans.cash_flow.amount.amount;
                    }
                };
            }
            TransactionType::Dividend { asset_id } => {
                match positions.assets.get_mut(&asset_id) {
                    None => {
                        let mut new_pos = Position::new(Some(asset_id), base_currency);
                        new_pos.dividend = trans.cash_flow.amount.amount;
                        positions.assets.insert(asset_id, new_pos);
                    }
                    Some(pos) => {
                        pos.dividend += trans.cash_flow.amount.amount;
                    }
                };
            }
            TransactionType::Fee { transaction_ref } => {
                let asset_id = get_asset_id(transactions, transaction_ref);
                if let Some(asset_id) = asset_id {
                    match positions.assets.get_mut(&asset_id) {
                        None => {
                            let mut new_pos = Position::new(Some(asset_id), base_currency);
                            new_pos.fees = trans.cash_flow.amount.amount;
                            positions.assets.insert(asset_id, new_pos);
                        }
                        Some(pos) => {
                            pos.fees += trans.cash_flow.amount.amount;
                        }
                    };
                } else {
                    positions.cash.fees += trans.cash_flow.amount.amount;
                }
            }
            TransactionType::Tax { transaction_ref } => {
                let asset_id = get_asset_id(transactions, transaction_ref);
                if let Some(asset_id) = asset_id {
                    match positions.assets.get_mut(&asset_id) {
                        None => {
                            let mut new_pos = Position::new(Some(asset_id), base_currency);
                            new_pos.tax = trans.cash_flow.amount.amount;
                            positions.assets.insert(asset_id, new_pos);
                        }
                        Some(pos) => {
                            pos.tax += trans.cash_flow.amount.amount;
                        }
                    };
                } else {
                    positions.cash.tax += trans.cash_flow.amount.amount;
                }
            }
        }
    }
    Ok(())
}

/// Calculate position and P&L since for list of transactions.
/// All transaction with cash flow dates before the given date are taken into account and valued
/// using the latest available quote before midnight of that date.
pub async fn calculate_position_and_pnl(
    currency: Currency,
    transactions: &[Transaction],
    date: Option<NaiveDate>,
    market: Arc<Market>,
) -> Result<(PortfolioPosition, PositionTotals), PositionError> {
    let mut position = calc_position(currency, transactions, date, market.clone()).await?;
    position
        .get_asset_names(market.db().into_arc_dispatch())
        .await?;
    let date_time: DateTime<Local> = if let Some(date) = date {
        Local.from_local_datetime(&date.and_hms(0, 0, 0)).unwrap()
    } else {
        Local::now()
    };
    position.add_quote(date_time, &market).await;
    let totals = position.calc_totals();
    Ok((position, totals))
}

/// Calculate position and P&L changes for a given range of dates.
/// The date range is inclusive, i.e. all transactions with cash flow dates on or after `start`
/// and on or before `end` a taken into account. The initial positions at `start` are valued
/// with the latest quotes before that date, the final position is valued with the latest
/// quotes before the date after `end`. With this method, P&L is additive, i.e. adding the
/// P&L figures of directly succeeding date periods should sum up to the P&L of the joined period.
pub async fn calculate_position_for_period(
    currency: Currency,
    transactions: &[Transaction],
    start: NaiveDate,
    end: NaiveDate,
    market: Arc<Market>,
) -> Result<(PortfolioPosition, PositionTotals), PositionError> {
    let (mut position, _) =
        calculate_position_and_pnl(currency, transactions, Some(start), market.clone()).await?;
    position.reset_pnl();
    calc_delta_position(&mut position, transactions, Some(start), Some(end), market.clone()).await?;
    position
        .get_asset_names(market.db().into_arc_dispatch())
        .await?;
    let end_date_time: DateTime<Local> = Local
        .from_local_datetime(&end.succ().and_hms(0, 0, 0))
        .unwrap();
    position.add_quote(end_date_time, &market).await;
    let totals = position.calc_totals();
    Ok((position, totals))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use chrono::NaiveDate;

    use crate::assert_fuzzy_eq;
    use crate::datatypes::{
        date_time_helper::make_time, Asset, AssetHandler, CashAmount, CashFlow, Currency,
        CurrencyISOCode, Quote, Stock, Ticker,
    };
    use crate::postgres::PostgresDB;

    #[test]
    fn test_portfolio_position() {
        let tol = 1e-4;
        let eur = Currency::from_str("EUR").unwrap();

        let mut transactions = Vec::new();
        let positions = calc_position(eur, &transactions, None).unwrap();
        assert_fuzzy_eq!(positions.cash.position, 0.0, tol);

        transactions.push(Transaction {
            id: Some(1),
            transaction_type: TransactionType::Cash,
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: 10000.0,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 1, 1),
            },
            note: None,
        });
        let positions = calc_position(eur, &transactions, None).unwrap();
        assert_fuzzy_eq!(positions.cash.position, 10000.0, tol);
        assert_eq!(positions.assets.len(), 0);

        transactions.push(Transaction {
            id: Some(2),
            transaction_type: TransactionType::Asset {
                asset_id: 1,
                position: 100.0,
            },
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: -104.0,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 1, 2),
            },
            note: None,
        });
        transactions.push(Transaction {
            id: Some(3),
            transaction_type: TransactionType::Fee {
                transaction_ref: Some(2),
            },
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: -5.0,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 1, 2),
            },
            note: None,
        });
        let positions = calc_position(eur, &transactions, None).unwrap();
        assert_fuzzy_eq!(positions.cash.position, 10000.0 - 104.0 - 5.0, tol);
        assert_eq!(positions.assets.len(), 1);
        let asset_pos_1 = positions.assets.get(&1).unwrap();
        assert_fuzzy_eq!(asset_pos_1.purchase_value, -104.0, tol);
        assert_fuzzy_eq!(asset_pos_1.position, 100.0, tol);
        assert_fuzzy_eq!(asset_pos_1.fees, -5.0, tol);
        assert_eq!(asset_pos_1.currency, eur);

        transactions.push(Transaction {
            id: Some(4),
            transaction_type: TransactionType::Asset {
                asset_id: 1,
                position: -50.0,
            },
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: 60.0,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 1, 31),
            },
            note: None,
        });
        transactions.push(Transaction {
            id: Some(5),
            transaction_type: TransactionType::Fee {
                transaction_ref: Some(4),
            },
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: -3.0,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 1, 31),
            },
            note: None,
        });
        transactions.push(Transaction {
            id: Some(6),
            transaction_type: TransactionType::Tax {
                transaction_ref: Some(4),
            },
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: -2.0,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 1, 31),
            },
            note: None,
        });
        let positions = calc_position(eur, &transactions, None).unwrap();
        assert_fuzzy_eq!(
            positions.cash.position,
            10000.0 - 104.0 - 5.0 + 60.0 - 2.0 - 3.0,
            tol
        );
        assert_eq!(positions.assets.len(), 1);
        let asset_pos_1 = positions.assets.get(&1).unwrap();
        assert_fuzzy_eq!(asset_pos_1.purchase_value, -52.0, tol);
        assert_fuzzy_eq!(asset_pos_1.position, 50.0, tol);
        assert_fuzzy_eq!(asset_pos_1.fees, -8.0, tol);
        assert_fuzzy_eq!(asset_pos_1.trading_pnl, 8.0, tol);
        assert_eq!(asset_pos_1.currency, eur);

        transactions.push(Transaction {
            id: Some(7),
            transaction_type: TransactionType::Asset {
                asset_id: 1,
                position: 150.0,
            },
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: -140.0,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 2, 15),
            },
            note: None,
        });
        transactions.push(Transaction {
            id: Some(8),
            transaction_type: TransactionType::Fee {
                transaction_ref: None,
            },
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: -7.0,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 2, 25),
            },
            note: None,
        });
        transactions.push(Transaction {
            id: Some(9),
            transaction_type: TransactionType::Tax {
                transaction_ref: None,
            },
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: -4.5,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 2, 26),
            },
            note: None,
        });
        transactions.push(Transaction {
            id: Some(10),
            transaction_type: TransactionType::Dividend { asset_id: 2 },
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: 13.0,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 2, 27),
            },
            note: None,
        });
        transactions.push(Transaction {
            id: Some(11),
            transaction_type: TransactionType::Interest { asset_id: 3 },
            cash_flow: CashFlow {
                amount: CashAmount {
                    amount: 6.6,
                    currency: eur,
                },
                date: NaiveDate::from_ymd(2020, 2, 28),
            },
            note: None,
        });
        let positions = calc_position(eur, &transactions, None).unwrap();
        assert_fuzzy_eq!(
            positions.cash.position,
            10000.0 - 104.0 - 5.0 + 60.0 - 2.0 - 3.0 - 140.0 - 7.0 - 4.5 + 13.0 + 6.6,
            tol
        );
        assert_eq!(positions.assets.len(), 3);
        let asset_pos_1 = positions.assets.get(&1).unwrap();
        assert_fuzzy_eq!(asset_pos_1.purchase_value, -192.0, tol);
        assert_fuzzy_eq!(asset_pos_1.position, 200.0, tol);
        assert_fuzzy_eq!(asset_pos_1.fees, -8.0, tol);
        assert_fuzzy_eq!(asset_pos_1.trading_pnl, 8.0, tol);

        // fees and taxes not associated to any transaction
        assert_fuzzy_eq!(positions.cash.fees, -7.0, tol);
        assert_fuzzy_eq!(positions.cash.tax, -4.5, tol);

        // standalone dividends/interest
        let asset_pos_2 = positions.assets.get(&2).unwrap();
        assert_fuzzy_eq!(asset_pos_2.dividend, 13.0, tol);
        let asset_pos_3 = positions.assets.get(&3).unwrap();
        assert_fuzzy_eq!(asset_pos_3.interest, 6.6, tol);
    }

    #[tokio::test]
    async fn test_add_quote_to_position() {
        use crate::datatypes::DataItem;

        let tol = 1e-4;
        // Setup database connection
        let db_url = std::env::var("FINQL_TEST_DATABASE_URL");
        assert!(
            db_url.is_ok(),
            "environment variable $FINQL_TEST_DATABASE_URL is not set"
        );
        let db = PostgresDB::new(&db_url.unwrap()).await.unwrap();
        db.clean().await.unwrap();

        // first add some assets and currencies
        let eur_stock_id = db
            .insert_asset(&Asset::Stock(Stock::new(
                None,
                "EUR Stock".to_string(),
                Some("EURS".to_string()),
                None,
                None,
            )))
            .await
            .unwrap();
        let us_stock_id = db
            .insert_asset(&Asset::Stock(Stock::new(
                None,
                "USD Stock".to_string(),
                Some("USDS".to_string()),
                None,
                None,
            )))
            .await
            .unwrap();
        let mut eur = Currency::new(None, CurrencyISOCode::new("EUR").unwrap(), Some(2));
        let eur_id = db.insert_asset(&Asset::Currency(eur)).await.unwrap();
        eur.set_id(eur_id).unwrap();

        let mut usd = Currency::new(None, CurrencyISOCode::new("USD").unwrap(), Some(2));
        let usd_id = db.insert_asset(&Asset::Currency(usd)).await.unwrap();
        usd.set_id(usd_id).unwrap();

        // add ticker
        let eur_ticker_id = db
            .insert_ticker(&Ticker {
                id: None,
                name: "EUR_STOCK.DE".to_string(),
                asset: eur_stock_id,
                priority: 10,
                currency: eur,
                source: "manual".to_string(),
                factor: 1.0,
                tz: None,
                cal: None,
            })
            .await
            .unwrap();
        let us_ticker_id = db
            .insert_ticker(&Ticker {
                id: None,
                name: "US_STOCK.DE".to_string(),
                asset: us_stock_id,
                priority: 10,
                currency: usd,
                source: "manual".to_string(),
                factor: 1.0,
                tz: None,
                cal: None,
            })
            .await
            .unwrap();
        // add quotes
        let time = make_time(2019, 12, 30, 10, 0, 0).unwrap();
        let _ = db
            .insert_quote(&Quote {
                id: None,
                ticker: eur_ticker_id,
                price: 12.34,
                time,
                volume: None,
            })
            .await
            .unwrap();
        let _ = db
            .insert_quote(&Quote {
                id: None,
                ticker: us_ticker_id,
                price: 43.21,
                time,
                volume: None,
            })
            .await
            .unwrap();
        let mut eur_position = Position::new(Some(eur_stock_id), eur);
        eur_position.name = "EUR Stock".to_string();
        eur_position.position = 1000.0;

        let mut usd_position = Position::new(Some(us_stock_id), eur);
        usd_position.name = "US Stock".to_string();
        usd_position.position = 1000.0;

        let qh: Arc<dyn QuoteHandler + Sync + Send> = Arc::new(db);
        crate::fx_rates::insert_fx_quote(2.0, usd, eur, time, qh.clone())
            .await
            .unwrap();
        let time = make_time(2019, 12, 30, 12, 0, 0).unwrap();
        let market = Market::new(qh.clone()).await;

        eur_position.add_quote(time, &market).await;
        assert_fuzzy_eq!(eur_position.last_quote.unwrap(), 12.34, tol);
        assert_eq!(
            eur_position
                .last_quote_time
                .unwrap()
                .format("%F %H:%M:%S")
                .to_string(),
            "2019-12-30 12:00:00"
        );

        usd_position.add_quote(time, &market).await;
        assert_fuzzy_eq!(usd_position.last_quote.unwrap(), 86.42, tol);
        assert_eq!(
            usd_position
                .last_quote_time
                .unwrap()
                .format("%F %H:%M:%S")
                .to_string(),
            "2019-12-30 10:00:00"
        );
    }
}
