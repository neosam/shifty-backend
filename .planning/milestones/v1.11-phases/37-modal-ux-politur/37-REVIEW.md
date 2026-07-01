---
phase: 37-modal-ux-politur
reviewed: 2026-07-01T16:15:33Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - shifty-dioxus/src/component/dialog.rs
  - shifty-dioxus/src/component/absence_convert_modal.rs
  - shifty-dioxus/src/component/contract_modal.rs
  - shifty-dioxus/src/i18n/mod.rs
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/cs.rs
findings:
  critical: 0
  warning: 2
  info: 4
  total: 6
status: issues_found
---

# Phase 37: Code Review Report

**Reviewed:** 2026-07-01T16:15:33Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Reviewed the MOD-01 drag-safe backdrop state machine (`dialog.rs` + inline copy in
`absence_convert_modal.rs`) and the MOD-02 contract-modal help-text i18n work. The
core deliverables are correct:

- **`BackdropPress` state machine is sound.** The three transitions
  (`press_backdrop` → true, `press_panel` → false, `release` → return+reset) are
  correct, and the DOM wiring is right: the panel `onmousedown` uses
  `stop_propagation` **plus** an explicit `press_panel()` call. The
  `stop_propagation` is load-bearing — without it the panel's mousedown would bubble
  to the backdrop's `onmousedown` (bubbling fires child-first, then parent) and
  wrongly re-arm the flag. Both are present in both implementations.
- **No Signal borrow-across-await risk.** The `if backdrop_press.write().release() { on_close.call(()) }`
  pattern is safe: Rust drops the `Write` guard at the end of the `if` *condition*
  expression, before the consequent block runs, so `on_close`/`on_cancel` never
  executes while the borrow is held. No `.await` is involved.
- **The two backdrop implementations are behaviorally identical** at the
  state-machine level (backdrop `onmousedown`=press_backdrop, `onclick`=release→close;
  panel `onmousedown`=stop_propagation+press_panel, `onclick`=stop_propagation).
- **ESC and X-close are unaffected (D-04).** ESC uses a window-level `keydown`
  listener; the X button retains `stop_propagation` + `on_close`. Neither touches
  the backdrop-press flag.
- **All six new `*Help` keys exist in de/en/cs with distinct copy** (no silent
  German fallback), the German texts match the spec verbatim, the `Key` enum in
  `mod.rs` is consistent, and pinning tests guard each locale. From/To correctly
  have no help span; `CommittedVoluntaryHelp` is correctly inside the
  `if show_committed` block.

No BLOCKER-level defects were found. The findings below are quality/robustness
issues, most of them pre-existing and surfaced by adjacency to the new work.

Note on the clippy gate: `shifty-dioxus` is a separate workspace **excluded** from
the CI `clippy -D warnings` gate (per project memory: ~198 pre-existing dioxus
lints), so the redundant-branch defect below does not break the build here even
though `clippy::if_same_then_else` would normally flag it.

## Narrative Findings (AI reviewer)

## Warnings

### WR-01: New panel `onmousedown` stop_propagation blocks document-level mousedown for Dialog children

**File:** `shifty-dioxus/src/component/dialog.rs:257-260` (and `absence_convert_modal.rs:103-106`)
**Issue:** MOD-01 added a **new** `onmousedown` handler on the panel that calls
`evt.stop_propagation()`. At base (`c67413b`) the panel had only an `onclick`
stop_propagation — mousedown events inside the panel previously bubbled all the way
to `document`/`window`. They now stop at the panel. `Dialog` is a generic, shared
shell; any child rendered inside it that relies on a global `document`/`window`
`mousedown` "click-outside-to-close" listener (e.g. a custom combobox/popover)
would silently stop receiving that event and fail to close on outside press. No
current in-tree consumer is affected (the only global mousedown listener,
`top_bar.rs`, is never mounted inside a Dialog, and `ContractModalBody` uses native
`<select>`/`<input>` which are unaffected), so this is latent — but it is a real
behavioral delta with no test guarding it.
**Fix:** If a future child needs document-level outside-click detection, prefer
having such popovers listen on the capture phase or scope their own containment
check rather than relying on bubbling. At minimum, document this constraint on the
`Dialog` component so callers know panel-internal mousedown does not reach the
document. If drag-safety only needs the flag (not propagation blocking), consider
whether `press_panel()` alone (already sets the flag false) plus keeping the
`onclick` stop_propagation is sufficient — but note the mousedown stop_propagation
is currently required to prevent the backdrop's `onmousedown` from re-arming the
flag, so removing it would require re-ordering the guard.

### WR-02: Redundant `if read_only { … } else { … }` with identical branches for `cancel_label`

**File:** `shifty-dioxus/src/component/contract_modal.rs:77-81`
**Issue:** Both arms produce the exact same value:
```rust
let cancel_label = if read_only {
    ImStr::from(i18n.t(Key::Cancel).as_ref())
} else {
    ImStr::from(i18n.t(Key::Cancel).as_ref())
};
```
This is a dead conditional that signals a dropped requirement — a read-only contract
dialog almost certainly wanted a "Close" label (e.g. `Key::ErrorBannerDismiss` or a
dedicated Close key) rather than "Cancel". As written it is redundant logic that
`clippy::if_same_then_else` would flag (only un-caught here because this workspace is
excluded from the clippy gate). Pre-existing at `c67413b` but present in a reviewed
file and worth resolving while the modal is being polished.
**Fix:** Either collapse to a single assignment
`let cancel_label = ImStr::from(i18n.t(Key::Cancel).as_ref());`, or — if the
read-only case is supposed to read "Close" — introduce/route a distinct key:
```rust
let cancel_label = if read_only {
    ImStr::from(i18n.t(Key::Close).as_ref()) // add Close to all three locales
} else {
    ImStr::from(i18n.t(Key::Cancel).as_ref())
};
```

## Info

### IN-01: `DynamicHourLabel` missing in cs.rs → silent English fallback next to new Czech help text

**File:** `shifty-dioxus/src/i18n/cs.rs:314` (gap; `DynamicHourHelp` added directly, no preceding `DynamicHourLabel`)
**Issue:** `de.rs` and `en.rs` both register `DynamicHourLabel`, but `cs.rs` does
not (confirmed absent at base and still absent). `contract_modal.rs:147` renders
`i18n.t(Key::DynamicHourLabel)`, which for Czech falls back to the English
"Dynamic hours". Phase 37 added the Czech `DynamicHourHelp` immediately below it, so
Czech users now see an English **label** paired with a Czech **help line** — the new
work makes the pre-existing gap more visible.
**Fix:** Add `i18n.add_text(Locale::Cs, Key::DynamicHourLabel, "Dynamické hodiny");`
(Czech copy TBD) so the label and its help are in the same language.

### IN-02: Hardcoded, untranslated user-facing error string

**File:** `shifty-dioxus/src/component/absence_convert_modal.rs:203`
**Issue:** The invalid-date branch sets `error_msg` to a literal
`"Invalid date format".to_string()` instead of an i18n key. English string leaks to
de/cs users. Pre-existing (outside the MOD-01 diff hunk) but in a reviewed file.
**Fix:** Add an `AbsenceConvertErrInvalidDate` key to all three locales and use
`i18n.t(...)` here, mirroring the existing `AbsenceConvertErrStartAfterEnd` handling.

### IN-03: Asymmetric drag-safety — a press starting on the backdrop that releases on the panel still closes

**File:** `shifty-dioxus/src/component/dialog.rs:243-250`
**Issue:** With `mousedown` on the backdrop (flag=true) and `mouseup` inside the
panel, the DOM dispatches the `click` at the deepest common ancestor — the backdrop —
so the panel's `onclick` stop_propagation never runs and `release()` returns true,
closing the modal. MOD-01's scope only requires the *panel→backdrop* drag to be
safe (which works correctly), and this reverse case behaves identically to the
pre-MOD-01 plain-onclick backdrop, so it is **not a regression**. Noting it as a
known limitation: drag-safety is one-directional.
**Fix (optional):** If full symmetry is desired, also clear/ignore the flag when the
`mouseup`/click target is the panel (e.g. gate the close on the click's actual
target being the backdrop element). Not required for this phase.

### IN-04: z-index divergence between the two backdrops

**File:** `shifty-dioxus/src/component/absence_convert_modal.rs:93` (`z-50`) vs `dialog.rs:70` (`z-index:200`)
**Issue:** The inline `AbsenceConvertModal` backdrop uses Tailwind `z-50` while the
central `Dialog` backdrop uses `z-index:200`. The state-machine behavior is
identical (the concern raised in the brief), but the stacking contexts differ. Only
relevant if both surfaces are ever open simultaneously; pre-existing styling, not a
MOD-01 behavior difference.
**Fix:** Align the two z-index values if concurrent modals are ever possible;
otherwise leave as-is.

---

_Reviewed: 2026-07-01T16:15:33Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
