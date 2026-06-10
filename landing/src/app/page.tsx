import { Nav } from "@/components/Nav";
import { Hero } from "@/components/Hero";
import { ZoomSection } from "@/components/ZoomSection";
import { MemoryVisual } from "@/components/visuals/MemoryVisual";
import { MCPVisual } from "@/components/visuals/MCPVisual";
import { GraphVisual } from "@/components/visuals/GraphVisual";
import { SpeedVisual } from "@/components/visuals/SpeedVisual";
import { OSSVisual } from "@/components/visuals/OSSVisual";
import { Marquee } from "@/components/Marquee";
import { InstallSection } from "@/components/InstallSection";
import { ComparisonSection } from "@/components/ComparisonSection";
import { CTASection } from "@/components/CTASection";
import { Footer } from "@/components/Footer";

export default function Home() {
  return (
    <main className="relative">
      <Nav />

      <Hero />

      <Marquee />

      {/* Feature 01 — Memory */}
      <ZoomSection
        id="memory"
        index="01"
        eyebrow="memory"
        variant="moss"
        align="left"
        title={
          <>
            Memories that<br />
            <span className="text-moss">survive reboots.</span>
          </>
        }
        description={
          <>
            Every <code className="font-mono text-moss">remember()</code> lands in SQLite on your disk.
            Every <code className="font-mono text-moss">search()</code> runs against a per-project
            turbovec index. testy&apos;s memories never bleed into scout-qa. The index
            self-maintains — scheduled decay, dedup, merge. The data doesn&apos;t rot while
            you&apos;re not looking.
          </>
        }
        visual={<MemoryVisual />}
      />

      {/* Feature 02 — MCP */}
      <ZoomSection
        id="mcp"
        index="02"
        eyebrow="mcp"
        variant="sky"
        align="right"
        title={
          <>
            19 tools.<br />
            <span className="text-sky">One stdio socket.</span>
          </>
        }
        description={
          <>
            biTurbo speaks Model Context Protocol out of the box. <code className="font-mono text-sky">remember</code>,
            {" "}<code className="font-mono text-sky">forget</code>, <code className="font-mono text-sky">search</code>,
            {" "}<code className="font-mono text-sky">recall_for_context</code> — the works. Plugs into Claude
            Code, Cursor, Cline, Mavis, anything that speaks MCP. No proxy server, no SaaS relay.
            Your embeddings never leave your machine.
          </>
        }
        visual={<MCPVisual />}
      />

      {/* Feature 03 — Graph */}
      <ZoomSection
        id="graph"
        index="03"
        eyebrow="graph"
        variant="lilac"
        align="left"
        title={
          <>
            Drop a folder.<br />
            <span className="text-lilac">Get a graph.</span>
          </>
        }
        description={
          <>
            Tree-sitter walks your Rust, TypeScript, Python, Go, JS — chunks functions,
            embeds each one, and lays them out in a Barnes–Hut graph view that runs in
            a Web Worker. 3,000+ nodes, viewport-culled, filter switches cancel stale
            requests. The seed renders in under 5ms; the worker refines in the background.
          </>
        }
        visual={<GraphVisual />}
      />

      {/* Feature 04 — Speed */}
      <ZoomSection
        id="speed"
        index="04"
        eyebrow="speed"
        variant="amber"
        align="right"
        title={
          <>
            16× smaller.<br />
            <span className="text-amber">1M in laptop RAM.</span>
          </>
        }
        description={
          <>
            turbovec 4-bit quantisation crushes float32 vectors to a quarter of their
            original size per dimension. A million BGE-small-en embeddings — the kind
            that would balloon a FAISS index to gigabytes — fit comfortably in laptop
            RAM. Recall k=8 in under 2ms. Cold start of the binary in under 50ms.
          </>
        }
        visual={<SpeedVisual />}
      />

      {/* Feature 05 — Open Source */}
      <ZoomSection
        id="oss"
        index="05"
        eyebrow="open source"
        variant="lilac"
        align="left"
        title={
          <>
            MIT licensed.<br />
            <span className="text-lilac">Free forever.</span>
          </>
        }
        description={
          <>
            No pro tier, no &quot;team plan&quot;, no usage-based pricing hidden in a footer.
            The whole codebase is on GitHub — Rust backend, React frontend, MCP server,
            the smoke test, the docs. Fork it, ship it, vendor it. We just ask for a star.
          </>
        }
        visual={<OSSVisual />}
      />

      <InstallSection />

      <ComparisonSection />

      <CTASection />

      <Footer />
    </main>
  );
}
