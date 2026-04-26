## ADDED Requirements

### Requirement: Dialog with four layout variants
The system SHALL provide a `Dialog` component supporting four layout variants: `Center`, `Sheet`, `Bottom`, and `Auto`. The `Auto` variant SHALL resolve at runtime to `Bottom` on viewports below 720 px and `Center` otherwise, and SHALL respond to viewport size changes while open.

#### Scenario: Center variant on desktop
- **WHEN** a `Dialog` is rendered with `variant: Auto` and the viewport is 1024 px wide
- **THEN** the dialog SHALL render as a centered panel with width capped at the `width` prop value

#### Scenario: Bottom variant on mobile
- **WHEN** a `Dialog` is rendered with `variant: Auto` and the viewport is 600 px wide
- **THEN** the dialog SHALL render as a bottom sheet with full width

#### Scenario: Live response to viewport resize
- **WHEN** a `Dialog` with `variant: Auto` is open and the viewport resizes from 1024 px to 600 px
- **THEN** the dialog SHALL switch from `Center` layout to `Bottom` layout without closing

#### Scenario: Sheet variant slides from right
- **WHEN** a `Dialog` is rendered with `variant: Sheet`
- **THEN** the panel SHALL be aligned to the right edge of the viewport, full height, with the slide-from-right animation

### Requirement: Dialog dismissal paths
The Dialog SHALL invoke its `on_close` callback when any of the following dismissal actions occur: backdrop click, ESC key press, close-X button click, or footer Cancel button click.

#### Scenario: Backdrop click dismisses
- **WHEN** the user clicks on the backdrop area of an open Dialog
- **THEN** the `on_close` callback SHALL be invoked

#### Scenario: Panel click does not dismiss
- **WHEN** the user clicks on the panel content of an open Dialog
- **THEN** the `on_close` callback SHALL NOT be invoked

#### Scenario: ESC key dismisses
- **WHEN** an open Dialog has focus context and the user presses the Escape key
- **THEN** the `on_close` callback SHALL be invoked

#### Scenario: Close-X button dismisses
- **WHEN** the user clicks the close-X button in the dialog header
- **THEN** the `on_close` callback SHALL be invoked

### Requirement: Body scroll lock
While a Dialog is open, the document body SHALL have its `overflow` style set to `hidden` to prevent scroll-through. When the Dialog closes, the prior `overflow` value SHALL be restored.

#### Scenario: Scroll lock applied on open
- **WHEN** a Dialog is rendered with `open: true`
- **THEN** `document.body.style.overflow` SHALL be set to `"hidden"`

#### Scenario: Prior scroll value restored on close
- **WHEN** the document body had `overflow: "auto"` before the Dialog opened, and the Dialog then closes
- **THEN** `document.body.style.overflow` SHALL be `"auto"` after close

### Requirement: Dialog header and footer
The Dialog SHALL display a header containing a required title (16 px / 700 / tracking -0.01em), an optional subtitle (12 px / `text-ink-muted`), and a close-X button. The Dialog SHALL accept an optional footer slot that renders sticky at the bottom of the panel.

#### Scenario: Title and close-X always present
- **WHEN** any Dialog is rendered
- **THEN** the header SHALL contain the provided `title` text and a close-X button

#### Scenario: Subtitle conditionally rendered
- **WHEN** a Dialog is rendered with `subtitle: Some("…")`
- **THEN** the subtitle text SHALL appear below the title in `text-ink-muted` 12 px

#### Scenario: Footer slot rendered when provided
- **WHEN** a Dialog is rendered with `footer: Some(element)`
- **THEN** the footer content SHALL appear at the bottom of the panel inside a row with `border-t border-border` and `bg-surface-alt`

### Requirement: Bottom variant has visual drag-handle
When rendered as `Bottom` (or `Auto` resolved to `Bottom`), the Dialog SHALL display a small visual drag-handle bar at the top of the panel as an affordance hint, even though no actual drag-to-dismiss interaction is implemented.

#### Scenario: Drag-handle in Bottom variant
- **WHEN** a Dialog is rendered with `variant: Bottom`
- **THEN** a 36×4 px pill-shaped element SHALL appear at the top of the panel using `bg-border-strong`

#### Scenario: No drag-handle in other variants
- **WHEN** a Dialog is rendered with `variant: Center` or `variant: Sheet`
- **THEN** no drag-handle element SHALL appear

### Requirement: `use_media_query` hook
The system SHALL provide a `use_media_query(query: &str) -> Signal<bool>` hook that returns a signal reflecting whether the given mediaquery matches, and updates live when the match changes.

#### Scenario: Initial match value
- **WHEN** `use_media_query("(max-width: 720px)")` is called on a viewport of width 600 px
- **THEN** the returned signal SHALL initially read `true`

#### Scenario: Live update on viewport resize
- **WHEN** the viewport resizes from 600 px to 1024 px while a `use_media_query("(max-width: 720px)")` signal is observed
- **THEN** the signal value SHALL transition from `true` to `false`

---

## ADDED Requirements

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
The system SHALL provide three input atoms — `FormTextInput`, `FormSelectInput`, `FormTextareaInput` — each using design tokens for borders/background/text and sharing a global `.form-input:focus` rule that produces an accent-colored focus ring (`border-color: var(--accent)` + `box-shadow: 0 0 0 3px var(--accent-soft)`).

#### Scenario: Default text input
- **WHEN** a `FormTextInput` is rendered with `value: "hello"`
- **THEN** the rendered `<input>` SHALL have height 34 px, padding 10 px horizontal, `border-border-strong`, `rounded-md`, `bg-surface`, `text-ink`, the class `form-input`, and value `"hello"`

#### Scenario: Focus ring on text input
- **WHEN** a user focuses a `FormTextInput`
- **THEN** the input border SHALL change to `var(--accent)` and a 3 px box-shadow in `var(--accent-soft)` SHALL appear

#### Scenario: Select input has custom arrow
- **WHEN** a `FormSelectInput` is rendered
- **THEN** the rendered `<select>` SHALL have `appearance: none` and a background-image rendering a chevron-down glyph aligned to the right

#### Scenario: Textarea expands vertically
- **WHEN** a `FormTextareaInput` is rendered
- **THEN** the rendered `<textarea>` SHALL allow vertical resize only and have a minimum height of 68 px

#### Scenario: Disabled state propagates
- **WHEN** any form input atom is rendered with `disabled: true`
- **THEN** the underlying HTML element SHALL be `disabled` and SHALL receive reduced visual contrast
