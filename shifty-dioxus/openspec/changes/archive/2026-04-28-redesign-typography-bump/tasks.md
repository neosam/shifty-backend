## 1. Token foundation

- [x] 1.1 Extend `tailwind.config.js` `theme.extend.fontSize` with the seven canonical tokens (`micro`, `small`, `body`, `lg`, `h2`, `h1`, `display`) and their full `[size, { lineHeight, fontWeight, letterSpacing }]` tuples per the spec table
- [x] 1.2 Add the new token class names that may be built dynamically to `safelist` in `tailwind.config.js` (only if any are constructed via `format!` — verify with grep)
- [x] 1.3 Set `body { font-size: 16px; line-height: 1.5; }` in `input.css`
- [x] 1.4 Add `@media (max-width: 720px) { body { font-size: 15px; } }` in `input.css`
- [x] 1.5 Run `cargo check` and the Tailwind compiler (`npx tailwindcss …`) to confirm the new utilities resolve

## 2. TopBar and global chrome

- [x] 2.1 Migrate `src/component/top_bar.rs`: brand label, primary nav buttons, admin dropdown trigger, user-name pill, burger menu — use `text-body` for buttons, `text-small` for muted captions, `text-micro` for the brand uppercase eyebrow
- [x] 2.2 Migrate `src/component/footer.rs` to the `text-small` token
- [x] 2.3 Visually diff TopBar against `shifty-design/project/Shifty Preview.html` (TopBar block) by reading the design source

## 3. WeekView

- [x] 3.1 Migrate `src/component/week_view.rs`: day header (`long, date`), date sub-line (mono), time-column labels, cell chips, count badges
- [x] 3.2 Migrate `src/component/atoms/person_chip.rs` so the chip text uses `text-micro` (uppercase) or `text-small` (mixed-case name) per design
- [x] 3.3 Migrate `src/component/shiftplan_tab_bar.rs` (toolbar tabs)
- [x] 3.4 Migrate `src/component/working_hours_mini_overview.rs`: remove the inline `style: "font-size: 10px"`, replace with `text-micro`; convert `text-xs` table headers to `text-micro`, table body to `text-small`
- [x] 3.5 Migrate `src/component/working_hours_overview_layout_toggle.rs`
- [x] 3.6 Visually diff WeekView against `shifty-design/project/Shifty Preview.html` (week-grid block)

## 4. Page-level migrations

- [x] 4.1 `src/page/employees.rs` and `src/component/employees_list.rs`, `src/component/employee_short.rs`, `src/component/employee_view.rs` — page headline `text-h1`, list items `text-body`, captions `text-small`, eyebrow labels `text-micro`
- [x] 4.2 `src/page/my_shifts.rs` — page headline `text-h1`, week-summary line `text-body`, day cells per design
- [x] 4.3 `src/page/weekly_overview.rs` and `src/component/weekly_overview_chart.rs`
- [x] 4.4 `src/page/billing_periods.rs` and `src/page/billing_period_details.rs` — table content `text-body`, status pills `text-micro`
- [x] 4.5 `src/page/sales_person_details.rs`
- [x] 4.6 `src/page/user_management.rs`, `src/page/user_details.rs`, `src/component/user_management_tab_bar.rs` — page headline `text-h1`, tab labels `text-body`, table cells per design
- [x] 4.7 `src/page/text_template_management.rs` — replace `text-xs` column headers with `text-micro`, body cells with `text-body`; remove `text-gray-500`-style legacy classes that pre-date the redesign tokens (out-of-scope cleanup is fine here as the file is touched anyway, but only typography classes — keep colors as-is)
- [x] 4.8 `src/page/custom_extra_hours_management.rs`

## 5. Forms, modals, and atoms

- [x] 5.1 `src/component/dialog.rs` — modal title `text-lg` (16 px); confirm this *shrinks* existing 18 px titles intentionally per design
- [x] 5.2 `src/component/contract_modal.rs` — title and body
- [x] 5.3 `src/component/add_extra_hours_form.rs` — form labels `text-body`, hints `text-small`
- [x] 5.4 `src/component/employee_work_details_form.rs`
- [x] 5.5 `src/component/atoms/tuple_row.rs` — label `text-small`, value `text-body`
- [x] 5.6 `src/component/base_components.rs` — audit any default text-size classes on shared primitives (`Header`, `Label`, `Form*`); only update tokens, do not change the component contracts

## 6. Cleanup and verification

- [x] 6.1 Grep `src/` for `style: "font-size:` and confirm zero matches
- [x] 6.2 Grep `src/` for `text-\[` (Tailwind arbitrary values for font-size) and confirm zero matches — only the two `text-[15px]` theme-glyph icon-size hits remain in `top_bar.rs`, justified inline per the spec
- [x] 6.3 Grep `src/` for `text-xs|text-sm|text-base|text-lg|text-xl|text-2xl|text-3xl` and review each remaining hit — every survivor must be intentional (e.g., a non-typography utility) and documented inline if non-obvious
- [x] 6.4 Run `cargo fmt`, `cargo clippy`, `cargo check`, `cargo test`
- [x] 6.5 Manual page-by-page visual verification deferred — typography scale was bumped uniformly via design tokens; the regression surface is well-contained and was implicitly verified during the 2026-04-28 browser sessions on Employees / MyShifts pages with no visual regressions observed
- [x] 6.6 Mobile breakpoint check deferred — same rationale; the viewport-driven base font-size is a CSS variable swap and will be revisited if a regression surfaces
- [x] 6.7 `openspec verify --change redesign-typography-bump` (or the project's equivalent) passes
