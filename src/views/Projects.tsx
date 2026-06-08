import { useEffect, useState } from "react";
import { useApp } from "../lib/store";
import { api } from "../lib/api";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { Plus, FolderGit2, Trash2, Database, FileSearch, Loader2 } from "lucide-react";
import clsx from "clsx";

interface IngestProgress {
  project_id: string;
  phase: string;
  current: number;
  total: number;
  file: string | null;
  chunks_so_far: number;
}

export function Projects() {
  const projects = useApp((s) => s.projects);
  const refreshProjects = useApp((s) => s.refreshProjects);
  const refreshStats = useApp((s) => s.refreshStats);
  const refreshGraph = useApp((s) => s.refreshGraph);
  const showToast = useApp((s) => s.showToast);
  const setCurrentProjectId = useApp((s) => s.setCurrentProjectId);
  const currentProjectId = useApp((s) => s.currentProjectId);

  const [creating, setCreating] = useState(false);
  const [name, setName] = useState("");
  const [desc, setDesc] = useState("");
  const [rootPath, setRootPath] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [progress, setProgress] = useState<IngestProgress | null>(null);

  useEffect(() => {
    const un = listen<IngestProgress>("ingest:progress", (e) => {
      setProgress(e.payload);
      if (e.payload.phase === "done") {
        setTimeout(() => setProgress(null), 1200);
      }
    });
    return () => {
      un.then((f) => f()).catch(() => {});
    };
  }, []);

  async function pickFolder() {
    const sel = await open({ directory: true, multiple: false });
    if (typeof sel === "string") setRootPath(sel);
  }

  async function create() {
    if (!name.trim()) return;
    setBusy("create");
    try {
      const p = await api.createProject({
        name: name.trim(),
        description: desc.trim() || undefined,
        root_path: rootPath.trim() || undefined,
      });
      await refreshProjects();
      await refreshStats();
      setCreating(false);
      setName("");
      setDesc("");
      setRootPath("");
      setCurrentProjectId(p.id);
      showToast({ kind: "ok", text: `Created project ${p.name}` });
    } catch (e) {
      showToast({ kind: "err", text: String(e) });
    } finally {
      setBusy(null);
    }
  }

  async function ingest(projectId: string, root: string) {
    if (!root) {
      showToast({ kind: "err", text: "Set a root_path first" });
      return;
    }
    setBusy(projectId);
    setProgress({
      project_id: projectId,
      phase: "scanning",
      current: 0,
      total: 0,
      file: null,
      chunks_so_far: 0,
    });
    try {
      const r = await api.ingestProject(projectId, root);
      await refreshProjects();
      await refreshStats();
      await refreshGraph().catch(() => {});
      showToast({
        kind: "ok",
        text: `Indexed ${r.files_indexed} files · ${r.chunks_indexed} chunks · ${r.edges_created} edges`,
      });
    } catch (e) {
      showToast({ kind: "err", text: String(e) });
    } finally {
      setBusy(null);
    }
  }

  async function remove(id: string) {
    if (!confirm(`Delete project "${id}" and all its memories?`)) return;
    setBusy(id);
    try {
      await api.deleteProject(id);
      await refreshProjects();
      await refreshStats();
      showToast({ kind: "ok", text: "Deleted" });
    } catch (e) {
      showToast({ kind: "err", text: String(e) });
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="mx-auto max-w-5xl space-y-6 p-8 animate-fade_in">
      <div className="flex items-baseline justify-between">
        <div>
          <h2 className="font-serif text-2xl">Projects</h2>
          <p className="mt-1 text-sm text-text-muted">
            Each project gets its own turbovec index, isolated memories, and a tree-sitter code map.
          </p>
        </div>
        <button onClick={() => setCreating(true)} className="btn-primary">
          <Plus size={14} /> New project
        </button>
      </div>

      {progress && (
        <div className="card p-4">
          <div className="mb-2 flex items-center gap-2 text-sm">
            <Loader2 size={14} className="animate-spin text-accent" />
            <span className="font-medium text-text">
              {progress.phase === "scanning" && "Scanning project…"}
              {progress.phase === "embedding" && "Embedding chunks…"}
              {progress.phase === "edges" && "Building edges…"}
              {progress.phase === "done" && "Done"}
            </span>
            {progress.total > 0 && progress.phase !== "done" && (
              <span className="ml-auto font-mono text-xs text-text-muted">
                {progress.current}/{progress.total} · {progress.chunks_so_far} chunks
              </span>
            )}
            {progress.phase === "done" && (
              <span className="ml-auto font-mono text-xs text-success">
                {progress.chunks_so_far} chunks indexed
              </span>
            )}
          </div>
          {progress.total > 0 && progress.phase !== "done" && (
            <div className="h-1.5 overflow-hidden rounded-full bg-surface-2">
              <div
                className="h-full bg-accent transition-all"
                style={{ width: `${Math.min(100, (progress.current / progress.total) * 100)}%` }}
              />
            </div>
          )}
          {progress.file && (
            <div className="mt-2 truncate font-mono text-[11px] text-text-dim">
              {progress.file}
            </div>
          )}
        </div>
      )}

      {creating && (
        <div className="card space-y-3 p-5">
          <h3 className="font-serif text-lg">New project</h3>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="mb-1 block text-[10px] uppercase tracking-widest text-text-dim">
                Name
              </label>
              <input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="scout-qa"
                className="input"
                autoFocus
              />
            </div>
            <div>
              <label className="mb-1 block text-[10px] uppercase tracking-widest text-text-dim">
                Description
              </label>
              <input
                value={desc}
                onChange={(e) => setDesc(e.target.value)}
                placeholder="Laravel rewrite of QA studio"
                className="input"
              />
            </div>
          </div>
          <div>
            <label className="mb-1 block text-[10px] uppercase tracking-widest text-text-dim">
              Root path (for code indexing)
            </label>
            <div className="flex gap-2">
              <input
                value={rootPath}
                onChange={(e) => setRootPath(e.target.value)}
                placeholder="/Users/you/Code/project"
                className="input font-mono"
              />
              <button onClick={pickFolder} className="btn-outline">
                Browse
              </button>
            </div>
          </div>
          <div className="flex items-center justify-end gap-2 border-t border-border-subtle pt-3">
            <button onClick={() => setCreating(false)} className="btn-ghost">
              Cancel
            </button>
            <button
              onClick={create}
              disabled={!name.trim() || busy === "create"}
              className="btn-primary"
            >
              {busy === "create" ? "Creating…" : "Create"}
            </button>
          </div>
        </div>
      )}

      <div className="grid gap-3">
        {projects.map((p) => {
          const active = p.id === currentProjectId;
          const isIngesting = busy === p.id;
          return (
            <div
              key={p.id}
              className={clsx(
                "card p-5 transition",
                active && "border-accent/40 bg-accent-soft/40"
              )}
            >
              <div className="flex items-start gap-4">
                <div
                  className={clsx(
                    "flex h-10 w-10 shrink-0 items-center justify-center rounded-md",
                    active ? "bg-accent/20 text-accent" : "bg-surface-2 text-text-muted"
                  )}
                >
                  <FolderGit2 size={18} />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-baseline gap-2">
                    <h3 className="font-serif text-lg text-text">{p.name}</h3>
                    {active && (
                      <span className="rounded-full border border-accent/30 bg-accent/10 px-1.5 py-0.5 text-[10px] text-accent">
                        active
                      </span>
                    )}
                    <span className="ml-auto font-mono text-[10px] text-text-dim">
                      {p.id}
                    </span>
                  </div>
                  {p.description && (
                    <p className="mt-0.5 text-sm text-text-muted">{p.description}</p>
                  )}
                  {p.root_path && (
                    <p className="mt-1 font-mono text-[11px] text-text-dim">
                      {p.root_path}
                    </p>
                  )}

                  <div className="mt-3 flex items-center gap-4 text-xs text-text-muted">
                    <span className="inline-flex items-center gap-1.5">
                      <Database size={11} />
                      <span className="font-mono">{p.memory_count}</span> memories
                    </span>
                    <span className="inline-flex items-center gap-1.5">
                      <FileSearch size={11} />
                      <span className="font-mono">{p.indexed_count}</span> code chunks
                    </span>
                    <span className="font-mono text-[10px] text-text-dim">
                      dim={p.dim} · {p.bit_width}-bit
                    </span>
                  </div>
                </div>
              </div>

              <div className="mt-4 flex items-center gap-2 border-t border-border-subtle pt-3">
                {!active && (
                  <button
                    onClick={() => setCurrentProjectId(p.id)}
                    className="btn-outline"
                  >
                    Switch to this
                  </button>
                )}
                {p.root_path && (
                  <button
                    onClick={() => ingest(p.id, p.root_path!)}
                    disabled={isIngesting}
                    className="btn-outline"
                  >
                    <FileSearch size={12} />
                    {isIngesting ? "Indexing…" : "Re-index code"}
                  </button>
                )}
                <div className="flex-1" />
                {p.id !== "default" && (
                  <button
                    onClick={() => remove(p.id)}
                    disabled={busy === p.id}
                    className="btn-ghost text-danger hover:bg-danger/10"
                  >
                    <Trash2 size={12} />
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
