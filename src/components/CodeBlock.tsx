import { tokenizeCode } from "../lib/format";

const TOKEN_CLASS: Record<string, string> = {
  keyword: "text-violet-300",
  string: "text-emerald-300",
  comment: "text-text-dim italic",
  number: "text-amber-300",
  plain: "text-text-muted",
};

interface CodeBlockProps {
  code: string;
  maxLines?: number;
  className?: string;
}

export function CodeBlock({ code, maxLines, className }: CodeBlockProps) {
  const lines = code.split("\n");
  const shown = maxLines ? lines.slice(0, maxLines) : lines;
  const truncated = maxLines != null && lines.length > maxLines;
  const lineClass = maxLines ? "code-block-line" : "code-block-line wrap";

  return (
    <pre className={`code-block ${className ?? ""}`}>
      <code>
        {shown.map((line, i) => (
          <div key={i} className={lineClass}>
            {tokenizeCode(line).map((tok, j) => (
              <span key={j} className={TOKEN_CLASS[tok.kind]}>
                {tok.text}
              </span>
            ))}
            {line.length === 0 && "\u00A0"}
          </div>
        ))}
        {truncated && <div className="code-block-line text-text-dim">…</div>}
      </code>
    </pre>
  );
}
