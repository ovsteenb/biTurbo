import { formatDistanceToNow, format } from "date-fns";

export function timeAgo(ms: number): string {
  return formatDistanceToNow(new Date(ms), { addSuffix: true });
}

export function shortDate(ms: number): string {
  return format(new Date(ms), "MMM d, HH:mm");
}

export function dayLabel(ms: number): string {
  return format(new Date(ms), "EEE");
}

export function bytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`;
  return `${(n / 1024 / 1024 / 1024).toFixed(2)} GB`;
}

export const MEM_TYPE_META: Record<
  string,
  { label: string; color: string; bg: string; ring: string; dot: string }
> = {
  fact: {
    label: "Fact",
    color: "text-sky-300",
    bg: "bg-sky-500/10",
    ring: "ring-sky-500/30",
    dot: "bg-sky-400",
  },
  decision: {
    label: "Decision",
    color: "text-amber-300",
    bg: "bg-amber-500/10",
    ring: "ring-amber-500/30",
    dot: "bg-amber-400",
  },
  preference: {
    label: "Preference",
    color: "text-violet-300",
    bg: "bg-violet-500/10",
    ring: "ring-violet-500/30",
    dot: "bg-violet-400",
  },
  pattern: {
    label: "Pattern",
    color: "text-emerald-300",
    bg: "bg-emerald-500/10",
    ring: "ring-emerald-500/30",
    dot: "bg-emerald-400",
  },
  episode: {
    label: "Episode",
    color: "text-rose-300",
    bg: "bg-rose-500/10",
    ring: "ring-rose-500/30",
    dot: "bg-rose-400",
  },
  reflection: {
    label: "Reflection",
    color: "text-indigo-300",
    bg: "bg-indigo-500/10",
    ring: "ring-indigo-500/30",
    dot: "bg-indigo-400",
  },
  code: {
    label: "Code",
    color: "text-orange-300",
    bg: "bg-orange-500/10",
    ring: "ring-orange-500/30",
    dot: "bg-orange-400",
  },
};

export function importanceDots(imp: number): number {
  // 0..1 → 1..5 dots
  return Math.max(1, Math.min(5, Math.round(imp * 5)));
}
