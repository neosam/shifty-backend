# Redesign 03 — TopBar & Layout

> Master plan: `../REDESIGN_PLAN.md`. Reference: `design_handoff_shifty/README.md` (Screens § 1) and `Shifty Preview.html` lines 173–230.

## Why

The TopBar appears on every page and exposes navigation, identity, and (after this change) the theme toggle. Done before page rewrites so each page is built into the new chrome from the start.

The current TopBar (`src/component/top_bar.rs`) is `bg-gray-800 text-white`, has 7 privilege-gated nav items, no active-route highlighting, no `Du bist [name]` pill (logout currently lives as a plain anchor with the username inline), no theme toggle, no fixed height. The new design demands a sticky 56 px header with theme awareness, accent-tinted active nav, a text-only identity pill, and a theme-cycle button.

## What Changes

- Rebuild `src/component/top_bar.rs`:
  - Sticky header, height 56 px, `bg-surface text-ink` with `border-b border-border`
  - Brand wordmark `Shifty` followed by accent-colored period (`.` in `text-accent`)
  - Non-prod environment hint stays inline next to the brand (preserved from current code)
  - Nav items, all privilege-gated as today (no scope creep — same visibility rules):
    - `Schichtplan` (sales OR shiftplanner)
    - `Meine Schichten` (sales)
    - `Meine Zeit` (paid && !hr) — preserved
    - `Jahresübersicht` (shiftplanner OR sales)
    - `Mitarbeiter` (hr)
    - `Benutzerverwaltung` (admin)
    - `Textvorlagen` (admin) — preserved
  - Active nav pill: `bg-accent-soft text-accent`, font-weight 600 — derived from `use_route::<Route>()`
  - Right side, in order:
    1. Theme-toggle button — `NavBtn`-styled (28×28), glyph reflects current `ThemeMode` (`☀` Light, `☾` Dark, `⌬` System), cycles on click via `cycle_theme` from `01`
    2. Identity pill `Du bist [name]` — text only, `surface-alt` rounded-full container, **no avatar circle, no initials**
    3. Logout / Login link — moved to a dropdown opened from the identity pill (see decision in `design.md`)
- Mobile (`<720px`):
  - Burger `☰` / close `✕` collapses nav into a dropdown panel below the bar (similar to current behavior, restyled)
  - Identity pill collapses to just the name, theme toggle stays visible
- Non-production warning banner (current code lines 133–141) stays as a separate `div` below the TopBar — visual style refreshed to use `bg-warn-soft text-warn`
- Wire theme-toggle button to dispatch `ThemeAction::SetMode(cycle_theme(current))` from `01`

## Out of scope

- Per-page chrome and toolbars (live with each page in `05`–`09`)
- Changing privilege rules or which user sees which nav item
- Login flow (still navigates to existing `/authenticate` route)
- Removing or renaming routes

## Capabilities

### New
- `top-bar`: sticky header with brand, privilege-gated navigation with active-route highlighting, theme toggle, identity pill with logout, non-prod warning banner, and mobile burger menu

## Impact

- Files: `src/component/top_bar.rs` (full rewrite), uses `Btn`/`NavBtn` from `02` and theme service from `01`
- Visual ripple across every page that imports `TopBar`; no logic change to pages
- Tests: privilege-gated nav visibility (matrix), active-route highlight selection, theme cycling, mobile burger toggle
