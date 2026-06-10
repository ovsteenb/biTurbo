"use client";

import { motion } from "framer-motion";

export function MemoryVisual() {
  return (
    <div className="relative h-full w-full bg-gradient-to-br from-ink-800/80 to-ink-900/90 p-6">
      {/* Header */}
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="h-2 w-2 rounded-full bg-moss" />
          <span className="font-mono text-[10px] uppercase tracking-widest text-ink-300">
            memory://recents
          </span>
        </div>
        <span className="font-mono text-[10px] text-ink-300">3 / 8 shown</span>
      </div>

      {/* Memory cards */}
      <div className="space-y-2.5">
        {[
          { tag: "decision", project: "testy", text: "Use SQLite WAL mode for concurrent agent writes", imp: 0.92, time: "2h ago" },
          { tag: "pattern", project: "scout-qa", text: "Laravel + Inertia requests need X-Inertia header on every POST", imp: 0.87, time: "5h ago" },
          { tag: "gotcha", project: "biTurbo", text: "pnpm 11 requires allowBuilds in workspace yaml for esbuild", imp: 0.78, time: "1d ago" },
          { tag: "context", project: "testy", text: "WebView inspector reads DOM via executeJavaScript sync", imp: 0.71, time: "2d ago" },
        ].map((m, i) => (
          <motion.div
            key={i}
            initial={false}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: false, margin: "-100px" }}
            transition={{ delay: i * 0.08, duration: 0.5 }}
            className="group rounded-lg border border-ink-200/10 bg-ink-900/60 p-3 transition-colors hover:border-moss/40"
          >
            <div className="flex items-center justify-between text-[10px] font-mono uppercase tracking-wider">
              <div className="flex items-center gap-2">
                <span className="text-moss">#{m.tag}</span>
                <span className="text-ink-300/60">·</span>
                <span className="text-ink-300">{m.project}</span>
              </div>
              <span className="text-ink-300/60">{m.time}</span>
            </div>
            <p className="mt-1.5 text-sm leading-snug text-ink/90">{m.text}</p>
            <div className="mt-2 flex items-center gap-2">
              <div className="h-1 flex-1 overflow-hidden rounded-full bg-ink-200/5">
                <motion.div
                  initial={false}
                  whileInView={{ width: `${m.imp * 100}%` }}
                  viewport={{ once: false, margin: "-100px" }}
                  transition={{ delay: 0.3 + i * 0.08, duration: 0.8 }}
                  className="h-full bg-gradient-to-r from-moss to-amber"
                />
              </div>
              <span className="font-mono text-[9px] text-ink-300">{m.imp.toFixed(2)}</span>
            </div>
          </motion.div>
        ))}
      </div>

      {/* Footer stats */}
      <div className="mt-4 flex items-center justify-between border-t border-ink-200/10 pt-3 font-mono text-[10px] text-ink-300">
        <span>1,247 memories</span>
        <span>·</span>
        <span>8 projects</span>
        <span>·</span>
        <span className="text-moss">2ms recall</span>
      </div>
    </div>
  );
}
