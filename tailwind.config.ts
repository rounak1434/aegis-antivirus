import type { Config } from "tailwindcss";

/**
 * Token mirror of design-prototype/css/app.css :root.
 * The CSS variables in src/styles.css are the single source of truth; these
 * Tailwind aliases simply expose the same values to utility classes so screens
 * can use either the ported semantic classes (.card, .btn, …) or utilities and
 * stay pixel-identical to the prototype.
 */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        bg: "var(--bg)",
        "bg-2": "var(--bg-2)",
        surface: "var(--surface)",
        "surface-2": "var(--surface-2)",
        raised: "var(--raised)",
        fg: "var(--fg)",
        "fg-2": "var(--fg-2)",
        muted: "var(--muted)",
        faint: "var(--faint)",
        border: "var(--border)",
        "border-2": "var(--border-2)",
        accent: "var(--accent)",
        "accent-dim": "var(--accent-dim)",
        danger: "var(--danger)",
        warn: "var(--warn)",
        info: "var(--info)"
      },
      fontFamily: {
        display: ['Georgia', '"Times New Roman"', "serif"],
        sans: ['Arial', "system-ui", "-apple-system", "sans-serif"],
        mono: ["ui-monospace", '"JetBrains Mono"', "Menlo", "monospace"]
      },
      borderRadius: {
        sm: "8px",
        DEFAULT: "8px",
        lg: "12px"
      },
      spacing: {
        sidebar: "236px",
        topbar: "56px",
        winbar: "36px"
      }
    }
  },
  plugins: []
} satisfies Config;
