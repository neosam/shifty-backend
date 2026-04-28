## Context

The existing `src/component/modal.rs` is intentionally minimal — it renders a centered dialog with `bg-white` and no dismiss logic, leaving call sites to manage their own open/close state. There are 8 active call sites:

- `src/page/employees.rs` (×2)
- `src/page/employee_details.rs`
- `src/page/my_employee_details.rs`
- `src/component/shiftplan_tab_bar.rs` (×2)
- `src/component/employee_view.rs`
- `src/component/slot_edit.rs`

Each of these wraps content in `Modal { ... }` and conditionally renders based on a local signal. The reference design in `design_handoff_shifty/modal.jsx` defines a richer surface: title/subtitle/footer slots, three layout variants plus `auto`, a CSS-animated backdrop, body scroll lock, and ESC handling.

The reference also defines four form atoms (`Field`, `TextInput`, `SelectInput`, `TextareaInput`). Two of those names collide with existing components in `base_components.rs` (`TextInput`, `Select` — close enough). The new atoms are visually different (34 px height, accent focus ring, token-based) and serve as the basis for redesigned modal forms.

Following the same purely-additive strategy as change `02` (`Btn` alongside legacy `Button`), this change introduces a new component named `Dialog` next to the existing `Modal`. The legacy `Modal` is not touched. Per-page migration happens in `05`–`09`.

## Goals / Non-Goals

**Goals:**
- New `Dialog` component with `Center` / `Sheet` / `Bottom` / `Auto` variants
- Live `Auto` resolution responding to viewport changes
- Standard dismiss paths (ESC, backdrop, X button, Cancel) wired through a single `on_close` callback
- Body scroll lock while open, properly restored on close
- Form atoms (`Field`, `FormTextInput`, `FormSelectInput`, `FormTextareaInput`) using design tokens
- Existing `Modal` and existing `TextInput`/etc. continue to work unchanged

**Non-Goals:**
- Migrating call sites
- Implementing actual drag-to-dismiss for `Bottom` variant (the drag-handle bar is purely visual)
- Focus trap inside the dialog (not in the reference design; revisit later if a11y review demands it)
- Stacked dialogs (only one open at a time; current code never opens multiple anyway)

## Decisions

### 1. New component named `Dialog`, legacy `Modal` untouched

Naming options considered:

| Option | Pros | Cons |
|---|---|---|
| Extend `Modal` with optional props | Single component, no name churn | Default-look change risks breaking 8 sites; "two-personality" component |
| New `Modal2` | Clear migration semantics | Ugly name |
| **New `Dialog`** | Idiomatic name from web standards (`<dialog>`); clear distinction | Two names for similar concepts during transition |
| Rename existing → new takes name | No suffix in the long term | Big-bang rename of 8 sites |

**Chosen: `Dialog`.** The HTML `<dialog>` element name precedent makes this idiomatic. After migration, the legacy `Modal` is removed in a cleanup change; `Dialog` keeps its name (no rename).

### 2. `Dialog` API

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum DialogVariant { Center, Sheet, Bottom, Auto }

#[derive(Props, Clone, PartialEq)]
pub struct DialogProps {
    pub open: bool,
    pub on_close: EventHandler<()>,
    pub title: ImStr,
    pub subtitle: Option<ImStr>,
    pub children: Element,
    pub footer: Option<Element>,
    #[props(default = DialogVariant::Auto)]
    pub variant: DialogVariant,
    #[props(default = 460)]
    pub width: u32,
}
```

Notes:
- `on_close` is **required** — there's no use case for a dialog that can't be dismissed; if a caller doesn't want backdrop-click to close, they pass an `on_close` that ignores irrelevant signals (rare, but possible)
- `title` is required; subtitle/footer are optional
- `variant` defaults to `Auto` (most pages benefit from mobile-friendly behavior)

### 3. `Auto` variant resolves via custom `use_media_query` hook

Dioxus 0.6 has no built-in mediaquery hook. We provide a small one:

```rust
// src/component/dialog.rs (or shared helper)
fn use_media_query(query: &'static str) -> Signal<bool> {
    let mut matches = use_signal(|| {
        web_sys::window()
            .and_then(|w| w.match_media(query).ok().flatten())
            .map(|mql| mql.matches())
            .unwrap_or(false)
    });

    use_effect(move || {
        let mql = web_sys::window()
            .and_then(|w| w.match_media(query).ok().flatten());
        // attach listener via Closure, store cleanup
        // ...
    });

    matches
}
```

Caller pattern:

```rust
let is_mobile = use_media_query("(max-width: 720px)");
let resolved = match variant {
    DialogVariant::Auto => if *is_mobile.read() { DialogVariant::Bottom } else { DialogVariant::Center },
    other => other,
};
```

The hook is reusable beyond `Dialog` (e.g. mobile burger logic in `03`); we put it in a shared location like `src/component/hooks.rs` so other consumers can use it. Decision: ship it inside `dialog.rs` initially with `pub fn` so it's reusable; promote to `hooks.rs` if a second consumer appears.

### 4. Body scroll lock managed via `use_effect` cleanup

When the dialog opens, lock body scroll; when it closes, restore. `use_effect` with cleanup:

```rust
use_effect(move || {
    if !open { return; }
    let body = web_sys::window().unwrap().document().unwrap().body().unwrap();
    let prev = body.style().get_property_value("overflow").unwrap_or_default();
    let _ = body.style().set_property("overflow", "hidden");
    // cleanup: restore prev
});
```

Edge case: if multiple dialogs open at once (which we said is non-goal), the second one's restore would clobber the first's. We don't guard against this; document as non-goal.

### 5. ESC handler attached at `window` while open

When the dialog is open, attach a `keydown` listener at `window`. On `Esc`, call `on_close`. Detach in cleanup.

The listener is local to the dialog instance — no global registry. If another dialog opens while this one is open (non-goal), both would receive ESC; but we don't support that case.

### 6. Form atoms named `Form*` to avoid collisions

Existing `base_components.rs` already has:
- `TextInput`, `DateInput`, `IntegerInput`, `TimeInput`, `FloatInput`, `Select`, `SimpleSelect`, `Checkbox`

We don't want to rename existing components in this change. Naming the new atoms `FormTextInput`, `FormSelectInput`, `FormTextareaInput`, plus `Field` (no collision):

```rust
// src/component/form/mod.rs
pub mod field;
pub mod inputs;
pub use field::Field;
pub use inputs::{FormTextInput, FormSelectInput, FormTextareaInput};
```

Rationale for the prefix: marks them as "the new tokens-based design system inputs" while leaving existing names intact. After migration, a cleanup change can drop the prefix and remove the legacy versions.

`Field` is a wrapper component — semantic `<label>` element, uppercase tracking-wide label, and slot for the input. Hint and error are mutually exclusive (error preempts hint).

### 7. Animations live in `input.css`

Keyframes and the `.dialog-*` rules live in `input.css` (added in `01`'s file but new keyframes here). This avoids inlining keyframe definitions in Rust strings and keeps CSS in CSS:

```css
@keyframes shifty-modal-fade { from { opacity: 0; } to { opacity: 1; } }
@keyframes shifty-modal-pop  { from { opacity: 0; transform: scale(0.96); } to { opacity: 1; transform: scale(1); } }
@keyframes shifty-modal-slide-right { from { transform: translateX(100%); } to { transform: translateX(0); } }
@keyframes shifty-modal-slide-up    { from { transform: translateY(100%); } to { transform: translateY(0); } }
```

Components apply via inline `style` attributes (since values depend on variant) or via classes per variant.

### 8. Focus ring on inputs is a global CSS rule, not per-component

To avoid repeating focus-ring boilerplate in every form input:

```css
.form-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}
```

Each `Form*Input` adds the `form-input` class. Component code stays focused on layout and prop-handling.

## Risks / Trade-offs

**[`use_media_query` hook complexity]** — Implementing it correctly in WASM requires `Closure::wrap` for the listener and proper `forget` / explicit drop semantics. This is the kind of code that's easy to leak. Mitigation: keep the hook small, comment the lifecycle, and add a unit test that exercises it. If it proves too tricky, fall back to reading the matchMedia value once on `use_effect` first run and not subscribing — auto resolution would then fix at mount time, which is acceptable for most cases (users rarely resize across the breakpoint mid-session).

**[Two modal components coexist]** — Same risk as `Btn` vs `Button` in `02`. Mitigation: doc-comment on legacy `Modal` pointing to `Dialog`.

**[Body scroll lock breaks if user navigates with dialog open]** — If a route change unmounts the dialog, the cleanup runs and restores scroll. Confirmed: `use_effect` cleanup runs on unmount. Non-issue in normal flow but worth noting.

**[ESC listener overlap with other components]** — Some components (e.g. dropdowns) might also handle ESC. With the dialog's listener at `window`, both fire. Mitigation: dropdowns inside dialogs should call `event.stopPropagation` if they want to close just themselves. Document in dropdown component if/when this comes up.

**[Form-atom naming `Form*` is permanent if migration stalls]** — If we never do the cleanup change, we're stuck with `FormTextInput` forever, which is uglier than `TextInput`. Mitigation: explicit task in the cleanup-change skeleton (to be added) to rename after migration completes.
