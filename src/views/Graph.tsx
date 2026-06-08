import { useEffect, useMemo, useRef, useState } from "react";
import { useApp } from "../lib/store";
import type { GraphNode, GraphEdge } from "../lib/types";
import { Share2, Filter, Search, ZoomIn, ZoomOut, RefreshCw } from "lucide-react";

type Pos = { x: number; y: number; vx: number; vy: number };

const KIND_COLORS: Record<string, { fill: string; stroke: string }> = {
  file: { fill: "#D4A574", stroke: "#8a6b4a" },
  function: { fill: "#7DC4E4", stroke: "#3d7488" },
  class: { fill: "#C7A0E0", stroke: "#6d4d8a" },
  struct: { fill: "#E0C58C", stroke: "#8a7434" },
  module: { fill: "#8FB87D", stroke: "#4d6e44" },
};

const EDGE_COLORS: Record<string, string> = {
  member_of: "#3a342d",
  imports: "#D4A574",
  calls: "#7DC4E4",
  extends: "#C7A0E0",
};

export function Graph() {
  const graph = useApp((s) => s.graph);
  const refreshGraph = useApp((s) => s.refreshGraph);
  const currentProjectId = useApp((s) => s.currentProjectId);
  const showToast = useApp((s) => s.showToast);
  const setSelected = useApp((s) => s.setSelectedMemoryUid);
  const [filter, setFilter] = useState<Set<string>>(
    new Set(["file", "function", "class", "struct"])
  );
  const [query, setQuery] = useState("");
  const [hover, setHover] = useState<string | null>(null);
  const [posMap, setPosMap] = useState<Record<string, Pos>>({});
  const [view, setView] = useState({ x: 0, y: 0, k: 1 });
  const [busy, setBusy] = useState(false);
  const svgRef = useRef<SVGSVGElement>(null);
  const dragRef = useRef<{ x: number; y: number; vx: number; vy: number } | null>(null);

  // Re-layout when graph data changes.
  const data = useMemo(() => {
    if (!graph) return null;
    const visibleNodes = graph.nodes.filter((n) => filter.has(n.kind));
    const visibleIds = new Set(visibleNodes.map((n) => n.uid));
    const visibleEdges = graph.edges.filter(
      (e) => visibleIds.has(e.from) && visibleIds.has(e.to)
    );
    return { nodes: visibleNodes, edges: visibleEdges };
  }, [graph, filter]);

  useEffect(() => {
    if (!data || data.nodes.length === 0) return;
    setPosMap(layout(data.nodes, data.edges));
  }, [data]);

  async function reload() {
    setBusy(true);
    try {
      await refreshGraph();
      showToast({ kind: "ok", text: "Graph refreshed" });
    } catch (e) {
      showToast({ kind: "err", text: String(e) });
    } finally {
      setBusy(false);
    }
  }

  if (!graph) {
    return (
      <div className="flex h-full items-center justify-center p-12 text-center">
        <div>
          <Share2 size={28} className="mx-auto mb-3 text-text-dim" />
          <div className="font-serif text-lg">No graph for this project yet.</div>
          <div className="mt-1 text-sm text-text-muted">
            Run <span className="kbd">ingest_project</span> to build the index, then refresh.
          </div>
          <button onClick={reload} className="btn-primary mt-4">
            <RefreshCw size={14} className={busy ? "animate-spin" : ""} />
            Build graph
          </button>
        </div>
      </div>
    );
  }

  const matches = useMemo(() => {
    if (!query.trim() || !data) return new Set<string>();
    const q = query.toLowerCase();
    return new Set(
      data.nodes
        .filter(
          (n) =>
            n.label.toLowerCase().includes(q) ||
            (n.file_path ?? "").toLowerCase().includes(q)
        )
        .map((n) => n.uid)
    );
  }, [query, data]);

  const hoverNode = hover && data ? data.nodes.find((n) => n.uid === hover) : null;
  const connectedToHover = useMemo(() => {
    if (!hover || !data) return new Set<string>();
    const s = new Set<string>([hover]);
    for (const e of data.edges) {
      if (e.from === hover) s.add(e.to);
      if (e.to === hover) s.add(e.from);
    }
    return s;
  }, [hover, data]);

  function nodeRadius(n: GraphNode): number {
    if (n.kind === "file") return 10 + Math.min(8, Math.sqrt(n.size));
    return 4 + Math.min(6, Math.log2(n.size + 1) * 1.5);
  }

  function onWheel(e: React.WheelEvent) {
    e.preventDefault();
    const factor = e.deltaY < 0 ? 1.1 : 0.9;
    setView((v) => ({ ...v, k: Math.max(0.2, Math.min(4, v.k * factor)) }));
  }

  function onMouseDown(e: React.MouseEvent) {
    if ((e.target as Element).tagName !== "svg" && (e.target as Element).tagName !== "rect") {
      return;
    }
    dragRef.current = { x: e.clientX, y: e.clientY, vx: view.x, vy: view.y };
  }

  function onMouseMove(e: React.MouseEvent) {
    if (!dragRef.current) return;
    const dx = e.clientX - dragRef.current.x;
    const dy = e.clientY - dragRef.current.y;
    setView((v) => ({ ...v, x: dragRef.current!.vx + dx, y: dragRef.current!.vy + dy }));
  }

  function onMouseUp() {
    dragRef.current = null;
  }

  function resetView() {
    setView({ x: 0, y: 0, k: 1 });
  }

  return (
    <div className="flex h-full">
      {/* Canvas */}
      <div className="relative flex-1 overflow-hidden bg-bg">
        <svg
          ref={svgRef}
          className="h-full w-full cursor-grab active:cursor-grabbing"
          onWheel={onWheel}
          onMouseDown={onMouseDown}
          onMouseMove={onMouseMove}
          onMouseUp={onMouseUp}
          onMouseLeave={onMouseUp}
        >
          <defs>
            {Object.entries(EDGE_COLORS).map(([k, c]) => (
              <marker
                key={k}
                id={`arrow-${k}`}
                viewBox="0 0 10 10"
                refX="9"
                refY="5"
                markerWidth="5"
                markerHeight="5"
                orient="auto"
              >
                <path d="M0,0 L10,5 L0,10 Z" fill={c} opacity="0.7" />
              </marker>
            ))}
          </defs>
          <g transform={`translate(${view.x},${view.y}) scale(${view.k})`}>
            <rect
              x={-10000}
              y={-10000}
              width={20000}
              height={20000}
              fill="transparent"
            />
            {data?.edges.map((e, i) => {
              const a = posMap[e.from];
              const b = posMap[e.to];
              if (!a || !b) return null;
              const isHighlighted = hover && (e.from === hover || e.to === hover);
              const dimmed = hover && !isHighlighted;
              const color = EDGE_COLORS[e.edge_type] ?? "#3a342d";
              return (
                <line
                  key={i}
                  x1={a.x}
                  y1={a.y}
                  x2={b.x}
                  y2={b.y}
                  stroke={color}
                  strokeWidth={isHighlighted ? 1.5 : 0.6}
                  opacity={dimmed ? 0.1 : isHighlighted ? 0.9 : 0.4}
                  markerEnd={e.edge_type === "imports" ? `url(#arrow-imports)` : undefined}
                />
              );
            })}
            {data?.nodes.map((n) => {
              const p = posMap[n.uid];
              if (!p) return null;
              const r = nodeRadius(n);
              const colors = KIND_COLORS[n.kind] ?? KIND_COLORS.function;
              const isHover = hover === n.uid;
              const isMatch = matches.has(n.uid);
              const dimmed = hover && !isHover && !connectedToHover.has(n.uid);
              const faded = query.trim() && !isMatch;
              return (
                <g
                  key={n.uid}
                  transform={`translate(${p.x},${p.y})`}
                  onMouseEnter={() => setHover(n.uid)}
                  onMouseLeave={() => setHover(null)}
                  onClick={() => {
                    if (n.kind !== "file" && n.file_path) {
                      setSelected(n.uid);
                    }
                  }}
                  style={{ cursor: n.kind !== "file" ? "pointer" : "default" }}
                  opacity={dimmed || faded ? 0.15 : 1}
                >
                  <circle
                    r={r + 3}
                    fill={isHover || isMatch ? colors.stroke : "transparent"}
                    opacity={isHover || isMatch ? 0.3 : 0}
                  />
                  <circle
                    r={r}
                    fill={colors.fill}
                    stroke={colors.stroke}
                    strokeWidth={n.kind === "file" ? 2 : 1}
                  />
                  {isHover && (
                    <text
                      y={-r - 8}
                      textAnchor="middle"
                      className="fill-text"
                      style={{ fontSize: 11, fontFamily: "Inter, system-ui" }}
                    >
                      {n.label}
                    </text>
                  )}
                </g>
              );
            })}
          </g>
        </svg>

        {/* Controls */}
        <div className="pointer-events-none absolute inset-0 flex flex-col">
          <div className="pointer-events-auto flex items-center gap-2 border-b border-border-subtle bg-bg/80 p-3 backdrop-blur">
            <Share2 size={14} className="text-accent" />
            <div className="font-serif text-lg">Graph</div>
            <span className="font-mono text-[10px] text-text-dim">
              {currentProjectId}
            </span>
            <span className="font-mono text-[10px] text-text-dim">
              · {data?.nodes.length ?? 0} nodes · {data?.edges.length ?? 0} edges
            </span>
            <div className="flex-1" />
            <div className="relative">
              <Search
                size={12}
                className="pointer-events-none absolute left-2 top-1/2 -translate-y-1/2 text-text-dim"
              />
              <input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search nodes…"
                className="input w-48 py-1 pl-7 text-xs"
              />
            </div>
            <button onClick={reload} className="btn-ghost" title="Refresh">
              <RefreshCw size={13} className={busy ? "animate-spin" : ""} />
            </button>
            <button onClick={resetView} className="btn-ghost" title="Reset view">
              reset
            </button>
          </div>
          <div className="flex-1" />
          <div className="pointer-events-auto absolute bottom-3 right-3 flex flex-col gap-1">
            <button
              onClick={() => setView((v) => ({ ...v, k: Math.min(4, v.k * 1.2) }))}
              className="btn-outline h-8 w-8 p-0"
              title="Zoom in"
            >
              <ZoomIn size={14} />
            </button>
            <button
              onClick={() => setView((v) => ({ ...v, k: Math.max(0.2, v.k * 0.83) }))}
              className="btn-outline h-8 w-8 p-0"
              title="Zoom out"
            >
              <ZoomOut size={14} />
            </button>
          </div>
        </div>
      </div>

      {/* Sidebar */}
      <div className="hidden w-72 shrink-0 overflow-y-auto border-l border-border-subtle bg-surface/40 p-4 lg:block">
        <div className="mb-4">
          <div className="mb-2 flex items-center gap-2 text-text-muted">
            <Filter size={12} />
            <span className="text-[10px] uppercase tracking-widest">Node kinds</span>
          </div>
          <div className="space-y-1.5">
            {(["file", "function", "class", "struct"] as const).map((k) => (
              <label
                key={k}
                className="flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-xs hover:bg-surface-2"
              >
                <input
                  type="checkbox"
                  checked={filter.has(k)}
                  onChange={() => {
                    const n = new Set(filter);
                    if (n.has(k)) n.delete(k);
                    else n.add(k);
                    setFilter(n);
                  }}
                  className="accent-accent"
                />
                <span
                  className="h-2.5 w-2.5 rounded-full"
                  style={{ background: KIND_COLORS[k].fill }}
                />
                <span className="capitalize text-text-muted">{k}</span>
                <span className="ml-auto font-mono text-[10px] text-text-dim">
                  {graph.nodes.filter((n) => n.kind === k).length}
                </span>
              </label>
            ))}
          </div>
        </div>

        <div className="mb-4">
          <div className="mb-2 text-[10px] uppercase tracking-widest text-text-muted">
            Edge types
          </div>
          <div className="space-y-1">
            {Object.entries(EDGE_COLORS).map(([k, c]) => {
              const n = graph.edges.filter((e) => e.edge_type === k).length;
              return (
                <div
                  key={k}
                  className="flex items-center gap-2 px-2 py-1 text-xs text-text-muted"
                >
                  <span
                    className="h-0.5 w-6 rounded"
                    style={{ background: c }}
                  />
                  <span className="font-mono">{k}</span>
                  <span className="ml-auto font-mono text-[10px] text-text-dim">{n}</span>
                </div>
              );
            })}
          </div>
        </div>

        {hoverNode && (
          <div className="card p-3">
            <div className="mb-1.5 flex items-center gap-2">
              <span
                className="h-2 w-2 rounded-full"
                style={{ background: KIND_COLORS[hoverNode.kind]?.fill }}
              />
              <span className="text-[10px] uppercase tracking-widest text-text-dim">
                {hoverNode.kind}
              </span>
            </div>
            <div className="font-serif text-sm text-text">{hoverNode.label}</div>
            {hoverNode.file_path && (
              <div className="mt-1.5 truncate font-mono text-[10px] text-text-dim">
                {hoverNode.file_path}
                {hoverNode.start_line && `:${hoverNode.start_line}`}
              </div>
            )}
          </div>
        )}

        <div className="mt-4 rounded-md border border-border-subtle bg-surface-2 p-3 text-[11px] text-text-muted">
          <div className="mb-1 font-medium text-text">Tips</div>
          <ul className="ml-3 list-disc space-y-0.5">
            <li>Drag to pan, wheel to zoom</li>
            <li>Click a function/struct to open detail</li>
            <li>Hover to highlight connections</li>
            <li>File nodes are containers, not clickable</li>
          </ul>
        </div>
      </div>
    </div>
  );
}

// Simple force-directed layout. ~150 LoC. Good enough for hundreds of nodes.
function layout(
  nodes: GraphNode[],
  edges: GraphEdge[]
): Record<string, Pos> {
  const W = 1200;
  const H = 800;
  const cx = W / 2;
  const cy = H / 2;

  // Initial: file nodes on a circle, others near their file.
  const files = nodes.filter((n) => n.kind === "file");
  const fileRadius = Math.min(W, H) * 0.32;
  const filePos: Record<string, { x: number; y: number }> = {};
  files.forEach((f, i) => {
    const ang = (i / Math.max(1, files.length)) * Math.PI * 2;
    filePos[f.uid] = {
      x: cx + Math.cos(ang) * fileRadius,
      y: cy + Math.sin(ang) * fileRadius,
    };
  });

  const pos: Record<string, Pos> = {};
  for (const n of nodes) {
    if (n.kind === "file") {
      const fp = filePos[n.uid];
      pos[n.uid] = { x: fp.x, y: fp.y, vx: 0, vy: 0 };
    } else {
      // Start near the file (best effort)
      const fileUid = nodes.find(
        (m) =>
          m.kind === "file" &&
          n.file_path &&
          m.file_path === n.file_path
      )?.uid;
      const fp = fileUid ? filePos[fileUid] : { x: cx, y: cy };
      const ang = Math.random() * Math.PI * 2;
      const r = 30 + Math.random() * 30;
      pos[n.uid] = {
        x: fp.x + Math.cos(ang) * r,
        y: fp.y + Math.sin(ang) * r,
        vx: 0,
        vy: 0,
      };
    }
  }

  const idx = new Map(nodes.map((n) => [n.uid, n]));
  const adj: [string, string][] = edges.map((e) => [e.from, e.to]);

  // Run ~300 iterations of force simulation.
  const iterations = 300;
  const repulsion = 4500;
  const springLen = 70;
  const springK = 0.04;
  const centerK = 0.005;
  const damping = 0.82;

  for (let it = 0; it < iterations; it++) {
    // Repulsion (O(n^2) but n is small)
    const ids = Object.keys(pos);
    for (let i = 0; i < ids.length; i++) {
      for (let j = i + 1; j < ids.length; j++) {
        const a = pos[ids[i]];
        const b = pos[ids[j]];
        const dx = a.x - b.x;
        const dy = a.y - b.y;
        let dist2 = dx * dx + dy * dy;
        if (dist2 < 1) dist2 = 1;
        const dist = Math.sqrt(dist2);
        const nodeA = idx.get(ids[i])!;
        const nodeB = idx.get(ids[j])!;
        // Pin files a bit so they don't fly around.
        const massA = nodeA.kind === "file" ? 8 : 1;
        const massB = nodeB.kind === "file" ? 8 : 1;
        const f = repulsion / dist2;
        const fx = (dx / dist) * f;
        const fy = (dy / dist) * f;
        a.vx += (fx / massA) * 0.5;
        a.vy += (fy / massA) * 0.5;
        b.vx -= (fx / massB) * 0.5;
        b.vy -= (fy / massB) * 0.5;
      }
    }
    // Spring along edges
    for (const [u, v] of adj) {
      const a = pos[u];
      const b = pos[v];
      if (!a || !b) continue;
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const dist = Math.sqrt(dx * dx + dy * dy) || 1;
      const diff = dist - springLen;
      const f = diff * springK;
      a.vx += (dx / dist) * f;
      a.vy += (dy / dist) * f;
      b.vx -= (dx / dist) * f;
      b.vy -= (dy / dist) * f;
    }
    // Centering
    for (const id of ids) {
      const p = pos[id];
      p.vx += (cx - p.x) * centerK;
      p.vy += (cy - p.y) * centerK;
    }
    // Integrate + damp + clamp
    for (const id of ids) {
      const p = pos[id];
      p.vx *= damping;
      p.vy *= damping;
      p.x += p.vx;
      p.y += p.vy;
      // Clamp to canvas
      p.x = Math.max(-W, Math.min(2 * W, p.x));
      p.y = Math.max(-H, Math.min(2 * H, p.y));
    }
  }

  return pos;
}
