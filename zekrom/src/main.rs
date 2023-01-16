use std::sync::{Arc, RwLock};
use std::time::Duration;

use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::tokio::{spawn, time};
use rocket::State;
use sysinfo::{CpuExt, System, SystemExt};

#[macro_use]
extern crate rocket;

// NOTE(patrik): Extra time for sleeping inside the update for the
// system info (in milliseconds)
const UPDATE_TIME_OFFSET: u64 = 500;

struct MyState {
    sys: Arc<RwLock<System>>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
#[serde(rename_all = "camelCase")]
struct SystemInfo {
    cpu_vendor_id: String,
    cpu_brand: String,
    cpu_usage: f32,
    cpu_freq: u64,

    total_memory: u64,
    free_memory: u64,
    available_memory: u64,
    used_memory: u64,

    total_swap: u64,
    free_swap: u64,
    used_swap: u64,
}

#[get("/system")]
fn system(state: &State<MyState>) -> Json<SystemInfo> {
    let sys_lock = state.sys.read().unwrap();
    let cpu = sys_lock.global_cpu_info();

    Json(SystemInfo {
        cpu_vendor_id: cpu.vendor_id().to_string(),
        cpu_brand: cpu.brand().to_string(),
        cpu_usage: cpu.cpu_usage(),
        cpu_freq: cpu.frequency(),

        total_memory: sys_lock.total_memory(),
        free_memory: sys_lock.free_memory(),
        available_memory: sys_lock.available_memory(),
        used_memory: sys_lock.used_memory(),

        total_swap: sys_lock.total_swap(),
        free_swap: sys_lock.free_swap(),
        used_swap: sys_lock.used_swap(),
    })
}

#[launch]
fn rocket() -> _ {
    let sys = Arc::new(RwLock::new(System::new_all()));

    let sys_lock = sys.clone();
    spawn(async move {
        let mut interval = time::interval(
            System::MINIMUM_CPU_UPDATE_INTERVAL +
                Duration::from_millis(UPDATE_TIME_OFFSET),
        );

        loop {
            interval.tick().await;
            {
                let mut t = sys_lock.write().unwrap();
                t.refresh_all()
            }
        }
    });

    let state = MyState { sys };
    rocket::build().manage(state).mount("/api", routes![system])
}
