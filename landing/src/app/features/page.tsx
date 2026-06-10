import { Nav } from "@/components/Nav";
import { Footer } from "@/components/Footer";
import { Marquee } from "@/components/Marquee";
import { CTASection } from "@/components/CTASection";
import { MemoryVisual } from "@/components/visuals/MemoryVisual";
import { MCPVisual } from "@/components/visuals/MCPVisual";
import { GraphVisual } from "@/components/visuals/GraphVisual";
import { SpeedVisual } from "@/components/visuals/SpeedVisual";
import { OSSVisual } from "@/components/visuals/OSSVisual";

type Tool = {
  name: string;
  category: "memory" | "search" | "project" | "system";
  signature: string;
  desc: string;
  example: string;
};

const tools: Tool[] = [
  // memory
  { name: "remember", category: "memory", signature: "remember(content, project_id, kind?, importance?)", desc: "Persist a memory. Kind can be decision, pattern, gotcha, context, fact. Importance is 0..1 and decays over time unless reinforced.", example: 'remember("SQLite WAL allows concurrent reads during write", "testy", kind="decision", importance=0.9)' },
  { name: "forget", category: "memory", signature: "forget(memory_id, reason?)", desc: "Delete a memory. Soft-delete by default; hard-delete with reason. Logs the deletion event in the activity audit.", example: 'forget("mem_abc123", reason="outdated after WAL migration")' },
  { name: "update", category: "memory", signature: "update(memory_id, content?, importance?, tags?)", desc: "Patch any field of a memory. Updating content re-embeds automatically. Importance changes propagate to the recall ranker.", example: 'update("mem_abc123", importance=0.95, tags=["wal", "concurrency"])' },
  { name: "get_memory", category: "memory", signature: "get_memory(memory_id)", desc: "Fetch the full record of a single memory by id, including metadata, tags, timestamps, and the originating agent.", example: 'get_memory("mem_abc123")' },
  // search
  { name: "search", category: "search", signature: "search(query, project_id, k?, filters?)", desc: "Hybrid semantic + lexical search over a project. Filters: kind, tag, importance_min, time_range. Returns top-k with scores.", example: 'search("agent auth flow", "scout-qa", k=8, filters={"kind": "pattern"})' },
  { name: "list", category: "search", signature: "list(project_id, filters?)", desc: "List memories in a project with optional filters and pagination. No semantic scoring — fast, deterministic.", example: 'list("testy", filters={"tag": "gotcha", "limit": 50})' },
  { name: "list_tags", category: "search", signature: "list_tags(project_id)", desc: "Enumerate all tags in a project with usage counts. Useful for the agent to discover its own vocabulary before searching.", example: 'list_tags("testy")' },
  { name: "recall_for_context", category: "search", signature: "recall_for_context(query, project_id, k?)", desc: "The hot path. Returns a formatted <biTurboContext> block ready to inject as system context. Use this before every non-trivial answer.", example: 'recall_for_context("why is the layout broken on mobile", "testy", k=4)' },
  // project
  { name: "list_projects", category: "project", signature: "list_projects()", desc: "Discover all projects on disk. Returns id, name, vector count, last activity. Usually the agent&apos;s first call after register_agent.", example: 'list_projects()' },
  { name: "get_project", category: "project", signature: "get_project(project_id)", desc: "Fetch a single project record with full stats: memory count, vector size, ingest path, last consolidate run.", example: 'get_project("prj_testy")' },
  { name: "create_project", category: "project", signature: "create_project(name, path?)", desc: "Create a new isolated project. Path optionally binds it to a code root for auto-ingest. Index starts empty and warms on first remember().", example: 'create_project("testy", path="/Users/.../testy")' },
  { name: "delete_project", category: "project", signature: "delete_project(project_id, confirm?)", desc: "Delete a project and all its memories, vectors, audit logs. Requires confirm=true as a safety net for agents.", example: 'delete_project("prj_old", confirm=true)' },
  { name: "ingest_project", category: "project", signature: "ingest_project(project_id, path, langs?)", desc: "Walk a code root with tree-sitter, chunk per function, embed each chunk, and add as context-kind memories. Default langs: rust, ts, js, py, go.", example: 'ingest_project("prj_testy", "/Users/.../testy", langs=["rust", "ts"])' },
  { name: "consolidate", category: "project", signature: "consolidate(project_id, mode?)", desc: "Manually trigger decay/dedup/merge. mode can be 'decay', 'dedup', 'merge', or 'all'. Runs synchronously by default; async with mode='all' for big indexes.", example: 'consolidate("testy", mode="all")' },
  { name: "consolidate_status", category: "project", signature: "consolidate_status(project_id)", desc: "Inspect the last consolidate run: timestamp, memories removed/merged, scheduler next run, decay config in effect.", example: 'consolidate_status("testy")' },
  // system
  { name: "stats", category: "system", signature: "stats(scope?)", desc: "System-wide or per-project stats. scope can be 'global' or a project_id. Returns memory counts, vector sizes, recall latency p50/p95.", example: 'stats(scope="global")' },
  { name: "bootstrap", category: "system", signature: "bootstrap()", desc: "First-run helper. Returns the recommended INSTRUCTIONS.md block for the agent kind (Claude Code, Cursor, Cline, Mavis).", example: 'bootstrap()' },
  { name: "recent_activity", category: "system", signature: "recent_activity(project_id?, n?)", desc: "Stream of the last n writes (or all) for a project. Used to show the agent what its peers are doing.", example: 'recent_activity("testy", n=20)' },
  { name: "register_agent", category: "system", signature: "register_agent(name, kind)", desc: "Claim an agent identity. All subsequent writes are attributed. Kind can be claude-code, cursor, cline, mavis, or a custom string.", example: 'register_agent(name="claude-opus-4.5", kind="claude-code")' },
];

const categoryColors: Record<Tool["category"], string> = {
  memory: "moss",
  search: "sky",
  project: "amber",
  system: "lilac",
};

export default function FeaturesPage() {
  return (
    <main className="relative">
      <Nav />

      {/* Hero */}
      <section className="relative min-h-[80vh] overflow-hidden pt-32">
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute left-1/3 top-1/3 h-[500px] w-[500px] rounded-full bg-moss/15 blur-[120px]" />
          <div className="absolute right-1/3 top-1/4 h-[400px] w-[400px] rounded-full bg-amber/15 blur-[120px]" />
        </div>
        <div className="grid-lines pointer-events-none absolute inset-0 opacity-30" />

        <div className="relative z-10 mx-auto max-w-7xl px-6 pb-20">
          <div className="flex items-center gap-3">
            <span className="chip">
              <span className="h-1.5 w-1.5 rounded-full bg-moss" />
              The full feature deep-dive
            </span>
            <span className="font-mono text-xs text-ink-300">
              Last updated · 2025
            </span>
          </div>

          <h1 className="mt-8 max-w-5xl font-display text-[clamp(3rem,9vw,8rem)] font-extrabold leading-[0.85] tracking-[-0.04em] text-ink">
            Every feature,<br />
            <span className="text-ink-300">explained.</span>
          </h1>

          <p className="mt-8 max-w-2xl text-balance text-xl text-ink-200/80 md:text-2xl">
            Five feature areas, nineteen MCP tools, one Rust binary. This page is the
            engineering reference — what each piece does, how it works, and the code
            signature you actually call.
          </p>

          <div className="mt-10 flex flex-wrap items-center gap-2">
            <a href="#memory" className="chip hover:border-moss/40">01 · Memory</a>
            <a href="#mcp" className="chip hover:border-sky/40">02 · MCP</a>
            <a href="#graph" className="chip hover:border-lilac/40">03 · Graph</a>
            <a href="#speed" className="chip hover:border-amber/40">04 · Speed</a>
            <a href="#oss" className="chip hover:border-lilac/40">05 · Open Source</a>
            <a href="#tools" className="chip hover:border-ink-200/40">→ 19 tools reference</a>
          </div>
        </div>
      </section>

      <Marquee />

      {/* Feature 01 — Memory deep dive */}
      <FeatureSection
        id="memory"
        index="01"
        eyebrow="memory"
        title={<>Your agent&apos;s hippocampus,<br /><span className="text-moss">in SQLite.</span></>}
        intro="Memory is the core. biTurbo stores every remembered fact in SQLite, embeds it locally with BGE-small-en, and indexes it with turbovec. Per-project isolation means testy's decisions stay out of scout-qa's recall results — and the index never sends a byte to the cloud."
        variant="moss"
        visual={<MemoryVisual />}
        columns={[
          {
            title: "Storage layer",
            bullets: [
              "SQLite with WAL mode, r2d2 connection pool",
              "Memories table with kind, importance, tags, project_id, agent_id, timestamps",
              "Per-project turbovec IdMapIndex — one file per project, never shared",
              "Activity audit log for every write/delete, queryable for debugging",
            ],
          },
          {
            title: "Memory kinds",
            bullets: [
              "decision — architectural or product choices",
              "pattern — recurring solutions in this codebase",
              "gotcha — things that broke before and will break again",
              "context — current state, in-flight work, env specifics",
              "fact — verified truths about the project",
            ],
          },
          {
            title: "Self-maintenance",
            bullets: [
              "Scheduled decay: importance * 0.95 per day, with a floor at 0.05",
              "Dedup: cosine sim > 0.96 with same kind → merge into the higher-importance one",
              "Merge: near-duplicates get a new embedding that's the mean",
              "Configurable decay/dedup per project via INSTRUCTIONS.md rules",
            ],
          },
        ]}
      />

      {/* Feature 02 — MCP deep dive */}
      <FeatureSection
        id="mcp"
        index="02"
        eyebrow="mcp"
        title={<>The protocol your agent<br /><span className="text-sky">already speaks.</span></>}
        intro="MCP is the universal adapter between an LLM and a tool. biTurbo exposes 19 of them — every operation your agent needs, from 'remember this' to 'recite everything relevant to this question' — over a single stdio socket. No HTTP, no auth, no proxy."
        variant="sky"
        align="right"
        visual={<MCPVisual />}
        columns={[
          {
            title: "How it boots",
            bullets: [
              "Standalone biturbo-mcp binary, spawned by your agent's MCP config",
              "Speaks stdio JSON-RPC — the official rmcp 1.7 Rust SDK",
              "First call: register_agent + list_projects (auto-bootstrapped)",
              "Every subsequent call scoped to a project_id",
            ],
          },
          {
            title: "Why stdio, not HTTP",
            bullets: [
              "Zero auth surface — the OS process boundary is the only access control",
              "No port to bind, no TLS to misconfigure, no firewall to argue with",
              "Works in any environment Claude Code / Cursor / Cline / Mavis can spawn a process in",
              "Latency: one pipe roundtrip per call, no HTTP overhead",
            ],
          },
          {
            title: "The hot path",
            bullets: [
              "recall_for_context(query, project_id, k) returns a <biTurboContext> block",
              "Pre-formatted for direct injection as system message or system prompt",
              "Hybrid ranking: cosine sim + tag match + importance + recency",
              "Average latency: < 2ms for k=8 on a 10k memory project",
            ],
          },
        ]}
      />

      {/* Feature 03 — Graph deep dive */}
      <FeatureSection
        id="graph"
        index="03"
        eyebrow="graph"
        title={<>A force-directed map<br /><span className="text-lilac">of your codebase.</span></>}
        intro="Drop a folder on a project. biTurbo walks it with tree-sitter, chunks per function, embeds each chunk, and renders a Barnes-Hut force layout in a Web Worker. 3,000+ nodes, 8,000+ edges, viewport-culled, with filter switches that cancel stale layout requests."
        variant="lilac"
        visual={<GraphVisual />}
        columns={[
          {
            title: "Code ingest",
            bullets: [
              "tree-sitter 0.25 with language crates: rust, ts, js, py, go",
              "Per-function chunks (not whole files) for tight semantic search",
              "Respects .gitignore; configurable include/exclude globs",
              "Re-ingest on demand; future: watch-folder with debounce",
            ],
          },
          {
            title: "Layout engine",
            bullets: [
              "Barnes-Hut n-body approximation — O(n log n) instead of O(n²)",
              "Runs in a dedicated Web Worker, off the main thread",
              "Seed renders in < 5ms; worker refines in 200–800ms for 3k nodes",
              "Filter switches cancel the active layout request via AbortController",
            ],
          },
          {
            title: "Interaction model",
            bullets: [
              "Click a node → its neighborhood highlights, sidebar opens with chunks",
              "Right-click a node → context menu: open in editor, search memories, focus",
              "Pan/zoom with momentum; viewport culling keeps it at 60fps",
              "Saved view states per project, shareable as a URL",
            ],
          },
        ]}
      />

      {/* Feature 04 — Speed deep dive */}
      <FeatureSection
        id="speed"
        index="04"
        eyebrow="speed"
        title={<>Sub-50ms cold start.<br /><span className="text-amber">Sub-2ms recall.</span></>}
        intro="biTurbo is fast because every layer of the stack is fast. The Rust binary is ~12MB, links zero Python, and cold-starts in under 50ms. The vector index is turbovec 4-bit — 16× smaller than float32, with recall parity you can actually measure."
        variant="amber"
        align="right"
        visual={<SpeedVisual />}
        columns={[
          {
            title: "Binary & startup",
            bullets: [
              "Pure Rust 1.77+, no Python runtime, no Docker, no JVM",
              "Single ~12MB binary; release build with LTO + strip + codegen-units=1",
              "Cold start < 50ms on M1, < 80ms on Intel",
              "Idle RAM: ~25MB. Scales linearly with vector count, not corpus size",
            ],
          },
          {
            title: "Vector compression",
            bullets: [
              "turbovec 0.8 IdMapIndex, 4-bit product quantisation",
              "16× smaller than float32 — 1M BGE-small-en vectors fit in 24MB",
              "Recall@10 within 0.5% of float32 on MS MARCO and BEIR subsets",
              "MIT licensed, beats FAISS-IVFPQ on the recall-per-byte curve",
            ],
          },
          {
            title: "Embedding model",
            bullets: [
              "fastembed 4 with BGE-small-en (~30MB ONNX, downloaded on first launch)",
              "CPU and Metal (Apple Silicon) backends; CUDA optional",
              "Per-agent override possible via INSTRUCTIONS.md (e.g. bge-large for accuracy)",
              "Embeddings cached by content hash; re-embed is a no-op if nothing changed",
            ],
          },
        ]}
      />

      {/* Feature 05 — Open Source deep dive */}
      <FeatureSection
        id="oss"
        index="05"
        eyebrow="open source"
        title={<>MIT. Forever.<br /><span className="text-lilac">No pro tier, ever.</span></>}
        intro="biTurbo is, and will always be, MIT licensed. There is no enterprise edition. There is no usage-based pricing. The whole codebase is on GitHub — Rust backend, React frontend, MCP server, smoke test, the docs. Fork it, vendor it, ship it in your own product. We just ask for a star."
        variant="lilac"
        visual={<OSSVisual />}
        columns={[
          {
            title: "What's in the repo",
            bullets: [
              "src/ — React 18 + Vite + Tailwind frontend (6 views, 5 primitives)",
              "src-tauri/src/ — Rust backend (db, index_engine, embed, memory, project, ingest, consolidate, mcp, scheduler, commands)",
              "src-tauri/bin/biturbo_mcp.rs — standalone MCP server",
              "scripts/mcp-smoke-test.ts — 19-tool validator (~2s end-to-end)",
            ],
          },
          {
            title: "Contributing",
            bullets: [
              "Issues and PRs welcome on GitHub",
              "No CLA — sign-off is the DCO (git commit -s)",
              "CI runs the smoke test on every PR (coming soon — Homebrew tap first)",
              "Roadmap and RFCs in the issue tracker",
            ],
          },
          {
            title: "Roadmap (next)",
            bullets: [
              "Watch-folder ingest — auto-reindex on file change",
              "Cross-encoder re-ranker for top-k (pluggable, opt-in)",
              "Encrypted-at-rest mode (project-level key, Argon2id-derived)",
              "Multi-device sync (CRDTs over the same on-disk format)",
            ],
          },
        ]}
      />

      {/* 19 Tools reference */}
      <section id="tools" className="relative border-t border-ink-200/10 py-32">
        <div className="mx-auto max-w-7xl px-6">
          <div className="mb-12 max-w-2xl">
            <span className="font-mono text-xs uppercase tracking-[0.2em] text-moss">
              § tools reference
            </span>
            <h2 className="mt-3 font-display text-5xl font-extrabold leading-[0.95] tracking-[-0.03em] text-ink md:text-7xl">
              The 19 MCP tools.
            </h2>
            <p className="mt-4 text-pretty text-lg text-ink-200/70">
              Every tool is a JSON-RPC method. Schemas below match the actual rmcp
              generated bindings — copy-paste safe.
            </p>
          </div>

          {/* Category legend */}
          <div className="mb-8 flex flex-wrap items-center gap-3">
            <span className="font-mono text-[10px] uppercase tracking-wider text-ink-300">categories:</span>
            {(["memory", "search", "project", "system"] as const).map((c) => (
              <span key={c} className={`chip text-${categoryColors[c]}`}>
                <span className={`h-1.5 w-1.5 rounded-full bg-${categoryColors[c]}`} />
                {c}
              </span>
            ))}
          </div>

          <div className="space-y-3">
            {tools.map((tool, i) => (
              <div
                key={tool.name}
                className="group rounded-xl border border-ink-200/10 bg-ink-200/[0.02] p-5 transition-colors hover:border-ink-200/20 hover:bg-ink-200/[0.04]"
              >
                <div className="grid grid-cols-1 gap-4 md:grid-cols-12 md:items-start">
                  <div className="md:col-span-3">
                    <div className="flex items-center gap-2">
                      <span className={`h-1.5 w-1.5 rounded-full bg-${categoryColors[tool.category]}`} />
                      <span className={`font-mono text-[10px] uppercase tracking-wider text-${categoryColors[tool.category]}`}>
                        {tool.category}
                      </span>
                    </div>
                    <h3 className="mt-1 font-display text-xl font-bold text-ink">
                      {tool.name}
                    </h3>
                    <code className="mt-1 block break-all font-mono text-[10px] text-ink-300">
                      {tool.signature}
                    </code>
                  </div>
                  <div className="md:col-span-6">
                    <p className="text-sm leading-relaxed text-ink-200/80">
                      {tool.desc}
                    </p>
                  </div>
                  <div className="md:col-span-3">
                    <div className="font-mono text-[9px] uppercase tracking-wider text-ink-300">example</div>
                    <pre className="mt-1 overflow-x-auto rounded-md border border-ink-200/10 bg-ink-900/60 p-2.5 font-mono text-[10px] leading-relaxed text-moss">
                      {tool.example}
                    </pre>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      <CTASection />

      <Footer />
    </main>
  );
}

type Column = { title: string; bullets: string[] };

function FeatureSection({
  id,
  index,
  eyebrow,
  title,
  intro,
  variant,
  align = "left",
  visual,
  columns,
}: {
  id: string;
  index: string;
  eyebrow: string;
  title: React.ReactNode;
  intro: string;
  variant: "moss" | "amber" | "sky" | "lilac";
  align?: "left" | "right";
  visual: React.ReactNode;
  columns: Column[];
}) {
  const variantText: Record<typeof variant, string> = {
    moss: "text-moss",
    amber: "text-amber",
    sky: "text-sky",
    lilac: "text-lilac",
  };
  const variantBorder: Record<typeof variant, string> = {
    moss: "border-moss/30",
    amber: "border-amber/30",
    sky: "border-sky/30",
    lilac: "border-lilac/30",
  };

  return (
    <section id={id} className="relative border-t border-ink-200/10 py-32">
      <div className="pointer-events-none absolute inset-0">
        <div
          className={
            variant === "moss" ? "gradient-radial-moss" :
            variant === "amber" ? "gradient-radial-amber" :
            variant === "sky" ? "gradient-radial-sky" :
            "gradient-radial-lilac"
          }
        />
      </div>

      <div className="relative z-10 mx-auto max-w-7xl px-6">
        <div className="mb-16 grid grid-cols-1 gap-12 lg:grid-cols-12 lg:items-end">
          <div className={`lg:col-span-7 ${align === "right" ? "lg:order-2" : ""}`}>
            <div className="mb-4 flex items-center gap-3">
              <span className={`font-mono text-xs uppercase tracking-[0.2em] ${variantText[variant]}`}>
                {index} · {eyebrow}
              </span>
              <div className={`h-px flex-1 bg-gradient-to-r from-current to-transparent opacity-30 ${variantText[variant]}`} />
            </div>
            <h2 className="font-display text-5xl font-extrabold leading-[0.95] tracking-[-0.03em] text-ink md:text-7xl text-balance">
              {title}
            </h2>
          </div>
          <div className={`lg:col-span-5 ${align === "right" ? "lg:order-1" : ""}`}>
            <p className="text-pretty text-lg text-ink-200/80 md:text-xl">
              {intro}
            </p>
          </div>
        </div>

        {/* Visual + columns side-by-side */}
        <div className="mb-12 grid grid-cols-1 gap-8 lg:grid-cols-12">
          <div
            className={`lg:col-span-7 overflow-hidden rounded-2xl border bg-ink-800/40 ${variantBorder[variant]} ${
              align === "right" ? "lg:order-2" : ""
            }`}
          >
            {visual}
          </div>
          <div className={`lg:col-span-5 ${align === "right" ? "lg:order-1" : ""}`}>
            <div className="grid grid-cols-1 gap-4">
              {columns.slice(0, 1).map((col) => (
                <div key={col.title} className={`rounded-2xl border bg-ink-200/[0.02] p-6 ${variantBorder[variant]}`}>
                  <h3 className="font-display text-lg font-bold text-ink">{col.title}</h3>
                  <ul className="mt-3 space-y-2">
                    {col.bullets.map((b, i) => (
                      <li key={i} className="flex gap-2 text-sm leading-relaxed text-ink-200/80">
                        <span className={`mt-1.5 h-1 w-1 flex-shrink-0 rounded-full bg-${categoryColors[Object.keys(categoryColors)[0] as Tool["category"]] ?? "moss"}`} />
                        <span>{b}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Remaining columns full-width */}
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
          {columns.slice(1).map((col) => (
            <div key={col.title} className={`rounded-2xl border bg-ink-200/[0.02] p-6 ${variantBorder[variant]}`}>
              <h3 className="font-display text-lg font-bold text-ink">{col.title}</h3>
              <ul className="mt-3 space-y-2">
                {col.bullets.map((b, i) => (
                  <li key={i} className="flex gap-2 text-sm leading-relaxed text-ink-200/80">
                    <span className={`mt-1.5 h-1 w-1 flex-shrink-0 rounded-full ${variantText[variant].replace("text-", "bg-")}`} />
                    <span>{b}</span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
