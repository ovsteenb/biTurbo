# biTurbo landing page

Stunning, animated Next.js 14 landing page for [biTurbo](../). Static-exported, zero runtime cost, deploys to GitHub Pages for free.

## Stack

- **Next.js 14** (App Router, `output: "export"`) → pure static HTML/JS
- **React 18** + **TypeScript** strict
- **Tailwind 3.4** + custom design tokens in `tailwind.config.ts`
- **Framer Motion 11** for scroll-driven zoom sections
- **Cabinet Grotesk** (Fontshare) + **Inter Tight** (Google) + **JetBrains Mono** (Google)

## Develop

```bash
pnpm install
pnpm dev               # http://localhost:3001
```

## Build (static)

```bash
pnpm build             # → ./out  (plain HTML/CSS/JS, no Node server)
pnpm serve             # preview the static build
```

The `out/` directory is what you deploy.

## Deploy to GitHub Pages

Two options:

### Option A — automatic (recommended)

The workflow at `.github/workflows/deploy-landing.yml` builds and deploys on every push to `main` / `master` (when files under `landing/` change).

To enable it on a fresh repo:

1. Push this folder to a GitHub repo (e.g. `biturbo-landing`).
2. **Settings → Pages → Source:** select **GitHub Actions**.
3. Push to `main`. The workflow runs and publishes at
   `https://<owner>.github.io/biturbo-landing/`.

For a **user/org site** (`https://<owner>.github.io/` with no path), add a repo variable `LANDING_BASE_PATH` set to empty string, OR change the `NEXT_PUBLIC_BASE_PATH` env in the workflow.

### Option B — manual `gh-pages` branch

```bash
pnpm deploy            # builds + pushes ./out to the gh-pages branch
```

Then **Settings → Pages → Branch:** select `gh-pages` / `root`.

## Project structure

```
landing/
├── src/
│   ├── app/
│   │   ├── layout.tsx        Root layout, font loading, metadata
│   │   ├── globals.css       Design tokens, grain, grid lines
│   │   ├── page.tsx          Home (hero + 5 zoom sections + install + comparison + CTA)
│   │   └── features/page.tsx Features deep-dive (5 sections + 19 tools reference)
│   ├── components/
│   │   ├── Nav.tsx           Sticky nav with scroll-aware blur backdrop
│   │   ├── Hero.tsx          Massive "Your agents have memory." headline
│   │   ├── ZoomSection.tsx   Reusable scroll-zoom wrapper
│   │   ├── Marquee.tsx       Auto-scrolling word strip
│   │   ├── InstallSection.tsx 4-step install with terminal visual
│   │   ├── ComparisonSection.tsx  vs cloud-hosted / bolt-on
│   │   ├── CTASection.tsx    Bottom CTA
│   │   ├── Footer.tsx
│   │   └── visuals/          Procedural SVG/CSS visuals (no images)
│   │       ├── MemoryVisual.tsx
│   │       ├── MCPVisual.tsx
│   │       ├── GraphVisual.tsx
│   │       ├── SpeedVisual.tsx
│   │       ├── OSSVisual.tsx
│   │       └── InstallVisual.tsx
│   └── lib/cn.ts
├── .github/workflows/deploy-landing.yml
├── next.config.mjs
├── tailwind.config.ts
└── package.json
```

## Performance

- `output: "export"` — no Node server, no serverless functions
- First Load JS: **~149 kB** (home), **~145 kB** (`/features`)
- All visuals are inline SVG/CSS — zero image weight
- 5 static pages, all pre-rendered
- Total `out/` size: **~330 kB** uncompressed
