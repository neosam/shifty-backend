# Redesign — Typography Bump

> Reference: `shifty-design/project/Shifty Preview.html` (the design handoff bundle, primary preview file).

## Why

User feedback on the redesigned frontend reports that text is too small to read comfortably. The design handoff in `shifty-design/` ships an updated typography scale (16 px body, 11/12/14/16/22 px component sizes, mobile body 15 px) that addresses this — but the current `shifty-dioxus` implementation still sits on the older, more compact scale (lots of `text-xs` for body content, ad-hoc `font-size: 10px` inline styles, smaller headlines and a missing mobile override).

This change aligns the frontend's typography 1:1 with the reference design. No creative interpretation, no alternative scale — the design is the source of truth.

## What Changes

- Add a canonical typography scale to `tailwind.config.js` `theme.extend.fontSize` matching the `kontor` direction in `shifty-design/project/tokens.jsx` and the explicit sizes used throughout `Shifty Preview.html`. Keys: `micro` (11 px), `small` (12 px), `body` (14 px), `lg` (16 px), `h2` (18 px), `h1` (22 px), `display` (32 px), each with the matching line-height, weight, and letter-spacing where defined.
- Set the html/body baseline in `input.css`: `body { font-size: 16px; line-height: 1.5; }` and a mobile media query `@media (max-width: 720px) { body { font-size: 15px; } }` to match the design.
- Sweep `src/` to replace the implementation's typography with the new tokens:
  - Remove all inline `style: "font-size: …"` declarations; they are replaced by Tailwind classes from the new scale (or the body baseline).
  - Replace `text-xs` (12 px) used for body text in tables, lists, and detail views with `text-body` (14 px) where the design uses 14 px.
  - Keep `text-xs` only where the design uses 12 px (sub-body, muted captions, table column metadata).
  - Replace `text-xs` used for uppercase eyebrow / section labels with the new `text-micro` (11 px) so tracking and weight match the design.
  - Replace page headlines using `text-xl` (20 px) with `text-h1` (22 px).
  - Replace modal titles using `text-lg` (18 px) with `text-lg` in the new sense (16 px) — the new token name resolves to 16 px to match the design's modal title size; existing `text-lg` call sites in modals must be re-checked, not blindly kept.
  - Update tracking/letter-spacing where the design specifies it (e.g., uppercase eyebrows: `tracking-wider`).
- Verify visually against `shifty-design/project/Shifty Preview.html` (read the source — do not run the prototype) for: TopBar, WeekView, Employees page, MyShifts page, BillingPeriods page, modals, settings, working-hours mini-overview.

**BREAKING**: existing components using `text-xs` / `text-sm` / `text-lg` / `text-xl` Tailwind classes will be migrated to the new token names. Old class names continue to work (Tailwind's defaults are not removed), but the canonical idiom for the codebase becomes the new token names. New code should use the tokens.

## Capabilities

### New Capabilities
- `typography`: canonical type scale (sizes, line-heights, weights, tracking) for the app, exposed as Tailwind utility classes and a body baseline in `input.css`, sourced 1:1 from the design handoff.

### Modified Capabilities
<!-- None: typography was previously implicit via raw Tailwind defaults; this change introduces the spec rather than modifying existing requirements. -->

## Impact

- Files: `tailwind.config.js`, `input.css`, broad sweep across `src/component/**/*.rs`, `src/page/**/*.rs`.
- No backend changes.
- No new runtime dependencies; new web fonts are already loaded via `index.html`.
- No `rest-types` changes.
- Visual regression risk is the main concern. Mitigation: the design is fully specified in HTML/CSS in `shifty-design/`, so each page can be diffed against its design counterpart during the sweep. Compile + test suite remain green throughout.
