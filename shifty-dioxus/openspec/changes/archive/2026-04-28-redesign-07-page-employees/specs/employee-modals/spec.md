## ADDED Requirements

### Requirement: ContractModal wraps the contract form in a Dialog with token-based footer
The `ContractModal` component SHALL render a `Dialog` (variant `DialogVariant::Auto`, width 520) whose body contains the contract form fields and whose footer contains a secondary-variant `Btn` labeled by `Key::Cancel` and (when `form_type != ReadOnly`) a primary-variant `Btn` labeled by `Key::Save`. When `props.open` is `false`, the modal SHALL render nothing. When `props.open` is `true`, the modal SHALL render the dialog title using `Key::AddWorkDetailsFormTitle`. The form fields inside the body SHALL use `Field` wrappers and the `Form*` input atoms (`FormTextInput`, `FormSelectInput`, `FormCheckbox`) instead of the legacy `base_components` (`DateInput`, `FloatInput`, `IntegerInput`, `Checkbox`, `FormPair`, `Form`).

#### Scenario: Closed modal renders nothing
- **WHEN** `ContractModal` is rendered with `open = false`
- **THEN** the rendered HTML SHALL NOT contain a `Dialog` panel and SHALL NOT contain any of the contract form fields

#### Scenario: Open modal renders Dialog title and footer
- **WHEN** `ContractModal` is rendered with `open = true` and `form_type = New`
- **THEN** the rendered HTML SHALL contain `i18n.t(Key::AddWorkDetailsFormTitle)` text inside an element with id `shifty-dialog-title`, AND the footer SHALL contain a Cancel button and a Save button

#### Scenario: ReadOnly form omits the Save button
- **WHEN** `ContractModal` is rendered with `open = true` and `form_type = ReadOnly`
- **THEN** the footer SHALL contain a Cancel button and SHALL NOT contain a Save button

#### Scenario: Form fields use Field and Form* atoms, not legacy components
- **WHEN** the contract modal body is rendered
- **THEN** the rendered HTML SHALL contain at least one `<label>` element produced by `Field` carrying classes resolving to `text-[11px]`, `uppercase`, and `font-semibold`, AND it SHALL contain at least one input element with the class `form-input`

### Requirement: ExtraHoursModal wraps the extra-hours form in a Dialog with token-based footer
The `ExtraHoursModal` component SHALL render a `Dialog` (variant `DialogVariant::Auto`, width 460) whose body contains the extra-hours form fields and whose footer contains a secondary-variant `Btn` labeled by `Key::Cancel` and a primary-variant `Btn` labeled by `Key::Submit`. The dialog title SHALL use `Key::AddExtraHoursFormTitle`. Form fields SHALL use `Field` wrappers and the `Form*` input atoms (`FormTextInput`, `FormSelectInput`) instead of legacy components. The category select SHALL include all standard categories (extra work, volunteer work, holiday, sick leave, vacation days, unavailable, unpaid leave, vacation), preceded by a divider when custom categories follow, and followed by one option per loaded `CustomExtraHoursDefinition`. The form SHALL render distinct field sets for the vacation-days mode (two `type=date` inputs for `from`/`to`) versus the non-vacation-days mode (an amount input plus a `type=datetime-local` "when" input).

#### Scenario: Closed modal renders nothing
- **WHEN** `ExtraHoursModal` is rendered with `open = false`
- **THEN** the rendered HTML SHALL NOT contain a `Dialog` panel and SHALL NOT contain any of the extra-hours form fields

#### Scenario: Open modal renders Dialog title and footer
- **WHEN** `ExtraHoursModal` is rendered with `open = true`
- **THEN** the rendered HTML SHALL contain `i18n.t(Key::AddExtraHoursFormTitle)` text, AND the footer SHALL contain a Cancel button and a Submit button

#### Scenario: Category select includes all standard categories
- **WHEN** `ExtraHoursModal` is rendered open
- **THEN** the rendered HTML SHALL contain `<option>` elements with values `extra_work`, `volunteer_work`, `holiday`, `sick_leave`, `vacation_days`, `unavailable`, `unpaid_leave`, and `vacation`

#### Scenario: Vacation-days mode swaps amount/when for from/to
- **WHEN** the user selects category `vacation_days`
- **THEN** the form body SHALL render two `type=date` inputs (`from` and `to`) and SHALL NOT render the `type=number` amount input or the `type=datetime-local` when input

#### Scenario: Submit dispatches the existing API call and triggers `on_saved`
- **WHEN** the user clicks the Submit button with valid input
- **THEN** the modal SHALL invoke `api::add_extra_hour` (or `api::add_vacation` for vacation-days mode) with the captured values, AND on success it SHALL call the `on_saved` event handler

### Requirement: New FormCheckbox atom under src/component/form/
The system SHALL provide a `FormCheckbox` atom in `src/component/form/checkbox.rs`, exported from `src/component/form/mod.rs`. The atom SHALL render an `<input type="checkbox">` paired with a label slot. Props: `value: bool`, `disabled: bool`, `on_change: Option<EventHandler<bool>>`, `label: Element`. The atom SHALL use design-token classes (no hardcoded colors). The atom SHALL coexist with the legacy `Checkbox` from `base_components` until the cleanup change.

#### Scenario: Renders an input element with type=checkbox
- **WHEN** `FormCheckbox` is rendered with any props
- **THEN** the rendered HTML SHALL contain an `<input>` element with `type="checkbox"`

#### Scenario: Reflects the value prop
- **WHEN** `FormCheckbox` is rendered with `value = true`
- **THEN** the rendered `<input>` SHALL include the `checked` attribute

#### Scenario: Disabled propagates
- **WHEN** `FormCheckbox` is rendered with `disabled = true`
- **THEN** the rendered `<input>` SHALL include the `disabled` attribute

#### Scenario: on_change fires with the new boolean value
- **WHEN** the user toggles the checkbox from unchecked to checked
- **THEN** the `on_change` handler SHALL be called with `true`

#### Scenario: Label slot renders alongside the input
- **WHEN** `FormCheckbox` is rendered with `label = rsx! { "Monday" }`
- **THEN** the rendered HTML SHALL contain the text `Monday` adjacent to the input element

### Requirement: New i18n keys for employees page actions
The system SHALL provide four new i18n keys covering the redesigned employees page actions: `Key::SearchPlaceholder`, `Key::OtherHours`, `Key::More`, and `Key::BackToList`. Each key SHALL have translations in all three supported locales (English, German, Czech).

#### Scenario: All four keys translate in all locales
- **WHEN** any locale is active
- **THEN** `i18n.t(Key::SearchPlaceholder)`, `i18n.t(Key::OtherHours)`, `i18n.t(Key::More)`, and `i18n.t(Key::BackToList)` SHALL each return a non-empty locale-specific string

#### Scenario: German values match the reference design
- **WHEN** the German locale is active
- **THEN** `i18n.t(Key::OtherHours)` SHALL equal `Sonstige Stunden`, `i18n.t(Key::More)` SHALL equal `Mehr`, and `i18n.t(Key::BackToList)` SHALL equal `Zurück`
