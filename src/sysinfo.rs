use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

pub mod models;

#[derive(Clone, Default)]
pub struct InnerState {
    /// (total, vec<cpus>)
    pub cpu: Arc<RwLock<(models::CPUStats, Vec<models::CPUStats>)>>,
    /// memory
    pub memory: Arc<RwLock<models::MemoryStats>>,
    /// disk
    pub disks: Arc<RwLock<HashMap<String, models::DiskBlockStats>>>,
}

#[derive(Clone)]
pub struct SysInfo {
    disks: Vec<String>,

    state: InnerState,
}

impl SysInfo {
    pub fn new() -> SysInfo {
        SysInfo {
            disks: vec!["sda".to_string()],
            state: InnerState::default(),
        }
    }

    pub async fn update(&self) -> Result<()> {
        // update cpu
        let stat = fs::read_to_string("/proc/stat").await?;
        let stat_lines: Vec<_> = stat.lines().collect();
        anyhow::ensure!(stat_lines.len() > 0, "empty cpu");

        let cpu_total_stats = parse_cpu_stat(stat_lines[0])?;
        let mut cpu_stats = vec![];
        for line in stat_lines
            .into_iter()
            .skip(1)
            .filter(|it| it.starts_with("cpu"))
        {
            let stats = parse_cpu_stat(line)?;
            cpu_stats.push(stats);
        }

        *self.state.cpu.write().await = (cpu_total_stats, cpu_stats);

        // update memory
        let stat = fs::read_to_string("/proc/meminfo").await?;
        let stat_map = stat
            .lines()
            .map(|it| {
                let mut iter = it.split(":").map(|it| it.trim());
                (iter.next().unwrap(), iter.next().unwrap())
            })
            .map(|(key, value)| (key, byte_unit::Byte::from_str(value).unwrap().get_bytes()))
            .collect::<HashMap<_, _>>();

        let total = stat_map.get("MemTotal").map(|it| *it).unwrap_or_default() as u64;
        let free = stat_map.get("MemFree").map(|it| *it).unwrap_or_default() as u64;
        let available = stat_map
            .get("MemAvailable")
            .map(|it| *it)
            .unwrap_or_default() as u64;
        let buffer = stat_map.get("Buffers").map(|it| *it).unwrap_or_default() as u64;
        let cached = stat_map.get("Cached").map(|it| *it).unwrap_or_default() as u64;
        let swap_total = stat_map.get("SwapTotal").map(|it| *it).unwrap_or_default() as u64;
        let swap_free = stat_map.get("SwapFree").map(|it| *it).unwrap_or_default() as u64;
        let memory_stats = models::MemoryStats {
            total,
            free,
            available,
            buffer,
            cached,
            swap_total,
            swap_free,
        };
        *self.state.memory.write().await = memory_stats;

        // update disk
        let mut disk_stats = HashMap::new();
        for path in &self.disks {
            let sys_path = format!("/sys/block/{}/stat", path);
            let line = fs::read_to_string(sys_path).await?;

            let stats = parse_disk_stat(&path, &line)?;
            disk_stats.insert(path.clone(), stats);
        }

        Ok(())
    }

    /// CPU total stats
    pub async fn cpu_stat(&self) -> models::CPUStats {
        self.state.cpu.read().await.0
    }

    /// All CPU stats
    pub async fn cpu_stats(&self) -> Vec<models::CPUStats> {
        self.state.cpu.read().await.1.clone()
    }

    /// Pre CPU stats
    pub async fn cpu_stat_pre(&self, id: u64) -> models::CPUStats {
        self.state.cpu.read().await.1[id as usize + 1]
    }

    /// Memory status
    pub async fn mem_stat(&self) -> models::MemoryStats {
        *self.state.memory.read().await
    }

    /// Disk stats
    pub async fn disk_stat(&self, disk: &str) -> Result<models::DiskBlockStats> {
        self.state
            .disks
            .read()
            .await
            .get(disk)
            .cloned()
            .ok_or(anyhow::anyhow!("no disk"))
    }

    /// All Disk stats
    pub async fn disks(&self) -> Vec<models::DiskBlockStats> {
        self.state.disks.read().await.values().cloned().collect()
    }
}

pub fn parse_cpu_stat(line: &str) -> Result<models::CPUStats> {
    let cpu_stats = line
        .split_ascii_whitespace()
        .skip(1)
        .filter_map(|it| it.parse::<u64>().ok())
        .collect::<Vec<_>>();

    let id = line
        .split_ascii_whitespace()
        .next()
        .map(|it| it.trim_start_matches("cpu"))
        .and_then(|it| it.parse::<u64>().ok());

    anyhow::ensure!(cpu_stats.len() == 10, "invalid elements in /proc/stat line");

    let user = cpu_stats[0];
    let nice = cpu_stats[1];
    let system = cpu_stats[2];
    let idle = cpu_stats[3];
    let iowait = cpu_stats[4];
    let irq = cpu_stats[5];
    let softirq = cpu_stats[6];
    let steal = cpu_stats[7];
    let guest = cpu_stats[8];
    let guest_nice = cpu_stats[9];

    Ok(models::CPUStats {
        id,
        user,
        nice,
        system,
        idle,
        iowait,
        irq,
        softirq,
        steal,
        guest,
        guest_nice,
    })
}

pub fn parse_disk_stat(path: &str, line: &str) -> Result<models::DiskBlockStats> {
    let stats = line
        .split_ascii_whitespace()
        .filter_map(|it| it.parse::<u64>().ok())
        .collect::<Vec<_>>();
    anyhow::ensure!(stats.len() >= 11, "invalid disk stats line");

    Ok(models::DiskBlockStats {
        path: path.to_owned(),
        reads: stats[0],
        reads_merged: stats[1],
        read_bytes: stats[2] * 512,
        read_time: stats[3],
        writes: stats[4],
        writes_merged: stats[5],
        write_bytes: stats[6] * 512,
        write_time: stats[7],
        io_in_progress: stats[8],
        io_time: stats[9],
        weighted_io_time: stats[10],
    })
}
