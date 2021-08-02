use crate::sysinfo::{
    models::{CPUStats, DiskBlockStats, MemoryStats},
    SysInfo,
};
use async_graphql::*;

#[derive(SimpleObject, Debug)]
pub struct ProcessStats {
    pub running: u64,
    pub blocked: u64,
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

        Ok(s.mem_stat().await)
    }

    async fn disks(&self, cx: &Context<'_>) -> Result<Vec<DiskBlockStats>> {
        let s = cx.data::<SysInfo>()?;
        Ok(s.disks().await)
    }
}

/// CPU node
struct CPUNode;

#[Object]
impl CPUNode {
    async fn total(&self, cx: &Context<'_>) -> Result<CPUStats> {
        let s = cx.data::<SysInfo>()?;
        let stats = s.cpu_stat().await;

        Ok(stats)
    }

    async fn detail(&self, cx: &Context<'_>, id: u64) -> Result<CPUStats> {
        let s = cx.data::<SysInfo>()?;

        let stats = s.cpu_stat_pre(id).await;
        Ok(stats)
    }

    async fn details(&self, cx: &Context<'_>) -> Result<Vec<CPUStats>> {
        let s = cx.data::<SysInfo>()?;

        Ok(s.cpu_stats().await)
    }
}
