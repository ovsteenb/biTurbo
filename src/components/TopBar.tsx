import { Plus, Sparkles } from "lucide-react";
import { useApp } from "../lib/store";
import { useState } from "react";
import { api } from "../lib/api";

export function TopBar() {
  const setQuickAddOpen = useApp((s) => s.setQuickAddOpen);
  const view = useApp((s) => s.view);
  const stats = useApp((s) => s.stats);
  const currentProjectId = useApp((s) => s.currentProjectId);
  const projects = useApp((s) => s.projects);
  const refreshStats = useApp((s) => s.refreshStats);
  const refreshActivity = useApp((s) => s.refreshActivity);
  const showToast = useApp((s) => s.showToast);
  const [consolidating, setConsolidating] = useState(false);

  const currentProject = projects.find((p) => p.id === currentProjectId);

  async function runConsolidate() {
    setConsolidating(true);
    try {
      const r = await api.consolidate(currentProjectId);
      showToast({
        kind: "ok",
        text: `Consolidated · ${r.decayed} decayed · ${r.merged} merged · ${r.duplicates_found} dupes`,
      });
      await refreshStats();
      await refreshActivity();
    } catch (e) {
      showToast({ kind: "err", text: String(e) });
    } finally {
      setConsolidating(false);
    }
  }

  return (
    <header
      data-tauri-drag-region
      className="flex h-14 shrink-0 items-center gap-3 border-b border-border-subtle bg-bg/40 px-4 backdrop-blur"
    >
      {/* View title */}
      <div className="flex items-baseline gap-3 pl-16">
        <h1 className="font-serif text-lg font-medium capitalize text-text">
          {view}
        </h1>
        {currentProject && view !== "projects" && view !== "settings" && (
          <span className="font-mono text-[11px] text-text-dim">
            {currentProject.name}
          </span>
        )}
      </div>

      <div className="flex-1" />

      {/* Index size badge */}
      {stats && (
        <div className="hidden items-center gap-2 rounded-md border border-border-subtle bg-surface px-2.5 py-1 text-[11px] text-text-muted md:flex">
          <span className="font-mono">
            {(stats.index_bytes / 1024 / 1024).toFixed(2)} MB
          </span>
          <span className="text-text-dim">·</span>
          <span>
            {stats.total_memories.toLocaleString()} memories
          </span>
        </div>
      )}

      <button
        onClick={runConsolidate}
        disabled={consolidating}
        className="btn-ghost"
        title="Run decay + dedup + merge"
      >
        <Sparkles size={14} className={consolidating ? "animate-pulse" : ""} />
        <span className="hidden sm:inline">Consolidate</span>
      </button>

      <button
        onClick={() => setQuickAddOpen(true)}
        className="btn-primary"
        title="Quick add (⌘K)"
      >
        <Plus size={14} />
        <span>Remember</span>
        <span className="kbd ml-1">⌘K</span>
      </button>
    </header>
  );
}
