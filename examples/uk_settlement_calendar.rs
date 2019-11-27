use chrono::{Weekday, NaiveDate};
use finql::{Calendar, Holiday, NthWeekday};

fn main() {
    let uk_settlement_holidays = vec![
        // Saturdays
        Holiday::WeekDay(Weekday::Sat),
        // Sundays
        Holiday::WeekDay(Weekday::Sun),
        // New Year's day
        Holiday::MovableYearlyDay{month: 1, day: 1, first: None, last: None},
        // Good Friday
        Holiday::EasterOffset(-3),
        // first Monday of May, moved two times in history to 8th of May
        Holiday::MonthWeekday{month: 5, weekday: Weekday::Mon, nth: NthWeekday::First, first: None, last: Some(1994) },
        Holiday::SingularDay(NaiveDate::from_ymd(1995,5,8)),
        Holiday::MonthWeekday{month: 5, weekday: Weekday::Mon, nth: NthWeekday::First, first: Some(1996), last: Some(2019) },
        Holiday::SingularDay(NaiveDate::from_ymd(2020,5,8)),
        Holiday::MonthWeekday{month: 5, weekday: Weekday::Mon, nth: NthWeekday::First, first: Some(2021), last: None },
        // last Monday of May (Spring Bank Holiday), has been skipped two times
        Holiday::MonthWeekday{month: 5, weekday: Weekday::Mon, nth: NthWeekday::Last, first: None, last: Some(2001) },
        Holiday::MonthWeekday{month: 5, weekday: Weekday::Mon, nth: NthWeekday::Last, first: Some(2003), last: Some(2011)},
        Holiday::MonthWeekday{month: 5, weekday: Weekday::Mon, nth: NthWeekday::Last, first: Some(2013), last: None },
        // last Monday of August (Summer Bank Holiday)
        Holiday::MonthWeekday{month: 8, weekday: Weekday::Mon, nth: NthWeekday::Last, first: None, last: None },
        // Christmas
        Holiday::MovableYearlyDay{month: 12, day: 25, first: None, last: None},
        // Boxing Day
        Holiday::MovableYearlyDay{month: 12, day: 26, first: None, last: None},
        // Golden Jubilee
        Holiday::SingularDay(NaiveDate::from_ymd(2002,6,3)),
        // Special Spring Holiday
        Holiday::SingularDay(NaiveDate::from_ymd(2002,6,4)),
        // Royal Wedding
        Holiday::SingularDay(NaiveDate::from_ymd(2011,4,29)),
        // Diamond Jubilee
        Holiday::SingularDay(NaiveDate::from_ymd(2012,6,4)),
        // Special Spring Holiday
        Holiday::SingularDay(NaiveDate::from_ymd(2012,6,5)),
        // Introduction of EUR
        Holiday::SingularDay(NaiveDate::from_ymd(1999,12,31)),
    ];
    let uk_cal = Calendar::calc_calendar(&uk_settlement_holidays, 1999, 2020);
    println!("{:#?}", uk_cal);
}
