use serde::{Deserialize, Serialize};
use std::time::Duration;
use sysinfo::System;
use tokio::sync::watch;

pub enum NetworkMessage {
    Io(u32),
}

/// About System static info
#[derive(Clone, Serialize, Deserialize)]
pub struct LandscapeStatic {
    /// System name
    pub name: String,
    /// kernel version
    pub kernel_version: String,
    /// OS version
    pub os_version: String,
    /// host name
    pub host_name: String,
}

/// CPU Usage
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CpuUsage {
    usage: f32,
    name: String,
    vendor_id: String,
    brand: String,
    frequency: u64,
}

impl From<&sysinfo::Cpu> for CpuUsage {
    fn from(cpu: &sysinfo::Cpu) -> Self {
        CpuUsage {
            usage: cpu.cpu_usage(),
            name: cpu.name().to_string(),
            vendor_id: cpu.vendor_id().to_string(),
            brand: cpu.brand().to_string(),
            frequency: cpu.frequency(),
        }
    }
}

/// 内存使用
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct MemUsage {
    total_mem: u64,
    used_mem: u64,
    total_swap: u64,
    used_swap: u64,
}

#[derive(Clone)]
pub struct LandscapeStatus {
    pub cpu_info_watch: watch::Sender<Vec<CpuUsage>>,
    pub mem_info_watch: watch::Sender<MemUsage>,
}

impl LandscapeStatus {
    pub fn new() -> Self {
        let (cpu_watcher_tx, _) = watch::channel(vec![]);
        let (men_watcher_tx, _) = watch::channel(MemUsage::default());

        let inner_cpu_watcher = cpu_watcher_tx.clone();
        let inner_mem_watcher = men_watcher_tx.clone();
        tokio::spawn(async move {
            let mut sys = System::new_all();
            loop {
                sys.refresh_cpu();
                let cpu = sys.global_cpu_info();
                let mut data: Vec<CpuUsage> = vec![];
                data.push(CpuUsage::from(cpu));
                for cpu in sys.cpus() {
                    data.push(CpuUsage::from(cpu));
                }
                let _ = inner_cpu_watcher.send_replace(data);
                sys.refresh_memory();
                let mem = MemUsage {
                    total_mem: sys.total_memory(),
                    used_mem: sys.used_memory(),
                    total_swap: sys.total_swap(),
                    used_swap: sys.used_swap(),
                };

                let _ = inner_mem_watcher.send_replace(mem);

                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        });

        LandscapeStatus {
            cpu_info_watch: cpu_watcher_tx,
            mem_info_watch: men_watcher_tx,
        }
    }
}
