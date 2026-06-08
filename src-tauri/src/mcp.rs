use crate::error::{BiError, BiResult};
use crate::memory::{self, Memory, MemoryWithScore, RememberInput, UpdateInput};
use crate::project::{self, CreateProjectInput};
use crate::state::AppState;
use crate::{consolidate, ingest};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub async fn run_mcp_server_stdio() -> anyhow::Result<()> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("no data dir"))?
        .join("com.biturbo.app");
    std::fs::create_dir_all(&data_dir).ok();
    let state = Arc::new(AppState::open(&data_dir)?);

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 { break; }
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        let response = match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(req) => dispatch(&state, req).await,
            Err(e) => json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32700, "message": format!("parse error: {e}") }
            }),
        };
        let out = serde_json::to_string(&response).unwrap_or_default();
        stdout.write_all(out.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

async fn dispatch(state: &Arc<AppState>, req: JsonRpcRequest) -> Value {
    let id = req.id.clone().unwrap_or(Value::Null);
    match req.method.as_str() {
        "initialize" => ok(&id, json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "biTurbo", "version": env!("CARGO_PKG_VERSION") },
            "instructions": "biTurbo is a local-first memory layer. Use `search`/`recall_for_context` before answering. Use `remember` for facts/decisions/preferences/patterns/episodes/reflections. Use `ingest_project` to index a code directory. Always pass `project_id` to scope to the current project. Call `register_agent` once at startup."
        })),
        "notifications/initialized" => json!({}),
        "tools/list" => ok(&id, json!({ "tools": tool_schemas() })),
        "tools/call" => {
            let params = req.params;
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match call_tool(state, name, args).await {
                Ok(content) => ok(&id, json!({ "content": content, "isError": false })),
                Err(e) => ok(&id, json!({
                    "content": [{ "type": "text", "text": format!("error: {e}") }],
                    "isError": true
                })),
            }
        }
        "ping" => ok(&id, json!({})),
        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("method not found: {}", req.method) }
        }),
    }
}

fn ok(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

async fn call_tool(state: &Arc<AppState>, name: &str, args: Value) -> BiResult<Vec<Value>> {
    let text = |v: &str| vec![json!({ "type": "text", "text": v })];
    let result = match name {
        "remember" => {
            let input: RememberInput = serde_json::from_value(args)?;
            let m = memory::remember(state, input)?;
            text(&serde_json::to_string_pretty(&m)?)
        }
        "forget" => {
            let uid = arg_str(&args, "uid")?;
            let b = memory::forget(state, &uid)?;
            text(&serde_json::to_string_pretty(&json!({ "forgotten": b }))?)
        }
        "update" => {
            let uid = arg_str(&args, "uid")?;
            let input: UpdateInput = serde_json::from_value(args)?;
            let m = memory::update(state, &uid, input)?;
            text(&serde_json::to_string_pretty(&m)?)
        }
        "get_memory" => {
            let uid = arg_str(&args, "uid")?;
            let m = memory::get(state, &uid)?;
            text(&serde_json::to_string_pretty(&m)?)
        }
        "search" => {
            let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
            let query = arg_str(&args, "query")?;
            let k = args.get("k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            let mem_type = args.get("mem_type").and_then(|v| v.as_str());
            let hits: Vec<MemoryWithScore> = memory::search(state, project_id, &query, k, mem_type)?;
            text(&serde_json::to_string_pretty(&hits)?)
        }
        "list" => {
            let project_id = args.get("project_id").and_then(|v| v.as_str());
            let mem_type = args.get("mem_type").and_then(|v| v.as_str());
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
            let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let m: Vec<Memory> = memory::list(state, project_id, mem_type, limit, offset)?;
            text(&serde_json::to_string_pretty(&m)?)
        }
        "recall_for_context" => {
            let project_id = args.get("project_id").and_then(|v| v.as_str()).unwrap_or("");
            let query = arg_str(&args, "query")?;
            let k = args.get("k").and_then(|v| v.as_u64()).unwrap_or(8) as usize;
            let mem_type = args.get("mem_type").and_then(|v| v.as_str());
            let hits = memory::search(state, project_id, &query, k, mem_type)?;
            text(&format_context_block(&hits))
        }
        "list_projects" => text(&serde_json::to_string_pretty(&project::list(state)?)?),
        "get_project" => {
            let id = arg_str(&args, "id")?;
            let p = project::get(state, &id)?;
            text(&serde_json::to_string_pretty(&p)?)
        }
        "create_project" => {
            let name = arg_str(&args, "name")?;
            let input = CreateProjectInput {
                id: args.get("id").and_then(|v| v.as_str().map(String::from)),
                name,
                description: args.get("description").and_then(|v| v.as_str().map(String::from)),
                root_path: args.get("root_path").and_then(|v| v.as_str().map(String::from)),
                bit_width: args.get("bit_width").and_then(|v| v.as_u64()).map(|n| n as u8),
            };
            let p = project::create(state, input)?;
            text(&serde_json::to_string_pretty(&p)?)
        }
        "delete_project" => {
            let id = arg_str(&args, "id")?;
            project::delete(state, &id)?;
            text(&serde_json::to_string_pretty(&json!({ "deleted": id }))?)
        }
        "ingest_project" => {
            let project_id = arg_str(&args, "project_id")?;
            let root_path = arg_str(&args, "root_path")?;
            let r = ingest::ingest_project(state, &project_id, std::path::Path::new(&root_path))?;
            text(&serde_json::to_string_pretty(&r)?)
        }
        "get_project_graph" => {
            let project_id = arg_str(&args, "project_id")?;
            let g = ingest::get_project_graph(state, &project_id)?;
            text(&serde_json::to_string_pretty(&g)?)
        }
        "consolidate" => {
            let project_id = args.get("project_id").and_then(|v| v.as_str());
            let r = consolidate::consolidate(state, project_id)?;
            text(&serde_json::to_string_pretty(&r)?)
        }
        "stats" => {
            let conn = state.db.conn()?;
            let total_memories: i64 = conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))?;
            let total_projects: i64 = conn.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))?;
            let by_type = memory::count_by_type(state, None)?;
            let out = json!({ "total_memories": total_memories, "total_projects": total_projects, "by_type": by_type });
            text(&serde_json::to_string_pretty(&out)?)
        }
        "recent_activity" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as i64;
            let conn = state.db.conn()?;
            let mut s = conn.prepare("SELECT id, project_id, agent_id, action, memory_uid, detail, created_at FROM activity ORDER BY created_at DESC LIMIT ?1")?;
            let rows: Vec<Value> = s.query_map(rusqlite::params![limit], |r| {
                let detail_str: Option<String> = r.get(5)?;
                let detail = detail_str.as_deref().and_then(|s| serde_json::from_str::<Value>(s).ok());
                Ok(json!({
                    "id": r.get::<_, i64>(0)?,
                    "project_id": r.get::<_, Option<String>>(1)?,
                    "agent_id": r.get::<_, Option<String>>(2)?,
                    "action": r.get::<_, String>(3)?,
                    "memory_uid": r.get::<_, Option<String>>(4)?,
                    "detail": detail,
                    "created_at": r.get::<_, i64>(6)?,
                }))
            })?.filter_map(|r| r.ok()).collect();
            drop(s);
            text(&serde_json::to_string_pretty(&rows)?)
        }
        "register_agent" => {
            let name = arg_str(&args, "name")?;
            let kind = arg_str(&args, "kind")?;
            let meta = args.get("meta").cloned();
            let now = chrono::Utc::now().timestamp_millis();
            let id = slugify(&name);
            let meta_str = meta.as_ref().map(|v| v.to_string());
            state.db.write(|tx| {
                tx.execute(
                    "INSERT INTO agents(id, name, kind, last_seen, created_at, meta) VALUES(?1,?2,?3,?4,?4,?5) ON CONFLICT(id) DO UPDATE SET last_seen = excluded.last_seen, meta = COALESCE(excluded.meta, agents.meta)",
                    rusqlite::params![id, name, kind, now, meta_str],
                )?;
                Ok(())
            })?;
            text(&serde_json::to_string_pretty(&json!({ "id": id, "name": name, "kind": kind, "last_seen": now }))?)
        }
        other => return Err(BiError::Invalid(format!("unknown tool: {other}"))),
    };
    Ok(result)
}

fn arg_str(args: &Value, key: &str) -> BiResult<String> {
    args.get(key).and_then(|v| v.as_str()).map(String::from)
        .ok_or_else(|| BiError::Invalid(format!("missing string arg: {key}")))
}

fn format_context_block(hits: &[MemoryWithScore]) -> String {
    if hits.is_empty() { return "<biTurboContext>no relevant memories</biTurboContext>".into(); }
    let mut s = String::from("<biTurboContext>\n");
    for (i, h) in hits.iter().enumerate() {
        s.push_str(&format!("[{}] ({} · score={:.3} · importance={:.2})\n{}\n\n",
            i + 1, h.memory.mem_type, h.score, h.memory.importance, h.memory.content.trim()));
    }
    s.push_str("</biTurboContext>");
    s
}

fn slugify(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>().split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-")
}

fn tool_schemas() -> Value {
    serde_json::from_str(SCHEMAS_JSON).unwrap_or_else(|_| json!([]))
}

const SCHEMAS_JSON: &str = r#"[
{"name":"remember","description":"Store a memory. mem_type: fact|decision|preference|pattern|episode|reflection|code.","inputSchema":{"type":"object","required":["content"],"properties":{"content":{"type":"string"},"mem_type":{"type":"string"},"project_id":{"type":"string"},"tags":{"type":"array","items":{"type":"string"}},"importance":{"type":"number"},"source_agent":{"type":"string"},"supersedes":{"type":"string"}}}},
{"name":"forget","description":"Delete a memory by uid.","inputSchema":{"type":"object","required":["uid"],"properties":{"uid":{"type":"string"}}}},
{"name":"update","description":"Edit a memory. Any omitted field is unchanged.","inputSchema":{"type":"object","required":["uid"],"properties":{"uid":{"type":"string"},"content":{"type":"string"},"mem_type":{"type":"string"},"tags":{"type":"array","items":{"type":"string"}},"importance":{"type":"number"}}}},
{"name":"get_memory","description":"Fetch one memory by uid.","inputSchema":{"type":"object","required":["uid"],"properties":{"uid":{"type":"string"}}}},
{"name":"search","description":"Semantic search. project_id scopes to one project. mem_type filters. k=top-N (default 10).","inputSchema":{"type":"object","required":["query"],"properties":{"query":{"type":"string"},"project_id":{"type":"string"},"mem_type":{"type":"string"},"k":{"type":"number"}}}},
{"name":"list","description":"List memories with optional filters. Newest first. Default 50.","inputSchema":{"type":"object","properties":{"project_id":{"type":"string"},"mem_type":{"type":"string"},"limit":{"type":"number"},"offset":{"type":"number"}}}},
{"name":"recall_for_context","description":"Build a <biTurboContext> block of top-k relevant memories. Use before answering.","inputSchema":{"type":"object","required":["query"],"properties":{"query":{"type":"string"},"project_id":{"type":"string"},"mem_type":{"type":"string"},"k":{"type":"number"}}}},
{"name":"list_projects","description":"List all projects.","inputSchema":{"type":"object","properties":{}}},
{"name":"get_project","description":"Fetch one project by id.","inputSchema":{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}},
{"name":"create_project","description":"Create a new project.","inputSchema":{"type":"object","required":["name"],"properties":{"name":{"type":"string"},"id":{"type":"string"},"description":{"type":"string"},"root_path":{"type":"string"},"bit_width":{"type":"number"}}}},
{"name":"delete_project","description":"Delete a project and all its memories. 'default' cannot be deleted.","inputSchema":{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}},
{"name":"ingest_project","description":"Index a code directory via tree-sitter (rust/typescript/javascript/python/go).","inputSchema":{"type":"object","required":["project_id","root_path"],"properties":{"project_id":{"type":"string"},"root_path":{"type":"string"}}}},
{"name":"consolidate","description":"Run memory maintenance: decay, dedup (cosine >= 0.95), merge.","inputSchema":{"type":"object","properties":{"project_id":{"type":"string"}}}},
{"name":"stats","description":"Global stats.","inputSchema":{"type":"object","properties":{}}},
{"name":"recent_activity","description":"Recent activity entries.","inputSchema":{"type":"object","properties":{"limit":{"type":"number"}}}},
{"name":"register_agent","description":"Register or update this agent's record. Call once per session.","inputSchema":{"type":"object","required":["name","kind"],"properties":{"name":{"type":"string"},"kind":{"type":"string"},"meta":{"type":"object"}}}}
]"#;
