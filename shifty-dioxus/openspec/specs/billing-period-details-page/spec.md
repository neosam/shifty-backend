# billing-period-details-page Specification

## Purpose
Defines the presentation contract for the BillingPeriodDetails page (`src/page/billing_period_details.rs`): which design tokens drive surfaces, ink, and status pills; how action buttons and form inputs map to atom components (`Btn`, `TextInput`, `SelectInput`); and the source-level guard test that prevents regression to legacy palette classes.

## Requirements

### Requirement: BillingPeriodDetails surfaces use design-token palette

The page rendered by `src/page/billing_period_details.rs` SHALL paint every container, card, and recessed surface using design-token color utilities (`bg-bg`, `bg-surface`, `bg-surface-alt`, `bg-surface-2`) instead of legacy palette classes (`bg-white`, `bg-gray-50`, `bg-gray-100`, etc.). Top-level cards (Basic Information, Custom Reports, Sales Persons sections) SHALL use `bg-surface border border-border rounded-lg` with no `shadow` utility. Recessed surfaces inside a card (the value tiles in the Sales Persons section) SHALL use `bg-surface-alt`. The Custom Report `<pre>` output container SHALL use `bg-surface-2` with `border border-border`.

#### Scenario: Cards render with token surface and border, not shadow

- **WHEN** the page is rendered with a billing period loaded
- **THEN** the rendered HTML for the Basic Information, Custom Reports, and Sales Persons cards SHALL contain the classes `bg-surface`, `border`, and `border-border` and SHALL NOT contain the substrings `bg-white`, `shadow`

#### Scenario: Value tiles use surface-alt

- **WHEN** the page renders a sales-person `values` block with at least one entry
- **THEN** each value tile SHALL carry the class `bg-surface-alt` and SHALL NOT contain the substring `bg-gray-50`

#### Scenario: Generated report uses surface-2

- **WHEN** the user generates a custom report and the result is rendered
- **THEN** the `<pre>` container SHALL carry the classes `bg-surface-2` and `border-border` and SHALL NOT contain the substring `bg-gray-50`

### Requirement: BillingPeriodDetails ink colors use token utilities

All text on the page SHALL be expressed using `text-ink`, `text-ink-soft`, or `text-ink-muted` utilities. Heading text and primary body content SHALL use `text-ink`. Field labels SHALL use `text-ink-soft`. Tertiary metadata, placeholder, and "italic" empty-state messages SHALL use `text-ink-muted`. The page SHALL NOT contain any class matching `text-gray-*`, `text-blue-*`, `text-green-*`, or `text-red-*` (status pills excepted, which use `text-good`/`text-bad` per the next requirement).

#### Scenario: Page source has no legacy gray ink

- **WHEN** the file `src/page/billing_period_details.rs` is read at compile time
- **THEN** the source SHALL contain no occurrences of the substrings `text-gray-`, `text-blue-`, `text-green-`, `text-red-` outside of the test module

#### Scenario: Field labels use ink-soft

- **WHEN** the Basic Information card renders
- **THEN** each `<label>` element SHALL carry the class `text-ink-soft` (or `text-ink`)

### Requirement: BillingPeriodDetails status pills use semantic soft tokens

The "Active" / "Deleted" status pills (rendered next to the period date range and inside each sales-person row) SHALL use `bg-good-soft text-good` for the active state and `bg-bad-soft text-bad` for the deleted state. They SHALL NOT use `bg-green-100 text-green-800` or `bg-red-100 text-red-800`.

#### Scenario: Active pill carries good-soft tokens

- **WHEN** the page renders a billing period whose `deleted_at` is `None`
- **THEN** the rendered "Active" pill SHALL carry the classes `bg-good-soft` and `text-good`

#### Scenario: Deleted pill carries bad-soft tokens

- **WHEN** the page renders a billing period whose `deleted_at` is `Some(...)`
- **THEN** the rendered "Deleted" pill SHALL carry the classes `bg-bad-soft` and `text-bad`

### Requirement: BillingPeriodDetails action buttons use the Btn atom

The "Generate Report" and "Copy to Clipboard" buttons SHALL be rendered via the `Btn` atom (`src/component/atoms/`) with `variant: BtnVariant::Primary`. They SHALL NOT be raw `<button>` elements styled with `bg-blue-500`, `bg-green-500`, or `bg-gray-400`. The disabled-while-generating state SHALL be expressed via `Btn`'s disabled mechanics (e.g. its `disabled` prop or the equivalent already used elsewhere in the codebase) rather than via class swap.

#### Scenario: Generate Report renders as Primary Btn

- **WHEN** the page renders the Custom Reports section with a template selected
- **THEN** the Generate-Report button SHALL be a `Btn` atom whose variant is `Primary`, and the rendered HTML SHALL NOT contain the substrings `bg-blue-500`, `bg-blue-700`, `bg-gray-400`

#### Scenario: Copy to Clipboard renders as Primary Btn

- **WHEN** the page renders a generated report
- **THEN** the Copy-to-Clipboard button SHALL be a `Btn` atom whose variant is `Primary`, and the rendered HTML SHALL NOT contain the substrings `bg-green-500`, `bg-green-700`

### Requirement: BillingPeriodDetails form inputs use token-aware focus styling

The filter `<input type="text">` for sales-person name search and the template `<select>` SHALL use either the `TextInput` / `Field` atoms or a bare element carrying the `form-input` class. The page SHALL NOT contain hard-coded focus or border classes targeting palette colors (`focus:ring-blue-500`, `focus:border-blue-300`, `focus:ring-blue-200`, `border-gray-300`).

#### Scenario: Filter text input has token focus

- **WHEN** the Sales Persons filter text input is rendered
- **THEN** the input SHALL carry the class `form-input` (or be a `TextInput` atom that does) and SHALL NOT carry `focus:ring-blue-500`, `focus:border-blue-300`, `focus:ring-blue-200`, `border-gray-300`

#### Scenario: Template selector has token focus

- **WHEN** the template `<select>` is rendered
- **THEN** the element SHALL carry the class `form-input` and SHALL NOT carry `border-gray-300`

### Requirement: BillingPeriodDetails loading spinner uses accent token

The loading spinner shown while `selected_billing_period` is `None` SHALL use `border-accent` (or equivalent token) instead of `border-blue-600`. The "Loading…" text SHALL use `text-ink-muted`.

#### Scenario: Spinner uses accent border

- **WHEN** the page is rendered with `selected_billing_period == None`
- **THEN** the spinner element SHALL carry a token-based border color class (`border-accent` or equivalent) and SHALL NOT contain the substring `border-blue-600`

#### Scenario: Loading text uses ink-muted

- **WHEN** the page is rendered with `selected_billing_period == None`
- **THEN** the "Loading…" `<p>` element SHALL carry the class `text-ink-muted` and SHALL NOT contain the substring `text-gray-500`

### Requirement: BillingPeriodDetails has a source-level legacy-class guard test

The file `src/page/billing_period_details.rs` SHALL include a `#[cfg(test)] mod tests` module containing a `no_legacy_classes_in_source` test that reads the file via `include_str!`, slices off the test-module suffix, and asserts the prefix contains none of: `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `bg-red-`, `border-black`, `border-gray-`. The test SHALL fail compilation or assertion if any forbidden substring is reintroduced.

#### Scenario: Test catches reintroduced legacy class

- **WHEN** a contributor changes a class in the page source from `bg-surface` back to `bg-white` and runs `cargo test -p shifty-dioxus`
- **THEN** the `no_legacy_classes_in_source` test SHALL fail with a message identifying the forbidden class

#### Scenario: Test passes on the migrated source

- **WHEN** the page has been fully migrated to token utilities and `cargo test -p shifty-dioxus` is run
- **THEN** the `no_legacy_classes_in_source` test SHALL pass
