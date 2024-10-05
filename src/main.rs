use std::{path::PathBuf, str::FromStr};
use kino_api::dicts::{
    Database,
    WordNetDatabase
};

fn main() {
    let database = WordNetDatabase::new(PathBuf::from_str("/home/metw/wn/").unwrap());

    let word = database.get(String::from("go"));

    println!("{}", serde_json::to_string_pretty(&word).unwrap());

    std::thread::sleep(std::time::Duration::from_secs(100))
}
