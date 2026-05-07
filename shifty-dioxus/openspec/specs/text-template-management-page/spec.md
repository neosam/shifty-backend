# text-template-management-page Specification

## Purpose
Defines the presentation contract for the TextTemplateManagement page (`src/page/text_template_management.rs`): which design tokens drive surfaces and ink; how the form panel, table, and action buttons map to atom components (`Btn`, `TextInput`, `SelectInput`, `TextareaInput`); and the source-level guard test that prevents regression to legacy palette classes.

## Requirements

### Requirement: TextTemplateManagement surfaces use design-token palette

The page rendered by `src/page/text_template_management.rs` SHALL paint every container, the form panel, and the table using design-token color utilities (`bg-bg`, `bg-surface`, `bg-surface-alt`) instead of legacy palette classes. The "Add new / Edit template" form panel SHALL use `bg-surface-alt border border-border rounded-lg` (replacing `bg-gray-100`). The table SHALL use `bg-surface` with a `border border-border` (replacing `bg-white border border-gray-300`); the `<thead>` row SHALL use `bg-surface-alt` (replacing `bg-gray-50`); row dividers SHALL use `divide-border` (replacing `divide-gray-200`).

#### Scenario: Form panel uses surface-alt

- **WHEN** the user clicks "Add new" so `show_form` becomes true
- **THEN** the form panel SHALL carry the classes `bg-surface-alt`, `border`, and `border-border` and SHALL NOT contain the substring `bg-gray-100`

#### Scenario: Table uses token surfaces

- **WHEN** the page renders the templates table
- **THEN** the `<table>` element SHALL carry the classes `bg-surface` and `border-border` and SHALL NOT contain the substrings `bg-white` or `border-gray-300`
- **AND** the `<thead>` row SHALL carry the class `bg-surface-alt` and SHALL NOT contain the substring `bg-gray-50`
- **AND** the `<tbody>` row dividers SHALL be expressed via `divide-border` and SHALL NOT contain the substring `divide-gray-200`

### Requirement: TextTemplateManagement ink colors use token utilities

All text on the page SHALL be expressed using `text-ink`, `text-ink-soft`, or `text-ink-muted` utilities. The table header cells SHALL use `text-ink-muted` (for the uppercase column titles). Body cells SHALL use `text-ink`. The "No name" italic placeholder SHALL use `text-ink-muted`. The page SHALL NOT contain any class matching `text-gray-*`, `text-blue-*`, `text-green-*`, or `text-red-*`.

#### Scenario: Page source has no legacy gray ink

- **WHEN** the file `src/page/text_template_management.rs` is read at compile time
- **THEN** the source SHALL contain no occurrences of the substrings `text-gray-`, `text-blue-`, `text-green-`, `text-red-` outside of the test module

#### Scenario: Table header cells use ink-muted

- **WHEN** the templates table renders
- **THEN** each `<th>` SHALL carry the class `text-ink-muted` (or `text-ink-soft`) and SHALL NOT contain the substring `text-gray-500`

#### Scenario: "No name" placeholder uses ink-muted

- **WHEN** a template row renders for a template whose `name` field is `None`
- **THEN** the italic placeholder span SHALL carry the class `text-ink-muted` and SHALL NOT contain the substring `text-gray-400`

### Requirement: TextTemplateManagement action buttons use the Btn atom

The Add-new, Save, Cancel, Edit, and Delete buttons SHALL each be rendered via the `Btn` atom (`src/component/atoms/`). The Add-new and Save buttons SHALL use `BtnVariant::Primary`. The Cancel button SHALL use `BtnVariant::Secondary`. The Edit button SHALL use `BtnVariant::Primary` (or `Secondary` — chosen at implementation time, but consistent with the codebase's "edit row action" convention). The Delete button SHALL use `BtnVariant::Danger`. The page SHALL NOT contain raw `<button>` elements styled with `bg-blue-500`, `bg-green-500`, `bg-red-500`, or `bg-gray-500`.

#### Scenario: Add-new button is a Primary Btn

- **WHEN** `show_form` is false and the page renders the "Add new" button
- **THEN** the button SHALL be a `Btn` atom whose variant is `Primary`, and the rendered HTML SHALL NOT contain the substrings `bg-blue-500`, `bg-blue-700`

#### Scenario: Save button is a Primary Btn

- **WHEN** the form panel is visible and the page renders the "Save" button
- **THEN** the button SHALL be a `Btn` atom whose variant is `Primary`, and the rendered HTML SHALL NOT contain the substrings `bg-green-500`, `bg-green-700`

#### Scenario: Cancel button is a Secondary Btn

- **WHEN** the form panel is visible and the page renders the "Cancel" button
- **THEN** the button SHALL be a `Btn` atom whose variant is `Secondary`, and the rendered HTML SHALL NOT contain the substrings `bg-gray-500`, `bg-gray-700`

#### Scenario: Delete button is a Danger Btn

- **WHEN** the page renders a row with its "Delete" action
- **THEN** the button SHALL be a `Btn` atom whose variant is `Danger`, and the rendered HTML SHALL NOT contain the substrings `bg-red-500`, `bg-red-700`

### Requirement: TextTemplateManagement form inputs use token-aware focus styling

The Name `<input type="text">`, the Template-type `<select>`, the Template-engine `<select>`, and the Template-text `<textarea>` SHALL use either the `TextInput` / `Field` atoms or a bare element carrying the `form-input` class. The page SHALL NOT contain hard-coded focus or border classes targeting palette colors (`border-gray-300`, `focus:ring-blue-500`, `focus:border-blue-300`).

#### Scenario: Name input has token focus

- **WHEN** the form panel is visible
- **THEN** the Name input SHALL carry the class `form-input` (or be a `TextInput` atom) and SHALL NOT carry `border-gray-300`

#### Scenario: Template-type select has token focus

- **WHEN** the form panel is visible
- **THEN** the Template-type `<select>` SHALL carry the class `form-input` and SHALL NOT carry `border-gray-300`

#### Scenario: Template-engine select has token focus

- **WHEN** the form panel is visible
- **THEN** the Template-engine `<select>` SHALL carry the class `form-input` and SHALL NOT carry `border-gray-300`

#### Scenario: Template-text textarea has token focus

- **WHEN** the form panel is visible
- **THEN** the Template-text `<textarea>` SHALL carry the class `form-input` and SHALL NOT carry `border-gray-300`

### Requirement: TextTemplateManagement has a source-level legacy-class guard test

The file `src/page/text_template_management.rs` SHALL include a `#[cfg(test)] mod tests` module containing a `no_legacy_classes_in_source` test that reads the file via `include_str!`, slices off the test-module suffix, and asserts the prefix contains none of: `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `bg-red-`, `border-black`, `border-gray-`. The test SHALL fail compilation or assertion if any forbidden substring is reintroduced.

#### Scenario: Test catches reintroduced legacy class

- **WHEN** a contributor changes a class in the page source from `bg-surface-alt` back to `bg-gray-100` and runs `cargo test -p shifty-dioxus`
- **THEN** the `no_legacy_classes_in_source` test SHALL fail with a message identifying the forbidden class

#### Scenario: Test passes on the migrated source

- **WHEN** the page has been fully migrated to token utilities and `cargo test -p shifty-dioxus` is run
- **THEN** the `no_legacy_classes_in_source` test SHALL pass
