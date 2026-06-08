use crate::consolidate::{self, ConsolidateReport};
use crate::db::log_activity;
use crate::error::BiResult;
use crate::state::AppState;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;

const INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
const STARTUP_DELAY: Duration = Duration::from_secs(60);

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ConsolidateStatus {
    pub last_run_at: Option<i64>,
    pub next_run_in_secs: u64,
    pub last_report: Option<ConsolidateReport>,
    pub running: bool,
    pub interval_secs: u64,
}

struct Shared {
    last_run_at: Option<i64>,
    last_report: Option<ConsolidateReport>,
    running: bool,
    last_finish: Option<Instant>,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            last_run_at: None,
            last_report: None,
            running: false,
            last_finish: None,
        }
    }
}

static STATE: once_cell::sync::Lazy<Arc<Mutex<Shared>>> = once_cell::sync::Lazy::new(|| {
    Arc::new(Mutex::new(Shared::default()))
});

pub fn spawn(state: Arc<AppState>) {
    let state_for_task = state.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(STARTUP_DELAY).await;
        loop {
            run_once(&state_for_task).await;
            tokio::time::sleep(INTERVAL).await;
        }
    });
}

pub fn run_now(state: &AppState) -> BiResult<ConsolidateReport> {
    let report = consolidate::consolidate(state, None)?;
    let now = chrono::Utc::now().timestamp_millis();
    {
        let mut g = STATE.lock();
        g.last_run_at = Some(now);
        g.last_report = Some(report.clone());
        g.running = false;
        g.last_finish = Some(Instant::now());
    }
    state.db.write(|tx| {
        log_activity(
            tx,
            None,
            None,
            "consolidate",
            None,
            Some(&serde_json::to_value(&report)?),
        )?;
        Ok(())
    })?;
    Ok(report)
}

pub fn get_status() -> ConsolidateStatus {
    let g = STATE.lock();
    let next = if let Some(finish) = g.last_finish {
        INTERVAL.as_secs().saturating_sub(finish.elapsed().as_secs())
    } else {
        INTERVAL.as_secs()
    };
    ConsolidateStatus {
        last_run_at: g.last_run_at,
        next_run_in_secs: next,
        last_report: g.last_report.clone(),
        running: g.running,
        interval_secs: INTERVAL.as_secs(),
    }
}

async fn run_once(state: &AppState) {
    {
        let mut g = STATE.lock();
        if g.running { return; }
        g.running = true;
    }
    let report = consolidate::consolidate(state, None);
    let now = chrono::Utc::now().timestamp_millis();
    {
        let mut g = STATE.lock();
        g.running = false;
        g.last_finish = Some(Instant::now());
    }
    match report {
        Ok(r) => {
            {
                let mut g = STATE.lock();
                g.last_run_at = Some(now);
                g.last_report = Some(r.clone());
            }
            let _ = state.db.write(|tx| {
                log_activity(
                    tx,
                    None,
                    None,
                    "consolidate",
                    None,
                    Some(&serde_json::to_value(&r)?),
                )?;
                Ok(())
            });
            if let Some(app) = &state.app {
                let _ = app.emit("consolidate:done", &r);
            }
        }
        Err(e) => {
            tracing::warn!("consolidate run failed: {e}");
        }
    }
}
