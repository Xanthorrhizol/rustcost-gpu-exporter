use serde_json::{Value, json};
use std::{path::Path, process::Command};

pub fn collect_nvidia_json(complex: bool) -> Value {
    if !Path::new("/usr/bin/nvidia-smi").exists() && which::which("nvidia-smi").is_err() {
        return json!([]);
    }

    if complex {
        collect_nvidia_complex()
    } else {
        collect_nvidia_simple()
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
            list.push(json!({
                "index": p[0].parse::<u32>().unwrap_or(0),
                "name": p[1],
                "gpu_utilization_percent": p[2].parse::<u32>().unwrap_or(0),
                "gpu_memory_used_mb": p[3].parse::<u32>().unwrap_or(0),
                "gpu_memory_total_mb": p[4].parse::<u32>().unwrap_or(0),
                "up": true
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
                gpu_list.push(json!({
                    "index": p[0].parse::<u32>().unwrap_or(0),
                    "name": p[1],
                    "gpu_utilization_percent": p[2].parse::<u32>().unwrap_or(0),
                    "gpu_memory_used_mb": p[3].parse::<u32>().unwrap_or(0),
                    "gpu_memory_total_mb": p[4].parse::<u32>().unwrap_or(0),
                    "power_watts": p[5].parse::<f32>().unwrap_or(0.0),
                    "temperature_celsius": p[6].parse::<u32>().unwrap_or(0),
                    "clock_sm_mhz": p[7].parse::<u32>().unwrap_or(0),
                    "clock_mem_mhz": p[8].parse::<u32>().unwrap_or(0),
                    "clock_graphics_mhz": p[9].parse::<u32>().unwrap_or(0),
                    "pcie_gen": p[10].parse::<u32>().unwrap_or(0),
                    "pcie_width": p[11].parse::<u32>().unwrap_or(0),
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
