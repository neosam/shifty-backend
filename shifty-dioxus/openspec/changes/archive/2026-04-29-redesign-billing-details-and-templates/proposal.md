## Why

`BillingPeriodDetails` (`src/page/billing_period_details.rs`) and `TextTemplateManagement` (`src/page/text_template_management.rs`) are the last two pages still painted in legacy Tailwind palette classes (`bg-white`, `bg-gray-*`, `text-gray-*`, `bg-blue-500`, `bg-green-100/text-green-800`, `bg-red-100/text-red-800`, hard-coded `border-gray-300`). They render correctly in light mode but break in dark mode: white card surfaces against a near-black `--bg`, illegible muted text on dark surfaces, and status pills that lose contrast. Every other primary page already migrated to the design-token palette during the redesign series, so these two pages are now the only places where dark mode visibly fails.

## What Changes

- Migrate `src/page/billing_period_details.rs` from legacy palette classes to the design-token palette (`bg-surface`, `bg-surface-alt`, `text-ink`, `text-ink-muted`, `border-border`, `bg-good-soft`/`text-good`, `bg-bad-soft`/`text-bad`, `bg-accent-soft`/`text-accent`).
- Migrate `src/page/text_template_management.rs` to the same token set.
- Replace ad-hoc `<button>` elements styled with `bg-blue-500`/`bg-green-500`/`bg-red-500`/`bg-gray-500` with the existing `Btn` atom in `Primary`, `Secondary`, and `Danger` variants. The semantically-green Save button becomes `Primary` (we do not introduce a new "good" button variant just to retain the green hue).
- Replace bare `<input type="text">`, `<input type="checkbox">`, `<select>`, and `<textarea>` styling on these two pages with the existing form atoms (`TextInput`, `Field`, `.form-input` focus class) so focus rings come from `--accent` / `--accent-soft` instead of hard-coded blue.
- The "Generated Report" `<pre>` block uses `bg-surface-2` (slightly recessed code-block look) with `text-ink`.
- Add per-file source guard tests modelled on `src/page/billing_periods.rs::tests::no_legacy_classes_in_source` to both pages so backsliding to legacy palette classes is caught at `cargo test` time.

This is purely a presentation change. No behavior, routing, data loading, action surface, or API call is modified. No translation keys are added or removed.

## Capabilities

### New Capabilities
- `billing-period-details-page`: presentation contract for the `BillingPeriodDetails` page — surfaces, ink colors, status pills, action buttons, form atoms, and the legacy-class guard test.
- `text-template-management-page`: presentation contract for the `TextTemplateManagement` page — surfaces, ink colors, action buttons, form atoms, table styling, and the legacy-class guard test.

### Modified Capabilities
<!-- None. The existing `billing-periods-page`, `design-tokens`, `atom-components`, and `dialog` specs are not changed. -->

## Impact

- Affected files: `src/page/billing_period_details.rs`, `src/page/text_template_management.rs`.
- No backend changes. No `rest-types/` changes. No routing changes. No new translation keys.
- No new Tailwind safelist entries expected — the token-based class names are all statically present in the migrated source.
- Visual regression risk on the two pages in light mode (cards lose `shadow`, status-pill hue shifts from saturated `green-100` to muted `good-soft`). The redesign-series precedent treats this shift as intended.
