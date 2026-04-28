## 1. Module Setup

- [x] 1.1 Create directory `src/component/atoms/`
- [x] 1.2 Create `src/component/atoms/mod.rs` that re-exports `btn::*`, `person_chip::*`, `tuple_row::*`, `nav_btn::*`
- [x] 1.3 Extend `src/component/mod.rs` to `pub mod atoms;` and re-export commonly used names (`Btn`, `BtnVariant`, `PersonChip`, `TupleRow`, `NavBtn`)

## 2. CSS Additions

- [x] 2.1 Add `--chip-ink: #0e1117;` to **both** the `:root` and `[data-theme="dark"]` blocks in `input.css` (identical value by design)
- [x] 2.2 Add a global `.person-pill { color: var(--chip-ink) !important; }` rule in `input.css` with a comment explaining why `!important` is intentional

## 3. `Btn` Component

- [x] 3.1 Create `src/component/atoms/btn.rs` with `BtnVariant { Primary, Secondary, Ghost, Danger }` and `Btn` component
- [x] 3.2 Implement variant-specific class strings using design tokens (`bg-accent`/`text-accent-ink`, `bg-surface`/`text-ink`/`border-border-strong`, transparent/`text-ink-soft`, `bg-surface`/`text-bad`/`border-bad`)
- [x] 3.3 Apply common base classes: `px-3 py-1.5 rounded-md text-[13px] font-medium border` + variant overrides
- [x] 3.4 Disabled state: `opacity-50 cursor-not-allowed`, suppress `on_click` invocation
- [x] 3.5 Optional icon: render `<span class="font-mono mr-1">{icon}</span>` before children when `icon.is_some()`
- [x] 3.6 Default `variant: Secondary` via `#[props(default = ...)]`
- [x] 3.7 Tests: render snapshot for each variant, disabled state, with-and-without icon

## 4. `PersonChip` Component

- [x] 4.1 Create `src/component/atoms/person_chip.rs` with `PersonChip` component and `PersonChipProps`
- [x] 4.2 With color: render `<span class="person-pill">` with inline `style: "background-color: {color}"` and Tailwind `inline-flex px-[4px] pl-[7px] py-px rounded-sm text-xs font-medium`
- [x] 4.3 Without color: render `<span>` (no `.person-pill` class) with `border border-dashed border-border-strong text-ink-soft bg-transparent` plus same shape classes
- [x] 4.4 `bold: true` adds `font-semibold` class (covers the highlight case for current edit person)
- [x] 4.5 `on_click` optional; if provided, attach `onclick` and `cursor-pointer`
- [x] 4.6 Document the color invariant in a module-level comment
- [x] 4.7 Tests: with-color produces `.person-pill` class; without-color produces dashed border classes; bold variant adds `font-semibold`

## 5. `TupleRow` Component

- [x] 5.1 Create `src/component/atoms/tuple_row.rs` with `TupleRow` component and `TupleRowProps`
- [x] 5.2 Render flex row: `flex items-baseline justify-between gap-3 py-1.5 border-b border-border text-[13px]`
- [x] 5.3 Label uses `text-ink-soft`, when `dim: true` whole row uses `text-ink-muted`
- [x] 5.4 Value slot is a generic `Element` (mono font is the caller's responsibility)
- [x] 5.5 If `description.is_some()`, render below the row in `text-xs text-ink-muted`
- [x] 5.6 Tests: dim variant changes class; description slot renders when present

## 6. `NavBtn` Component

- [x] 6.1 Create `src/component/atoms/nav_btn.rs` with `NavBtn` component and `NavBtnProps`
- [x] 6.2 Render: `w-7 h-7 inline-flex items-center justify-center border border-border-strong rounded-md font-mono text-ink-soft hover:bg-surface-alt`
- [x] 6.3 Disabled state: `opacity-50 cursor-not-allowed`, suppress `on_click`
- [x] 6.4 Apply `aria-label` when provided, otherwise omit
- [x] 6.5 Tests: glyph rendering, disabled suppresses click, aria-label propagation

## 7. Documentation

- [x] 7.1 Add a doc-comment on `src/component/base_components.rs::Button` marking it as legacy and pointing to `Btn`
- [x] 7.2 Add a module-level doc-comment to `src/component/atoms/mod.rs` summarizing the atoms and their intended use
