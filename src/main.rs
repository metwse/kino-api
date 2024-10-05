use std::{path::PathBuf, str::FromStr};
use kino_api::dicts::{
    Database,
    WordNetDatabase
};

fn main() {
    let start = std::time::Instant::now();
    let database = WordNetDatabase::new(PathBuf::from_str("/home/metw/wn/").unwrap());
    println!("load database: {:?}ms", start.elapsed().as_millis());


    let start = std::time::Instant::now();
    let word = database.get(String::from("go"));
    serde_json::to_string_pretty(&word).unwrap();
    //println!("{:?}", database.suggest(String::from("gone")));
    println!("get go: {:?}us", start.elapsed().as_micros());


    let start = std::time::Instant::now();
    database.suggest_search(String::from("adject"));
    println!("suggest_search adject: {:?}us", start.elapsed().as_micros());
    println!("{:?}", database.suggest_search(String::from("adject")));

    let start = std::time::Instant::now();
    database.suggest(String::from("synomyt"));
    println!("suggest synomyt: {:?}us", start.elapsed().as_micros());
    println!("{:?}", database.suggest(String::from("synomyt")));

    std::thread::sleep(std::time::Duration::from_secs(100))
}
