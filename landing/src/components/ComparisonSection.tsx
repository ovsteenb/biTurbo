"use client";

import { motion } from "framer-motion";

const rows = [
  { feature: "Storage", biturbo: "Local file (SQLite + turbovec)", cloud: "Hosted vector DB", other: "Bolt-on to chat app" },
  { feature: "Cold start", biturbo: "< 50 ms", cloud: "300–1500 ms", other: "Varies" },
  { feature: "Offline", biturbo: "✓ 100%", cloud: "✗", other: "Partial" },
  { feature: "Project isolation", biturbo: "✓ per-project index", cloud: "✗ namespace", other: "Weak" },
  { feature: "MCP-native", biturbo: "19 stdio tools", cloud: "Requires proxy", other: "—" },
  { feature: "Code indexing", biturbo: "tree-sitter, 5 langs", cloud: "—", other: "— " },
  { feature: "Cost", biturbo: "$0 forever", cloud: "$/mo per seat", other: "Free tier only" },
  { feature: "License", biturbo: "MIT", cloud: "Proprietary", other: "Mixed" },
];

export function ComparisonSection() {
  return (
    <section className="relative border-t border-ink-200/10 py-32">
      <div className="mx-auto max-w-7xl px-6">
        <div className="mb-12 max-w-2xl">
          <span className="font-mono text-xs uppercase tracking-[0.2em] text-amber">
            § comparison
          </span>
          <h2 className="mt-3 font-display text-5xl font-extrabold leading-[0.95] tracking-[-0.03em] text-ink md:text-6xl">
            Not a hosted vector DB.
          </h2>
          <p className="mt-4 max-w-xl text-pretty text-lg text-ink-200/70">
            biTurbo runs where your code lives. No servers, no proxies, no telemetry, no surprises
            on your bill.
          </p>
        </div>

        <div className="overflow-hidden rounded-2xl border border-ink-200/10 bg-ink-200/[0.02]">
          <div className="grid grid-cols-3 border-b border-ink-200/10 bg-ink-200/[0.03] px-6 py-4 font-mono text-xs uppercase tracking-wider text-ink-300">
            <div>Dimension</div>
            <div className="flex items-center gap-2 text-moss">
              <span className="h-2 w-2 rounded-full bg-moss" />
              biTurbo
            </div>
            <div>Cloud-hosted / bolt-on</div>
          </div>
          {rows.map((row, i) => (
            <motion.div
              key={row.feature}
              initial={false}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: false, margin: "-50px" }}
              transition={{ delay: i * 0.04 }}
              className="grid grid-cols-3 border-b border-ink-200/5 px-6 py-4 text-sm last:border-b-0 hover:bg-ink-200/[0.02]"
            >
              <div className="font-mono text-xs uppercase tracking-wider text-ink-300">
                {row.feature}
              </div>
              <div className="font-medium text-ink">{row.biturbo}</div>
              <div className="text-ink-300/60">{row.cloud} / {row.other}</div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
}
