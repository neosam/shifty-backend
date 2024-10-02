/** @type {import('tailwindcss').Config} */

const colors = require("tailwindcss/colors");
module.exports = {
  mode: "all",
  content: ["./src/**/*.{rs,html,css}", "./dist/**/*.html"],
  theme: {
    extend: {
      screens: {
        print: { raw: 'print' },
        screen: { raw: 'screen' },
      },
      colors: {
        missingColor: colors.amber[200],
        blockedColor: colors.red[500],
      },
    },
  },
  plugins: [],
  safelist: [
    "bg-red-200",
    "print:bg-white",
    "cursor-not-allowed",
    "text-green-800",
    "text-red-800",
    "bg-missingColor",
    "bg-blockedColor"
  ]
};
