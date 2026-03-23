import type { Config } from "tailwindcss";

const config: Config = {
  darkMode: ["class"],
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        meridian: {
          primary: "#6366f1",
          "primary-dark": "#818cf8",
          surface: "#ffffff",
          "surface-dark": "#09090b",
          bg: "#fafafa",
          "bg-dark": "#18181b",
          border: "#e4e4e7",
          "border-dark": "#27272a",
          text: "#18181b",
          "text-dark": "#fafafa",
          muted: "#71717a",
        },
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "monospace"],
      },
    },
  },
  plugins: [],
};

export default config;
