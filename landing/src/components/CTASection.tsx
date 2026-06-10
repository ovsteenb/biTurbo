"use client";

import { motion } from "framer-motion";

export function CTASection() {
  return (
    <section className="relative overflow-hidden border-t border-ink-200/10 py-32">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-1/2 top-1/2 h-[600px] w-[600px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-moss/10 blur-[120px]" />
        <div className="absolute left-1/3 top-1/2 h-[400px] w-[400px] -translate-y-1/2 rounded-full bg-amber/10 blur-[120px]" />
        <div className="absolute right-1/3 top-1/2 h-[400px] w-[400px] -translate-y-1/2 rounded-full bg-lilac/10 blur-[120px]" />
      </div>

      <div className="relative z-10 mx-auto max-w-5xl px-6 text-center">
        <motion.h2
          initial={false}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: false, margin: "-100px" }}
          transition={{ duration: 0.6 }}
          className="font-display text-[clamp(3rem,9vw,7.5rem)] font-extrabold leading-[0.9] tracking-[-0.04em] text-ink"
        >
          Give your agents<br />
          <span className="bg-gradient-to-r from-moss via-amber to-sky bg-clip-text text-transparent">
            a memory.
          </span>
        </motion.h2>

        <motion.p
          initial={false}
          whileInView={{ opacity: 1 }}
          viewport={{ once: false, margin: "-100px" }}
          transition={{ delay: 0.2, duration: 0.6 }}
          className="mx-auto mt-8 max-w-2xl text-pretty text-xl text-ink-200/80"
        >
          Free. Open source (MIT). One Rust binary. Five minutes from
          <code className="mx-1.5 rounded bg-ink-200/10 px-1.5 py-0.5 font-mono text-base text-moss">
            cargo install
          </code>
          to your agent writing memories that survive a reboot.
        </motion.p>

        <motion.div
          initial={false}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: false, margin: "-100px" }}
          transition={{ delay: 0.4, duration: 0.6 }}
          className="mt-12 flex flex-wrap items-center justify-center gap-4"
        >
          <a
            href="https://github.com/RyanCodrai/biturbo"
            className="group relative inline-flex items-center gap-3 overflow-hidden rounded-full bg-ink px-7 py-4 text-base font-medium text-ink-900 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-[0_20px_50px_-15px_rgba(236,235,227,0.5)]"
          >
            <svg
              viewBox="0 0 24 24"
              className="h-5 w-5"
              fill="currentColor"
            >
              <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.44 9.8 8.21 11.39.6.11.82-.26.82-.58 0-.29-.01-1.05-.02-2.05-3.34.72-4.04-1.61-4.04-1.61-.55-1.39-1.34-1.76-1.34-1.76-1.09-.74.08-.73.08-.73 1.2.08 1.84 1.24 1.84 1.24 1.07 1.83 2.81 1.3 3.5 1 .11-.78.42-1.3.76-1.6-2.67-.3-5.47-1.33-5.47-5.93 0-1.31.47-2.38 1.24-3.22-.13-.3-.54-1.52.12-3.18 0 0 1.01-.32 3.3 1.23a11.5 11.5 0 0 1 6 0c2.29-1.55 3.3-1.23 3.3-1.23.66 1.66.25 2.88.12 3.18.77.84 1.24 1.91 1.24 3.22 0 4.61-2.81 5.62-5.48 5.92.43.37.81 1.1.81 2.22 0 1.6-.01 2.89-.01 3.29 0 .32.22.69.83.57A12.01 12.01 0 0 0 24 12c0-6.63-5.37-12-12-12z" />
            </svg>
            <span>Star on GitHub</span>
            <span className="font-mono text-xs opacity-60">★ soon</span>
            <span className="absolute inset-0 -translate-x-full bg-gradient-to-r from-transparent via-white/20 to-transparent transition-transform duration-700 group-hover:translate-x-full" />
          </a>
          <a
            href="/features"
            className="inline-flex items-center gap-2 rounded-full border border-ink-200/20 px-7 py-4 text-base font-medium text-ink transition-all duration-200 hover:border-ink-200/40 hover:bg-ink-200/5"
          >
            Read the full feature deep-dive →
          </a>
        </motion.div>

        <motion.div
          initial={false}
          whileInView={{ opacity: 1 }}
          viewport={{ once: false, margin: "-100px" }}
          transition={{ delay: 0.6 }}
          className="mt-12 flex flex-wrap items-center justify-center gap-6 font-mono text-xs text-ink-300"
        >
          <span>~12 MB binary</span>
          <span>·</span>
          <span>cold start &lt; 50 ms</span>
          <span>·</span>
          <span>MIT licensed</span>
          <span>·</span>
          <span>no telemetry</span>
        </motion.div>
      </div>
    </section>
  );
}
