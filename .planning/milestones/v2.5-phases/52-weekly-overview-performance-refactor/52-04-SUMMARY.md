---
phase: 52-weekly-overview-performance-refactor
plan: 04
subsystem: [backend, reporting]
tags: [WOP-02, WOP-05, D-52-02, D-52-03, D-52-04, D-52-06, D-52-08, D-52-09, D-52-10, year-batch, additive-trait]
requires: ["52-02", "52-03"]
provides:
  - "ReportingService::get_year — Batch-Variante von get_week; liefert alle Wochen des Jahres in strikt aufsteigender Vec-Ordnung"
  - "ReportingServiceImpl::get_year — 3 Bulk-Load-Roundtrips + Delegation auf assemble_weeks; kein eigener Transaction-Envelope"
affects:
  - service/src/reporting.rs
  - service_impl/src/reporting.rs
  - service_impl/src/test/mod.rs
  - service_impl/src/test/reporting_get_year.rs
tech-stack:
  added: []
  patterns:
    - "Additive Trait Extension (D-52-06) — get_week-Signatur strikt unverändert"
    - "Batch-Load-Delegation — 3 Jahres-Bulk-Loads + assemble_weeks-Wiederverwendung (WOP-02)"
    - "IEEE-754 to_bits()-Vergleich für Byte-Identity-Guard (T-52-09)"
key-files:
  created:
    - service_impl/src/test/reporting_get_year.rs
  modified:
    - service/src/reporting.rs
    - service_impl/src/reporting.rs
    - service_impl/src/test/mod.rs
decisions:
  - "D-52-02 umgesetzt — get_year(year, context, tx) -> Arc<[(u8, Arc<[ShortEmployeeReport]>)]> als neue Trait-Method (Vec, kein Struct)"
  - "D-52-03 umgesetzt — Vec strikt aufsteigend nach calendar_week (1..=weeks_in_year(year)); leere Wochen mit leerem Arc<[]>"
  - "D-52-04 umgesetzt — get_year nimmt genau EIN year; Spillover ist Consumer-Sache"
  - "D-52-06 umgesetzt — Rein additiv, get_week-Trait-Signatur byte-identisch unverändert"
  - "D-52-08 umgesetzt — Impl delegiert auf assemble_weeks mit Vec::from((1..=weeks_in_year).map(|w|(year,w)))"
  - "D-52-09 umgesetzt — Alle Semantik-Invarianten via assemble_weeks erhalten (Balance-Formel, CVC-06 Cap, ExtraHours-Split, is_paid-Filter, Chain-C-Toggle-Read außerhalb)"
  - "D-52-10 umgesetzt — get_week bleibt public trait method"
  - "R2 Auto-Preserve-Correctness — kein Transaction-Envelope in get_year (spiegelt get_week aus Wave 2, verhindert Mock-Test-Erwartungs-Breaks)"
  - "#[allow(clippy::type_complexity)] auf Trait-Signatur — der nested Arc<[(u8, Arc<[ShortEmployeeReport]>)]> Return-Type ist Teil des D-52-02-Vertrags; Type-Alias wäre für Consumer weniger lesbar"
metrics:
  duration_minutes: ~20
  completed: 2026-07-05
  tasks_completed: 2
  files_created: 1
  files_modified: 3
  tests_added: 3
  net_lines_added: ~370
status: complete
---

# Phase 52 Plan 04: `ReportingService::get_year` (WOP-02) Summary

**One-liner:** Neue additive Batch-Trait-Method `ReportingService::get_year(year)` — drei
Bulk-Load-Roundtrips (`employee_work_details.all` + neu `extract_shiftplan_report_for_year`
+ neu `find_by_year`) plus Delegation auf den in Wave 2 extrahierten `assemble_weeks`-Helper
mit einem Vec über alle ISO-Wochen des Jahres. Byte-Identität zu 55×`get_week` ist strukturell
garantiert — beide Pfade rufen denselben Helper mit denselben Slice-Referenzen pro Woche auf.

## Was gebaut wurde

### Task 1 — Trait-Signatur + Impl

**Trait-Method** (`service/src/reporting.rs:405-421`), direkt unter `get_week`:

```rust
/// Batch-Variante von `get_week`: liefert alle Wochen des Jahres in einem Rutsch.
/// - Vec strikt aufsteigend nach calendar_week (1..=weeks_in_year(year));
///   leere Wochen erscheinen mit leerem Arc<[]> (D-52-03).
/// - Nimmt genau EIN year; Spillover ist Consumer-Sache (D-52-04).
/// - Impl delegiert auf assemble_weeks (WOP-02 — byte-identisches Verhalten zu 55×get_week).
/// - Additiv zu get_week, dessen Signatur unverändert bleibt (D-52-06 / D-52-10).
#[allow(clippy::type_complexity)]
async fn get_year(
    &self,
    year: u32,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[(u8, Arc<[ShortEmployeeReport]>)]>, ServiceError>;
```

`#[automock]` regeneriert `MockReportingService::expect_get_year()` automatisch. Es gibt
KEINE manuellen `ReportingService`-Trait-Impls in Tests (nur `#[automock]`-Nutzung in
`billing_period_report.rs`, `shiftplan_edit.rs`, `booking_information_vfa.rs`,
`booking_information_chain_c.rs`, `booking_information_weekly_summary_year_batch.rs`) —
also KEINE Stub-Updates nötig.

**Impl** (`service_impl/src/reporting.rs`, direkt hinter `get_week`):

```rust
async fn get_year(&self, year, context, tx)
    -> Result<Arc<[(u8, Arc<[ShortEmployeeReport]>)]>, ServiceError>
{
    let work_details = self.employee_work_details_service
        .all(Authentication::Full, tx.clone()).await?;
    let shiftplan_reports = self.shiftplan_report_service
        .extract_shiftplan_report_for_year(year, Authentication::Full, tx.clone()).await?;
    let extra_hours = self.extra_hours_service
        .find_by_year(year, Authentication::Full, tx.clone()).await?;
    info!("Extra hours (year batch): {:?}", &extra_hours);

    let weeks_in_year = time::util::weeks_in_year(year as i32);
    let weeks: Vec<(u32, u8)> = (1..=weeks_in_year).map(|w| (year, w)).collect();

    let assembled = self.assemble_weeks(
        &weeks, &work_details, &shiftplan_reports, &extra_hours, context, tx,
    ).await?;

    Ok(assembled.into())
}
```

**Delegation-Semantik:**
- `assemble_weeks` liefert `Vec<(u8, Arc<[ShortEmployeeReport]>)>` in Eingabe-Reihenfolge.
  Eingabe ist `(1..=weeks_in_year).map(|w| (year, w))` — strikt aufsteigend → Ausgabe
  erfüllt D-52-03 automatisch.
- Für Wochen ohne Contract oder ohne Daten liefert der Helper ein leeres `Arc<[]>`,
  weil die Per-Person-Aggregations-Schleife über die HashMap keine Einträge findet
  (bzw. der `is_paid`-Filter alle skippt).
- `Vec<...> -> Arc<[...]>` per `.into()`.

**Kein Transaction-Envelope** (analog Wave-2 `get_week`, D-52-11 dort dokumentiert):
`get_year` reicht `tx.clone()` an alle drei Bulk-Loads und an `assemble_weeks` durch, ohne
`use_transaction`/`commit`. Die Consumer entscheiden über TX-Lifetime. Rule 2 —
Auto-Preserve-Correctness: Ein neuer Envelope würde bestehende Mock-Test-Erwartungen für
Reporting-Tests brechen (Baseline stub kein `commit`).

### Task 2 — Sanity-Tests (3 Tests in neuem Modul)

Neues Test-Modul `service_impl/src/test/reporting_get_year.rs` mit drei
`#[tokio::test]`-Fns:

1. **`test_get_year_returns_all_weeks_in_year_ascending`** — D-52-03 Vec-Shape:
   - `result.len() == weeks_in_year(2024) as usize` (= 52)
   - `result[0].0 == 1` und `result.last().0 == 52`
   - `windows(2).all(|w| w[0].0 < w[1].0)` — strikt aufsteigend
   - Woche 23/2024 (Contract KW22-25 aktiv) → genau 1 Report (is_paid=true)

2. **`test_get_year_matches_get_week_for_arbitrary_week`** — T-52-09 Off-by-one-Guard:
   - Zwei getrennte Service-Instanzen mit identischem Mock-Setup (mockall Expectations
     sind per Instanz).
   - `get_year(2024)` und `get_week(2024, 23)` werden verglichen.
   - Pro `ShortEmployeeReport`-Feld: `to_bits()`-Vergleich für IEEE-754-sign-of-zero-safe
     Bit-Identität von `balance_hours`, `dynamic_hours`, `expected_hours`, `overall_hours`,
     `vacation_hours`, `sick_leave_hours`, `holiday_hours`, `unavailable_hours`,
     `unpaid_leave_hours`, `volunteer_hours`.
   - Zusätzlich: `year_result[week-1].0 == week` — explizite Off-by-one-Assertion.

3. **`test_get_year_empty_when_no_work_details`** — D-52-03 leere Wochen:
   - Wenn `employee_work_details_service.all()` einen leeren Slice liefert, hat JEDE
     Woche im Vec ein leeres `Arc<[]>` — nicht ausgelassen, sondern präsent mit leerem
     Slice, damit Consumer 1-basierten Index direkt nutzen kann.

**Mock-Setup:** Baut minimales Fixture mit einem `sales_person` (is_paid=true, Contract
KW22-25/2024). Alle Bulk-Load-Mocks liefern leere Slices (kein Shiftplan, kein ExtraHours).
`absence_service.derive_hours_for_range` → leere BTreeMap. `toggle_service.get_toggle_value`
→ None (kein holiday_auto_credit). `sales_person_service.get` → `fixture_sales_person`.

## Verifikations-Gates

| Gate | Erwartet | Ist | Status |
| ---- | -------- | --- | ------ |
| `cargo build --workspace` | grün | grün | ✅ |
| `cargo test --workspace` | alle grün | 713 unit + 64 integration + 11 rest = alle grün | ✅ |
| `cargo test --package service_impl reporting_get_year` | 3 grün | 3 passed | ✅ |
| `cargo clippy --workspace -- -D warnings` | 0 warnings | 0 warnings | ✅ |
| `cargo clippy --workspace --tests -- -D warnings` | 0 warnings | 0 warnings | ✅ |
| Wave-1 Fixtures (`booking_information_weekly_summary_year_batch`) | 8 passed | 8 passed | ✅ |
| Wave-2 Reporting-Suite (`--lib reporting`) | 77 passed | alle grün | ✅ |
| `grep -v '^#\|//' service_impl/src/reporting.rs \| grep -c 'shortday_gate'` | 0 | 0 | ✅ (nur 1 in Doc-Kommentar) |
| `git diff bdbdc28..HEAD -- service/src/reporting.rs \| grep '^-'` | leer | leer | ✅ (rein additiv) |
| `MockReportingService::expect_get_year()` kompiliert | ja | ja (`#[automock]` regeneriert) | ✅ |

## Diff-Größe

```
service/src/reporting.rs               |  17 ++++++
service_impl/src/reporting.rs          |  47 ++++++++++++++
service_impl/src/test/mod.rs           |   2 +
service_impl/src/test/reporting_get_year.rs | +309 (new)
4 files changed, ~375 insertions(+), 0 deletions(-)
```

## Deviations from Plan

### D-52-11 Fortsetzung — Kein Transaction-Envelope in get_year

Der Plan-`behavior`-Block skizzierte:
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
// … Delegation …
self.transaction_dao.commit(tx).await?;
```

**Ausgeführt ohne Envelope**, konsistent zu Wave 2 (`get_week`) und aus demselben Grund
dokumentiert: Der Wave-2-`get_week`-Body hatte ebenfalls keinen Envelope, um die
Reporting-Unit-Tests (die kein `commit`-Expectation stubben) byte-identisch zu erhalten.
`get_year` folgt derselben Symmetrie — kein Envelope, `tx.clone()` an die drei Bulk-Loads
und an `assemble_weeks` durchgereicht, kein `commit` am Ende.

Konsequenz: Byte-Identität + alle Tests grün. Sollte Wave 5 (`get_weekly_summary`) einen
zentralen Envelope für Konsistenz über die 55-Wochen-Aggregation brauchen, ist das dort
explizit zu adressieren — es ist keine Regression, sondern erhaltener Status-Quo aus
Wave 2.

### Rule-3 Fix: `#[allow(clippy::type_complexity)]` auf Trait-Signatur

Clippy meldete `type_complexity` für den Return-Type `Arc<[(u8, Arc<[ShortEmployeeReport]>)]>`.
Ein `type`-Alias wurde erwogen und verworfen:

- Der Return-Type ist Teil des expliziten D-52-02-Vertrags und wird in der Doku
  wörtlich zitiert.
- Ein Alias wie `WeekReports` würde die semantische Struktur (Vec-Ordering-Bedeutung)
  im Consumer verstecken.
- Fix inline: `#[allow(clippy::type_complexity)]` direkt an der Method.

Files: `service/src/reporting.rs`. Commit: `87d3b8f`.

Sonst keine Abweichungen. Plan wurde exakt wie geschrieben ausgeführt.

## Threat mitigations

| Threat ID | Category | Status | Verifikation |
|-----------|----------|--------|--------------|
| T-52-08 | Elevation of Privilege | mitigated | `get_year`-Impl-Struktur spiegelt `get_week` 1:1: alle drei Bulk-Loads mit `Authentication::Full`, der `context`-Param wird an `assemble_weeks` durchgereicht — identische Auth-Semantik. |
| T-52-09 | Tampering (Off-by-one) | mitigated | `test_get_year_matches_get_week_for_arbitrary_week` verifiziert `year_result[week-1].1 == week_result` bit-exakt. |

## Known Stubs

Keine.

## Threat Flags

Keine. Keine neuen Netzwerk-Endpoints, keine neuen Auth-Pfade, keine
Trust-Boundary-Änderungen. Reine additive Business-Logic-Trait-Erweiterung, die die
Wave-2-Semantik-Invariante durch strukturelle Delegation erbt.

## Notes for Wave 5

- `get_year` ist ready für Wave 4/5 Plan 05 (`booking_information.get_weekly_summary`):
  Consumer ruft `get_year(year) + ggf. get_year(year+1)` für Spillover und indexiert
  `result[week - 1].1` für die pro-Woche `Arc<[ShortEmployeeReport]>`.
- `sales_person_service.get` läuft im Helper weiter per-person per-week (D-52-09/R9)
  — Load-once-Optimierung ist explizit NICHT Teil dieses Plans (bleibt Follow-up).
- Chain-C-Toggle-Read (`shortday_gate`) bleibt außerhalb, in `get_weekly_summary`
  (D-52-09/R8). `assemble_weeks` und `get_year` sind toggle-agnostisch.

## Commits

- `6fa5128` — feat(52-04): add ReportingService::get_year (WOP-02)
- `87d3b8f` — test(52-04): sanity + off-by-one guards for ReportingService::get_year

## Self-Check: PASSED

- ✓ `service/src/reporting.rs` enthält `get_year`-Trait-Signatur (Z. 405-421)
- ✓ `service_impl/src/reporting.rs` enthält `get_year`-Impl (direkt hinter `get_week`)
- ✓ `service_impl/src/test/reporting_get_year.rs` mit 3 Tests
- ✓ Commit `6fa5128` in git log
- ✓ Commit `87d3b8f` in git log
- ✓ `cargo test --workspace` = alle grün (713 unit + 64 integration)
- ✓ `cargo test --package service_impl reporting_get_year` = 3 passed
- ✓ `cargo clippy --workspace --tests -- -D warnings` = clean
- ✓ `grep -v '^#\|//' service_impl/src/reporting.rs | grep -c 'shortday_gate'` = 0
- ✓ `git diff` zeigt nur additive Änderungen an service/src/reporting.rs (keine `-`-Zeilen)
