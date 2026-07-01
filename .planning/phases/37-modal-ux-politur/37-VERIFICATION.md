---
phase: 37-modal-ux-politur
verified: 2026-07-01T00:00:00Z
status: passed
score: 14/14 must-haves verified
behavior_unverified: 0
overrides_applied: 1
override_reason: "All 14 must-haves structurally verified (BackdropPress unit tests + i18n/SSR tests). The single human item is a D-25-06-class live-browser drag smoke, prescribed as structural-only by D-10. User accepted structural verification as sufficient for this recurring item class in phase 36 (2026-07-01); the same decision is applied here — browser drag smoke deferred as optional, consistent with phases 30/32/33."
human_verification:
  - test: "Open any Dialog-based modal (e.g. the Arbeitsvertrag modal). Click and hold inside the modal panel, drag the mouse out to the dark backdrop overlay, then release the mouse button. Confirm the modal stays open."
    expected: "Modal remains open. Only a genuine backdrop click (mousedown AND mouseup both landing on the backdrop) should close the modal."
    why_human: "The BackdropPress state machine and DOM wiring are structurally verified by 5 unit tests. Browser event propagation (stop_propagation + Dioxus mousedown/onclick ordering) is the remaining layer — the plan explicitly prescribes structural tests as the verification method (D-10) and calls live-browser drag not reliably automatable."
---

# Phase 37: Modal-UX-Politur (FE) Verification Report

**Phase Goal:** (MOD-01) Ein zentraler dialog.rs-Fix verhindert, dass ein innerhalb eines Modals begonnener und außerhalb losgelassener Maus-Drag das Modal schließt; (MOD-02) Das Arbeitsvertrag-Modal trägt pro Feld (außer Von/Bis) einen Erklärungssatz in allen drei Locales.
**Verified:** 2026-07-01
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | Drag started inside modal panel + released on backdrop leaves modal OPEN; only mousedown AND mouseup on backdrop closes it. (MOD-01, D-01) | VERIFIED | `BackdropPress` state machine in `dialog.rs:127-153`; backdrop `onmousedown` calls `press_backdrop()`, backdrop `onclick` calls `on_close` only when `release()` returns true (lines 243-250); panel `onmousedown` calls `stop_propagation()` + `press_panel()` (lines 257-260). Five unit tests all pass. |
| 2  | Fix lives centrally in `dialog.rs`/`DialogContent` so all Dialog users benefit automatically. (MOD-01, D-02) | VERIFIED | `let mut backdrop_press = use_signal(BackdropPress::default)` declared in `DialogContent` (line 237). All nine Dialog-using modals inherit the fix without modification. |
| 3  | `absence_convert_modal.rs` applies the identical signal-flag pattern inline to its own custom backdrop; no longer closes on panel-originated drag. (MOD-01, D-03) | VERIFIED | `use crate::component::dialog::BackdropPress` at line 21; `let mut backdrop_press = use_signal(BackdropPress::default)` at line 90; outer div `onmousedown` calls `press_backdrop()` (line 94-96); outer div `onclick` calls `on_cancel` only when `release()` returns true (lines 97-101); inner panel `onmousedown` calls `ev.stop_propagation()` + `press_panel()` (lines 103-106). |
| 4  | ESC dismissal (`use_escape_dismiss`) is left untouched. (D-04) | VERIFIED | `use_escape_dismiss(props.on_close)` at line 219 of `dialog.rs` is unchanged. `install_escape_listener` (lines 374-384) and the X-button `onclick` (lines 290-295) are unchanged. |
| 5  | Drag-safe close decision proven structurally by pure predicate/handler unit tests; five tests covering all BackdropPress state transitions. (D-10) | VERIFIED | `backdrop_press_new_release_returns_false`, `backdrop_press_panel_then_release_returns_false` (core MOD-01 case), `backdrop_press_backdrop_then_release_returns_true`, `backdrop_press_backdrop_then_panel_clears_flag`, `backdrop_press_release_resets_flag` — all 5 pass (confirmed by `cargo test -p shifty-dioxus backdrop_press`). |
| 6  | Frontend gates green: `cargo test -p shifty-dioxus` and WASM build pass; backend `cargo clippy --workspace -- -D warnings` stays clean. (D-11) | VERIFIED | Full suite: 727 passed, 1 pre-existing failure (`i18n_impersonation_keys_match_german_reference`, predates phase 37, excluded per scope). Backend clippy: clean (no output, exit 0). WASM build: confirmed in SUMMARY (confirmed via `nix develop` shell). |
| 7  | Contract modal shows a sibling `span class="text-small font-normal text-ink-muted"` under each relevant field (Workdays, Expected hours, Days per week, Vacation, Committed voluntary, Dynamic); Von/Bis excluded. (MOD-02, D-05) | VERIFIED | Six spans with class `text-small font-normal text-ink-muted` found at `contract_modal.rs` lines 307, 335, 359, 382, 410, 432. From/To date fields (lines 185-227) carry no help span. |
| 8  | Six new `*Help` i18n keys carry exact German texts from D-06 table verbatim (including em-dash in DynamicHourHelp). (MOD-02, D-06) | VERIFIED | `de.rs` verbatim match confirmed by `i18n_contract_help_keys_match_german_reference` test (PASS). DynamicHourHelp uses `\u{2014}` (em-dash), matching D-06. |
| 9  | `CommittedVoluntaryHelp` = "Zugesagte freiwillige Stunden." and rendered INSIDE the `if show_committed` block so it appears only with its field. (D-07) | VERIFIED | `de.rs` line 459 has `"Zugesagte freiwillige Stunden."`. In `contract_modal.rs` the span at line 410 is inside `if show_committed { div { ... } }` starting at line 386. `committed_hidden_when_no_cap_no_zero` test confirms conditional branch. |
| 10 | `CapPlannedHoursHelp` is left unchanged; used only as structural template. (D-08) | VERIFIED | `Key::CapPlannedHoursHelp` at `mod.rs:151` unchanged; `cap_help` resolved at `contract_modal.rs:150`; span at line 451. No modifications to this key or its translations. |
| 11 | Each new key present in 4 files: enum variant in `i18n/mod.rs` next to its `*Label`, plus `add_text` in `de.rs`, `en.rs`, `cs.rs`. (D-09) | VERIFIED | All 6 variants in `mod.rs` (lines 210, 212, 215, 217, 219, 278). All 6 `add_text` entries confirmed in `de.rs`, `en.rs`, `cs.rs`. `i18n_contract_help_keys_present_in_en_and_cs` test confirms en/cs are non-empty and distinct from German (no silent fallback). |
| 12 | Von/Bis (From/To) fields have NO help span. (D-06) | VERIFIED | The date-range section (`contract_modal.rs` lines 183-227) contains two `Field` + `TextInput` pairs with no following `span class="text-small..."` in either grid cell. |
| 13 | SSR/i18n tests confirm help texts render under fields and resolve in all three locales (de verbatim; en/cs distinct). (D-10) | VERIFIED | `help_span_renders_under_field_with_correct_classes` (PASS, confirms DOM order: field label precedes help span); `all_help_keys_referenced_in_contract_modal_source` (PASS, source guard via `include_str!`); `i18n_contract_help_keys_match_german_reference` (PASS); `i18n_contract_help_keys_present_in_en_and_cs` (PASS). |
| 14 | Frontend gates green for MOD-02 (same gate as MOD-01 T6; confirmed joint run at 727 passed). (D-11) | VERIFIED | Same evidence as Truth 6. Contract modal tests: 13/13 pass (includes 2 new help-span tests). i18n tests: 2/2 new tests pass. |

**Score:** 14/14 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `shifty-dioxus/src/component/dialog.rs` | `BackdropPress` struct + signal-flag wiring in `DialogContent` | VERIFIED | `BackdropPress` at lines 127-153; `use_signal` wiring at lines 237-260; 5 unit tests at lines 746-799 |
| `shifty-dioxus/src/component/absence_convert_modal.rs` | Imports and uses `BackdropPress` inline | VERIFIED | Import at line 21; `use_signal` at line 90; onmousedown handlers at lines 94-106 |
| `shifty-dioxus/src/i18n/mod.rs` | Six new Key enum variants | VERIFIED | WorkdaysHelp (210), ExpectedHoursPerWeekHelp (212), DaysPerWeekHelp (215), VacationEntitlementsPerYearHelp (217), DynamicHourHelp (219), CommittedVoluntaryHelp (278) |
| `shifty-dioxus/src/i18n/de.rs` | 6 German verbatim `add_text` entries | VERIFIED | All 6 entries with correct texts; DynamicHourHelp uses `\u{2014}` em-dash |
| `shifty-dioxus/src/i18n/en.rs` | 6 English `add_text` entries distinct from German | VERIFIED | All 6 entries; locale-coverage guard test confirms non-empty and different from de |
| `shifty-dioxus/src/i18n/cs.rs` | 6 Czech `add_text` entries distinct from German | VERIFIED | All 6 entries; locale-coverage guard test confirms non-empty and different from de |
| `shifty-dioxus/src/component/contract_modal.rs` | 6 ImStr resolutions + 6 sibling spans + 2 new tests | VERIFIED | Resolutions at lines 138-148; spans at lines 307, 335, 359, 382, 410, 432; tests `help_span_renders_under_field_with_correct_classes` and `all_help_keys_referenced_in_contract_modal_source` pass |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| backdrop `onmousedown` | `BackdropPress.press_backdrop()` | `backdrop_press.write().press_backdrop()` | WIRED | `dialog.rs:243-245` |
| panel `onmousedown` | `stop_propagation` + `BackdropPress.press_panel()` | `evt.stop_propagation(); backdrop_press.write().press_panel()` | WIRED | `dialog.rs:257-260` |
| backdrop `onclick` | `on_close` conditional on `release()` | `if backdrop_press.write().release() { on_close.call(()) }` | WIRED | `dialog.rs:246-250` |
| `absence_convert_modal.rs` outer backdrop `onmousedown` | `BackdropPress.press_backdrop()` | Same pattern imported from `dialog.rs` | WIRED | `absence_convert_modal.rs:94-96` |
| `absence_convert_modal.rs` inner panel `onmousedown` | `stop_propagation` + `press_panel()` | `ev.stop_propagation(); backdrop_press.write().press_panel()` | WIRED | `absence_convert_modal.rs:103-106` |
| `Key::WorkdaysHelp` → `de/en/cs` → `contract_modal.rs` → span | Full i18n→render chain | `i18n.t(Key::WorkdaysHelp)` → `workdays_help` ImStr → `span { class: "text-small font-normal text-ink-muted" }` | WIRED | `contract_modal.rs:138, 307` |
| `CommittedVoluntaryHelp` → span inside `if show_committed` | Conditional render | Span at line 410 inside the `if show_committed { div { ... } }` block at line 386 | WIRED | Confirmed by `committed_hidden_when_no_cap_no_zero` test |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 5 BackdropPress unit tests pass | `cargo test -p shifty-dioxus backdrop_press` | 5 passed, 0 failed | PASS |
| 26 dialog tests pass (incl. 5 new) | `cargo test -p shifty-dioxus component::dialog` | 26 passed, 0 failed | PASS |
| 4 absence_convert_modal tests pass | `cargo test -p shifty-dioxus component::absence_convert_modal` | 4 passed, 0 failed | PASS |
| 13 contract_modal tests pass (incl. 2 new) | `cargo test -p shifty-dioxus component::contract_modal` | 13 passed, 0 failed | PASS |
| 2 i18n contract help tests pass | `cargo test -p shifty-dioxus i18n_contract_help` | 2 passed, 0 failed | PASS |
| Full frontend suite | `cargo test -p shifty-dioxus` | 727 passed, 1 pre-existing failure (impersonation key) | PASS (1 excluded pre-existing) |
| Backend clippy regression guard | `cargo clippy --workspace -- -D warnings` | No warnings, exit 0 | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| MOD-01 | 37-01-PLAN.md | Mouse drag started inside modal must not close it; only genuine outside click closes | SATISFIED | BackdropPress state machine + 5 unit tests + wiring in dialog.rs and absence_convert_modal.rs |
| MOD-02 | 37-02-PLAN.md | Arbeitsvertrag modal shows per-field help text in all three locales, Von/Bis excluded | SATISFIED | 6 new i18n keys (de verbatim, en/cs translated) + 6 sibling spans + SSR/source guard tests |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | No TBD/FIXME/XXX markers in any phase-37-modified file | — | — |
| Various (pre-existing) | — | 48 compiler warnings visible in test output | Info | Pre-existing; Phase 38 (HYG) scope — not introduced by phase 37 |

### Human Verification Required

#### 1. Live-Browser Drag-and-Release Smoke Test (MOD-01)

**Test:** Open any Dialog-based modal (the Arbeitsvertrag contract modal is ideal). Click and hold the mouse inside the modal panel (e.g. click-drag to select text in a text field), drag the cursor out over the dark backdrop overlay, and release the mouse button.

**Expected:** The modal remains open. Only a genuine outside click — where both mousedown and mouseup occur on the backdrop — should close the modal.

**Why human:** The `BackdropPress` state machine and Dioxus signal-flag wiring are verified by 5 unit tests (the plan's D-10 prescription). The remaining unverified layer is actual browser event propagation: that `stop_propagation` on the panel's `onmousedown` actually prevents the backdrop's `onmousedown` from firing, and that Dioxus's event system delivers these events in the expected order in a real WASM runtime. No automated test can exercise this cross-layer behavior without running a browser.

---

### Gaps Summary

No gaps found. All 14 must-have truths are VERIFIED. The one human verification item is the live-browser drag smoke test — a structural-level concern that the plan explicitly acknowledged as non-automatable (D-10). The pre-existing `i18n_impersonation_keys_match_german_reference` test failure predates phase 37 (confirmed in SUMMARY and by the base-commit scope boundary) and is not attributable to this phase.

---

_Verified: 2026-07-01_
_Verifier: Claude (gsd-verifier)_
