import Link from "next/link";
import { Logo } from "./Logo";

export function Footer() {
  return (
    <footer className="relative border-t border-ink-200/10 py-16">
      <div className="mx-auto max-w-7xl px-6">
        <div className="grid grid-cols-2 gap-8 md:grid-cols-4">
          <div className="col-span-2 md:col-span-1">
            <Link href="/" className="inline-flex">
              <Logo size={34} />
            </Link>
            <p className="mt-4 max-w-xs text-sm text-ink-300">
              Local-first memory for AI coding agents. Persistent, project-scoped, semantic.
            </p>
          </div>

          <FooterCol title="Product">
            <FooterLink href="/features">All features</FooterLink>
            <FooterLink href="/#install">Install</FooterLink>
            <FooterLink href="/#mcp">MCP tools</FooterLink>
            <FooterLink href="/#graph">Graph view</FooterLink>
          </FooterCol>

          <FooterCol title="Stack">
            <FooterLink href="https://tauri.app" external>Tauri 2</FooterLink>
            <FooterLink href="https://github.com/RyanCodrai/turbovec" external>turbovec 4-bit</FooterLink>
            <FooterLink href="https://github.com/modelcontextprotocol/rust-sdk" external>rmcp</FooterLink>
            <FooterLink href="https://tree-sitter.github.io" external>tree-sitter</FooterLink>
          </FooterCol>

          <FooterCol title="Source">
            <FooterLink href="https://github.com/RyanCodrai/biturbo" external>GitHub</FooterLink>
            <FooterLink href="https://github.com/RyanCodrai/biturbo/blob/main/LICENSE" external>MIT license</FooterLink>
            <FooterLink href="https://github.com/RyanCodrai/biturbo/blob/main/INSTRUCTIONS.md" external>INSTRUCTIONS.md</FooterLink>
            <FooterLink href="https://github.com/RyanCodrai/biturbo/issues" external>Issues</FooterLink>
          </FooterCol>
        </div>

        <div className="mt-12 flex flex-col items-start justify-between gap-4 border-t border-ink-200/10 pt-6 font-mono text-xs text-ink-300 md:flex-row md:items-center">
          <div>© 2025 biTurbo · built with Rust + React · no analytics, no cookies, no BS</div>
          <div className="flex items-center gap-4">
            <span className="flex items-center gap-1.5">
              <span className="h-1.5 w-1.5 animate-pulse-glow rounded-full bg-moss" />
              all systems normal
            </span>
          </div>
        </div>
      </div>
    </footer>
  );
}

function FooterCol({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h4 className="font-mono text-[10px] uppercase tracking-[0.2em] text-ink-300">{title}</h4>
      <ul className="mt-3 space-y-2">{children}</ul>
    </div>
  );
}

function FooterLink({ href, children, external }: { href: string; children: React.ReactNode; external?: boolean }) {
  if (external) {
    return (
      <li>
        <a
          href={href}
          target="_blank"
          rel="noopener noreferrer"
          className="text-sm text-ink-200/70 transition-colors hover:text-ink"
        >
          {children}
        </a>
      </li>
    );
  }
  return (
    <li>
      <Link href={href} className="text-sm text-ink-200/70 transition-colors hover:text-ink">
        {children}
      </Link>
    </li>
  );
}
