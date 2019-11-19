use finql;
use chrono::NaiveDate;
use finql::TimePeriod;


fn main() {
    let date = NaiveDate::from_ymd(2019,11,18);
    let date_in_three_months = finql::QUARTERLY.add_to(date);
    println!("The date a quarter year after {} is the date {}.", date, date_in_three_months);
    println!("The date has been calculated by adding the period {} to the original date.", finql::QUARTERLY)
}
