import type { Metadata } from "next";
import { Inter_Tight, JetBrains_Mono } from "next/font/google";
import "./globals.css";

const interTight = Inter_Tight({
  subsets: ["latin"],
  variable: "--font-body",
  display: "swap",
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-mono",
  display: "swap",
});

export const metadata: Metadata = {
  title: "biTurbo — Local-first memory for AI coding agents",
  description:
    "Persistent, project-scoped, semantic memory that lives on your disk. No cloud, no SaaS, no embedding leakage. Open source, MIT, single binary.",
  metadataBase: new URL("https://biturbo.dev"),
  icons: {
    icon: [{ url: "/favicon-32.png", sizes: "32x32", type: "image/png" }],
    apple: { url: "/apple-touch-icon.png", sizes: "180x180", type: "image/png" },
  },
  openGraph: {
    title: "biTurbo — Local-first memory for AI coding agents",
    description:
      "Persistent, project-scoped, semantic memory that lives on your disk. One Rust binary, MCP-native, 4-bit compressed.",
    type: "website",
    images: [{ url: "/logo-full-600.png", width: 600, height: 218, alt: "biTurbo" }],
  },
  twitter: {
    card: "summary_large_image",
    title: "biTurbo",
    description: "Local-first memory for AI coding agents. Open source. MIT.",
    images: ["/logo-full-600.png"],
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className={`${interTight.variable} ${jetbrainsMono.variable}`}>
      <head>
        {/* Cabinet Grotesk from Fontshare — a free, gorgeous editorial display font */}
        <link rel="preconnect" href="https://api.fontshare.com" crossOrigin="" />
        <link
          rel="stylesheet"
          href="https://api.fontshare.com/v2/css?f[]=cabinet-grotesk@800,700,500,400&f[]=switzer@400,500,600&display=swap"
        />
      </head>
      <body className="grain antialiased">
        {children}
      </body>
    </html>
  );
}
