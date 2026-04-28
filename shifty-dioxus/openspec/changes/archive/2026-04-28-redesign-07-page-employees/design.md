## Context

`src/page/employees.rs` is the entry point of the Mitarbeiter (Employees) area. The current implementation:

- Wraps the page in `<div class="ml-1 mr-1 pt-4 md:m-8">` and stacks two unrelated sections under one heading: a flat list of employees (rendering `EmployeeShort` rows linked to detail) and a billing-period CRUD section (with two legacy `Modal` dialogs and inline `text-red-*`/`bg-blue-*` button styling).
- Loads employees via `loader::load_employees(...)` for the current year and renders them as a vertical list of `Link { Route::EmployeeDetails }` blocks. There is no search field, no list/detail split, no active-row highlight.
- `EmployeeShort` (`src/component/employee_short.rs`) renders only `<name>` and `<balance> {hours_str}` in legacy `text-gray-*` classes — no color dot, no hours-vs-target value, no styling for an active row.

`src/page/employee_details.rs` mounts a single `EmployeeView` component plus a legacy `Modal { EmployeeWorkDetailsForm { ... } }` for the contract dialog.

`src/component/employee_view.rs` is an 838-line component that renders the entire detail page as a single tall column:

- Header row with the employee's `name`, year nav as legacy two-button cluster, and a `DropdownTrigger` exposing four actions (`AddEntry`, `ShowFullYear`, `ShowUntilNow`, `AddWorkDetails`).
- A flex column listing **everything** in sequence: balance/required/overall and breakdown via `TupleView` (two-column dashed-border legacy layout), per-week working-hours expansion (`WorkingHoursView`, with its own per-week toggle), and the extra-hours list (`ExtraHoursView`).
- Modals are mounted from inside `EmployeeViewPlain` for `AddExtraHoursForm` and from `EmployeeDetails` for `EmployeeWorkDetailsForm`.
- All dialogs use the legacy `Modal` (centered, no header/footer, no token styling) and the legacy form components (`Button`, `DateInput`, `FloatInput`, `IntegerInput`, `Checkbox`, `FormPair`).

The reference design (`design_handoff_shifty/README.md` § 5) reframes this area as a two-column master/detail layout:

1. **Left list (280–360 px)**: searchable, scroll-confined; each row carries a color dot (`background_color` of the SalesPerson — no initials), the name, hours/target in mono, and an active-row tint (`bg-accent-soft` + 3 px `accent` left border). On `<720 px`, the list and detail are mutually exclusive (back button on detail).
2. **Right detail (flex-grow)**:
   - Header row: large color circle, name, type pill (Bezahlt/Freiwillig), target hours, year nav (`NavBtn`), `Sonstige Stunden` button (the existing extra-hours add flow), `Mehr ▾` (the existing actions dropdown).
   - Three-column sub-grid with `auto-fit, minmax(280px, 1fr)`:
     - **Gesamtansicht** — `TupleRow` stack: Stundenkonto (balance), Gesamt (overall), Soll (required), then a dim breakdown (shiftplan, extra-work, vacation, sick-leave, holidays, unpaid-leave, volunteer, custom).
     - **Arbeitsverträge + Stunden pro Woche** — clickable contract cards, then a 17-bar SVG histogram (previous 17 ISO weeks ending on the current week) where each bar is the week's `overall_hours` and a dashed line marks `expected_hours`. Bars below the line render in `var(--warn)`. Clicking a bar toggles an inline week-detail panel below the histogram showing per-day hours-by-category — same data shape (`WorkingHoursDay { date, hours, category }`) as today's `WorkingHoursView`. **Block-based time view is explicitly out of scope** (see master plan).
     - **Zusatzarbeit** — the existing extra-hours list (vacation / holidays / sick-leave / extra-work / unavailable / unpaid-leave / volunteer / custom categories), restyled with tokens but using the same data and per-row delete affordance.
3. **Modals** rebuilt on the new `Dialog` + form atoms (`Field`, `FormTextInput`, `FormSelectInput`, `FormTextareaInput`):
   - `ContractModal` — wraps the existing `EmployeeWorkDetailsForm` semantics in a tokenized layout for new contracts and editing existing ones.
   - `ExtraHoursModal` — wraps the existing `AddExtraHoursForm` semantics in a tokenized layout, opened by the `Sonstige Stunden` button.

This page is the most layout-heavy of the redesign series. It is also the largest single source file in `src/component/`, so the migration is unavoidably broad. The approach below partitions the work into small, independently-reviewable units.

### Data shape constraints

`Employee` (in `state::employee`) carries: `sales_person`, `working_hours_by_week: Rc<[WorkingHours]>`, totals (`overall_working_hours`, `expected_working_hours`, `balance`, `carryover_balance`), category breakdown (`shiftplan_hours`, `extra_work_hours`, `vacation_hours`, `sick_leave_hours`, `holiday_hours`, `unpaid_leave_hours`, `volunteer_hours`), vacation tracking (`vacation_days`, `vacation_entitlement`, `vacation_carryover`), and `custom_extra_hours`. Sufficient for everything the reference design renders — no new fields are required.

`WorkingHours` (per-week entries inside `working_hours_by_week`) carries `from`, `to`, `expected_hours`, `overall_hours`, `balance`, the same category breakdown, and `days: Rc<[WorkingHoursDay]>`. The histogram reads `overall_hours` and `expected_hours` per week; the inline week-detail panel reads `days`.

`SalesPerson.background_color: Rc<str>` carries the pastel hex (e.g. `#dbe0ff`) used everywhere a person is rendered. The reference design's color dot and color circle reuse this value verbatim.

`SalesPerson.is_paid: bool` drives the `Bezahlt`/`Freiwillig` type pill.

`EMPLOYEE_STORE` and `EMPLOYEE_WORK_DETAILS_STORE` already manage the active employee, year, until-week, and contracts list. The new list page reads `loader::load_employees(...)` (existing) for the list rows and the existing store coroutines (`EmployeeAction::LoadEmployeeDataUntilNow`, `NextYear`, `PrevYear`, etc.) drive the detail. No store changes are required.

The current page also hosts billing-period CRUD. **That section is moved out of the redesigned employees page** (see decision 1).

### Existing atoms and components

- `Btn` / `BtnVariant` (Primary, Secondary, Ghost, Danger) — used for the "Sonstige Stunden" / "Mehr" / contract-card / save / cancel buttons.
- `NavBtn` — used for prev/next year navigation in the detail header.
- `PersonChip` — used for the type pill (Bezahlt/Freiwillig) since the chip has the dark-ink-on-pastel invariant baked in. Pill background uses `--accent-soft` for paid, `--warn-soft` for volunteer.
- `TupleRow` — used for every value in the Gesamtansicht column. Replaces the legacy `TupleView`/`TripleView` from `employee_view.rs`.
- `Dialog` / `DialogVariant` — replaces legacy `Modal` for the two new modals. `Auto` variant: bottom-sheet on mobile, centered card on desktop.
- `Field` + `FormTextInput` / `FormSelectInput` / `FormTextareaInput` — replace the legacy form components inside the new modals.

### Reference HTML

The Mitarbeiter screen is implemented in plain HTML inside `design_handoff_shifty/Shifty Preview.html`. The histogram and inline week-detail are SVG within an HTML wrapper. Use this file as the visual ground truth when wiring tokens, spacing, and SVG geometry.

## Goals / Non-Goals

**Goals:**
- Rebuild `src/page/employees.rs` as a two-column master/detail page when the route is `Employees`.
- Add a search field above the list (filters by `sales_person.name`, case-insensitive substring).
- Render each list row as `[color-dot] name | hours/target` (mono); active row carries `bg-accent-soft` and a 3 px `accent` left border.
- On `<720 px`, render either the list or the detail (mutually exclusive). The detail shows a back button that returns to the list.
- Move the billing-period CRUD section out of `src/page/employees.rs` to a new `src/page/billing_periods.rs` page (route `BillingPeriods`, accessible from a top-bar entry or a button on the new employees page — see decision 1).
- Rebuild the detail header: large color circle (32 px), name (text-xl), type pill (`PersonChip` with `accent-soft` for paid / `warn-soft` for volunteer), target hours (mono), year nav (`NavBtn` ‹ year ›), `Sonstige Stunden` button (`Btn` Primary), and `Mehr ▾` dropdown (existing `DropdownTrigger`, restyled trigger).
- Replace the existing single-column detail body with a 3-column sub-grid (`grid-template-columns: repeat(auto-fit, minmax(280px, 1fr))`):
  - **Gesamtansicht** — primary `TupleRow` for balance / overall / required, then dim `TupleRow` rows for the category breakdown (and custom hours).
  - **Arbeitsverträge** + **Stunden pro Woche** in the same column: contract cards (clickable, opens edit; "+ Add" button at the bottom opens new contract), then the 17-bar histogram and inline week-detail panel.
  - **Zusatzarbeit** — restyled extra-hours list with token classes and `Btn` Danger (icon-only) for delete.
- Build a new `src/component/employee_weekly_histogram.rs` component that renders 17 SVG bars for the last 17 ISO weeks ending on the currently-displayed week (the year-nav-driven "until_week" already in the store). Each bar shows `overall_hours`; a dashed line shows the most recent contract's `expected_hours_per_week`; bars below the line render in `var(--warn)`, the rest in `var(--accent)` (current-week bar at full opacity, others at `0.85`). Clicking a bar selects the week and triggers the inline week-detail panel below.
- Build the inline week-detail panel as a sub-component or inline RSX inside the detail page: shows the selected week's `from`–`to` range, `balance`/`overall`/`expected` line, and a list of `WorkingHoursDay { date, hours, category }` rows (one row per day). Hours per day, **not** time blocks.
- Build `ContractModal` (new sub-component in `src/component/contract_modal.rs`) using `Dialog` (variant `Auto`, width 520) and the form atoms. Wraps the existing `EmployeeWorkDetailsForm` semantics — the modal is the new shell; the field set is unchanged.
- Build `ExtraHoursModal` (new sub-component in `src/component/extra_hours_modal.rs`) using `Dialog` (variant `Auto`, width 460) and the form atoms. Wraps the existing `AddExtraHoursForm` semantics.
- Migrate the existing `EmployeeShort` to be the new list-row atom. Either rewrite it in place (if no other consumer) or replace its body and add new props (`hours`, `target`, `active`, `color`).
- All new and rewritten code SHALL use design tokens. No `bg-gray-*`, `bg-white`, `text-gray-*`, `text-blue-*`, `text-red-*`, `text-green-*`, `bg-blue-*`, `bg-green-*`, `border-gray-*`, `border-black` shall remain in `src/page/employees.rs`, `src/page/employee_details.rs`, `src/component/employee_view.rs`, `src/component/employee_short.rs`, or in any of the new files.
- Add four new i18n keys: `Key::SearchPlaceholder` (search input placeholder), `Key::OtherHours` (Sonstige Stunden button), `Key::More` (Mehr ▾ button), `Key::BackToList` (mobile back button). Reuse existing keys for everything else (Balance, Overall, Required, CategoryShiftplan, …, `Paid`, `Volunteer`, year-nav already covered by `PreviousYear`/`NextYear` from change 06).

**Non-Goals:**
- No backend changes. `Employee`, `WorkingHours`, `WorkingHoursDay`, and `SalesPerson` shapes stay as-is. No new endpoint.
- No block-based time view in the week detail (explicitly out — see master plan and proposal). The week detail keeps the existing per-day hours-by-category shape.
- No restyling of the still-mounted `MyEmployeeDetails` page (`src/page/my_employee_details.rs`) in this change. That page reuses `EmployeeView` today; the rewrite gives `EmployeeViewPlain` a tokenized look, which `MyEmployeeDetails` inherits, but no new layout work happens there.
- No removal of the legacy `Modal`, `Button`, `DateInput`, `FloatInput`, `IntegerInput`, `Checkbox`, `Form*` components from `src/component/base_components.rs` — the cleanup change (`99`) handles deletion once every page is migrated. New code SHALL NOT reach into `base_components.rs`.
- No removal of `DropdownTrigger`. The Mehr ▾ dropdown reuses it; a tokenized restyle is out of scope here and lives in `99`.
- No filtering of inactive employees outside of the existing `e.sales_person.inactive` filter (the list still hides inactive employees by default).
- No persistence of the active employee selection across navigation. Selecting an employee navigates to `/employees/<id>` (existing route) and the active id is read from the URL parameter.
- No drag/drop reorder of contracts.
- No edit-in-place of histogram bars (e.g. clicking can't change hours).

## Decisions

### 1. Move billing-period CRUD out of the employees page

The current `src/page/employees.rs` mounts two distinct sections under one route: the employee list and the billing-period CRUD. The reference design has no billing-period content on this screen — billing periods belong to a separate flow. Keeping both on one page also fights the new master/detail layout (the list-and-detail span the viewport; there's no obvious place for an unrelated CRUD section).

**Decision**: split the page. After this change:
- `/employees` (route `Employees`) — the new master/detail Mitarbeiter page only. List on left, detail on right (or full-width on mobile).
- `/billing-periods` (new route `BillingPeriods`) — the billing-period CRUD, lifted as-is from the existing employees page, with its two legacy `Modal` dialogs **migrated to `Dialog`** (small wins, but the dialogs are short and using the new component is consistent with this change's tokenization goals).
- Add an "Abrechnungszeiträume" entry in the top-bar nav (or, if that overflows, behind the existing `Mehr` ▾ in `TopBar`). Matches the spec for the redesigned `TopBar` in change `03` — see its tokens for nav-button styling.
- `Route::BillingPeriodDetails { ... }` (already exists) keeps working unchanged.

This deviates from the proposal's "rewrite `src/page/employees.rs`" framing — the proposal does not call out the billing-period split. Recorded here so the reviewer doesn't expect billing-period CRUD inside the new employees page. The split keeps each page focused, halves the file size, and lets the new page be tested without billing-period mocks.

### 2. Capabilities

The proposal lists `employees-page`, `employee-details-page`, and `employee-weekly-histogram`. These three remain. Add one more for the modals because the form atoms and the dialog wrapper compose into testable units of their own:

- MODIFY `employees-page`: list-with-search + master/detail layout.
- MODIFY `employee-details-page`: header + 3-column sub-grid + week-detail panel.
- ADD `employee-weekly-histogram`: 17-bar SVG with click-to-select + dashed expected-line.
- ADD `employee-modals`: `ContractModal` + `ExtraHoursModal` wrappers around `Dialog` and the form atoms.
- ADD `billing-periods-page`: lifted from current employees page; `Dialog`-migrated.

The four added/modified specs each cover ≤3 requirements; total spec churn is moderate.

### 3. Master/detail at the route level, not inside one giant component

Two layouts considered:

| Option | UX |
|---|---|
| One `Employees` page that hosts both list and detail in the same render tree, switching the right pane on URL change | Tight integration; the active row tint comes from the same component that knows the URL param |
| List page renders list; detail page is a separate route — both share a shell that renders list+detail side-by-side on desktop, single-column on mobile | Cleaner separation; the detail can be reached directly via `/employees/<id>` and shows the back button on mobile |

**Chosen: option 2.** The router already exposes `Route::Employees` and `Route::EmployeeDetails { employee_id }` — keep the routing model. Wrap both pages in a shared shell `EmployeesShell` that:

1. On `≥720 px`: renders `<EmployeeList active={current_id} />` on the left and `{children}` (the routed page) on the right.
2. On `<720 px`: renders the list when on `/employees` (no active id) and the detail (with a back button) when on `/employees/<id>`.

The shell is a small composition layer with no fetch logic of its own. Implementation detail: do this without router middleware — just have both `Employees` and `EmployeeDetails` page components compose `<EmployeesShell>` and pass their content as children. Keeps the change shallow.

The mobile detection reuses the `use_media_query("(max-width: 720px)")` hook already implemented in `src/component/dialog.rs` — promote it to `src/component/atoms/` so the shell can use it without a circular dep, **or** keep it module-private and inline a small copy in the shell (the function is ~6 lines).

**Sub-decision**: promote `use_media_query` to `src/component/atoms/media_query.rs` and re-export from `atoms::mod.rs`. Update the `Dialog` import path. Cheap promotion, no behavior change. The hook is generally useful and the dialog file is already too big.

### 4. List row anatomy and active state

The current `EmployeeShort` is one tiny component. Replacement layout:

```
<a class="flex items-center gap-3 px-3 py-2 border-l-[3px] border-transparent hover:bg-surface-alt"
   [active]="bg-accent-soft border-accent">
  <span class="w-2.5 h-2.5 rounded-full" style="background:{color}"/>
  <span class="flex-1 text-ink truncate">{name}</span>
  <span class="font-mono tabular-nums text-xs text-ink-muted">
    {balance:.1}/{expected:.0}
  </span>
</a>
```

- Color dot: 10 px circle, `background-color` from `sales_person.background_color`. **No initials, no text inside.**
- Name: truncate with `truncate` class so long names don't wrap.
- Hours: `balance/expected` in mono tabular-nums. `expected` reads from the most recent `WorkingHours` entry; when none exist yet, render `0` for the target so the format stays stable.
- Active row: `bg-accent-soft` plus the left border switches from `border-transparent` to `border-accent`. The 3 px width is fixed — the whole row uses `border-l-[3px]` which reserves the space even on inactive rows so layout doesn't jump.
- Wrapper: `<Link to={EmployeeDetails {...}}>` so click navigates and the URL drives the active state.

**Decision**: rewrite `EmployeeShort` in place rather than create a new component. The current `EmployeeShort` is tiny (29 lines) and has only one consumer (the employees list). New name stays `EmployeeShort` to minimize ripple. New props: `active: bool`, `target_hours: f32`. No other consumers reference the old `EmployeeShort`.

### 5. Detail header

Three rows are involved:

```
[color circle 32px] [name text-xl font-semibold] [type pill] [target mono]
                                                              [‹ year ›] [Sonstige Stunden] [Mehr ▾]
```

On mobile the second row wraps below the first.

- Color circle: 32 px, `background-color: {color}`, `border-radius: 50%`. Larger than the list-row dot. Like the list dot, no text inside.
- Name: `text-xl font-semibold text-ink` — bigger than the list's name to anchor the right pane.
- Type pill: a `PersonChip` with `color: Some(--accent-soft hex)` for paid (renders `Bezahlt`) and `color: Some(--warn-soft hex)` for volunteer (renders `Freiwillig`). The `.person-pill` rule keeps the text dark in both themes — the pastel tokens are the soft variants, which already meet contrast against `var(--chip-ink)` (`#0e1117`).
  - Implementation detail: `PersonChip` takes its color as `Option<ImStr>` of the hex, so resolve `--accent-soft` and `--warn-soft` to their literal hex values (`#eaecfb` and `#fef0d6` from the tokens table) at the call site. This is a small forward-compat trade-off — when the tokens move (theme rework), the chip backgrounds drift. Acceptable given how pinned these values are.
- Target hours: `font-mono tabular-nums text-ink-muted` rendering `{expected:.0} h` (or use `Hours` translation).
- Year nav: `<NavBtn glyph=‹ aria_label=Key::PreviousYear>`, `<span class="font-mono">{year}</span>`, `<NavBtn glyph=› aria_label=Key::NextYear>`. Same pattern as the redesigned `WeeklyOverview` page from change 06. `print:hidden` on the wrapper.
- Sonstige Stunden: `<Btn variant=Primary on_click={open_extra_hours_modal}>Key::OtherHours</Btn>`.
- Mehr ▾: keep `DropdownTrigger` from the existing code, but render its trigger as `<Btn variant=Secondary>{i18n.t(Key::More)} ▾</Btn>`. Entries are the same as today (`AddEntry` remap → no longer needed because we have `Sonstige Stunden`; `ShowFullYear` / `ShowUntilNow` / `AddWorkDetails`).
  - **Sub-decision**: drop `AddEntry` from the dropdown — it duplicates the new "Sonstige Stunden" button. The dropdown keeps only the year-toggle entries and the `AddWorkDetails` entry. If "AddWorkDetails" itself becomes redundant (it could move to a "+ Add" button at the bottom of the contracts column — see decision 6), drop it too and the dropdown collapses to just full-year/until-now. Watch for empty dropdowns; if everything else is removed, remove the `Mehr ▾` button entirely.
  - Final scope: keep `Mehr ▾` for full-year and until-now toggles only. AddWorkDetails moves to the contracts column.

### 6. Three-column sub-grid

CSS Grid with `repeat(auto-fit, minmax(280px, 1fr))`. Stacks on narrow viewports (one column when `<560 px`, two when `560–840 px`, three when `≥840 px`).

```
section { class: "grid gap-6 mt-6", style: "grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));",
    div { /* Gesamtansicht */ }
    div { /* Arbeitsverträge + Stunden pro Woche */ }
    div { /* Zusatzarbeit */ }
}
```

#### 6a. Gesamtansicht column

Three primary `TupleRow`s (no `dim`):

- Balance (Stundenkonto)
- Overall (Gesamt)
- Required (Soll)

Then a divider (`<div class="border-t border-border my-3"/>`) and a stack of `dim={true}` `TupleRow`s for the breakdown:

- Shiftplan
- Extra work
- Vacation hours / vacation days (`x / y`) / vacation carryover (only when `props.show_vacation`)
- Sick leave
- Holidays
- Unpaid leave
- Volunteer
- Carryover balance
- One row per `custom_extra_hours` entry

The values on each row use mono tabular-nums via `<span class="font-mono tabular-nums">{value}</span>` rendered as the row's `value` element.

**Decision**: keep this composition inline in the detail page (or in a small `EmployeeOverview` sub-component). Don't try to make it data-driven from a config — the rows are stable, and a small pattern repeated 10 times reads better than a dynamic builder.

#### 6b. Arbeitsverträge + Stunden pro Woche column

Stacked vertically:

1. **Contract cards** — one card per `EmployeeWorkDetails`, ordered by `from` ascending. Card layout:
   ```
   <button class="w-full text-left rounded-md border border-border bg-surface px-3 py-2 hover:bg-surface-alt"
           onclick={open_edit_contract}>
     <div class="flex items-baseline justify-between gap-2">
       <div class="text-sm font-semibold text-ink">{from} – {to}</div>
       <div class="font-mono tabular-nums text-xs text-ink-muted">{expected_hours} h/Woche</div>
     </div>
   </button>
   ```
   Below the cards, an "+ Vertrag" button (`<Btn variant=Secondary icon=+>` with `i18n.t(Key::AddWorkDetailsLabel)`).
2. **Histogram** + **inline week-detail** — the new `EmployeeWeeklyHistogram` component (decision 7). When a bar is selected, the inline panel renders below with the day rows.

The "+ Vertrag" button is the home for adding a new contract; the `Mehr ▾` dropdown drops the `AddWorkDetails` entry per decision 5. Existing contract cards open the edit modal on click.

#### 6c. Zusatzarbeit column

The existing categorized list (vacation, holidays, sick leave, extra work, unavailable, unpaid leave, volunteer, custom). Restyled:

- Each category heading: `<h3 class="text-xs uppercase tracking-wide font-semibold text-ink-muted mt-4 first:mt-0">{label}</h3>`.
- Each entry row replaces the legacy `TripleView` with a small inline RSX:
  ```
  <div class="flex items-baseline justify-between gap-2 py-1.5 border-b border-border">
    <div class="min-w-0 flex flex-col">
      <span class="text-sm text-ink">{date}</span>
      <span class="text-xs text-ink-muted truncate">{description}</span>
    </div>
    <div class="flex items-center gap-2">
      <span class="font-mono tabular-nums text-sm text-ink">{value}</span>
      <Btn variant=Danger icon=🗑 on_click={delete} />
    </div>
  </div>
  ```
- Empty categories render no heading — drop legacy heading-without-rows behavior.

This column carries the heaviest content visually (potentially dozens of rows). Cap nothing in this change; if the list grows long, the column scrolls naturally inside the page.

### 7. EmployeeWeeklyHistogram component

`src/component/employee_weekly_histogram.rs`, a fresh component with no migration story.

#### Signature

```rust
#[derive(Props, Clone, PartialEq)]
pub struct EmployeeWeeklyHistogramProps {
    pub weeks: Rc<[WorkingHours]>,         // last 17 entries; render in the order given
    pub expected_per_week: f32,            // dashed reference line; 0.0 hides the line
    pub current_year: u32,                 // for is_current detection
    pub current_week: u8,
    pub selected_week: Option<(u32, u8)>,  // year/week pair, drives selected highlight
    pub on_select: EventHandler<(u32, u8)>,
}
```

The component is a pure SVG renderer — it does not fetch data, does not own selection state, does not import store stuff. The page passes `weeks = employee.working_hours_by_week.iter().rev().take(17).rev()` (last 17 weeks ending on the most recent loaded week). Page also tracks `selected_week` state and forwards `on_select`.

#### SVG geometry

- Viewport: `viewBox="0 0 340 120"`, `preserveAspectRatio="none"` so the bars stretch with the container.
- Bar width: `(340 - 16) / 17 = 19.05` (1-px gap).
- Bar height: `(week.overall_hours / max_y) * 90`, where `max_y = max(expected_per_week, weeks.iter().map(|w| w.overall_hours).fold(0.0, f32::max), 1.0)` (avoid divide-by-zero on empty data).
- Bottom 20 px reserved for the X-axis row (week label `KW <n>`, every 4th week + always the current week, mono `9px text-ink-muted`).
- Dashed expected-line: `<line x1=0 x2=340 y1={(1.0 - expected_per_week / max_y) * 90} stroke-dasharray="4 3" stroke="var(--ink-muted)" />` (geometry mirroring the redesigned weekly-overview chart).
- Bar fill: `var(--accent)` for `overall_hours >= expected_per_week`, `var(--warn)` otherwise. Selected bar: opacity 1; non-selected (when something is selected): opacity 0.85; nothing-selected: all opacity 1.
- Bars are rendered inside a `<g>` per week with `cursor: pointer` and an `onclick` that invokes `on_select`. The whole bar+label group is the click target (not just the rect — easier to hit on mobile).

**Sub-decision**: SVG geometry uses plain attributes; colors use the `style:` attribute to resolve `var(--token)`. Mirrors the geometry/style split established in change 06 for `weekly_overview_chart.rs`.

#### Inline week-detail panel

Lives in the detail page, not in the histogram. The histogram only emits selection events. On select:

```
section.mt-3 {
  div { class: "flex items-baseline justify-between",
      h4 { "{from_date} – {to_date}" }
      span { class: "font-mono tabular-nums", "{overall:.2} / {expected:.2} h" }
  }
  ul {
      for day in week.days {
          li.flex.justify-between {
              span.font-mono { "{date_label} ({weekday})" }
              span { "{i18n.t(category.to_i18n_key())}" }
              span.font-mono.tabular-nums { "{day.hours:.2} h" }
          }
      }
  }
  div { /* close affordance — clicking another bar replaces; an "x" button at top-right closes via on_select(None) */ }
}
```

The `on_select` handler in the page sets `selected_week` to `Some((year, week))` on first click, and to `None` on click of the same bar (toggle). The page's "x" button on the panel also sets `None`.

### 8. ContractModal

`src/component/contract_modal.rs` — the new shell over the existing `EmployeeWorkDetailsForm` field set.

```rust
#[derive(Props, Clone, PartialEq)]
pub struct ContractModalProps {
    pub open: bool,
    pub form_type: EmployeeWorkDetailsFormType,
    pub on_save: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
}

#[component]
pub fn ContractModal(props: ContractModalProps) -> Element {
    rsx! {
        Dialog {
            open: props.open,
            on_close: move |_| props.on_cancel.call(()),
            title: ImStr::from(i18n.t(Key::AddWorkDetailsFormTitle).as_ref()),
            variant: DialogVariant::Auto,
            width: 520,
            footer: Some(rsx! {
                Btn { variant: BtnVariant::Secondary, on_click: ..., "{cancel_str}" }
                if props.form_type != ReadOnly {
                    Btn { variant: BtnVariant::Primary, on_click: ..., "{save_str}" }
                }
            }),
            // Body: the form fields (ported to use Field + FormTextInput / FormSelectInput / Checkbox-equivalent)
            ContractModalBody { form_type: props.form_type, ... }
        }
    }
}
```

`ContractModalBody` is a sub-component that contains the actual field set. It reads from `EMPLOYEE_WORK_DETAILS_STORE` (same as today) and dispatches `EmployeeWorkDetailsAction::*` (same as today). The field layout uses `Field` wrappers for each labeled input plus a 7-checkbox grid for the weekday selectors.

**Sub-decision: weekday checkbox replacement.** The form atoms in change 04 do not include a checkbox. Two options:

1. Add a `FormCheckbox` atom now as part of this change's scope.
2. Reuse the legacy `Checkbox` from `base_components.rs` and accept that one form atom remains unmigrated until the cleanup change.

**Choose option 1** (add `FormCheckbox`). Reasons: (a) the cleanup change is supposed to delete `base_components.rs`, and a half-migrated modal blocks that; (b) the checkbox atom is small and reusable across user-management, billing-period, etc.; (c) the alternative leaves `EmployeeWorkDetailsForm` stuck in the legacy form world, which contradicts the goal of fully tokenizing this page.

`FormCheckbox` lives in `src/component/form/checkbox.rs`, exported from `src/component/form/mod.rs`. The atom carries the same `value: bool`, `disabled: bool`, `on_change: Option<EventHandler<bool>>` shape as today's legacy `Checkbox` plus a `label: Element` slot (so the trailing label text can carry tokens).

This decision is recorded as a deviation from the proposal — the proposal says "Implement `ContractModal` and `ExtraHoursModal` using `Modal` and form atoms from `04`". `04` shipped Field + 3 input atoms but no checkbox. Adding `FormCheckbox` here is consistent with `04`'s pattern (`Form*` prefix to coexist with legacy until cleanup) and keeps this change self-contained.

### 9. ExtraHoursModal

`src/component/extra_hours_modal.rs` — the new shell over the existing `AddExtraHoursForm` semantics.

```rust
#[derive(Props, Clone, PartialEq)]
pub struct ExtraHoursModalProps {
    pub open: bool,
    pub sales_person_id: Uuid,
    pub on_saved: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
}
```

Body uses `Field` + `FormSelectInput` (category), `FormTextInput` (description, when, amount), `FormTextareaInput` (description if it grows beyond 60 chars — see sub-decision). Vacation-days mode renders two `FormTextInput type=date` fields ("From", "To") plus the description. Non-vacation modes render the amount, the description, and the `when` (`type=datetime-local`).

**Sub-decision**: the description input stays as a single-line `FormTextInput`. The legacy form already used a single line. Switching to a textarea would be a UX choice, not a tokenization requirement, and is out of scope.

The footer carries Cancel (`Btn` Secondary) and Submit (`Btn` Primary). Submit dispatches `api::add_extra_hour(...)` or `api::add_vacation(...)` directly (mirrors today's coroutine). On success, the modal calls `on_saved` and the page's coroutine refreshes the employee. The existing `AddExtraHoursForm` component stays in the codebase for now (still used by `MyEmployeeDetails`); a future cleanup change deletes both legacy forms.

### 10. Search field

A simple `<input>` above the list:

```
input { class: SHARED_INPUT_CLASSES, placeholder: i18n.t(Key::SearchPlaceholder),
        oninput: move |evt| search.set(evt.value()) }
```

Reuse the form atoms' `SHARED_INPUT_CLASSES` constant or just write the equivalent classes inline (the form atoms are not exported with public access to that constant — fine to duplicate the small string here). Filter logic: lowercase the search term and lowercase the name, return rows where the name contains the term. Empty search → all rows.

**Decision**: keep the search local to the list (a `use_signal` in the list component). No router-level state, no URL param, no debounce. Naive on-input filter is fine for the row count expected (typically <50 employees). If perf becomes an issue we add debounce later.

### 11. Active row + URL synchronization

The list reads the active employee id from the route. When the route is `Employees` (no id), no row is active. When the route is `EmployeeDetails { employee_id }`, the matching row is active.

Implementation: each list row uses `<Link to=Route::EmployeeDetails { employee_id: id.to_string() }>`. The active state comes from comparing `id == current_route_id`. The current route id is read via `use_route::<Route>()` (Dioxus router hook) inside the list component.

When the user types in the search box and the active row gets filtered out, no row appears active — that's fine and matches expected behavior.

### 12. Mobile back button on detail

When `<720 px` and the route is `EmployeeDetails`, the detail's first row (above the header) renders:

```
<Btn variant=Ghost icon=‹ on_click={navigate_to_/employees}>
  Key::BackToList
</Btn>
```

`Btn` already supports an icon prop. The handler navigates to `Route::Employees`.

Detection mirrors the `Dialog` use of `use_media_query("(max-width: 720px)")`. Hook gets promoted per decision 3.

### 13. i18n: minimal new keys

Four new keys cover the redesign:

- `Key::SearchPlaceholder` — list search input placeholder (`"Search…"` / `"Suchen…"` / `"Hledat…"`).
- `Key::OtherHours` — `Sonstige Stunden` button text (`"Other hours"` / `"Sonstige Stunden"` / `"Ostatní hodiny"`).
- `Key::More` — `Mehr ▾` button text (`"More"` / `"Mehr"` / `"Více"`).
- `Key::BackToList` — mobile back button text (`"Back"` / `"Zurück"` / `"Zpět"`).

Existing keys reused: `Employees`, `Balance`, `Overall`, `Required`, `Hours`, `HoursShort`, `Days`, `CarryoverBalance`, `Category*`, `VacationDaysLabel`, `VacationCarryoverLabel`, `Paid`, `Volunteer`, `WorkDetailsHeading`, `AddWorkDetailsLabel`, `WorkingHoursPerWeekHeading`, `ExtraHoursHeading`, `OverallHeading`, `ShowFullYearLabel`, `ShowUntilNowLabel`, `ActionsLabel`, `PreviousYear`, `NextYear`, `WeekShort`, `Cancel`, `Submit`, `Save`, `ShowDetails`, `HideDetails`, etc.

### 14. Test strategy

Two tiers, mirroring change 06:

**Unit tests (pure functions):**
- `histogram_max_y_uses_max_of_data_or_expected` — covers the divide-by-zero guard.
- `histogram_y_pos_for_value` — height proportional to `(value / max_y) * 90`.
- `histogram_bar_color_classifies_warn_below_expected` — `warn` when below, `accent` otherwise.
- `list_filter_case_insensitive_substring` — filter helper exercising upper/lower mixed.

**SSR tests (rendered HTML assertions):**
- List row: rendered HTML contains color dot inline style, name, `mono tabular-nums` hours, no initials.
- List active row: when `active=true`, HTML contains `bg-accent-soft border-accent`.
- List search: render the list with several names and a search term; assert only matching rows render.
- Detail header: contains the color circle inline style, the type pill (`Bezahlt` for paid, `Freiwillig` for volunteer), the year span, two `NavBtn`s, the `Sonstige Stunden` button, the `Mehr ▾` button.
- Gesamtansicht: contains TupleRow elements for Balance / Overall / Required and dim rows for the breakdown.
- Histogram SSR: 17 `<rect>` elements; dashed line element; X-axis labels at the right cadence.
- Histogram color: with `expected=20`, weeks `[10, 25]`, the first bar's style contains `var(--warn)` and the second `var(--accent)`.
- Histogram select: rendering with `selected_week=Some((2026, 17))` includes a dimmed style on non-selected bars.
- Inline week-detail: when `selected_week` is set, a panel renders below with the week's `from`–`to` heading and one row per `WorkingHoursDay`.
- ContractModal closed: `open=false` renders nothing.
- ContractModal open: `open=true` renders a `Dialog` shell containing the form fields and a footer with Cancel/Save buttons.
- ExtraHoursModal: same shape, with the category select including all standard categories.
- Token sweep: the non-test source of `src/page/employees.rs`, `src/page/employee_details.rs`, `src/component/employee_view.rs`, `src/component/employee_short.rs`, `src/component/employee_weekly_histogram.rs`, `src/component/contract_modal.rs`, `src/component/extra_hours_modal.rs` SHALL NOT contain `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `border-gray-`, `border-black`.
- Billing-periods page tests (lifted/migrated): one SSR test for the create dialog using `Dialog` and one for the delete-confirmation dialog using `Dialog`.

Tests live next to their source under `#[cfg(test)] mod tests`.

### 15. EmployeeView migration approach

`EmployeeView` is currently used by both `EmployeeDetails` (HR view) and `MyEmployeeDetails` (self-view). Approach:

- Keep `EmployeeView` as the public component (preserve the call site in `MyEmployeeDetails`).
- Replace `EmployeeViewPlain`'s body wholesale: the new layout (header + 3-column sub-grid + week detail) replaces the legacy column.
- `MyEmployeeDetails` automatically inherits the new look. Visually the page still uses the same data but with the new tokens — acceptable since `MyEmployeeDetails` is a subset of the HR view (same shape, fewer permissions).
- Drop `TupleView`, `TripleView`, `WorkingHoursView` from `employee_view.rs`. They are unused after the rewrite. Drop only the names that no other consumer imports — verify via `grep -r "TupleView\|TripleView\|WorkingHoursView" src/` before deleting.
- `ExtraHoursView` stays as a sub-component but its body switches to the new row layout (decision 6c). Other pages don't import `ExtraHoursView` directly — verify likewise.

The refactor lands `employee_view.rs` near 400 lines (down from 838).

### 16. File-level plan

New files:
- `src/component/atoms/media_query.rs` (promoted from `dialog.rs`).
- `src/component/employee_weekly_histogram.rs`.
- `src/component/contract_modal.rs`.
- `src/component/extra_hours_modal.rs`.
- `src/component/form/checkbox.rs` (new `FormCheckbox` atom).
- `src/page/billing_periods.rs` (lifted from `employees.rs`).

Modified files:
- `src/page/employees.rs` — strip billing-period content; mount the new master/detail shell + list.
- `src/page/employee_details.rs` — mount the new shell + the new detail layout (header, 3-column grid, week-detail panel).
- `src/component/employee_view.rs` — rewrite `EmployeeViewPlain` body; drop dead types; restyle `ExtraHoursView`.
- `src/component/employee_short.rs` — rewrite for color-dot row + active state.
- `src/component/dialog.rs` — change `use_media_query` import path.
- `src/component/mod.rs` — re-export the new components.
- `src/component/atoms/mod.rs` — add `media_query` export.
- `src/component/form/mod.rs` — add `checkbox` export.
- `src/router.rs` — add `Route::BillingPeriods`.
- `src/page/mod.rs` — re-export `BillingPeriods`.
- `src/component/top_bar.rs` — add billing-periods nav entry (TBD: only if it cleanly fits the existing nav row; otherwise leave the new page unlinked from the top bar but reachable via the existing flow → a `Btn` on the employees page is acceptable).
- `src/i18n/mod.rs`, `src/i18n/en.rs`, `src/i18n/de.rs`, `src/i18n/cs.rs` — four new keys.

Estimated diff size: ~1500 LOC change, ~600 LOC net addition (after deletions).

## Risks / Trade-offs

**[Splitting billing-period CRUD breaks anyone deep-linking to it]** → Today the URL is `/employees`. After the split, it moves to `/billing-periods` and the existing `/billing-periods/<id>` keeps working. Users who bookmarked `/employees` for billing-period work need to re-bookmark. Mitigation: announce in release notes; offer a discoverable link on the new employees page. Acceptable cost — billing-period CRUD is HR-only and rarely deep-linked.

**[Adding a `FormCheckbox` atom expands the change's scope]** → The form atoms in change 04 explicitly excluded checkbox/radio. Adding it here grows the change. Mitigation: the atom is small (~50 LOC + tests), follows the established `Form*` prefix convention, and unblocks the contract modal. The alternative — leaving the contract modal tied to the legacy `Checkbox` — keeps `base_components.rs` alive past the cleanup change, which is worse.

**[The 3-column grid stacks awkwardly on tablet widths]** → `auto-fit, minmax(280px, 1fr)` produces 1 column at <560 px, 2 at 560–840 px, 3 at ≥840 px. The 2-column case puts Gesamtansicht alone in column 1 and Arbeitsverträge+Histogram in column 2 with Zusatzarbeit wrapping below — readable but uneven. Mitigation: this is the standard CSS Grid auto-fit behavior; the reference design accepts it. If feedback is bad, bump `minmax` to 320 px so the 2-column case becomes 1 column on tablets too.

**[Histogram with fewer than 17 weeks of data renders sparse bars]** → New employees or recently-joined sales persons have <17 weeks. Mitigation: render only the available bars (don't pad with zero-height bars), keep the SVG width fixed, distribute the bars across the full width via `bar_width = (340 - 16) / weeks.len()`. The dashed expected-line still spans the full width.

**[Promoting `use_media_query` may introduce a circular dep]** → The `atoms` module currently has no use of WASM bindings; promoting the hook brings `wasm-bindgen` and `web-sys` into the atoms module's compile graph (already in the crate, but currently confined to `dialog.rs`). Mitigation: the dependencies are crate-wide; the promotion is a code-organization change with no new transitive imports. Verified by trial-build during implementation.

**[The reference design's "type pill" is rendered with `accent-soft` for paid and `warn-soft` for volunteer, but `warn-soft` is also used elsewhere as "missing staff" tinting]** → Two semantic uses of the same token color. Mitigation: in this page, the pill text (`Freiwillig`) carries the meaning. The user sees the color in context (a person card, not a shift cell), so the visual collision is unlikely to confuse. If feedback shows confusion, switch volunteer to `--good-soft` (matching the volunteer-bar token from the redesigned chart). Token rename not needed.

**[`EmployeeView` is consumed by two pages and the rewrite is sweeping]** → Risk that `MyEmployeeDetails` regresses. Mitigation: cover both call sites with SSR tests; when the dust settles, do a manual smoke of `/my-employee-details`. The data shape is identical in both cases, so the rewrite carries equal risk to both pages and equal coverage.

**[Selected-week state on the histogram is page-local, not URL-persistent]** → If the user reloads the page, the selected week is lost. Mitigation: acceptable trade-off. The selection is ephemeral inspection state; persisting it in the URL would clutter the URL bar without adding usability value. If a need arises (e.g. share-link to a specific week's detail), add a `?w=<n>` query param later.

**[The reference design says "color-dot avatar (no initials, no text inside circle)"; we render the same color in both list dot and detail circle]** → Anyone parsing color-by-position (e.g. dot at left of list = current employee) might expect them to match exactly. They do — both pull from `sales_person.background_color`. Verified by running both renderers off the same field. No further mitigation needed.

**[Contract modal field set is unchanged but the framing differs]** → Users familiar with the legacy modal layout (vertical `<Form>` with `FormPair` rows) will see a denser tokenized version. The fields and validations are identical; only the chrome changes. Risk minimal; mitigation: the labels and behaviors are preserved one-to-one.
