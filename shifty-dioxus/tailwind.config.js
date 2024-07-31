/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  content: ["./src/**/*.{rs,html,css}", "./dist/**/*.html"],
  theme: {
    extend: {
      screens: {
        print: { raw: 'print' },
        screen: { raw: 'screen' },
      },
    },
  },
  plugins: [],
  safelist: [
    "bg-red-200",
    "print:bg-white",
    "cursor-not-allowed"
  ]
};
