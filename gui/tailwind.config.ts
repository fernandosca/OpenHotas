import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: ["class"],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        border: "var(--theme-border)",
        input: "var(--theme-border-strong)",
        ring: "rgb(var(--theme-accent-rgb) / <alpha-value>)",
        background: "rgb(var(--theme-main-rgb) / <alpha-value>)",
        foreground: "rgb(var(--theme-text-primary-rgb) / <alpha-value>)",
        primary: {
          DEFAULT: "rgb(var(--theme-accent-rgb) / <alpha-value>)",
          foreground: "rgb(var(--theme-text-on-accent-rgb) / <alpha-value>)",
        },
        secondary: {
          DEFAULT: "rgb(var(--theme-surface-rgb) / <alpha-value>)",
          foreground: "rgb(var(--theme-text-primary-rgb) / <alpha-value>)",
        },
        destructive: {
          DEFAULT: "rgb(var(--theme-danger-rgb) / <alpha-value>)",
          foreground: "rgb(var(--theme-text-primary-rgb) / <alpha-value>)",
        },
        muted: {
          DEFAULT: "rgb(var(--theme-surface-rgb) / <alpha-value>)",
          foreground: "rgb(var(--theme-text-muted-rgb) / <alpha-value>)",
        },
        accent: {
          DEFAULT: "rgb(var(--theme-raised-rgb) / <alpha-value>)",
          foreground: "rgb(var(--theme-text-primary-rgb) / <alpha-value>)",
        },
        popover: {
          DEFAULT: "rgb(var(--theme-raised-rgb) / <alpha-value>)",
          foreground: "rgb(var(--theme-text-primary-rgb) / <alpha-value>)",
        },
        card: {
          DEFAULT: "rgb(var(--theme-panel-rgb) / <alpha-value>)",
          foreground: "rgb(var(--theme-text-primary-rgb) / <alpha-value>)",
        },
        // ── HOTAS design tokens ──────────────────────────────────
        // Dark theme (cockpit / HUD aesthetic)
        hud: {
          bg:       "var(--hud-bg)",
          surface:  "var(--hud-surface)",
          surface2: "var(--hud-surface2)",
          raised:   "var(--hud-raised)",
          border:   "var(--hud-border)",
          border2:  "var(--hud-border2)",
        },
        content: {
          primary: "var(--theme-text-primary)",
          muted:   "var(--theme-text-muted)",
          dim:     "var(--theme-text-dim)",
          inverse: "var(--theme-text-on-accent)",
        },
        axis: {
          x:    "var(--axis-x)",
          y:    "var(--axis-y)",
          tw:   "var(--axis-tw)",
        },
        // Semantic
        ok:    "rgb(var(--theme-ok-rgb) / <alpha-value>)",
        warn:  "rgb(var(--theme-warning-rgb) / <alpha-value>)",
        danger:"rgb(var(--theme-danger-rgb) / <alpha-value>)",
        // Accent
        cyan: {
          DEFAULT: "rgb(var(--theme-accent-rgb) / <alpha-value>)",
          dim:     "var(--cyan-dim)",
          glow:    "var(--cyan-glow)",
        },
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "Fira Code", "Courier New", "monospace"],
      },
      borderRadius: {
        lg: "0.625rem",
        md: "0.5rem",
        sm: "0.375rem",
      },
      gridTemplateColumns: {
        16: "repeat(16, minmax(0, 1fr))",
      },
      keyframes: {
        "pulse-dot": {
          "0%,100%": { opacity: "1" },
          "50%":     { opacity: "0.35" },
        },
        "fade-in": {
          from: { opacity: "0", transform: "translateY(4px)" },
          to:   { opacity: "1", transform: "translateY(0)" },
        },
      },
      animation: {
        "pulse-dot": "pulse-dot 2s ease-in-out infinite",
        "fade-in":   "fade-in 0.2s ease-out",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
};

export default config;
