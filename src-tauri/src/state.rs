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
use tauri::AppHandle;

pub struct AppState {
    pub data_dir: PathBuf,
    pub db: Db,
    pub embedder: Arc<Embedder>,
    pub indices: RwLock<HashMap<String, Arc<ProjectIndex>>>,
    pub default_project_id: String,
    pub app: Option<AppHandle>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            data_dir: self.data_dir.clone(),
            db: self.db.clone(),
            embedder: self.embedder.clone(),
            indices: RwLock::new(self.indices.read().clone()),
            default_project_id: self.default_project_id.clone(),
            app: self.app.clone(),
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
            indices: RwLock::new(HashMap::new()),
            default_project_id: default_id,
            app: None,
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

        let mut indices = self.indices.write();
        for (pid, dim, bw) in rows {
            indices.entry(pid.clone()).or_insert_with(|| {
                Arc::new(
                    ProjectIndex::open_or_create(
                        &pid,
                        dim,
                        bw as usize,
                        &self.data_dir.join("indices"),
                    )
                    .expect("open project index"),
                )
            });
        }
        Ok(())
    }

    pub fn get_or_load_index(&self, project_id: &str) -> BiResult<Arc<ProjectIndex>> {
        if let Some(idx) = self.indices.read().get(project_id).cloned() {
            return Ok(idx);
        }
        self.refresh_indices()?;
        self.indices
            .read()
            .get(project_id)
            .cloned()
            .ok_or_else(|| BiError::NotFound(format!("project {project_id}")))
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
