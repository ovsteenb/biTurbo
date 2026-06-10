"use client";

import { motion, useScroll, useTransform } from "framer-motion";
import { useRef, useEffect, useState } from "react";

export function Hero() {
  const ref = useRef<HTMLDivElement>(null);
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);

  const { scrollYProgress } = useScroll({
    target: ref,
    offset: ["start start", "end start"],
  });

  const y = useTransform(scrollYProgress, [0, 1], [0, 200]);
  const opacity = useTransform(scrollYProgress, [0, 0.6], [1, 0]);
  const scale = useTransform(scrollYProgress, [0, 1], [1, 0.92]);

  return (
    <section ref={ref} className="relative min-h-[100vh] overflow-hidden">
      <AmbientOrbs />
      <div className="grid-lines pointer-events-none absolute inset-0 opacity-50" />

      {mounted ? (
        <motion.div
          style={{ y, opacity, scale }}
          className="relative z-10 mx-auto flex min-h-screen max-w-7xl flex-col justify-between px-6 pb-12 pt-32"
        >
          <HeroContent />
        </motion.div>
      ) : (
        <div
          suppressHydrationWarning
          className="relative z-10 mx-auto flex min-h-screen max-w-7xl flex-col justify-between px-6 pb-12 pt-32"
        >
          <HeroContent />
        </div>
      )}

      <ScrollIndicator />
    </section>
  );
}

function AmbientOrbs() {
  return (
    <div className="pointer-events-none absolute inset-0">
      <div className="absolute left-1/4 top-1/4 h-[500px] w-[500px] -translate-x-1/2 rounded-full bg-moss/15 blur-[120px] animate-drift" />
      <div
        className="absolute right-1/4 top-1/3 h-[400px] w-[400px] translate-x-1/2 rounded-full bg-amber/15 blur-[120px] animate-drift"
        style={{ animationDelay: "3s" }}
      />
      <div
        className="absolute bottom-1/4 left-1/2 h-[350px] w-[350px] -translate-x-1/2 rounded-full bg-lilac/10 blur-[120px] animate-drift"
        style={{ animationDelay: "6s" }}
      />
    </div>
  );
}

function ScrollIndicator() {
  return (
    <div className="pointer-events-none absolute bottom-6 left-1/2 z-10 -translate-x-1/2">
      <div className="flex flex-col items-center gap-2 text-ink-300">
        <span className="font-mono text-[10px] uppercase tracking-widest">Scroll</span>
        <div className="h-8 w-px bg-gradient-to-b from-ink-200/60 to-transparent" />
      </div>
    </div>
  );
}

function HeroContent() {
  return (
    <>
      <div className="flex flex-wrap items-center gap-2">
        <span className="chip">
          <span className="h-1.5 w-1.5 animate-pulse-glow rounded-full bg-moss" />
          Local-first · MIT licensed
        </span>
        <span className="chip">Rust 1.77+ · 4-bit compressed</span>
        <span className="chip">MCP · 19 tools</span>
      </div>

      <div className="max-w-[1400px]">
        <h1 className="font-display text-[clamp(3rem,11vw,10.5rem)] font-extrabold leading-[0.85] tracking-[-0.04em] text-ink">
          <span className="block">Your agents</span>
          <span className="block">
            have{" "}
            <span className="relative inline-block">
              <span className="relative z-10 bg-gradient-to-br from-ink via-ink to-ink-300 bg-clip-text text-transparent">
                memory
              </span>
              <span className="absolute -bottom-3 left-0 right-0 h-4 rounded-full bg-gradient-to-r from-moss via-amber to-sky opacity-70 blur-md" />
            </span>
            <span className="text-ink">.</span>
          </span>
        </h1>

        <div className="mt-10 grid grid-cols-1 gap-12 md:grid-cols-12 md:items-end">
          <p className="md:col-span-7 max-w-2xl text-balance text-xl text-ink-200/80 md:text-2xl">
            biTurbo is a single Rust binary that gives your AI coding agents{" "}
            <span className="text-ink">persistent, project-scoped, semantic memory</span> on your
            disk. No cloud. No SaaS. No embedding leakage. Cold start under 50ms.
          </p>
          <div className="md:col-span-5 flex flex-wrap items-center gap-3">
            <a href="#install" className="button-primary group">
              Install biTurbo
              <span className="inline-block transition-transform group-hover:translate-x-0.5">→</span>
            </a>
            <a href="/features" className="button-ghost">
              See all features
            </a>
          </div>
        </div>
      </div>

      <div className="mt-16 flex flex-col gap-4 border-t border-ink-200/10 pt-6 font-mono text-xs text-ink-300 md:flex-row md:items-center md:justify-between">
        <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
          <span>v0.2.0 · ~12 MB binary</span>
          <span className="hidden md:inline">·</span>
          <span>16× smaller than float32</span>
          <span className="hidden md:inline">·</span>
          <span>1M memories in laptop RAM</span>
        </div>
        <a
          href="https://github.com/RyanCodrai/biturbo"
          className="group inline-flex items-center gap-2 text-ink-200 transition-colors hover:text-ink"
        >
          <span className="h-1.5 w-1.5 rounded-full bg-moss" />
          Open source · MIT
          <span className="text-ink-300 transition-transform group-hover:translate-x-0.5">→</span>
        </a>
      </div>
    </>
  );
}
