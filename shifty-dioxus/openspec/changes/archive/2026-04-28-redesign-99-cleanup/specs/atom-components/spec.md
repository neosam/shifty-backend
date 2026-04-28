## MODIFIED Requirements

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

## ADDED Requirements

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
