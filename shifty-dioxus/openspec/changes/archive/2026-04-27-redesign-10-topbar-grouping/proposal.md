# Redesign 10 — TopBar Admin Grouping ("Verwaltung")

> Reference: `shifty-design/project/Shifty Preview.html` lines 167–311 (the `TopBar` component and `ADMIN_GROUP` constant). Master plan: `../REDESIGN_PLAN.md`. Builds on the `top-bar` capability introduced by `redesign-03-topbar-layout`.

## Why

The reference design groups administrative routes under a single "Verwaltung" dropdown trigger so the bar stays narrow enough to never need horizontal scroll on a laptop. The shipped TopBar (`src/component/top_bar.rs`) ignores this grouping and renders all eight privilege-gated items flat, which overflows on common laptop widths once a user has the union of `sales + hr + admin` privileges (Schichtplan, Meine Schichten, Meine Zeit, Jahresübersicht, Mitarbeiter, Abrechnungszeiträume, Benutzerverwaltung, Textvorlagen). The grouping was missing from `redesign-03-topbar-layout`'s scope and never made it into the implementation.

## What Changes

- Split the privilege-gated nav items into two visual buckets, matching the design's `ADMIN_GROUP` pattern:
  - Top-level: `Schichtplan`, `Meine Schichten`, `Meine Zeit`, `Jahresübersicht`
  - "Verwaltung" dropdown: `Mitarbeiter`, `Abrechnungszeiträume`, `Benutzerverwaltung`, `Textvorlagen`
- Add a "Verwaltung" trigger button at the end of the desktop nav that opens an account-style dropdown listing the admin items (matching `Shifty Preview.html` lines 247–297).
- The trigger button's label SHALL switch to the active admin item's label when one of its children is active (e.g. shows "Mitarbeiter" instead of "Verwaltung"), and SHALL adopt the active-pill styling (`bg-accent-soft text-accent`).
- The trigger SHALL include a `▾` chevron at the end (`fontSize: 11, opacity: 0.7`).
- The dropdown panel uses `position: fixed` aligned under the trigger via `getBoundingClientRect` (top + 4 px), `min-width: 220 px`, surface background, border, rounded corners, and a soft shadow — exactly as the reference defines (lines 270–295).
- Outside-click and `Escape` close the dropdown; the trigger itself is excluded from outside-click detection so opening cannot immediately re-close.
- Mobile (<720 px) flattens the dropdown into an inline labelled section, NOT a nested dropdown. The grouped items appear inline in the burger panel under a section header reading `VERWALTUNG` — uppercase, font-weight 700, font-size 11 px, letter-spacing `0.06em`, color `var(--ink-muted)`, with a top border separator (matching lines 299–310).
- The "Verwaltung" trigger SHALL be hidden entirely when none of its admin items are visible to the current user (e.g. a `sales`-only user sees no trigger).
- Hide the entire TopBar nav (group + items) only when there are zero visible items — preserve current empty-state behavior.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `top-bar`: navigation rendering changes — admin items move from flat top-level into a "Verwaltung" dropdown trigger with active-state propagation; mobile burger gains a labelled section instead of a nested dropdown.

## Impact

- Files: `src/component/top_bar.rs` (rewrite the `nav` block; introduce a small `AdminGroup` rendering helper or inline section)
- New i18n key: `Key::TopBarAdminGroupLabel` with `De: "Verwaltung"`, `En: "Administration"`, `Cs: "Správa"`
- Tests in `src/component/top_bar.rs` — extend the existing matrix to assert: which items render as top-level vs. grouped, trigger visibility per privilege, trigger label substitution when an admin route is active, and active-pill state on the trigger
- No backend or routing changes
- No change to privilege rules or which items each privilege unlocks
