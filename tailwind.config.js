/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        serif: ["Fraunces", "Georgia", "serif"],
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "ui-monospace", "monospace"],
      },
      colors: {
        bg: "#0F0E0C",
        surface: "#181613",
        "surface-2": "#211E1A",
        "surface-3": "#2A2620",
        border: "#322D26",
        "border-subtle": "#221F1B",
        text: "#E8E2D6",
        "text-muted": "#94897A",
        "text-dim": "#6B6258",
        accent: {
          DEFAULT: "#D4A574",
          soft: "rgba(212, 165, 116, 0.12)",
          glow: "rgba(212, 165, 116, 0.25)",
        },
        success: "#8FB87D",
        warning: "#D4B574",
        danger: "#C77B6E",
      },
      keyframes: {
        pulse_dot: {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.4" },
        },
        fade_in: {
          from: { opacity: "0", transform: "translateY(4px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
      },
      animation: {
        pulse_dot: "pulse_dot 2s ease-in-out infinite",
        fade_in: "fade_in 200ms ease-out",
      },
    },
  },
  plugins: [],
};
