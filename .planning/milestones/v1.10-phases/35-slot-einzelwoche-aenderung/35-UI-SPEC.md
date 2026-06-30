---
phase: 35
slug: slot-einzelwoche-aenderung
status: draft
shadcn_initialized: false
preset: none
created: 2026-06-30
---

# Phase 35 — UI Design Contract

> Visual and interaction contract for the slot-editor mode switch ("nur diese Woche" vs "ab dieser
> Woche"). Scope is intentionally narrow: one new radio group added to the existing
> `SlotEditInner` dialog in `shifty-dioxus/src/component/slot_edit.rs`. No new color tokens,
> no new spacing scale, no new design language — strict reuse of the existing token system.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (Tailwind CSS + CSS custom-property tokens, no shadcn) |
| Preset | not applicable |
| Component library | none (project-internal atoms: `Btn`, `Dialog`, `Field`, `FormCheckbox`, `SelectInput`, `TextInput`) |
| Icon library | none |
| Font | Inter (sans-serif), JetBrains Mono (mono) — both already loaded; no change |

Source: `shifty-dioxus/tailwind.config.js` + `shifty-dioxus/input.css` (verified).

---

## Spacing Scale

Declared values (multiples of 4). These are pre-existing project tokens — Phase 35 introduces
no new spacing values. The only constraint is that new RSX must match the existing `gap-3`
column rhythm of `SlotEditInner`.

| Token | Tailwind class | Px value | Usage in this phase |
|-------|---------------|----------|---------------------|
| xs | `gap-1` / `space-y-1` | 4px | Vertical gap between radio options in the group |
| sm | `gap-2` | 8px | Horizontal gap between radio input and its label |
| md | `gap-3` | 12px | Matches existing `flex flex-col gap-3` outer column; radio group inserted as one more item |
| form | `px-[10px]` / `h-[34px]` | 10px / 34px | Existing form-input sizing — not repeated on radio buttons |

Exceptions: none. The `p-2.5` (10px) banner padding for conditional hint text follows the existing
overage-banner pattern in `slot_edit.rs:259`.

---

## Typography

All tokens are pre-existing. Phase 35 uses the following subsets in the new control:

| Role | Tailwind token | Px / line-height | Weight | Usage |
|------|---------------|-----------------|--------|-------|
| Body | `text-body` | 14px / 20px | 400 | Radio option labels |
| Small | `text-small` | 12px / 16px | 500 | Mode group section label; conditional hint paragraph |
| Ink-muted modifier | `text-ink-muted` | — | — | Section label + hint text (de-emphasised) |

Source: `tailwind.config.js` fontSize table (verified). No new sizes or weights introduced.

---

## Color

All tokens are pre-existing CSS custom properties. Phase 35 adds no new color values.

| Role | CSS token | Light hex | Usage in this phase |
|------|-----------|-----------|---------------------|
| Dominant (60%) | `--bg` / `bg-surface` | `#fbfbfc` / `#ffffff` | Dialog and radio button background |
| Secondary (30%) | `--surface-alt` / `border-strong` | `#f4f5f7` / `#d0d3da` | Radio input border |
| Accent (10%) | `--accent` / `accent-accent` | `#3a4cd1` | Radio button checked state (`accent-accent`); focus ring (`form-input:focus`) |
| Ink | `--ink` / `text-ink` | `#0e1117` | Radio option labels |
| Ink-muted | `--ink-muted` / `text-ink-muted` | `#6b7382` | Section label + hint paragraph |

Accent reserved for: checked-state of the radio inputs; focus ring on the radio inputs (via
global `form-input:focus` rule in `input.css`). Consistent with existing `FormCheckbox` usage.

No semantic color (bad/warn/good) is used in the new control. The conditional hint paragraph
for "nur diese Woche" uses `text-ink-muted` (informational, not a warning).

---

## Component Specification: Mode Radio Group

### Control type

Two `<input type="radio">` buttons inside a `<fieldset>`-equivalent container. Not a toggle
switch, not a `<select>`, not a checkbox. Reason: two mutually exclusive options with distinct
semantic meaning benefit from visible, simultaneously-readable labels (WCAG 1.3.1, 3.3.2).

### Visibility gate

Render the mode radio group ONLY when `props.slot_edit_type == SlotEditType::Edit`.
When `SlotEditType::New`, omit the group entirely (new slots are always "from this week").

### Position in dialog

Insert as the **second item** in the `flex flex-col gap-3` column — immediately after the
existing info `ul` (explanation text), and before the `Field { label: weekday_label }` row.
This placement frames the mode choice before the user edits field values.

```
Dialog (460px wide)
  flex flex-col gap-3
    ul.info-text                      ← existing
    [NEW] mode radio group            ← insert here (Edit mode only)
    Field: Wochentag                  ← existing
    Field: Von                        ← existing
    Field: Bis                        ← existing
    Field: Benötigte Personen         ← existing
    Field: Max. bezahlte Mitarbeiter  ← existing
    [conditional] overage banner      ← existing
    [conditional] error text          ← existing
```

### Default value

`single_week: bool = false` — "ab dieser Woche" is pre-selected when the dialog opens.
This preserves 100 % backward compatibility with the existing save behavior.

### Styling specification

**Outer wrapper (group label row + options column):**
```
div.flex.flex-col.gap-2
```

**Group section label:**
```
span.text-small.font-medium.text-ink-muted   — i18n key: SlotEditModeScopeLabel
```

**Options row (horizontal, two options side-by-side):**
```
div.flex.gap-4
```

**Each radio option (label wrapping input + text):**
```
label.inline-flex.items-center.gap-2.cursor-pointer.text-body.text-ink
  input[type="radio"].h-4.w-4.border.border-border-strong.accent-accent.form-input
  {option label text}
```

**Conditional hint paragraph** (visible ONLY when `single_week == true`):
```
p.text-small.font-normal.text-ink-muted.mt-1   — i18n key: SlotEditModeThisWeekOnlyHint
```

The hint is NOT a warning banner (no `border-l`, no `bg-warn-soft`). It is purely informational
and uses the same muted style as the existing explanation bullet text.

### Interaction states

| State | Rendering |
|-------|-----------|
| Dialog opens (Edit mode) | "Ab dieser Woche" radio pre-checked; hint hidden |
| User selects "Nur diese Woche" | "Nur diese Woche" radio checked; hint paragraph appears below group |
| User selects "Ab dieser Woche" | "Ab dieser Woche" radio checked; hint paragraph hides |
| Dialog opens (New mode) | Entire radio group absent (not rendered) |
| `on_save` fired | `SlotEditAction::SaveSlot` routed to `save_slot_edit` which branches on `store.single_week` |

No disabled state for the radio group itself. The group is always interactive when visible.

---

## Copywriting Contract

All user-facing strings are i18n-keyed (de/en/cs mandatory per SWO-04 + project convention).

### New i18n keys (this phase)

| Key | De | En | Cs |
|-----|----|----|-----|
| `SlotEditModeScopeLabel` | Geltungsbereich | Scope | Rozsah platnosti |
| `SlotEditModeFromThisWeek` | Ab dieser Woche (Standard) | From this week on (default) | Od tohoto týdne (výchozí) |
| `SlotEditModeThisWeekOnly` | Nur diese Woche | This week only | Pouze tento týden |
| `SlotEditModeThisWeekOnlyHint` | Die Änderungen gelten ausschließlich für Kalenderwoche {week}/{year}. Ab der Folgewoche werden die ursprünglichen Slot-Werte automatisch wiederhergestellt. | Changes apply exclusively to calendar week {week}/{year}. From the following week, the original slot values are automatically restored. | Změny platí výhradně pro kalendářní týden {week}/{year}. Od následujícího týdne jsou původní hodnoty slotu automaticky obnoveny. |

Interpolation tokens `{week}` and `{year}` follow the existing `i18n.t_m_rc(Key, map)` pattern
already used in `SlotEditExplanation`.

### Existing strings — no change

| Element | Existing key | Existing De copy |
|---------|-------------|-----------------|
| Primary CTA | `SaveLabel` | Speichern |
| Secondary CTA | `CancelLabel` | Abbrechen |
| Dialog title (Edit) | `SlotEditTitle` | Slot bearbeiten |
| Dialog title (New) | `SlotNewTitle` | Neuen Slot erstellen |
| Existing explanation bullet | `SlotEditExplanation` | Diese Änderungen werden ab der Kalenderwoche {week}/{year} angewendet... |
| Save error | `SlotEditSaveError` | Fehler beim Speichern |

The existing `SlotEditExplanation` bullet text already describes the "ab dieser Woche" behavior
correctly. When "nur diese Woche" is active, the new `SlotEditModeThisWeekOnlyHint` paragraph
provides the complementary information. The existing bullet is retained unchanged.

### Empty state, destructive confirmation

Not applicable to this phase. The mode radio group has no empty state (always one option
selected). There is no destructive action introduced in Phase 35.

---

## Structural State Changes

Phase 35 extends two existing files with no visual side effects on the New path:

| File | Change |
|------|--------|
| `state/slot_edit.rs` — `SlotEdit` struct | Add `pub single_week: bool` (default `false`). |
| `component/slot_edit.rs` — `SlotEditProps` | Optionally add `single_week: bool` prop OR read directly from `SLOT_EDIT_STORE` inside `SlotEdit` (either is acceptable; `SlotEditInner` tests use `SlotEditProps` directly so adding the prop keeps SSR tests straightforward). |

The `SlotEditInner` SSR tests must gain one new case:
- Mode radio group absent in HTML when `slot_edit_type == SlotEditType::New`.
- Mode radio group present and "ab dieser Woche" pre-checked when `slot_edit_type == SlotEditType::Edit, single_week = false`.
- Hint paragraph absent when `single_week = false`; present when `single_week = true`.

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | none | not applicable |
| Third-party | none | not applicable |

Phase 35 installs zero new packages and uses zero third-party component registries.

---

## Legacy Class Guard

The existing test `slot_edit_no_legacy_classes_in_source` in `slot_edit.rs` enforces that
the production source contains no legacy Tailwind classes (`bg-gray-*`, `bg-white`,
`text-gray-*`, `text-blue-*`, `text-red-*`, etc.). The new radio group MUST pass this test.
Use ONLY design-token-aliased classes (`bg-surface`, `text-ink`, `border-border-strong`,
`accent-accent`, etc.).

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
