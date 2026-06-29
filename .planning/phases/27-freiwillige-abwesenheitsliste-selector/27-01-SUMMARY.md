---
phase: 27-freiwillige-abwesenheitsliste-selector
plan: 01
subsystem: ui
tags: [dioxus, wasm, i18n, absences, optgroup, sales-person]

# Dependency graph
requires:
  - phase: 26-volunteer-absence
    provides: backend support for volunteer (unpaid) absences (VFA) + full sales_persons list loaded on the absences page
  - phase: 17-employee-work-details
    provides: EmployeeWorkDetails / volunteer hour handling
provides:
  - "PersonGroup enum (Employees | Volunteers) + pure grouped_selectable() partitioning the loaded sales_persons into employees-first / volunteers groups (active-only, empty groups omitted)"
  - "Shared grouped_person_options() RSX helper rendering two native <optgroup>s, consumed by BOTH the AbsenceModal person dropdown and the AbsenceFilterBar HR person filter"
  - "Two i18n keys AbsenceGroupEmployees / AbsenceGroupVolunteers translated in en/de/cs"
affects: [absences, vacation-overview, phase-28-urlaubsanspruch]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pure grouping function + thin RSX passthrough helper feeding two call-sites (no copy-paste) — native optgroup passes through the unchanged SelectInput as children"
    - "Grouping uses its OWN !inactive predicate, intentionally decoupled from is_selectable_employee/selectable_balances (paid-only stays paid-only)"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/page/absences.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/cs.rs

key-decisions:
  - "D-27-01: Employees optgroup rendered before Volunteers; both call-sites share one helper"
  - "D-27-02: is_selectable_employee (is_paid && !inactive) and selectable_balances left UNCHANGED — grouping uses its own predicate; HR VacationPerPersonList stays paid-only"
  - "D-27-03: grouped_selectable omits empty groups so no empty optgroup is ever rendered"
  - "D-27-04: two new i18n keys; de.rs uses Locale::De (not the historical Locale::En bug)"
  - "D-27-05: category dropdown and SelectInput unchanged — volunteers get the same categories"
  - "D-27-06: inactive persons land in neither group; FilterBar 'Alle' option preserved before the groups (parse-fail → None keeps 'Alle' selected)"

patterns-established:
  - "Pattern: pure partition fn returns only non-empty (PersonGroup, Vec<&SalesPerson>) groups → RSX helper maps each to a localized <optgroup>, so empty-group suppression lives in the pure layer and is unit-tested"

requirements-completed: [VOL-SEL-01]

coverage:
  - id: D1
    description: "grouped_selectable partitions active-paid → Employees, active-unpaid → Volunteers, omits inactive, employees-first, empty groups dropped, load order preserved"
    requirement: "VOL-SEL-01"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/page/absences.rs#grouped_selectable_partitions_active_paid_and_unpaid, #grouped_selectable_orders_employees_before_volunteers, #grouped_selectable_omits_empty_volunteers_group, #grouped_selectable_omits_empty_employees_group, #grouped_selectable_preserves_order_within_group"
        status: pass
    human_judgment: false
  - id: D2
    description: "is_selectable_employee + selectable_balances unchanged; HR vacation overview stays paid-only (D-27-02)"
    requirement: "VOL-SEL-01"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/page/absences.rs selectable_* tests (4 is_selectable_employee + 5 selectable_balances, unchanged)"
        status: pass
      - kind: other
        ref: "grep -nF 'sales_person.is_paid && !sales_person.inactive' shifty-dioxus/src/page/absences.rs (matches exactly once)"
        status: pass
    human_judgment: false
  - id: D3
    description: "AbsenceGroupEmployees / AbsenceGroupVolunteers resolve non-empty (never '??') in en/de/cs; de.rs uses Locale::De"
    requirement: "VOL-SEL-01"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/i18n/mod.rs#i18n_absence_keys_present_in_all_locales (extended with both new keys)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Volunteers grouped/selectable in both selectors with a working backend create-path roundtrip and localized labels (browser smoke)"
    requirement: "VOL-SEL-01"
    verification:
      - kind: e2e
        ref: "Task 3 checkpoint browser smoke — left to orchestrator/human"
        status: unknown
    human_judgment: true
    rationale: "Requires a running backend+frontend, an active unpaid sales person, and human visual confirmation of the optgroup ordering and the create-path roundtrip (create-path ≠ edit-path). Not automatable here."

# Metrics
duration: 11min
completed: 2026-06-29
status: complete
---

# Phase 27 Plan 01: Freiwillige in Abwesenheitsliste auswählbar Summary

**Active volunteers (unpaid sales persons) are now selectable in both absence person selectors, grouped under a localized "Freiwillige" optgroup below "Angestellte", via one shared pure-grouping + RSX helper — without touching the paid-only HR vacation overview.**

## Performance

- **Duration:** ~11 min
- **Started:** 2026-06-29T05:19:22Z
- **Completed:** 2026-06-29T05:30:21Z
- **Tasks:** 2 of 3 (Task 3 is a human-verify checkpoint — automated gates run, browser smoke deferred to orchestrator)
- **Files modified:** 5

## Accomplishments
- Added `PersonGroup` enum + pure `grouped_selectable(&[SalesPerson]) -> Vec<(PersonGroup, Vec<&SalesPerson>)>` that partitions active-paid → Employees, active-unpaid → Volunteers, drops inactive, orders Employees-first, and omits empty groups (5 new unit tests, all green).
- Added the shared `grouped_person_options(&[SalesPerson], Option<Uuid>, &I18nType) -> Element` RSX helper rendering two native `<optgroup>`s; rewired BOTH the AbsenceModal person dropdown and the AbsenceFilterBar HR person filter to call it (no copy-paste). FilterBar "Alle" option preserved before the groups.
- Added i18n keys `AbsenceGroupEmployees` / `AbsenceGroupVolunteers` in en ("Employees"/"Volunteers"), de ("Angestellte"/"Freiwillige", using `Locale::De`), cs ("Zaměstnanci"/"Dobrovolníci"); extended the absence-domain coverage test with both keys.
- `is_selectable_employee` and `selectable_balances` left untouched (grep guard matches exactly once); the 4 is_selectable_employee + 5 selectable_balances tests stay green — HR vacation overview remains paid-only (D-27-02).

## Task Commits

Atomic jj changes (one per task; Task 3 checkpoint not committed):

1. **Task 1: Add the two grouping i18n keys in all three locales** — `a63943f1` (feat)
2. **Task 2: Pure grouping logic + RSX helper, rewire both selectors** — `2db3c522` (feat; TDD RED→GREEN done in one change)

_Task 3 (browser smoke, human-verify checkpoint) is intentionally NOT committed — left to the orchestrator/user per the jj-only / checkpoint protocol._

## Files Created/Modified
- `shifty-dioxus/src/page/absences.rs` — `PersonGroup` enum, pure `grouped_selectable`, `grouped_person_options` RSX helper, both selectors rewired, 5 new pure-function tests, `I18nType` import added.
- `shifty-dioxus/src/i18n/mod.rs` — two new `Key` variants + coverage-test extension.
- `shifty-dioxus/src/i18n/en.rs`, `de.rs`, `cs.rs` — translations for both new keys (de uses `Locale::De`).

## Decisions Made
None beyond the plan — D-27-01..06 followed as specified. Czech wording chosen per D-27-04 discretion: "Zaměstnanci" / "Dobrovolníci" (common professional translation).

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
- **WASM build linker (`lld`) absent in the interactive shell.** The current dev shell (backend `devShells.default`, nixpkgs `rustc`/`cargo`) bundles no `rust-lld` and has no `lld`/`wasm-ld` on PATH, so `cargo build --target wasm32-unknown-unknown` failed at the final link step (codegen fully completed — 46 pre-existing warnings emitted, code error-free). Resolved by running the WASM build through the frontend flake's `devShells.default` (`nix develop` in `shifty-dioxus/`), which provides the rust-overlay `rustToolchain` with the wasm32 target + bundled rust-lld. Build then finished clean (`Finished dev profile in 1m08s`, only pre-existing warnings). No code change was required — purely an environment/toolchain-shell discrepancy versus the spawn note.

## Gate Results
- **WASM build** (`cargo build --target wasm32-unknown-unknown`, via frontend `nix develop`): PASS — finished clean, only pre-existing warnings (e.g. `has_sunday_slots` never used).
- **`cargo test`** (host shell, full FE suite): PASS — 677 passed, 0 failed. Includes the 5 new `grouped_selectable_*` tests, the 4 unchanged `is_selectable_employee` tests, the 5 unchanged `selectable_balances` tests, and the absence i18n coverage test (extended with the 2 new keys).
- **Clippy** (soft for this FE workspace, run from host/backend shell): 207 total warnings — consistent with the ~198 pre-existing baseline; grep for the new identifiers (`grouped_selectable`, `grouped_person_options`, `PersonGroup`, `AbsenceGroup*`) returns ZERO clippy hits. No NEW warnings introduced by the changed files. Pre-existing lints elsewhere are not a blocker (workspace excluded from CI clippy).
- **D-27-02 guard** (`grep -nF 'sales_person.is_paid && !sales_person.inactive' shifty-dioxus/src/page/absences.rs`): matches exactly once (line 116) — shared predicate intact.

## Next Phase Readiness
- Code-complete and all automated gates green. **Task 3 browser smoke (backend roundtrip: create-path volunteer absence in the Modal + filter in the HR FilterBar + de/cs label check) is left to the orchestrator/human** per the human-verify checkpoint.
- Phase 28 (Urlaubsanspruch / volunteers in the vacation overview) is the intended follow-up — this phase deliberately did NOT loosen the paid-only vacation list.

## Self-Check: PASSED
- All 5 modified source files present.
- SUMMARY.md present.
- Both task commits present in jj history: `a63943f1` (Task 1), `2db3c522` (Task 2).
- Task 3 (human-verify checkpoint) intentionally uncommitted — browser smoke pending.

---
*Phase: 27-freiwillige-abwesenheitsliste-selector*
*Completed: 2026-06-29*
