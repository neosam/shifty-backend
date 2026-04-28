# Redesign 02 — Atom Components

> Master plan: `../REDESIGN_PLAN.md`. Reference: `design_handoff_shifty/README.md` (Component inventory).

## Why

Pages share a small set of low-level building blocks. Implementing them once, before any page is redesigned, prevents inconsistency and rework across `03`–`09`.

The current `src/component/base_components.rs` contains a `Button` component (only 2 variants: `primary: bool`), no `PersonChip`, no `TupleRow`, no `NavBtn`. Person-name pills are currently rendered inline inside `week_view.rs` with ad-hoc Tailwind classes and an inline `style: "background-color: …"`. The redesign needs richer, semantically-named atoms.

## What Changes

- **`Btn`** — new component in `src/component/atoms/btn.rs`:
  - 4 variants: `Primary`, `Secondary`, `Ghost`, `Danger` (Rust enum `BtnVariant`)
  - Padding `6px 12px`, radius `md`, font 13/500
  - Optional `icon: Option<&str>` mono prefix glyph
  - `disabled: bool` → 50% opacity, `cursor-not-allowed`
  - Uses tokens from `01`: `bg-accent`/`text-accent-ink`, `bg-surface`/`text-ink`, transparent/`text-ink-soft`, `bg-surface`/`text-bad`/`border-bad`
- **`PersonChip`** — new component in `src/component/atoms/person_chip.rs`:
  - Background = `person.color` (pastel hex) when set, transparent dashed border when `None`
  - Text color **always dark ink** in both themes — enforced via dedicated `.person-pill` CSS rule with `color: var(--chip-ink) !important` (with `--chip-ink: #0e1117` defined globally in `input.css` so it survives dark theme)
  - Padding `1px 4px 1px 7px`, radius `sm`, font 12/500
  - **No initials**, no avatar circle inside
  - Optional `bold: bool` for highlighting the currently-edited person (used in week-view)
- **`TupleRow`** — new component in `src/component/atoms/tuple_row.rs`:
  - Label left, mono value right, 1 px bottom border, 13 px text
  - Optional `dim: bool` variant for secondary fields (lower contrast via `text-ink-muted`)
  - Optional `description` slot below the value (multi-line allowed)
- **`NavBtn`** — new component in `src/component/atoms/nav_btn.rs`:
  - Square 28×28, `border-strong`, mono glyph (`‹` `›` typically)
  - `disabled: bool`
- New module `src/component/atoms/mod.rs` re-exports all four; `src/component/mod.rs` re-exports `atoms::*`
- Add `--chip-ink: #0e1117` CSS variable to both light and dark theme blocks in `input.css` (added in `01`); the value is identical in both themes by design
- Add `.person-pill` rule to `input.css` with `color: var(--chip-ink) !important;`

## Out of scope

- Migrating existing call sites: the legacy `Button` and inline person-pill code stay in place. Migration happens per-page in `05`–`09` and the existing `Button` may be removed in a later cleanup change once all call sites are converted
- Form atoms (`Field`, `TextInput`, `SelectInput`, `TextareaInput`) — those live with the modal in change `04`
- Layout containers (TopBar lives in `03`)

## Capabilities

### New
- `atom-components`: reusable button, person chip, tuple row, nav button

## Impact

- Files: new `src/component/atoms/{mod,btn,person_chip,tuple_row,nav_btn}.rs`, extended `src/component/mod.rs`, extended `input.css` (chip-ink variable + `.person-pill` rule)
- No existing call sites are touched — pure additive change. The two component systems (legacy `Button` + new `Btn`) coexist until migrated.
- Tests: render snapshot per atom with each variant; PersonChip-without-color path; Btn-disabled state
