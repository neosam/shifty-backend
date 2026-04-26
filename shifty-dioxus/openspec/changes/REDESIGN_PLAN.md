# Shifty UI Redesign — Master Plan

This file tracks the overall plan for migrating `shifty-dioxus` to the new design described in `design_handoff_shifty/`. Each numbered change below is its own OpenSpec proposal — see the corresponding directory for details.

**Reference design**: `design_handoff_shifty/README.md` and `Shifty Preview.html`.

## Order of execution

Each change depends on all earlier ones. `01` is foundation and blocks everything else; `02–04` are cross-cutting building blocks; `05–09` are individual pages, in ascending complexity.

| # | Change | Depends on | Notes |
|---|---|---|---|
| 01 | `redesign-01-design-tokens` | — | CSS variables, Tailwind aliases, fonts, light/dark/system theming infra |
| 02 | `redesign-02-atom-components` | 01 | `Btn` (4 variants), `PersonChip`, `TupleRow`, `navBtn` |
| 03 | `redesign-03-topbar-layout` | 01, 02 | New `TopBar` (56 px) + theme toggle + mobile burger |
| 04 | `redesign-04-modal-variants` | 01, 02 | `Modal` with `auto`/`center`/`sheet`/`bottom`, `Field`/`TextInput`/`SelectInput`/`TextareaInput` |
| 05 | `redesign-05-page-myshifts` | 01–04 | Easiest page first — week cards layout |
| 06 | `redesign-06-page-overview` | 01–04 | Year overview chart + 10-week table |
| 07 | `redesign-07-page-employees` | 01–04 | Two-column layout, 3-column sub-grid, **new** weekly histogram |
| 08 | `redesign-08-page-usermgmt` | 01–04 | Tab split: SalesPerson / Benutzer |
| 09 | `redesign-09-page-shiftplan` | 01–04 | Most complex — toolbar, tabs, week grid with single +/− button per cell |
| 99 | `redesign-99-cleanup` | 05–09 all merged | Remove legacy `Modal`/`Button`/`TextInput`, drop `Form*` prefix, remove unused Tailwind custom colors |

## Out of scope (explicit)

These were considered and intentionally **not** included in the redesign:

- **Block view in employee weekly detail** — keep showing hours-per-day with category, not time blocks (`from–to`). Reason: backend delivers `WorkingHoursDay { date, hours, category }`; block view would require booking aggregation and lose category info. See `07`.
- **Per-cell red conflict tinting** — not present today, not added in `09`. Conflicts continue to surface as the list above the grid. Optional future: column-wide tinting.
- **Tag (single-day) view variant** — toggle exists in current code; redesign keeps it as-is, no logic added.
- **In-app zoom/density control** — current zoom dropdown stays; no new density token toggle.

## Cross-cutting decisions

- **No initials anywhere.** No `LB`/`MH`/avatar circles with text.
- **PersonChip text always dark ink**, even in dark mode (pastel colors are designed for dark text).
- **Sollstunden** stays in employee detail / Arbeitsverträge — never in SalesPerson admin.
- **Theme** persists in `localStorage` under key `shifty-theme`, applied via `<html data-theme="...">`.
- **Charts**: keep existing SVG `weekly_overview_chart.rs`; refactor to use tokens, dashed required-line, current-week highlight. New SVG histogram for employee weekly view in `07`.

## Why a separate cleanup change at the end

Foundation changes (`02`, `04`) intentionally ship new components alongside legacy ones to stay small and additive. Page changes (`05`–`09`) migrate consumers incrementally. Once all consumers are migrated, the legacy components are dead code and the transitional `Form*` prefix on form atoms can be dropped. Doing this housekeeping in a dedicated change keeps each preceding change focused and reviewable, and makes it explicit when the migration is "really done."

`99` only runs after every page change is merged and archived. If a page change is reverted or postponed, `99` waits.

## Status

- All 9 redesign changes plus `99` cleanup currently exist as proposals; flesh out in execution order. `01`–`04` are fully drafted; `05`–`09` and `99` are skeletons.
- Each skeleton lists the most important `What Changes` bullets so we don't lose context.
