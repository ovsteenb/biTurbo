import { useEffect, useState } from "react";
import type { Memory } from "../lib/types";
import { MEM_TYPE_META, timeAgo, shortDate, importanceDots } from "../lib/format";
import { api } from "../lib/api";
import { useApp } from "../lib/store";
import { X, Trash2, Edit3, Save, FileCode2, Hash } from "lucide-react";
import clsx from "clsx";

export function MemoryDetail({ memory, onClose }: { memory: Memory; onClose: () => void }) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(memory.content);
  const [draftTags, setDraftTags] = useState(memory.tags.join(", "));
  const [draftImp, setDraftImp] = useState(memory.importance);
  const [related, setRelated] = useState<{ uid: string; content: string; score: number }[]>([]);
  const refreshMemories = useApp((s) => s.refreshMemories);
  const refreshStats = useApp((s) => s.refreshStats);
  const showToast = useApp((s) => s.showToast);
  const setSelected = useApp((s) => s.setSelectedMemoryUid);

  useEffect(() => {
    setDraft(memory.content);
    setDraftTags(memory.tags.join(", "));
    setDraftImp(memory.importance);
    setEditing(false);
  }, [memory.uid]);

  useEffect(() => {
    (async () => {
      try {
        const hits = await api.search({
          project_id: memory.project_id,
          query: memory.content.slice(0, 200),
          k: 6,
        });
        setRelated(
          hits
            .filter((h) => h.uid !== memory.uid)
            .slice(0, 5)
            .map((h) => ({ uid: h.uid, content: h.content, score: h.score }))
        );
      } catch {
        /* ignore */
      }
    })();
  }, [memory.uid, memory.project_id, memory.content]);

  async function save() {
    try {
      await api.update(memory.uid, {
        content: draft,
        tags: draftTags
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean),
        importance: draftImp,
      });
      await refreshMemories();
      showToast({ kind: "ok", text: "Saved" });
      setEditing(false);
    } catch (e) {
      showToast({ kind: "err", text: String(e) });
    }
  }

  async function forget() {
    if (!confirm("Forget this memory? It will be removed from the vector index too.")) return;
    try {
      await api.forget(memory.uid);
      setSelected(null);
      await refreshMemories();
      await refreshStats();
      showToast({ kind: "ok", text: "Forgotten" });
    } catch (e) {
      showToast({ kind: "err", text: String(e) });
    }
  }

  const meta = MEM_TYPE_META[memory.mem_type] ?? MEM_TYPE_META.fact;
  const dots = importanceDots(memory.importance);

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-start gap-2 border-b border-border-subtle p-4">
        <div className="flex-1">
          <div className="mb-1 flex items-center gap-2">
            <span
              className={clsx(
                "inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-[10px] font-medium uppercase tracking-wider",
                meta.bg,
                meta.color
              )}
            >
              <span className={clsx("h-1 w-1 rounded-full", meta.dot)} />
              {meta.label}
            </span>
            <span className="font-mono text-[10px] text-text-dim">
              {memory.uid.slice(0, 8)}
            </span>
          </div>
        </div>
        <button onClick={onClose} className="btn-ghost p-1.5">
          <X size={14} />
        </button>
      </div>

      {/* Body */}
      <div className="flex-1 overflow-y-auto p-4">
        {editing ? (
          <textarea
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            rows={8}
            className="input resize-none font-sans text-sm"
            autoFocus
          />
        ) : (
          <div className="whitespace-pre-wrap text-sm leading-relaxed text-text text-pretty">
            {memory.content}
          </div>
        )}

        {/* Code location */}
        {memory.mem_type === "code" && memory.file_path && (
          <div className="mt-3 flex items-center gap-2 rounded-md border border-orange-500/20 bg-orange-500/5 p-2.5 font-mono text-[11px] text-orange-200">
            <FileCode2 size={12} />
            <span className="truncate">
              {memory.file_path}:{memory.start_line}-{memory.end_line}
            </span>
            {memory.language && (
              <span className="ml-auto rounded border border-orange-500/30 px-1.5 py-0.5 text-[10px]">
                {memory.language}
              </span>
            )}
          </div>
        )}

        {/* Tags */}
        <div className="mt-4">
          <div className="mb-1.5 text-[10px] uppercase tracking-widest text-text-dim">
            Tags
          </div>
          {editing ? (
            <input
              value={draftTags}
              onChange={(e) => setDraftTags(e.target.value)}
              className="input"
              placeholder="comma-separated"
            />
          ) : (
            <div className="flex flex-wrap gap-1.5">
              {memory.tags.length === 0 && (
                <span className="text-xs text-text-dim">—</span>
              )}
              {memory.tags.map((t) => (
                <span key={t} className="chip">
                  <Hash size={9} />
                  {t}
                </span>
              ))}
            </div>
          )}
        </div>

        {/* Importance slider */}
        <div className="mt-4">
          <div className="mb-1.5 flex items-center justify-between text-[10px] uppercase tracking-widest text-text-dim">
            <span>Importance</span>
            <span className="font-mono text-text-muted">
              {editing ? draftImp.toFixed(2) : memory.importance.toFixed(2)}
            </span>
          </div>
          {editing ? (
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={draftImp}
              onChange={(e) => setDraftImp(parseFloat(e.target.value))}
              className="w-full accent-accent"
            />
          ) : (
            <div className="flex items-center gap-0.5">
              {Array.from({ length: 5 }).map((_, i) => (
                <span
                  key={i}
                  className={clsx(
                    "h-1.5 w-6 rounded-full",
                    i < dots ? "bg-accent" : "bg-surface-2"
                  )}
                />
              ))}
            </div>
          )}
        </div>

        {/* Metadata grid */}
        <div className="mt-5 grid grid-cols-2 gap-3 border-t border-border-subtle pt-4 text-xs">
          <Meta label="Project" value={memory.project_id} mono />
          <Meta label="Source" value={memory.source_agent ?? "—"} mono />
          <Meta label="Created" value={shortDate(memory.created_at)} mono />
          <Meta label="Updated" value={shortDate(memory.updated_at)} mono />
          <Meta label="Accesses" value={String(memory.access_count)} mono />
          <Meta
            label="Last access"
            value={memory.last_access ? timeAgo(memory.last_access) : "—"}
          />
        </div>

        {/* Related */}
        {related.length > 0 && (
          <div className="mt-5 border-t border-border-subtle pt-4">
            <div className="mb-2 text-[10px] uppercase tracking-widest text-text-dim">
              Related
            </div>
            <div className="space-y-1.5">
              {related.map((r) => (
                <button
                  key={r.uid}
                  onClick={() => setSelected(r.uid)}
                  className="block w-full rounded-md border border-border-subtle bg-surface p-2 text-left text-[11px] text-text-muted transition hover:border-border hover:bg-surface-2"
                >
                  <div className="line-clamp-2 text-pretty">{r.content}</div>
                  <div className="mt-1 font-mono text-[10px] text-text-dim">
                    score {r.score.toFixed(3)}
                  </div>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Footer actions */}
      <div className="flex items-center gap-2 border-t border-border-subtle p-3">
        {editing ? (
          <>
            <button onClick={save} className="btn-primary flex-1">
              <Save size={14} /> Save
            </button>
            <button onClick={() => setEditing(false)} className="btn-outline">
              Cancel
            </button>
          </>
        ) : (
          <>
            <button onClick={() => setEditing(true)} className="btn-outline flex-1">
              <Edit3 size={14} /> Edit
            </button>
            <button onClick={forget} className="btn-outline text-danger hover:bg-danger/10">
              <Trash2 size={14} /> Forget
            </button>
          </>
        )}
      </div>
    </div>
  );
}

function Meta({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div>
      <div className="text-[10px] uppercase tracking-widest text-text-dim">
        {label}
      </div>
      <div className={clsx("mt-0.5 text-text-muted", mono && "font-mono")}>
        {value}
      </div>
    </div>
  );
}
