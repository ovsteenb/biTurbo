"use client";

import { motion } from "framer-motion";

export function SpeedVisual() {
  return (
    <div className="relative h-full w-full bg-gradient-to-br from-ink-800/80 to-ink-900/90 p-6">
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="h-2 w-2 rounded-full bg-amber" />
          <span className="font-mono text-[10px] uppercase tracking-widest text-ink-300">
            turbovec 4-bit · compression
          </span>
        </div>
        <span className="font-mono text-[10px] text-amber">16× smaller</span>
      </div>

      <div className="grid grid-cols-2 gap-4">
        {/* float32 */}
        <div>
          <div className="mb-1.5 font-mono text-[9px] uppercase tracking-wider text-ink-300">
            float32
          </div>
          <div className="space-y-0.5">
            {Array.from({ length: 32 }).map((_, i) => (
              <motion.div
                key={i}
                initial={false}
                whileInView={{ opacity: 1, scaleX: 1 }}
                viewport={{ once: false, margin: "-100px" }}
                transition={{ delay: i * 0.012, duration: 0.2 }}
                className="h-1.5 origin-left rounded-sm bg-gradient-to-r from-ink-300/60 to-ink-300/30"
                style={{ width: `${85 + (i % 4) * 4}%` }}
              />
            ))}
          </div>
          <div className="mt-2 font-mono text-[10px] text-ink-300">384 MB / 1M vectors</div>
        </div>

        {/* turbovec 4-bit */}
        <div>
          <div className="mb-1.5 font-mono text-[9px] uppercase tracking-wider text-amber">
            turbovec 4-bit
          </div>
          <div className="space-y-0.5">
            {Array.from({ length: 8 }).map((_, i) => (
              <motion.div
                key={i}
                initial={false}
                whileInView={{ opacity: 1, scaleX: 1 }}
                viewport={{ once: false, margin: "-100px" }}
                transition={{ delay: 0.2 + i * 0.03, duration: 0.2 }}
                className="h-3 origin-left rounded-sm bg-gradient-to-r from-amber to-moss"
                style={{ width: `${85 + (i % 4) * 4}%` }}
              />
            ))}
          </div>
          <div className="mt-2 font-mono text-[10px] text-amber">24 MB / 1M vectors</div>
        </div>
      </div>

      {/* Compression bar */}
      <div className="mt-6">
        <div className="mb-1 flex items-center justify-between font-mono text-[9px] text-ink-300">
          <span>RAM footprint · 1M memories</span>
          <span>compressed</span>
        </div>
        <div className="relative h-3 overflow-hidden rounded-full bg-ink-200/5">
          <motion.div
            initial={false}
            whileInView={{ width: "6.25%" }}
            viewport={{ once: false, margin: "-100px" }}
            transition={{ delay: 0.4, duration: 1.2, ease: [0.16, 1, 0.3, 1] }}
            className="h-full bg-gradient-to-r from-amber via-moss to-amber"
          />
        </div>
        <div className="mt-1 flex items-center justify-between font-mono text-[9px]">
          <span className="text-ink-300">384 MB</span>
          <motion.span
            initial={false}
            whileInView={{ opacity: 1 }}
            transition={{ delay: 1.6 }}
            className="text-amber"
          >
            24 MB
          </motion.span>
        </div>
      </div>

      {/* Stats grid */}
      <div className="mt-6 grid grid-cols-3 gap-2">
        {[
          { v: "<50ms", l: "cold start" },
          { v: "<2ms", l: "recall k=8" },
          { v: "5 langs", l: "tree-sitter" },
        ].map((s, i) => (
          <motion.div
            key={s.l}
            initial={false}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: false, margin: "-100px" }}
            transition={{ delay: 0.6 + i * 0.1 }}
            className="rounded-lg border border-ink-200/10 bg-ink-900/60 p-2.5"
          >
            <div className="font-display text-xl font-bold text-amber">{s.v}</div>
            <div className="mt-0.5 font-mono text-[9px] uppercase tracking-wider text-ink-300">
              {s.l}
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
