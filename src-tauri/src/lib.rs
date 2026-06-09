//! biTurbo — local-first memory layer for AI coding agents.
//!
//! Architecture
//! ────────────
//! ┌─────────────────────────────────────────────────────────────┐
//! │  Tauri 2 desktop app  (this crate)                          │
//! │   ├── commands::*        — IPC handlers (GUI ↔ backend)     │
//! │   ├── mcp                 — MCP stdio server for AI agents   │
//! │   ├── memory             — CRUD over memory entries         │
//! │   ├── project            — multi-project isolation          │
//! │   ├── index_engine       — turbovec IdMapIndex wrapper      │
//! │   ├── embed              — fastembed (BGE) embeddings       │
//! │   ├── ingest             — tree-sitter project indexing     │
//! │   ├── consolidate        — decay / dedup / merge            │
//! │   └── db                 — SQLite schema + connection pool  │
//! └─────────────────────────────────────────────────────────────┘
//!
//! Data lives in the OS app-data dir (~/Library/Application Support/com.biturbo.app/
//! on macOS). Both the GUI and the MCP server share the same on-disk state.

pub mod commands;
pub mod consolidate;
pub mod db;
pub mod embed;
pub mod error;
pub mod index_engine;
pub mod ingest;
pub mod io;
pub mod mcp;
pub mod memory;
pub mod project;
pub mod scheduler;
pub mod smoke;
pub mod state;

pub use error::{BiError, BiResult};
pub use state::AppState;

use tauri::Manager;
use tracing::info;
use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "biturbo_lib=info,tauri=info".into()),
        )
        .with_target(false)
        .compact()
        .init();

    info!("biTurbo starting…");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("app data dir resolvable");
            std::fs::create_dir_all(&data_dir).ok();

            let mut state = AppState::open(&data_dir).expect("open app state");
            state.app = Some(app.handle().clone());
            let state_arc = Arc::new(state);
            scheduler::spawn(state_arc.clone());
            io::resume_watches(&state_arc);
            app.manage((*state_arc).clone());
            info!("biTurbo ready @ {}", data_dir.display());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::list_memories,
            commands::get_memory,
            commands::remember,
            commands::forget_memory,
            commands::update_memory,
            commands::search_memories,
            commands::list_projects,
            commands::create_project,
            commands::delete_project,
            commands::get_project,
            commands::ingest_project,
            commands::get_project_graph,
            commands::list_tags,
            commands::consolidate_now,
            commands::consolidate_status,
            commands::import_folder,
            commands::export_memories,
            commands::set_watch,
            commands::watch_status,
            commands::set_project_embed_model,
            commands::stats,
            commands::list_agents,
            commands::register_agent,
            commands::recent_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
