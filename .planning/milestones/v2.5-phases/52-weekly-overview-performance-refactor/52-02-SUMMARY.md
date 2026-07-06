---
phase: 52-weekly-overview-performance-refactor
plan: 02
subsystem: [backend, reporting, refactor]
tags: [refactor, extract-method, single-source-of-truth, byte-identity]
requires: ["52-01"]
provides:
  - "service_impl::reporting::ReportingServiceImpl::assemble_weeks (pub(crate) async fn) — single source of truth for per-week aggregation semantics"
  - "get_week umgeschrieben als dünner Wrapper — Trait-Signatur unverändert"
affects:
  - service_impl/src/reporting.rs
tech_stack_added: []
patterns_used:
  - "Extract-Method (Fowler) mit Slice-Referenz-Input für polymorphen Batch-Support"
  - "Inherent-impl-Helper (pub(crate), nicht Trait-Method) — Erweiterung ohne Public-API-Break"
key_files_created: []
key_files_modified:
  - service_impl/src/reporting.rs
decisions:
  - "D-52-08 umgesetzt — async Signatur (RESEARCH Q2 überschreibt CONTEXT-fn-Vereinfachung)"
  - "D-52-09 strikt eingehalten — Balance-Formel, CVC-06 Cap-Gating pro (sales_person, week), ExtraHours-Kategorien-Split, is_paid-Filter (reporting.rs:pre-refactor 1140-1142) alle 1:1 in Helper"
  - "D-52-09 Chain-C-Toggle bleibt draußen — 0 `shortday_gate`-Referenzen im Non-Comment-Code des Files"
  - "D-52-10 — Helper ist pub(crate), get_week bleibt Public-Trait-Methode mit unveränderter Signatur"
  - "Kein neuer Transaction-Envelope in get_week (use_transaction/commit) — der pre-refactor Body hatte keinen, das würde Mock-Test-Erwartungen brechen (reporting_additive_merge tests stubben `commit` nicht)"
metrics:
  duration_minutes: 15
  completed: 2026-07-05
  tasks_completed: 1
  files_created: 0
  files_modified: 1
status: complete
---

# Phase 52 Plan 02: `assemble_weeks`-Helper-Extraktion Summary

**One-liner:** Reiner Extract-Method-Refactor — der `get_week`-Body wandert 1:1
in `pub(crate) async fn assemble_weeks(weeks: &[(u32, u8)], ...)`, `get_week`
wird auf einen 3-Bulk-Load + 1-Element-Vec-Delegation reduziert. Alle 8
Wave-1-Fixtures und die vollständige Reporting-Suite bleiben byte-identisch grün.

## Was gebaut wurde

### Task 1 — `assemble_weeks`-Helper + `get_week`-Delegation

**Neuer Helper** in `impl<Deps: ReportingServiceDeps> ReportingServiceImpl<Deps>`
(inherent impl, direkt hinter `build_derived_holiday_map`):

```rust
#[allow(clippy::too_many_arguments)]
pub(crate) async fn assemble_weeks(
    &self,
    weeks: &[(u32, u8)],
    work_details: &[EmployeeWorkDetails],
    shiftplan_reports: &[ShiftplanReportDay],
    extra_hours: &[ExtraHours],
    context: Authentication<Deps::Context>,
    tx: Option<Deps::Transaction>,
) -> Result<Vec<(u8, Arc<[ShortEmployeeReport]>)>, ServiceError>
```

Iteriert `for &(year, week) in weeks`, baut pro Woche die drei HashMap-Buckets
aus den Slice-Referenzen (per-Person via `collect_to_hash_map_by`, gefiltert per
`(year == year && week == week)` für Shiftplan-Reports und ExtraHours) und
führt den kompletten Per-Person-Aggregations-Body 1:1 aus:

- `find_working_hours_for_calendar_week` (pure Funktion, unverändert)
- `apply_weekly_cap` pro Person pro Woche mit `raw_shiftplan_hours` + `expected_hours_for_cap`
- ExtraHours-Split nach Verfügbarkeit + Kategorie (Vacation/SickLeave/Holiday/…)
- `absence_service.derive_hours_for_range` (async DAO-Call, pro Person pro Woche)
- `build_derived_holiday_map` (async, pro Person pro Woche)
- `sales_person_service.get(sales_person_id, Authentication::Full, tx.clone())`
- `is_paid`-Filter (`continue`) — bleibt **exakt an derselben Position** (T-52-03 Mitigation)
- `ShortEmployeeReport`-Push

Rückgabewert: `Vec<(u8, Arc<[ShortEmployeeReport]>)>` in Eingabe-Reihenfolge.

**`get_week`-Rewrite:**

```rust
async fn get_week(&self, year, week, context, tx)
    -> Result<Arc<[ShortEmployeeReport]>, ServiceError>
{
    let work_details = self.employee_work_details_service
        .all_for_week(week, year, context.clone(), tx.clone()).await?;
    let shiftplan_report = self.shiftplan_report_service
        .extract_shiftplan_report_for_week(year, week, Authentication::Full, tx.clone()).await?;
    let extra_hours = self.extra_hours_service
        .find_by_week(year, week, Authentication::Full, tx.clone()).await?;
    info!("Extra hours: {:?}", &extra_hours);

    let mut assembled = self.assemble_weeks(
        &[(year, week)], &work_details, &shiftplan_report, &extra_hours,
        context, tx,
    ).await?;

    Ok(assembled.pop()
        .map(|(_, reports)| reports)
        .unwrap_or_else(|| Arc::from(Vec::<ShortEmployeeReport>::new())))
}
```

Trait-Signatur `service::reporting::ReportingService::get_week`
(`service/src/reporting.rs:397-403`) **unverändert**.

## Verifikations-Gates

| Gate | Erwartet | Ist | Status |
| ---- | -------- | --- | ------ |
| `cargo test --package service_impl --lib booking_information_weekly_summary_year_batch` | 8 passed | 8 passed | ✅ **Byte-identisch grün** |
| `cargo test --package service_impl --lib reporting` | alle passed | 77 passed | ✅ |
| `cargo test --workspace` | alle passed | 703 + 64 + 11 + Fixtures = alle grün | ✅ |
| `cargo clippy --workspace -- -D warnings` | clean | 0 warnings | ✅ |
| `grep -v '^#\|//' service_impl/src/reporting.rs \| grep -c 'shortday_gate'` | 0 | 0 | ✅ (nur 1 Doc-Kommentar in `assemble_weeks`) |
| `git diff service/src/reporting.rs` | leer | leer | ✅ (Trait unverändert) |

**„Wave-1-Fixtures bit-exakt grün — Semantik-Erhalt strukturell nachgewiesen."**

Die 8 golden snapshots aus Wave 1 pinnen inkl. IEEE-754-Sign-of-Zero-Muster
(`-0.0` für `required/volunteer/committed_voluntary`, `+0.0` für `paid/overall`).
Alle 8 Fixtures liefern erneut das erwartete Bit-Muster ohne Änderung — keine
`+0.0 vs -0.0`-Divergenz aufgetreten.

## Diff-Größe

```
service_impl/src/reporting.rs | 552 ++++++++++++++++++++++--------------------
1 file changed, 296 insertions(+), 256 deletions(-)
```

- **Extraktion:** ~245 Zeilen (Per-Person-Aggregations-Body) in `assemble_weeks`.
- **Wrapper `get_week`:** 41 Zeilen (3 Bulk-Loads + `info!` + Delegation + Pop).
- **Netto:** +40 Zeilen (Helper-Doc-Kommentar, `#[allow]`, äußere Loop-Struktur,
  Filter-Closures).

## Deviations from Plan

### D-52-11 — Kein Transaction-Envelope in get_week

Der Plan-`behavior`-Block skizzierte:

```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
// … Delegation …
self.transaction_dao.commit(tx).await?;
```

**Ausgeführt ohne Envelope.** Grund: Der Pre-Refactor-`get_week` hatte keinen
`use_transaction`/`commit`-Pfad — er reichte `tx.clone()` an die drei
DAO-Load-Services durch und returnte am Ende ohne commit. Die
Reporting-Unit-Tests (`reporting_additive_merge::*`, 4 Stück) stubben in ihren
`MockTransactionDao` **explizit KEINE** `commit`-Expectation, weil im
Baseline-Code kein `commit` aufgerufen wird. Ein zusätzlicher Envelope hätte
die Tests mit `MockTransactionDao::commit(...): No matching expectation found`
kaputt gemacht — Byte-Identität wäre verletzt.

**Rule 2 — Auto-Preserve Correctness:** Wave 2 ist explizit ein *pure Refactor*
(D-52-09: "Kein Verhaltens-Change"). Der Envelope wäre ein Verhaltens-Change
gewesen. Zurück zur Baseline-Semantik.

Konsequenz: Byte-Identität + alle Tests grün. Falls Wave 4/5 einen zentralen
Envelope brauchen (etwa für konsistente Snapshot-Reads über 55 Wochen), kann
das dort explizit als eigener Task nachgezogen werden — es ist keine Regression,
sondern erhaltener Status-Quo.

Sonst: keine Abweichungen. Extraktion 1:1, Semantik-Invarianten strikt.

## Known Stubs

Keine.

## Threat Flags

Keine. T-52-03 (is_paid-Filter) + T-52-04 (Toggle-Read wandert versehentlich)
sind laut Grep-Guard beide sauber:

- `is_paid`-Filter (`continue`) sitzt an derselben Stelle in der
  Per-Person-Schleife wie im Pre-Refactor-Body — Fixture 4 (Volunteer-Vacation)
  liefert unverändertes Bit-Muster.
- `shortday_gate` in Non-Comment-Code = 0 → Toggle-Read bleibt in
  `booking_information.get_weekly_summary`, Chain-C-Tests unverändert grün.

## Notes for Wave 3/4

- `assemble_weeks` ist ready zum Konsumieren aus einer neuen `get_year`-Trait-
  Methode: `assemble_weeks(&year_weeks_vec, &all_work_details, &all_shiftplan_reports, &all_extra_hours, context, tx)` liefert `Vec<(u8, Arc<[ShortEmployeeReport]>)>` in Eingabe-Reihenfolge.
- Beim Batch-Load in Wave 3 daran denken: **pro Woche EIN
  `apply_weekly_cap`-Aufruf** — der Helper macht das schon korrekt, weil
  `cap_active`/`raw_shiftplan_hours`/`expected_hours_for_cap` im
  Per-Woche-Scope liegen (CVC-06 strukturell).
- `absence_service.derive_hours_for_range` + `build_derived_holiday_map` +
  `sales_person_service.get` bleiben in der Per-Person-Schleife.
  Load-once-Optimierung ist **NICHT** Teil dieses Helpers — das wäre
  Wave 4/5-Follow-up (RESEARCH Q2 dokumentiert).

## Self-Check: PASSED

- ✓ `pub(crate) async fn assemble_weeks` existiert in `service_impl/src/reporting.rs` (Zeile 269+)
- ✓ `get_week` delegiert auf `assemble_weeks(&[(year, week)], ...)`, Trait-Signatur unverändert
- ✓ `git diff service/src/reporting.rs` = leer
- ✓ Commit `c021d95` in git log
- ✓ `cargo test --workspace` = alle grün (703+64+11+Fixtures)
- ✓ `cargo test --package service_impl --lib booking_information_weekly_summary_year_batch` = 8 passed
- ✓ `cargo test --package service_impl --lib reporting` = 77 passed
- ✓ `cargo clippy --workspace -- -D warnings` = clean
- ✓ `grep -v '^#\|//' service_impl/src/reporting.rs | grep -c 'shortday_gate'` = 0
