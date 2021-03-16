use chrono::Utc;
use finql::time_period::TimePeriod;
#[macro_use]
extern crate text_io;

fn main() {
    let today = Utc::now().naive_local().date();
    let quarterly = "3M".parse::<TimePeriod>().unwrap();
    let date_in_three_months = quarterly.add_to(today, None);
    println!(
        "The date a quarter year after {} is the date {}.",
        today, date_in_three_months
    );
    println!(
        "The date has been calculated by adding the period {} to the original date.",
        quarterly
    );

    println!("Try it yourself, enter an arbitrary time period:");
    let input: String = read!("{}\n");
    let period = input.parse::<TimePeriod>().unwrap();
    let future_date = period.add_to(today, None);
    println!("Today plus a time period of {} is {}.", period, future_date);
}
