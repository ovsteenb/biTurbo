# biTurbo

**Local-first memory layer for AI coding agents.** Pure-Rust Tauri 2 desktop app + stdio MCP server, built on Google's [TurboQuant](https://arxiv.org/abs/2504.19874) via the [turbovec](https://github.com/RyanCodrai/turbovec) crate (16× vector compression vs float32, MIT, beats FAISS on ARM).

Give your AI agents (Mavis, Claude Code, Cursor, Cline, anything MCP) persistent, project-scoped, semantic memory. Everything stays on disk. Search it, browse it, index whole codebases with tree-sitter, run decay / dedup / merge on a schedule.

```
┌─────────────────────────────────────────────────────────┐
│  biTurbo (single binary)                                │
│                                                          │
│   Tauri 2 GUI  ──┐                                       │
│                  │                                       │
│   MCP stdio  ────┼──→  AppState                          │
│                  │      ├── SQLite (metadata)            │
│   ingest  ───────┘      ├── turbovec IdMapIndex per proj │
│   (tree-sitter)         ├── fastembed (BGE-small ONNX)   │
│                         └── activity audit log           │
└─────────────────────────────────────────────────────────┘
```

## Why

- **Memory is the missing primitive for AI agents.** Every session = blank slate. biTurbo fixes that.
- **Local-first.** No cloud, no SaaS, no embedding leakage. Your context stays on your disk.
- **Multi-project.** Per-project indices with hybrid allowlist filters. testy memories don't pollute scout-qa.
- **Maximum compression.** turbovec 4-bit = 16× smaller than float32. A million memories fit in laptop RAM.
- **MCP-native.** 16 tools. Any agent that speaks MCP can read, write, search, ingest, consolidate.
- **Tree-sitter indexed code.** Drop a folder, get semantic code search. "Where is auth handled?"

## Quick start

Requires: `pnpm 11+`, `node 20+`, `rustc 1.77+`.

```bash
# 1. install deps
pnpm install

# 2. run desktop app (dev mode, hot-reload)
pnpm tauri:dev

# 3. in another shell — run MCP server standalone
pnpm mcp:dev
```

On first launch, the embedder downloads `BGE-small-en` (~30 MB) into your OS cache. Subsequent launches are instant.

## Project structure

```
biTurbo/
├── src/                          React + Vite + Tailwind frontend
│   ├── views/                    Overview · Memories · Projects · Agents · Settings
│   ├── components/               Sidebar · TopBar · MemoryCard · MemoryDetail · QuickAdd · Heatmap · Toast
│   └── lib/                      api.ts (Tauri invoke) · store.ts (zustand) · types.ts · format.ts
├── src-tauri/                    Rust backend
│   ├── src/
│   │   ├── main.rs               Tauri entry
│   │   ├── lib.rs                Tauri builder + IPC registration
│   │   ├── state.rs              AppState (shared, mpsc-free, behind parking_lot::RwLock)
│   │   ├── db.rs                 SQLite + r2d2 pool + schema
│   │   ├── index_engine.rs       turbovec IdMapIndex wrapper (1 file per project)
│   │   ├── embed.rs              fastembed (BGE-small-en default)
│   │   ├── memory.rs             CRUD + search + types
│   │   ├── project.rs            multi-project isolation
│   │   ├── ingest.rs             tree-sitter project walker (rust/ts/js/py/go)
│   │   ├── consolidate.rs        decay / dedup / merge
│   │   ├── mcp.rs                rmcp stdio server (16 tools)
│   │   └── commands.rs           Tauri IPC handlers
│   └── bin/biturbo_mcp.rs        Standalone MCP server binary
├── INSTRUCTIONS.md               Rules for AI agents using the MCP tools — read this!
├── README.md                     You are here.
└── pnpm-workspace.yaml           esbuild allowBuilds (pnpm 11 strict)
```

## MCP setup (for AI agents)

The standalone `biturbo-mcp` binary speaks MCP over stdio. Add it to your agent's MCP config:

```json
{
  "mcpServers": {
    "biturbo": {
      "command": "/Users/you/.cargo/bin/biturbo-mcp",
      "args": []
    }
  }
}
```

The first time an agent connects, it should:

1. Call `register_agent(name=..., kind=...)` so its writes are attributed.
2. Call `list_projects()` to see existing projects.
3. Call `recall_for_context(query, project_id, k=8)` before answering each turn.

See [INSTRUCTIONS.md](./INSTRUCTIONS.md) for the full ruleset, tool reference, and anti-patterns.

## MCP tools (16)

| | | |
|---|---|---|
| `remember` | `forget` | `update` |
| `get_memory` | `search` | `list` |
| `recall_for_context` | | |
| `list_projects` | `get_project` | `create_project` |
| `delete_project` | | |
| `ingest_project` | | |
| `consolidate` | `stats` | `recent_activity` |
| `register_agent` | | |

## Tech stack

| Layer | Choice | Why |
|---|---|---|
| Shell | Tauri 2 | Smaller binaries, native, webview UI, IPC via `invoke` |
| Frontend | React 18 + Vite + Tailwind 3 | Fast, mature, ergonomic |
| State | Zustand | Tiny, no boilerplate |
| Icons | lucide-react | Clean, consistent, tree-shakeable |
| Backend | Rust (1.77+) | Cold start < 50ms, single binary, no Python env |
| DB | SQLite + r2d2 + rusqlite | Local, WAL, zero-config |
| Vector | turbovec 0.8 (IdMapIndex, 4-bit) | 16× compression, beats FAISS, MIT |
| Embed | fastembed 4 (BGE-small-en ONNX) | No PyTorch, Metal/CPU, ~30MB model |
| MCP | rmcp 1.7 (official Rust SDK) | First-class stdio, macros, server handler |
| Tree-sitter | 0.25 + lang crates (rust/ts/js/py/go) | Per-function chunks, structural code search |

## Roadmap

- [ ] Watch-folder ingest (auto-reindex on file change)
- [ ] Cross-encoder re-ranker for top-k (optional, pluggable)
- [ ] Encrypted-at-rest mode (project-level key)
- [ ] Multi-device sync (CRDTs over the same on-disk format)
- [ ] Built-in chat view that calls an LLM with recalled context (optional, off by default)
- [ ] Web export of memories for sharing
- [ ] Memory diffing between projects

## License

MIT.
