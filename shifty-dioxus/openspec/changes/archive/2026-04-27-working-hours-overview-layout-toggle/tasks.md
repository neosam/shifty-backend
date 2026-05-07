## 1. UI-preferences service

- [x] 1.1 Create `src/service/ui_prefs.rs` with a `WorkingHoursLayout` enum (`Cards`, `Table`) deriving `Clone`, `Copy`, `PartialEq`, `Debug`
- [x] 1.2 Add `pub const STORAGE_KEY: &str = "shifty.ui.workingHoursLayout";` (or equivalent module constant)
- [x] 1.3 Implement `get_working_hours_layout() -> WorkingHoursLayout` reading via `web_sys::window().and_then(|w| w.local_storage().ok().flatten())`, returning `Cards` on missing/unknown/unavailable
- [x] 1.4 Implement `set_working_hours_layout(layout: WorkingHoursLayout)` writing `"cards"` / `"table"`, no-op when storage unavailable, never panicking
- [x] 1.5 Re-export the module from `src/service/mod.rs`
- [x] 1.6 Unit tests for the string parsing helpers (parse `"cards"`/`"table"`/unknown) so the WASM-only `localStorage` paths are not the only thing covered

## 2. Component: layout prop and table layout

- [x] 2.1 In `src/component/working_hours_mini_overview.rs`, add a `layout: WorkingHoursLayout` prop (default `Cards`) to `WorkingHoursMiniOverviewProps`
- [x] 2.2 Extract the existing card-grid body into a `CardsLayout` inner function/component that takes the prepared rows and the existing event handlers
- [x] 2.3 Add a `TableLayout` inner function/component rendering a `<table>` with header (`Employee`, `Actual`, `Target`, `Difference`, `Utilization`), one body row per employee (avatar dot, name, mono actual / target, signed diff, progress bar + percent), and a `Total` footer row
- [x] 2.4 Wrap the table in an `overview-table-wrap`-equivalent: a `div` with `overflow-x: auto` so the table can scroll horizontally on narrow columns
- [x] 2.5 Drive table cell colors from the existing helpers (`progress_bar_class`, `hours_text_class`, `progress_bar_percent`); add a `signed_hours(actual, target)` helper for the difference column
- [x] 2.6 Branch on `props.layout` in the component body — render `CardsLayout` for `Cards`, `TableLayout` for `Table`
- [x] 2.7 Verify only design tokens are used (no hex literals); keep the existing `working_hours_mini_overview_no_legacy_classes_in_source` test green

## 3. Component: tests

- [x] 3.1 Update existing SSR tests to pass `layout: WorkingHoursLayout::Cards` (default behavior)
- [x] 3.2 SSR test: with `layout: Table` and three rows, the rendered HTML contains a `<table>`, `<thead>`, `<tbody>` with three rows, and a footer row with `Total`
- [x] 3.3 SSR test: a row with `actual=22.0, target=20.0` renders `+2.0h` in the difference cell with `text-good`
- [x] 3.4 SSR test: a row with `actual=15.0, target=20.0` renders `-5.0h` with `text-warn`
- [x] 3.5 SSR test: utilization cell renders the progress bar at `width: 50%` and the text `50%` for `actual=5, target=10`
- [x] 3.6 SSR test: footer aggregates totals over visible rows and matches the expected `actual` and `target` strings; difference in footer is signed
- [x] 3.7 SSR test: rows are alphabetical in both layouts (provide unsorted input, render once each, assert order)
- [x] 3.8 SSR test: `selected_sales_person_id` highlights the matching row in the table layout (assert accent class on the `<tr>`)
- [x] 3.9 SSR test: double-click handler is wired on the `<tr>` in table layout (assert presence of an `ondblclick` attribute or its class hook)

## 4. Page integration

- [x] 4.1 In `src/page/shiftplan.rs`, replace the static `WorkingHoursMiniOverview { ... }` call (around line 1085) with a `use_signal(|| ui_prefs::get_working_hours_layout())` plus the corresponding prop
- [x] 4.2 Render a segmented toggle directly above the overview: two buttons inside a pill container, labelled by new i18n keys for "Cards" and "Table"; active button uses `bg-surface` with shadow tokens, inactive uses `text-ink-muted`
- [x] 4.3 On click, update the signal and call `ui_prefs::set_working_hours_layout(...)` so the choice persists immediately
- [x] 4.4 Visual: keep the toggle in the same column as the overview, aligned with the existing heading row of the section
- [x] 4.5 Test (page-level if feasible, otherwise extract a thin sub-component): toggle reflects the current signal, clicking switches both the rendered layout and the persisted value (mock the prefs service)

## 5. i18n keys

- [x] 5.1 Add new `Key::WorkingHoursLayoutCards` and `Key::WorkingHoursLayoutTable` (toggle labels) to `src/i18n/mod.rs`
- [x] 5.2 Add new `Key::WorkingHoursTableEmployee`, `WorkingHoursTableActual`, `WorkingHoursTableTarget`, `WorkingHoursTableDifference`, `WorkingHoursTableUtilization`, `WorkingHoursTableTotal` to `src/i18n/mod.rs`
- [x] 5.3 Add German translations in `src/i18n/de.rs` (`Karten`, `Tabelle`, `Mitarbeiter`, `Ist`, `Soll`, `Differenz`, `Auslastung`, `Summe`)
- [x] 5.4 Add English translations in `src/i18n/en.rs` (`Cards`, `Table`, `Employee`, `Actual`, `Target`, `Difference`, `Utilization`, `Total`)
- [x] 5.5 Add Czech translations in `src/i18n/cs.rs`
- [x] 5.6 Replace any literal strings in the new component / toggle code with the i18n keys

## 6. Validation

- [x] 6.1 Run `cargo fmt`
- [x] 6.2 Run `cargo clippy` — address any warnings introduced
- [x] 6.3 Run `cargo test` — all new and existing tests pass
- [x] 6.4 Run `dx serve --hot-reload` (with Tailwind in watch mode) and visually verify on the Shiftplan page: toggle renders, switching repaints the overview, reload restores the chosen layout, dark mode renders correctly in both layouts
- [x] 6.5 Run `openspec validate working-hours-overview-layout-toggle`
