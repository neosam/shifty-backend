## Why

The Shiftplan page renders the working-hours mini overview only as a card grid. The new design (`shifty-design/project/Shifty Preview.html` lines 556–717) introduces a second layout — a compact table with totals — and a toggle between the two. Some users want a denser, sortable-looking view at a glance; others prefer the visual cards. A persistent per-browser preference avoids re-clicking the toggle on every reload.

## What Changes

- Add a table layout variant to the working-hours mini overview, alongside the existing card grid. Columns: employee, actual hours, target hours, difference, utilization (progress bar + percent). Footer row aggregates the totals.
- Add a segmented toggle ("Karten" / "Tabelle") above the overview that switches the rendered layout.
- Persist the chosen layout per browser via `localStorage` so the choice survives page reloads. Default is `cards` when no value is stored.
- Move the cards-grid markup out of `WorkingHoursMiniOverview` into a `cards` branch and add a `table` branch behind a `layout` prop driven by the toggle.
- Add a small UI-prefs service (sibling of `service/theme.rs`) that reads/writes the layout key in `localStorage`.

## Capabilities

### New Capabilities

- `working-hours-mini-overview`: The working-hours mini overview rendered below the shift plan. Covers card layout, table layout, layout toggle, and persistence of the layout choice.
- `ui-preferences`: Browser-local UI preferences stored in `localStorage`. Initial scope is the working-hours overview layout; designed to host further per-browser UI prefs over time.

### Modified Capabilities

<!-- None: the existing working-hours mini overview has no spec yet, so its current behavior is captured by the new spec instead of as a delta. -->

## Impact

- `src/component/working_hours_mini_overview.rs` — split rendering into `cards` and `table` branches; add `layout` prop.
- `src/page/shiftplan.rs` (call site around line 1085) — read layout from the new prefs service, render the toggle, pass `layout` into the component, persist on change.
- New `src/service/ui_prefs.rs` — `get_working_hours_layout` / `set_working_hours_layout` mirroring the `localStorage` pattern in `service/theme.rs`.
- i18n: new keys for the toggle labels ("Karten" / "Tabelle") and the table headers ("Employee", "Actual", "Target", "Difference", "Utilization", "Total"), with translations in `de`, `en`, `cs`.
- Tests: SSR tests for the new table layout and the toggle behavior; unit tests for the prefs service (parse / fallback / round-trip).
- No backend or API changes.
