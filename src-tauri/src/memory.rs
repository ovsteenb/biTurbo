use crate::db::log_activity;
use crate::error::{BiError, BiResult};
use crate::state::AppState;
use rusqlite::OptionalExtension;
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
    #[allow(clippy::should_implement_trait)]
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
    let mem_type = MemType::from_str(input.mem_type.as_deref().unwrap_or("fact"))?
        .as_str()
        .to_string();
    let importance = input.importance.unwrap_or(0.5).clamp(0.0, 1.0);
    let uid = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let tags_json = serde_json::to_string(&input.tags.clone().unwrap_or_default())?;

    state.db.write(|tx| {
        let supersedes_id: Option<i64> = match input.supersedes.as_deref() {
            Some(old_uid) => tx
                .query_row(
                    "SELECT id FROM memories WHERE uid = ?1 AND project_id = ?2 AND superseded_by IS NULL",
                    rusqlite::params![old_uid, project_id],
                    |r| r.get(0),
                )
                .optional()?,
            None => None,
        };
        tx.execute(
            "INSERT INTO memories(uid, project_id, mem_type, content, tags, source_agent,
                                  importance, created_at, updated_at, last_access,
                                  access_count, file_path, start_line, end_line, language, supersedes)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8,?8,0,?9,?10,?11,?12,?13)",
            rusqlite::params![
                uid,
                project_id,
                mem_type,
                input.content,
                tags_json,
                input.source_agent,
                importance,
                now,
                input.file_path,
                input.start_line,
                input.end_line,
                input.language,
                supersedes_id,
            ],
        )?;
        let new_id = tx.last_insert_rowid();
        if let Some(old_id) = supersedes_id {
            tx.execute(
                "UPDATE memories
                 SET superseded_by = ?1, updated_at = ?2
                 WHERE id = ?3 AND superseded_by IS NULL",
                rusqlite::params![new_id, now, old_id],
            )?;
        }
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

    // Marks the index dirty; the background flusher in AppState persists it.
    state.embed_and_add(&project_id, &uid, &input.content)?;

    get(state, &uid)?.ok_or_else(|| BiError::Internal("memory not found post-insert".into()))
}

pub fn get(state: &AppState, uid: &str) -> BiResult<Option<Memory>> {
    let conn = state.db.conn()?;
    let mut stmt = conn.prepare_cached(
        "SELECT uid, project_id, mem_type, content, tags, source_agent, importance,
                supersedes, superseded_by, created_at, updated_at, last_access,
                access_count, file_path, start_line, end_line, language
         FROM memories WHERE uid = ?1",
    )?;
    match stmt.query_row(rusqlite::params![uid], row_to_memory) {
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
        tx.execute(
            "DELETE FROM memories WHERE uid = ?1",
            rusqlite::params![uid],
        )?;
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
    let new_content = input
        .content
        .clone()
        .unwrap_or_else(|| existing.content.clone());
    let new_type = match input.mem_type.clone() {
        Some(mem_type) => MemType::from_str(&mem_type)?.as_str().to_string(),
        None => existing.mem_type.clone(),
    };
    let new_tags_json = match input.tags.clone() {
        Some(t) => serde_json::to_string(&t)?,
        None => serde_json::to_string(&existing.tags)?,
    };
    let new_imp = input
        .importance
        .unwrap_or(existing.importance)
        .clamp(0.0, 1.0);

    state.db.write(|tx| {
        tx.execute(
            "UPDATE memories SET content = ?1, mem_type = ?2, tags = ?3,
                                 importance = ?4, updated_at = ?5
             WHERE uid = ?6",
            rusqlite::params![new_content, new_type, new_tags_json, new_imp, now, uid],
        )?;
        log_activity(
            tx,
            Some(&existing.project_id),
            None,
            "update",
            Some(uid),
            None,
        )?;
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
    let k = k.clamp(1, 100);
    if let Some(mem_type) = mem_type {
        MemType::from_str(mem_type)?;
    }
    let project_id = if project_id.is_empty() {
        state.default_project_id.clone()
    } else {
        project_id.to_string()
    };

    let kk = (k * 3).max(30);
    let conn = state.db.conn()?;

    let allowlist_uids: Option<Vec<String>> = if let Some(t) = mem_type {
        let mut stmt = conn.prepare_cached(
            "SELECT uid FROM memories WHERE project_id = ?1 AND mem_type = ?2 AND superseded_by IS NULL",
        )?;
        let rows = stmt.query_map(rusqlite::params![project_id, t], |r| r.get::<_, String>(0))?;
        Some(rows.filter_map(|r| r.ok()).collect())
    } else {
        None
    };

    let vec_hits = state.embed_and_search(&project_id, query, kk, allowlist_uids.as_deref())?;
    let fts_uids = fts_search(&conn, query, &project_id, mem_type, kk)?;

    let mut fused: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    const RRF_K: f32 = 60.0;
    for (rank, h) in vec_hits.iter().enumerate() {
        let rank_score = 1.0 / (RRF_K + rank as f32 + 1.0);
        let sim = h.score.clamp(0.0, 1.0);
        let score = rank_score * (0.5 + 0.5 * sim);
        *fused.entry(h.uid.clone()).or_insert(0.0) += score;
    }
    for (rank, (uid, _bm25)) in fts_uids.iter().enumerate() {
        let score = 1.0 / (RRF_K + rank as f32 + 1.0);
        *fused.entry(uid.clone()).or_insert(0.0) += score;
    }

    let mut ranked: Vec<(String, f32)> = fused.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    // Keep more candidates for reranking (k*4 gives reranker more to work with).
    ranked.truncate(k.saturating_mul(4).max(k));

    if ranked.is_empty() {
        return Ok(Vec::new());
    }

    let n = ranked.len();
    let placeholders = std::iter::repeat_n("?", n).collect::<Vec<_>>().join(",");
    let select_sql = format!(
        "SELECT uid, project_id, mem_type, content, tags, source_agent, importance,
                supersedes, superseded_by, created_at, updated_at, last_access,
                access_count, file_path, start_line, end_line, language
         FROM memories
         WHERE uid IN ({placeholders}) AND superseded_by IS NULL"
    );
    let mut stmt = conn.prepare_cached(&select_sql)?;
    let mut by_uid: std::collections::HashMap<String, Memory> = stmt
        .query_map(
            rusqlite::params_from_iter(ranked.iter().map(|(u, _)| u.as_str())),
            row_to_memory,
        )?
        .filter_map(|r| r.ok())
        .map(|m| (m.uid.clone(), m))
        .collect();
    drop(stmt);
    drop(conn);

    // Bookkeeping in one transaction: bump access stats for all hits and log a
    // single activity row for the whole search (not one per hit).
    let now = chrono::Utc::now().timestamp_millis();
    let hit_uids: Vec<String> = ranked
        .iter()
        .filter_map(|(u, _)| by_uid.get(u).map(|_| u.clone()))
        .collect();
    let top_uids: Vec<String> = hit_uids.iter().take(5).cloned().collect();
    if !hit_uids.is_empty() {
        let placeholders = std::iter::repeat_n("?", hit_uids.len())
            .collect::<Vec<_>>()
            .join(",");
        state.db.write(|tx| {
            let update_sql = format!(
                "UPDATE memories SET access_count = access_count + 1, last_access = ? WHERE uid IN ({placeholders})"
            );
            let mut upd = tx.prepare_cached(&update_sql)?;
            upd.execute(rusqlite::params_from_iter(
                std::iter::once(rusqlite::types::Value::Integer(now)).chain(
                    hit_uids
                        .iter()
                        .map(|u| rusqlite::types::Value::Text(u.clone())),
                ),
            ))?;
            log_activity(
                tx,
                Some(&project_id),
                None,
                "read",
                None,
                Some(&serde_json::json!({"query": query, "hits": hit_uids.len(), "top_uids": top_uids})),
            )?;
            Ok(())
        })?;
    }

    // Second-stage reranking: boost scores based on term matches in content, tags, path, and language
    let query_terms = tokenize_query(query);
    let mut reranked: Vec<MemoryWithScore> = ranked
        .into_iter()
        .enumerate()
        .filter_map(|(rank, (uid, base_score))| {
            by_uid.remove(&uid).map(|memory| {
                let reranked_score = rerank_memory_score(base_score, &memory, &query_terms, rank);
                MemoryWithScore {
                    memory,
                    score: reranked_score,
                }
            })
        })
        .collect();

    // Re-sort by reranked score
    reranked.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(reranked.into_iter().take(k).collect())
}

fn fts_search(
    conn: &rusqlite::Connection,
    query: &str,
    project_id: &str,
    mem_type: Option<&str>,
    limit: usize,
) -> BiResult<Vec<(String, f64)>> {
    let kk_i64 = limit as i64;
    let or_query = sanitize_fts_query(query, FtsCombine::Or);
    if !or_query.is_empty() {
        if let Ok(hits) = run_fts_query(conn, &or_query, project_id, mem_type, kk_i64) {
            if !hits.is_empty() {
                return Ok(hits);
            }
        }
    }
    let and_query = sanitize_fts_query(query, FtsCombine::And);
    if and_query.is_empty() {
        return Ok(Vec::new());
    }
    run_fts_query(conn, &and_query, project_id, mem_type, kk_i64)
}

fn run_fts_query(
    conn: &rusqlite::Connection,
    fts_query: &str,
    project_id: &str,
    mem_type: Option<&str>,
    limit: i64,
) -> BiResult<Vec<(String, f64)>> {
    let mut out = Vec::new();
    let row_map = |r: &rusqlite::Row<'_>| -> rusqlite::Result<(String, f64)> {
        Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
    };
    if let Some(t) = mem_type {
        let mut stmt = conn.prepare_cached(
            "SELECT m.uid, bm25(memories_fts)
             FROM memories_fts
             JOIN memories m ON m.uid = memories_fts.uid
             WHERE memories_fts MATCH ?1 AND m.mem_type = ?2 AND m.project_id = ?3
               AND m.superseded_by IS NULL
             ORDER BY bm25(memories_fts) ASC LIMIT ?4",
        )?;
        let rows = stmt.query_map(rusqlite::params![fts_query, t, project_id, limit], row_map)?;
        for r in rows.flatten() {
            out.push(r);
        }
    } else {
        let mut stmt = conn.prepare_cached(
            "SELECT m.uid, bm25(memories_fts)
             FROM memories_fts
             JOIN memories m ON m.uid = memories_fts.uid
             WHERE memories_fts MATCH ?1 AND m.project_id = ?2
               AND m.superseded_by IS NULL
             ORDER BY bm25(memories_fts) ASC LIMIT ?3",
        )?;
        let rows = stmt.query_map(rusqlite::params![fts_query, project_id, limit], row_map)?;
        for r in rows.flatten() {
            out.push(r);
        }
    }
    Ok(out)
}

#[derive(Clone, Copy)]
enum FtsCombine {
    And,
    Or,
}

fn sanitize_fts_query(q: &str, combine: FtsCombine) -> String {
    let tokens: Vec<String> = q
        .split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-'))
        .filter(|t| !t.is_empty() && t.len() >= 2)
        .map(|t| {
            let safe = t.replace('"', "");
            format!("\"{safe}\"*")
        })
        .collect();
    if tokens.is_empty() {
        return String::new();
    }
    let sep = match combine {
        FtsCombine::And => " ",
        FtsCombine::Or => " OR ",
    };
    tokens.join(sep)
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
            "SELECT mem_type, COUNT(*) FROM memories WHERE project_id = ?1 GROUP BY mem_type"
                .to_string(),
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
    let mut sql = String::from(
        "SELECT tags FROM memories WHERE tags IS NOT NULL AND tags != '[]' AND tags != ''",
    );
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

/// Tokenize a query into normalized search terms.
fn tokenize_query(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .filter(|t| t.len() >= 2)
        .collect()
}

/// Filter out common stopwords from query terms
fn filter_stopwords(terms: &[String]) -> Vec<String> {
    const STOPWORDS: &[&str] = &[
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
        "by", "from", "up", "about", "into", "through", "during", "before", "after", "above",
        "below", "between", "among", "is", "was", "are", "were", "been", "be", "have", "has",
        "had", "do", "does", "did", "will", "would", "could", "should", "may", "might", "must",
        "shall", "can", "need", "dare", "ought", "used", "how", "what", "when", "where", "why",
        "which", "who", "whom", "whose", "this", "that", "these", "those", "i", "you", "he",
        "she", "it", "we", "they", "me", "him", "her", "us", "them", "my", "your", "his", "its",
        "our", "their", "myself", "yourself", "himself", "herself", "itself", "ourselves",
        "themselves", "all", "each", "every", "both", "few", "more", "most", "other", "some",
        "such", "no", "nor", "not", "only", "own", "same", "so", "than", "too", "very",
    ];

    terms
        .iter()
        .filter(|term| !STOPWORDS.contains(&term.as_str()))
        .cloned()
        .collect()
}

/// Second-stage reranker: boost score based on term matches in content, tags, path, and language.
/// Uses multiplicative boosts to refine rather than override the base RRF ranking.
fn rerank_memory_score(base_score: f32, mem: &Memory, terms: &[String], _rank: usize) -> f32 {
    let filtered_terms = filter_stopwords(terms);
    if filtered_terms.is_empty() {
        return base_score;
    }

    let mut boost_factor = 1.0;

    // Content match boost (multiplicative boost per match)
    let content_lower = mem.content.to_lowercase();
    let content_matches = filtered_terms
        .iter()
        .filter(|t| content_lower.contains(t.as_str()))
        .count();
    if content_matches > 0 {
        boost_factor *= 1.0 + (content_matches as f32 * 0.12);
    }

    // Tag match boost (bidirectional: query term in tag OR tag in query term)
    let tag_matches = filtered_terms
        .iter()
        .filter(|term| {
            mem.tags.iter().any(|tag| {
                let tag_lower = tag.to_lowercase();
                tag_lower.contains(term.as_str()) || term.contains(&tag_lower)
            })
        })
        .count();
    if tag_matches > 0 {
        boost_factor *= 1.0 + (tag_matches as f32 * 0.20);
    }

    // Path match boost (multiplicative boost per match)
    if let Some(path) = &mem.file_path {
        let path_lower = path.to_lowercase();
        let path_matches = filtered_terms
            .iter()
            .filter(|t| path_lower.contains(t.as_str()))
            .count();
        if path_matches > 0 {
            boost_factor *= 1.0 + (path_matches as f32 * 0.15);
        }
    }

    // Language match boost (small boost if language matches query)
    if let Some(lang) = &mem.language {
        let lang_lower = lang.to_lowercase();
        if filtered_terms
            .iter()
            .any(|t| lang_lower.contains(t.as_str()))
        {
            boost_factor *= 1.08;
        }
    }

    // Importance boost (small additive factor)
    let importance_boost = mem.importance * 0.05;

    base_score * boost_factor + importance_boost
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

#[cfg(test)]
mod search_tests {
    use super::*;

    #[test]
    fn fts_or_query_matches_any_token() {
        let q = sanitize_fts_query("hybrid search turbovec", FtsCombine::Or);
        assert!(q.contains(" OR "));
        assert!(q.contains("\"hybrid\"*"));
    }

    #[test]
    fn fts_skips_single_char_tokens() {
        let q = sanitize_fts_query("a MCP server", FtsCombine::Or);
        assert!(!q.contains("\"a\"*"));
        assert!(q.contains("\"MCP\"*"));
    }

    #[test]
    fn fts_query_strips_quotes_and_punctuation() {
        let q = sanitize_fts_query(r#"auth: "login-flow"!"#, FtsCombine::Or);
        assert!(q.contains("\"auth\"*"));
        assert!(q.contains("\"login-flow\"*"));
        assert!(!q.contains("!"));
    }
}
