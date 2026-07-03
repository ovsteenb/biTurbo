//! Tauri IPC commands. The frontend calls these via `invoke<T>("name", { args })`.

use crate::error::BiResult;
use crate::ingest;
use crate::memory::{self, Memory, MemoryWithScore, RememberInput, UpdateInput};
use crate::project::{self, CreateProjectInput, Project};
use crate::scheduler::ConsolidateStatus;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

#[derive(Serialize)]
pub struct Stats {
    pub total_memories: i64,
    pub total_projects: i64,
    pub total_agents: i64,
    pub by_type: Vec<(String, i64)>,
    pub by_project: Vec<(String, i64)>,
    pub index_bytes: u64,
    pub recent_writes_7d: i64,
    pub recent_reads_7d: i64,
}

#[derive(Serialize)]
pub struct ActivityEntry {
    pub id: i64,
    pub project_id: Option<String>,
    pub agent_id: Option<String>,
    pub action: String,
    pub memory_uid: Option<String>,
    pub detail: Option<serde_json::Value>,
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct AgentEntry {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub last_seen: i64,
    pub created_at: i64,
    pub meta: Option<serde_json::Value>,
}

#[tauri::command]
pub fn ping() -> &'static str {
    "pong"
}

#[tauri::command]
pub fn list_memories(
    state: State<'_, AppState>,
    project_id: Option<String>,
    mem_type: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> BiResult<Vec<Memory>> {
    memory::list(
        state.inner(),
        project_id.as_deref(),
        mem_type.as_deref(),
        limit.unwrap_or(50),
        offset.unwrap_or(0),
    )
}

#[tauri::command]
pub fn list_tags(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> BiResult<Vec<(String, i64)>> {
    memory::list_tags(state.inner(), project_id.as_deref())
}

#[tauri::command]
pub fn get_memory(state: State<'_, AppState>, uid: String) -> BiResult<Option<Memory>> {
    memory::get(state.inner(), &uid)
}

#[tauri::command]
pub fn remember(state: State<'_, AppState>, input: RememberInput) -> BiResult<Memory> {
    memory::remember(state.inner(), input)
}

#[tauri::command]
pub fn forget_memory(state: State<'_, AppState>, uid: String) -> BiResult<bool> {
    memory::forget(state.inner(), &uid)
}

#[tauri::command]
pub fn update_memory(
    state: State<'_, AppState>,
    uid: String,
    input: UpdateInput,
) -> BiResult<Memory> {
    memory::update(state.inner(), &uid, input)
}

#[derive(Deserialize)]
pub struct SearchArgs {
    pub project_id: Option<String>,
    pub query: String,
    pub k: Option<usize>,
    pub mem_type: Option<String>,
}

#[tauri::command]
pub fn search_memories(
    state: State<'_, AppState>,
    args: SearchArgs,
) -> BiResult<Vec<MemoryWithScore>> {
    memory::search(
        state.inner(),
        args.project_id.as_deref().unwrap_or(""),
        &args.query,
        args.k.unwrap_or(10),
        args.mem_type.as_deref(),
    )
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> BiResult<Vec<Project>> {
    project::list(state.inner())
}

#[tauri::command]
pub fn get_project(state: State<'_, AppState>, id: String) -> BiResult<Project> {
    project::get(state.inner(), &id)
}

#[tauri::command]
pub fn create_project(state: State<'_, AppState>, input: CreateProjectInput) -> BiResult<Project> {
    project::create(state.inner(), input)
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, id: String) -> BiResult<()> {
    project::delete(state.inner(), &id)
}

#[tauri::command]
pub fn ensure_project_marker_files(
    state: State<'_, AppState>,
    project_id: String,
) -> BiResult<project::EnsureMarkerFilesResult> {
    project::ensure_marker_files(state.inner(), &project_id)
}

#[derive(Deserialize)]
pub struct IngestArgs {
    pub project_id: String,
    pub root_path: String,
}

#[derive(Deserialize)]
pub struct MultiIngestArgs {
    pub projects: Vec<(String, String)>,
}

#[derive(Serialize)]
pub struct IngestJobResponse {
    pub job_id: String,
    pub project_id: String,
}

#[derive(Serialize, Clone)]
pub struct IngestDone {
    pub job_id: String,
    pub project_id: String,
    pub files_indexed: usize,
    pub chunks_indexed: usize,
    pub edges_created: usize,
    pub elapsed_ms: u64,
}

#[derive(Serialize, Clone)]
pub struct IngestError {
    pub job_id: String,
    pub project_id: String,
    pub error: String,
}

#[derive(Serialize, Clone)]
pub struct MultiIngestDone {
    pub job_id: String,
    pub total_files_indexed: usize,
    pub total_chunks_indexed: usize,
    pub total_edges_created: usize,
    pub elapsed_ms: u64,
    pub results: Vec<ingest::IngestResult>,
}

#[tauri::command]
pub fn ingest_project(state: State<'_, AppState>, args: IngestArgs) -> BiResult<IngestJobResponse> {
    project::get(state.inner(), &args.project_id).map_err(|_| {
        crate::error::BiError::Invalid(format!(
            "project '{}' does not exist — create it first",
            args.project_id
        ))
    })?;
    let root = std::path::PathBuf::from(&args.root_path);
    if !root.exists() {
        return Err(crate::error::BiError::Invalid(format!(
            "root_path '{}' does not exist on disk",
            args.root_path
        )));
    }

    let job_id = format!("ing-{}", uuid::Uuid::new_v4());
    let state_clone = state.inner().clone();
    let project_id = args.project_id.clone();
    let job_id_for_thread = job_id.clone();

    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        match ingest::ingest_project(&state_clone, &project_id, &root) {
            Ok(result) => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                if let Some(app) = &state_clone.app {
                    let _ = app.emit(
                        "ingest:done",
                        IngestDone {
                            job_id: job_id_for_thread.clone(),
                            project_id: project_id.clone(),
                            files_indexed: result.files_indexed,
                            chunks_indexed: result.chunks_indexed,
                            edges_created: result.edges_created,
                            elapsed_ms,
                        },
                    );
                }
            }
            Err(e) => {
                if let Some(app) = &state_clone.app {
                    let _ = app.emit(
                        "ingest:error",
                        IngestError {
                            job_id: job_id_for_thread,
                            project_id: project_id.clone(),
                            error: e.to_string(),
                        },
                    );
                }
            }
        }
    });

    Ok(IngestJobResponse {
        job_id,
        project_id: args.project_id,
    })
}

#[tauri::command]
pub fn ingest_multiple_projects(
    state: State<'_, AppState>,
    args: MultiIngestArgs,
) -> BiResult<IngestJobResponse> {
    // Validate all projects exist and paths exist
    for (project_id, root_path) in &args.projects {
        project::get(state.inner(), project_id).map_err(|_| {
            crate::error::BiError::Invalid(format!(
                "project '{}' does not exist — create it first",
                project_id
            ))
        })?;
        let root = std::path::PathBuf::from(root_path);
        if !root.exists() {
            return Err(crate::error::BiError::Invalid(format!(
                "root_path '{}' for project '{}' does not exist on disk",
                root_path, project_id
            )));
        }
    }

    let job_id = format!("multi-ing-{}", uuid::Uuid::new_v4());
    let state_clone = state.inner().clone();
    let job_id_for_thread = job_id.clone();
    let projects: Vec<(String, std::path::PathBuf)> = args
        .projects
        .into_iter()
        .map(|(project_id, root_path)| (project_id, std::path::PathBuf::from(root_path)))
        .collect();

    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        match ingest::ingest_multiple_projects(&state_clone, projects) {
            Ok(result) => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                if let Some(app) = &state_clone.app {
                    let _ = app.emit(
                        "multi-ingest:done",
                        MultiIngestDone {
                            job_id: job_id_for_thread.clone(),
                            total_files_indexed: result.total_files_indexed,
                            total_chunks_indexed: result.total_chunks_indexed,
                            total_edges_created: result.total_edges_created,
                            elapsed_ms,
                            results: result.results,
                        },
                    );
                }
            }
            Err(e) => {
                if let Some(app) = &state_clone.app {
                    let _ = app.emit(
                        "ingest:error",
                        IngestError {
                            job_id: job_id_for_thread,
                            project_id: "multiple".to_string(),
                            error: e.to_string(),
                        },
                    );
                }
            }
        }
    });

    Ok(IngestJobResponse {
        job_id,
        project_id: "multiple".to_string(),
    })
}

#[derive(Deserialize)]
pub struct GraphArgs {
    pub project_id: String,
}

#[tauri::command]
pub fn get_project_graph(
    state: State<'_, AppState>,
    args: GraphArgs,
) -> BiResult<ingest::GraphData> {
    ingest::get_project_graph(state.inner(), &args.project_id)
}

/// Enqueue a consolidate job. Returns immediately; the UI observes progress
/// through `consolidate_status` and the `consolidate:done` Tauri event.
#[tauri::command]
pub fn consolidate_now(
    state: State<'_, AppState>,
    project_id: Option<String>,
) -> BiResult<ConsolidateStatus> {
    crate::scheduler::enqueue(state.inner(), project_id)?;
    Ok(crate::scheduler::get_status())
}

#[tauri::command]
pub fn consolidate_status(
    _state: State<'_, AppState>,
) -> BiResult<crate::scheduler::ConsolidateStatus> {
    Ok(crate::scheduler::get_status())
}

#[derive(Deserialize)]
pub struct ImportArgs {
    pub project_id: String,
    pub root_path: String,
}

#[tauri::command]
pub fn import_folder(
    state: State<'_, AppState>,
    args: ImportArgs,
) -> BiResult<crate::io::ImportResult> {
    project::get(state.inner(), &args.project_id).map_err(|_| {
        crate::error::BiError::Invalid(format!(
            "project '{}' does not exist — create it first",
            args.project_id
        ))
    })?;
    let root = std::path::Path::new(&args.root_path);
    if !root.exists() {
        return Err(crate::error::BiError::Invalid(format!(
            "root_path '{}' does not exist on disk",
            args.root_path
        )));
    }
    crate::io::import_folder(state.inner(), &args.project_id, root)
}

#[derive(Deserialize)]
pub struct ExportArgs {
    pub project_id: Option<String>,
    pub output_path: String,
}

#[tauri::command]
pub fn export_memories(
    state: State<'_, AppState>,
    args: ExportArgs,
) -> BiResult<crate::io::ExportResult> {
    crate::io::export_memories(
        state.inner(),
        args.project_id.as_deref(),
        std::path::Path::new(&args.output_path),
    )
}

#[derive(Deserialize)]
pub struct WatchArgs {
    pub project_id: String,
    pub root_path: Option<String>,
    pub enabled: bool,
}

#[tauri::command]
pub fn set_watch(state: State<'_, AppState>, args: WatchArgs) -> BiResult<crate::io::WatchStatus> {
    if args.enabled {
        let root =
            if let Some(r) = args.root_path.as_ref() {
                std::path::PathBuf::from(r)
            } else {
                let conn = state.db.conn()?;
                let root: Option<String> = conn
                    .query_row(
                        "SELECT root_path FROM projects WHERE id = ?1",
                        rusqlite::params![&args.project_id],
                        |r| r.get(0),
                    )
                    .ok()
                    .flatten();
                std::path::PathBuf::from(root.ok_or_else(|| {
                    crate::error::BiError::Invalid("no root_path on project".into())
                })?)
            };
        crate::io::enable_watch(state.inner(), &args.project_id, &root)?;
    } else {
        crate::io::disable_watch(state.inner(), &args.project_id)?;
    }
    Ok(crate::io::watch_status())
}

#[tauri::command]
pub fn watch_status() -> crate::io::WatchStatus {
    crate::io::watch_status()
}

#[derive(Deserialize)]
pub struct SetModelArgs {
    pub project_id: String,
    pub model: Option<String>,
}

#[tauri::command]
pub fn set_project_embed_model(state: State<'_, AppState>, args: SetModelArgs) -> BiResult<()> {
    crate::io::set_project_embed_model(state.inner(), &args.project_id, args.model.as_deref())
}

#[tauri::command]
pub fn stats(state: State<'_, AppState>) -> BiResult<Stats> {
    let conn = state.db.conn()?;
    let total_memories: i64 = conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))?;
    let total_projects: i64 = conn.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))?;
    let total_agents: i64 = conn
        .query_row("SELECT COUNT(*) FROM agents", [], |r| r.get(0))
        .unwrap_or(0);
    let by_type = memory::count_by_type(state.inner(), None)?;
    let mut by_project: Vec<(String, i64)> = {
        let mut s = conn.prepare("SELECT id, memory_count FROM projects")?;
        let v: Vec<_> = s
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        drop(s);
        v
    };
    by_project.sort_by_key(|b| std::cmp::Reverse(b.1));

    let index_bytes = state.index_bytes();

    let week_ago = chrono::Utc::now().timestamp_millis() - 7 * 24 * 3600 * 1000;
    let recent_writes_7d: i64 = conn.query_row(
        "SELECT COUNT(*) FROM activity WHERE action IN ('write','update') AND created_at > ?1",
        rusqlite::params![week_ago],
        |r| r.get(0),
    )?;
    let recent_reads_7d: i64 = conn.query_row(
        "SELECT COUNT(*) FROM activity WHERE action = 'read' AND created_at > ?1",
        rusqlite::params![week_ago],
        |r| r.get(0),
    )?;

    Ok(Stats {
        total_memories,
        total_projects,
        total_agents,
        by_type,
        by_project,
        index_bytes,
        recent_writes_7d,
        recent_reads_7d,
    })
}

#[derive(Serialize)]
pub struct Bootstrap {
    pub stats: Stats,
    pub projects: Vec<Project>,
    pub recent: Vec<ActivityEntry>,
    pub tags: Vec<(String, i64)>,
    pub agents: Vec<AgentEntry>,
    pub consolidate: crate::scheduler::ConsolidateStatus,
}

#[tauri::command]
pub fn bootstrap(state: State<'_, AppState>) -> BiResult<Bootstrap> {
    Ok(Bootstrap {
        stats: stats(state.clone())?,
        projects: project::list(state.inner())?,
        recent: recent_activity_inner(state.inner(), 25)?,
        tags: memory::list_tags(state.inner(), None)?,
        agents: list_agents_inner(state.inner())?,
        consolidate: crate::scheduler::get_status(),
    })
}

fn recent_activity_inner(state: &AppState, limit: usize) -> BiResult<Vec<ActivityEntry>> {
    let conn = state.db.conn()?;
    let mut s = conn.prepare(
        "SELECT id, project_id, agent_id, action, memory_uid, detail, created_at
         FROM activity ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = s.query_map(rusqlite::params![limit as i64], |r| {
        let detail_str: Option<String> = r.get(5)?;
        let detail = detail_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        Ok(ActivityEntry {
            id: r.get(0)?,
            project_id: r.get(1)?,
            agent_id: r.get(2)?,
            action: r.get(3)?,
            memory_uid: r.get(4)?,
            detail,
            created_at: r.get(6)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

fn list_agents_inner(state: &AppState) -> BiResult<Vec<AgentEntry>> {
    let conn = state.db.conn()?;
    let mut s = conn.prepare(
        "SELECT id, name, kind, last_seen, created_at, meta FROM agents ORDER BY last_seen DESC",
    )?;
    let rows = s.query_map([], |r| {
        let meta_str: Option<String> = r.get(5)?;
        let meta = meta_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        Ok(AgentEntry {
            id: r.get(0)?,
            name: r.get(1)?,
            kind: r.get(2)?,
            last_seen: r.get(3)?,
            created_at: r.get(4)?,
            meta,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn list_agents(state: State<'_, AppState>) -> BiResult<Vec<AgentEntry>> {
    let conn = state.db.conn()?;
    let mut s = conn.prepare(
        "SELECT id, name, kind, last_seen, created_at, meta FROM agents ORDER BY last_seen DESC",
    )?;
    let rows = s.query_map([], |r| {
        let meta_str: Option<String> = r.get(5)?;
        let meta = meta_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        Ok(AgentEntry {
            id: r.get(0)?,
            name: r.get(1)?,
            kind: r.get(2)?,
            last_seen: r.get(3)?,
            created_at: r.get(4)?,
            meta,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[derive(Deserialize)]
pub struct RegisterAgentArgs {
    pub name: String,
    pub kind: String,
    pub meta: Option<serde_json::Value>,
}

#[tauri::command]
pub fn register_agent(state: State<'_, AppState>, args: RegisterAgentArgs) -> BiResult<AgentEntry> {
    let now = chrono::Utc::now().timestamp_millis();
    let id = slugify(&args.name);
    let meta_str = args.meta.as_ref().map(|v| v.to_string());
    state.db.write(|tx| {
        tx.execute(
            "INSERT INTO agents(id, name, kind, last_seen, created_at, meta)
             VALUES(?1,?2,?3,?4,?4,?5)
             ON CONFLICT(id) DO UPDATE SET last_seen = excluded.last_seen,
                                            meta = COALESCE(excluded.meta, agents.meta)",
            rusqlite::params![id, args.name, args.kind, now, meta_str],
        )?;
        Ok(())
    })?;
    Ok(AgentEntry {
        id,
        name: args.name,
        kind: args.kind,
        last_seen: now,
        created_at: now,
        meta: args.meta,
    })
}

#[tauri::command]
pub fn recent_activity(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> BiResult<Vec<ActivityEntry>> {
    let conn = state.db.conn()?;
    let mut s = conn.prepare(
        "SELECT id, project_id, agent_id, action, memory_uid, detail, created_at
         FROM activity ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = s.query_map(rusqlite::params![limit.unwrap_or(100) as i64], |r| {
        let detail_str: Option<String> = r.get(5)?;
        let detail = detail_str
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok());
        Ok(ActivityEntry {
            id: r.get(0)?,
            project_id: r.get(1)?,
            agent_id: r.get(2)?,
            action: r.get(3)?,
            memory_uid: r.get(4)?,
            detail,
            created_at: r.get(6)?,
        })
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
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

// ── MCP config auto-install ─────────────────────────────────────────────

#[derive(Serialize)]
pub struct ResolveMcpBinaryResult {
    pub path: String,
    pub is_absolute: bool,
}

/// Resolve the absolute path to the bundled `biturbo-mcp` binary.
/// Tries: current_exe parent dir → dev build paths → bare name fallback.
#[tauri::command]
pub fn resolve_mcp_binary_path() -> ResolveMcpBinaryResult {
    let exe_name = if cfg!(windows) { "biturbo-mcp.exe" } else { "biturbo-mcp" };

    // 1. Look next to the running app binary (installed builds)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(exe_name);
            if candidate.exists() {
                return ResolveMcpBinaryResult {
                    path: candidate.to_string_lossy().to_string(),
                    is_absolute: true,
                };
            }
            // macOS .app bundle: exe is Contents/MacOS/biTurbo, binary is alongside
            // Already covered by parent.join above.
        }
    }

    // 2. Dev build paths
    for rel in &["src-tauri/target/release", "src-tauri/target/debug"] {
        if let Ok(cwd) = std::env::current_dir() {
            let candidate = cwd.join(rel).join(exe_name);
            if candidate.exists() {
                return ResolveMcpBinaryResult {
                    path: candidate.to_string_lossy().to_string(),
                    is_absolute: true,
                };
            }
        }
    }

    // 3. Fallback: bare name
    ResolveMcpBinaryResult {
        path: "biturbo-mcp".to_string(),
        is_absolute: false,
    }
}

#[derive(Deserialize)]
pub struct InstallMcpConfigArgs {
    pub target: String,
}

#[derive(Serialize)]
pub struct InstallMcpConfigResult {
    pub target: String,
    pub path: String,
    pub created: bool,
    pub merged: bool,
}

/// Install or merge biTurbo's MCP config into an agent's config file.
/// Non-destructive: preserves existing servers, only adds/overwrites the `biturbo` entry.
#[tauri::command]
pub fn install_mcp_config(
    _state: State<'_, AppState>,
    args: InstallMcpConfigArgs,
) -> BiResult<InstallMcpConfigResult> {
    let bin = resolve_mcp_binary_path();
    let bin_path = &bin.path;

    let home = dirs::home_dir().ok_or_else(|| {
        crate::error::BiError::Invalid("cannot resolve home directory".into())
    })?;

    let (config_path, format): (std::path::PathBuf, &str) = match args.target.as_str() {
        "cursor" => (home.join(".cursor").join("mcp.json"), "json-cursor"),
        "windsurf" => (home.join(".codeium").join("windsurf").join("mcp_config.json"), "json-cursor"),
        "claude" => (home.join(".claude.json"), "json-cursor"),
        "opencode" => {
            let base = if cfg!(target_os = "macos") {
                home.join("Library").join("Application Support").join("opencode")
            } else {
                home.join(".config").join("opencode")
            };
            (base.join("opencode.json"), "json-opencode")
        }
        "codex" => (home.join(".codex").join("config.toml"), "toml-codex"),
        other => return Err(crate::error::BiError::Invalid(format!("unknown target: {other}"))),
    };

    // Create parent dirs
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            crate::error::BiError::Invalid(format!("failed to create {}: {e}", parent.display()))
        })?;
    }

    let existed = config_path.exists();

    match format {
        "json-cursor" => {
            // Cursor/Windsurf/Claude: { "mcpServers": { "biturbo": { ... } } }
            let mut root: serde_json::Value = if existed {
                let content = std::fs::read_to_string(&config_path).map_err(|e| {
                    crate::error::BiError::Invalid(format!("failed to read {}: {e}", config_path.display()))
                })?;
                serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            // Ensure mcpServers object exists
            if root.get("mcpServers").is_none() {
                root["mcpServers"] = serde_json::json!({});
            }

            // Add/overwrite biturbo entry, preserve others
            root["mcpServers"]["biturbo"] = serde_json::json!({
                "command": bin_path,
                "args": [],
                "env": {}
            });

            let output = serde_json::to_string_pretty(&root).map_err(|e| {
                crate::error::BiError::Invalid(format!("failed to serialize JSON: {e}"))
            })?;
            std::fs::write(&config_path, output).map_err(|e| {
                crate::error::BiError::Invalid(format!("failed to write {}: {e}", config_path.display()))
            })?;
        }
        "json-opencode" => {
            // OpenCode: { "mcp": { "biturbo": { "type": "local", "command": [...], ... } } }
            let mut root: serde_json::Value = if existed {
                let content = std::fs::read_to_string(&config_path).map_err(|e| {
                    crate::error::BiError::Invalid(format!("failed to read {}: {e}", config_path.display()))
                })?;
                serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if root.get("mcp").is_none() {
                root["mcp"] = serde_json::json!({});
            }

            root["mcp"]["biturbo"] = serde_json::json!({
                "type": "local",
                "command": [bin_path],
                "enabled": true
            });

            let output = serde_json::to_string_pretty(&root).map_err(|e| {
                crate::error::BiError::Invalid(format!("failed to serialize JSON: {e}"))
            })?;
            std::fs::write(&config_path, output).map_err(|e| {
                crate::error::BiError::Invalid(format!("failed to write {}: {e}", config_path.display()))
            })?;
        }
        "toml-codex" => {
            // Codex: ~/.codex/config.toml — [mcp_servers.biturbo] table
            // Simple text manipulation: remove existing [mcp_servers.biturbo] block, append new one.
            let biturbo_block = format!(
                "[mcp_servers.biturbo]\ncommand = \"{bin_path}\"\nargs = []\n"
            );

            let content = if existed {
                std::fs::read_to_string(&config_path).unwrap_or_default()
            } else {
                String::new()
            };

            // Remove existing [mcp_servers.biturbo] section (from header line to next section or EOF)
            let mut new_content = String::new();
            let mut skip = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && !trimmed.starts_with("[mcp_servers.biturbo") {
                    skip = false;
                }
                if trimmed == "[mcp_servers.biturbo]" || trimmed.starts_with("[mcp_servers.biturbo]") {
                    skip = true;
                    continue;
                }
                if !skip {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }

            // Ensure there's a blank line before our block if content doesn't end with one
            if !new_content.is_empty() && !new_content.ends_with("\n\n") {
                new_content.push('\n');
            }
            new_content.push_str(&biturbo_block);

            std::fs::write(&config_path, new_content).map_err(|e| {
                crate::error::BiError::Invalid(format!("failed to write {}: {e}", config_path.display()))
            })?;
        }
        _ => unreachable!(),
    }

    Ok(InstallMcpConfigResult {
        target: args.target,
        path: config_path.to_string_lossy().to_string(),
        created: !existed,
        merged: existed,
    })
}
