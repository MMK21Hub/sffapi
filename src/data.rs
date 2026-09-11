use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::structs::{Bytes, MiniPCStats};

const TTL: Duration = Duration::from_secs(1);

struct Entry {
    fetched_at: Instant,
    stats: MiniPCStats,
}

#[derive(Default)]
pub struct StatsCache {
    entries: Mutex<HashMap<i64, Entry>>,
}

/// Returns cached stats if they're younger than TTL, otherwise fetches fresh.
pub async fn get_stats(cache: &StatsCache, id: i64) -> anyhow::Result<MiniPCStats> {
    // Fast path: only hold the lock while checking + cloning, never across `.await`.
    {
        let entries = cache.entries.lock().unwrap();
        if let Some(entry) = entries.get(&id) {
            if entry.fetched_at.elapsed() < TTL {
                return Ok(entry.stats.clone());
            }
        }
    } // guard dropped here, before the await -> future stays Send

    let stats = fetch_stats(id).await?;

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

/// Placeholder for the real network call to the mini PC.
async fn fetch_stats(id: i64) -> anyhow::Result<MiniPCStats> {
    Ok({
        MiniPCStats {
            cpu: "Intel Core i3".to_string(),
            ram_total: Bytes(16 * 1024 * 1024 * 1024),
            ram_used: Bytes(8 * 1024 * 1024 * 1024),
        }
    })
}
