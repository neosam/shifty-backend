# dialog Specification

## Purpose
TBD - created by archiving change redesign-04-modal-variants. Update Purpose after archive.
## Requirements
### Requirement: `Field` form wrapper
The system SHALL provide a `Field` component that wraps a label, an input slot, and an optional hint or error message in a vertical layout. The label SHALL use uppercase 11 px text in `text-ink-soft`. Hint and error messages SHALL be mutually exclusive — when both are provided, error preempts hint.

#### Scenario: Label and child rendered
- **WHEN** a `Field { label: "Email", children: <FormTextInput …/> }` is rendered
- **THEN** the rendered output SHALL contain a `<label>` element with the text `"Email"` styled as uppercase 11 px ink-soft, followed by the input

#### Scenario: Hint visible when no error
- **WHEN** a `Field` is rendered with `hint: Some("Optional"), error: None`
- **THEN** the hint text SHALL render below the input in `text-[11px] text-ink-muted`

#### Scenario: Error preempts hint
- **WHEN** a `Field` is rendered with both `hint: Some("Optional")` and `error: Some("Required")`
- **THEN** only the error text SHALL render in `text-[11px] text-bad`; the hint SHALL NOT appear

#### Scenario: Span 2 in grid context
- **WHEN** a `Field` is rendered with `span: 2` inside a grid container
- **THEN** the rendered element SHALL include a style or class that resolves to `grid-column: span 2`

### Requirement: Form input atoms with accent focus ring
The system SHALL provide three input atoms — `TextInput`, `SelectInput`, `TextareaInput` — each using design tokens for borders/background/text and sharing a global `.form-input:focus` rule that produces an accent-colored focus ring (`border-color: var(--accent)` + `box-shadow: 0 0 0 3px var(--accent-soft)`). The `Form*` prefix that was used during the migration period SHALL no longer appear in the public API; the atoms SHALL be re-exported from `crate::component::form` and from `crate::component` under their unprefixed names.

#### Scenario: Default text input
- **WHEN** a `TextInput` is rendered with `value: "hello"`
- **THEN** the rendered `<input>` SHALL have height 34 px, padding 10 px horizontal, `border-border-strong`, `rounded-md`, `bg-surface`, `text-ink`, the class `form-input`, and value `"hello"`

#### Scenario: Focus ring on text input
- **WHEN** a user focuses a `TextInput`
- **THEN** the input border SHALL change to `var(--accent)` and a 3 px box-shadow in `var(--accent-soft)` SHALL appear

#### Scenario: Select input has custom arrow
- **WHEN** a `SelectInput` is rendered
- **THEN** the rendered `<select>` SHALL have `appearance: none` and a background-image rendering a chevron-down glyph aligned to the right

#### Scenario: Textarea expands vertically
- **WHEN** a `TextareaInput` is rendered
- **THEN** the rendered `<textarea>` SHALL allow vertical resize only and have a minimum height of 68 px

#### Scenario: Disabled state propagates
- **WHEN** any form input atom is rendered with `disabled: true`
- **THEN** the underlying HTML element SHALL be `disabled` and SHALL receive reduced visual contrast

#### Scenario: Public re-export under unprefixed names
- **WHEN** a consumer writes `use crate::component::form::TextInput;` or `use crate::component::TextInput;`
- **THEN** the import SHALL resolve to the token-based input atom (the same component that was named `FormTextInput` prior to this cleanup)
- **AND** no public symbol named `FormTextInput`, `FormSelectInput`, or `FormTextareaInput` SHALL be exported from `crate::component::form` or `crate::component`

### Requirement: All Dialog consumers use the token-based component
After the cleanup, every dialog mount in the application SHALL use the `Dialog` component (or its dedicated wrappers `ContractModal` / `ExtraHoursModal`). The legacy `Modal` component SHALL no longer exist, and no source file SHALL import `Modal` from `crate::component`.

#### Scenario: No legacy `Modal` consumer remains
- **WHEN** `src/` is searched (recursive, case-sensitive) for `Modal {` (RSX-style mount) or `use crate::component::Modal`
- **THEN** zero matches SHALL be found, except inside doc-comments referring to the historical name

#### Scenario: `MyEmployeeDetails` uses `Dialog`
- **WHEN** the user opens the contract detail dialog from `src/page/my_employee_details.rs`
- **THEN** the rendering pipeline SHALL produce a `Dialog` panel (with `role="dialog"` on the panel and `role="presentation"` on the backdrop)
- **AND** SHALL NOT mount the legacy `Modal` component

