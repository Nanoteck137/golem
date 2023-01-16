use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Duration;

use rocket::tokio::{self, time};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    machines: Vec<Machine>,
}

#[derive(Deserialize, Debug)]
struct Machine {
    name: String,
    ip: String,
    api_port: u16,
}

struct MachineData {}

#[macro_use]
extern crate rocket;

#[get("/")]
fn test() -> String {
    format!("Test")
}

fn fetch() {
    // println!("Fetch Data from machines");
}

fn read_file<P>(filepath: P) -> String
where
    P: AsRef<Path>,
{
    let mut file =
        File::open(filepath).expect("Failed to open file for reading");
    let mut result = String::new();
    file.read_to_string(&mut result)
        .expect("Failed to read file");

    result
}

#[launch]
fn rocket() -> _ {
    let config = read_file("config.json");
    let config = serde_json::from_str::<Config>(&config)
        .expect("Failed to parse config.json");

    println!("Config: {:#?}", config);

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            fetch();
        }
    });

    rocket::build().mount("/test", routes![test])
}
