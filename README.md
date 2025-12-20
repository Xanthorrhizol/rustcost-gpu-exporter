
<h1 align="center" style="border-bottom: none">
    <img alt="rustcost Logo" src="https://avatars.githubusercontent.com/u/233272142" width="120"><br>
    <b>rustcost-gpu-exporter</b>
</h1>

<p align="center">
A lightweight, high-performance GPU metrics exporter for NVIDIA & AMD GPUs.<br>
Deployable as a standalone container or as a Kubernetes DaemonSet using the official Helm chart.
</p>

<div align="center">

<!-- Badges -->

<img alt="Docker Pulls" src="https://img.shields.io/docker/pulls/rustcost/gpu-exporter.svg">
<img alt="Docker Image Size" src="https://img.shields.io/docker/image-size/rustcost/gpu-exporter/0.0.1">
<img alt="License: Apache-2.0" src="https://img.shields.io/badge/License-Apache--2.0-blue.svg">

</div>

---

## **Overview**

`rustcost-gpu-exporter` exposes real-time GPU metrics over HTTP
in JSON.

It is designed to be:

* **Extremely fast** (Rust)
* **Tiny footprint** (milliseconds CPU time)
* Supports **NVIDIA & AMD GPUs**
* Deployable on **Kubernetes or standalone containers**

---

## **Usage Options**

You can use rustcost-gpu-exporter in **two ways**:

---

# 1. Deploy on Kubernetes
(DaemonSet via Helm Chart)  
Recommended: Use with rustcost Core + Dashboard

For a complete cost monitoring experience  
including dashboards, alerting, and extended rustcost features  
install the rustcost core Helm chart, which includes ready-to-use dashboards:

https://github.com/rustcost/rustcost-helmchart

---

# **2. Standalone Docker Container**

## Prerequisites for NVIDIA GPUs on Linux

When using NVIDIA GPUs on Linux systems, the following must be installed:

- **NVIDIA Driver**: NVIDIA GPU driver must be installed on the host system
- **NVIDIA Container Toolkit**: Required to expose GPUs to Docker containers

## Running the Container

Run manually on any Linux system:

```bash
docker run -d \
  --gpus all \
  -p 8000:8000 \
  -e GPU_EXPORTER_COMPLEX=0 \
  -e PORT=8000 \
  -e COLLECT_INTERVAL_SEC=60 \
  rustcost/gpu-exporter:0.0.2
```

Metrics now available at:

```
http://localhost:8000/metrics
```

---

## **Environment Variables**

| Variable               | Default | Description                  |
| ---------------------- | ------- | ---------------------------- |
| `GPU_EXPORTER_COMPLEX` | `0`     | Enable heavy/complex metrics |
| `PORT`                 | `8000`  | Web server port              |
| `COLLECT_INTERVAL_SEC` | `60`    | Scrape refresh interval      |

---

## **Example Output**

```json
{
  "amd": [],
  "nvidia": [
    {
      "gpu_memory_total_mb": 4096,
      "gpu_memory_used_mb": 565,
      "gpu_utilization_percent": 15,
      "index": 0,
      "name": "NVIDIA GeForce RTX 3050 Laptop GPU",
      "up": true
    }
  ]
}
```

---

## **Contributing**

We welcome:

✔ Issues
✔ Feature requests
✔ Pull requests

---

## **License**

Licensed under the Apache License, Version 2.0.



