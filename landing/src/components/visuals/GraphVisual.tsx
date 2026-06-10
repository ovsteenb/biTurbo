"use client";

import { motion } from "framer-motion";
import { useMemo } from "react";

type Node = { id: string; x: number; y: number; r: number; color: string; label?: string };
type Edge = { from: string; to: string };

export function GraphVisual() {
  const { nodes, edges } = useMemo(() => {
    // Deterministic seed positions for a code dependency graph
    const N: Node[] = [
      { id: "auth", x: 50, y: 50, r: 14, color: "moss" },
      { id: "session", x: 24, y: 30, r: 10, color: "amber" },
      { id: "user", x: 76, y: 28, r: 12, color: "amber" },
      { id: "middleware", x: 50, y: 22, r: 8, color: "ink" },
      { id: "router", x: 50, y: 75, r: 14, color: "sky" },
      { id: "views", x: 24, y: 84, r: 9, color: "lilac" },
      { id: "api", x: 76, y: 84, r: 9, color: "lilac" },
      { id: "db", x: 50, y: 90, r: 7, color: "ink" },
      { id: "config", x: 16, y: 50, r: 6, color: "ink" },
      { id: "types", x: 84, y: 50, r: 6, color: "ink" },
      { id: "store", x: 30, y: 60, r: 5, color: "ink" },
      { id: "hooks", x: 70, y: 60, r: 5, color: "ink" },
    ];
    const E: Edge[] = [
      { from: "auth", to: "session" },
      { from: "auth", to: "user" },
      { from: "auth", to: "middleware" },
      { from: "auth", to: "router" },
      { from: "session", to: "store" },
      { from: "user", to: "store" },
      { from: "middleware", to: "config" },
      { from: "middleware", to: "types" },
      { from: "router", to: "views" },
      { from: "router", to: "api" },
      { from: "router", to: "db" },
      { from: "views", to: "hooks" },
      { from: "api", to: "hooks" },
      { from: "store", to: "db" },
      { from: "hooks", to: "db" },
    ];
    return { nodes: N, edges: E };
  }, []);

  const colorMap: Record<string, string> = {
    moss: "var(--moss)",
    amber: "var(--amber)",
    sky: "var(--sky)",
    lilac: "var(--lilac)",
    ink: "#52513f",
  };

  return (
    <div className="relative h-full w-full bg-gradient-to-br from-ink-800/80 to-ink-900/90 p-6">
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="h-2 w-2 rounded-full bg-sky" />
          <span className="font-mono text-[10px] uppercase tracking-widest text-ink-300">
            graph://testy/dependency
          </span>
        </div>
        <div className="flex gap-3 font-mono text-[9px] text-ink-300">
          <span><span className="text-moss">●</span> auth</span>
          <span><span className="text-amber">●</span> domain</span>
          <span><span className="text-sky">●</span> route</span>
          <span><span className="text-lilac">●</span> view</span>
        </div>
      </div>

      <div className="relative h-[calc(100%-3rem)] w-full">
        <svg viewBox="0 0 100 100" className="h-full w-full" preserveAspectRatio="xMidYMid meet">
          {/* Edges */}
          {edges.map((e, i) => {
            const a = nodes.find((n) => n.id === e.from)!;
            const b = nodes.find((n) => n.id === e.to)!;
            return (
              <motion.line
                key={`${e.from}-${e.to}`}
                x1={a.x}
                y1={a.y}
                x2={b.x}
                y2={b.y}
                stroke="rgba(199, 160, 224, 0.25)"
                strokeWidth="0.3"
                initial={false}
                whileInView={{ pathLength: 1, opacity: 1 }}
                viewport={{ once: false, margin: "-50px" }}
                transition={{ delay: 0.3 + i * 0.05, duration: 0.6 }}
              />
            );
          })}

          {/* Nodes */}
          {nodes.map((n, i) => (
            <motion.g
              key={n.id}
              initial={false}
              whileInView={{ opacity: 1, scale: 1 }}
              viewport={{ once: false, margin: "-50px" }}
              transition={{ delay: 0.1 + i * 0.06, type: "spring", stiffness: 200 }}
            >
              <circle
                cx={n.x}
                cy={n.y}
                r={n.r * 0.6}
                fill={colorMap[n.color]}
                opacity={n.color === "ink" ? 0.5 : 0.9}
              />
              <circle
                cx={n.x}
                cy={n.y}
                r={n.r * 0.6}
                fill="none"
                stroke={colorMap[n.color]}
                strokeWidth="0.2"
                opacity="0.4"
              >
                <animate
                  attributeName="r"
                  values={`${n.r * 0.6};${n.r * 0.9};${n.r * 0.6}`}
                  dur="3s"
                  repeatCount="indefinite"
                />
                <animate
                  attributeName="opacity"
                  values="0.6;0;0.6"
                  dur="3s"
                  repeatCount="indefinite"
                />
              </circle>
            </motion.g>
          ))}
        </svg>

        {/* Floating labels */}
        <motion.div
          initial={false}
          whileInView={{ opacity: 1 }}
          transition={{ delay: 0.8 }}
          className="absolute right-3 top-3 rounded-md border border-sky/30 bg-ink-900/80 px-2 py-1 font-mono text-[9px] text-sky backdrop-blur"
        >
          <div>3,247 nodes</div>
          <div className="text-ink-300">8,914 edges</div>
          <div className="text-moss">render &lt; 5ms</div>
        </motion.div>
      </div>
    </div>
  );
}
