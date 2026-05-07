## Context

The Shiftplan page (`src/page/shiftplan.rs:1085`) renders a `WorkingHoursMiniOverview` below the week view. Today this component (`src/component/working_hours_mini_overview.rs`) emits a single CSS-grid layout of cards (`repeat(auto-fit, minmax(180px, 1fr))`) — one card per employee with avatar, "actual / target h" line and a 3px progress bar.

The new design (`shifty-design/project/Shifty Preview.html`, `WorkingHoursOverview` component, lines 556–717) keeps the cards but adds:

1. A second layout — a borderless table with columns `Employee | Actual | Target | Difference | Utilization` and a totals footer row.
2. A segmented toggle ("Karten" / "Tabelle") rendered above the overview, with the active option styled as a raised pill on `var(--surface)`.
3. A `defaultLayout` prop sourced from `window.SHIFTY_TWEAKS?.workingHoursLayout` (the design's tweaks panel hook). In our app, that role is filled by `localStorage`.

The codebase already has a `localStorage` precedent: `src/service/theme.rs:84-93` reads/writes the theme key. The error and i18n services follow the same shape. There is no existing UI-prefs service to extend.

The component is consumed by exactly one call site (`page/shiftplan.rs:1085`), so prop changes are local.

## Goals / Non-Goals

**Goals:**

- Render the overview as either cards or table, controlled by a single `layout` prop on the component.
- Provide an explicit toggle in the UI directly above the overview, in the same column.
- Persist the toggle choice per browser via `localStorage`. Default to `cards` when no value or an unknown value is stored.
- Keep the cards branch visually identical to today's output (no regressions to the existing capability).
- Match the design's table columns, totals footer, and visual tokens (`var(--surface)`, `var(--surface-alt)`, `var(--border)`, `var(--warn)`, `var(--good)`, `var(--ink-muted)`).

**Non-Goals:**

- Server-side / per-user persistence. The choice is browser-local, like the theme.
- Sorting, filtering, or column hiding in the table. Render order matches the cards view (alphabetical by name).
- A general "tweaks panel" equivalent to the design's debug panel.
- Refactoring the existing card markup beyond what is needed to nest it into a layout branch.

## Decisions

### Decision 1: One component with an enum prop, not two components

The existing `WorkingHoursMiniOverview` gains a `layout: WorkingHoursLayout` prop (enum: `Cards`, `Table`). The component branches on the prop and emits the correct markup.

**Why:** the input data, sort order, accessor logic (`actual`, `target`, classes from `progress_bar_class` / `hours_text_class` / `progress_bar_percent`) and the per-row identity (selected accent, double-click handler) are identical between the two layouts. Splitting into two components would duplicate that logic and force the call site to branch twice.

**Alternative considered:** two sibling components (`WorkingHoursMiniOverviewCards`, `WorkingHoursMiniOverviewTable`) with a shared helpers module. Rejected — the helpers are already module-private and the rendering bodies share the per-row data prep; a single `match` on the prop is shorter and keeps tests in one file.

### Decision 2: Toggle lives at the call site, not inside the component

The toggle and the localStorage round-trip live in `page/shiftplan.rs`. The component receives a finished `layout` prop and an `on_layout_change: EventHandler<WorkingHoursLayout>` it does not need to know about persistence. The toggle is a small inline element rendered directly above the component.

**Why:** the component is also useful in contexts where a toggle is not appropriate (e.g. embedded in a print layout). Keeping the component pure makes it composable and keeps tests simple. The page already owns layout chrome above the overview.

**Alternative considered:** bake the toggle into the component and have it own the prefs-service call. Rejected — couples a presentational component to a side-effecting service.

### Decision 3: New `service/ui_prefs.rs` mirroring `service/theme.rs`

A new file `src/service/ui_prefs.rs` exposes:

```rust
pub fn get_working_hours_layout() -> WorkingHoursLayout;
pub fn set_working_hours_layout(layout: WorkingHoursLayout);
```

Storage key: `"shifty.ui.workingHoursLayout"`. Values: `"cards"` or `"table"`. Unknown / missing / non-WASM → `Cards` (the default).

The `WorkingHoursLayout` enum lives in this module (or in a shared `state` module if a second consumer arrives), and is re-exported for the component.

**Why:** mirrors the existing `theme.rs` pattern, so a future reader sees one consistent shape for all `localStorage`-backed UI prefs. Keeping a dedicated service (not inline `web_sys` calls in the page) makes the future "add another pref" change a one-line addition.

**Alternative considered:** put the helpers in `service/theme.rs` and rename it to `ui_prefs.rs`. Rejected — out of scope for this change; would invite a noisier diff.

### Decision 4: Toggle persists eagerly on click; no debounce

`set_working_hours_layout` is called from the toggle's `onclick` handler, before the state update. There is no debounce / batching.

**Why:** the call is one synchronous `localStorage.setItem`. Cheap. The simpler thing.

### Decision 5: Default is `Cards`

The current behavior is the cards layout, so existing users see no change on first load. Stored value `"table"` opts in.

### Decision 6: Table sort order matches cards

Both layouts sort employees alphabetically by `sales_person_name` (the existing behavior). The totals footer aggregates `actual` and `target` over the same rows.

**Why:** consistent mental model when a user toggles between the two views.

## Risks / Trade-offs

- **Risk:** localStorage unavailable (private mode, disabled, SSR test environment) → **Mitigation:** prefs service silently falls back to default and to a no-op write, exactly like `theme.rs`. SSR tests cover the no-op path.
- **Risk:** The table is wider than the available column on narrow screens → **Mitigation:** wrap the table in a horizontal-scroll container (the design uses `.overview-table-wrap` with `overflow-x: auto` and `min-width: 540px` on the inner table). Carry that pattern over with Tailwind utilities.
- **Risk:** The toggle is invisible/unclear in dark mode → **Mitigation:** use only design tokens (`var(--surface)`, `var(--surface-alt)`, `var(--ink)`, `var(--ink-muted)`) so dark mode is automatic. Keep the existing `no_hex_color_literals_in_source` test green.
- **Trade-off:** A stored `"table"` value will silently revert to cards if the enum is later renamed. Acceptable — UI prefs are non-critical and the user can re-toggle.
- **Trade-off:** Table view duplicates the same data the cards already show. That is intentional — it is a presentational alternative, not new information.

## Migration Plan

No migration needed. The component default (`Cards`) and the prefs service default (`Cards` on missing key) reproduce the current behavior for every existing user. New users opt in via the toggle.

## Open Questions

- Do we want to expose the same toggle on other pages where the mini overview might appear in the future? Not in scope here, but the component shape is ready for it.
