use crate::graphql::{CPUInfo, CPUStats, DiskInfo, DiskStats, MemoryStats};
use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::fs;

pub struct SysInfo {
    disks: Vec<String>,
}

impl SysInfo {
    pub fn new() -> SysInfo {
        SysInfo {
            disks: vec!["sda".to_string()],
        }
    }

    pub async fn cpu_stat(&self) -> Result<CPUStats> {
        let stat = fs::read_to_string("/proc/stat").await?;
        let stat_lines: Vec<_> = stat.lines().take(1).collect();

        let cpu_line = stat_lines.first().context("unable retrieve cpu info")?;
        parse_cpu_stat(cpu_line)
    }

    pub async fn cpu_stats(&self) -> Result<Vec<CPUInfo>> {
        let stat = fs::read_to_string("/proc/stat").await?;
        let stat_lines = stat
            .lines()
            .skip(1)
            .filter(|it| it.starts_with("cpu"))
            .map(|it| &it[3..])
            .collect::<Vec<_>>();

        let mut info = Vec::with_capacity(stat_lines.len());
        for line in stat_lines {
            let (id, _) = line.split_once(" ").context("failed parse stat line")?;
            let id = id.parse::<u64>()?;
            let stats = parse_cpu_stat(line)?;
            info.push(CPUInfo {
                id: Some(id),
                stats,
            })
        }

        Ok(info)
    }

    pub async fn cpu_stat_pre(&self, id: u64) -> Result<CPUStats> {
        let stat = fs::read_to_string("/proc/stat").await?;
        let stat_line = stat
            .lines()
            .find(|it| it.starts_with(&format!("cpu{}", id)))
            .with_context(|| format!("invalid cpu id {}", id))?;

        parse_cpu_stat(stat_line)
    }

    pub async fn mem_stat(&self) -> Result<MemoryStats> {
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
        Ok(MemoryStats {
            total,
            free,
            available,
            buffer,
            cached,
            swap_total,
            swap_free,
        })
    }

    pub async fn disk_stat(&self, disk: &str) -> Result<DiskStats> {
        let path = format!("/sys/block/{}/stat", disk);
        let line = fs::read_to_string(path).await?;

        parse_disk_stat(&line)
    }

    pub async fn disks(&self) -> Result<Vec<DiskInfo>> {
        let mut info = Vec::with_capacity(self.disks.len());

        for path in &self.disks {
            let stats = self.disk_stat(path).await?;
            info.push(DiskInfo {
                path: path.clone(),
                stats,
            });
        }

        Ok(info)
    }
}

pub fn parse_cpu_stat(line: &str) -> Result<CPUStats> {
    let cpu_stats = line
        .split_ascii_whitespace()
        .skip(1)
        .filter_map(|it| it.parse::<u64>().ok())
        .collect::<Vec<_>>();

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

    Ok(CPUStats {
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

pub fn parse_disk_stat(line: &str) -> Result<DiskStats> {
    let stats = line
        .split_ascii_whitespace()
        .filter_map(|it| it.parse::<u64>().ok())
        .collect::<Vec<_>>();
    anyhow::ensure!(stats.len() >= 11, "invalid disk stats line");

    Ok(DiskStats {
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
