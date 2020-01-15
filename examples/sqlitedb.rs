use finql::asset::Asset;
///! Demonstration of storing Assets in Sqlite3 database
use finql::data_handler::DataHandler;
use finql::sqlite_handler::SqliteDB;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() == 2,
        format!("usage: {} <name of sqlite3 db>", args[0])
    );
    let db = SqliteDB::create(&args[1]).unwrap();
    let asset = Asset::new(
        None,
        "Admiral Group plc",
        Some("AODJ58".to_string()),
        Some("GB00B02J6398".to_string()),
        Some("Here are my notes".to_string()),
    );
    let new_id = db.insert_asset(&asset);
    match new_id {
        Ok(id) => println!("Asset has been stored successfully with id {}", id),
        Err(err) => println!(
            "Could not insert asset! Did the database already contained an entry? Error was {}",
            err
        ),
    }
}
