use chrono::{NaiveDate,Datelike};

trait TimePeriod {
    fn add_to_date(&self, date: NaiveDate) -> NaiveDate;
}

struct MonthlyPeriod {
    periods: i32
}

impl TimePeriod for MonthlyPeriod {
    fn add_to_date(&self, date: NaiveDate) -> NaiveDate {
        let mut day = date.day();
        let mut month = date.month() as i32;
        let mut year = date.year();
        year += self.periods / 12;
        let periods = self.periods % 12;
        if month+periods < 1 {
            year -= 1;
            month += 12+periods;
        } else if periods+month > 12{
            year += 1;
            month += periods-12;
        }
        if day>28 {
            let last_date_of_month = get_days_from_month(year, month as u32);
            day = std::cmp::max(day, last_date_of_month);
        }
        NaiveDate::from_ymd(year, month as u32, day)
    }
}

fn get_days_from_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd(
        match month {
            12 => year + 1,
            _ => year,
        },
        match month {
            12 => 1,
            _ => month + 1,
        },
        1,
    )
    .signed_duration_since(NaiveDate::from_ymd(year, month, 1))
    .num_days() as u32
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_quarterly_period() {
        let quarterly = MonthlyPeriod{ periods: 3 };
        let date = NaiveDate::from_ymd(2019,11,18);
        let three_month_later = quarterly.add_to_date(date);
        let ref_date = NaiveDate::from_ymd(2020,2,18);
        assert_eq!(three_month_later, ref_date);
    }
}
