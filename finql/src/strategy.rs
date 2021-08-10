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


pub struct InvestAllInSingleStock {
    asset_id: usize,
    ticker_id: usize,
    market: Market,
}

impl InvestAllInSingleStock {
    pub fn new(asset_id: usize, ticker_id: usize, market: Market) -> InvestAllInSingleStock {
        InvestAllInSingleStock{
            asset_id,
            ticker_id,
            market,
        }
    }
}

#[async_trait]
impl Strategy for InvestAllInSingleStock {
    async fn apply(&self, position: &PortfolioPosition, date: NaiveDate) -> Result<Vec<Transaction>, DataError> {
        if position.cash.position != 0.0 {
            let (asset_quote, _quote_currency) = self.market.db().get_last_quote_before_by_id(self.ticker_id, naive_date_to_date_time(&date, 20)).await?;
            let transaction = Transaction{
                id: None,
                transaction_type: TransactionType::Asset{
                    asset_id: self.asset_id,
                    position: position.cash.position/asset_quote.price,
                },
                cash_flow: CashFlow::new(-position.cash.position, position.cash.currency, date),
                note: None,
            };
            Ok(vec![transaction])
        } else {
            Ok(Vec::new())
        }
    }

    fn next_day(&self, date: NaiveDate) -> NaiveDate {
        let one_day = "1D".parse::<TimePeriod>().unwrap();
        one_day.add_to(date, None)
    }
}