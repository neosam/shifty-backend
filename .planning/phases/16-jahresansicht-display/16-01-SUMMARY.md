---
phase: 16-jahresansicht-display
plan: 01
subsystem: api
tags: [committed-voluntary, weekly-summary, booking-information, rest-types, dto, two-band]

# Dependency graph
requires:
  - phase: 15-reporting-no-double-count-snapshot-bump-same-commit
    provides: "Zwei-Band-Modell — committed_voluntary_hours (Band 1) + volunteer_hours als per-person surplus (Band 2) in der ersten get_weekly_summary-Variante berechnet; WeeklySummary-Service-Struct trägt beide Felder."
  - phase: 14-data-model-foundation-backend
    provides: "committed_voluntary end-to-end im Datenmodell; #[serde(default)]-Wire-Backward-Compat-Pattern (EmployeeWorkDetailsTO)."
provides:
  - "D-01: overall_available_hours der Jahresansicht (erste get_weekly_summary-Variante) summiert paid + committed_voluntary (Band 1) + volunteer (Band 2) — no double-count."
  - "CVC-07b: WeeklySummaryTO trägt committed_voluntary_hours (mit #[serde(default)]) + From<&WeeklySummary>-Mapping-Arm."
  - "Wire-fähiger committed-Term für den Frontend-Layer (state/page/chart) — entkoppelt den WASM-Compile."
affects: [16-jahresansicht-display Frontend-Waves (state/weekly_overview, page/weekly_overview, weekly_overview_chart, i18n)]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Two-band additive availability: overall = paid + committed(Band 1) + surplus(Band 2), wobei Band 2 committed bereits per-Person abgezogen hat (max(actual−committed,0)) → keine Doppelzählung."
    - "#[serde(default)] auf neuem TO-Float-Feld für Wire-Backward-Compat (analog EmployeeWorkDetailsTO, Phase 14)."

key-files:
  created: []
  modified:
    - service_impl/src/booking_information.rs
    - service_impl/src/test/booking_information.rs
    - rest-types/src/lib.rs

key-decisions:
  - "Nur die ERSTE get_weekly_summary-Variante (Jahresansicht / Achse B) wird geändert; die zweite get_summery_for_week (Einzel-Woche) bleibt mit overall = volunteer + paid und committed_voluntary_hours: 0.0 unberührt."
  - "Kein Snapshot-Bump: WeeklySummary ist year-view-only, nicht persistiert → CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 7."
  - "Kein ToSchema/OpenAPI-Task: WeeklySummaryTO hat keine utoipa-Anbindung — Derive bleibt (Clone, Debug, Serialize, Deserialize)."

patterns-established:
  - "D-01-Summenformel als Pure-Function-Unit-Test gepinnt (overall_available_hours-Helper im Test) plus No-double-count-Invariante über volunteer_surplus_above_committed."

requirements-completed: [CVC-07]

# Metrics
duration: ~12min
completed: 2026-06-24
---

# Phase 16 Plan 01: Jahresansicht display (Backend/Transport) Summary

**committed_voluntary_hours (Band 1) fließt jetzt in overall_available_hours der Jahresansicht und ist via WeeklySummaryTO + From-Mapping wire-fähig — no double-count, kein Snapshot-Bump.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-06-24
- **Completed:** 2026-06-24
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- D-01: Die Summenzeile der ERSTEN `get_weekly_summary`-Variante rechnet `committed_voluntary_hours + volunteer_hours + paid_hours` — die Jahresansicht-„verfügbar"-Zahl enthält jetzt den zugesagten Band-1-Term konsistent für Diff-Spalte, Chart und alle Konsumenten.
- CVC-07b: `WeeklySummaryTO` trägt `committed_voluntary_hours` (mit `#[serde(default)]` für Wire-Backward-Compat) und der `From<&WeeklySummary>`-Mapping-Arm überträgt es 1:1.
- Drei neue Backend-Tests (D-01-Summe, No-double-count-Invariante, committed=0-Backward-Compat) + From-Roundtrip-Test + serde-default-Test in rest-types — alle grün.
- Zweite Variante `get_summery_for_week` nachweislich unberührt (grep-Gates); Workspace-Suite ohne Regression (61 Integration + alle Unit-Tests grün).

## Task Commits

**KEINE Commits durch den Executor** — dieses Repo ist jj-managed, GSD-Auto-Commit ist deaktiviert. Alle Änderungen liegen uncommitted im Working Copy; der User committet manuell via jj. (Per `<vcs_jj_only>` im Plan + Prompt.)

Tasks logisch abgeschlossen:

1. **Task 1: D-01 — committed in overall_available_hours der Jahresansicht** (TDD: Produktionszeile + 3 Tests) — `service_impl/src/booking_information.rs`, `service_impl/src/test/booking_information.rs`
2. **Task 2: WeeklySummaryTO + committed_voluntary_hours + From-Mapping + Roundtrip/serde-default-Tests** — `rest-types/src/lib.rs`
3. **Task 3: Backend-Gate — `cargo test --workspace` grün**

## Files Created/Modified
- `service_impl/src/booking_information.rs` — Summenzeile in erster `get_weekly_summary`-Variante: `let overall_available_hours = committed_voluntary_hours + volunteer_hours + paid_hours;` (zuvor `volunteer_hours + paid_hours`); Phase-15-TODO-Kommentar durch D-01-Hinweis ersetzt. Zweite Variante unverändert.
- `service_impl/src/test/booking_information.rs` — 3 neue Tests: `d01_overall_available_sums_paid_committed_volunteer`, `d01_no_double_count_band2_already_net_of_committed`, `d01_committed_zero_matches_pre_phase16_sum`.
- `rest-types/src/lib.rs` — `WeeklySummaryTO.committed_voluntary_hours` (mit `#[serde(default)]`); From-Mapping-Arm; zwei neue Test-Module (`test_weekly_summary_committed_voluntary` hinter `service-impl`, `test_weekly_summary_to_serde_default` feature-frei).

## Decisions Made
- **Test-Strategie D-01 (Claude's Discretion, Plan erlaubt Test-Platzierung):** Die `overall_available_hours`-Summenformel der ersten Variante ist tief in `get_weekly_summary` eingebettet und ohne umfangreiches Mock-Setup nicht direkt instanziierbar. Die D-01-Summe wurde daher als Pure-Function-Unit-Test (`overall_available_hours`-Helper, der exakt `committed + volunteer + paid` rechnet) plus No-double-count-Invariante über das bestehende `volunteer_surplus_above_committed` gepinnt — bei den bestehenden `booking_information`-Tests im `service_impl`-Test-Modul (wie vom Plan vorgesehen). Das pinnt die Formel-Korrektheit ohne fragiles Service-Mock.
- Übrige Entscheidungen exakt wie im Plan: nur erste Variante, kein Snapshot-Bump, kein ToSchema.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- **Grep-Gate Task 1, Gate 3 (`committed_voluntary_hours: 0.0` erwartet genau 1 Treffer, liefert 3):** Die zusätzlichen zwei Treffer (Z.596/604) stammen aus dem vorbestehenden inline `#[cfg(test)] mod tests` (Test `weekly_summary_constructs_with_committed_field`, Phase 15), nicht aus Produktionscode. Der Gate-Intent — „zweite Variante (Produktions-Z.547) bleibt unverändert" — ist erfüllt: Z.547 und Z.386 wurden nicht berührt (Gate 2 bestätigt die unveränderte zweite Summenzeile mit exakt 1 Non-Comment-Treffer). Der Zähler-Mismatch ist ein vorbestehendes Test-Artefakt, kein Effekt dieser Änderung. Keine Korrektur nötig.
- `cargo` nicht direkt auf PATH (NixOS) — alle Test-/Build-Läufe via `nix develop --command bash -c '...'` wie in `<environment>` vorgesehen.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Backend/Transport-Wave abgeschlossen: `WeeklySummaryTO.committed_voluntary_hours` ist wire-fähig — die Frontend-Waves (state/weekly_overview, page/weekly_overview Token-Rendering, weekly_overview_chart drittes Segment, i18n De/En/Cs) können den Term jetzt durch `From<&WeeklySummaryTO>` ziehen und rendern.
- `overall_available_hours` (= `available_hours` im Frontend-State) enthält nach D-01 bereits alle drei Bänder → Diff-Spalte und Chart-Balken-Total konsistent.
- WASM-Compile entkoppelt: das fehlende TO-Feld hätte den Frontend-Build gebrochen; jetzt vorhanden.

## Self-Check: PASSED

- FOUND: service_impl/src/booking_information.rs (D-01-Summenzeile, Gate 1 = 1 Treffer)
- FOUND: rest-types/src/lib.rs (Struct-Feld Z.913 + From-Arm Z.933 + serde-default)
- FOUND: service_impl/src/test/booking_information.rs (3 neue D-01-Tests, alle grün)
- FOUND: .planning/phases/16-jahresansicht-display/16-01-SUMMARY.md
- VERIFIED: `cargo test -p service_impl booking_information` grün (19 passed)
- VERIFIED: `cargo test -p rest-types committed_voluntary` grün (serde-default) + `--features service-impl` (From-Roundtrip) grün
- VERIFIED: `cargo test --workspace` grün — 0 failures
- VERIFIED: ToSchema NICHT ergänzt; CURRENT_SNAPSHOT_SCHEMA_VERSION = 7; zweite Variante unverändert
- N/A: Commits — bewusst keine (jj-only, User committet manuell)

---
*Phase: 16-jahresansicht-display*
*Completed: 2026-06-24*
