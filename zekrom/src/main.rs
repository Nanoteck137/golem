use std::net::Ipv4Addr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use rocket::serde::json::Json;
use rocket::{Config, State};
use sysinfo::{CpuExt, System, SystemExt};
use system_info::{Capabilities, CpuCoreInfo, SystemInfo};

#[macro_use]
extern crate rocket;

// NOTE(patrik): Extra time for sleeping inside the update for the
// system info (in milliseconds)
const UPDATE_TIME: u64 = 500;

struct MyState {
    sys: Arc<RwLock<System>>,
}

#[get("/capabilities")]
fn capabilities() -> Json<Capabilities> {
    Json(Capabilities {
        has_system_info: true,
        has_docker_info: false,
    })
}

#[get("/system")]
fn system(state: &State<MyState>) -> Json<SystemInfo> {
    let sys_lock = state.sys.read().unwrap();
    let cpu = sys_lock.global_cpu_info();

    let cores = sys_lock
        .cpus()
        .iter()
        .map(|cpu| CpuCoreInfo {
            name: cpu.name().to_string(),
            cpu_usage: cpu.cpu_usage(),
            cpu_freq: cpu.frequency(),
        })
        .collect::<Vec<_>>();

    Json(SystemInfo {
        cpu_vendor_id: cpu.vendor_id().to_string(),
        cpu_brand: cpu.brand().to_string(),
        cpu_usage: cpu.cpu_usage(),
        cpu_freq: cpu.frequency(),
        cpu_cores: cores,

        total_memory: sys_lock.total_memory(),
        free_memory: sys_lock.free_memory(),
        available_memory: sys_lock.available_memory(),
        used_memory: sys_lock.used_memory(),

        total_swap: sys_lock.total_swap(),
        free_swap: sys_lock.free_swap(),
        used_swap: sys_lock.used_swap(),
    })
}

fn fetch_update(sys: Arc<RwLock<System>>) {
    loop {
        {
            let mut lock = sys.write().unwrap();
            lock.refresh_all();
        }
        std::thread::sleep(Duration::from_millis(UPDATE_TIME));
    }
}

#[launch]
fn rocket() -> _ {
    let sys = Arc::new(RwLock::new(System::new_all()));

    let sys_lock = sys.clone();
    std::thread::spawn(move || fetch_update(sys_lock));

    let config = Config {
        address: Ipv4Addr::new(0, 0, 0, 0).into(),
        ..Default::default()
    };

    let state = MyState { sys };
    rocket::custom(config)
        .manage(state)
        .mount("/api", routes![capabilities, system])
}
