/** @type {import('tailwindcss').Config} */

// Note: Tailwind only ships utility classes for class names that appear
// statically in the source files. When constructing class strings dynamically
// (e.g. `format!("bg-{}-soft", state)` in Rust), prefer static `if`/`match`
// branches that yield literal class strings. If a dynamic pattern is
// unavoidable, add the resulting classes to the `safelist` below.
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
        // Design token aliases — values resolved via CSS variables in input.css
        bg: 'var(--bg)',
        surface: 'var(--surface)',
        'surface-alt': 'var(--surface-alt)',
        'surface-2': 'var(--surface-2)',
        border: 'var(--border)',
        'border-strong': 'var(--border-strong)',
        ink: 'var(--ink)',
        'ink-soft': 'var(--ink-soft)',
        'ink-muted': 'var(--ink-muted)',
        accent: 'var(--accent)',
        'accent-ink': 'var(--accent-ink)',
        'accent-soft': 'var(--accent-soft)',
        good: 'var(--good)',
        'good-soft': 'var(--good-soft)',
        warn: 'var(--warn)',
        'warn-soft': 'var(--warn-soft)',
        bad: 'var(--bad)',
        'bad-soft': 'var(--bad-soft)',
        'modal-veil': 'var(--modal-veil)',
      },
      borderRadius: {
        sm: 'var(--r-sm)',
        md: 'var(--r-md)',
        lg: 'var(--r-lg)',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
        mono: ['"JetBrains Mono"', 'ui-monospace', 'Menlo', 'monospace'],
      },
      // Canonical typography scale — sourced 1:1 from
      // shifty-design/project/Shifty Preview.html and the `kontor` direction
      // in shifty-design/project/tokens.jsx. See
      // openspec/changes/redesign-typography-bump/specs/typography/spec.md
      // for the canonical table and binding rules.
      //
      // Note: `lg` intentionally resolves to 16px (not Tailwind's default
      // 18px). This is the modal-title size in the design.
      fontSize: {
        micro:   ['11px', { lineHeight: '14px', fontWeight: '600', letterSpacing: '0.06em' }],
        small:   ['12px', { lineHeight: '16px', fontWeight: '500', letterSpacing: '0' }],
        body:    ['14px', { lineHeight: '20px', fontWeight: '400', letterSpacing: '0' }],
        lg:      ['16px', { lineHeight: '22px', fontWeight: '600', letterSpacing: '0' }],
        h2:      ['18px', { lineHeight: '24px', fontWeight: '600', letterSpacing: '-0.005em' }],
        h1:      ['22px', { lineHeight: '28px', fontWeight: '600', letterSpacing: '-0.01em' }],
        display: ['32px', { lineHeight: '38px', fontWeight: '700', letterSpacing: '-0.02em' }],
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
    // State-dependent classes constructed dynamically
    "bg-bad-soft",
    "bg-warn-soft",
    "bg-accent-soft",
    "text-bad",
    "text-warn",
    "text-good",
    "border-bad",
    "border-warn",
    "border-accent",
  ]
};
