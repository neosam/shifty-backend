## Context

`shifty-dioxus` has a small base-component library in `src/component/base_components.rs`: `Button`, `Header`, `Label`, `Form*`, `Checkbox`, several typed inputs (`TextInput`, `DateInput`, `IntegerInput`, `TimeInput`, `FloatInput`), `Select`/`SimpleSelect`. Most of these are styled with ad-hoc Tailwind classes (`border-2 border-gray-200 p-2`) that predate the new design system.

Two of the four atoms required by the redesign have an existing-but-different equivalent:

- **`Button`** exists with `primary: bool` (two variants); the redesign needs four variants and different padding/radius/typography. APIs are not compatible.
- **PersonChip** does not exist as a component; person pills are rendered inline inside `week_view.rs::ColumnViewSlot` (line ~128) as a `<p>` tag with a runtime `style: "background-color: {item.background_color}"` and Tailwind classes for shape. Other call sites (e.g. `day_aggregate_view.rs`, employee list) use similar inline patterns.

The remaining two atoms (`TupleRow`, `NavBtn`) have no equivalent today.

The redesign master plan defers actual migration of call sites to per-page changes (`05`–`09`). This change therefore is purely additive: it introduces the four new atoms without altering or replacing any existing render paths.

## Goals / Non-Goals

**Goals:**
- Provide a clear, idiomatic Rust API for each atom
- Use design tokens from `01` exclusively — no hardcoded colors or radii
- Enforce the **no-initials** and **dark-text-on-pastel** invariants for `PersonChip` at the CSS level so they cannot drift later
- Keep the change purely additive: old `Button`, inline person pills, and legacy patterns remain untouched

**Non-Goals:**
- Migrating any existing call site (per-page in `05`–`09`)
- Removing the legacy `Button` (later cleanup change)
- Form-related atoms (`Field`, inputs) — change `04`
- A11y refinements beyond the basics (focus rings via the accent token, semantic HTML)

## Decisions

### 1. New module `src/component/atoms/` rather than appending to `base_components.rs`

`base_components.rs` is already 390 lines and mixes form widgets, layout helpers, and a button. Putting the new atoms there would entrench the legacy with the new and make the eventual cleanup harder. A separate `atoms/` module makes the boundary explicit and lets each atom live in its own file with focused tests.

```
src/component/
├── atoms/
│   ├── mod.rs           // pub use {btn, person_chip, tuple_row, nav_btn}::*
│   ├── btn.rs           // Btn, BtnVariant
│   ├── person_chip.rs   // PersonChip
│   ├── tuple_row.rs     // TupleRow
│   └── nav_btn.rs       // NavBtn
├── base_components.rs   // legacy, untouched
└── mod.rs               // pub use atoms::*
```

### 2. `Btn` API: enum variant, owned-string children, no breaking-change-spillover

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum BtnVariant { Primary, Secondary, Ghost, Danger }

#[derive(Props, Clone, PartialEq)]
pub struct BtnProps {
    pub children: Element,
    #[props(default = BtnVariant::Secondary)]
    pub variant: BtnVariant,
    #[props(default = false)]
    pub disabled: bool,
    pub icon: Option<ImStr>,
    pub on_click: Option<EventHandler<()>>,
}
```

**Why a separate `Btn` instead of extending `Button`:**
- The existing `Button` is consumed in many places with `primary: true|false`. Adding a `variant: BtnVariant` prop would create a confusing API where two props mean similar things. Defaulting `variant` from `primary` would be a magic coupling.
- Renaming `Button` → `Btn` and shipping a Big-Bang migration violates the master plan's "purely additive" rule for `02`.
- Two coexisting components for a short window is the lowest-risk path. The eventual cleanup is a one-grep find-and-replace.

**Default variant** is `Secondary` to match neutral usage; `Primary` must be opted into.

### 3. `PersonChip` enforces dark text via CSS rule, not inline style

The reference design demands that text on pastel backgrounds stays dark even in dark mode. If we set the color inline based on the current theme, future contributors might break the invariant by tweaking the dark-theme text color. Instead, we add a dedicated `.person-pill` rule in `input.css`:

```css
.person-pill {
  color: var(--chip-ink) !important;
}
```

with `--chip-ink: #0e1117` defined identically in both `:root` and `[data-theme="dark"]`. The component just adds the class.

The `!important` is acceptable because the rule is intentionally exempt from theming — that's its purpose.

`PersonChip` API:

```rust
#[derive(Props, Clone, PartialEq)]
pub struct PersonChipProps {
    pub name: ImStr,
    pub color: Option<ImStr>,    // pastel hex, e.g. "#dbe0ff"; None = dashed border
    #[props(default = false)]
    pub bold: bool,              // currently-edited highlight
    pub on_click: Option<EventHandler<()>>,
}
```

When `color = None`, render `border: 1px dashed var(--border-strong)`, transparent background, `color: var(--ink-soft)` — note: this fall-through path is the only place where `PersonChip` text color is **not** the chip-ink override; the `.person-pill` `!important` only applies when the pastel background is present. We achieve this by applying `.person-pill` only when `color.is_some()`.

### 4. `TupleRow` accepts `Element` for value, not just text

The reference design uses `TupleRow` for monetary values (mono), date ranges, sometimes a small color dot + text. Hard-coding it to `(label: ImStr, value: ImStr)` would force callers to compose strings and lose styling. Instead:

```rust
#[derive(Props, Clone, PartialEq)]
pub struct TupleRowProps {
    pub label: ImStr,
    pub value: Element,           // mono-styled at the call site if needed
    #[props(default = false)]
    pub dim: bool,                // ink-muted for secondary rows
    pub description: Option<Element>,
}
```

The mono font for numeric values is the caller's responsibility — `TupleRow` only provides the row layout (flex, label/value/optional description, 1 px bottom border).

### 5. `NavBtn` is `Btn`'s sibling, not a `Btn` variant

A square 28×28 icon button has different layout constraints than a regular button (no padding, glyph-only, fixed size). Cramming it into `BtnVariant` would muddy `Btn`'s API. Keep them separate:

```rust
#[derive(Props, Clone, PartialEq)]
pub struct NavBtnProps {
    pub glyph: ImStr,             // "‹", "›", "▾", etc.
    #[props(default = false)]
    pub disabled: bool,
    pub on_click: Option<EventHandler<()>>,
    pub aria_label: Option<ImStr>,
}
```

### 6. Tests use `dioxus_ssr` render snapshots

Existing component tests in the project use `cargo test` with utility functions. For atoms, snapshot tests via `dioxus_ssr::render` give us coverage on:
- Class presence per variant
- Disabled state class addition
- PersonChip no-color path producing dashed border
- Conditional `.person-pill` class based on `color.is_some()`

Tests live next to each atom (`#[cfg(test)] mod tests`).

## Risks / Trade-offs

**[Two button systems coexist]** — During `03`–`09`, both `Button` and `Btn` are reachable. Risk: contributors pick the wrong one. Mitigation: doc-comment on `Button` saying it is legacy and new code should use `Btn`; clarifying note in the master plan.

**[`!important` in CSS]** — `.person-pill` uses `!important` to enforce dark text. This is intentional per design intent but generally a smell. Mitigation: comment the rule explaining why; it's the only `!important` introduced.

**[`PersonChip` color invariant only enforced by class application logic]** — If a future contributor adds `.person-pill` without checking `color.is_some()`, the dashed-no-color path would also get the dark-ink override. Mitigation: keep the conditional in `PersonChip`, document it in the file's module comment, cover with a test.
