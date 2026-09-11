use anyhow::Context;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use rusqlite::OptionalExtension;
use std::path::Path;
use std::time::Duration;

pub type Pool = r2d2::Pool<SqliteConnectionManager>;

#[derive(Clone, Debug)]
pub struct PhysicalMiniPC {
    pub id: i64,
    pub title: String,
    pub homebox_id: Option<i64>,
    pub cpu: Option<String>,
    pub hostname: Option<String>,
    pub deployed: bool,
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS physical_mini_pc (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    homebox_id INTEGER,
    cpu TEXT,
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
        db.prepare("SELECT id, title, homebox_id, cpu, hostname, deployed FROM physical_mini_pc")?;
    let mini_pcs = statement
        .query_map([], |row| {
            Ok(PhysicalMiniPC {
                id: row.get(0)?,
                title: row.get(1)?,
                homebox_id: row.get(2)?,
                cpu: row.get(3)?,
                hostname: row.get(4)?,
                deployed: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(mini_pcs)
}

pub fn get_mini_pc(db: &Connection, id: i64) -> anyhow::Result<Option<PhysicalMiniPC>> {
    // he made a statement
    let mut statement = db.prepare(
        "SELECT id, title, homebox_id, cpu, hostname, deployed FROM physical_mini_pc WHERE id = ?",
    )?;
    let mini_pc = statement
        .query_row([id], |row| {
            Ok(PhysicalMiniPC {
                id: row.get(0)?,
                title: row.get(1)?,
                homebox_id: row.get(2)?,
                cpu: row.get(3)?,
                hostname: row.get(4)?,
                deployed: row.get(5)?,
            })
        })
        .optional()?;

    Ok(mini_pc)
}
