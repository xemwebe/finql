use std::fs::File;
use std::io::Read;
use finql;
use serde_json;

fn main() {
    let mut file = File::open("./examples/Euroboden_deb_bond.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let bond1 : finql::Bond = serde_json::from_str(&data).unwrap();
    println!("{:?}", bond1);

    let mut file = File::open("./examples/photon_energy_bond.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let bond2 : finql::Bond = serde_json::from_str(&data).unwrap();
    println!("{:?}", bond2);
}

