---
phase: 37-modal-ux-politur
plan: "02"
subsystem: ui
tags: [dioxus, i18n, contract-modal, help-text, tdd]

requires:
  - phase: 37-modal-ux-politur-01
    provides: MOD-01 drag-safe backdrop close in Dialog

provides:
  - Six *Help i18n keys (WorkdaysHelp, ExpectedHoursPerWeekHelp, DaysPerWeekHelp, VacationEntitlementsPerYearHelp, DynamicHourHelp, CommittedVoluntaryHelp) across de/en/cs
  - Per-field help spans (text-small font-normal text-ink-muted) in ContractModalBody
  - CommittedVoluntaryHelp scoped inside if show_committed block

affects:
  - contract_modal
  - i18n

tech-stack:
  added: []
  patterns:
    - "Help sibling-span pattern: wrap Field + span in flex flex-col gap-1 div, span carries text-small font-normal text-ink-muted, mirroring existing CapPlannedHoursHelp at contract_modal.rs:427"
    - "i18n Key placement: each *Help variant placed directly after its *Label in i18n/mod.rs enum"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
    - shifty-dioxus/src/component/contract_modal.rs

key-decisions:
  - "D-05: Help rendered as sibling span (text-small font-normal text-ink-muted), not via Field hint prop"
  - "D-06: From/To excluded (self-explanatory); six fields covered"
  - "D-07: CommittedVoluntaryHelp inside if show_committed block — appears only with its field"
  - "D-08: CapPlannedHoursHelp left unchanged, used only as structural template"
  - "D-09: 4 files per key (mod.rs + de/en/cs) — all three locales mandatory"
  - "D-10: i18n-resolution tests (de verbatim + en/cs differ-from-de guard) + source wiring guard cover SSR verification"

patterns-established:
  - "Sibling-span help: Field + help span wrapped in div { class: 'flex flex-col gap-1' }"
  - "Locale-coverage guard test: assert en/cs non-empty AND != German text (catches silent fallback)"
  - "Source-presence guard: include_str! + Key::XHelp lookup before #[cfg(test)] (catches forgotten wiring)"

requirements-completed: [MOD-02]

coverage:
  - id: D1
    description: "Six new *Help i18n keys present in all three locales with correct German verbatim texts"
    requirement: MOD-02
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/i18n/mod.rs#i18n_contract_help_keys_match_german_reference"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/i18n/mod.rs#i18n_contract_help_keys_present_in_en_and_cs"
        status: pass
    human_judgment: false

  - id: D2
    description: "Six help spans render under their fields in ContractModalBody with class text-small font-normal text-ink-muted"
    requirement: MOD-02
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/component/contract_modal.rs#help_span_renders_under_field_with_correct_classes"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/component/contract_modal.rs#all_help_keys_referenced_in_contract_modal_source"
        status: pass
    human_judgment: false

  - id: D3
    description: "CommittedVoluntaryHelp appears only inside the if show_committed block"
    requirement: MOD-02
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/component/contract_modal.rs#all_help_keys_referenced_in_contract_modal_source"
        status: pass
    human_judgment: true
    rationale: "Source guard confirms key is wired; conditional render requires visual verification that the span hides when show_committed=false. Automated SSR of the full modal requires coroutine/global store context that tests deliberately avoid."

  - id: D4
    description: "From/To fields have no help span (D-06)"
    requirement: MOD-02
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/component/contract_modal.rs#all_help_keys_referenced_in_contract_modal_source"
        status: pass
    human_judgment: false

  - id: D5
    description: "All pre-existing contract_modal and i18n tests continue to pass; no new failures beyond pre-existing impersonation failure"
    requirement: MOD-02
    verification:
      - kind: unit
        ref: "cargo test -p shifty-dioxus — 727 passed, 1 pre-existing failure"
        status: pass
      - kind: unit
        ref: "cargo build --target wasm32-unknown-unknown — WASM build OK"
        status: pass
    human_judgment: false

duration: 35min
completed: 2026-07-01
status: complete
---

# Phase 37 Plan 02: Contract-Modal Help Texts (MOD-02) Summary

**Six per-field help spans (text-small font-normal text-ink-muted) added to the Arbeitsvertrag modal via 6 new *Help i18n keys (de verbatim, en/cs translated) across all three locales, mirroring the existing CapPlannedHoursHelp pattern**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-07-01T00:00:00Z
- **Completed:** 2026-07-01
- **Tasks:** 2 (each with RED + GREEN commits)
- **Files modified:** 5

## Accomplishments

- Added 6 Key enum variants (`WorkdaysHelp`, `ExpectedHoursPerWeekHelp`, `DaysPerWeekHelp`, `VacationEntitlementsPerYearHelp`, `DynamicHourHelp`, `CommittedVoluntaryHelp`) to `i18n/mod.rs`, each placed next to its corresponding `*Label`
- Added 18 `add_text` entries (6 keys x de/en/cs) with German texts verbatim from D-06/D-07 (including em-dash in DynamicHourHelp) and faithful English/Czech translations
- Rendered six sibling `<span class="text-small font-normal text-ink-muted">` spans in `ContractModalBody`, one under each relevant field; From/To excluded (D-06)
- CommittedVoluntaryHelp wired INSIDE the `if show_committed` block so it only appears with its field (D-07)
- Dynamic field wrapped in `flex flex-col gap-1` mirroring the existing cap help block exactly
- Added 4 new tests: 2 i18n-resolution tests (de verbatim + en/cs locale-coverage guard) + 2 contract_modal tests (help span render + source wiring guard)

## Task Commits

TDD tasks with RED → GREEN commits:

**Task 1: Add six *Help i18n keys across mod.rs + all three locales**
1. `6bd7ccb` — `test(37-02)`: add failing i18n tests for contract help keys (RED)
2. `83a0d91` — `feat(37-02)`: add six *Help i18n keys across all three locales (GREEN)

**Task 2: Render the six help spans under their fields in the contract modal**
3. `c778225` — `test(37-02)`: add failing help-key wiring guard for contract modal (RED)
4. `dd94fe7` — `feat(37-02)`: render six help spans under contract modal fields (GREEN)

## Files Created/Modified

- `shifty-dioxus/src/i18n/mod.rs` — 6 new Key enum variants + 2 i18n tests (RED+GREEN)
- `shifty-dioxus/src/i18n/de.rs` — 6 new add_text entries with German verbatim texts
- `shifty-dioxus/src/i18n/en.rs` — 6 new add_text entries with English translations
- `shifty-dioxus/src/i18n/cs.rs` — 6 new add_text entries (incl. DynamicHourHelp, which has no DynamicHourLabel in cs.rs — help placed near vacation section)
- `shifty-dioxus/src/component/contract_modal.rs` — 6 help ImStr resolutions + 6 sibling spans + 2 new tests

## Decisions Made

- D-05: Sibling-span approach (not Field hint prop) — consistent with existing CapPlannedHoursHelp
- D-07: CommittedVoluntaryHelp scoped inside `if show_committed` block per user decision
- D-08: CapPlannedHoursHelp unchanged, used only as structural template
- D-09: 4 files per key (mod.rs + de/en/cs) — all three locales mandatory to prevent silent German fallback

## Deviations from Plan

None — plan executed exactly as written. All verbatim German texts match D-06/D-07 table exactly.

### Observation (not a deviation)

`DynamicHourLabel` is absent from `cs.rs` (pre-existing gap). `DynamicHourHelp` was still added to cs.rs near the vacation section (the closest logical placement within the work-details block). This is not a fix — the pre-existing label gap is out of scope and left for deferred-items.

## Issues Encountered

WASM build required `nix develop` shell (lld linker needed); build succeeded cleanly once run in the correct shell environment.

## Known Stubs

None — all six help texts are wired to real i18n keys resolving runtime locale strings.

## Threat Flags

None — only static, localized help copy rendered client-side. No new network endpoints, no user input, no persisted data (T-37-02: accepted as low severity).

## TDD Gate Compliance

Both tasks followed RED → GREEN:
- Task 1: `test(37-02)` commit (RED) → `feat(37-02)` commit (GREEN)
- Task 2: `test(37-02)` commit (RED) → `feat(37-02)` commit (GREEN)

## Next Phase Readiness

Phase 37 MOD-02 complete. Phase 37 (both MOD-01 and MOD-02) is now done. Ready for Phase 38 (HYG build-hygiene per Roadmap).

---
*Phase: 37-modal-ux-politur*
*Completed: 2026-07-01*
