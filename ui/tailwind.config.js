/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        "primary": "#135bec",
        "primary-hover": "#104bc4",
        "primary-dark": "#0f4bc4",
        "background-light": "#f6f6f8",
        "background-dark": "#0b0e14",
        "background-card": "#151a23",
        "background-sidebar": "#101622",
        "panel-dark": "#18202f",
        "surface-dark": "#1a2234",
        "surface-darker": "#151b2b",
        "terminal-bg": "#0d1117",
        "text-secondary": "#94a3b8",
        "border-color": "#2a3140",
        "border-dark": "#2a313e",
      },
      fontFamily: {
        "display": ["Inter", "sans-serif"],
        "mono": ["JetBrains Mono", "monospace"]
      },
      borderRadius: {
        "DEFAULT": "0.25rem",
        "lg": "0.5rem",
        "xl": "0.75rem",
        "2xl": "1rem",
        "full": "9999px"
      },
    },
  },
  plugins: [],
}
