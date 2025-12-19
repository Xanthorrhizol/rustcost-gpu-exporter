mod amd_collector;
mod nvidia_collector;
mod server;

use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use serde_json::json;

fn main() {
    dotenvy::dotenv().ok(); // load .env automatically

    let complex_mode = std::env::var("GPU_EXPORTER_COMPLEX").unwrap_or_else(|_| "0".into()) == "1";

    let interval = std::env::var("COLLECT_INTERVAL_SEC")
        .unwrap_or_else(|_| "60".into())
        .parse::<u64>()
        .unwrap_or(60);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".into());

    let metrics = Arc::new(Mutex::new(String::from("{\"status\":\"starting\"}")));
    let metrics_bg = metrics.clone();

    thread::spawn(move || {
        loop {
            let nvidia = nvidia_collector::collect_nvidia_json(complex_mode);
            let amd = amd_collector::collect_amd_json(complex_mode);

            let combined = json!({
                "nvidia": nvidia,
                "amd": amd
            })
            .to_string();

            let mut path = std::env::temp_dir();
            path.push("gpu_metrics.json");
            let _ = fs::write(&path, &combined);

            if let Ok(mut lock) = metrics_bg.lock() {
                *lock = combined;
            }

            thread::sleep(Duration::from_secs(interval));
        }
    });

    server::run_server(metrics);
}
