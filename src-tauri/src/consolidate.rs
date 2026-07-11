use crate::db::log_activity;
use crate::error::BiResult;
use crate::memory::{self, Memory};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsolidateReport {
    pub decayed: usize,
    pub duplicates_found: usize,
    pub merged: usize,
    pub removed: usize,
}

pub fn consolidate(state: &AppState, project_id: Option<&str>) -> BiResult<ConsolidateReport> {
    let mut report = ConsolidateReport {
        decayed: apply_decay(state, project_id)?,
        ..Default::default()
    };
    let dupes = find_duplicates(state, project_id)?;
    report.duplicates_found = dupes.len();
    for (keep_uid, drop_uid) in dupes {
        if merge_pair(state, &keep_uid, &drop_uid)? {
            report.merged += 1;
            report.removed += 1;
        }
    }

    state.db.write(|tx| {
        log_activity(
            tx,
            project_id,
            None,
            "consolidate",
            None,
            Some(&serde_json::to_value(&report)?),
        )?;
        Ok(())
    })?;

    Ok(report)
}

fn apply_decay(state: &AppState, project_id: Option<&str>) -> BiResult<usize> {
    let now = chrono::Utc::now().timestamp_millis();
    let half_life_ms: i64 = 60 * 24 * 3600 * 1000;
    let conn = state.db.conn()?;

    let rows: Vec<(String, f64, i64, i64, i64)> = match project_id {
        Some(p) => {
            let mut stmt = conn.prepare(
                "SELECT uid, importance, created_at, access_count, last_access
                 FROM memories WHERE project_id = ?1",
            )?;
            let v: Vec<_> = stmt
                .query_map(rusqlite::params![p], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, f64>(1)?,
                        r.get::<_, i64>(2)?,
                        r.get::<_, i64>(3)?,
                        r.get::<_, i64>(4)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);
            v
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT uid, importance, created_at, access_count, last_access FROM memories",
            )?;
            let v: Vec<_> = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, f64>(1)?,
                        r.get::<_, i64>(2)?,
                        r.get::<_, i64>(3)?,
                        r.get::<_, i64>(4)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);
            v
        }
    };

    // Compute new importances first, then apply every change inside ONE
    // transaction with a cached statement — previously each row was its own
    // autocommit transaction.
    let mut updates: Vec<(String, f32)> = Vec::new();
    for (uid, importance, created_at, access_count, last_access) in rows {
        let age_ms = (now - created_at).max(0) as f64;
        let decay = (-age_ms / half_life_ms as f64).exp();
        let recent_access = (now - last_access) < 30 * 24 * 3600 * 1000;
        let boost = if recent_access {
            (access_count as f64 * 0.05).min(0.3)
        } else {
            0.0
        };
        let new_imp = (importance * decay + boost).clamp(0.05, 1.0) as f32;
        if (new_imp - importance as f32).abs() > 0.001 {
            updates.push((uid, new_imp));
        }
    }
    drop(conn);
    let touched = updates.len();
    if !updates.is_empty() {
        state.db.write(|tx| {
            let mut stmt =
                tx.prepare_cached("UPDATE memories SET importance = ?1 WHERE uid = ?2")?;
            for (uid, new_imp) in &updates {
                stmt.execute(rusqlite::params![new_imp, uid])?;
            }
            Ok(())
        })?;
    }
    Ok(touched)
}

fn find_duplicates(state: &AppState, project_id: Option<&str>) -> BiResult<Vec<(String, String)>> {
    let conn = state.db.conn()?;
    let project_ids: Vec<String> = match project_id {
        Some(p) => vec![p.to_string()],
        None => {
            let mut s = conn.prepare("SELECT DISTINCT project_id FROM memories")?;
            let v: Vec<String> = s
                .query_map([], |r| r.get::<_, String>(0))?
                .filter_map(|r| r.ok())
                .collect();
            drop(s);
            v
        }
    };
    drop(conn);

    let mut dupes: HashSet<(String, String)> = HashSet::new();
    for pid in project_ids {
        // Process in batches to bound RAM. Skip code-type memories —
        // deduplicating code chunks is expensive and rarely useful.
        let idx = state.get_or_load_index(&pid)?;
        let mut offset = 0usize;
        const BATCH: usize = 1000;
        loop {
            let mems: Vec<Memory> = memory::list(state, Some(&pid), None, BATCH, offset)?;
            if mems.is_empty() {
                break;
            }
            let non_code: Vec<&Memory> = mems.iter().filter(|m| m.mem_type != "code").collect();
            if !non_code.is_empty() {
                let by_uid: std::collections::HashMap<&str, &Memory> =
                    non_code.iter().map(|m| (m.uid.as_str(), *m)).collect();
                let texts: Vec<&str> = non_code.iter().map(|m| m.content.as_str()).collect();
                let embeddings = state.embedder.embed_batch(&texts)?;
                for (i, vec) in embeddings.iter().enumerate() {
                    let a = non_code[i];
                    let hits = idx.search(vec, 5, None)?;
                    for h in hits {
                        if h.score < 0.95 || h.uid == a.uid {
                            continue;
                        }
                        if let Some(b) = by_uid.get(h.uid.as_str()) {
                            let (keep, drop_) = if a.importance >= b.importance {
                                (a.uid.clone(), b.uid.clone())
                            } else {
                                (b.uid.clone(), a.uid.clone())
                            };
                            dupes.insert((keep, drop_));
                        }
                    }
                }
            }
            offset += BATCH;
            if mems.len() < BATCH {
                break;
            }
        }
    }
    Ok(dupes.into_iter().collect())
}

fn merge_pair(state: &AppState, keep_uid: &str, drop_uid: &str) -> BiResult<bool> {
    let keep = memory::get(state, keep_uid)?;
    let drop_ = memory::get(state, drop_uid)?;
    let (Some(keep), Some(_drop_)) = (keep, drop_) else {
        return Ok(false);
    };

    let mut merged_tags: Vec<String> = keep.tags.clone();
    for t in &_drop_.tags {
        if !merged_tags.contains(t) {
            merged_tags.push(t.clone());
        }
    }
    memory::update(
        state,
        keep_uid,
        crate::memory::UpdateInput {
            tags: Some(merged_tags),
            ..Default::default()
        },
    )?;

    let now = chrono::Utc::now().timestamp_millis();
    state.db.write(|tx| {
        tx.execute(
            "UPDATE memories SET superseded_by = (SELECT id FROM memories WHERE uid = ?1),
                                 updated_at = ?2 WHERE uid = ?3",
            rusqlite::params![keep_uid, now, drop_uid],
        )?;
        Ok(())
    })?;
    memory::forget(state, drop_uid)?;
    Ok(true)
}
