# Redesign 04 — Modal Variants & Form Atoms

> Master plan: `../REDESIGN_PLAN.md`. Reference: `design_handoff_shifty/modal.jsx`.

## Why

The existing `src/component/modal.rs` is a single 32-line component (`Modal { children: Element }`) that renders a fixed center dialog with white background and no dismiss handling (no ESC, no backdrop-click). The new design defines four layout variants (`center`, `sheet`, `bottom`, `auto`), proper dismissal, body scroll lock, and a small library of styled form atoms.

Eight call sites use the existing `Modal` today (`employees.rs`, `employee_details.rs`, `shiftplan_tab_bar.rs` ×2, `my_employee_details.rs`, `slot_edit.rs`, `employee_view.rs`, `employees.rs` ×2 actually). To avoid touching all of them in this foundation change, we add a parallel new component and migrate per-page in `05`–`09`. Same strategy as `Btn` vs. `Button` in change `02`.

## What Changes

- New component `Dialog` in `src/component/dialog.rs`:
  - Variants: `Center` | `Sheet` | `Bottom` | `Auto`
  - `Auto` resolves at runtime via `(max-width: 720px)` mediaquery → `Bottom` on mobile, `Center` otherwise; updates live on viewport changes
  - Backdrop click + `Esc` + close-X button + Cancel button all dismiss when `on_close` is provided
  - Body scroll lock while open (set `document.body.style.overflow = 'hidden'`, restore prior value on close)
  - Drag-handle bar visible at the top of the `Bottom` variant (purely visual cue, no actual drag-to-dismiss)
  - Header: title (16/700, tracking -0.01em), optional subtitle (12/ink-muted), close-X button on the right
  - Footer: optional `Element` slot, sticky at bottom with `surface-alt` background and 1 px top border
  - `width` prop (default 460 px) caps the `Center` panel; `Sheet` adds 60 px to it
- Form atoms in new module `src/component/form/`:
  - `Field { label, hint?, error?, span?, children }` — uppercase 11 px label, hint or error line below
  - `FormTextInput`, `FormSelectInput`, `FormTextareaInput` — height 34 px (textarea is auto), padding `0 10px`, `border-strong` border, accent focus ring (`box-shadow: 0 0 0 3px var(--accent-soft)`)
  - Form-atom names are prefixed `Form*` to avoid collision with existing `TextInput` etc. in `base_components.rs`
- CSS keyframes in `input.css`:
  - `shifty-modal-fade` (backdrop, 160 ms ease-out)
  - `shifty-modal-pop` (center, 180 ms cubic-bezier)
  - `shifty-modal-slide-right` (sheet, 220 ms cubic-bezier)
  - `shifty-modal-slide-up` (bottom, 220 ms cubic-bezier)
- Custom hook `use_media_query(query: &str) -> Signal<bool>` in `src/component/dialog.rs` (or shared helper module) for the auto-variant resolution; subscribes to `MediaQueryList` change events

## Out of scope

- Migrating the 8 existing `Modal` call sites — happens per-page in `05`–`09`. Existing `Modal` stays untouched.
- The actual `ContractModal` and `ExtraHoursModal` content modals — those compose `Dialog` + form atoms in change `07`
- The `SlotModal` redesign — composes `Dialog` + form atoms in change `09`
- Removing the legacy `Modal` — later cleanup change once all call sites are migrated

## Capabilities

### New
- `dialog`: layered modal with center/sheet/bottom/auto variants, dismiss handling, body scroll lock, header/footer slots
- `form-atoms`: Field, FormTextInput, FormSelectInput, FormTextareaInput

## Impact

- Files: new `src/component/dialog.rs`, new `src/component/form/{mod,field,inputs}.rs`, extended `input.css` (keyframes + global focus-ring rules), `src/component/mod.rs` for re-exports
- Cargo: existing `web-sys` features may need `MediaQueryList`, `MediaQueryListEvent` enabled — verify
- No existing call sites are touched; the two modal components coexist
- Tests: variant resolution in viewport sizes, dismiss paths (ESC, backdrop, X, cancel), focus-ring style on inputs, body scroll-lock add/remove
