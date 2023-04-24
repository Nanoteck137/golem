use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use rocket::serde::json::Json;
use rocket::State;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use system_info::{Capabilities, SystemInfo};

#[macro_use]
extern crate rocket;

// TODO(patrik):
//   - Add icons to the machines
//   - Serve static files for the icons so that the frontend can load them

// NOTE(patrik): Routes
//   - GET /api/machines
//     - Get basic infomation about all of the machines registered by
//       the backend
//   - GET /api/machine/:id
//     - Get full infomation about the machine

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

#[derive(Serialize, Clone, Debug)]
struct MachineData {
    system_info: SystemInfo,
}

#[derive(Serialize, Clone, Debug)]
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

#[derive(Serialize)]
struct Res {
    name: String,
    status: String,
    data: Option<MachineData>,
}

#[get("/")]
fn test(program_state: &State<ProgramState>) -> Json<Vec<Res>> {
    let lock = program_state.machine_states.read().unwrap();
    let machines = &program_state.machines;

    let mut result = Vec::new();

    for (index, machine) in machines.iter().enumerate() {
        let state = &lock[index];

        let (status, data) = match state {
            MachineStatus::Unreachable => ("Unreachable", None),
            MachineStatus::Success(data) => ("Success", Some(data.clone())),
        };

        result.push(Res {
            name: machine.name.clone(),
            status: status.to_string(),
            data,
        });
    }

    Json(result)
}

fn fetch_data_from_machine<T>(machine: &Machine, api: &str) -> Option<T>
where
    T: DeserializeOwned,
{
    let url = format!("{}/{}", machine.api_url(), api);

    match reqwest::blocking::get(&url) {
        Ok(res) => Some(res.json::<T>().unwrap()),
        Err(_) => None,
    }
}

fn check_machine(program_state: &mut ProgramState) {
    let machines = &mut program_state.machines;

    let mut which = Vec::new();

    {
        let machine_states = program_state
            .machine_states
            .read()
            .expect("Failed to get read lock on machine states");

        for (index, (machine, state)) in
            machines.iter().zip(machine_states.iter()).enumerate()
        {
            if let MachineStatus::Unreachable = state {
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
        );

        machines[index].cap = cap;
    }
}

fn gather_machine_info(program_state: &ProgramState) -> Vec<MachineStatus> {
    let machines = &program_state.machines;

    let mut states =
        vec![MachineStatus::Unreachable; program_state.num_machines];

    for (index, machine) in machines.iter().enumerate() {
        if let Some(cap) = &machine.cap {
            if cap.has_system_info {
                let sys_info =
                    fetch_data_from_machine::<SystemInfo>(machine, "system");

                if let Some(sys_info) = sys_info {
                    states[index] = MachineStatus::Success(MachineData {
                        system_info: sys_info,
                    });
                }
            }
        }
    }

    states
}

fn update_state(program_state: &ProgramState, states: Vec<MachineStatus>) {
    let mut lock = program_state.machine_states.write().unwrap();
    *lock = states;
}

fn fetch(program_state: &mut ProgramState) {
    // If the machines is unreachable then we need to try to get
    // the capabilities, if this failes then try again on next iteration
    // If the machines is not unreachable then we assume the capabilities
    // we have is still valid

    check_machine(program_state);
    let states = gather_machine_info(program_state);
    update_state(program_state, states);
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
    std::thread::spawn(move || loop {
        fetch(&mut p);
        std::thread::sleep(Duration::from_secs(1));
    });

    rocket::build()
        .manage(program_state)
        .mount("/test", routes![test])
}
