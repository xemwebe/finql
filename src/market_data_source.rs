use crate::market_quotes;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MarketDataSource {
    Manual,
    Yahoo,
    GuruFocus,
    EodHistData,
    AlphaVantage,
    Comdirect,
}

#[derive(Debug, Clone)]
pub struct ParseMarketDataSourceError {}

impl fmt::Display for ParseMarketDataSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parsing market data source failed")
    }
}

impl FromStr for MarketDataSource {
    type Err = ParseMarketDataSourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "manual" => Ok(Self::Manual),
            "yahoo" => Ok(Self::Yahoo),
            "gurufocus" => Ok(Self::GuruFocus),
            "eodhistdata" => Ok(Self::EodHistData),
            "alpha_vantage" => Ok(Self::AlphaVantage),
            "comdirect" => Ok(Self::Comdirect),
            _ => Err(ParseMarketDataSourceError {}),
        }
    }
}

impl fmt::Display for MarketDataSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manual => write!(f, "manual"),
            Self::Yahoo => write!(f, "yahoo"),
            Self::GuruFocus => write!(f, "gurufocus"),
            Self::EodHistData => write!(f, "eodhistdata"),
            Self::AlphaVantage => write!(f, "alpha_vantage"),
            Self::Comdirect => write!(f, "comdirect"),
        }
    }
}

impl MarketDataSource {
    pub fn get_provider(
        &self,
        token: String,
    ) -> Option<Box<dyn market_quotes::MarketQuoteProvider>> {
        match self {
            Self::Yahoo => Some(Box::new(market_quotes::yahoo::Yahoo {})),
            Self::GuruFocus => Some(Box::new(market_quotes::guru_focus::GuruFocus::new(token))),
            Self::EodHistData => Some(Box::new(
                market_quotes::eod_historical_data::EODHistData::new(token),
            )),
            Self::AlphaVantage => Some(Box::new(market_quotes::alpha_vantage::AlphaVantage::new(
                token,
            ))),
            Self::Comdirect => Some(Box::new(market_quotes::comdirect::Comdirect::new())),
            _ => None,
        }
    }
}

