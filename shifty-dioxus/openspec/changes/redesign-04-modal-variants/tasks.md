## 1. CSS Animations and Global Rules

- [ ] 1.1 Add `@keyframes shifty-modal-fade`, `shifty-modal-pop`, `shifty-modal-slide-right`, `shifty-modal-slide-up` to `input.css`
- [ ] 1.2 Add `.form-input:focus { ... }` global rule with accent focus-ring (border-color + box-shadow) in `input.css`
- [ ] 1.3 Add `.dialog-backdrop` and `.dialog-panel-{center,sheet,bottom}` rules if any panel-specific styles benefit from CSS over inline (decision during implementation)

## 2. `use_media_query` Hook

- [ ] 2.1 Create `pub fn use_media_query(query: &'static str) -> Signal<bool>` in `src/component/dialog.rs`
- [ ] 2.2 Implement initial value via `web_sys::window().match_media(query)`
- [ ] 2.3 Implement live updates via `MediaQueryList::add_listener` with a `Closure::wrap` callback that mutates the signal
- [ ] 2.4 Implement cleanup: detach listener and drop the closure on `use_effect` cleanup
- [ ] 2.5 Verify `web-sys` features include `MediaQueryList` and `MediaQueryListEvent` (extend `Cargo.toml` if needed)

## 3. `Dialog` Component

- [ ] 3.1 Create `src/component/dialog.rs` with `DialogVariant { Center, Sheet, Bottom, Auto }` and `Dialog` component
- [ ] 3.2 Resolve `Auto` variant: use `use_media_query("(max-width: 720px)")` to pick `Bottom` (mobile) or `Center` (desktop)
- [ ] 3.3 Backdrop: `position: fixed; inset: 0; background: var(--modal-veil); z-index: 200;` plus alignment per variant (center/end/end)
- [ ] 3.4 Panel: variant-specific border-radius, animation, height/width caps (Center: width prop; Sheet: width+60; Bottom: full width)
- [ ] 3.5 Header: title (16/700, `tracking-tight`), optional subtitle (12 px / `text-ink-muted`), close-X button on the right
- [ ] 3.6 Body: padding `14px 18px 16px`, `overflow-y: auto`, `flex: 1`
- [ ] 3.7 Footer (when provided): `flex justify-end gap-2 px-[18px] py-3 border-t border-border bg-surface-alt`
- [ ] 3.8 Drag-handle for `Bottom` variant: 36×4 pill at top, `bg-border-strong rounded-full`
- [ ] 3.9 Body scroll lock via `use_effect` (set/restore `body.style.overflow`)
- [ ] 3.10 ESC dismiss: `window.addEventListener("keydown", ...)` on open, detach on close/unmount
- [ ] 3.11 Backdrop click dismiss: `onclick` on backdrop calls `on_close.call(())`; panel `onclick` does `event.stop_propagation()`
- [ ] 3.12 Close-X button calls `on_close.call(())`

## 4. Form Atoms — `Field`

- [ ] 4.1 Create `src/component/form/mod.rs` with `pub mod field; pub mod inputs;` and re-exports
- [ ] 4.2 Create `src/component/form/field.rs` with `Field` component and `FieldProps`
- [ ] 4.3 Render `<label>` semantic element wrapping a column-flex container
- [ ] 4.4 Label text: `text-[11px] font-semibold text-ink-soft uppercase tracking-[0.04em]`
- [ ] 4.5 `span` prop optional integer (1 or 2) → maps to `grid-column: span 2` for grid layouts
- [ ] 4.6 Hint slot below input: `text-[11px] text-ink-muted`; not rendered when `error.is_some()`
- [ ] 4.7 Error slot below input: `text-[11px] text-bad`; preempts hint

## 5. Form Atoms — Inputs

- [ ] 5.1 Create `src/component/form/inputs.rs` with `FormTextInput`, `FormSelectInput`, `FormTextareaInput` components
- [ ] 5.2 `FormTextInput`: `<input>` with `h-[34px] px-[10px] border border-border-strong rounded-md bg-surface text-ink text-[13px] form-input` plus value/onchange props
- [ ] 5.3 `FormSelectInput`: `<select>` similar styling, with custom dropdown-arrow background-image (right 10 px center) and `appearance: none`
- [ ] 5.4 `FormTextareaInput`: `<textarea>` with `min-h-[68px] px-[10px] py-2 leading-[1.45]`, vertical-resize allowed
- [ ] 5.5 Each component accepts `disabled: bool`, propagates to the underlying element
- [ ] 5.6 Each component accepts `placeholder: Option<ImStr>`

## 6. Module Re-Exports and Documentation

- [ ] 6.1 Extend `src/component/mod.rs` to `pub mod dialog;`, `pub mod form;` and re-export `Dialog`, `DialogVariant`, `Field`, `FormTextInput`, `FormSelectInput`, `FormTextareaInput`
- [ ] 6.2 Add doc-comment to legacy `src/component/modal.rs::Modal` marking it as legacy and pointing to `Dialog`

## 7. Tests

- [ ] 7.1 `use_media_query`: unit test initial value when `window.matchMedia(...)` matches and doesn't match
- [ ] 7.2 `Dialog`: render test for each variant — verify panel classes/inline-style
- [ ] 7.3 `Dialog`: ESC simulation calls `on_close`
- [ ] 7.4 `Dialog`: backdrop click calls `on_close`; panel click does NOT
- [ ] 7.5 `Dialog`: close-X button calls `on_close`
- [ ] 7.6 `Dialog`: opening sets `body.style.overflow = "hidden"`; closing restores prior value
- [ ] 7.7 `Field`: hint shown when no error; error shown and hint suppressed when error provided
- [ ] 7.8 `FormTextInput`: focus state applies via `.form-input` class; disabled state propagates
