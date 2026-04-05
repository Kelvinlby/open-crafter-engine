use nvml_wrapper::Nvml;

/// Returns VRAM utilization as a percentage (0.0–100.0).
/// Pass an already-initialized `Nvml` instance to avoid redundant init costs.
/// Returns 0.0 if the GPU is unavailable.
pub fn vram_utilization(nvml: &Nvml) -> f64 {
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
/// Pass an already-initialized `Nvml` instance to avoid redundant init costs.
/// Returns 0.0 if the GPU is unavailable.
pub fn gpu_utilization(nvml: &Nvml) -> f64 {
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
