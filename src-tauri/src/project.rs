//! Multi-project isolation. Each project gets its own turbovec index and a row in
//! the projects table. The "default" project is auto-created on first run.

use crate::db::log_activity;
use crate::error::{BiError, BiResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub root_path: Option<String>,
    pub bit_width: i64,
    pub dim: i64,
    pub memory_count: i64,
    pub indexed_count: i64,
    pub embed_model: Option<String>,
    pub watch_enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn list(state: &AppState) -> BiResult<Vec<Project>> {
    let conn = state.db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, description, root_path, bit_width, dim, memory_count,
                indexed_count, embed_model, watch_enabled, created_at, updated_at
         FROM projects ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], row_to_project)?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get(state: &AppState, id: &str) -> BiResult<Project> {
    let conn = state.db.conn()?;
    conn.query_row(
        "SELECT id, name, description, root_path, bit_width, dim, memory_count,
                indexed_count, embed_model, watch_enabled, created_at, updated_at
         FROM projects WHERE id = ?1",
        rusqlite::params![id],
        row_to_project,
    )
    .map_err(|_| BiError::NotFound(format!("project {id}")))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub root_path: Option<String>,
    pub bit_width: Option<u8>,
}

pub fn create(state: &AppState, input: CreateProjectInput) -> BiResult<Project> {
    let id = input.id.unwrap_or_else(|| slugify(&input.name));
    let bit_width = input.bit_width.unwrap_or(4) as i64;
    let dim = state.embedder.dim as i64;
    let now = chrono::Utc::now().timestamp_millis();

    state.db.write(|tx| {
        tx.execute(
            "INSERT INTO projects(id, name, description, root_path, bit_width, dim, created_at, updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?7)",
            rusqlite::params![
                id,
                input.name,
                input.description,
                input.root_path,
                bit_width,
                dim,
                now
            ],
        )?;
        log_activity(tx, Some(&id), None, "create_project", None, None)?;
        Ok(())
    })?;

    state.refresh_indices()?;
    get(state, &id)
}

pub fn delete(state: &AppState, id: &str) -> BiResult<()> {
    if id == state.default_project_id {
        return Err(BiError::Invalid("cannot delete default project".into()));
    }
    // Drop index file.
    let file = state.data_dir.join("indices").join(format!("{id}.tvim"));
    let _ = std::fs::remove_file(&file);
    let meta = file.with_extension("uidmap.json");
    let _ = std::fs::remove_file(&meta);
    state.db.write(|tx| {
        tx.execute(
            "DELETE FROM memories WHERE project_id = ?1",
            rusqlite::params![id],
        )?;
        tx.execute("DELETE FROM projects WHERE id = ?1", rusqlite::params![id])?;
        log_activity(tx, Some(&id), None, "delete_project", None, None)?;
        Ok(())
    })?;
    state.indices.write().remove(id);
    Ok(())
}

fn row_to_project(r: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: r.get(0)?,
        name: r.get(1)?,
        description: r.get(2)?,
        root_path: r.get(3)?,
        bit_width: r.get(4)?,
        dim: r.get(5)?,
        memory_count: r.get(6)?,
        indexed_count: r.get(7)?,
        embed_model: r.get(8)?,
        watch_enabled: r.get::<_, i64>(9)? != 0,
        created_at: r.get(10)?,
        updated_at: r.get(11)?,
    })
}

fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
