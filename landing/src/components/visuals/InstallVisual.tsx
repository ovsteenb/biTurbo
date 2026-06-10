"use client";

import { motion } from "framer-motion";

export function InstallVisual() {
  return (
    <div className="relative h-full w-full bg-gradient-to-br from-ink-800/80 to-ink-900/90 p-6 font-mono text-xs">
      <div className="mb-3 flex items-center gap-2">
        <div className="flex gap-1.5">
          <div className="h-2 w-2 rounded-full bg-ink-300/40" />
          <div className="h-2 w-2 rounded-full bg-ink-300/40" />
          <div className="h-2 w-2 rounded-full bg-ink-300/40" />
        </div>
        <span className="ml-2 text-[10px] uppercase tracking-widest text-ink-300">
          zsh · ~/Projekte
        </span>
      </div>

      <div className="space-y-3 text-[12px] leading-relaxed">
        {[
          { type: "comment", text: "# 1. Install the MCP binary" },
          { type: "cmd", text: "cargo install --path src-tauri --bin biturbo-mcp" },
          { type: "out", text: "  Compiling biturbo v0.2.0" },
          { type: "out", text: "  Compiling turbovec v0.8" },
          { type: "out", text: "  Compiling rmcp v1.7" },
          { type: "out", text: "   Finished release [optimized] in 47.2s" },
          { type: "success", text: "✓ biturbo-mcp installed → ~/.cargo/bin/" },
          { type: "spacer" },
          { type: "comment", text: "# 2. Verify (smoke test 19 MCP tools)" },
          { type: "cmd", text: "pnpm mcp:test" },
          { type: "out", text: "  → 19 pass · 0 fail · 0 skip   ⏱ 1.9s" },
          { type: "spacer" },
          { type: "comment", text: "# 3. Wire into your agent (Claude, Cursor, Cline, ...)" },
          { type: "code", text: '{"mcpServers": {"biturbo": {"command": "biturbo-mcp"}}}' },
          { type: "spacer" },
          { type: "comment", text: "# 4. Open the desktop app" },
          { type: "cmd", text: "pnpm tauri:dev" },
        ].map((line, i) => {
          if (line.type === "spacer") return <div key={i} className="h-1" />;
          return (
            <motion.div
              key={i}
              initial={false}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: false, margin: "-100px" }}
              transition={{ delay: i * 0.06, duration: 0.25 }}
              className={
                line.type === "comment" ? "text-ink-300" :
                line.type === "cmd" ? "text-ink" :
                line.type === "out" ? "text-ink-300" :
                line.type === "success" ? "text-moss" :
                "text-amber"
              }
            >
              {line.type === "cmd" && <span className="mr-1.5 text-moss">$</span>}
              {line.text}
            </motion.div>
          );
        })}

        <motion.div
          initial={false}
          whileInView={{ opacity: 1 }}
          viewport={{ once: false, margin: "-100px" }}
          transition={{ delay: 1.2 }}
          className="mt-2 inline-block rounded border border-moss/30 bg-moss/5 px-2 py-0.5 text-[10px] text-moss"
        >
          ready in 49s · 11.8 MB binary
        </motion.div>
      </div>
    </div>
  );
}
