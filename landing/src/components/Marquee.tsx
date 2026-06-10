"use client";

export function Marquee() {
  const items = [
    "persistent",
    "project-scoped",
    "semantic",
    "local-first",
    "MCP-native",
    "MIT",
    "4-bit compressed",
    "open source",
    "rust",
    "offline",
    "tree-sitter",
    "fastembed",
  ];
  const doubled = [...items, ...items];
  return (
    <div className="relative overflow-hidden border-y border-ink-200/10 bg-ink-200/[0.02] py-5">
      <div className="flex animate-marquee whitespace-nowrap">
        {doubled.map((item, i) => (
          <div key={i} className="flex items-center gap-8 px-8">
            <span className="font-display text-2xl font-bold tracking-tight text-ink-200/60 md:text-3xl">
              {item}
            </span>
            <span className="text-ink-300/30">✦</span>
          </div>
        ))}
      </div>
      <div className="pointer-events-none absolute inset-y-0 left-0 w-32 bg-gradient-to-r from-ink-900 to-transparent" />
      <div className="pointer-events-none absolute inset-y-0 right-0 w-32 bg-gradient-to-l from-ink-900 to-transparent" />
    </div>
  );
}
