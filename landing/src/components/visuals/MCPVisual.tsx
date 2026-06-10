"use client";

import { motion } from "framer-motion";

export function MCPVisual() {
  const tools = [
    { name: "remember", args: '("decide WAL mode", "testy")' },
    { name: "search", args: '("agent auth", "scout-qa", k=8)' },
    { name: "ingest_project", args: '("/Users/.../testy")' },
    { name: "recall_for_context", args: '("layout bug", "testy", k=4)' },
    { name: "consolidate", args: '("testy")' },
  ];

  return (
    <div className="relative h-full w-full bg-gradient-to-br from-ink-800/80 to-ink-900/90 p-6 font-mono text-xs">
      <div className="mb-3 flex items-center gap-2">
        <div className="flex gap-1.5">
          <div className="h-2 w-2 rounded-full bg-amber/60" />
          <div className="h-2 w-2 rounded-full bg-amber/40" />
          <div className="h-2 w-2 rounded-full bg-moss/60" />
        </div>
        <span className="text-[10px] uppercase tracking-widest text-ink-300">stdio · JSON-RPC</span>
      </div>

      {/* The wire */}
      <div className="space-y-2">
        <motion.div
          initial={false}
          whileInView={{ opacity: 1 }}
          transition={{ duration: 0.4 }}
          className="flex items-start gap-2"
        >
          <span className="text-ink-300">→</span>
          <div className="flex-1 rounded border border-sky/20 bg-sky/5 p-2 text-[11px] leading-relaxed">
            <div>{`{`}<span className="text-ink-300">&quot;jsonrpc&quot;</span>:<span className="text-moss">&quot;2.0&quot;</span>, <span className="text-ink-300">&quot;id&quot;</span>:<span className="text-amber">1</span>, <span className="text-ink-300">&quot;method&quot;</span>:<span className="text-amber">&quot;tools/list&quot;</span>{`}`}</div>
          </div>
        </motion.div>

        <motion.div
          initial={false}
          whileInView={{ opacity: 1 }}
          transition={{ duration: 0.4, delay: 0.2 }}
          className="flex items-start gap-2"
        >
          <span className="text-moss">←</span>
          <div className="flex-1 rounded border border-moss/20 bg-moss/5 p-2 text-[11px] leading-relaxed">
            <div>{`{`} <span className="text-ink-300">tools</span>: <span className="text-amber">[</span> remember, forget, update, search, list, recall_for_context, ... <span className="text-amber">]</span> {`}`}</div>
          </div>
        </motion.div>

        {/* Tool call sequence */}
        <motion.div
          initial={false}
          whileInView={{ opacity: 1 }}
          transition={{ duration: 0.4, delay: 0.4 }}
          className="mt-3 space-y-1.5"
        >
          {tools.map((tool, i) => (
            <motion.div
              key={tool.name}
              initial={false}
              whileInView={{ opacity: 1, x: 0 }}
              transition={{ delay: 0.5 + i * 0.1, duration: 0.3 }}
              className="flex items-center gap-2 rounded border border-ink-200/10 bg-ink-900/60 px-2 py-1.5"
            >
              <span className="text-amber">→</span>
              <span className="text-moss">{tool.name}</span>
              <span className="truncate text-ink-300">{tool.args}</span>
            </motion.div>
          ))}
        </motion.div>

        <motion.div
          initial={false}
          whileInView={{ opacity: 1 }}
          transition={{ duration: 0.4, delay: 1.2 }}
          className="mt-3 rounded border border-amber/20 bg-amber/5 p-2 text-[10px] leading-relaxed"
        >
          <div className="text-amber">{"// injected into agent context"}</div>
          <div className="text-ink/80">&lt;biTurboContext project=<span className="text-moss">&quot;testy&quot;</span>&gt;</div>
          <div className="pl-3 text-ink-200/80">· SQLite WAL allows concurrent reads during write</div>
          <div className="pl-3 text-ink-200/80">· pnpm 11 strict-dep builds need allowBuilds</div>
          <div className="pl-3 text-ink-200/80">· Tauri 2 webview inspect via executeJavaScript</div>
          <div className="text-ink/80">&lt;/biTurboContext&gt;</div>
        </motion.div>
      </div>
    </div>
  );
}
