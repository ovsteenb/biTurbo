//! Shared application state. Lives in the Tauri-managed container; cloned into
//! background tasks. The standalone MCP binary builds its own from the same data dir.

use crate::db::Db;
use crate::embed::Embedder;
use crate::error::{BiError, BiResult};
use crate::index_engine::ProjectIndex;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tauri::AppHandle;

pub struct AppState {
    pub data_dir: PathBuf,
    pub db: Db,
    pub embedder: Arc<Embedder>,
    pub indices: Arc<RwLock<HashMap<String, Arc<ProjectIndex>>>>,
    pub default_project_id: String,
    pub app: Option<AppHandle>,
    pub index_size_cache: parking_lot::Mutex<Option<(Instant, u64)>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            data_dir: self.data_dir.clone(),
            db: self.db.clone(),
            embedder: self.embedder.clone(),
            indices: self.indices.clone(),
            default_project_id: self.default_project_id.clone(),
            app: self.app.clone(),
            index_size_cache: parking_lot::Mutex::new(None),
        }
    }
}

impl AppState {
    pub fn open(data_dir: &Path) -> BiResult<Self> {
        std::fs::create_dir_all(data_dir).ok();
        let db_path = data_dir.join("biturbo.db");
        let db = Db::open(&db_path)?;

        let embedder = Arc::new(Embedder::new("BGE-small-en")?);

        // Ensure default project exists.
        let conn = db.conn()?;
        let default_id = "default".to_string();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT OR IGNORE INTO projects(id, name, bit_width, dim, created_at, updated_at)
             VALUES(?1, ?2, 4, ?3, ?4, ?4)",
            rusqlite::params![default_id, "default", embedder.dim as i64, now],
        )?;

        let state = Self {
            data_dir: data_dir.to_path_buf(),
            db,
            embedder,
            indices: Arc::new(RwLock::new(HashMap::new())),
            default_project_id: default_id,
            app: None,
            index_size_cache: parking_lot::Mutex::new(None),
        };

        // Warm indices for existing projects.
        state.refresh_indices()?;

        Ok(state)
    }

    /// (Re-)scan projects table and ensure an in-memory index exists for each.
    pub fn refresh_indices(&self) -> BiResult<()> {
        let conn = self.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, dim, bit_width FROM projects",
        )?;
        let rows: Vec<(String, usize, u8)> = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)? as usize,
                    r.get::<_, i64>(2)? as u8,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);

        // Open any missing index files BEFORE taking the write lock, so the
        // critical section is just map inserts.
        let data_dir = self.data_dir.join("indices");
        let to_open: Vec<(String, usize, u8)> = {
            let existing = self.indices.read();
            rows.into_iter()
                .filter(|(pid, _, _)| !existing.contains_key(pid))
                .collect()
        };
        let mut opened: Vec<(String, Arc<ProjectIndex>)> = Vec::with_capacity(to_open.len());
        for (pid, dim, bw) in to_open {
            let idx = Arc::new(
                ProjectIndex::open_or_create(&pid, dim, bw as usize, &data_dir)
                    .expect("open project index"),
            );
            opened.push((pid, idx));
        }
        if !opened.is_empty() {
            let mut indices = self.indices.write();
            for (pid, idx) in opened {
                indices.entry(pid).or_insert(idx);
            }
        }
        Ok(())
    }

    pub fn get_or_load_index(&self, project_id: &str) -> BiResult<Arc<ProjectIndex>> {
        if let Some(idx) = self.indices.read().get(project_id).cloned() {
            return Ok(idx);
        }
        // Open the one missing file directly without scanning the projects table.
        let conn = self.db.conn()?;
        let row: Option<(i64, i64)> = conn
            .query_row(
                "SELECT dim, bit_width FROM projects WHERE id = ?1",
                rusqlite::params![project_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        let (dim, bw) = match row {
            Some((d, b)) => (d as usize, b as u8 as usize),
            None => return Err(BiError::NotFound(format!("project {project_id}"))),
        };
        let idx = Arc::new(
            ProjectIndex::open_or_create(
                project_id,
                dim,
                bw,
                &self.data_dir.join("indices"),
            )?,
        );
        self.indices.write().insert(project_id.to_string(), idx.clone());
        Ok(idx)
    }

    /// Total bytes on disk for project index files. Cached for 5s.
    pub fn index_bytes(&self) -> u64 {
        if let Some((when, n)) = *self.index_size_cache.lock() {
            if when.elapsed().as_secs() < 5 {
                return n;
            }
        }
        let n: u64 = walkdir::WalkDir::new(self.data_dir.join("indices"))
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.metadata().ok())
            .filter(|m| m.is_file())
            .map(|m| m.len())
            .sum();
        *self.index_size_cache.lock() = Some((Instant::now(), n));
        n
    }

    /// Embed text and add to a project's index. Returns the vector length.
    pub fn embed_and_add(
        &self,
        project_id: &str,
        uid: &str,
        text: &str,
    ) -> BiResult<usize> {
        let vec = self.embedder.embed(text)?;
        let idx = self.get_or_load_index(project_id)?;
        idx.add(uid, &vec)?;
        Ok(vec.len())
    }

    /// Flush every dirty project index to disk. Cheap no-op if nothing changed.
    pub fn flush_all_indices(&self) {
        let indices = self.indices.read();
        for idx in indices.values() {
            let _ = idx.maybe_flush(std::time::Duration::from_millis(500), false);
        }
    }

    pub fn embed_and_search(
        &self,
        project_id: &str,
        query: &str,
        k: usize,
        allowlist: Option<&[String]>,
    ) -> BiResult<Vec<crate::index_engine::SearchHit>> {
        let vec = self.embedder.embed(query)?;
        let idx = self.get_or_load_index(project_id)?;
        idx.search(&vec, k, allowlist)
    }
}
