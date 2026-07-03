---
phase: 46-backend-hygiene-i18n
plan: 02
subsystem: shifty-dioxus/i18n + shifty-dioxus/page/shiftplan
tags: [i18n, hyg-04, phase-46, dropdown, locale-parity]
status: complete
requires:
  - 46-01
provides:
  - HYG-04
affects:
  - shifty-dioxus/src/i18n/mod.rs (Key enum + 3 variants)
  - shifty-dioxus/src/i18n/en.rs (3 add_text calls)
  - shifty-dioxus/src/i18n/de.rs (3 add_text calls)
  - shifty-dioxus/src/i18n/cs.rs (3 add_text calls)
  - shifty-dioxus/src/page/shiftplan.rs (DropdownTrigger entries block, i18n-bound)
tech-stack:
  added: []
  patterns:
    - i18n-key-add-per-locale (mod.rs enum + add_text in en/de/cs)
    - ImStr::from(i18n.t(Key::…).as_ref()) for dropdown-entry text (matches week_status_dropdown.rs precedent)
key-files:
  created: []
  modified:
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/cs.rs
    - shifty-dioxus/src/page/shiftplan.rs
decisions:
  - "HYG-04 D-CONTEXT-Q2: no presence test — user explicitly said 'brauchst du echt nicht testen', only the three add_text mappings + call-site swap"
  - "Copy: En preserved from existing literals; De = 'Struktur bearbeiten' / 'Normalansicht' / 'Neuer Slot'; Cs = 'Upravit strukturu' / 'Normální zobrazení' / 'Nový slot'"
  - "Type-choice at call-site: ImStr::from(i18n.t(Key::…).as_ref()) — chosen over .to_string() because the DropdownEntry From-impl accepts ImStr directly, matching week_status_dropdown.rs precedent"
metrics:
  duration: ~10 min
  completed: 2026-07-02
requirements: [HYG-04]
---

# Phase 46 Plan 02: HYG-04 — Schichtplan-Struktur-Dropdown i18n-Bindung

One-liner: Replace three hardcoded English string literals (`"Edit structure"`, `"Normal mode"`, `"New slot"`) in the Schichtplan header DropdownTrigger with three new `Key::Shiftplan…` variants translated across En/De/Cs — so De- and Cs-users no longer see English literals in the structure-mode dropdown.

## Result

Plan executed exactly as written. Two tasks, five files, no deviations.

- **Enum:** three new variants (`ShiftplanEditStructure`, `ShiftplanNormalMode`, `ShiftplanNewSlot`) appended after `WeekStatusChangeAriaLabel` in `mod.rs`.
- **Locales:** three `add_text` calls added at the tail of each of `en.rs` / `de.rs` / `cs.rs`. Copy per Locale as specified in the plan.
- **Call-site:** in `page/shiftplan.rs` inside `if is_shiftplanner { DropdownTrigger { entries: [ … ] } }` (lines ~1161–1194), the three string literals were swapped to `ImStr::from(i18n.t(Key::…).as_ref())`. `let`-bindings did not work inside RSX macro scope, so the conversions were inlined in the tuple positions themselves.
- **No presence test** added (per CONTEXT.md HYG-04 Q2 user decision).

## Deviations from Plan

None — plan executed exactly as written.

The plan mentioned that `as_ref()` vs `to_string()` may need to be chosen based on the compiler error. Neither was accepted directly by the tuple-`From` impls (which want `&'static str` or `ImStr`, not `Rc<str>`), so the working form is `ImStr::from(i18n.t(Key::…).as_ref())`. This matches the pre-existing pattern in `shifty-dioxus/src/component/week_status_dropdown.rs:76` and does not require any new API. Not a deviation from the plan's intent — the plan explicitly said "the executor picks the variant that compiles"; the third variant (`ImStr::from`) was the one that compiled.

## Verification

### Automated gates (all green)

| Gate                                             | Result                             |
| ------------------------------------------------ | ---------------------------------- |
| `cargo check -p shifty-dioxus`                   | ok (Finished in 0.17s incremental) |
| `cargo build --target wasm32-unknown-unknown`    | ok (Finished in 30.58s)            |
| `cargo clippy -p shifty-dioxus -- -D warnings`   | ok (0 warnings)                    |
| `cargo test -p shifty-dioxus`                    | ok (778 passed, 0 failed)          |
| `cargo clippy --workspace -- -D warnings` (BE)   | ok (unchanged)                     |

### Grep verification

```
mod.rs enum:     3 hits of ShiftplanEditStructure|ShiftplanNormalMode|ShiftplanNewSlot
de.rs:           3 hits
en.rs:           3 hits
cs.rs:           3 hits
shiftplan.rs:    3 hits (call-site)
Residual hardcoded literals in shiftplan.rs (excluding comments): 0
```

Global scan `grep -rn '"Edit structure"|"Normal mode"|"New slot"' shifty-dioxus/src/` shows only the three expected En `add_text` lines. No stale literals anywhere else.

## Success Criteria

1. ✅ Three literals no longer rendered as hardcoded English — dropdown reads from `i18n.t(Key::…)`.
2. ✅ De-users see „Struktur bearbeiten" / „Normalansicht" / „Neuer Slot"; Cs-users see „Upravit strukturu" / „Normální zobrazení" / „Nový slot"; En-users keep original copy.
3. ✅ FE-WASM-Build + FE-Clippy `-D warnings` + FE-Test-Suite green (778/0). Backend clippy unchanged.

## Self-Check: PASSED

- File `.planning/phases/46-backend-hygiene-i18n/46-02-SUMMARY.md` — FOUND (this file).
- Enum variants present in `mod.rs`: FOUND (3/3).
- add_text calls in `en.rs`/`de.rs`/`cs.rs`: FOUND (3/3/3).
- Call-site in `page/shiftplan.rs`: FOUND (3 Key references, 0 residual literals excluding comments).
- All automated gates: PASSED.
