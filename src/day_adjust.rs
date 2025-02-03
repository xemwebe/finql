use cal_calc::Calendar;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

/// Rules to adjust dates to business days
/// The rule "Modified Preceding" commonly referred to in text books
/// was intentionally left out since
#[derive(Deserialize, Serialize, Debug)]
pub enum DayAdjust {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "following")]
    Following,
    #[serde(rename = "preceding")]
    Preceding,
    /// Next business day, if it falls in the same month, otherwise preceding business day
    #[serde(rename = "modified")]
    #[serde(alias = "modified following")]
    Modified,
}

impl DayAdjust {
    pub fn adjust_date(&self, date: NaiveDate, cal: &Calendar) -> NaiveDate {
        match self {
            DayAdjust::None => date,
            DayAdjust::Following => {
                if cal.is_holiday(date.into()) {
                    cal.next_bday(date.into())?.into()
                } else {
                    date
                }
            }
            DayAdjust::Preceding => {
                if cal.is_holiday(date.into()) {
                    cal.prev_bday(date.into())?.into()
                } else {
                    date
                }
            }
            DayAdjust::Modified => {
                if cal.is_business_day(date) {
                    date
                } else {
                    let new_date = cal.next_bday(date.into());
                    if new_date.month() != date.month() {
                        cal.prev_bday(date.into())?.into()
                    } else {
                        new_date
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cal_calc::Holiday;

    #[test]
    fn day_adjust() {
        let holidays = vec![
            Holiday::SingularDay(date!(2019, 10, 10)),
            Holiday::SingularDay(date!(2019, 10, 31)),
        ];
        let cal = Calendar::calc_calendar(&holidays, 2019, 2019);
        let rule = DayAdjust::None;
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 1), &cal),
            date!(2019, 10, 1)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 10), &cal),
            date!(2019, 10, 10)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 31), &cal),
            date!(2019, 10, 31)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 11, 30), &cal),
            date!(2019, 11, 30)
        );
        let rule = DayAdjust::Following;
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 1), &cal),
            date!(2019, 10, 1)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 10), &cal),
            date!(2019, 10, 11)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 31), &cal),
            date!(2019, 11, 1)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 11, 30), &cal),
            date!(2019, 11, 30)
        );
        let rule = DayAdjust::Preceding;
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 1), &cal),
            date!(2019, 10, 1)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 10), &cal),
            date!(2019, 10, 9)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 31), &cal),
            date!(2019, 10, 30)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 11, 30), &cal),
            date!(2019, 11, 30)
        );
        let rule = DayAdjust::Modified;
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 1), &cal),
            date!(2019, 10, 1)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 10), &cal),
            date!(2019, 10, 11)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 10, 31), &cal),
            date!(2019, 10, 30)
        );
        assert_eq!(
            rule.adjust_date(date!(2019, 11, 30), &cal),
            date!(2019, 11, 30)
        );
    }
}
