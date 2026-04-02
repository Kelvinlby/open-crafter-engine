use nvml_wrapper::Nvml;
use sysinfo::System;

/// Returns RAM utilization as a percentage (0.0–100.0).
pub fn ram_utilization() -> f64 {
    let mut sys = System::new();
    sys.refresh_memory();

    let total = sys.total_memory();
    if total == 0 {
        return 0.0;
    }
    let used = sys.used_memory();
    (used as f64 / total as f64) * 100.0
}

/// Returns VRAM utilization as a percentage (0.0–100.0).
/// Returns 0.0 if no GPU is available.
pub fn vram_utilization() -> f64 {
    let nvml = match Nvml::init() {
        Ok(n) => n,
        Err(_) => return 0.0,
    };
    let device = match nvml.device_by_index(0) {
        Ok(d) => d,
        Err(_) => return 0.0,
    };
    let mem = match device.memory_info() {
        Ok(m) => m,
        Err(_) => return 0.0,
    };
    if mem.total == 0 {
        return 0.0;
    }
    (mem.used as f64 / mem.total as f64) * 100.0
}

/// Returns GPU utilization as a percentage (0.0–100.0).
/// Returns 0.0 if no GPU is available.
pub fn gpu_utilization() -> f64 {
    let nvml = match Nvml::init() {
        Ok(n) => n,
        Err(_) => return 0.0,
    };
    let device = match nvml.device_by_index(0) {
        Ok(d) => d,
        Err(_) => return 0.0,
    };
    let util = match device.utilization_rates() {
        Ok(u) => u,
        Err(_) => return 0.0,
    };
    util.gpu as f64
}
