use chrono::naive::NaiveDate;
use async_trait::async_trait;

use finql_data::{
    DataError,
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

#[async_trait]
pub trait Strategy {
    async fn apply(&self, position: &PortfolioPosition, date: NaiveDate) -> Result<Vec<Transaction>,  DataError>;
    fn next_day(&self, date: NaiveDate) -> NaiveDate;
}


fn cash_flow_idx(date: NaiveDate, cash_flows: &[CashFlow]) -> Option<usize> {
    for i in 0..cash_flows.len() {
        if cash_flows[i].date == date {
            return Some(i);
        }
    }
    None
}

pub struct StaticInSingleStock {
    asset_id: usize,
    dividends: Vec<CashFlow>
}

impl StaticInSingleStock {
    pub fn new(asset_id: usize, dividends: Vec<CashFlow>) -> StaticInSingleStock {
        StaticInSingleStock{
            asset_id,
            dividends,
        }
    }
}

#[async_trait]
impl Strategy for StaticInSingleStock {
    async fn apply(&self, position: &PortfolioPosition, date: NaiveDate) -> Result<Vec<Transaction>, DataError> {
        let mut transactions = Vec::new();
        if let Some(idx) = cash_flow_idx(date, &self.dividends) {
            let mut dividend = self.dividends[idx].clone();
            dividend.amount.amount *= position.assets[&self.asset_id].position;
            let dividend_transaction = Transaction {
                id: None,
                transaction_type: TransactionType::Dividend {
                    asset_id: self.asset_id,
                },
                cash_flow: dividend,
                note: None,
            };
            transactions.push(dividend_transaction);
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
    dividends: Vec<CashFlow>
}

impl ReInvestInSingleStock {
    pub fn new(asset_id: usize, ticker_id: usize, market: Market, dividends: Vec<CashFlow>) -> ReInvestInSingleStock {
        ReInvestInSingleStock{
            asset_id,
            ticker_id,
            market,
            dividends,
        }
    }
}

#[async_trait]
impl Strategy for ReInvestInSingleStock {
    async fn apply(&self, position: &PortfolioPosition, date: NaiveDate) -> Result<Vec<Transaction>, DataError> {
        let mut transactions = Vec::new();
        if let Some(idx) = cash_flow_idx(date, &self.dividends) {
            let mut dividend = self.dividends[idx].clone();
            let total_dividend = dividend.amount.amount * position.assets[&self.asset_id].position;
            dividend.amount.amount *= total_dividend;
            let dividend_transaction = Transaction {
                id: None,
                transaction_type: TransactionType::Dividend {
                    asset_id: self.asset_id,
                },
                cash_flow: dividend,
                note: None,
            };
            transactions.push(dividend_transaction);
            // reinvest in stock
            let (asset_quote, _quote_currency) = self.market.db().get_last_quote_before_by_id(self.ticker_id, naive_date_to_date_time(&date, 20)).await?;
            let transaction = Transaction{
                id: None,
                transaction_type: TransactionType::Asset{
                    asset_id: self.asset_id,
                    position: total_dividend/asset_quote.price,
                },
                cash_flow: CashFlow::new(-total_dividend, position.cash.currency, date),
                note: None,
            };
            transactions.push(transaction);
        } 

        Ok(transactions)
    }

    fn next_day(&self, date: NaiveDate) -> NaiveDate {
        let one_day = "1D".parse::<TimePeriod>().unwrap();
        one_day.add_to(date, None)
    }
}