"use client";

import { motion, useScroll, useTransform } from "framer-motion";
import { useRef, useEffect, useState } from "react";
import { InstallVisual } from "./visuals/InstallVisual";

export function InstallSection() {
  const ref = useRef<HTMLDivElement>(null);
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);

  const { scrollYProgress } = useScroll({
    target: ref,
    offset: ["start end", "end start"],
  });
  const y = useTransform(scrollYProgress, [0, 1], [80, -80]);
  const opacity = useTransform(scrollYProgress, [0, 0.2, 0.8, 1], [0, 1, 1, 0.3]);

  return (
    <section ref={ref} id="install" className="relative min-h-[90vh] overflow-hidden py-32">
      <div className="pointer-events-none absolute inset-x-0 top-0 h-px bg-gradient-to-r from-transparent via-moss/40 to-transparent" />
      <div className="grid-lines pointer-events-none absolute inset-0 opacity-30" />

      {mounted ? (
        <motion.div
          style={{ y, opacity }}
          className="relative z-10 mx-auto max-w-7xl px-6"
        >
        <div className="mb-12">
          <span className="font-mono text-xs uppercase tracking-[0.2em] text-moss">
            § install
          </span>
          <h2 className="mt-3 font-display text-5xl font-extrabold leading-[0.95] tracking-[-0.03em] text-ink md:text-7xl">
            Four lines.<br />
            <span className="text-ink-300">You&apos;re done.</span>
          </h2>
        </div>

        <div className="grid grid-cols-1 gap-8 lg:grid-cols-2">
          <div className="rounded-2xl border border-ink-200/10 bg-ink-200/[0.02] p-2">
            <InstallVisual />
          </div>

          <div className="flex flex-col justify-center space-y-6">
            <Step n="1" title="Install the binary">
              Single Rust crate. ~12 MB. Builds in 50s on a cold cache.
            </Step>
            <Step n="2" title="Smoke test 19 tools">
              <code className="font-mono text-moss">pnpm mcp:test</code> spawns the real binary and
              validates every tool in ~2 seconds.
            </Step>
            <Step n="3" title="Wire it into your agent">
              One JSON block in your MCP config. Works with Claude Code, Cursor, Cline, Mavis.
            </Step>
            <Step n="4" title="Open the desktop app">
              Tauri 2 window. Browse memories, inspect graph, manage projects.
            </Step>
          </div>
        </div>
        </motion.div>
      ) : (
        <div
          suppressHydrationWarning
          className="relative z-10 mx-auto max-w-7xl px-6"
        >
          <InstallContent />
        </div>
      )}
    </section>
  );
}

function InstallContent() {
  return (
    <>
      <div className="mb-12">
        <span className="font-mono text-xs uppercase tracking-[0.2em] text-moss">
          § install
        </span>
        <h2 className="mt-3 font-display text-5xl font-extrabold leading-[0.95] tracking-[-0.03em] text-ink md:text-7xl">
          Four lines.<br />
          <span className="text-ink-300">You&apos;re done.</span>
        </h2>
      </div>

      <div className="grid grid-cols-1 gap-8 lg:grid-cols-2">
        <div className="rounded-2xl border border-ink-200/10 bg-ink-200/[0.02] p-2">
          <InstallVisual />
        </div>

        <div className="flex flex-col justify-center space-y-6">
          <Step n="1" title="Install the binary">
            Single Rust crate. ~12 MB. Builds in 50s on a cold cache.
          </Step>
          <Step n="2" title="Smoke test 19 tools">
            <code className="font-mono text-moss">pnpm mcp:test</code> spawns the real binary and
            validates every tool in ~2 seconds.
          </Step>
          <Step n="3" title="Wire it into your agent">
            One JSON block in your MCP config. Works with Claude Code, Cursor, Cline, Mavis.
          </Step>
          <Step n="4" title="Open the desktop app">
            Tauri 2 window. Browse memories, inspect graph, manage projects.
          </Step>
        </div>
      </div>
    </>
  );
}

function Step({ n, title, children }: { n: string; title: string; children: React.ReactNode }) {
  return (
    <div className="flex gap-4">
      <div className="flex flex-col items-center">
        <div className="flex h-8 w-8 items-center justify-center rounded-full border border-moss/40 bg-moss/10 font-mono text-xs text-moss">
          {n}
        </div>
        <div className="mt-2 w-px flex-1 bg-gradient-to-b from-moss/30 to-transparent" />
      </div>
      <div className="pb-2">
        <div className="font-display text-xl font-bold text-ink">{title}</div>
        <div className="mt-1 text-pretty text-ink-200/70">{children}</div>
      </div>
    </div>
  );
}
