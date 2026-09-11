use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use anyhow::Context;

use crate::{
    database::{self, Pool},
    netdata,
    structs::MiniPCStats,
};

const TTL: Duration = Duration::from_secs(1);

struct Entry {
    fetched_at: Instant,
    stats: MiniPCStats,
}

pub struct StatsCache {
    entries: Mutex<HashMap<i64, Entry>>,
    client: netdata::Client,
}

impl Default for StatsCache {
    fn default() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            client: netdata::Client::default(),
        }
    }
}

/// Get (cached) stats for a mini PC.
pub async fn get_stats(cache: &StatsCache, pool: &Pool, id: i64) -> anyhow::Result<MiniPCStats> {
    {
        let entries = cache.entries.lock().unwrap();
        if let Some(entry) = entries.get(&id) {
            if entry.fetched_at.elapsed() < TTL {
                return Ok(entry.stats.clone());
            }
        }
    }

    let stats = fetch_stats(&cache.client, pool, id).await?;

    let mut entries = cache.entries.lock().unwrap();
    entries.insert(
        id,
        Entry {
            fetched_at: Instant::now(),
            stats: stats.clone(),
        },
    );
    Ok(stats)
}

/// Look up the mini PC's hostname, then fetch live stats from its Netdata agent.
async fn fetch_stats(
    client: &netdata::Client,
    pool: &Pool,
    id: i64,
) -> anyhow::Result<MiniPCStats> {
    let connection = pool.get().context("failed to get database connection")?;
    let hostname = database::get_mini_pc(&connection, id)?
        .and_then(|mini_pc| mini_pc.hostname)
        .with_context(|| format!("mini PC {id} has no hostname"))?;

    client.fetch_stats(&hostname).await
}
