import type { Memory } from "../lib/types";
import { MEM_TYPE_META, timeAgo, importanceDots } from "../lib/format";
import { FileCode2, Hash } from "lucide-react";
import clsx from "clsx";

export function MemoryCard({
  memory,
  active,
  onClick,
}: {
  memory: Memory;
  active?: boolean;
  onClick?: () => void;
}) {
  const meta = MEM_TYPE_META[memory.mem_type] ?? MEM_TYPE_META.fact;
  const dots = importanceDots(memory.importance);
  const isCode = memory.mem_type === "code";

  return (
    <div onClick={onClick} className={clsx("memory-card", active && "active")}>
      {/* Top row: type badge + meta */}
      <div className="mb-2 flex items-center gap-2">
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
        {memory.superseded_by && (
          <span className="rounded-full border border-border-subtle px-1.5 py-0.5 text-[10px] text-text-dim">
            superseded
          </span>
        )}
        <span className="ml-auto font-mono text-[10px] text-text-dim">
          {timeAgo(memory.created_at)}
        </span>
      </div>

      {/* Content preview */}
      <div className="line-clamp-3 text-sm leading-relaxed text-text text-pretty">
        {memory.content}
      </div>

      {/* Code location */}
      {isCode && memory.file_path && (
        <div className="mt-2 flex items-center gap-1.5 font-mono text-[11px] text-orange-300/80">
          <FileCode2 size={11} />
          <span className="truncate">
            {memory.file_path.split("/").slice(-2).join("/")}
            {memory.start_line && `:${memory.start_line}`}
          </span>
        </div>
      )}

      {/* Footer: tags + importance + source */}
      <div className="mt-3 flex items-center gap-2 text-[11px] text-text-muted">
        {memory.tags.slice(0, 3).map((t) => (
          <span key={t} className="inline-flex items-center gap-1 text-text-dim">
            <Hash size={9} />
            {t}
          </span>
        ))}

        <div className="ml-auto flex items-center gap-2">
          {memory.source_agent && (
            <span className="font-mono text-[10px] text-text-dim">
              {memory.source_agent}
            </span>
          )}
          <div className="flex items-center gap-0.5" title={`importance ${memory.importance.toFixed(2)}`}>
            {Array.from({ length: 5 }).map((_, i) => (
              <span
                key={i}
                className={clsx(
                  "h-1 w-1 rounded-full",
                  i < dots ? "bg-accent" : "bg-text-dim/40"
                )}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
