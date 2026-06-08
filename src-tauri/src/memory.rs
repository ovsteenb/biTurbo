use crate::db::{self, log_activity};
use crate::error::{BiError, BiResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemType {
    Fact,
    Decision,
    Preference,
    Pattern,
    Episode,
    Reflection,
    Code,
}

impl MemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemType::Fact => "fact",
            MemType::Decision => "decision",
            MemType::Preference => "preference",
            MemType::Pattern => "pattern",
            MemType::Episode => "episode",
            MemType::Reflection => "reflection",
            MemType::Code => "code",
        }
    }
    pub fn from_str(s: &str) -> BiResult<Self> {
        Ok(match s {
            "fact" => Self::Fact,
            "decision" => Self::Decision,
            "preference" => Self::Preference,
            "pattern" => Self::Pattern,
            "episode" => Self::Episode,
            "reflection" => Self::Reflection,
            "code" => Self::Code,
            other => return Err(BiError::Invalid(format!("unknown mem_type {other}"))),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub uid: String,
    pub project_id: String,
    pub mem_type: String,
    pub content: String,
    pub tags: Vec<String>,
    pub source_agent: Option<String>,
    pub importance: f32,
    pub supersedes: Option<i64>,
    pub superseded_by: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_access: i64,
    pub access_count: i64,
    pub file_path: Option<String>,
    pub start_line: Option<i64>,
    pub end_line: Option<i64>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryWithScore {
    #[serde(flatten)]
    pub memory: Memory,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RememberInput {
    pub content: String,
    pub mem_type: Option<String>,
    pub project_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub importance: Option<f32>,
    pub source_agent: Option<String>,
    pub file_path: Option<String>,
    pub start_line: Option<i64>,
    pub end_line: Option<i64>,
    pub language: Option<String>,
    pub supersedes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateInput {
    pub content: Option<String>,
    pub mem_type: Option<String>,
    pub tags: Option<Vec<String>>,
    pub importance: Option<f32>,
}

pub fn remember(state: &AppState, input: RememberInput) -> BiResult<Memory> {
    state.embedder.release_if_idle();
    if input.content.trim().is_empty() {
        return Err(BiError::Invalid("content is empty".into()));
    }
    let project_id = input
        .project_id
        .clone()
        .unwrap_or_else(|| state.default_project_id.clone());
    crate::project::get(state, &project_id).map_err(|_| {
        BiError::Invalid(format!(
            "project '{project_id}' does not exist — create it first with create_project"
        ))
    })?;
    let mem_type = input.mem_type.as_deref().unwrap_or("fact").to_string();
    let importance = input.importance.unwrap_or(0.5).clamp(0.0, 1.0);
    let uid = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let tags_json = serde_json::to_string(&input.tags.clone().unwrap_or_default())?;

    state.db.write(|tx| {
        if let Some(old_uid) = &input.supersedes {
            tx.execute(
                "UPDATE memories SET superseded_by = id, updated_at = ?1
                 WHERE uid = ?2 AND superseded_by IS NULL",
                rusqlite::params![now, old_uid],
            )?;
        }
        tx.execute(
            "INSERT INTO memories(uid, project_id, mem_type, content, tags, source_agent,
                                  importance, created_at, updated_at, last_access,
                                  access_count, file_path, start_line, end_line, language)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8,?8,0,?9,?10,?11,?12)",
            rusqlite::params![
                uid, project_id, mem_type, input.content, tags_json, input.source_agent,
                importance, now, input.file_path, input.start_line, input.end_line, input.language,
            ],
        )?;
        tx.execute(
            "UPDATE projects SET memory_count = memory_count + 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, project_id],
        )?;
        log_activity(
            tx,
            Some(&project_id),
            input.source_agent.as_deref(),
            "write",
            Some(&uid),
            Some(&serde_json::json!({"mem_type": mem_type})),
        )?;
        Ok(())
    })?;

    state.embed_and_add(&project_id, &uid, &input.content)?;

    get(state, &uid)?.ok_or_else(|| BiError::Internal("memory not found post-insert".into()))
}

pub fn get(state: &AppState, uid: &str) -> BiResult<Option<Memory>> {
    let conn = state.db.conn()?;
    match conn.query_row(
        "SELECT uid, project_id, mem_type, content, tags, source_agent, importance,
                supersedes, superseded_by, created_at, updated_at, last_access,
                access_count, file_path, start_line, end_line, language
         FROM memories WHERE uid = ?1",
        rusqlite::params![uid],
        row_to_memory,
    ) {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn forget(state: &AppState, uid: &str) -> BiResult<bool> {
    let mem = get(state, uid)?.ok_or_else(|| BiError::NotFound(uid.into()))?;
    if let Ok(idx) = state.get_or_load_index(&mem.project_id) {
        let _ = idx.remove(uid);
    }
    let now = chrono::Utc::now().timestamp_millis();
    state.db.write(|tx| {
        tx.execute("DELETE FROM memories WHERE uid = ?1", rusqlite::params![uid])?;
        tx.execute(
            "UPDATE projects SET memory_count = MAX(0, memory_count - 1), updated_at = ?1
             WHERE id = ?2",
            rusqlite::params![now, mem.project_id],
        )?;
        log_activity(tx, Some(&mem.project_id), None, "forget", Some(uid), None)?;
        Ok(())
    })?;
    Ok(true)
}

pub fn update(state: &AppState, uid: &str, input: UpdateInput) -> BiResult<Memory> {
    let existing = get(state, uid)?.ok_or_else(|| BiError::NotFound(uid.into()))?;
    let now = chrono::Utc::now().timestamp_millis();
    let new_content = input.content.clone().unwrap_or_else(|| existing.content.clone());
    let new_type = input.mem_type.clone().unwrap_or_else(|| existing.mem_type.clone());
    let new_tags_json = match input.tags.clone() {
        Some(t) => serde_json::to_string(&t)?,
        None => serde_json::to_string(&existing.tags)?,
    };
    let new_imp = input.importance.unwrap_or(existing.importance).clamp(0.0, 1.0);

    state.db.write(|tx| {
        tx.execute(
            "UPDATE memories SET content = ?1, mem_type = ?2, tags = ?3,
                                 importance = ?4, updated_at = ?5
             WHERE uid = ?6",
            rusqlite::params![new_content, new_type, new_tags_json, new_imp, now, uid],
        )?;
        log_activity(tx, Some(&existing.project_id), None, "update", Some(uid), None)?;
        Ok(())
    })?;

    if input.content.is_some() {
        state.embed_and_add(&existing.project_id, uid, &new_content)?;
    }

    get(state, uid)?.ok_or_else(|| BiError::Internal("memory vanished after update".into()))
}

pub fn search(
    state: &AppState,
    project_id: &str,
    query: &str,
    k: usize,
    mem_type: Option<&str>,
) -> BiResult<Vec<MemoryWithScore>> {
    state.embedder.release_if_idle();
    let project_id = if project_id.is_empty() {
        state.default_project_id.clone()
    } else {
        project_id.to_string()
    };

    let allowlist = if let Some(t) = mem_type {
        let conn = state.db.conn()?;
        let mut stmt = conn.prepare(
            "SELECT uid FROM memories WHERE project_id = ?1 AND mem_type = ?2",
        )?;
        let uids: Vec<String> = stmt
            .query_map(rusqlite::params![&project_id, t], |r| r.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);
        Some(uids)
    } else {
        None
    };

    let kk = (k * 2).max(20);
    let vec_hits = state.embed_and_search(&project_id, query, kk, allowlist.as_deref())?;

    let conn = state.db.conn()?;
    let mut fts_uids: Vec<(String, f64)> = Vec::new();
    let fts_query = sanitize_fts_query(query);
    if !fts_query.is_empty() {
        let fts_sql = if let Some(t) = mem_type {
            format!(
                "SELECT uid, bm25(memories_fts) FROM memories_fts
                 WHERE memories_fts MATCH ?1 AND mem_type = ?2 AND project_id = ?3
                 ORDER BY bm25(memories_fts) ASC LIMIT ?4"
            )
        } else {
            "SELECT uid, bm25(memories_fts) FROM memories_fts
             WHERE memories_fts MATCH ?1 AND project_id = ?2
             ORDER BY bm25(memories_fts) ASC LIMIT ?3".to_string()
        };
        let mut stmt = conn.prepare(&fts_sql)?;
        let mut rows: Box<dyn Iterator<Item = (String, f64)>> = if let Some(t) = mem_type {
            Box::new(
                stmt.query_map(
                    rusqlite::params![fts_query, t, &project_id, kk as i64],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)),
                )?
                .filter_map(|r| r.ok()),
            )
        } else {
            Box::new(
                stmt.query_map(
                    rusqlite::params![fts_query, &project_id, kk as i64],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)),
                )?
                .filter_map(|r| r.ok()),
            )
        };
        for r in rows.by_ref() {
            fts_uids.push(r);
        }
    }

    let mut fused: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    const RRF_K: f32 = 60.0;
    for (rank, h) in vec_hits.iter().enumerate() {
        let score = 1.0 / (RRF_K + rank as f32 + 1.0);
        *fused.entry(h.uid.clone()).or_insert(0.0) += score;
    }
    for (rank, (uid, _bm25)) in fts_uids.iter().enumerate() {
        let score = 1.0 / (RRF_K + rank as f32 + 1.0);
        *fused.entry(uid.clone()).or_insert(0.0) += score;
    }

    let mut ranked: Vec<(String, f32)> = fused.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(k);

    if ranked.is_empty() {
        return Ok(Vec::new());
    }

    let n = ranked.len();
    let placeholders = std::iter::repeat("?").take(n).collect::<Vec<_>>().join(",");
    let select_sql = format!(
        "SELECT uid, project_id, mem_type, content, tags, source_agent, importance,
                supersedes, superseded_by, created_at, updated_at, last_access,
                access_count, file_path, start_line, end_line, language
         FROM memories WHERE uid IN ({})",
        placeholders
    );
    let mut stmt = conn.prepare(&select_sql)?;
    let params: Vec<&dyn rusqlite::ToSql> = ranked.iter().map(|(u, _)| u as &dyn rusqlite::ToSql).collect();
    let by_uid: std::collections::HashMap<String, Memory> = stmt
        .query_map(params.as_slice(), row_to_memory)?
        .filter_map(|r| r.ok())
        .map(|m| (m.uid.clone(), m))
        .collect();
    drop(stmt);

    let now = chrono::Utc::now().timestamp_millis();
    let update_sql = format!(
        "UPDATE memories SET access_count = access_count + 1, last_access = ?1 WHERE uid IN ({})",
        placeholders
    );
    let mut upd_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(n + 1);
    upd_params.push(Box::new(now));
    for (u, _) in &ranked {
        upd_params.push(Box::new(u.clone()));
    }
    let upd_refs: Vec<&dyn rusqlite::ToSql> = upd_params.iter().map(|p| p.as_ref()).collect();
    conn.execute(&update_sql, upd_refs.as_slice())?;

    let act_placeholders: Vec<String> = (0..n)
        .map(|i| {
            let b = i * 6;
            format!("(?{},?{},?{},?{},?{},?{})", b + 1, b + 2, b + 3, b + 4, b + 5, b + 6)
        })
        .collect();
    let act_sql = format!(
        "INSERT INTO activity(project_id, agent_id, action, memory_uid, detail, created_at) VALUES {}",
        act_placeholders.join(",")
    );
    let mut act_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(n * 6);
    for (u, _score) in &ranked {
        act_params.push(Box::new(project_id.clone()));
        act_params.push(Box::new(Option::<String>::None));
        act_params.push(Box::new("read"));
        act_params.push(Box::new(u.clone()));
        act_params.push(Box::new(
            serde_json::to_string(&serde_json::json!({"query": query}))?,
        ));
        act_params.push(Box::new(now));
    }
    let act_refs: Vec<&dyn rusqlite::ToSql> = act_params.iter().map(|p| p.as_ref()).collect();
    conn.execute(&act_sql, act_refs.as_slice())?;

    Ok(ranked
        .into_iter()
        .filter_map(|(uid, score)| by_uid.get(&uid).cloned().map(|memory| MemoryWithScore { memory, score }))
        .collect())
}

fn sanitize_fts_query(q: &str) -> String {
    let tokens: Vec<String> = q
        .split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-'))
        .filter(|t| !t.is_empty())
        .map(|t| {
            let safe = t.replace('"', "");
            format!("\"{safe}\"*")
        })
        .collect();
    tokens.join(" ")
}

pub fn list(
    state: &AppState,
    project_id: Option<&str>,
    mem_type: Option<&str>,
    limit: usize,
    offset: usize,
) -> BiResult<Vec<Memory>> {
    let conn = state.db.conn()?;
    let mut sql = String::from(
        "SELECT uid, project_id, mem_type, content, tags, source_agent, importance,
                supersedes, superseded_by, created_at, updated_at, last_access,
                access_count, file_path, start_line, end_line, language
         FROM memories WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(p) = project_id {
        sql.push_str(" AND project_id = ?");
        params.push(Box::new(p.to_string()));
    }
    if let Some(t) = mem_type {
        sql.push_str(" AND mem_type = ?");
        params.push(Box::new(t.to_string()));
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
    params.push(Box::new(limit as i64));
    params.push(Box::new(offset as i64));

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), row_to_memory)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn count_by_type(state: &AppState, project_id: Option<&str>) -> BiResult<Vec<(String, i64)>> {
    let conn = state.db.conn()?;
    let (sql, p): (String, Vec<Box<dyn rusqlite::ToSql>>) = match project_id {
        Some(pid) => (
            "SELECT mem_type, COUNT(*) FROM memories WHERE project_id = ?1 GROUP BY mem_type".to_string(),
            vec![Box::new(pid.to_string())],
        ),
        None => (
            "SELECT mem_type, COUNT(*) FROM memories GROUP BY mem_type".to_string(),
            vec![],
        ),
    };
    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::ToSql> = p.iter().map(|b| b.as_ref()).collect();
    let rows = stmt.query_map(refs.as_slice(), |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
    })?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn list_tags(state: &AppState, project_id: Option<&str>) -> BiResult<Vec<(String, i64)>> {
    let conn = state.db.conn()?;
    let mut out: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut sql = String::from("SELECT tags FROM memories WHERE tags IS NOT NULL AND tags != '[]' AND tags != ''");
    let params: Vec<Box<dyn rusqlite::ToSql>> = match project_id {
        Some(p) => {
            sql.push_str(" AND project_id = ?1");
            vec![Box::new(p.to_string())]
        }
        None => Vec::new(),
    };
    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(refs.as_slice(), |r| r.get::<_, String>(0))?;
    for r in rows.flatten() {
        if let Ok(arr) = serde_json::from_str::<Vec<String>>(&r) {
            for t in arr {
                *out.entry(t).or_insert(0) += 1;
            }
        }
    }
    let mut v: Vec<(String, i64)> = out.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok(v)
}

fn row_to_memory(r: &rusqlite::Row<'_>) -> rusqlite::Result<Memory> {
    let tags_str: Option<String> = r.get(4)?;
    let tags: Vec<String> = tags_str
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    Ok(Memory {
        uid: r.get(0)?,
        project_id: r.get(1)?,
        mem_type: r.get(2)?,
        content: r.get(3)?,
        tags,
        source_agent: r.get(5)?,
        importance: r.get(6)?,
        supersedes: r.get(7)?,
        superseded_by: r.get(8)?,
        created_at: r.get(9)?,
        updated_at: r.get(10)?,
        last_access: r.get(11)?,
        access_count: r.get(12)?,
        file_path: r.get(13)?,
        start_line: r.get(14)?,
        end_line: r.get(15)?,
        language: r.get(16)?,
    })
}
