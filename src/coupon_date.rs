use std::fmt::{Display,Formatter};
use std::fmt;
use std::error;
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self,Visitor};

/// Month and day that serves as a reference for rolling out the cash flows
/// This should equal the (unadjusted) first coupon's end date
#[derive(Debug, PartialEq)]
pub struct CouponDate {
    day: u32,
    month: u32,
}

/// Special error for parsing type CouponDate
#[derive(Debug, Clone)]
pub enum CouponDateError {
    ParseError,
    DayOutOfRange,
    InvalidDay,
    DayToBig,
}

impl fmt::Display for CouponDateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CouponDateError::ParseError => write!(f, "parsing of coupon date failed"),
            CouponDateError::DayOutOfRange => write!(f, "day or month is out of range"),
            CouponDateError::InvalidDay => write!(f, "parsing date or month failed"),
            CouponDateError::DayToBig => write!(f, "day must not be larger than last day of month or 29th of February"),
        }
    }
}

/// This is important for other errors to wrap this one.
impl error::Error for CouponDateError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl de::Error for CouponDateError {
    fn custom<T: fmt::Display>(_: T) -> Self {
        CouponDateError::ParseError
    }
}

use std::num::ParseIntError;
impl std::convert::From<ParseIntError> for CouponDateError {
    fn from(_: ParseIntError) -> CouponDateError {
        CouponDateError::ParseError
    }
}

/// Constructor for CouponDate to check for valid parameters
impl CouponDate {
    pub fn new(day: u32, month: u32) -> Result<CouponDate,CouponDateError> {
        if day==0 || month==0 || month>12 {
            return Err(CouponDateError::DayOutOfRange);
        } 
        // Any year that is not a leap year will do.
        // We exclude explicitly February 29th, which is not a proper chosen coupon date
        let last = crate::calendar::last_day_of_month(2019, month);
        if day>0 && month>0 && month <=12 && day<=last {
            Ok(CouponDate{day:day, month:month})
        } else {
            Err(CouponDateError::DayToBig)
        }
    }
    pub fn day(&self) -> u32 { self.day }
    pub fn month(&self) -> u32 { self.month }
}

/// Write CouponDate as in the form dd.mm
impl Display for CouponDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:0>2}.{:0>2}", self.day, self.month)
    }
}

/// Transform a string into a CouponDate
impl FromStr for CouponDate {
    type Err = CouponDateError;

    fn from_str(coupon_date: &str) -> Result<Self, Self::Err> {
            let nums: Vec<_> = coupon_date.trim().split('.').collect();
            let day =nums[0].parse::<u32>()?;
            let month = nums[1].parse::<u32>()?;
            CouponDate::new(day, month)
        }
}

impl Serialize for CouponDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}",&self))
    }
}

struct CouponDateVisitor;

impl<'de> Visitor<'de> for CouponDateVisitor {
    type Value = CouponDate;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a coupon date of the format <day>.<month>")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match CouponDate::from_str(value) {
            Ok(val) => Ok(val),
            Err(err) => Err(E::custom(format!("{}",err)))
        }
    
    }
}

impl<'de> Deserialize<'de> for CouponDate 
{

    fn deserialize<D>(deserializer: D) -> Result<CouponDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CouponDateVisitor)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sane_coupon_date() {
        let cd = CouponDate::new(2, 1);
        assert!(cd.is_ok());
        let cd = CouponDate::new(29, 2);
        assert!(cd.is_err());
        let cd = CouponDate::new(31, 11);
        assert!(cd.is_err());
        let cd = CouponDate::new(0, 11);
        assert!(cd.is_err());
        let cd = CouponDate::new(31, 0);
        assert!(cd.is_err());
        let cd = CouponDate::new(12, 31);
        assert!(cd.is_err());
    }

    #[test]
    fn deserialize_coupon_date() {
        let input = r#""10.12""#;

        let cd: CouponDate = serde_json::from_str(input).unwrap();
        assert_eq!(cd.day, 10);
        assert_eq!(cd.month, 12);
        let cdt = CouponDate{day: 10, month: 12 };
        assert_eq!(cd, cdt);
    }
    #[test]
    fn serialize_coupon_date() {
        let cd = CouponDate::new(2, 1).unwrap();
        let json = serde_json::to_string(&cd).unwrap();
        assert_eq!(json, r#""02.01""#);
        let cd = CouponDate::new(22, 2).unwrap();
        let json = serde_json::to_string(&cd).unwrap();
        assert_eq!(json, r#""22.02""#);
        let cd = CouponDate::new(10, 12).unwrap();
        let json = serde_json::to_string(&cd).unwrap();
        assert_eq!(json, r#""10.12""#);
        let cd = CouponDate::new(1, 12).unwrap();
        let json = serde_json::to_string(&cd).unwrap();
        assert_eq!(json, r#""01.12""#);
    }
}
