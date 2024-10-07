use std::{path::PathBuf, str::FromStr};
use kino_api::dicts::{
    Database,
    WordNetDatabase
};
use kino_api::google_signin::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::new(&["700878237197-mkgtttho9bah89s928hte3bdkvcmhjqd.apps.googleusercontent.com"], &["metw.cc"]);
    client.init().await;
    println!("{:?}", client.validate(&String::from("eyJhbGciOiJSUzI1NiIsImtpZCI6IjI4YTQyMWNhZmJlM2RkODg5MjcxZGY5MDBmNGJiZjE2ZGI1YzI0ZDQiLCJ0eXAiOiJKV1QifQ.eyJpc3MiOiJodHRwczovL2FjY291bnRzLmdvb2dsZS5jb20iLCJhenAiOiI3MDA4NzgyMzcxOTctbWtndHR0aG85YmFoODlzOTI4aHRlM2Jka3ZjbWhqcWQuYXBwcy5nb29nbGV1c2VyY29udGVudC5jb20iLCJhdWQiOiI3MDA4NzgyMzcxOTctbWtndHR0aG85YmFoODlzOTI4aHRlM2Jka3ZjbWhqcWQuYXBwcy5nb29nbGV1c2VyY29udGVudC5jb20iLCJzdWIiOiIxMTA1ODA4MTQ0MzM4MTU2MjA2MDgiLCJoZCI6Im1ldHcuY2MiLCJlbWFpbCI6Im1lQG1ldHcuY2MiLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwibmJmIjoxNzI4MzMxNDAwLCJuYW1lIjoiTWV0ZWhhbiBTZWx2aSIsImdpdmVuX25hbWUiOiJNZXRlaGFuIiwiZmFtaWx5X25hbWUiOiJTZWx2aSIsImlhdCI6MTcyODMzMTcwMCwiZXhwIjoxNzI4MzM1MzAwLCJqdGkiOiJlYWM2MjIzNGE2ZmJiMThkMGU5NmMzMWMyZjFiNGVlODE3ZTU4Y2U0In0.Eba9vDEkvJ3RlaEY2oalD6hgOvU7mZHlMZBoVqCPFpKUaCeI3WRguk4QCVYbBo94nqcQ3B19ncmMw8_vdOK4k05I8s1VO47OUix5DUkPiHqUSeFbIbpGPVtH6GCoEdBy8G9CpBzuyYCnws4TS3SxJ33_GJsYLILjSOIfUGvK_eblH2MJf7XwOkJajAkgEiCvOaQ-OUuCVXGSg5Y49ig7ctfmEggekpRRN4H5W5JX-ISnHfHb-iBRsnlnX-3YQDkMwtpvyswy04YUc6X3jMgUaJ7Le9bHgbYLxQz1IhGH9Irgxyhu6XDGRya5_Sc2VWt1Jif7zx-0ddrIs8Zdfn2SgA")));

    let start = std::time::Instant::now();
    let database = WordNetDatabase::new(PathBuf::from_str("/home/metw/wn/").unwrap());
    println!("load database: {:?}ms", start.elapsed().as_millis());


    let start = std::time::Instant::now();
    let word = database.get(String::from("in_a_nut_shell"));
    println!("{}", serde_json::to_string_pretty(&word).unwrap());
    println!("{:?}", database.suggest(&String::from("and")));
    println!("get go: {:?}us", start.elapsed().as_micros());


    let start = std::time::Instant::now();
    database.suggest_search(&String::from("adject"));
    println!("suggest_search so: {:?}us", start.elapsed().as_micros());
    println!("{:?}", database.suggest_search(&String::from("so")));

    let start = std::time::Instant::now();
    let mut suggestion = None;
    for _ in 0..20 {
        suggestion = std::hint::black_box(Some(database.suggest(&String::from("in_a_nut_shell"))));
    }
    println!("suggest: {:?}us", start.elapsed().as_micros() / 20);
    println!("{:?}", suggestion);

    std::thread::sleep(std::time::Duration::from_secs(100))
}
