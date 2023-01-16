use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub has_system_info: bool,
    pub has_docker_info: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub cpu_vendor_id: String,
    pub cpu_brand: String,
    pub cpu_usage: f32,
    pub cpu_freq: u64,

    pub total_memory: u64,
    pub free_memory: u64,
    pub available_memory: u64,
    pub used_memory: u64,

    pub total_swap: u64,
    pub free_swap: u64,
    pub used_swap: u64,
}
