use super::{GpuCollector, Smi};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::process::Command;

pub struct Amd;

#[derive(Debug, serde::Deserialize)]
struct Card {
    #[serde(rename = "GPU use (%)")]
    gpu_utilization_percent: String,
    #[serde(rename = "VRAM Total Memory (B)")]
    gpu_memory_total: String,
    #[serde(rename = "VRAM Total Used Memory (B)")]
    gpu_memory_used: String,
    #[serde(rename = "Card Series")]
    series: String,
    #[serde(rename = "Card SKU")]
    sku: String,
}

impl Smi for Card {
    fn name(&self) -> String {
        format!("{} {}", self.series, self.sku)
    }
    fn gpu_utilization_percent(&self) -> u32 {
        self.gpu_utilization_percent.parse::<u32>().unwrap_or(0)
    }
    #[allow(dead_code)]
    fn gpu_memory_total_mib(&self) -> u32 {
        (self.gpu_memory_total.parse::<u64>().unwrap_or(0) / 1024_u64.pow(2)) as u32
    }
    fn gpu_memory_total_mb(&self) -> u32 {
        (self.gpu_memory_total.parse::<u64>().unwrap_or(0) / 1000_u64.pow(2)) as u32
    }
    #[allow(dead_code)]
    fn gpu_memory_used_mib(&self) -> u32 {
        (self.gpu_memory_used.parse::<u64>().unwrap_or(0) / 1024_u64.pow(2)) as u32
    }
    fn gpu_memory_used_mb(&self) -> u32 {
        (self.gpu_memory_used.parse::<u64>().unwrap_or(0) / 1000_u64.pow(2)) as u32
    }
}

impl GpuCollector for Amd {
    fn collect(&self, complex_mode: bool) -> Value {
        if complex_mode {
            unimplemented!();
        } else {
            collect_amd_simple()
        }
    }
}
fn collect_amd_simple() -> Value {
    let out = Command::new("/opt/rocm/bin/rocm-smi")
        .args([
            "--showuse",
            "--showproductname",
            "--showmeminfo=vram",
            "--json",
        ])
        .output();

    let mut gpus = Vec::new();

    if let Ok(o) = out {
        let raw = String::from_utf8_lossy(&o.stdout);

        if let Ok(v) = serde_json::from_str::<HashMap<String, Card>>(&raw) {
            for (card_id, card) in v {
                let idx = card_id
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                    .unwrap_or(0);
                gpus.push(json!({
                    "index": idx,
                    "name": card.name(),
                    "gpu_utilization_percent": card.gpu_utilization_percent(),
                    "gpu_memory_used_mb": card.gpu_memory_used_mb(),
                    "gpu_memory_total_mb": card.gpu_memory_total_mb(),
                    "up": card.up(),
                }));
            }
        }
    }

    Value::Array(gpus)
}
