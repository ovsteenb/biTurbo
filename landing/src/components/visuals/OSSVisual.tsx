"use client";

import { motion } from "framer-motion";

export function OSSVisual() {
  return (
    <div className="relative h-full w-full bg-gradient-to-br from-ink-800/80 to-ink-900/90 p-6 font-mono text-xs">
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="h-2 w-2 rounded-full bg-lilac" />
          <span className="text-[10px] uppercase tracking-widest text-ink-300">
            github.com/RyanCodrai/biturbo
          </span>
        </div>
        <span className="rounded-full bg-lilac/15 px-2 py-0.5 text-[9px] text-lilac">
          ⭐ MIT
        </span>
      </div>

      {/* File tree */}
      <motion.div
        initial={false}
        whileInView={{ opacity: 1 }}
        transition={{ duration: 0.4 }}
        className="space-y-1"
      >
        {[
          { indent: 0, name: "biturbo/", color: "lilac", type: "dir" },
          { indent: 1, name: "src/", color: "sky", type: "dir" },
          { indent: 2, name: "App.tsx", color: "ink" },
          { indent: 2, name: "store.ts", color: "ink" },
          { indent: 2, name: "views/", color: "sky", type: "dir" },
          { indent: 3, name: "Memories.tsx", color: "ink" },
          { indent: 3, name: "Graph.tsx", color: "ink" },
          { indent: 1, name: "src-tauri/", color: "sky", type: "dir" },
          { indent: 2, name: "src/", color: "sky", type: "dir" },
          { indent: 3, name: "main.rs", color: "amber" },
          { indent: 3, name: "index_engine.rs", color: "amber" },
          { indent: 3, name: "mcp.rs", color: "amber" },
          { indent: 3, name: "consolidate.rs", color: "amber" },
          { indent: 2, name: "Cargo.toml", color: "moss" },
          { indent: 1, name: "scripts/", color: "sky", type: "dir" },
          { indent: 2, name: "mcp-smoke-test.ts", color: "lilac" },
          { indent: 0, name: "README.md", color: "moss" },
          { indent: 0, name: "INSTRUCTIONS.md", color: "moss" },
          { indent: 0, name: "LICENSE", color: "ink" },
        ].map((f, i) => (
          <motion.div
            key={i}
            initial={false}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: false, margin: "-100px" }}
            transition={{ delay: 0.05 + i * 0.025 }}
            className="flex items-center gap-1.5"
            style={{ paddingLeft: `${f.indent * 12}px` }}
          >
            <span className={f.type === "dir" ? "text-sky" : "text-ink-300/40"}>
              {f.type === "dir" ? "▸" : "·"}
            </span>
            <span
              className={
                f.color === "lilac" ? "text-lilac" :
                f.color === "sky" ? "text-sky" :
                f.color === "amber" ? "text-amber" :
                f.color === "moss" ? "text-moss" :
                "text-ink-200/80"
              }
            >
              {f.name}
            </span>
          </motion.div>
        ))}
      </motion.div>

      {/* Stats footer */}
      <div className="mt-4 grid grid-cols-3 gap-2 border-t border-ink-200/10 pt-3">
        {[
          { v: "MIT", l: "license" },
          { v: "Rust + React", l: "stack" },
          { v: "0 deps", l: "at runtime" },
        ].map((s, i) => (
          <motion.div
            key={s.l}
            initial={false}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: false, margin: "-100px" }}
            transition={{ delay: 0.8 + i * 0.08 }}
            className="rounded border border-lilac/20 bg-lilac/5 p-2 text-center"
          >
            <div className="font-display text-sm font-bold text-lilac">{s.v}</div>
            <div className="mt-0.5 text-[8px] uppercase tracking-wider text-ink-300">
              {s.l}
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
