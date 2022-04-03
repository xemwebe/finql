use chrono::naive::NaiveDate;
use async_trait::async_trait;
use log::{debug, trace};
use thiserror::Error;

use crate::datatypes::{
    Transaction, 
    TransactionType, 
    CashFlow,
    date_time_helper::naive_date_to_date_time
};
use crate::{
    portfolio::PortfolioPosition,
    Market,
    time_period::TimePeriod, 
};


#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("Failed to retreive data from database")]
    RectreivingDataFailed(#[from] crate::datatypes::DataError),
    #[error("date/time conversion error")]
    DateTimeError(#[from] crate::datatypes::date_time_helper::DateTimeError),
}

#[derive(Default, Debug, Clone)]
pub struct StockTransactionFee {
    min_fee: f64,
    max_fee: Option<f64>,
    proportional_fee: f64,
}

impl StockTransactionFee {
    pub fn new(min_fee: f64, max_fee: Option<f64>, proportional_fee: f64) -> Self {
        StockTransactionFee{min_fee, max_fee, proportional_fee}
    }

    pub fn calc_fee(&self, total_price: f64) -> f64 {
        let fee = (total_price*self.proportional_fee).max(self.min_fee);
        if let Some(max_fee) = self.max_fee {
            fee.min(max_fee)
        } else {
            fee
        }
    }
}

#[derive(Default,Debug,Clone)]
pub struct StockTransactionCosts {
    pub fee: StockTransactionFee,
    pub tax_rate: f64,
}

#[async_trait]
pub trait Strategy {
    async fn apply(&self, position: &PortfolioPosition, date: NaiveDate) -> Result<Vec<Transaction>,  StrategyError>;
    fn next_day(&self, date: NaiveDate) -> NaiveDate;
}


fn cash_flow_idx(date: NaiveDate, cash_flows: &[CashFlow]) -> Option<usize> {
    for (i, cf) in cash_flows.iter().enumerate() {
        if cf.date == date {
            return Some(i);
        }
    }
    None
}

pub struct StaticInSingleStock {
    asset_id: usize,
    dividends: Vec<CashFlow>,
    costs: StockTransactionCosts,
}

impl StaticInSingleStock {
    pub fn new(asset_id: usize, dividends: Vec<CashFlow>, costs: StockTransactionCosts) -> StaticInSingleStock {
        StaticInSingleStock{
            asset_id,
            dividends,
            costs,
        }
    }
}

#[async_trait]
impl Strategy for StaticInSingleStock {
    async fn apply(&self, position: &PortfolioPosition, date: NaiveDate) -> Result<Vec<Transaction>, StrategyError> {
        let mut transactions = Vec::new();
        if let Some(idx) = cash_flow_idx(date, &self.dividends) {
            let mut dividend = self.dividends[idx];
            dividend.amount.amount *= position.assets[&self.asset_id].position;
            let mut tax = dividend;
            tax.amount.amount = -self.costs.tax_rate * dividend.amount.amount;
            let dividend_transaction = Transaction {
                id: None,
                transaction_type: TransactionType::Dividend {
                    asset_id: self.asset_id,
                },
                cash_flow: dividend,
                note: None,
            };
            trace!("ReinvestInSingleStock: added transaction {:?}", dividend_transaction);
            transactions.push(dividend_transaction);

            if tax.amount.amount != 0.0 {
            let tax_transaction = Transaction {
                id: None,
                transaction_type: TransactionType::Tax {
                    transaction_ref: None,
                },
                cash_flow: tax,
                note: None,
            };
            trace!("ReinvestInSingleStock: added transaction {:?}", tax_transaction);
            transactions.push(tax_transaction);

        }
            debug!("StaticInSingleStock: added dividend without amount {} and tax {} at date {}.", dividend.amount.amount, -tax.amount.amount, date);
        } 
        Ok(transactions)
    }

    fn next_day(&self, date: NaiveDate) -> NaiveDate {
        let one_day = "1D".parse::<TimePeriod>().unwrap();
        one_day.add_to(date, None)
    }
}


pub struct ReInvestInSingleStock {
    asset_id: usize,
    ticker_id: usize,
    market: Market,
    dividends: Vec<CashFlow>,
    costs: StockTransactionCosts,
}

impl ReInvestInSingleStock {
    pub fn new(asset_id: usize, ticker_id: usize, market: Market, dividends: Vec<CashFlow>, costs: StockTransactionCosts) -> ReInvestInSingleStock {
        ReInvestInSingleStock {
            asset_id,
            ticker_id,
            market,
            dividends,
            costs,
        }
    }
}

#[async_trait]
impl Strategy for ReInvestInSingleStock {
    async fn apply(&self, position: &PortfolioPosition, date: NaiveDate) -> Result<Vec<Transaction>, StrategyError> {
        let mut transactions = Vec::new();
        if let Some(idx) = cash_flow_idx(date, &self.dividends) {
            let mut dividend = self.dividends[idx];
            dividend.amount.amount *= position.assets[&self.asset_id].position;
            let mut tax = dividend;
            tax.amount.amount = -self.costs.tax_rate * dividend.amount.amount;
            let available_cash = dividend.amount.amount + tax.amount.amount + position.cash.position;
            let dividend_transaction = Transaction {
                id: None,
                transaction_type: TransactionType::Dividend {
                    asset_id: self.asset_id,
                },
                cash_flow: dividend,
                note: None,
            };
            trace!("ReinvestInSingleStock: added transaction {:?}", dividend_transaction);
            transactions.push(dividend_transaction);
            if tax.amount.amount != 0.0 {
                let tax_transaction = Transaction {
                    id: None,
                    transaction_type: TransactionType::Tax {
                        transaction_ref: None,
                    },
                    cash_flow: tax,
                    note: None,
                };
                trace!("ReinvestInSingleStock: added transaction {:?}", tax_transaction);
                transactions.push(tax_transaction);
            }
            // reinvest in stock
            let (asset_quote, _quote_currency) = self.market.db().get_last_quote_before_by_id(self.ticker_id, naive_date_to_date_time(&date, 20, None)?).await?;
            let (additional_position, fee) = self.calc_position_and_fee(available_cash, asset_quote.price);
            if additional_position>0.0 {
                let buy_transaction = Transaction{
                    id: None,
                    transaction_type: TransactionType::Asset{
                        asset_id: self.asset_id,
                        position: additional_position,
                    },
                    cash_flow: CashFlow::new(-additional_position*asset_quote.price, position.cash.currency, date),
                    note: None,
                };
                trace!("ReinvestInSingleStock: added transaction {:?}", buy_transaction);
                transactions.push(buy_transaction);
                if fee!=0.0 {
                    let fee_transaction = Transaction{
                        id: None,
                        transaction_type: TransactionType::Fee{
                            transaction_ref: None,
                        },
                        cash_flow: CashFlow::new(-fee, position.cash.currency, date),
                        note: None,
                    };
                    trace!("ReinvestInSingleStock: added transaction {:?}", fee_transaction);
                    transactions.push(fee_transaction);
                }
            }
            debug!("ReInvestInSingleStock: added dividend with amount {} and tax {} and buying {} shares with fee {} from available cash {} with price {} at date {}.", 
                dividend.amount.amount, -tax.amount.amount, additional_position, fee, available_cash, asset_quote.price, date);
        } 

        Ok(transactions)
    }

    fn next_day(&self, date: NaiveDate) -> NaiveDate {
        let one_day = "1D".parse::<TimePeriod>().unwrap();
        one_day.add_to(date, None)
    }
}

impl ReInvestInSingleStock {

    fn calc_position_and_fee(&self, cash: f64, price: f64) -> (f64, f64) {
        let mut max_position = (cash/price).floor();
        let mut fee = self.costs.fee.calc_fee(max_position*price);
        while max_position>0.0 && (max_position*price-fee)<0.0 {
            max_position -= 1.0;
            fee = self.costs.fee.calc_fee(max_position*price);
        }
        (max_position, fee)
    }
}
