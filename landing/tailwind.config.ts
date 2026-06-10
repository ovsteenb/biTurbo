import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  safelist: [
    "bg-moss", "bg-amber", "bg-sky", "bg-lilac", "bg-ink", "bg-sand",
    "text-moss", "text-amber", "text-sky", "text-lilac", "text-ink",
    "border-moss", "border-amber", "border-sky", "border-lilac", "border-ink",
  ],
  theme: {
    extend: {
      colors: {
        ink: {
          DEFAULT: "#ecebe3",
          50: "#f7f6f2",
          100: "#ecebe3",
          200: "#d4d2c4",
          300: "#a8a594",
          400: "#7a7866",
          500: "#52513f",
          600: "#36352a",
          700: "#24241c",
          800: "#16160f",
          900: "#0c0c07",
        },
        cream: "#f4f1e8",
        bone: "#ebe7d8",
        moss: "#8FB87D",
        amber: "#D4A574",
        sky: "#7DC4E4",
        lilac: "#C7A0E0",
        sand: "#E8E2D6",
      },
      fontFamily: {
        display: ['"Cabinet Grotesk"', "ui-sans-serif", "system-ui", "sans-serif"],
        body: ['"Inter Tight"', "ui-sans-serif", "system-ui", "sans-serif"],
        mono: ['"JetBrains Mono"', "ui-monospace", "SFMono-Regular", "monospace"],
      },
      animation: {
        "marquee": "marquee 40s linear infinite",
        "spin-slow": "spin 18s linear infinite",
        "drift": "drift 16s ease-in-out infinite",
        "pulse-glow": "pulse-glow 4s ease-in-out infinite",
      },
      keyframes: {
        marquee: {
          "0%": { transform: "translateX(0%)" },
          "100%": { transform: "translateX(-50%)" },
        },
        drift: {
          "0%, 100%": { transform: "translate(0, 0)" },
          "50%": { transform: "translate(8px, -12px)" },
        },
        "pulse-glow": {
          "0%, 100%": { opacity: "0.55" },
          "50%": { opacity: "0.9" },
        },
      },
      backgroundImage: {
        "grain": "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.4'/%3E%3C/svg%3E\")",
      },
    },
  },
  plugins: [],
};

export default config;
