//! Persisted supervision for long-running work.

use crate::error::{BiError, BiResult};
use crate::state::AppState;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub kind: String,
    pub project_id: Option<String>,
    pub status: String,
    pub phase: Option<String>,
    pub current: i64,
    pub total: i64,
    pub checkpoint: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub cancel_requested: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

pub fn create(
    state: &AppState,
    kind: &str,
    project_id: Option<&str>,
    checkpoint: Option<&serde_json::Value>,
) -> BiResult<Operation> {
    let prefix = match kind {
        "ingest" | "watch_ingest" => "ing",
        "multi_ingest" => "multi-ing",
        "consolidate" => "con",
        "model_rebuild" => "model",
        _ => "op",
    };
    let id = format!("{prefix}-{}", uuid::Uuid::new_v4());
    let now = chrono::Utc::now().timestamp_millis();
    let checkpoint = checkpoint.map(serde_json::Value::to_string);
    state.db.write(|tx| {
        tx.execute(
            "INSERT INTO operations(id, kind, project_id, status, phase, checkpoint, created_at, updated_at)
             VALUES(?1, ?2, ?3, 'queued', 'queued', ?4, ?5, ?5)",
            rusqlite::params![id, kind, project_id, checkpoint, now],
        )?;
        Ok(())
    })?;
    get(state, &id)
}

pub fn get(state: &AppState, id: &str) -> BiResult<Operation> {
    let conn = state.db.conn()?;
    conn.query_row(
        "SELECT id, kind, project_id, status, phase, current, total, checkpoint,
                result, error, cancel_requested, created_at, updated_at, started_at, finished_at
         FROM operations WHERE id = ?1",
        rusqlite::params![id],
        row_to_operation,
    )
    .optional()?
    .ok_or_else(|| BiError::NotFound(format!("operation {id}")))
}

pub fn list(state: &AppState, limit: usize) -> BiResult<Vec<Operation>> {
    let conn = state.db.conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, kind, project_id, status, phase, current, total, checkpoint,
                result, error, cancel_requested, created_at, updated_at, started_at, finished_at
         FROM operations ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(rusqlite::params![limit.clamp(1, 500) as i64], row_to_operation)?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn mark_running(state: &AppState, id: &str) -> BiResult<()> {
    let now = chrono::Utc::now().timestamp_millis();
    update_status(
        state,
        id,
        "running",
        Some("starting"),
        None,
        None,
        Some(now),
    )
}

pub fn update_progress(
    state: &AppState,
    id: &str,
    phase: &str,
    current: usize,
    total: usize,
    checkpoint: Option<&serde_json::Value>,
) -> BiResult<()> {
    let now = chrono::Utc::now().timestamp_millis();
    let checkpoint = checkpoint.map(serde_json::Value::to_string);
    state.db.write(|tx| {
        tx.execute(
            "UPDATE operations SET phase = ?1, current = ?2, total = ?3,
                 checkpoint = COALESCE(?4, checkpoint), updated_at = ?5 WHERE id = ?6",
            rusqlite::params![phase, current as i64, total as i64, checkpoint, now, id],
        )?;
        Ok(())
    })?;
    if let (Some(app), Ok(operation)) = (&state.app, get(state, id)) {
        let _ = app.emit("operation:changed", operation);
    }
    Ok(())
}

pub fn complete(state: &AppState, id: &str, result: &serde_json::Value) -> BiResult<()> {
    update_status(
        state,
        id,
        "succeeded",
        Some("done"),
        Some(result),
        None,
        None,
    )
}

pub fn fail(state: &AppState, id: &str, error: &str) -> BiResult<()> {
    update_status(state, id, "failed", Some("failed"), None, Some(error), None)
}

pub fn mark_cancelled(state: &AppState, id: &str) -> BiResult<()> {
    update_status(
        state,
        id,
        "cancelled",
        Some("cancelled"),
        None,
        None,
        None,
    )
}

pub fn request_cancel(state: &AppState, id: &str) -> BiResult<Operation> {
    let now = chrono::Utc::now().timestamp_millis();
    state.db.write(|tx| {
        let changed = tx.execute(
            "UPDATE operations SET cancel_requested = 1, updated_at = ?1
             WHERE id = ?2 AND status IN ('queued', 'running')",
            rusqlite::params![now, id],
        )?;
        if changed == 0 {
            return Err(BiError::Invalid(format!(
                "operation {id} is not cancellable"
            )));
        }
        Ok(())
    })?;
    get(state, id)
}

pub fn is_cancel_requested(state: &AppState, id: &str) -> BiResult<bool> {
    let conn = state.db.conn()?;
    Ok(conn.query_row(
        "SELECT cancel_requested FROM operations WHERE id = ?1",
        rusqlite::params![id],
        |r| r.get::<_, i64>(0),
    )? != 0)
}

pub fn recover_interrupted(state: &AppState) -> BiResult<usize> {
    let now = chrono::Utc::now().timestamp_millis();
    state.db.write(|tx| {
        Ok(tx.execute(
            "UPDATE operations SET status = 'queued', phase = 'recovered',
                 cancel_requested = 0, updated_at = ?1, started_at = NULL
             WHERE status = 'running'",
            rusqlite::params![now],
        )?)
    })
}

pub fn start_ingest(state: &AppState, project_id: &str, root: &Path) -> BiResult<Operation> {
    crate::project::get(state, project_id)?;
    if !root.is_dir() {
        return Err(BiError::Invalid(format!(
            "root_path '{}' does not exist on disk",
            root.display()
        )));
    }
    let checkpoint = serde_json::json!({"root_path": root.to_string_lossy()});
    let operation = create(state, "ingest", Some(project_id), Some(&checkpoint))?;
    spawn_ingest(
        Arc::new(state.clone()),
        operation.id.clone(),
        project_id.to_string(),
        root.to_path_buf(),
    );
    Ok(operation)
}

pub fn run_ingest_blocking(
    state: &AppState,
    project_id: &str,
    root: &Path,
) -> BiResult<crate::ingest::IngestResult> {
    run_ingest_blocking_with_kind(state, project_id, root, "ingest")
}

pub fn run_watch_ingest_blocking(
    state: &AppState,
    project_id: &str,
    root: &Path,
) -> BiResult<crate::ingest::IngestResult> {
    run_ingest_blocking_with_kind(state, project_id, root, "watch_ingest")
}

fn run_ingest_blocking_with_kind(
    state: &AppState,
    project_id: &str,
    root: &Path,
    kind: &str,
) -> BiResult<crate::ingest::IngestResult> {
    crate::project::get(state, project_id)?;
    let checkpoint = serde_json::json!({"root_path": root.to_string_lossy()});
    let operation = create(state, kind, Some(project_id), Some(&checkpoint))?;
    execute_ingest(state, &operation.id, project_id, root)
}

pub fn start_multi_ingest(
    state: &AppState,
    projects: Vec<(String, PathBuf)>,
) -> BiResult<Operation> {
    for (project_id, root) in &projects {
        crate::project::get(state, project_id)?;
        if !root.is_dir() {
            return Err(BiError::Invalid(format!(
                "root_path '{}' does not exist on disk",
                root.display()
            )));
        }
    }
    let checkpoint = serde_json::json!({
        "projects": projects.iter().map(|(project_id, root)| {
            serde_json::json!({"project_id": project_id, "root_path": root.to_string_lossy()})
        }).collect::<Vec<_>>()
    });
    let operation = create(state, "multi_ingest", None, Some(&checkpoint))?;
    let state = Arc::new(state.clone());
    let id = operation.id.clone();
    std::thread::Builder::new()
        .name(format!("biturbo-operation-{id}"))
        .spawn(move || {
            let _ = execute_multi_ingest(&state, &id, projects);
        })
        .ok();
    Ok(operation)
}

pub fn start_consolidate(state: &AppState, project_id: Option<&str>) -> BiResult<Operation> {
    if let Some(project_id) = project_id {
        crate::project::get(state, project_id)?;
    }
    let operation = create(state, "consolidate", project_id, None)?;
    let state = Arc::new(state.clone());
    let id = operation.id.clone();
    let project_id = project_id.map(String::from);
    std::thread::Builder::new()
        .name(format!("biturbo-operation-{id}"))
        .spawn(move || {
            let _ = execute_consolidate(&state, &id, project_id.as_deref());
        })
        .ok();
    Ok(operation)
}

pub fn resume_pending(state: Arc<AppState>) -> BiResult<usize> {
    recover_interrupted(&state)?;
    let pending = list(&state, 500)?;
    let mut resumed = 0;
    for operation in pending
        .into_iter()
        .filter(|op| {
            op.status == "queued" && matches!(op.kind.as_str(), "ingest" | "watch_ingest")
        })
    {
        let Some(project_id) = operation.project_id.clone() else {
            fail(&state, &operation.id, "queued ingest has no project_id")?;
            continue;
        };
        let root = operation
            .checkpoint
            .as_ref()
            .and_then(|v| v.get("root_path"))
            .and_then(|v| v.as_str())
            .map(PathBuf::from);
        let Some(root) = root else {
            fail(&state, &operation.id, "queued ingest has no root_path checkpoint")?;
            continue;
        };
        spawn_ingest(state.clone(), operation.id, project_id, root);
        resumed += 1;
    }
    Ok(resumed)
}

fn execute_multi_ingest(
    state: &AppState,
    id: &str,
    projects: Vec<(String, PathBuf)>,
) -> BiResult<crate::ingest::MultiIngestResult> {
    mark_running(state, id)?;
    let mut combined = crate::ingest::MultiIngestResult::default();
    let total = projects.len();
    for (position, (project_id, root)) in projects.into_iter().enumerate() {
        if is_cancel_requested(state, id)? {
            mark_cancelled(state, id)?;
            return Err(BiError::Ingest("operation cancelled".into()));
        }
        update_progress(state, id, "ingesting", position, total, None)?;
        match crate::ingest::ingest_project_controlled(state, &project_id, &root, Some(id)) {
            Ok(result) => {
                combined.total_files_indexed += result.files_indexed;
                combined.total_chunks_indexed += result.chunks_indexed;
                combined.total_bytes_processed += result.bytes_processed;
                combined.total_errors += result.errors.len();
                combined.total_edges_created += result.edges_created;
                combined.results.push(result);
            }
            Err(error) => {
                if is_cancel_requested(state, id).unwrap_or(false) {
                    mark_cancelled(state, id)?;
                } else {
                    fail(state, id, &error.to_string())?;
                }
                return Err(error);
            }
        }
    }
    complete(state, id, &serde_json::to_value(&combined)?)?;
    if let Some(app) = &state.app {
        let _ = app.emit(
            "multi-ingest:done",
            serde_json::json!({
                "job_id": id,
                "total_files_indexed": combined.total_files_indexed,
                "total_chunks_indexed": combined.total_chunks_indexed,
                "total_edges_created": combined.total_edges_created,
                "elapsed_ms": 0,
                "results": combined.results,
            }),
        );
    }
    Ok(combined)
}

fn execute_consolidate(
    state: &AppState,
    id: &str,
    project_id: Option<&str>,
) -> BiResult<crate::consolidate::ConsolidateReport> {
    mark_running(state, id)?;
    update_progress(state, id, "consolidating", 0, 1, None)?;
    if is_cancel_requested(state, id)? {
        mark_cancelled(state, id)?;
        return Err(BiError::Invalid("operation cancelled".into()));
    }
    match crate::consolidate::consolidate(state, project_id) {
        Ok(report) => {
            if is_cancel_requested(state, id)? {
                mark_cancelled(state, id)?;
                return Err(BiError::Invalid("operation cancelled".into()));
            }
            complete(state, id, &serde_json::to_value(&report)?)?;
            if let Some(app) = &state.app {
                let _ = app.emit("consolidate:done", &report);
            }
            Ok(report)
        }
        Err(error) => {
            fail(state, id, &error.to_string())?;
            Err(error)
        }
    }
}

fn spawn_ingest(state: Arc<AppState>, id: String, project_id: String, root: PathBuf) {
    std::thread::Builder::new()
        .name(format!("biturbo-operation-{id}"))
        .spawn(move || {
            let _ = execute_ingest(&state, &id, &project_id, &root);
        })
        .ok();
}

fn execute_ingest(
    state: &AppState,
    id: &str,
    project_id: &str,
    root: &Path,
) -> BiResult<crate::ingest::IngestResult> {
    mark_running(state, id)?;
    let started = std::time::Instant::now();
    let outcome = crate::ingest::ingest_project_controlled(state, project_id, root, Some(id));
    match outcome {
        Ok(result) => {
            complete(state, id, &serde_json::to_value(&result)?)?;
            if let Some(app) = &state.app {
                let _ = app.emit(
                    "operation:changed",
                    get(state, id).unwrap_or_else(|_| Operation {
                        id: id.to_string(),
                        kind: "ingest".into(),
                        project_id: Some(project_id.into()),
                        status: "succeeded".into(),
                        phase: Some("done".into()),
                        current: 0,
                        total: 0,
                        checkpoint: None,
                        result: None,
                        error: None,
                        cancel_requested: false,
                        created_at: 0,
                        updated_at: 0,
                        started_at: None,
                        finished_at: None,
                    }),
                );
                let _ = app.emit(
                    "ingest:done",
                    serde_json::json!({
                        "job_id": id,
                        "project_id": project_id,
                        "files_indexed": result.files_indexed,
                        "chunks_indexed": result.chunks_indexed,
                        "edges_created": result.edges_created,
                        "elapsed_ms": started.elapsed().as_millis() as u64,
                    }),
                );
            }
            Ok(result)
        }
        Err(error) => {
            let cancelled = is_cancel_requested(state, id).unwrap_or(false);
            if cancelled {
                mark_cancelled(state, id)?;
            } else {
                fail(state, id, &error.to_string())?;
            }
            if let Some(app) = &state.app {
                if let Ok(operation) = get(state, id) {
                    let _ = app.emit("operation:changed", operation);
                }
                let _ = app.emit(
                    "ingest:error",
                    serde_json::json!({
                        "job_id": id,
                        "project_id": project_id,
                        "error": error.to_string(),
                    }),
                );
            }
            Err(error)
        }
    }
}

fn update_status(
    state: &AppState,
    id: &str,
    status: &str,
    phase: Option<&str>,
    result: Option<&serde_json::Value>,
    error: Option<&str>,
    started_at: Option<i64>,
) -> BiResult<()> {
    let now = chrono::Utc::now().timestamp_millis();
    let result = result.map(serde_json::Value::to_string);
    let terminal = matches!(status, "succeeded" | "failed" | "cancelled");
    state.db.write(|tx| {
        let changed = tx.execute(
            "UPDATE operations SET status = ?1, phase = COALESCE(?2, phase), result = ?3,
                 error = ?4, updated_at = ?5,
                 started_at = COALESCE(?6, started_at),
                 finished_at = CASE WHEN ?7 THEN ?5 ELSE finished_at END
             WHERE id = ?8",
            rusqlite::params![
                status,
                phase,
                result,
                error,
                now,
                started_at,
                terminal,
                id
            ],
        )?;
        if changed == 0 {
            return Err(BiError::NotFound(format!("operation {id}")));
        }
        Ok(())
    })
}

fn row_to_operation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Operation> {
    let checkpoint: Option<String> = row.get(7)?;
    let result: Option<String> = row.get(8)?;
    Ok(Operation {
        id: row.get(0)?,
        kind: row.get(1)?,
        project_id: row.get(2)?,
        status: row.get(3)?,
        phase: row.get(4)?,
        current: row.get(5)?,
        total: row.get(6)?,
        checkpoint: checkpoint.and_then(|v| serde_json::from_str(&v).ok()),
        result: result.and_then(|v| serde_json::from_str(&v).ok()),
        error: row.get(9)?,
        cancel_requested: row.get::<_, i64>(10)? != 0,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
        started_at: row.get(13)?,
        finished_at: row.get(14)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::state::AppState;

    fn state() -> (AppState, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "biturbo-operation-test-{}",
            uuid::Uuid::new_v4()
        ));
        (AppState::open(&dir).unwrap(), dir)
    }

    #[test]
    fn operation_lifecycle_and_restart_recovery_are_persisted() {
        let (state, dir) = state();
        let operation = super::create(
            &state,
            "ingest",
            Some(&state.default_project_id),
            Some(&serde_json::json!({"root_path": "/tmp/project"})),
        )
        .unwrap();
        super::mark_running(&state, &operation.id).unwrap();
        super::request_cancel(&state, &operation.id).unwrap();

        let stored = super::get(&state, &operation.id).unwrap();
        assert_eq!(stored.status, "running");
        assert!(stored.cancel_requested);

        super::recover_interrupted(&state).unwrap();
        let recovered = super::get(&state, &operation.id).unwrap();
        assert_eq!(recovered.status, "queued");
        assert!(!recovered.cancel_requested);

        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn cancelled_ingest_stops_before_mutating_project() {
        let (state, dir) = state();
        let root = dir.join("project");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("lib.rs"), "pub fn should_not_be_indexed() {}").unwrap();
        let operation = super::create(
            &state,
            "ingest",
            Some(&state.default_project_id),
            Some(&serde_json::json!({"root_path": root})),
        )
        .unwrap();
        super::request_cancel(&state, &operation.id).unwrap();

        assert!(super::execute_ingest(
            &state,
            &operation.id,
            &state.default_project_id,
            &root,
        )
        .is_err());
        let stored = super::get(&state, &operation.id).unwrap();
        assert_eq!(stored.status, "cancelled");
        let project = crate::project::get(&state, &state.default_project_id).unwrap();
        assert_eq!(project.indexed_count, 0);

        std::fs::remove_dir_all(dir).ok();
    }
}
