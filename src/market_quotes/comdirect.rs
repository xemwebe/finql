/// A tool to fetch prices by parsing comdirect web page
use super::{MarketQuoteError, MarketQuoteProvider};
use crate::date_time_helper::date_time_from_str;
use crate::quote::{Quote, Ticker};
use chrono::{DateTime, Utc};
use scraper::{Html, Selector};

#[derive(Debug)]
pub struct ComdirectQuote {
    date: DateTime<Utc>,
    high: Option<f64>,
    low: Option<f64>,
    close: f64,
    volume: Option<f64>,
}

pub struct Comdirect {
    url: String,
    hurl1: String,
    hurl2: String,
    hurl3: String,
}

impl Comdirect {
    pub fn new() -> Comdirect {
        Comdirect{
            url: "https://www.comdirect.de/inf/aktien/detail/uebersicht.html?ID_NOTATION=".to_string(),
            hurl1: "https://www.comdirect.de/inf/kursdaten/historic.csv?DATETIME_TZ_END_RANGE_FORMATED=".to_string(),
            hurl2: "&DATETIME_TZ_START_RANGE_FORMATED=".to_string(),
            hurl3: "&INTERVALL=16&SHOW_CORPORATE_ACTION=1&WITH_EARNINGS=false&ID_NOTATION=".to_string(),
        }
    }

    pub fn get_latest_quote(&self, id: &str) -> Result<f64, MarketQuoteError> {
        let mut resp = reqwest::get(&format!("{}{}", self.url, id))
            .map_err(|_| MarketQuoteError::FetchFailed("request failed".to_string()))?;
        if !resp.status().is_success() {
            return Err(MarketQuoteError::FetchFailed(
                "unexpected server response".to_string(),
            ));
        }

        let body = resp
            .text()
            .map_err(|_| MarketQuoteError::FetchFailed("couln't extract body".to_string()))?;
        // parses string of HTML as a document
        let fragment = Html::parse_document(&body);
        // parses based on a CSS selector
        let quote_selector = Selector::parse(".realtime-indicator").unwrap();
        // fetch the first hit, which is the moste recent quote
        match fragment.select(&quote_selector).next() {
            Some(first_quote) => {
                let quote = first_quote.text().collect::<Vec<_>>();
                quote[0]
                    .replace(".", "")
                    .replace(",", ".")
                    .parse()
                    .map_err(|_| {
                        MarketQuoteError::FetchFailed("quote is not a valid number".to_string())
                    })
            }
            None => Err(MarketQuoteError::FetchFailed(
                "couldn't found quote".to_string(),
            )),
        }
    }

    // Get history as quote list formatted list
    pub fn get_quote_history(
        &self,
        id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<ComdirectQuote>, MarketQuoteError> {
        let url = format!(
            "{}{}{}{}{}{}",
            self.hurl1,
            end.format("%d.%m.%Y"),
            self.hurl2,
            start.format("%d.%m.%Y"),
            self.hurl3,
            id
        );
        let mut resp = reqwest::get(&url)
            .map_err(|_| MarketQuoteError::FetchFailed("request failed".to_string()))?;
        if !resp.status().is_success() {
            return Err(MarketQuoteError::FetchFailed(
                "unexpected server response".to_string(),
            ));
        }

        let body = resp
            .text()
            .map_err(|_| MarketQuoteError::FetchFailed("couln't extract body".to_string()))?;

        Self::parse_csv(&body)
    }

    pub fn parse_csv(text: &str) -> Result<Vec<ComdirectQuote>, MarketQuoteError> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b';')
            .flexible(true)
            .from_reader(text.as_bytes());
        let mut skip_line = true;
        let mut quotes = Vec::new();
        for record in reader.records() {
            if let Ok(record) = record {
                if skip_line {
                    if record.len() >= 1 {
                        if let Some(first_field) = record.get(0) {
                            // start with next line
                            if first_field == "Datum" {
                                skip_line = false;
                            }
                        }
                    }
                    continue;
                }
                let close = Self::num_opt(record.get(3));
                if close.is_none() {
                    continue;
                }
                let date_time_str = record
                    .get(0)
                    .ok_or(MarketQuoteError::FetchFailed("empty field".to_string()))?;
                let date = date_time_from_str(date_time_str, "%d.%m.%Y", 18);
                if date.is_err() {
                    continue;
                }
                quotes.push(ComdirectQuote {
                    date: date.unwrap(),
                    high: Self::num_opt(record.get(1)),
                    low: Self::num_opt(record.get(2)),
                    close: close.unwrap(),
                    volume: Self::num_opt(record.get(4)),
                });
            }
        }
        Ok(quotes)
    }

    fn num_opt(num_str: Option<&str>) -> Option<f64> {
        match num_str {
            None => None,
            Some(num_str) => num_str.replace(".", "").replace(",", ".").parse().ok(),
        }
    }
}

impl MarketQuoteProvider for Comdirect {
    /// Fetch latest quote
    fn fetch_latest_quote(&self, ticker: &Ticker) -> Result<Quote, MarketQuoteError> {
        let codi = Comdirect::new();
        let price = codi.get_latest_quote(&ticker.name)?;
        let time = Utc::now();
        Ok(Quote {
            id: None,
            ticker: ticker.id.unwrap(),
            price,
            time: time,
            volume: None,
        })
    }
    /// Fetch historic quotes between start and end date
    fn fetch_quote_history(
        &self,
        ticker: &Ticker,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Quote>, MarketQuoteError> {
        let codi = Comdirect::new();
        let codi_quotes = codi.get_quote_history(&ticker.name, start, end)?;
        let mut quotes = Vec::new();
        let ticker = ticker.id.unwrap();
        for quote in &codi_quotes {
            quotes.push(Quote {
                id: None,
                ticker,
                price: quote.close,
                time: quote.date,
                volume: quote.volume,
            })
        }
        Ok(quotes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::Currency;
    use crate::quote::MarketDataSource;
    use chrono::offset::TimeZone;
    use std::str::FromStr;

    #[test]
    fn test_comdirect_fetch_quote() {
        let codi = Comdirect::new();
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            // comdirects id for AAPL quote at Nasdaq
            name: "253929".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: MarketDataSource::Comdirect,
            priority: 1,
            factor: 1.0,
        };
        let quote = codi.fetch_latest_quote(&ticker).unwrap();
        assert!(quote.price != 0.0);
    }

    #[test]
    fn test_comdirect_quote_history() {
        let codi = Comdirect::new();
        let ticker = Ticker {
            id: Some(1),
            asset: 1,
            // comdirects id for AAPL quote at Nasdaq
            name: "253929".to_string(),
            currency: Currency::from_str("EUR").unwrap(),
            source: MarketDataSource::Comdirect,
            priority: 1,
            factor: 1.0,
        };
        let start = Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0);
        let end = Utc.ymd(2020, 1, 31).and_hms_milli(23, 59, 59, 999);
        let quotes = codi.fetch_quote_history(&ticker, start, end).unwrap();
        assert_eq!(quotes.len(), 21);
        assert!(quotes[0].price != 0.0);
    }

    #[test]
    fn test_parse_codi_csv() {
        let input = r#""Some skipped asset info"

"Datum";"Er�ffnung";"Hoch";"Tief";"Schluss";"Volumen"
"24.04.2020";"48,204";"48,80";"48,13";"48,80";"341,00"
"23.04.2020";"48,294";"48,752";"48,01";"48,01";"5.153,00"
"22.04.2020";"47,35";"48,176";"47,35";"47,987";"3.610,00"
"21.04.2020";"48,542";"48,542";"46,736";"46,787";"4.621,00"
"20.04.2020";"49,124";"49,152";"48,219";"48,219";"10.023,00""#;

        let quotes = Comdirect::parse_csv(input).unwrap();
        println!("{:#?}", quotes);
        assert_eq!(quotes.len(), 5);
        assert_eq!(quotes[4].close, 48.219);
    }
}
