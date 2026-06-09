//! Integration smoke tests exercising AppState end-to-end without the GUI.
//! Covers remember/search, debounced index flush, ingest, and MCP-style
//! persistence (AppState::open spawns the flusher thread in every process).

#[cfg(test)]
mod tests {
    use crate::memory::{self, RememberInput};
    use crate::state::AppState;
    use std::path::Path;
    use std::time::{Duration, Instant};

    fn temp_data_dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("biturbo-smoke-{}", uuid::Uuid::new_v4()))
    }

    fn wait_for_index_flush(state: &AppState, project_id: &str, timeout: Duration) -> bool {
        let tvim = state
            .data_dir
            .join("indices")
            .join(format!("{project_id}.tvim"));
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if tvim.exists() {
                let tmp = tvim.with_extension("tvim.tmp");
                let meta_tmp = state
                    .data_dir
                    .join("indices")
                    .join(format!("{project_id}.uidmap.json.tmp"));
                if !tmp.exists() && !meta_tmp.exists() {
                    return true;
                }
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        tvim.exists()
    }

    #[test]
    fn remember_search_and_debounced_flush() {
        let data_dir = temp_data_dir();
        let state = AppState::open(&data_dir).expect("open state");
        let project_id = &state.default_project_id;

        let mem = memory::remember(
            &state,
            RememberInput {
                content: "biTurbo smoke test: debounced flush verification".into(),
                mem_type: Some("fact".into()),
                tags: Some(vec!["smoke".into()]),
                ..Default::default()
            },
        )
        .expect("remember");

        let hits = memory::search(&state, project_id, "debounced flush verification", 5, None)
            .expect("search");
        assert!(
            hits.iter().any(|h| h.memory.uid == mem.uid),
            "search did not return remembered memory"
        );

        assert!(
            wait_for_index_flush(&state, project_id, Duration::from_secs(3)),
            "index file not flushed within 3s"
        );

        let indices_dir = data_dir.join("indices");
        assert!(indices_dir.join(format!("{project_id}.tvim")).exists());
        assert!(indices_dir
            .join(format!("{project_id}.uidmap.json"))
            .exists());
        assert!(
            !indices_dir.read_dir().unwrap().any(|e| {
                e.ok()
                    .map(|f| f.path().extension().and_then(|s| s.to_str()) == Some("tmp"))
                    .unwrap_or(false)
            }),
            "stale .tmp files left behind"
        );

        std::fs::remove_dir_all(&data_dir).ok();
    }

    #[test]
    fn ingest_small_rust_tree() {
        let data_dir = temp_data_dir();
        let state = AppState::open(&data_dir).expect("open state");
        let project_id = &state.default_project_id;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

        let result = crate::ingest::ingest_project(&state, project_id, &root).expect("ingest");
        assert!(result.files_indexed > 0, "no files indexed");
        assert!(result.chunks_indexed > 0, "no chunks indexed");

        let hits = memory::search(
            &state,
            project_id,
            "tree-sitter project indexing",
            5,
            Some("code"),
        )
        .expect("search code");
        assert!(!hits.is_empty(), "ingested code not searchable");

        assert!(
            wait_for_index_flush(&state, project_id, Duration::from_secs(3)),
            "index not flushed after ingest"
        );

        std::fs::remove_dir_all(&data_dir).ok();
    }

    #[test]
    fn mcp_style_reopen_persists_index() {
        let data_dir = temp_data_dir();

        let uid = {
            let state = AppState::open(&data_dir).expect("open state 1");
            let mem = memory::remember(
                &state,
                RememberInput {
                    content: "MCP persistence smoke test memory".into(),
                    ..Default::default()
                },
            )
            .expect("remember");
            assert!(wait_for_index_flush(
                &state,
                &state.default_project_id,
                Duration::from_secs(3)
            ));
            mem.uid
        };

        // Simulate MCP process restart: new AppState on same data dir.
        let state2 = AppState::open(&data_dir).expect("open state 2");
        let hits = memory::search(
            &state2,
            &state2.default_project_id,
            "MCP persistence smoke",
            5,
            None,
        )
        .expect("search after reopen");
        assert!(
            hits.iter().any(|h| h.memory.uid == uid),
            "memory not found after AppState reopen (MCP persistence regression)"
        );

        std::fs::remove_dir_all(&data_dir).ok();
    }
}
