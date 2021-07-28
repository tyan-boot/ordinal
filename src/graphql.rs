use crate::sysinfo::SysInfo;
use async_graphql::*;

#[derive(SimpleObject, Debug)]
pub struct CPUStats {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
    pub guest: u64,
    pub guest_nice: u64,
}

#[derive(SimpleObject, Debug)]
pub struct MemoryStats {
    pub total: u64,
    pub free: u64,
    pub available: u64,
    pub buffer: u64,
    pub cached: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

#[derive(SimpleObject, Debug)]
pub struct ProcessStats {
    pub running: u64,
    pub blocked: u64,
}

#[derive(SimpleObject, Debug)]
pub struct DiskStats {
    pub reads: u64,
    pub reads_merged: u64,
    pub read_bytes: u64,
    pub read_time: u64,
    pub writes: u64,
    pub writes_merged: u64,
    pub write_bytes: u64,
    pub write_time: u64,
    pub io_in_progress: u64,
    pub io_time: u64,
    pub weighted_io_time: u64,
}

pub struct Query;

pub type RootSchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[Object]
impl Query {
    async fn cpu(&self) -> CPUNode {
        CPUNode
    }

    async fn memory(&self, cx: &Context<'_>) -> Result<MemoryStats> {
        let s = cx.data::<SysInfo>()?;

        s.mem_stat().await.map_err(|e| Error::new(e.to_string()))
    }

    async fn disks(&self, cx: &Context<'_>) -> Result<Vec<DiskInfo>> {
        let s = cx.data::<SysInfo>()?;
        s.disks().await.map_err(|e| Error::new(e.to_string()))
    }
}

/// Disk
#[derive(SimpleObject, Debug)]
pub struct DiskInfo {
    pub path: String,
    pub stats: DiskStats,
}

/// CPU
#[derive(SimpleObject, Debug)]
pub struct CPUInfo {
    pub id: Option<u64>,
    pub stats: CPUStats,
}

struct CPUNode;

#[Object]
impl CPUNode {
    async fn total(&self, cx: &Context<'_>) -> Result<CPUInfo> {
        let s = cx.data::<SysInfo>()?;
        let stats = s.cpu_stat().await.map_err(|e| Error::new(e.to_string()))?;

        Ok(CPUInfo { id: None, stats })
    }

    async fn detail(&self, cx: &Context<'_>, id: u64) -> Result<CPUInfo> {
        let s = cx.data::<SysInfo>()?;

        let stats = s
            .cpu_stat_pre(id as u64)
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(CPUInfo {
            id: Some(id),
            stats,
        })
    }

    async fn details(&self, cx: &Context<'_>) -> Result<Vec<CPUInfo>> {
        let s = cx.data::<SysInfo>()?;

        s.cpu_stats().await.map_err(|e| Error::new(e.to_string()))
    }
}