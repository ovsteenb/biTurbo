"use client";

import Link from "next/link";
import { motion, useScroll, useTransform } from "framer-motion";
import { useEffect, useState } from "react";
import { cn } from "@/lib/cn";
import { Logo } from "./Logo";

export function Nav() {
  const [scrolled, setScrolled] = useState(false);
  const [mounted, setMounted] = useState(false);
  const { scrollY } = useScroll();
  const opacity = useTransform(scrollY, [0, 80], [0, 1]);
  const blur = useTransform(scrollY, [0, 80], [0, 12]);

  useEffect(() => {
    setMounted(true);
    const onScroll = () => setScrolled(window.scrollY > 12);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <>
      {mounted ? (
        <motion.div
          style={{ opacity, backdropFilter: blur.get() ? `blur(${blur.get()}px)` : undefined }}
          className="fixed inset-x-0 top-0 z-50 h-16 bg-ink-900/60"
        />
      ) : (
        <div
          suppressHydrationWarning
          className="fixed inset-x-0 top-0 z-50 h-16 bg-ink-900/60"
        />
      )}
      <header
        className={cn(
          "fixed inset-x-0 top-0 z-50 transition-all duration-300",
          scrolled ? "border-b border-ink-200/10" : ""
        )}
      >
        <nav className="mx-auto flex h-16 max-w-7xl items-center justify-between px-6">
          <Link href="/" className="group flex items-center gap-3">
            <Logo size={36} priority />
            <span className="hidden font-mono text-[10px] uppercase tracking-widest text-ink-300 sm:inline-block">
              v0.2
            </span>
          </Link>

          <div className="hidden items-center gap-1 rounded-full border border-ink-200/10 bg-ink-200/[0.03] p-1 md:flex">
            <NavLink href="/features">Features</NavLink>
            <NavLink href="/#memory">Memory</NavLink>
            <NavLink href="/#mcp">MCP</NavLink>
            <NavLink href="/#graph">Graph</NavLink>
            <NavLink href="/#speed">Speed</NavLink>
            <NavLink href="/#oss">Open Source</NavLink>
          </div>

          <div className="flex items-center gap-2">
            <a
              href="https://github.com/RyanCodrai/biturbo"
              className="hidden font-mono text-xs text-ink-200 transition-colors hover:text-ink sm:inline"
            >
              ★ GitHub
            </a>
            <Link href="/features" className="button-primary !px-4 !py-2 !text-xs">
              Explore features →
            </Link>
          </div>
        </nav>
      </header>
    </>
  );
}

function NavLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <Link
      href={href}
      className="rounded-full px-3.5 py-1.5 text-xs font-medium text-ink-200/70 transition-colors hover:bg-ink-200/10 hover:text-ink"
    >
      {children}
    </Link>
  );
}
