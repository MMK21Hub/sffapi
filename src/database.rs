use anyhow::Context;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use serde::Deserialize;
use serde_rusqlite::from_rows;
use std::path::Path;
use std::time::Duration;

pub type Pool = r2d2::Pool<SqliteConnectionManager>;

#[derive(Clone, Debug, Deserialize)]
pub struct PhysicalMiniPC {
    pub id: i64,
    pub title: String,
    pub homebox_id: Option<i64>,
    pub hostname: Option<String>,
    pub deployed: bool,
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS physical_mini_pc (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    homebox_id INTEGER,
    hostname TEXT,
    deployed BOOLEAN NOT NULL DEFAULT 0
);
"#;

pub fn get_db_pool(path: impl AsRef<Path>) -> anyhow::Result<Pool> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create database directory {}", parent.display())
            })?;
        }
    }

    let manager = SqliteConnectionManager::file(path).with_init(|conn| {
        conn.busy_timeout(Duration::from_secs(5))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.execute_batch(SCHEMA)
    });

    r2d2::Pool::builder()
        .build(manager)
        .with_context(|| format!("Failed to open database {}", path.display()))
}

pub fn all_mini_pcs(db: &Connection) -> anyhow::Result<Vec<PhysicalMiniPC>> {
    let mut statement =
        db.prepare("SELECT id, title, homebox_id, hostname, deployed FROM physical_mini_pc")?;
    let mini_pcs = from_rows::<PhysicalMiniPC>(statement.query([])?)
        .collect::<serde_rusqlite::Result<Vec<_>>>()?;

    Ok(mini_pcs)
}

pub fn get_mini_pc(db: &Connection, id: i64) -> anyhow::Result<Option<PhysicalMiniPC>> {
    // he made a statement
    let mut statement = db.prepare(
        "SELECT id, title, homebox_id, hostname, deployed FROM physical_mini_pc WHERE id = ?",
    )?;
    let mut rows = from_rows::<PhysicalMiniPC>(statement.query([id])?);

    match rows.next() {
        Some(mini_pc) => Ok(Some(mini_pc?)),
        None => Ok(None),
    }
}
