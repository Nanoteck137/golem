use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use rocket::tokio::{self, time};
use rocket::State;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use system_info::{Capabilities, SystemInfo};

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

    #[serde(skip)]
    cap: Option<Capabilities>,
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

type Machines = Vec<Machine>;
type MachineStates = Arc<RwLock<Vec<MachineStatus>>>;

#[derive(Clone)]
struct ProgramState {
    num_machines: usize,
    machines: Machines,
    machine_states: MachineStates,
}

#[get("/")]
fn test(program_state: &State<ProgramState>) -> String {
    let lock = program_state.machine_states.read();
    format!("{:?}", lock)
}

async fn fetch_data_from_machine<T>(machine: &Machine, api: &str) -> Option<T>
where
    T: DeserializeOwned,
{
    let url = format!("{}/{}", machine.api_url(), api);

    match reqwest::get(&url).await {
        Ok(res) => Some(res.json::<T>().await.unwrap()),
        Err(_) => None,
    }
}

async fn check_machine(program_state: &mut ProgramState) {
    // TODO(patrik): Check for machines that is unreachable
    // TODO(patrik): Try to get the capabilities from the machine
    // TODO(patrik): Update the machines capabilities inside the program_state

    let machines = &mut program_state.machines;

    let mut which = Vec::new();

    {
        // TODO(patrik): Remove unwrap
        let machine_states = program_state.machine_states.read().unwrap();

        for (index, (machine, state)) in
            machines.iter().zip(machine_states.iter()).enumerate()
        {
            if let MachineStatus::Unreachable = state {
                // TODO(patrik): Fetch capabilities
                println!(
                    "Machine({}): '{}' is unreachable",
                    index, machine.name
                );
                which.push(index);
            }
        }
    }

    for index in which {
        let cap = fetch_data_from_machine::<Capabilities>(
            &machines[index],
            "capabilities",
        )
        .await;

        machines[index].cap = cap;
    }
}

async fn gather_machine_info(
    program_state: &ProgramState,
) -> Vec<MachineStatus> {
    let machines = &program_state.machines;

    let mut states =
        vec![MachineStatus::Unreachable; program_state.num_machines];

    for (index, machine) in machines.iter().enumerate() {
        if let Some(cap) = &machine.cap {
            if cap.has_system_info {
                let sys_info =
                    fetch_data_from_machine::<SystemInfo>(machine, "system")
                        .await;
                println!("System Info: {:#?}", sys_info);
                states[index] = MachineStatus::Success(MachineData {
                    raw: sys_info.unwrap(),
                });
            }
        }
    }

    states
}

async fn update_state(
    _program_state: &ProgramState,
    states: Vec<MachineStatus>,
) {
}

async fn fetch(program_state: &mut ProgramState) {
    // let mut handles = Vec::new();
    println!("Machines: {:#?}", program_state.machines);

    // If the machines is unreachable then we need to try to get
    // the capabilities, if this failes then try again on next iteration
    // If the machines is not unreachable then we assume the capabilities
    // we have is still valid

    check_machine(program_state).await;
    let states = gather_machine_info(program_state).await;
    update_state(program_state, states).await;

    // TODO(patrik): check_machine()
    // TODO(patrik): gather_machine_info()
    // TODO(patrik): update_state()
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

    let num_machines = config.machines.len();
    let machines = config.machines;
    let machine_states = MachineStates::new(RwLock::new(vec![
        MachineStatus::Unreachable;
        num_machines
    ]));

    let program_state = ProgramState {
        num_machines,
        machines,
        machine_states,
    };

    let mut p = program_state.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            fetch(&mut p).await;
        }
    });

    rocket::build()
        .manage(program_state)
        .mount("/test", routes![test])
}
