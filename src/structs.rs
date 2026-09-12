use crate::database::PhysicalMiniPC;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Bytes(pub u64);

#[derive(Clone, Debug)]
pub struct MiniPC {
    pub id: i64,
    pub hostname: String,
    pub physical: PhysicalMiniPC,
    pub stats: Option<MiniPCStats>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OperatingSystem {
    pub kernel: String,
    pub kernel_version: String,
    pub os: String,
    pub os_version: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct MiniPCStats {
    pub cpu: String,
    pub operating_system: OperatingSystem,
    pub ram_total: Bytes,
    pub ram_used: Bytes,
}
