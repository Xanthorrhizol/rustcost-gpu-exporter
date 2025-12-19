use serde_json::{json, Value};
use std::process::Command;

pub fn collect_amd_json(_complex: bool) -> Value {
    // ROCm does not support advanced metrics consistently.
    collect_amd_simple()
}

fn collect_amd_simple() -> Value {
    let out = Command::new("/opt/rocm/bin/rocm-smi")
        .args(["--showuse", "--showmemuse", "--showtemp", "--json"])
        .output();

    let mut gpus = Vec::new();

    if let Ok(o) = out {
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

                        let parts: Vec<&str> = mem.split('/').map(|x| x.trim()).collect();
                        let used = parts[0].parse::<u32>().unwrap_or(0);
                        let total = parts[1].parse::<u32>().unwrap_or(0);

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

    Value::Array(gpus)
}
