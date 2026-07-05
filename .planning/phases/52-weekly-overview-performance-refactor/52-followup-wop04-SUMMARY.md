---
phase: 52-weekly-overview-performance-refactor
plan: followup-wop04
subsystem: [backend, reporting, performance]
tags: [perf, load-once, hash-map-index, byte-identity, WOP-04]
requires: ["52-05"]
provides:
  - "assemble_weeks-Helper akzeptiert zwei zusätzliche In-Memory-Indexe (sales_person_index + working_hours_by_sp) für per-Woche O(1)-Lookup statt N_persons × N_weeks Roundtrips/Re-Bucketing"
affects:
  - service_impl/src/reporting.rs
  - service_impl/src/test/reporting_additive_merge.rs
  - service_impl/src/test/reporting_get_year.rs
  - service_impl/src/test/reporting_holiday_auto_credit.rs
tech-stack:
  added: []
  patterns:
    - "Load-once + pre-index caller-owned HashMap durchgereicht an per-Week-Aggregation-Helper"
    - "Fallback-Path zur Baseline-DAO-Semantik bei Index-Miss (Rückwärts-kompatibel, byte-identisch)"
key-files:
  created:
    - .planning/phases/52-weekly-overview-performance-refactor/52-followup-latency-post-optimization.txt
    - .planning/phases/52-weekly-overview-performance-refactor/52-followup-wop04-SUMMARY.md
  modified:
    - service_impl/src/reporting.rs
    - service_impl/src/test/reporting_additive_merge.rs
    - service_impl/src/test/reporting_get_year.rs
    - service_impl/src/test/reporting_holiday_auto_credit.rs
decisions:
  - "assemble_weeks bekommt zwei zusätzliche &HashMap-Parameter — Signaturänderung ist pub(crate), keine Public-API-Erweiterung"
  - "get_week UND get_year bauen die Indexe intern (aus ihren bereits geladenen Bulk-Slices) — Callers außerhalb (booking_information) müssen nichts anpassen. Vorteil: byte-identische Public-API, Kosten: 1× O(N_sp) get_all-Roundtrip im get_year-Pfad zusätzlich (der wird durch die eingesparten ~55×N_sp per-Person get()-Roundtrips überkompensiert)"
  - "Iteration in assemble_weeks iteriert jetzt working_hours_by_sp statt der pro-Woche neu gebauten HashMap — Iteration-Reihenfolge bleibt HashMap-non-deterministic, aber pro Prozess stabil, identisch zur Pre-Follow-Up-Baseline (die auch schon HashMap-basiert iterierte)"
  - "Fallback für Index-Miss (should-not-happen — Index kommt aus get_all, WD kommt aus dieselbe DB) ruft alten .get(id, …)-Pfad — bewahrt Baseline-Fehler-Shape falls Race passiert"
metrics:
  duration_minutes: ~35
  completed: 2026-07-05
  tasks_completed: 2
  files_created: 2
  files_modified: 4
  tests_added: 0
  net_lines_added: ~90
status: complete
---

# Phase 52 Follow-Up: `assemble_weeks` sales_person + working_hours Load-Once (WOP-04)

**One-liner:** Zwei byte-identische In-Memory-Optimierungen im `assemble_weeks`-
Helper: (1) `sales_person_service.get(sp_id, …)` per (Person × Woche) durch
`HashMap<Uuid, SalesPerson>`-Lookup ersetzt, (2) `working_hours` einmalig per
`sales_person_id` vor der Wochen-Schleife gebucketet statt pro Woche neu. Alle
8 Wave-1-Fixtures bleiben byte-identisch grün; zusätzlicher Speedup 13-16% ggü
Wave 5 (2.33 s → 1.13 s → ~0.97 s repräsentativer Median). WOP-04 <0.5 s bleibt
verfehlt — die verbleibende Latenz kommt aus `special_day`/`toggle`/`absence_period`-
Roundtrips im Per-Person-Loop, dokumentiert als weitere Follow-Ups.

## Was gebaut wurde

### Task 1 — `assemble_weeks`-Signatur erweitert

Zwei neue Parameter (pub(crate), keine trait-öffentliche Änderung):

```rust
#[allow(clippy::too_many_arguments)]
pub(crate) async fn assemble_weeks(
    &self,
    weeks: &[(u32, u8)],
    work_details: &[EmployeeWorkDetails],
    shiftplan_reports: &[ShiftplanReportDay],
    extra_hours: &[ExtraHours],
    sales_person_index: &HashMap<Uuid, SalesPerson>,      // ◀ NEU
    working_hours_by_sp: &HashMap<Uuid, Arc<[EmployeeWorkDetails]>>, // ◀ NEU
    context: Authentication<Deps::Context>,
    tx: Option<Deps::Transaction>,
) -> Result<Vec<(u8, Arc<[ShortEmployeeReport]>)>, ServiceError>
```

**Body-Änderungen:**

- Der pre-follow-up Loop-Kopf `let working_hours = work_details.iter().cloned().collect_to_hash_map_by(|wh| wh.sales_person_id);` (per-Woche re-bucketing) ist entfallen.
- Die Iteration wechselt von `for (sales_person_id, working_hours) in working_hours` (per-Woche-HashMap-Iteration) auf `for (sales_person_id, working_hours) in working_hours_by_sp.iter()` mit `let working_hours: &[EmployeeWorkDetails] = working_hours.as_ref();` — semantisch derselbe Slice, Iteration-Reihenfolge äquivalent (beide HashMaps über dieselben Keys).
- Die 6× `find_working_hours_for_calendar_week(&working_hours, year, week)`-Calls und der `build_derived_holiday_map(…, &working_hours, …)`-Call arbeiten jetzt direkt auf dem `&[EmployeeWorkDetails]`-Slice statt auf einer per-Woche neu erzeugten `Arc<[EmployeeWorkDetails]>`.
- Der DAO-Roundtrip `sales_person_service.get(sales_person_id, Authentication::Full, tx.clone())` wurde durch einen Match ersetzt:

```rust
let sales_person = match sales_person_index.get(&sales_person_id) {
    Some(sp) => sp.clone(),
    None => {
        // Fallback bewahrt pre-follow-up Fehler-Shape (should-not-happen).
        self.sales_person_service
            .get(sales_person_id, Authentication::Full, tx.clone())
            .await?
    }
};
```

- Der `is_paid`-Filter (`continue;`) sitzt exakt an derselben Stelle wie pre-follow-up (T-52-03/CVC-10-Mitigation).

**`let _ = work_details;`-Guard:** `work_details` bleibt als Parameter in der
Signatur (Wave-2-Contract für Callers, die eventuell noch nicht auf die
Index-Semantik migriert sind), wird intern aber nicht mehr gelesen — der
`let _ = …;`-Guard verhindert dead_code-Warnings und dokumentiert die
Absicht.

### Task 1 — Aufrufer `get_week` erweitert

```rust
let all_sales_persons = self
    .sales_person_service
    .get_all(Authentication::Full, tx.clone())
    .await?;
let sales_person_index: HashMap<Uuid, SalesPerson> = all_sales_persons
    .iter()
    .map(|sp| (sp.id, sp.clone()))
    .collect();
let working_hours_by_sp: HashMap<Uuid, Arc<[EmployeeWorkDetails]>> = work_details
    .iter()
    .cloned()
    .collect_to_hash_map_by(|wh| wh.sales_person_id);

let mut assembled = self.assemble_weeks(
    &[(year, week)],
    &work_details,
    &shiftplan_report,
    &extra_hours,
    &sales_person_index,
    &working_hours_by_sp,
    context,
    tx,
).await?;
```

Kosten für den Einzelwochen-Konsumenten: 1× zusätzlicher `get_all`-Roundtrip
und 1× `collect_to_hash_map_by`-Traversierung. Nutzen: 1× ID-Lookup statt bis
zu N_sp DAO-Roundtrips im Fallback-Fall (in der 1-Woche-Delegation minimal,
aber strukturell konsistent zum Year-Batch).

### Task 1 — Aufrufer `get_year` erweitert (der eigentliche Gewinner)

Identisch zu `get_week`, aber im Kontext von 55 Wochen: **einmalig** vor der
Wochen-Schleife (im `assemble_weeks`-Helper wird die Schleife über die 55
Wochen durchlaufen). Der `get_all`-Roundtrip kostet ~2 ms auf der Dev-DB,
spart aber 55 × 6 = 330 `get(id, …)`-Calls plus die 55 vollen `collect_to_hash_map_by`-
Rebucketings des `work_details`-Slices.

**Der `booking_information.get_weekly_summary`-Aufrufer bleibt UNVERÄNDERT** —
er ruft nur `reporting_service.get_year(year, …)` auf; die Präambel-Logik dort
(Bulk-Loads von Special-Days, Shiftplan-Reports, Slots, ShiftplanCatalog) ist
unabhängig. Die neue Wave-5-Chain-C-Toggle-Read-Position (Zeile 320,
`shortday_gate::read_active_from`) ist ebenfalls **unangetastet**.

### Task 2 — Test-Anpassungen

Bei 11 Reporting-Tests fehlte ein `MockSalesPersonService::expect_get_all()`-
Setup, weil pre-follow-up der `get_week`-Wrapper nur `expect_get()`-basierte
Mocks brauchte. Ich habe pro Test das Fixture ergänzt:

- `service_impl/src/test/reporting_additive_merge.rs` — 8 Setups gepatcht
  (`setup_common_mocks`, `test_get_week_additive_merge`, `build_parity_service`,
  `build_parity_service_dynamic`, `test_balance_parity_dynamic_get_week` × 2,
  `get_week_skips_unpaid_person`, `get_week_unpaid_no_paid_hours_leak`).
- `service_impl/src/test/reporting_get_year.rs` — 2 Setups gepatcht
  (`build_service`, `test_get_year_empty_when_no_work_details`).
- `service_impl/src/test/reporting_holiday_auto_credit.rs` — 4 Setups gepatcht
  (`setup_holiday_common_mocks`, `test_holiday_auto_credit_get_week_reduces_soll_bands_unchanged`,
  `test_hsp04_before_cutoff`, `test_hsp04_manual_wins`,
  `test_hsp03_cap_active_holiday_no_band_leak`).

Jeder Patch fügt eine `expect_get_all()`-Expectation hinzu, die dieselbe
`fixture_sales_person()` (bzw. bei den `unpaid`-Tests: sowohl paid als
auch unpaid) als `Arc<[SalesPerson]>` zurückgibt. `expect_get()` bleibt in
allen Tests bestehen — die Fallback-Route im Helper würde ihn im
Index-Miss-Fall aufrufen; im Happy-Path wird der Fallback nicht aktiviert
(die Fixtures haben SP-Ids in Index UND WD → HashMap-Hit).

**Wave-1-Fixtures (`booking_information_weekly_summary_year_batch.rs`) sind
NICHT angepasst worden** — sie hatten schon `expect_get_all()` seit Wave 5
(dort für den Volunteer-Filter). Sie sind der Byte-Identity-Gate und bleiben
strukturell und semantisch unangetastet.

### Task 3 — Latenz-Messung

Drei separate 5-Run-Median-Messungen (jeweils 3 Warmup-Runs davor), identisches
Verfahren wie Wave 5:

| Messung | Median | Streuung | Ambient |
|---------|--------|----------|---------|
| 1 (kalter Cache-Ramp) | 1.180 s | 0.12 s | direkt nach Server-Start |
| 2 (voller Cache) | 0.969 s | 0.16 s | nach vollem Cache-Warmup |
| 3 (Konsistenz-Check) | 0.975 s | 0.18 s | Bestätigung Messung 2 |
| **Rep. Median (2+3)** | **~0.97 s** | | stabiler steady-state |

Detail siehe `.planning/phases/52-weekly-overview-performance-refactor/52-followup-latency-post-optimization.txt`.

## Verifikations-Gates

| Gate | Erwartet | Ist | Status |
| ---- | -------- | --- | ------ |
| `cargo build -p service_impl` | grün | grün | ✅ |
| `cargo test --package service_impl --lib booking_information_weekly_summary_year_batch` | 8 passed **byte-identisch** | 8 passed byte-identisch | ✅ **Byte-Identity-Gate** |
| `cargo test --package service_impl --lib reporting` | alle passed | 77 passed | ✅ |
| `cargo test --package service_impl --lib` | alle passed | 713 passed | ✅ |
| `cargo test --workspace` | alle passed | alle passed (713 unit + 64 integration + weitere) | ✅ |
| `cargo clippy --workspace -- -D warnings` | 0 warnings | 0 warnings | ✅ |
| `cargo clippy --workspace --tests -- -D warnings` | 0 warnings | 0 warnings | ✅ |
| Latenz-Median post-optimization | best-effort | ~0.97 s (rep.); 1.18 s (cold) | ⚠️ **FAIL** bzgl. WOP-04 <0.5 s (dokumentiert) |
| Frontend touched | nein | nein (`git diff … shifty-dioxus/` = leer) | ✅ |

## Wave-1-Fixture-Ergebnis (Byte-Identity-Gate)

```
test fixture_1_baseline ... ok
test fixture_2_holiday_week_n ... ok
test fixture_3_shortday_week_n ... ok
test fixture_4_volunteer_vacation_period ... ok
test fixture_5_cvc06_cap_active ... ok
test fixture_6_gate_off_legacy ... ok
test fixture_7_gate_on_active_from_before_week ... ok
test fixture_8_combined_holiday_shortday_volunteer_cap_gate ... ok
test result: ok. 8 passed; 0 failed
```

IEEE-754-Sign-of-Zero-Muster (`-0.0` für `required_hours`, `volunteer_hours`,
`committed_voluntary_hours`; `+0.0` für `paid_hours`/`overall_available_hours`)
bleibt bit-identisch erhalten. Fixture 8 (Kombi + Spillover) bestätigt: der
neue HashMap-Iteration-Path erzeugt dieselbe Reihenfolge-Semantik wie die
Pre-Follow-Up-Impl (beide iterieren `HashMap<Uuid, …>`; per-Prozess-Random-
Seed konsistent).

## Latenz-Verlauf über die Waves

| Messung | Median | Streuung | Faktor ggü. Baseline |
| ------- | ------ | -------- | -------------------- |
| Wave-0-Baseline | 2.330 s | 1.60 s (68%) | 1.0× |
| Wave-5-Post-Refactor | 1.126 s | 0.13 s (11%) | 2.07× |
| Follow-Up-Post (repräsentativ) | ~0.97 s | 0.16 s (16%) | **~2.40×** |
| WOP-04-Ziel | <0.500 s | — | Ziel = 4.66× |

**Der zusätzliche Speedup ist real (13-16% ggü Wave 5), aber der Rest bis
zum 0.5 s-Ziel lebt in DAO-Chains, die noch nicht bulk-loaded sind
(`special_day`, `toggle`, `absence_period` im Per-Person-Loop des Helpers).**

## WOP-04-Status

**FAIL** — Median liegt weiterhin ~2× über dem Zielwert.

**Ursachenanalyse (via sqlx-Query-Log der Dev-Session, 9 Anfragen)**:

| Table | Query-Count | Semantik |
|-------|-------------|----------|
| `sales_person_user` | 19020 | User-Assignment-Lookups (nicht Teil dieser Optimierung) |
| `special_day` | 11466 | `get_by_week` pro (Person, Woche) in `build_derived_holiday_map` |
| `toggle` | 9588 | `holiday_auto_credit` pro (Person, Woche) in `build_derived_holiday_map` |
| `absence_period` | 4746 | `derive_hours_for_range` pro (Person, Woche) im Loop |
| `sales_person` (get all) | 29 | ✔ Bulk-Load — **korrekt** |
| `sales_person WHERE id = ?` | 12 | ~1 pro Anfrage aus anderen Konsumenten (nicht `assemble_weeks`) |

**Beweis**: der Follow-Up hat den `sales_person`-Bottleneck strukturell beseitigt
(12 statt vorher potentiell 55 × 6 × 9 = ~3000 by-id-Queries).
Die verbleibende Latenz konzentriert sich auf drei nicht batch-geladene
Chains — dokumentiert in der Latenz-Datei als Follow-Up #1-3.

## Nächste Optimierungs-Kandidaten (dokumentiert für v2.5 oder v2.6)

1. **`special_day`-Jahres-Bulk-Load durchreichen an `assemble_weeks`**
   — der Consumer `booking_information.get_weekly_summary` lädt bereits
   `special_days_this` + `_next` per `get_by_year`. Dieselbe Bulk-Slice
   an `assemble_weeks` durchreichen und `build_derived_holiday_map` auf
   In-Memory-Filter statt DAO-Roundtrip umstellen. Erwarteter Impact:
   ~30-40% zusätzlicher Speedup (die 11k `special_day`-Queries fallen
   auf 2 pro Anfrage).

2. **`holiday_auto_credit`-Toggle-Read einmalig per Method-Call cachen.**
   Der Toggle-Wert hängt weder an Jahr noch Woche. Ein `Option<Option<Date>>`-
   Cache im Helper-Aufrufer würde die 9588 `toggle`-Queries auf 1 pro Anfrage
   reduzieren.

3. **`absence_period`-Jahres-Bulk-Load an `assemble_weeks` durchreichen.**
   Der Consumer hat bereits `all_absences`; ein neuer Helper-Parameter
   könnte diese Slice übernehmen und den `derive_hours_for_range`-DAO-Call
   inline in-memory ersetzen. Höherer Refactor-Aufwand.

4. **DB-Indices** (RESEARCH Q3): Migration mit `booking(year, calendar_week)`,
   `extra_hours(date_time)`, `working_hours(from_year, to_year)`. Separater
   Task (Migration = eigener PR).

**Konsequente Umsetzung von Follow-Up #1+#2 würde WOP-04 <0.5 s realistisch
erreichbar machen.**

## Deviations from Plan

**Keine architekturellen Abweichungen** (Rule 4 — nichts). Zwei Rule-3-Auto-Fixes:

### D-1 — Test-Mocks für 11 Reporting-Tests erweitert (Rule 3)

- **Ausgangslage:** 11 reporting_*-Tests hatten pre-follow-up nur `expect_get()`
  gestubbt und keinen `expect_get_all`. Der Follow-Up bringt einen neuen
  `sales_person_service.get_all(…)`-Call in `get_week`/`get_year` — die Tests
  wurden ohne Anpassung sofort panic'ken mit `No matching expectation found`.
- **Fix:** Pro Test ein `.expect_get_all()`-Return mit demselben SP wie das
  bestehende `.expect_get()`. Keine semantische Änderung — der neue Setup
  liefert das identische SP-Set. `expect_get()` bleibt unverändert (Fallback-
  Pfad in der neuen Impl würde ihn ansonsten aufrufen).
- **Klassifizierung:** Rule 3 (auto-fix blocking issue). Byte-Identität der
  Wave-1-Fixtures beweist, dass die Test-Anpassung ausschließlich Test-Setup
  ist, keine Semantik-Änderung.

### D-2 — WOP-04 <0.5 s weiterhin nicht erreicht (dokumentierter Follow-Up)

- **Erwartung:** Best-Effort-Optimierung zur Zielerreichung.
- **Ist:** ~0.97 s repräsentativer Median (Faktor 2.40× ggü Baseline,
  1.16× ggü Wave 5). Zielwert-Delta: ~2× über 0.5 s.
- **Warum kein weiterer Fix in diesem Follow-Up:** Der Auftrag hat sich
  explizit auf die zwei benannten Optimierungen im `assemble_weeks`-Helper
  beschränkt (D-52-16-Regel: „Als Follow-Up dokumentieren, NICHT diese Phase
  blocken."). Die verbleibende Latenz lebt in drei separaten DAO-Chains
  (`special_day`, `toggle`, `absence_period`), die einen eigenen
  Bulk-Load-Refactor benötigen — außerhalb der Auftrags-Grenzen.
- **Klassifizierung:** kein Bug, keine fehlende kritische Funktionalität;
  explizit als Follow-Up erlaubt.

## Frontend-Bestätigung

`git diff -- shifty-dioxus/` = leer. Frontend unangetastet.

## Threat mitigations

| Threat ID | Category | Status | Verifikation |
|-----------|----------|--------|--------------|
| T-52-03 | Tampering (is_paid-Filter-Verschiebung) | mitigated | Die `continue;`-Position ist unverändert — direkt nach dem `sales_person`-Resolve (jetzt via Index oder Fallback). Fixture 4 (Volunteer-Vacation) grün. |
| T-52-13 | Denial of Service (Latenz) | accepted | ~0.97 s ist besser als Wave 5, aber <0.5 s bleibt offen; dokumentiert als Follow-Up. |
| T-52-14 (neu) | Tampering (Iteration-Reihenfolge kippt Byte-Identität) | mitigated | Iteration wechselte von `for x in local_hashmap` auf `for x in caller_hashmap` — beide sind `HashMap<Uuid, …>` mit denselben Keys. Wave-1-Fixture-8 (Volunteer + Cap + Spillover) bestätigt: keine Byte-Identity-Divergenz durch Reihenfolge-Änderung. |

## Known Stubs

Keine.

## Threat Flags

Keine neuen. Trust-Boundaries unverändert; keine neuen Netzwerk-Endpoints,
keine neuen Auth-Pfade, keine Schema-Änderungen, kein
`CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump (keine persistierten Werte
betroffen).

## Commits

- `perf(52): sales_person load-once + working_hours HashMap pre-index in assemble_weeks (WOP-04)` — Helper + get_week + get_year
- `test(52): add expect_get_all mocks for reporting tests hitting get_week/get_year (WOP-04)` — Test-Mock-Anpassungen
- `perf(52): post-optimization latency measurement + follow-up SUMMARY (WOP-04)` — Latenz-File + SUMMARY

## Self-Check: PASSED

- ✓ `assemble_weeks` in `service_impl/src/reporting.rs` hat zwei neue `&HashMap`-Parameter
- ✓ Beide Aufrufer (`get_week`, `get_year`) bauen die Indexe intern aus ihren bereits geladenen Bulk-Slices und delegieren durch
- ✓ `sales_person_index`-Lookup + Fallback zum alten `sales_person_service.get(…)`-Call bewahrt Baseline-Fehler-Shape
- ✓ Wave-1-Fixtures (`booking_information_weekly_summary_year_batch::fixture_*`) = **8/8 byte-identisch grün**
- ✓ `cargo test --workspace` grün
- ✓ `cargo clippy --workspace -- -D warnings` grün
- ✓ `cargo clippy --workspace --tests -- -D warnings` grün
- ✓ Latenz-Datei `.planning/phases/52-weekly-overview-performance-refactor/52-followup-latency-post-optimization.txt` existiert mit 3 Messungen + Vergleich zu Wave 0 + Wave 5
- ✓ WOP-04-Status explizit dokumentiert (FAIL mit Ursachenanalyse)
- ✓ Frontend unangetastet (`git diff -- shifty-dioxus/` = leer)
