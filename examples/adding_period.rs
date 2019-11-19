use finql;
use chrono::Utc;
use finql::TimePeriod;
#[macro_use] extern crate text_io;


fn main() {
    let today = Utc::now().naive_local().date();
    let date_in_three_months = finql::QUARTERLY.add_to(today);
    println!("The date a quarter year after {} is the date {}.", today, date_in_three_months);
    println!("The date has been calculated by adding the period {} to the original date.", finql::QUARTERLY);

    println!("Try it yourself, enter an arbitrary time period:");
    let input: String = read!("{}\n");
    let period = finql::time_period::from_str(&input).unwrap();
    let future_date = period.add_to(today);
    println!("The today plus a time period of {} is {}.", period, future_date);
}
