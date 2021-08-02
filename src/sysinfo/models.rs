use async_graphql::SimpleObject;

#[derive(Debug, Copy, Clone, SimpleObject, Default)]
pub struct CPUStats {
    pub id: Option<u64>,
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

#[derive(Debug, Clone, SimpleObject, Default)]
pub struct DiskBlockStats {
    pub path: String,
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

#[derive(Debug, Copy, Clone, SimpleObject, Default)]
pub struct MemoryStats {
    pub total: u64,
    pub free: u64,
    pub available: u64,
    pub buffer: u64,
    pub cached: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}
