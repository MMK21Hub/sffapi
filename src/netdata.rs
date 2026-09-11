use std::{collections::HashMap, time::Duration};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::structs::{Bytes, MiniPCStats};

/// Every mini PC runs a Netdata agent at this domain and port.
const DOMAIN: &str = "ts.slevel.xyz";
const PORT: u16 = 19999;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

/// Shown when Netdata does not report the CPU model.
const CPU_PLACEHOLDER: &str = "Intel Core i3";

pub struct Client {
    client: awc::Client,
}

impl Default for Client {
    fn default() -> Self {
        let client = awc::Client::builder().timeout(REQUEST_TIMEOUT).finish();
        Self { client }
    }
}

impl Client {
    /// Fetch live stats from the mini PC's Netdata agent.
    pub async fn fetch_stats(&self, hostname: &str) -> anyhow::Result<MiniPCStats> {
        let base = format!("http://{hostname}.{DOMAIN}:{PORT}");

        let info: InfoResponse = self.get_json(format!("{base}/api/v3/info")).await?;
        let hw = &info
            .agents
            .first()
            .with_context(|| format!("Netdata info from {base} reported no agents"))?
            .application
            .hw;
        let ram_total = hw
            .get("ram")
            .with_context(|| format!("Netdata info from {base} reported no ram_total"))?
            .parse::<u64>()
            .context("Netdata reported a non-numeric ram_total")?;

        let nodes: NodesResponse = self.get_json(format!("{base}/api/v3/nodes")).await?;
        let cpu = nodes
            .nodes
            .first()
            .and_then(|node| node.labels.get("_system_cpu_model"))
            .cloned()
            .unwrap_or_else(|| CPU_PLACEHOLDER.to_string());

        let ram: RamResponse = self
            .get_json(format!(
                "{base}/api/v3/data?contexts=system.ram&dimensions=used&after=-1&points=1"
            ))
            .await?;
        let dimensions = ram.view.dimensions;
        let index = dimensions
            .ids
            .iter()
            .position(|name| name == "used")
            .context("Netdata system.ram has no 'used' dimension")?;
        let used = dimensions
            .sts
            .get("avg")
            .and_then(|avg| avg.get(index))
            .context("Netdata system.ram returned no values")?;
        let units = dimensions
            .units
            .get(index)
            .context("Netdata system.ram returned no units for 'used'")?;

        Ok(MiniPCStats {
            cpu,
            ram_total: Bytes(ram_total),
            ram_used: Bytes(to_bytes(*used, units)?),
        })
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: String) -> anyhow::Result<T> {
        let mut response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|err| anyhow::anyhow!("failed to fetch {url}: {err}"))?;
        if !response.status().is_success() {
            bail!("Netdata request to {url} returned {}", response.status());
        }
        response
            .json()
            .await
            .map_err(|err| anyhow::anyhow!("failed to parse Netdata response from {url}: {err}"))
    }
}

fn to_bytes(value: f64, units: &str) -> anyhow::Result<u64> {
    let multiplier = match units {
        "bytes" => 1.0,
        "KiB" => 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        other => bail!("unexpected Netdata units for system.ram: {other}"),
    };
    Ok((value * multiplier) as u64)
}

#[derive(Deserialize)]
struct InfoResponse {
    agents: Vec<InfoAgent>,
}

#[derive(Deserialize)]
struct InfoAgent {
    application: InfoApplication,
}

#[derive(Deserialize)]
struct InfoApplication {
    hw: HashMap<String, String>,
}

#[derive(Deserialize)]
struct NodesResponse {
    nodes: Vec<NodeInfo>,
}

#[derive(Deserialize)]
struct NodeInfo {
    #[serde(default)]
    labels: HashMap<String, String>,
}

#[derive(Deserialize)]
struct RamResponse {
    view: RamView,
}

#[derive(Deserialize)]
struct RamView {
    dimensions: RamDimensions,
}

#[derive(Deserialize)]
struct RamDimensions {
    ids: Vec<String>,
    units: Vec<String>,
    sts: HashMap<String, Vec<f64>>,
}
