/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{svelte,js,ts}"],
  theme: {
    extend: {
      colors: {
        fluent: {
          bg: "#202020",
          card: "#2b2b2b",
          cardHover: "#323232",
          border: "#3e3e3e",
          accent: "#60cdff",
          accentHover: "#70d6ff",
          subtext: "#a0a0a0",
          danger: "#ff6b6b",
          warning: "#f59e0b",
          success: "#10b981",
        },
      },
      fontFamily: {
        sans: ['"Segoe UI Variable"', '"Segoe UI"', 'system-ui', '-apple-system', 'sans-serif'],
      },
    },
  },
  plugins: [],
};
