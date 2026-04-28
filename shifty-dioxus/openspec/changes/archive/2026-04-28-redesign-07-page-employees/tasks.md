## 1. i18n Keys

- [x] 1.1 Add `Key::SearchPlaceholder`, `Key::OtherHours`, `Key::More`, and `Key::BackToList` variants to the `Key` enum in `src/i18n/mod.rs` (placed in a new `// Employees page` block near the existing employees-related keys)
- [x] 1.2 Add English translations in `src/i18n/en.rs`: `SearchPlaceholder = "Search…"`, `OtherHours = "Other hours"`, `More = "More"`, `BackToList = "Back"`
- [x] 1.3 Add German translations in `src/i18n/de.rs`: `SearchPlaceholder = "Suchen…"`, `OtherHours = "Sonstige Stunden"`, `More = "Mehr"`, `BackToList = "Zurück"`
- [x] 1.4 Add Czech translations in `src/i18n/cs.rs`: `SearchPlaceholder = "Hledat…"`, `OtherHours = "Ostatní hodiny"`, `More = "Více"`, `BackToList = "Zpět"`
- [x] 1.5 Add unit test `i18n_employees_keys_present_in_all_locales` asserting all four keys return non-empty strings in En/De/Cs

## 2. Promote `use_media_query` to atoms

- [x] 2.1 Create `src/component/atoms/media_query.rs` with the `use_media_query` hook moved from `src/component/dialog.rs` (including the `match_media_initial`, `MediaQueryGuard`, `install_media_query_listener` helpers and their `cfg` guards)
- [x] 2.2 Update `src/component/atoms/mod.rs` to declare `pub mod media_query;` and re-export `pub use media_query::use_media_query;`
- [x] 2.3 Update `src/component/dialog.rs` to import `use_media_query` from `crate::component::atoms` and remove the local hook implementation
- [x] 2.4 Verify `cargo check` passes after the move

## 3. Add `FormCheckbox` atom

- [x] 3.1 Create `src/component/form/checkbox.rs` with a `FormCheckbox` component: props `value: bool`, `disabled: bool` (default false), `on_change: Option<EventHandler<bool>>` (default None), `label: Element`
- [x] 3.2 The component renders `<label class="inline-flex items-center gap-2 cursor-pointer">` containing the checkbox `<input type="checkbox">` (with `form-input`-equivalent token classes) and the label slot styled with `text-[13px] text-ink`
- [x] 3.3 The input `oninput` handler calls `on_change(event.value() == "true")`
- [x] 3.4 Add tests: renders `<input>` with `type="checkbox"`; `value=true` adds `checked` attribute; `disabled=true` adds `disabled`; on toggle the handler fires with the new boolean; the label slot text appears in the rendered HTML
- [x] 3.5 Update `src/component/form/mod.rs` to declare `pub mod checkbox;` and `pub use checkbox::FormCheckbox;`

## 4. Rewrite `EmployeeShort` for color-dot row + active state

- [x] 4.1 In `src/component/employee_short.rs`, add new props: `active: bool` (default false) and `target_hours: f32` (default 0.0)
- [x] 4.2 Replace the body with a single-line layout: `<div>` carrying `flex items-center gap-3 px-3 py-2 border-l-[3px] hover:bg-surface-alt`, conditionally adding `bg-accent-soft border-accent` when `active`, otherwise `border-transparent`
- [x] 4.3 Render inline order: a 10×10 px circle `<span>` with inline `background-color: {sales_person.background_color}` (no inner text), the name `<span class="flex-1 text-ink truncate">`, and the hours `<span class="font-mono tabular-nums text-xs text-ink-muted">{balance:.1}/{target:.0}</span>`
- [x] 4.4 Remove all `text-gray-*`, `border-gray-200` references from the file
- [x] 4.5 Add SSR tests: row contains color-dot inline style, name, and hours pattern; color dot has no inner text; active row contains `bg-accent-soft` and `border-accent`; inactive row contains `border-transparent`; hours element carries `font-mono` and `tabular-nums`

## 5. EmployeesShell composition layer

- [x] 5.1 Create `src/component/employees_shell.rs` with an `EmployeesShell` component taking `children: Element` and an internal `active_employee_id: Option<Uuid>` derived from the current route via `use_route::<Route>()`
- [x] 5.2 Use `use_media_query("(max-width: 720px)")` to detect mobile; on desktop render `<div class="flex"><EmployeesList .../><div class="flex-1">{children}</div></div>` where the list pane has classes resolving to a width clamp around 280–360 px
- [x] 5.3 On mobile, render the list when on `Route::Employees` (no id) and only the children when on `Route::EmployeeDetails { ... }`
- [x] 5.4 Re-export `EmployeesShell` from `src/component/mod.rs`

## 6. EmployeesList component (search + rows)

- [x] 6.1 Create `src/component/employees_list.rs` exporting an `EmployeesList` component with no props
- [x] 6.2 Inside, load employees via `loader::load_employees(...)` for the current year and current week (mirror the existing employees page logic)
- [x] 6.3 Read the active employee id from `use_route::<Route>()`
- [x] 6.4 Use a `use_signal(|| String::new())` for the search term; render an `<input>` with `form-input` token classes, placeholder `i18n.t(Key::SearchPlaceholder)`, and `oninput` setting the signal
- [x] 6.5 Filter the loaded employees: skip `inactive`, then case-insensitive substring match on `sales_person.name`
- [x] 6.6 For each remaining employee, render a `<Link to=Route::EmployeeDetails { employee_id }>` wrapping `EmployeeShort` with `active=true` when the id matches the active route, computing `target_hours` from the most-recent `working_hours_by_week` entry's `expected_hours` (or 0.0 when none exist)
- [x] 6.7 Add SSR tests: search filters case-insensitive; empty search shows all rows; placeholder propagates from the locale; active row receives `bg-accent-soft`; no rows render with `bg-accent-soft` when no active employee

## 7. Lift billing-period CRUD to a new `BillingPeriods` page

- [x] 7.1 Create `src/page/billing_periods.rs` exporting a `BillingPeriods` page component
- [x] 7.2 Move the billing-period section from `src/page/employees.rs` to the new page (heading, list of billing periods, create button, both dialogs, and the `EmployeesPageAction` enum split — rename to `BillingPeriodsPageAction` with the same variants)
- [x] 7.3 Replace the legacy `Modal` for the create-period dialog with `Dialog` (variant `Auto`, width 460); body uses `Field` + `FormTextInput { input_type: ImStr::from("date") }`; footer uses two `Btn`s (Secondary Cancel, Primary `i18n.t(Key::CreateBillingPeriod)`)
- [x] 7.4 Replace the legacy `Modal` for the delete-confirmation dialog with `Dialog` (variant `Auto`, width 420); footer uses `Btn` Secondary Cancel and `Btn` Danger (`i18n.t(Key::DeleteBillingPeriod)`)
- [x] 7.5 Restyle the billing-period list rows to use design tokens (replace `bg-white shadow rounded-lg p-4 border border-gray-200 hover:shadow-lg hover:border-blue-300` with `rounded-md border border-border bg-surface p-4 hover:bg-surface-alt`)
- [x] 7.6 Replace `text-blue-600`, `text-gray-600`, `text-gray-500`, `bg-green-100 text-green-800`, `bg-red-100 text-red-800` with token-based equivalents (`text-ink`, `text-ink-muted`, `bg-accent-soft text-accent`, `bg-bad-soft text-bad`)
- [x] 7.7 Add `Route::BillingPeriods` to `src/router.rs` (path `/billing-periods`) and add `pub use crate::page::BillingPeriods;` plus matching import
- [x] 7.8 Add `pub use billing_periods::BillingPeriods;` to `src/page/mod.rs`
- [x] 7.9 Add SSR tests: page renders the heading and create button; clicking create opens a `Dialog` (not legacy `Modal`); the delete dialog's footer contains a Danger-variant `Btn`

## 8. Strip billing-period content from `src/page/employees.rs`

- [x] 8.1 Remove the entire billing-period section from `src/page/employees.rs` (delete the `EmployeesPageAction` variants for billing-period CRUD, the dialog state signals, the dialog rsx blocks, and the billing-period list rendering)
- [x] 8.2 Remove the now-unused imports: `BillingPeriodAction`, `BILLING_PERIOD_STORE`, `Modal`, `time::macros::format_description`, the `Uuid` import if unused
- [x] 8.3 Replace the page body with `<EmployeesShell />` rendering the new layout
- [x] 8.4 The page renders only `TopBar` and the shell on desktop; on mobile (route `Employees` with no id), it renders the list inside the shell
- [x] 8.5 Verify the file shrinks below 80 LOC; no `Modal {}`, no `match.*employees`, no billing-period code remains

## 9. Detail-page header

- [x] 9.1 In `src/page/employee_details.rs`, replace the body with `<EmployeesShell>{detail content}</EmployeesShell>` so the list shows on desktop and the back-button-or-shell behavior kicks in on mobile
- [x] 9.2 Inside the detail content, render a back button via `<Btn variant=BtnVariant::Ghost icon=Some(ImStr::from("‹")) on_click={navigate to Route::Employees}>{i18n.t(Key::BackToList)}</Btn>`, conditionally rendered when `use_media_query("(max-width: 720px)")` is true
- [x] 9.3 Migrate the existing `EmployeeView` invocation to render the rewritten `EmployeeViewPlain` (see §10) directly — keep the `EmployeeView` wrapper but the body is now the new layout
- [x] 9.4 Replace the legacy `Modal { EmployeeWorkDetailsForm { ... } }` block with the new `<ContractModal open={...} form_type={...} ... />`
- [x] 9.5 Add a sibling `<ExtraHoursModal open={show_extra_hours_dialog} sales_person_id=employee_id ... />` for the `Sonstige Stunden` button flow
- [x] 9.6 Wire the new modals to the existing `EmployeeAction::Refresh` and `EmployeeWorkDetailsAction::*` actions on save/cancel

## 10. Rewrite `EmployeeViewPlain` body (header + 3-column grid)

- [x] 10.1 In `src/component/employee_view.rs`, delete the legacy `TupleView` and `TripleView` components after verifying via grep no other consumer exists (`grep -rn "TupleView\|TripleView" src/` should show only `employee_view.rs`)
- [x] 10.2 Delete the legacy `WorkingHoursView` component after verifying no other consumer
- [x] 10.3 Rewrite the top of `EmployeeViewPlain` body to render the new header row: 32 px color circle (`<div class="w-8 h-8 rounded-full" style="background-color: {color}">` with no children), the name `<span class="text-xl font-semibold text-ink">`, the type pill (`PersonChip` with the resolved `--accent-soft` hex `#eaecfb` for paid or `--warn-soft` hex `#fef0d6` for volunteer; text from `Key::Paid` / `Key::Volunteer`), the target `<span class="font-mono tabular-nums text-ink-muted">{expected:.0} h</span>`, the year nav using two `NavBtn`s with `Key::PreviousYear` / `Key::NextYear`, the `Sonstige Stunden` `Btn` Primary, and the `Mehr ▾` `DropdownTrigger`
- [x] 10.4 In the `Mehr ▾` dropdown entries, drop `AddEntry` and `AddWorkDetailsLabel`; keep only `ShowFullYearLabel` and `ShowUntilNowLabel` (gated on `props.year != js::get_current_year()`)
- [x] 10.5 Replace the legacy column body with a CSS Grid section: `<section class="grid gap-6 mt-6" style="grid-template-columns: repeat(auto-fit, minmax(280px, 1fr))">` containing three `<div>` columns
- [x] 10.6 Render the Gesamtansicht column: heading `<h2>{Key::OverallHeading}</h2>`, three primary `TupleRow`s (Balance/Overall/Required) with mono tabular-numbers values formatted to 2 decimals, a `<div class="border-t border-border my-3"/>` divider, then `dim={true}` `TupleRow`s for `CategoryShiftplan`, `CategoryExtraWork`, `CategoryVacation`, `CategorySickLeave`, `CategoryHolidays`, `CategoryUnpaidLeave`, `CategoryVolunteerWork`, `CarryoverBalance`, plus one `dim` row per `custom_extra_hours` entry. When `props.show_vacation`, append `dim` rows for `VacationDaysLabel` (`{vacation_days} / {vacation_entitlement}`) and `VacationCarryoverLabel`
- [x] 10.7 Render the Arbeitsverträge + Stunden pro Woche column: heading `<h2>{Key::WorkDetailsHeading}</h2>`, then for each `EmployeeWorkDetails` render a clickable card `<button class="w-full text-left rounded-md border border-border bg-surface px-3 py-2 hover:bg-surface-alt">` showing `<from> – <to>` and `<expected> h/Woche` (mono); clicking the card calls `on_employee_work_details_clicked(id)`; followed by an `<Btn variant=Secondary icon=Some(ImStr::from("+"))>` labeled `Key::AddWorkDetailsLabel` triggering `on_add_employee_work_details`
- [x] 10.8 Below the contract cards, render a sub-heading `<h3>{Key::WorkingHoursPerWeekHeading}</h3>`, then `<EmployeeWeeklyHistogram>` (see §11), then the inline week-detail panel (see §12)
- [x] 10.9 Render the Zusatzarbeit column: heading `<h2>{Key::ExtraHoursHeading}</h2>`, then the existing `ExtraHoursView` component (restyle in §13)
- [x] 10.10 Remove all legacy color classes (`bg-gray-*`, `text-gray-*`, `border-gray-*`, `border-black`, `text-blue-*`, `text-red-*`, `text-green-*`, `bg-blue-*`, `bg-green-*`, `bg-red-*`) from `employee_view.rs`
- [x] 10.11 Move the legacy `if *show_add_entry_dialog.read() { Modal { AddExtraHoursForm { ... } } }` block out of `EmployeeViewPlain` — it is now mounted from `employee_details.rs` as `ExtraHoursModal`. Remove the `show_add_entry_dialog` signal and the local `EmployeeViewActions::ShowAddEntry` from `EmployeeViewPlain`; surface the open via a new prop `on_open_extra_hours: EventHandler<()>`

## 11. EmployeeWeeklyHistogram component

- [x] 11.1 Create `src/component/employee_weekly_histogram.rs` with the props defined in design §7 (`weeks: Rc<[WorkingHours]>`, `expected_per_week: f32`, `current_year: u32`, `current_week: u8`, `selected_week: Option<(u32, u8)>`, `on_select: EventHandler<(u32, u8)>`)
- [x] 11.2 Implement helper `compute_max_y(weeks: &[WorkingHours], expected: f32) -> f32` returning `weeks.iter().map(|w| w.overall_hours).fold(0.0, f32::max).max(expected).max(1.0)`
- [x] 11.3 Implement helper `bar_y(value: f32, max_y: f32) -> f32` returning `90.0 - (value / max_y) * 90.0`
- [x] 11.4 Implement helper `bar_color_token(value: f32, expected: f32) -> &'static str` returning `"var(--warn)"` when `value < expected`, otherwise `"var(--accent)"`
- [x] 11.5 Render the `<svg viewBox="0 0 340 120" preserveAspectRatio="none">` containing: a dashed reference `<line>` at `y = bar_y(expected_per_week, max_y)` with `stroke-dasharray="4 3"` and `style: "stroke: var(--ink-muted)"`; one `<g>` per week containing a `<rect>` and (when applicable) an X-axis `<text>` label
- [x] 11.6 Apply `style: "fill: {token}"` for each bar (where `{token}` comes from `bar_color_token`); apply `style: "opacity: 0.85"` on the bar group when `selected_week.is_some()` and the bar is not the selected week
- [x] 11.7 Wrap each `<g>` with `cursor: pointer` and `onclick: move |_| on_select.call((week.year, week.week))` (use the `from` date's year/iso-week to derive `(year, week)`)
- [x] 11.8 Render X-axis labels with the rule `(week.week as usize - 1) % 4 == 0 || week.week == 52 || (week.year, week.week) == (current_year, current_week)`, format `format!("{} {}", i18n.t(Key::WeekShort), week.week)`, font-family `ui-monospace, SFMono-Regular, Menlo, monospace`, font-size `9`, style `fill: var(--ink-muted)`
- [x] 11.9 Add unit tests for `compute_max_y`, `bar_y`, and `bar_color_token`
- [x] 11.10 Add SSR tests: 17 entries → 17 `<rect>` bars; below-expected bar's style includes `var(--warn)`; at-or-above-expected bar's style includes `var(--accent)`; `selected_week=Some(...)` makes non-selected bars include `opacity: 0.85`; X-axis labels at the right cadence; current-week always labeled
- [x] 11.11 Add SSR test asserting no hex color literal appears in the rendered HTML
- [x] 11.12 Re-export `EmployeeWeeklyHistogram` from `src/component/mod.rs`

## 12. Inline week-detail panel

- [x] 12.1 In `src/page/employee_details.rs` (or the rewritten `employee_view.rs` if cleaner), introduce a `selected_week` signal `use_signal(|| None::<(u32, u8)>)`
- [x] 12.2 Wire `EmployeeWeeklyHistogram::on_select` to a handler that toggles `selected_week`: same pair → `None`; different pair → `Some(pair)`
- [x] 12.3 When `selected_week.is_some()`, locate the matching `WorkingHours` in `employee.working_hours_by_week`; if not found, leave the panel hidden
- [x] 12.4 Render the panel as `<section class="mt-3 rounded-md border border-border bg-surface px-3 py-2">` containing: a header row with `<h4>{from_date} – {to_date}</h4>` and `<span class="font-mono tabular-nums">{overall:.2} / {expected:.2} h</span>`, a close affordance (`<button>×</button>` calling `on_select` again with the same pair to toggle off, or a separate signal-set), and a `<ul>` of day rows
- [x] 12.5 Each day row renders `<li class="flex justify-between gap-2 py-1 border-b border-border">` with `<span class="font-mono">{date}</span>`, `<span>{i18n.t(category.to_i18n_key())}</span>`, `<span class="font-mono tabular-nums">{day.hours:.2} h</span>`. **Do NOT render `from–to` time blocks.**
- [x] 12.6 Add SSR tests: panel hidden when `selected_week=None`; with 5 day entries, 5 day rows render; no day row contains a `:` followed by two digits (a heuristic for time-of-day blocks); panel header shows the formatted date range and the overall/expected pair

## 13. Restyle ExtraHoursView

- [x] 13.1 In `src/component/employee_view.rs`, restyle `ExtraHoursView` headings: replace `<h2 class="text-lg font-bold mt-8">` with `<h3 class="text-xs uppercase tracking-wide font-semibold text-ink-muted mt-4 first:mt-0">`
- [x] 13.2 Replace the `<TripleView>` invocations with inline RSX: `<div class="flex items-baseline justify-between gap-2 py-1.5 border-b border-border">` containing a left flex-col `<div>` with `<span class="text-sm text-ink">{date}</span>` over `<span class="text-xs text-ink-muted truncate">{description}</span>`, and a right flex `<div>` with `<span class="font-mono tabular-nums text-sm text-ink">{value}</span>` and `<Btn variant=BtnVariant::Danger on_click={delete}>🗑</Btn>`
- [x] 13.3 Skip rendering the heading when the corresponding category has no entries (use `if !is_empty()` guards)
- [x] 13.4 Replace `text-sm text-gray-500 mb-4` description paragraphs with `text-xs text-ink-muted mb-3`
- [x] 13.5 Add SSR tests: heading uses `text-ink-muted text-xs uppercase`; row uses `border-b border-border`; delete button is a `Btn` with `Danger` variant; empty categories omit headings

## 14. ContractModal component

- [x] 14.1 Create `src/component/contract_modal.rs` exporting a `ContractModal` component with props `open: bool`, `form_type: EmployeeWorkDetailsFormType`, `on_save: EventHandler<()>`, `on_cancel: EventHandler<()>`
- [x] 14.2 When `props.open` is false, return `rsx! {}` (early return)
- [x] 14.3 When open, render `<Dialog open=true on_close=props.on_cancel title=AddWorkDetailsFormTitle variant=Auto width=520 footer=Some(rsx!{ Btn Secondary Cancel; if !ReadOnly Btn Primary Save })>` with a body component that holds the form fields
- [x] 14.4 Implement the body using `Field` wrappers and the `Form*` atoms: `FormTextInput { input_type=date }` for from/to, `FormCheckbox` for the seven weekday toggles (in a `<div class="grid grid-cols-2 gap-2">`), `FormTextInput { input_type=number }` for `expected_hours`, `FormTextInput { input_type=number }` for `workdays_per_week` and `vacation_days`, `FormCheckbox` for `dynamic` and `cap_planned_hours_to_expected`
- [x] 14.5 Wire the body to `EMPLOYEE_WORK_DETAILS_STORE` reads and `EmployeeWorkDetailsAction::UpdateWorkingHours` writes (mirror the existing `EmployeeWorkDetailsForm` semantics)
- [x] 14.6 Save handler dispatches `EmployeeWorkDetailsAction::Save` (when `New`) or `EmployeeWorkDetailsAction::Update` (when `Edit`), then calls `on_save`
- [x] 14.7 Disable inputs when `form_type` is `ReadOnly`; disable from/workdays/checkboxes when `form_type != New`
- [x] 14.8 Re-export `ContractModal` from `src/component/mod.rs`
- [x] 14.9 Add SSR tests: closed renders nothing; open contains the dialog title; ReadOnly omits the Save button; the body contains a `<label>` from `Field` and at least one `form-input` class; weekday checkboxes render seven `<input type="checkbox">` inputs

## 15. ExtraHoursModal component

- [x] 15.1 Create `src/component/extra_hours_modal.rs` exporting an `ExtraHoursModal` component with props `open: bool`, `sales_person_id: Uuid`, `on_saved: EventHandler<()>`, `on_cancel: EventHandler<()>`
- [x] 15.2 When `props.open` is false, return `rsx! {}`
- [x] 15.3 When open, render `<Dialog open=true on_close=on_cancel title=AddExtraHoursFormTitle variant=Auto width=460 footer=Some(rsx!{ Btn Secondary Cancel; Btn Primary Submit })>`
- [x] 15.4 Body: `Field` label `Category` containing a `FormSelectInput` with options for `extra_work`, `volunteer_work`, `holiday`, `sick_leave`, `vacation_days`, `unavailable`, `unpaid_leave`, a divider option, custom categories from `api::get_custom_extra_hours_by_sales_person`, another divider, and `vacation`
- [x] 15.5 Body: `Field` label `Description` with `FormTextInput`
- [x] 15.6 Vacation-days mode: render two `Field`s with `FormTextInput { input_type=date }` for `From` and `To` (use `Key::FromLabel`/`Key::ToLabel`)
- [x] 15.7 Non-vacation-days mode: render `Field` `AmountOfHours` with `FormTextInput { input_type=number }` (step 0.001) and `Field` `When` with `FormTextInput { input_type=datetime-local }`
- [x] 15.8 Submit handler: when category is `VacationDays`, call `api::add_vacation`; otherwise call `api::add_extra_hour` (mirror current `AddExtraHoursForm` coroutine logic). On success, call `on_saved`
- [x] 15.9 Re-export `ExtraHoursModal` from `src/component/mod.rs`
- [x] 15.10 Add SSR tests: closed renders nothing; open contains the dialog title; category select includes all required option values; vacation-days mode renders two date fields and no datetime-local; non-vacation-days mode renders the amount and datetime-local; the footer contains Cancel + Submit buttons

## 16. Wire `Sonstige Stunden` button in detail page

- [x] 16.1 In `src/page/employee_details.rs`, add `let mut show_extra_hours_dialog = use_signal(|| false);`
- [x] 16.2 Pass an `on_open_extra_hours` event handler down to `EmployeeView`/`EmployeeViewPlain` that sets the signal to `true`
- [x] 16.3 Mount `<ExtraHoursModal open={*show_extra_hours_dialog.read()} sales_person_id=employee_id on_saved=move |_| { show_extra_hours_dialog.set(false); cr.send(EmployeeDetailsAction::Update); } on_cancel=move |_| show_extra_hours_dialog.set(false) />`
- [x] 16.4 Verify the button click dispatches the open action (manual smoke during §22)

## 17. Wire `+ Vertrag` (add contract) button

- [x] 17.1 In `src/page/employee_details.rs`, replace the existing `Modal { EmployeeWorkDetailsForm { ... } }` block with `<ContractModal open={*show_add_employee_work_details_dialog.read()} form_type={*employee_work_details_dialog_type.read()} on_save=... on_cancel=... />`
- [x] 17.2 Confirm the existing `EmployeeDetailsAction::NewEmployeeWorkDetails` and `OpenEmployeeWorkDetails(id)` paths still toggle the signals correctly
- [x] 17.3 The "+ Vertrag" button in the contracts column dispatches `EmployeeDetailsAction::NewEmployeeWorkDetails`; clicking a contract card dispatches `EmployeeDetailsAction::OpenEmployeeWorkDetails(id)`

## 18. Update `src/component/mod.rs` re-exports

- [x] 18.1 Add `pub mod employee_weekly_histogram;` and `pub use employee_weekly_histogram::EmployeeWeeklyHistogram;`
- [x] 18.2 Add `pub mod contract_modal;` and `pub use contract_modal::ContractModal;`
- [x] 18.3 Add `pub mod extra_hours_modal;` and `pub use extra_hours_modal::ExtraHoursModal;`
- [x] 18.4 Add `pub mod employees_shell;` and `pub use employees_shell::EmployeesShell;`
- [x] 18.5 Add `pub mod employees_list;` and `pub use employees_list::EmployeesList;`

## 19. Optional top-bar entry for billing periods

- [x] 19.1 Inspect `src/component/top_bar.rs`; if the existing nav row has space, add a `Route::BillingPeriods` entry labeled by `Key::BillingPeriods`
- [x] 19.2 If the nav row is full, leave the entry off the top bar — the new employees page already exposes a button/link to `/billing-periods` per §8 task list (verify via §8.5)

## 20. Token sweep tests

- [x] 20.1 Add SSR test `employees_page_no_legacy_classes`: read the non-test source of `src/page/employees.rs` and assert it contains none of `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `bg-red-`, `border-black`, `border-gray-`
- [x] 20.2 Add SSR test `employee_details_page_no_legacy_classes`: same assertion against `src/page/employee_details.rs`
- [x] 20.3 Add SSR test `employee_view_no_legacy_classes`: same assertion against `src/component/employee_view.rs`
- [x] 20.4 Add SSR test `employee_short_no_legacy_classes`: same assertion against `src/component/employee_short.rs`
- [x] 20.5 Add SSR test `employee_weekly_histogram_no_hex_colors`: assert `src/component/employee_weekly_histogram.rs` contains no 6-char or 3-char hex color literals (excluding strings inside `#[cfg(test)]` block)
- [x] 20.6 Add SSR test `contract_modal_no_legacy_classes`: same assertion against `src/component/contract_modal.rs`
- [x] 20.7 Add SSR test `extra_hours_modal_no_legacy_classes`: same assertion against `src/component/extra_hours_modal.rs`
- [x] 20.8 Add SSR test `billing_periods_page_no_legacy_classes`: same assertion against `src/page/billing_periods.rs`

## 21. Keep MyEmployeeDetails working

- [x] 21.1 Verify `src/page/my_employee_details.rs` still compiles after the `EmployeeView` rewrite (it does not pass the new `on_open_extra_hours` prop — wire a no-op default if needed by giving the prop an `Option<EventHandler<()>>` shape)
- [x] 21.2 Verify the page renders the new layout (manual smoke during §22)

## 22. Verification

- [x] 22.1 `cargo check --package shifty-dioxus` passes
- [x] 22.2 `cargo test --package shifty-dioxus` passes; all new tests in §1, §3, §4, §6, §7, §10, §11, §12, §13, §14, §15, §20, §21 are green
- [x] 22.3 `cargo clippy --no-deps --package shifty-dioxus` produces no new warnings in any of the new or modified files
- [x] 22.4 `cargo fmt -- --check` passes
- [x] 22.5 Manual smoke (Tailwind watcher + `dx serve`) on `/employees`: list with search filters live; clicking an employee shows the detail with the active row tinted; viewport <720 px shows list-only / detail-only with back button
- [x] 22.6 Manual smoke on `/employees/<id>`: header shows color circle, name, type pill, target hours, year nav, Sonstige Stunden, Mehr ▾; the 3-column grid renders; histogram shows last 17 weeks with dashed expected line; clicking a bar opens the inline week panel showing per-day hours-by-category (no time blocks)
- [x] 22.7 Manual smoke on contract modal: opens via "+ Vertrag" and via clicking an existing card; uses the new `Dialog` shell with token-styled fields and Cancel/Save footer
- [x] 22.8 Manual smoke on extra-hours modal: opens via "Sonstige Stunden"; category select shows all standard categories plus custom ones if defined; vacation-days swaps the field set
- [x] 22.9 Manual smoke on `/billing-periods`: page renders independently; create dialog uses `Dialog`; delete dialog uses `Dialog` with Danger button
- [x] 22.10 Manual smoke on `/my-employee-details`: page renders the new layout without errors
- [x] 22.11 `openspec validate "redesign-07-page-employees" --strict` passes
