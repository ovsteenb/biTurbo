import { useMemo, useState } from "react";
import { useApp } from "../lib/store";
import { Terminal, Folder, Cpu, Copy, Check, FileCode2, Sun, Moon } from "lucide-react";
import clsx from "clsx";

export function Settings() {
  const stats = useApp((s) => s.stats);
  const currentProjectId = useApp((s) => s.currentProjectId);
  const projects = useApp((s) => s.projects);
  const showToast = useApp((s) => s.showToast);
  const theme = useApp((s) => s.theme);
  const setTheme = useApp((s) => s.setTheme);
  const [copied, setCopied] = useState<string | null>(null);

  const project = projects.find((p) => p.id === currentProjectId);

  const projectRule = useMemo(() => {
    const pid = currentProjectId;
    const start = "\u003cbiturbo-rule-start project=\"" + pid + "\"\u003e";
    const end = "\u003c/biturbo-rule-start\u003e";
    return `${start}
## biTurbo memory rules for project "${pid}"

You have access to biTurbo, a persistent semantic memory layer via MCP.

### Core loop — follow this EVERY turn:

1. **RECALL** — call \`recall_for_context(query=<user msg>, project_id="${pid}", k=8)\`
2. **ANSWER** — respond using the recalled context
3. **REMEMBER** — call \`remember()\` after every response to store durable information

### When to \`remember\` (call AFTER EVERY response):

- User states a fact about themselves/environment/project → \`fact\`
- You make a decision with rationale → \`decision\`
- User expresses a preference (style, verbosity, tools) → \`preference\`
- User corrects you → \`fact\` with \`supersedes\`
- You discover a codebase pattern → \`pattern\`
- Something noteworthy happened → \`episode\` with timestamp
- Meta-observation about work → \`reflection\`
- ❌ Transient state — don't remember
- ❌ Public knowledge — don't remember
- ❌ Secrets, tokens, PII — NEVER

### Memory types:
\`fact\`, \`decision\`, \`preference\`, \`pattern\`, \`episode\`, \`reflection\`, \`code\` (auto)

### Importance (0-1):
- 0.8-1.0: cross-project rules, key decisions
- 0.5-0.7: typical (default 0.6)
- 0.2-0.4: specific/stale details

### Tags: 1-3 per memory. Good: \`auth\`, \`ui\`, \`db\`, \`api\`. Bad: \`important\`, \`todo\`.

### Session:
- START → \`register_agent(name, kind)\`, \`list_projects()\`
- EVERY TURN → recall → answer → remember
- END → \`consolidate(project_id)\`, final \`remember\`

### Anti-patterns:
- Don't dump 10k memories — use recall_for_context k=5-10
- Don't skip recall — amnesia is worse than no tool
- Don't cross-project leak — always pass project_id="${pid}"
- Never store secrets, tokens, PII
${end}`;
  }, [currentProjectId]);

  const globalRule = useMemo(() => {
    const start = "\u003cbiturbo-rule-start scope=\"global\"\u003e";
    const end = "\u003c/biturbo-rule-start\u003e";
    return `${start}
## biTurbo memory rules (global / cross-project)

You have access to biTurbo, a persistent semantic memory layer via MCP.
Project-agnostic preferences and cross-project facts live here.

### Core loop — follow this EVERY turn:

1. **RECALL** — call \`recall_for_context(query=<user msg>, project_id="default", k=8)\`
2. **ANSWER** — respond using the recalled context
3. **REMEMBER** — call \`remember()\` after every response to store durable information

### When to \`remember\` (call AFTER EVERY response):

- User states a cross-project fact → \`fact\`
- You make a decision with rationale → \`decision\`
- User expresses a preference → \`preference\`
- You discover a pattern → \`pattern\`
- Something noteworthy happened → \`episode\`

### Memory types:
\`fact\`, \`decision\`, \`preference\`, \`pattern\`, \`episode\`, \`reflection\`

### Importance (0-1):
- 0.8-1.0: life rules, key decisions
- 0.5-0.7: typical (default 0.6)

### Session:
- START → \`register_agent\`, \`list_projects()\`
- EVERY TURN → recall → answer → remember
- END → \`consolidate\`

### Anti-patterns:
- Don't skip recall
- When working in a project, scope memories with that project_id
- Never store secrets, tokens, PII
${end}`;
  }, []);

  function copy(label: string, text: string) {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(label);
      showToast({ kind: "ok", text: `Copied ${label}` });
      setTimeout(() => setCopied(null), 1500);
    });
  }

  const dataDir = "~/.local/share/com.biturbo.app (macOS: ~/Library/Application Support/com.biturbo.app)";

  const mcpConfig = `{
  "mcpServers": {
    "biturbo": {
      "command": "biturbo-mcp",
      "args": [],
      "env": {}
    }
  }
}`;

  return (
    <div className="mx-auto max-w-3xl space-y-8 p-8 animate-fade_in">
      <div>
        <h2 className="font-serif text-2xl">Settings</h2>
        <p className="mt-1 text-sm text-text-muted">
          Local data, MCP integration, and the embedding model.
        </p>
      </div>

      <Section icon={Folder} title="Data location">
        <p className="text-sm text-text-muted">
          Everything is stored locally. SQLite, turbovec indices, and the embedding model cache.
        </p>
        <pre className="mt-3 overflow-x-auto rounded-md border border-border-subtle bg-surface-2 p-3 font-mono text-xs text-text-muted">
          {dataDir}
        </pre>
      </Section>

      <Section icon={theme === "dark" ? Moon : Sun} title="Appearance">
        <p className="text-sm text-text-muted">
          Pick the interface theme. Saved per device.
        </p>
        <div className="mt-3 flex items-center gap-2">
          <button
            onClick={() => setTheme("dark")}
            className={clsx(
              "btn-outline",
              theme === "dark" && "border-accent/50 bg-accent-soft text-text"
            )}
          >
            <Moon size={13} /> Dark
          </button>
          <button
            onClick={() => setTheme("light")}
            className={clsx(
              "btn-outline",
              theme === "light" && "border-accent/50 bg-accent-soft text-text"
            )}
          >
            <Sun size={13} /> Light
          </button>
        </div>
      </Section>

      <Section icon={Cpu} title="Embedding model">
        <div className="space-y-2 text-sm text-text-muted">
          <Row k="Model" v="BGE-small-en (default)" />
          <Row k="Dimension" v="384" />
          <Row k="Backend" v="ONNX Runtime via fastembed" />
          <Row k="Index size" v={`${((stats?.index_bytes ?? 0) / 1024 / 1024).toFixed(2)} MB`} />
          <Row k="Quantization" v="turbovec 4-bit (16× compression vs float32)" />
        </div>
        <p className="mt-3 text-xs text-text-dim">
          To change the model, edit <span className="kbd">src-tauri/src/state.rs</span> →
          <span className="kbd ml-1">Embedder::new(...)</span>. Supported: BGE-small-en, BGE-base-en,
          BGE-large-en, BGE-M3, all-MiniLM-L6-v2.
        </p>
      </Section>

      <Section icon={Terminal} title="MCP server">
        <p className="text-sm text-text-muted">
          The standalone <span className="kbd">biturbo-mcp</span> binary speaks MCP over stdio.
          Add it to your agent's MCP config:
        </p>
        <pre className="mt-3 overflow-x-auto rounded-md border border-border-subtle bg-surface-2 p-3 font-mono text-xs text-text-muted">
{mcpConfig}
        </pre>
        <p className="mt-3 text-sm text-text-muted">
          Once connected, your agent has 16 tools (search, remember, forget, ingest_project,
          consolidate, list_projects, …). See <span className="kbd">INSTRUCTIONS.md</span> in the
          project root for the full tool reference and usage rules.
        </p>
      </Section>

      <Section icon={Terminal} title="What to read next">
        <ul className="ml-4 list-disc space-y-1 text-sm text-text-muted">
          <li><span className="kbd">README.md</span> — quick start + architecture</li>
          <li><span className="kbd">INSTRUCTIONS.md</span> — for AI agents using MCP</li>
          <li>Run <span className="kbd">pnpm tauri:dev</span> to launch the desktop app</li>
          <li>Run <span className="kbd">pnpm mcp:dev</span> to launch the MCP server standalone</li>
        </ul>
      </Section>

      <Section icon={FileCode2} title="Agent rule blocks">
        <p className="text-sm text-text-muted">
          Drop these into <span className="kbd">AGENTS.md</span>,{" "}
          <span className="kbd">CLAUDE.md</span>,{" "}
          <span className="kbd">.cursorrules</span>, or whatever your agent reads on
          startup. They'll wire your agent into biTurbo with the right MCP tool surface and
          behavior rules.
        </p>

        <div className="mt-4 space-y-4">
          <RuleBlock
            label={`project · ${currentProjectId}`}
            text={projectRule}
            copied={copied === "project"}
            onCopy={() => copy("project rule", projectRule)}
            hint={
              project
                ? `Paste in the root of your ${currentProjectId} repo.`
                : "No active project — defaults to your current selection."
            }
          />

          <RuleBlock
            label="global"
            text={globalRule}
            copied={copied === "global"}
            onCopy={() => copy("global rule", globalRule)}
            hint="Paste in your home AGENTS.md or wherever your agent reads cross-project rules."
          />
        </div>
      </Section>
    </div>
  );
}

function Section({
  icon: Icon,
  title,
  children,
}: {
  icon: import("lucide-react").LucideIcon;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="card p-5">
      <div className="mb-3 flex items-center gap-2 text-text-muted">
        <Icon size={14} className="text-accent" />
        <h3 className="font-serif text-lg text-text">{title}</h3>
      </div>
      {children}
    </div>
  );
}

function Row({ k, v }: { k: string; v: string }) {
  return (
    <div className="flex items-center justify-between border-b border-border-subtle py-1.5 last:border-0">
      <span className="text-text-dim">{k}</span>
      <span className="font-mono text-text">{v}</span>
    </div>
  );
}

function RuleBlock({
  label,
  text,
  copied,
  onCopy,
  hint,
}: {
  label: string;
  text: string;
  copied: boolean;
  onCopy: () => void;
  hint?: string;
}) {
  return (
    <div className="rounded-md border border-border-subtle bg-surface-2 p-3">
      <div className="mb-2 flex items-center gap-2">
        <span className="font-mono text-[11px] uppercase tracking-widest text-accent">
          {label}
        </span>
        {hint && <span className="text-[10px] text-text-dim">{hint}</span>}
        <div className="flex-1" />
        <button
          onClick={onCopy}
          className="btn-outline text-xs"
        >
          {copied ? <Check size={12} className="text-success" /> : <Copy size={12} />}
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre className="overflow-x-auto rounded bg-bg p-3 font-mono text-[11px] leading-relaxed text-text-muted">
{text}
      </pre>
    </div>
  );
}
