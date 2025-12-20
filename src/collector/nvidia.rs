use super::{GpuCollector, Smi};
use serde_json::{Value, json};
use std::process::Command;

pub struct Nvidia;

#[derive(Default, Debug)]
struct Card {
    pub index: u32,
    pub name: String,
    pub gpu_utilization_percent: u32,
    pub gpu_memory_used_mib: u32,
    pub gpu_memory_total_mib: u32,
    pub power_watts: Option<f32>,
    pub temperature_celsius: Option<u32>,
    pub clock_sm_mhz: Option<u32>,
    pub clock_mem_mhz: Option<u32>,
    pub clock_graphics_mhz: Option<u32>,
    pub pcie_gen: Option<u32>,
    pub pcie_width: Option<u32>,
}

impl Smi for Card {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn gpu_utilization_percent(&self) -> u32 {
        self.gpu_utilization_percent
    }
    fn gpu_memory_total_mib(&self) -> u32 {
        self.gpu_memory_total_mib
    }
    fn gpu_memory_total_mb(&self) -> u32 {
        (self.gpu_memory_total_mib as u64 * 1024_u64.pow(2) / 1000_u64.pow(2)) as u32
    }
    fn gpu_memory_used_mib(&self) -> u32 {
        self.gpu_memory_used_mib
    }
    fn gpu_memory_used_mb(&self) -> u32 {
        (self.gpu_memory_used_mib as u64 * 1024_u64.pow(2) / 1000_u64.pow(2)) as u32
    }
    fn power_watts(&self) -> f32 {
        self.power_watts.unwrap_or(0.0)
    }
    fn temperature_celsius(&self) -> u32 {
        self.temperature_celsius.unwrap_or(0)
    }
    fn clock_sm_mhz(&self) -> u32 {
        self.clock_sm_mhz.unwrap_or(0)
    }
    fn clock_mem_mhz(&self) -> u32 {
        self.clock_mem_mhz.unwrap_or(0)
    }
    fn clock_graphics_mhz(&self) -> u32 {
        self.clock_graphics_mhz.unwrap_or(0)
    }
    fn pcie_gen(&self) -> u32 {
        self.pcie_gen.unwrap_or(0)
    }
    fn pcie_width(&self) -> u32 {
        self.pcie_width.unwrap_or(0)
    }
}

impl GpuCollector for Nvidia {
    fn collect(&self, complex_mode: bool) -> Value {
        if complex_mode {
            collect_nvidia_complex()
        } else {
            collect_nvidia_simple()
        }
    }
}

//
// SIMPLE MODE (Type3-friendly)
//
fn collect_nvidia_simple() -> Value {
    let out = Command::new("nvidia-smi")
        .args([
            "--query-gpu=index,name,utilization.gpu,memory.used,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output();

    let Ok(o) = out else {
        return json!([]);
    };
    if o.stdout.is_empty() {
        return json!([]);
    }

    let mut list = Vec::new();
    let s = String::from_utf8_lossy(&o.stdout);

    for line in s.lines() {
        let p: Vec<&str> = line.split(',').map(|x| x.trim()).collect();
        if p.len() == 5 {
            let card = Card {
                index: p[0].parse::<u32>().unwrap_or(0),
                name: p[1].to_string(),
                gpu_utilization_percent: p[2].parse::<u32>().unwrap_or(0),
                gpu_memory_used_mib: p[3].parse::<u32>().unwrap_or(0),
                gpu_memory_total_mib: p[4].parse::<u32>().unwrap_or(0),
                ..Default::default()
            };

            list.push(json!({
                "index": card.index,
                "name": card.name(),
                "gpu_utilization_percent": card.gpu_utilization_percent(),
                "gpu_memory_used_mb": card.gpu_memory_used_mib(),
                "gpu_memory_total_mb": card.gpu_memory_total_mib(),
                "up": card.up(),
            }));
        }
    }

    Value::Array(list)
}

//
// COMPLEX MODE (Type3-friendly)
//
fn collect_nvidia_complex() -> Value {
    let out = Command::new("nvidia-smi")
        .args([
            "--query-gpu=index,name,utilization.gpu,memory.used,memory.total,\
power.draw,temperature.gpu,clocks.sm,clocks.mem,clocks.gr,\
pcie.link.gen.current,pcie.link.width.current",
            "--format=csv,noheader,nounits",
        ])
        .output();

    let mut gpu_list = Vec::new();

    if let Ok(o) = out {
        let txt = String::from_utf8_lossy(&o.stdout);
        for line in txt.lines() {
            let p: Vec<&str> = line.split(',').map(|v| v.trim()).collect();
            if p.len() == 11 {
                let card = Card {
                    index: p[0].parse::<u32>().unwrap_or(0),
                    name: p[1].to_string(),
                    gpu_utilization_percent: p[2].parse::<u32>().unwrap_or(0),
                    gpu_memory_used_mib: p[3].parse::<u32>().unwrap_or(0),
                    gpu_memory_total_mib: p[4].parse::<u32>().unwrap_or(0),
                    power_watts: Some(p[5].parse::<f32>().unwrap_or(0.0)),
                    temperature_celsius: Some(p[6].parse::<u32>().unwrap_or(0)),
                    clock_sm_mhz: Some(p[7].parse::<u32>().unwrap_or(0)),
                    clock_mem_mhz: Some(p[8].parse::<u32>().unwrap_or(0)),
                    clock_graphics_mhz: Some(p[9].parse::<u32>().unwrap_or(0)),
                    pcie_gen: Some(p[10].parse::<u32>().unwrap_or(0)),
                    pcie_width: Some(p[11].parse::<u32>().unwrap_or(0)),
                };
                gpu_list.push(json!({
                    "index": card.index,
                    "name": card.name(),
                    "gpu_utilization_percent": card.gpu_utilization_percent(),
                    "gpu_memory_used_mb": card.gpu_memory_used_mib(),
                    "gpu_memory_total_mb": card.gpu_memory_total_mib(),
                    "power_watts": card.power_watts(),
                    "temperature_celsius": card.temperature_celsius(),
                    "clock_sm_mhz": card.clock_sm_mhz(),
                    "clock_mem_mhz": card.clock_mem_mhz(),
                    "clock_graphics_mhz": card.clock_graphics_mhz(),
                    "pcie_gen": card.pcie_gen(),
                    "pcie_width": card.pcie_width(),
                    "up": true
                }));
            }
        }
    }

    // PROCESS LIST
    let proc_out = Command::new("nvidia-smi")
        .args([
            "--query-compute-apps=gpu_uuid,pid,process_name,used_memory",
            "--format=csv,noheader,nounits",
        ])
        .output();

    let mut processes = Vec::new();
    if let Ok(p) = proc_out {
        let s = String::from_utf8_lossy(&p.stdout);
        for line in s.lines() {
            let parts: Vec<&str> = line.split(',').map(|x| x.trim()).collect();
            if parts.len() == 4 {
                processes.push(json!({
                    "gpu_uuid": parts[0],
                    "pid": parts[1].parse::<u32>().unwrap_or(0),
                    "process_name": parts[2],
                    "used_gpu_memory_mb": parts[3].parse::<u32>().unwrap_or(0)
                }));
            }
        }
    }

    json!({
        "gpus": gpu_list,
        "processes": processes
    })
}
