# Redesign 01 — Design Tokens & Theming

> Master plan: `../REDESIGN_PLAN.md`. Reference: `design_handoff_shifty/README.md` (Design Tokens, Theming behavior sections).

## Why

The current `input.css` has no design tokens and `tailwind.config.js` only defines two project-specific colors (`missingColor`, `blockedColor`). The reference design uses a shared CSS-variable system with light/dark/system theming, which all subsequent redesign steps (`02`–`09`) depend on.

This change adds the foundation only: tokens, theming infrastructure, and font loading. Existing pages keep their current Tailwind classes — token migration happens per-page in later changes.

## What Changes

- Add CSS variables to `input.css` for both `:root` (light) and `[data-theme="dark"]`:
  - Surfaces: `--bg`, `--surface`, `--surface-alt`, `--surface-2`
  - Borders: `--border`, `--border-strong`
  - Ink: `--ink`, `--ink-soft`, `--ink-muted`
  - Accent: `--accent`, `--accent-ink`, `--accent-soft`
  - Semantic: `--good`/`--good-soft`, `--warn`/`--warn-soft`, `--bad`/`--bad-soft`
  - Modal veil: `--modal-veil`
  - Radii: `--r-sm` (4 px), `--r-md` (6 px), `--r-lg` (10 px)
- Alias all tokens in `tailwind.config.js` `theme.extend.colors` and `theme.extend.borderRadius`
- Add `Inter` and `JetBrains Mono` to `theme.extend.fontFamily`; load them via `index.html`
- Extend `safelist` for state-dependent classes built dynamically: `bg-bad-soft`, `bg-warn-soft`, `bg-accent-soft`, `text-bad`, `text-warn`, `text-good`, `border-bad`, `border-warn`, `border-accent`
- Add a small theme service in `src/service/theme.rs`:
  - Three modes: `Light`, `Dark`, `System`
  - Persist to `localStorage` under key `shifty-theme`
  - On change, write resolved value to `<html data-theme="...">`
  - On system mode, listen to `prefers-color-scheme` mediaquery
- Pre-paint theme application: small inline `<script>` in `index.html` reads `localStorage` and applies `data-theme` to `<html>` before the WASM bundle initializes — prevents flash of wrong theme
- Wire theme service initialization into `src/app.rs` so the signal stays in sync with the DOM

## Out of scope

- TopBar theme toggle button (lives in `03`)
- Refactoring existing pages to consume tokens (lives in `05`–`09`)
- Self-hosting fonts (initial implementation uses Google Fonts; can be revisited later)

## Capabilities

### New
- `design-tokens`: shared CSS-variable system with light/dark/system theming and font loading

## Impact

- Files: `input.css`, `tailwind.config.js`, `index.html`, new `src/service/theme.rs`, `src/app.rs` for init, `src/service/mod.rs` for re-export
- Cargo: new dep `gloo-storage` (or use `web_sys` directly for `localStorage` and mediaquery — decision in `design.md`)
- No breaking changes — existing components keep their current Tailwind classes; tokens are additive
