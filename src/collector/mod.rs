use serde_json::Value;

pub mod amd;
pub mod nvidia;

pub use amd::Amd;
pub use nvidia::Nvidia;

pub trait Smi {
    fn name(&self) -> String;
    fn gpu_utilization_percent(&self) -> u32;
    #[allow(dead_code)]
    fn gpu_memory_total_mib(&self) -> u32;
    fn gpu_memory_total_mb(&self) -> u32;
    #[allow(dead_code)]
    fn gpu_memory_used_mib(&self) -> u32;
    fn gpu_memory_used_mb(&self) -> u32;
    fn power_watts(&self) -> f32 {
        0.0_f32
    }
    fn temperature_celsius(&self) -> u32 {
        0_u32
    }
    fn clock_sm_mhz(&self) -> u32 {
        0_u32
    }
    fn clock_mem_mhz(&self) -> u32 {
        0_u32
    }
    fn clock_graphics_mhz(&self) -> u32 {
        0_u32
    }
    fn pcie_gen(&self) -> u32 {
        0_u32
    }
    fn pcie_width(&self) -> u32 {
        0_u32
    }
    fn up(&self) -> bool {
        true
    }
}

pub trait GpuCollector {
    fn collect(&self, complex_mode: bool) -> Value;
}
