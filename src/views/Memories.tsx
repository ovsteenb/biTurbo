import { useEffect, useMemo, useState } from "react";
import { useApp } from "../lib/store";
import { api } from "../lib/api";
import { MemoryCard } from "../components/MemoryCard";
import { MemoryDetail } from "../components/MemoryDetail";
import { Search, X, FileCode2, Hash } from "lucide-react";
import clsx from "clsx";

const TYPES = ["fact", "decision", "preference", "pattern", "episode", "reflection", "code"] as const;

export function Memories() {
  const memories = useApp((s) => s.memories);
  const selectedUid = useApp((s) => s.selectedMemoryUid);
  const setSelected = useApp((s) => s.setSelectedMemoryUid);
  const currentProjectId = useApp((s) => s.currentProjectId);

  const [query, setQuery] = useState("");
  const [searching, setSearching] = useState(false);
  const [results, setResults] = useState<typeof memories>([]);
  const [activeTypes, setActiveTypes] = useState<Set<string>>(new Set());
  const [activeTags, setActiveTags] = useState<Set<string>>(new Set());
  const [minImportance, setMinImportance] = useState(0);
  const [loadingMore, setLoadingMore] = useState(false);
  const hasMore = useApp((s) => s.hasMoreMemories);
  const loadMore = useApp((s) => s.loadMoreMemories);
  const tags = useApp((s) => s.tags);

  async function handleLoadMore() {
    if (loadingMore || !hasMore) return;
    setLoadingMore(true);
    try { await loadMore(); } finally { setLoadingMore(false); }
  }

  const selected = useMemo(
    () => memories.find((m) => m.uid === selectedUid) ?? results.find((m) => m.uid === selectedUid),
    [memories, results, selectedUid]
  );

  // Semantic search when query non-empty
  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      return;
    }
    let cancelled = false;
    (async () => {
      setSearching(true);
      try {
        const hits = await api.search({
          project_id: currentProjectId,
          query: query.trim(),
          k: 50,
        });
        if (!cancelled) setResults(hits);
      } finally {
        setSearching(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [query, currentProjectId]);

  const memoryOffset = useApp((s) => s.memoryOffset);

  const visible = useMemo(() => {
    const source = query.trim() ? results : memories;
    return source.filter((m) => {
      if (m.project_id !== currentProjectId) return false;
      if (activeTypes.size > 0 && !activeTypes.has(m.mem_type)) return false;
      if (activeTags.size > 0) {
        const hasAny = m.tags.some((t) => activeTags.has(t));
        if (!hasAny) return false;
      }
      if (m.importance < minImportance) return false;
      return true;
    });
  }, [query, results, memories, activeTypes, activeTags, minImportance, currentProjectId]);

  function toggleType(t: string) {
    const n = new Set(activeTypes);
    if (n.has(t)) n.delete(t);
    else n.add(t);
    setActiveTypes(n);
  }

  function toggleTag(t: string) {
    const n = new Set(activeTags);
    if (n.has(t)) n.delete(t);
    else n.add(t);
    setActiveTags(n);
  }

  const topTags = tags.slice(0, 20);

  return (
    <div className="flex h-full">
      {/* List column */}
      <div className="flex w-full flex-col overflow-hidden lg:w-[55%] xl:w-[60%]">
        {/* Filter bar */}
        <div className="flex flex-col gap-3 border-b border-border-subtle p-4">
          <div className="relative">
            <Search
              size={14}
              className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-text-dim"
            />
            <input
              id="global-search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search memories semantically… (or filter by tag below)"
              className="input pl-9 pr-9"
            />
            {query && (
              <button
                onClick={() => setQuery("")}
                className="absolute right-2 top-1/2 -translate-y-1/2 rounded p-1 text-text-dim hover:text-text"
              >
                <X size={13} />
              </button>
            )}
          </div>

          <div className="flex flex-wrap items-center gap-1.5">
            <span className="text-[10px] uppercase tracking-widest text-text-dim">
              type
            </span>
            {TYPES.map((t) => {
              const active = activeTypes.has(t);
              return (
                <button
                  key={t}
                  onClick={() => toggleType(t)}
                  className={clsx(
                    "rounded-full px-2.5 py-0.5 text-xs capitalize transition",
                    active
                      ? "bg-accent text-bg"
                      : "border border-border bg-surface-2 text-text-muted hover:text-text"
                  )}
                >
                  {t}
                </button>
              );
            })}

            {topTags.length > 0 && (
              <>
                <span className="ml-2 text-[10px] uppercase tracking-widest text-text-dim">
                  tag
                </span>
                {topTags.map(([t, n]) => {
                  const active = activeTags.has(t);
                  return (
                    <button
                      key={t}
                      onClick={() => toggleTag(t)}
                      title={`${n} memories`}
                      className={clsx(
                        "rounded-full px-2.5 py-0.5 text-xs transition",
                        active
                          ? "bg-violet-500/20 text-violet-200 ring-1 ring-violet-500/40"
                          : "border border-border bg-surface-2 text-text-muted hover:text-text"
                      )}
                    >
                      #{t}
                    </button>
                  );
                })}
              </>
            )}

            <div className="ml-auto flex items-center gap-2">
              <span className="text-[10px] uppercase tracking-widest text-text-dim">
                min importance
              </span>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={minImportance}
                onChange={(e) => setMinImportance(parseFloat(e.target.value))}
                className="w-24 accent-accent"
              />
              <span className="w-7 text-right font-mono text-[10px] text-text-muted">
                {minImportance.toFixed(2)}
              </span>
            </div>
          </div>
        </div>

        {/* List */}
        <div className="flex-1 overflow-y-auto p-4">
          {searching && (
            <div className="mb-3 text-xs text-text-dim">Searching…</div>
          )}
          {visible.length === 0 ? (
            <div className="flex h-64 flex-col items-center justify-center text-center text-text-dim">
              <FileCode2 size={24} className="mb-2 opacity-50" />
              <div className="text-sm">No memories match.</div>
              <div className="mt-1 text-xs">
                Press <span className="kbd">⌘K</span> to add one.
              </div>
            </div>
          ) : (
            <div className="space-y-2">
              {visible.map((m) => (
                <MemoryCard
                  key={m.uid}
                  memory={m}
                  active={selectedUid === m.uid}
                  onClick={() => setSelected(m.uid)}
                />
              ))}
              {!query && hasMore && (
                <button
                  onClick={handleLoadMore}
                  disabled={loadingMore}
                  className="mt-2 w-full rounded-md border border-border-subtle bg-surface px-3 py-2 text-xs text-text-muted transition hover:border-border hover:text-text disabled:opacity-50"
                >
                  {loadingMore ? "Loading…" : "Load 50 more"}
                </button>
              )}
            </div>
          )}
        </div>

        <div className="border-t border-border-subtle px-4 py-2 font-mono text-[10px] text-text-dim">
          {visible.length} {visible.length === 1 ? "memory" : "memories"}
          {query && results.length > 0 && ` · semantic search · k=${results.length}`}
          {!query && memories.length < 200 && (
            <> · page {Math.floor(memoryOffset / 50) || 1}</>
          )}
        </div>
      </div>

      {/* Detail column */}
      <div className="hidden w-[45%] border-l border-border-subtle bg-surface/30 lg:block xl:w-[40%]">
        {selected ? (
          <MemoryDetail memory={selected} onClose={() => setSelected(null)} />
        ) : (
          <div className="flex h-full flex-col items-center justify-center p-8 text-center text-text-dim">
            <Hash size={28} className="mb-3 opacity-30" />
            <div className="font-serif text-lg text-text-muted">Select a memory</div>
            <div className="mt-1 max-w-xs text-xs">
              Click any card on the left to inspect, edit, or forget.
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
