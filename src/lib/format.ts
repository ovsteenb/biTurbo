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

export function truncatePath(path: string, maxLen = 40): string {
  if (path.length <= maxLen) return path;
  const segments = path.split("/");
  const file = segments.pop() ?? "";
  let result = file;
  while (segments.length > 0) {
    const next = segments.pop() + "/" + result;
    if (("…/" + next).length > maxLen) break;
    result = next;
  }
  return result === path ? result : "…/" + result;
}

export function importanceDots(imp: number): number {
  // 0..1 → 1..5 dots
  return Math.max(1, Math.min(5, Math.round(imp * 5)));
}

/**
 * Code memory content is often stored with a redundant leading header comment
 * (e.g. `// C:\path\file.ts:1-133`) that duplicates the path/range already
 * shown in the code chip. Strip it so the code block only shows real code.
 */
export function stripLeadingPathComment(content: string, filePath: string | null): string {
  if (!filePath) return content;
  const lines = content.split("\n");
  const first = lines[0]?.trim() ?? "";
  const isCommentLine = /^(\/\/|#|--|\*)/.test(first);
  const fileName = filePath.split(/[/\\]/).pop() ?? filePath;
  if (isCommentLine && first.includes(fileName)) {
    return lines.slice(1).join("\n").replace(/^\n+/, "");
  }
  return content;
}

const KEYWORDS = new Set([
  "import", "from", "export", "default", "const", "let", "var", "function",
  "return", "if", "else", "for", "while", "do", "switch", "case", "break",
  "continue", "class", "interface", "type", "extends", "implements", "public",
  "private", "protected", "static", "readonly", "async", "await", "new",
  "this", "super", "null", "undefined", "true", "false", "void", "try",
  "catch", "finally", "throw", "typeof", "instanceof", "in", "of", "as",
  "enum", "namespace", "declare", "yield", "delete", "fn", "impl", "struct",
  "pub", "mut", "use", "def", "self", "None", "True", "False", "match",
]);

export interface CodeToken {
  text: string;
  kind: "keyword" | "string" | "comment" | "number" | "plain";
}

/** Lightweight, dependency-free tokenizer for a code preview (not a full lexer). */
export function tokenizeCode(line: string): CodeToken[] {
  const tokens: CodeToken[] = [];
  const pattern =
    /(\/\/.*$|#.*$)|("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`)|(\b\d+(?:\.\d+)?\b)|([A-Za-z_$][\w$]*)|(\s+)|(.)/g;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(line))) {
    const [, comment, str, num, word, space, other] = match;
    if (comment !== undefined) tokens.push({ text: comment, kind: "comment" });
    else if (str !== undefined) tokens.push({ text: str, kind: "string" });
    else if (num !== undefined) tokens.push({ text: num, kind: "number" });
    else if (word !== undefined)
      tokens.push({ text: word, kind: KEYWORDS.has(word) ? "keyword" : "plain" });
    else if (space !== undefined) tokens.push({ text: space, kind: "plain" });
    else if (other !== undefined) tokens.push({ text: other, kind: "plain" });
  }
  return tokens;
}
