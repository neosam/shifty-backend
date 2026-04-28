# atom-components Specification

## Purpose
TBD - created by archiving change redesign-02-atom-components. Update Purpose after archive.
## Requirements
### Requirement: `Btn` component with four variants
The system SHALL provide a `Btn` component supporting four visual variants (`Primary`, `Secondary`, `Ghost`, `Danger`), an optional mono icon prefix, and a disabled state. Variants SHALL use design tokens from the design-tokens capability.

#### Scenario: Primary variant renders accent background
- **WHEN** a `Btn` is rendered with `variant: Primary`
- **THEN** the rendered element SHALL include classes that resolve to `background: var(--accent)` and `color: var(--accent-ink)`

#### Scenario: Secondary variant renders surface background
- **WHEN** a `Btn` is rendered with `variant: Secondary`
- **THEN** the rendered element SHALL include classes that resolve to `background: var(--surface)`, `color: var(--ink)`, and `border: 1px solid var(--border-strong)`

#### Scenario: Ghost variant renders transparent
- **WHEN** a `Btn` is rendered with `variant: Ghost`
- **THEN** the rendered element SHALL have a transparent background and `color: var(--ink-soft)`

#### Scenario: Danger variant renders bad text on surface
- **WHEN** a `Btn` is rendered with `variant: Danger`
- **THEN** the rendered element SHALL include classes that resolve to `background: var(--surface)`, `color: var(--bad)`, and `border: 1px solid var(--bad)`

#### Scenario: Disabled state visible and click suppressed
- **WHEN** a `Btn` is rendered with `disabled: true`
- **THEN** the rendered element SHALL include `opacity-50 cursor-not-allowed` classes AND any `on_click` handler SHALL not be invoked when the button is clicked

#### Scenario: Icon prefix renders in mono font
- **WHEN** a `Btn` is rendered with a non-`None` `icon` prop
- **THEN** the rendered element SHALL include a `<span class="font-mono">` containing the icon glyph before the children

### Requirement: `PersonChip` enforces dark text on pastel backgrounds
The `PersonChip` component SHALL render a name pill whose text color is forced to dark ink (`--chip-ink: #0e1117`) regardless of the active theme, when a pastel background color is provided. This invariant SHALL be enforced by a CSS rule with `!important` so it cannot be accidentally overridden by theme rules.

#### Scenario: Chip with pastel color
- **WHEN** a `PersonChip` is rendered with a non-`None` `color` (e.g. `"#dbe0ff"`)
- **THEN** the rendered element SHALL have inline `background-color: #dbe0ff` AND the class `person-pill` AND the resulting computed text color SHALL be `var(--chip-ink)` in both light and dark themes

#### Scenario: Chip without color uses dashed border
- **WHEN** a `PersonChip` is rendered with `color: None`
- **THEN** the rendered element SHALL have a transparent background with a dashed border in `var(--border-strong)`, text in `var(--ink-soft)`, AND SHALL NOT carry the `person-pill` class

#### Scenario: Bold highlight for current-edit person
- **WHEN** a `PersonChip` is rendered with `bold: true`
- **THEN** the rendered element SHALL include a `font-semibold` class

#### Scenario: No initials anywhere
- **WHEN** any `PersonChip` is rendered
- **THEN** the rendered output SHALL contain only the `name` text, no abbreviation, no avatar circle, no two-letter initials

### Requirement: `TupleRow` provides label-value layout with optional description
The `TupleRow` component SHALL render a horizontal label/value pair with a 1 px bottom border, an optional dim variant for secondary fields, and an optional description slot rendered below the row.

#### Scenario: Default tuple row
- **WHEN** a `TupleRow` is rendered with `label` and `value`
- **THEN** the row SHALL display label-left and value-right with `border-b border-border` and `text-[13px]`

#### Scenario: Dim variant
- **WHEN** a `TupleRow` is rendered with `dim: true`
- **THEN** both label and value SHALL render in `text-ink-muted`

#### Scenario: Description slot
- **WHEN** a `TupleRow` is rendered with a non-`None` `description`
- **THEN** the description SHALL appear below the label/value row in `text-xs text-ink-muted`

### Requirement: `NavBtn` is a 28×28 square icon button
The `NavBtn` component SHALL render a square 28×28 button with a mono glyph, used for prev/next navigation in week and year contexts.

#### Scenario: Default render
- **WHEN** a `NavBtn` is rendered with `glyph: "‹"`
- **THEN** the rendered element SHALL be 28×28 px with classes resolving to `border: 1px solid var(--border-strong)`, `border-radius: var(--r-md)`, `font-family: var(--font-mono)`, and the glyph centered

#### Scenario: Disabled state
- **WHEN** a `NavBtn` is rendered with `disabled: true`
- **THEN** the rendered element SHALL include `opacity-50 cursor-not-allowed` AND any `on_click` SHALL be suppressed

#### Scenario: Optional aria-label
- **WHEN** a `NavBtn` is rendered with `aria_label: Some("Previous week")`
- **THEN** the rendered element SHALL carry an `aria-label="Previous week"` attribute
- **WHEN** `aria_label: None`
- **THEN** the rendered element SHALL NOT carry an `aria-label` attribute

### Requirement: Atoms live in a dedicated module
All four atom components SHALL be implemented in a dedicated `src/component/atoms/` module. The module SHALL re-export the public types from `src/component/mod.rs` so that consumers can import from `crate::component::{Btn, PersonChip, TupleRow, NavBtn}`. After the cleanup, no parallel legacy implementations of these atoms SHALL exist in `src/component/base_components.rs` or elsewhere.

#### Scenario: Module re-export
- **WHEN** a consumer writes `use crate::component::Btn;`
- **THEN** the import SHALL resolve to `crate::component::atoms::btn::Btn`

#### Scenario: No legacy `Button` parallel
- **WHEN** the project source is compiled
- **THEN** `src/component/base_components.rs` SHALL NOT define a `Button` component
- **AND** `src/component/mod.rs` SHALL NOT re-export `Button`

#### Scenario: No legacy `TextInput` parallel after rename
- **WHEN** the project source is compiled
- **THEN** the only `TextInput` symbol available via `crate::component::TextInput` SHALL be the renamed token-based input atom (formerly `FormTextInput`), not the legacy plain-styled input from `base_components.rs`

### Requirement: Legacy parallel atom components are removed
Legacy components in `src/component/base_components.rs` and `src/component/modal.rs` that were superseded by the redesigned atoms SHALL be deleted once their consumers are migrated. The set of legacy components removed by this cleanup SHALL be exactly: `Button` (replaced by `Btn`), `TextInput` (replaced by the renamed `TextInput` formerly known as `FormTextInput`), and `Modal` (replaced by `Dialog`).

#### Scenario: Legacy `Button` is deleted
- **WHEN** the project source is searched for `pub fn Button` or `struct ButtonProps`
- **THEN** zero matches SHALL be found in `src/`

#### Scenario: Legacy `TextInput` is deleted
- **WHEN** the project source is searched for `pub fn TextInput` or `struct TextInputProps` in `src/component/base_components.rs`
- **THEN** zero matches SHALL be found in that file

#### Scenario: Legacy `Modal` file is deleted
- **WHEN** the project tree is inspected
- **THEN** `src/component/modal.rs` SHALL NOT exist
- **AND** `src/component/mod.rs` SHALL NOT contain `pub mod modal;` or `pub use modal::Modal;`

#### Scenario: Other `base_components.rs` items remain
- **WHEN** the project source is inspected
- **THEN** `Header`, `Label`, `Form`, `FormPair`, `FormItem`, `FormGroup`, `Checkbox`, `DateInput`, `IntegerInput`, `TimeInput`, `FloatInput`, `Select`, `SimpleSelect`, and `Option` SHALL still be defined in `src/component/base_components.rs`
- **AND** they SHALL still be re-exported from `src/component/mod.rs`

### Requirement: Custom legacy Tailwind colors are removed
The custom color names `missingColor` and `blockedColor` SHALL be removed from `tailwind.config.js`, both from `theme.extend.colors` and from `safelist`. Any remaining call site that references `bg-missingColor` or `bg-blockedColor` SHALL be migrated to the token-based equivalents (`bg-warn-soft` and `bg-bad-soft`, respectively) before the colors are removed.

#### Scenario: Tailwind config no longer defines `missingColor`
- **WHEN** `tailwind.config.js` is read
- **THEN** the `theme.extend.colors` object SHALL NOT contain a `missingColor` key
- **AND** SHALL NOT contain a `blockedColor` key

#### Scenario: Tailwind safelist no longer mentions custom legacy colors
- **WHEN** `tailwind.config.js` is read
- **THEN** the `safelist` array SHALL NOT contain `"bg-missingColor"` or `"bg-blockedColor"`

#### Scenario: No source-code reference remains
- **WHEN** `src/` is searched (case-sensitive, recursive) for `missingColor` or `blockedColor`
- **THEN** zero matches SHALL be found

