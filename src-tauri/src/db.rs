//! SQLite schema + connection pool. Holds all metadata; turbovec holds vectors.
//!
//! Tables
//! ──────
//! projects  — multi-project isolation (one index per project + a global one)
//! memories  — memory entries with type, importance, tags, access stats
//! agents    — registered AI agents (Mavis, Cursor, Claude Code, Cline…)
//! activity  — append-only audit log of writes/reads
//! code_index — tree-sitter chunks per project (file:range → memory)

use crate::error::{BiError, BiResult};
use parking_lot::Mutex;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension};
use std::path::Path;
use std::sync::Arc;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn open_pool(db_path: &Path) -> BiResult<DbPool> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let manager = SqliteConnectionManager::file(db_path).with_init(|c| {
        c.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;
             PRAGMA temp_store=MEMORY;
             PRAGMA busy_timeout=5000;",
        )
    });
    let pool = Pool::builder().max_size(8).build(manager)?;
    let conn = pool.get()?;
    init_schema(&conn)?;
    Ok(pool)
}

pub fn init_schema(conn: &rusqlite::Connection) -> BiResult<()> {
    conn.execute_batch(SCHEMA)?;
    Ok(())
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS projects (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL UNIQUE,
    description   TEXT,
    root_path     TEXT,
    bit_width     INTEGER NOT NULL DEFAULT 4,
    dim           INTEGER NOT NULL DEFAULT 384,
    memory_count  INTEGER NOT NULL DEFAULT 0,
    indexed_count INTEGER NOT NULL DEFAULT 0,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS agents (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    kind        TEXT NOT NULL,
    last_seen   INTEGER NOT NULL,
    created_at  INTEGER NOT NULL,
    meta        TEXT
);

CREATE TABLE IF NOT EXISTS memories (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    uid          TEXT UNIQUE NOT NULL,
    project_id   TEXT NOT NULL,
    mem_type     TEXT NOT NULL,        -- fact | decision | preference | pattern | episode | reflection | code
    content      TEXT NOT NULL,
    tags         TEXT,                 -- JSON array
    source_agent TEXT,
    importance   REAL NOT NULL DEFAULT 0.5,
    supersedes   INTEGER,              -- memory id this one replaces
    superseded_by INTEGER,
    created_at   INTEGER NOT NULL,
    updated_at   INTEGER NOT NULL,
    last_access  INTEGER NOT NULL,
    access_count INTEGER NOT NULL DEFAULT 0,
    file_path    TEXT,                 -- for code_index entries
    start_line   INTEGER,
    end_line     INTEGER,
    language     TEXT,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_memories_project ON memories(project_id);
CREATE INDEX IF NOT EXISTS idx_memories_type    ON memories(mem_type);
CREATE INDEX IF NOT EXISTS idx_memories_imp     ON memories(importance DESC);
CREATE INDEX IF NOT EXISTS idx_memories_time    ON memories(created_at DESC);

CREATE TABLE IF NOT EXISTS activity (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  TEXT,
    agent_id    TEXT,
    action      TEXT NOT NULL,         -- write | read | forget | update | search | consolidate | ingest
    memory_uid  TEXT,
    detail      TEXT,                  -- JSON
    created_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_activity_time ON activity(created_at DESC);

CREATE TABLE IF NOT EXISTS code_edges (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  TEXT NOT NULL,
    from_uid    TEXT NOT NULL,
    to_uid      TEXT NOT NULL,
    edge_type   TEXT NOT NULL,
    weight      REAL NOT NULL DEFAULT 1.0,
    created_at  INTEGER NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_code_edges_project ON code_edges(project_id);
CREATE INDEX IF NOT EXISTS idx_code_edges_from ON code_edges(from_uid);
CREATE INDEX IF NOT EXISTS idx_code_edges_to ON code_edges(to_uid);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
"#;

pub struct Db {
    pub pool: Arc<DbPool>,
    write_lock: Mutex<()>,
}

impl Db {
    pub fn open(db_path: &Path) -> BiResult<Self> {
        let pool = open_pool(db_path)?;
        Ok(Self {
            pool: Arc::new(pool),
            write_lock: Mutex::new(()),
        })
    }

    pub fn conn(&self) -> BiResult<r2d2::PooledConnection<SqliteConnectionManager>> {
        Ok(self.pool.get()?)
    }

    /// Run a write transaction with an exclusive lock so we never race with the MCP server.
    pub fn write<F, T>(&self, f: F) -> BiResult<T>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> BiResult<T>,
    {
        let _g = self.write_lock.lock();
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        let out = f(&tx)?;
        tx.commit()?;
        Ok(out)
    }
}

pub fn get_setting(conn: &rusqlite::Connection, key: &str) -> BiResult<Option<String>> {
    let v: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |r| r.get(0),
        )
        .optional()?;
    Ok(v)
}

pub fn set_setting(conn: &rusqlite::Connection, key: &str, value: &str) -> BiResult<()> {
    conn.execute(
        "INSERT INTO settings(key, value) VALUES(?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn log_activity(
    conn: &rusqlite::Connection,
    project_id: Option<&str>,
    agent_id: Option<&str>,
    action: &str,
    memory_uid: Option<&str>,
    detail: Option<&serde_json::Value>,
) -> BiResult<()> {
    let detail_str = detail.map(|v| v.to_string());
    conn.execute(
        "INSERT INTO activity(project_id, agent_id, action, memory_uid, detail, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            project_id,
            agent_id,
            action,
            memory_uid,
            detail_str,
            chrono::Utc::now().timestamp_millis(),
        ],
    )?;
    Ok(())
}
