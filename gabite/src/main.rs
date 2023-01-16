use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use rocket::tokio::{self, time};
use rocket::State;
use serde::Deserialize;
use system_info::SystemInfo;

#[macro_use]
extern crate rocket;

#[derive(Deserialize, Debug)]
struct Config {
    machines: Vec<Machine>,
}

#[derive(Deserialize, Clone, Debug)]
struct Machine {
    name: String,
    ip: String,
    api_port: u16,
}

impl Machine {
    fn api_url(&self) -> String {
        format!("http://{}:{}/api", self.ip, self.api_port)
    }
}

#[derive(Clone, Debug)]
struct MachineData {
    raw: SystemInfo,
}

#[derive(Clone, Debug)]
enum MachineStatus {
    Unreachable,
    Success(MachineData),
}

type Test = Arc<RwLock<Vec<MachineStatus>>>;

struct ProgramState {
    test: Test,
}

#[get("/")]
fn test(program_state: &State<ProgramState>) -> String {
    let lock = program_state.test.read();
    format!("{:?}", lock)
}

async fn fetch(machines: &Vec<Machine>, test: &Test) {
    let mut handles = Vec::new();

    for (index, machine) in machines.iter().enumerate() {
        let machine_name = machine.name.clone();
        let machine_url = machine.api_url();

        let handle = tokio::spawn(async move {
            let url = format!("{machine_url}/system");
            // println!("URL: {}", url);
            match reqwest::get(&url).await {
                Ok(res) => {
                    println!("{} {}: {:?}", machine_name, url, res);
                    let info = res.json::<SystemInfo>().await.unwrap();
                    return (
                        index,
                        MachineStatus::Success(MachineData { raw: info }),
                    );
                }

                Err(e) => {
                    if e.is_connect() {
                        println!("{}: Failed to connect", machine_name);
                    } else {
                        println!("Unknown error");
                    }

                    return (index, MachineStatus::Unreachable);
                }
            }
        });

        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        let res = handle.await.unwrap();
        results.push(res);
    }

    {
        let mut lock = test.write().unwrap();
        for (index, res) in results {
            lock[index] = res;
        }
    }
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

    let program_state = {
        let arr = vec![MachineStatus::Unreachable; config.machines.len()];
        let test = Test::new(RwLock::new(arr));
        ProgramState { test }
    };

    let machines = config.machines.clone();
    let test = program_state.test.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            fetch(&machines, &test).await;
        }
    });

    rocket::build()
        .manage(program_state)
        .mount("/test", routes![test])
}
