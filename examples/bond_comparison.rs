use chrono::NaiveDate;
use finql::bond::Bond;
use finql::currency::Currency;
use finql::fixed_income::{get_cash_flows_after, CashFlow, FixedIncome};
use finql::market::Market;
use serde_json;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

fn main() {
    let mut file = File::open("./examples/Euroboden_deb_bond.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let today = NaiveDate::from_ymd(2019, 12, 11);
    let bond1: Bond = serde_json::from_str(&data).unwrap();
    let market = Market::new();
    let cfs1 = bond1.rollout_cash_flows(1., &market).unwrap();
    let cfs1 = get_cash_flows_after(&cfs1, today);

    let mut file = File::open("./examples/photon_energy_bond.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let bond2: Bond = serde_json::from_str(&data).unwrap();
    let cfs2 = bond2.rollout_cash_flows(1., &market).unwrap();
    let cfs2 = get_cash_flows_after(&cfs2, today);

    let clean_price1 = 103.;
    let clean_price2 = 104.;
    let max_len = std::cmp::max(cfs1.len(), cfs2.len());
    println!("\n\nComparison of key figures as of {}\n", today);
    println!("                          bond1     |     bond2");
    println!("                     =================================");
    println!(
        "Clean price:       {:17.4}|{:17.4}",
        clean_price1, clean_price2
    );

    let accrued1 = bond1.accrued_interest(today).unwrap();
    let accrued2 = bond2.accrued_interest(today).unwrap();
    println!("Accr. interest:    {:17.4}|{:17.4}", accrued1, accrued2);

    let price_quote_factor1 = (bond1.denomination as f64) / 100.;
    let price_quote_factor2 = (bond2.denomination as f64) / 100.;
    let dirty_price1 = clean_price1 + accrued1 / price_quote_factor1;
    let dirty_price2 = clean_price2 + accrued2 / price_quote_factor2;
    println!(
        "Dirty price:       {:17.4}|{:17.4}",
        dirty_price1, dirty_price2
    );

    let eur_curr = Currency::from_str("EUR").unwrap();
    let purchase1_cash_flow = CashFlow::new(-dirty_price1 * price_quote_factor1, eur_curr, today);
    let purchase2_cash_flow = CashFlow::new(-dirty_price2 * price_quote_factor2, eur_curr, today);
    println!(
        "Yield-to-Maturity: {:16.4}%|{:16.4}%",
        100. * bond1.calculate_ytm(&purchase1_cash_flow, &market).unwrap(),
        100. * bond2.calculate_ytm(&purchase2_cash_flow, &market).unwrap()
    );
    println!("\n    Future cash flows bond1      |    Future cash flows bond2");
    println!("===================================================================");
    for i in 0..max_len {
        if i < cfs1.len() {
            print!("{}", cfs1[i]);
        } else {
            print!("                               ");
        }
        print!("  |  ");
        if i < cfs2.len() {
            print!("{}", cfs2[i]);
        }
        println!("");
    }
}
