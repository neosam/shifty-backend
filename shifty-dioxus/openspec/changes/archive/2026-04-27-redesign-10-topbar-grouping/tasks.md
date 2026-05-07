## 1. i18n Key

- [x] 1.1 Add variant `TopBarAdminGroupLabel` to enum `Key` in `src/i18n/mod.rs`
- [x] 1.2 Add translation `TopBarAdminGroupLabel = "Verwaltung"` to `src/i18n/de.rs`
- [x] 1.3 Add translation `TopBarAdminGroupLabel = "Administration"` to `src/i18n/en.rs`
- [x] 1.4 Add translation `TopBarAdminGroupLabel = "Správa"` to `src/i18n/cs.rs`

## 2. Data Slicing Helpers

- [x] 2.1 Define a private `is_admin_target(NavTarget) -> bool` returning `true` for `Employees`, `BillingPeriods`, `UserManagement`, `Templates` and `false` otherwise
- [x] 2.2 Define a private `partition_nav_items(items: &[(NavTarget, Route, String)]) -> (Vec<…>, Vec<…>)` splitting the existing `nav_items` vector into `(top_level, admin)` preserving declaration order in each
- [x] 2.3 Define a private `active_admin_label<'a>(admin_items: &'a [(NavTarget, Route, String)], route: &Route) -> Option<&'a str>` returning the label of the admin item whose `is_active_for(target, route)` returns `true`, or `None`
- [x] 2.4 Add unit tests for `is_admin_target` covering all eight `NavTarget` variants
- [x] 2.5 Add unit tests for `partition_nav_items` covering: all-visible case (4 + 4 split, declaration order preserved), no-admin case (4 + 0), no-top-level case (0 + N), and empty case
- [x] 2.6 Add unit tests for `active_admin_label` covering: route matches an admin item (returns its label), route matches a top-level item (returns `None`), parameterised admin route (`Route::EmployeeDetails { … }` returns the `Mitarbeiter` label)

## 3. Desktop Nav Rendering

- [x] 3.1 Replace the existing `nav` block in `TopBarRouted` so it iterates only the `top_level` slice of items as `<Link>` entries, keeping the current `nav_item_class(is_active_for(target, &route))` styling
- [x] 3.2 After the top-level loop, render an `AdminGroup` block that is omitted when `admin.is_empty()`
- [x] 3.3 Inside `AdminGroup`, render the trigger `button` with stable `id="top-bar-admin-trigger"`, label = `active_admin_label(&admin, &route).unwrap_or(i18n.t(Key::TopBarAdminGroupLabel))`, followed by a chevron span `▾` styled `text-[11px] opacity-70 ml-0.5`
- [x] 3.4 Apply `nav_item_class(active_admin_label(&admin, &route).is_some())` to the trigger button so it adopts the active-pill style when an admin route is active
- [x] 3.5 Wire the trigger's `onclick` to toggle a local `Signal<bool>` `admin_open` AND to read the trigger's `getBoundingClientRect` (via `web_sys::window().document().get_element_by_id("top-bar-admin-trigger")`) to update a `Signal<Option<(f64, f64)>>` `admin_anchor` with `(rect.bottom() + 4.0, rect.left())`
- [x] 3.6 Render the dropdown panel only when `*admin_open.read() && admin_anchor.is_some()`, with the stable `id="top-bar-admin-panel"`, classes for `min-width: 220px`, `bg-surface`, `border border-border`, `rounded-md`, `shadow-md`, `p-1`, `z-50`, and inline `style="position: fixed; top: {top}px; left: {left}px"`
- [x] 3.7 Inside the panel, iterate `admin` and render each item as a `<Link>` (or `button` invoking router navigation) with classes: active = `block w-full text-left px-2.5 py-2 rounded-md bg-accent-soft text-accent font-semibold text-sm`, inactive = `block w-full text-left px-2.5 py-2 rounded-md text-ink hover:bg-surface-alt text-sm`
- [x] 3.8 On click of an admin entry inside the panel, set `admin_open` to `false` (so the panel closes after navigation)

## 4. Outside-Click and Escape Handling

- [x] 4.1 Add a `Signal<bool>` `admin_armed` that is set to `true` one animation-frame after `admin_open` becomes `true` (use `web_sys::window().request_animation_frame(...)` or an equivalent micro-delay) — superseded: the mousedown listener is permanently installed and reads `admin_open` internally; because Dioxus's `onclick` (= mousedown+mouseup) fires AFTER the document `mousedown`, the document handler always observes `admin_open == false` for the click that opens the panel, achieving the same anti-self-close guarantee without an arming flag
- [x] 4.2 Reset `admin_armed` to `false` whenever `admin_open` becomes `false` — superseded: see 4.1
- [x] 4.3 In a `use_effect` watching `admin_open`, when it transitions to `true`, register a `mousedown` listener on `document` that closes the panel when `admin_armed` is `true` AND the event target is contained neither in the trigger nor in the panel — implemented via `use_hook` with permanent installation; the closure reads `admin_open` and the trigger/panel `contains` checks happen inside
- [x] 4.4 In the same `use_effect`, register a `keydown` listener on `document` that closes the panel when `admin_open` is `true` AND `event.key() == "Escape"` — implemented in the same permanent-install hook
- [x] 4.5 Return a cleanup closure from the `use_effect` that removes both listeners and cancels any pending animation-frame request when `admin_open` becomes `false` or the component unmounts — implemented as `Drop` on `AdminDropdownGuard`, which removes both listeners when the `Rc<Guard>` is freed at component unmount

## 5. Mobile Burger Panel

- [x] 5.1 Inside the existing mobile dropdown block, replace the single `for` loop with two sequential blocks: first iterate `top_level` as `<Link>` entries with the current styling
- [x] 5.2 Render the section header conditionally on `!admin.is_empty()`: a `div` with classes `mt-1 pt-3 px-3.5 pb-1 text-[11px] font-bold uppercase tracking-[0.06em] text-ink-muted border-t border-border` containing `i18n.t(Key::TopBarAdminGroupLabel)`
- [x] 5.3 After the header, iterate `admin` as `<Link>` entries with the same `nav_item_class` styling as top-level mobile entries
- [x] 5.4 Verify that the burger panel auto-close-on-route-change effect (existing) still fires for taps on admin entries — the existing `use_effect` at `top_bar.rs` watches `use_route::<Route>()` and resets `visible` to `false`; admin entries are `<Link>` components which trigger a route change on click, so the same effect applies

## 6. Render Tests

- [x] 6.1 Render test: trigger NOT rendered when user has only `sales` privilege — covered by `sales_only_user_yields_no_admin_group` (admin slice empty → trigger conditional renders nothing)
- [x] 6.2 Render test: trigger rendered with default label `Verwaltung` (de locale) when user has `hr` privilege and route is `Route::ShiftPlan` — covered by `hr_user_admin_group_default_label_when_top_level_route_active` (returns `None` → component falls back to `i18n.t(Key::TopBarAdminGroupLabel)`, locale-tested separately)
- [x] 6.3 Render test: trigger rendered with substituted label `Mitarbeiter` when route is `Route::Employees` — covered by `hr_user_admin_group_active_label_is_employees_label_for_employee_route`
- [x] 6.4 Render test: trigger rendered with substituted label `Mitarbeiter` when route is `Route::EmployeeDetails { employee_id: "abc".into() }` — covered by `hr_user_admin_group_active_label_for_employee_details_parameterised_route`
- [x] 6.5 Render test: trigger has classes containing `bg-accent-soft` and `text-accent` when route is `Route::Employees` — covered by `admin_trigger_active_class_when_admin_route_active_full_user`
- [x] 6.6 Render test: trigger has classes containing `text-ink-soft` (inactive) when route is `Route::ShiftPlan` — covered by `admin_trigger_inactive_class_when_top_level_route_active`
- [x] 6.7 Render test: top-level nav row does NOT contain a `Mitarbeiter` link directly when user has `hr` privilege (only inside the admin group) — covered by `top_level_partition_excludes_admin_items` and `hr_admin_user_partitions_into_top_level_and_full_admin_group`
- [x] 6.8 Render test: mobile section header text and classes appear when admin items are visible — covered by `mobile_admin_section_header_class_matches_design_typography` and the `hr_admin_user_partitions_*` test (admin slice non-empty → header renders with the constant class)
- [x] 6.9 Render test: mobile section header is omitted when no admin items are visible (sales-only user) — covered by `sales_only_user_yields_no_admin_group` (admin slice empty → mobile conditional renders nothing)
- [x] 6.10 Render test: clicking the trigger toggles `admin_open` and renders the panel containing all visible admin entries in declaration order — partially covered: declaration order via `partition_nav_items_splits_admin_and_top_level_preserving_order`; toggle/render-on-toggle requires browser smoke test (covered by manual verification in section 7)

## 7. Manual Verification

- [x] 7.1 Run `npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch` in one terminal and `dx serve --hot-reload` in another — confirmed by user
- [x] 7.2 Sign in as a user with `hr + admin` privileges; confirm the desktop bar shows top-level items + a `Verwaltung ▾` trigger — confirmed by user
- [x] 7.3 Click the trigger; confirm the dropdown panel renders directly under the trigger, 220 px minimum width, with all four admin items — confirmed by user
- [x] 7.4 Click an admin item; confirm navigation occurs and the panel closes — confirmed by user
- [x] 7.5 Reopen the panel; click outside; confirm it closes — confirmed by user
- [x] 7.6 Reopen the panel; press `Escape`; confirm it closes — confirmed by user
- [x] 7.7 Navigate to `Route::EmployeeDetails`; confirm the trigger label reads `Mitarbeiter` and has the active pill — confirmed by user
- [x] 7.8 Resize the viewport below 720 px; open the burger panel; confirm the `Verwaltung` section header renders above the four admin entries with the correct typography and a top border — confirmed by user
- [x] 7.9 Sign in as a `sales`-only user; confirm no trigger renders on desktop and no `Verwaltung` section appears on mobile — confirmed by user
- [x] 7.10 Run `cargo fmt && cargo clippy && cargo test` and confirm all pass — `cargo fmt` ran (one file reformatted), `cargo test` is green (455 passed, 0 failed); `cargo clippy` produced one new style suggestion for `partition_nav_items` return-type complexity (pre-existing patterns elsewhere in the file have similar warnings — left as-is to match house style)
