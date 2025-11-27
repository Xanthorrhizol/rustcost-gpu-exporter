use std::{
    fs,
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tiny_http::{Header, Response, Server};
use serde_json::{json, Value};

//
// ---------------- NVIDIA MULTI-GPU COLLECTOR -----------------------------
//
fn collect_nvidia_json() -> Value {
    let out = Command::new("nvidia-smi")
        .args([
            "--query-gpu=index,name,utilization.gpu,memory.used,memory.total,\
power.draw,temperature.gpu,clocks.sm,clocks.mem,clocks.gr,\
pcie.link.gen.current,pcie.link.width.current",
            "--format=csv,noheader,nounits",
        ])
        .output();

    let mut gpu_list = Vec::new();

    match out {
        Ok(o) if !o.stdout.is_empty() => {
            let text = String::from_utf8_lossy(&o.stdout);

            for line in text.lines() {
                let parts: Vec<&str> = line.split(',').map(|v| v.trim()).collect();

                if parts.len() == 11 {
                    gpu_list.push(json!({
                        "index": parts[0].parse::<u32>().unwrap_or(0),
                        "name": parts[1],

                        // usage
                        "gpu_utilization_percent": parts[2].parse::<u32>().unwrap_or(0),

                        // memory
                        "gpu_memory_used_mb": parts[3].parse::<u32>().unwrap_or(0),
                        "gpu_memory_total_mb": parts[4].parse::<u32>().unwrap_or(0),

                        // power
                        "power_watts": parts[5].parse::<f32>().unwrap_or(0.0),

                        // temps
                        "temperature_celsius": parts[6].parse::<u32>().unwrap_or(0),

                        // clocks
                        "clock_sm_mhz": parts[7].parse::<u32>().unwrap_or(0),
                        "clock_mem_mhz": parts[8].parse::<u32>().unwrap_or(0),
                        "clock_graphics_mhz": parts[9].parse::<u32>().unwrap_or(0),

                        // PCIe
                        "pcie_gen": parts[10].parse::<u32>().unwrap_or(0),
                        "pcie_width": parts[11].parse::<u32>().unwrap_or(0),

                        "up": true
                    }));
                }
            }
        }
        _ => {}
    }

    // ---------------- GPU process list ------------------------------------
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
            let parts: Vec<&str> = line.split(',').map(|v| v.trim()).collect();
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
//
// ---------------- AMD MULTI-GPU COLLECTOR -------------------------------
//
fn collect_amd_json() -> Value {
    let out = Command::new("/opt/rocm/bin/rocm-smi")
        .args(["--showuse", "--showmemuse", "--showtemp", "--json"])
        .output();

    let mut gpus = Vec::new();

    match out {
        Ok(o) if !o.stdout.is_empty() => {
            let raw = String::from_utf8_lossy(&o.stdout);

            if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                if let Some(cards) = v.get("card") {
                    if let Some(map) = cards.as_object() {
                        for (index, gpuinfo) in map {
                            let idx = index.parse::<u32>().unwrap_or(0);

                            let name = gpuinfo
                                .get("Card series")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown AMD GPU");

                            let util = gpuinfo
                                .get("GPU use (%)")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<u32>().ok())
                                .unwrap_or(0);

                            let mem = gpuinfo
                                .get("GPU Memory Usage (MB)")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0 / 0");

                            let parts: Vec<&str> =
                                mem.split('/').map(|x| x.trim()).collect();

                            let used = parts.get(0).unwrap_or(&"0").parse::<u32>().unwrap_or(0);
                            let total = parts.get(1).unwrap_or(&"0").parse::<u32>().unwrap_or(0);

                            let temp = gpuinfo
                                .get("Temperature (Sensor edge) (C)")
                                .and_then(|v| v.as_str())
                                .and_then(|s| s.parse::<u32>().ok())
                                .unwrap_or(0);

                            gpus.push(json!({
                                "index": idx,
                                "name": name,
                                "gpu_utilization_percent": util,
                                "gpu_memory_used_mb": used,
                                "gpu_memory_total_mb": total,
                                "temperature_celsius": temp,
                                "up": true
                            }));
                        }
                    }
                }
            }
        }

        _ => {}
    }

    json!(gpus)
}

//
// ------------------------- MAIN PROGRAM ---------------------------------
//
fn main() {
    let metrics = Arc::new(Mutex::new(String::from("{\"status\":\"starting\"}")));
    let metrics_bg = metrics.clone();

    // Background collector
    thread::spawn(move || loop {
        let nvidia = collect_nvidia_json();
        let amd = collect_amd_json();

        let combined = json!({
            "nvidia": nvidia,
            "amd": amd
        })
            .to_string();

        // safe cross-platform temporary directory
        let mut path = std::env::temp_dir();
        path.push("gpu_metrics.json");
        let _ = fs::write(&path, &combined);

        if let Ok(mut lock) = metrics_bg.lock() {
            *lock = combined;
        }

        thread::sleep(Duration::from_secs(60));
    });

    // Web server
    let server = Server::http("0.0.0.0:8000").unwrap();
    println!("Serving metrics on http://localhost:8000/metrics");

    for req in server.incoming_requests() {
        if req.url() == "/metrics" {
            let body = metrics
                .lock()
                .map(|s| s.clone())
                .unwrap_or_else(|_| "{\"error\":\"unavailable\"}".to_string());

            let response = Response::from_string(body)
                .with_header(Header::from_bytes(b"Content-Type", b"application/json").unwrap());

            let _ = req.respond(response);
        } else {
            let _ = req.respond(
                Response::from_string("{\"error\":\"not_found\"}")
                    .with_status_code(404),
            );
        }
    }
}
