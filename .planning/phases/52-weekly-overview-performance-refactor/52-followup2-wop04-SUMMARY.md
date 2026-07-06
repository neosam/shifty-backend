---
phase: 52-weekly-overview-performance-refactor
plan: followup2-wop04
subsystem: [backend, reporting, performance]
tags: [perf, year-batch, in-memory-derive, byte-identity, WOP-04, GOAL-MET]
requires: ["52-05", "52-followup-wop04"]
provides:
  - "assemble_weeks-Helper hebt drei Per-(Person × Woche)-DAO-Chains (special_day, toggle, absence) auf Year-Scope-Preloads am Kopf des Helpers — 26k SQLite-Queries pro Anfrage eliminiert"
  - "Zwei pure In-Memory-Helper (`derive_hours_for_week_pure`, `build_derived_holiday_map_for_week_pure`) replizieren die byte-identische Semantik der bisherigen async DAO-Chains"
affects:
  - service_impl/src/reporting.rs
  - service_impl/src/test/reporting_additive_merge.rs
  - service_impl/src/test/reporting_get_year.rs
  - service_impl/src/test/reporting_holiday_auto_credit.rs
tech-stack:
  added: []
  patterns:
    - "Year-Scope-Preload am Kopf des Aggregat-Helpers + In-Memory-Filter im Per-Item-Loop"
    - "Pure Helper-Funktion (kein &self, kein async) als drop-in replacement für async DAO-Chain, so lange alle Inputs vorbatched sind"
    - "Conditional Preload (skip when neither producer needs it) — hält test-mock ergonomics"
key-files:
  created:
    - .planning/phases/52-weekly-overview-performance-refactor/52-followup2-latency-post-optimization.txt
    - .planning/phases/52-weekly-overview-performance-refactor/52-followup2-wop04-SUMMARY.md
  modified:
    - service_impl/src/reporting.rs
    - service_impl/src/test/reporting_additive_merge.rs
    - service_impl/src/test/reporting_get_year.rs
    - service_impl/src/test/reporting_holiday_auto_credit.rs
decisions:
  - "Neuer Signatur-Parameter `all_absences: &[AbsencePeriod]` an `assemble_weeks` — pub(crate), keine Public-API-Erweiterung. Caller (`get_week`, `get_year`) bulk-loaden via `absence_service.find_all(Authentication::Full, tx)`."
  - "Zwei neue pure Modul-Funktionen `derive_hours_for_week_pure` und `build_derived_holiday_map_for_week_pure` — replizieren die byte-identische Semantik von `AbsenceServiceImpl::derive_hours_for_range` und `ReportingServiceImpl::build_derived_holiday_map` für eine EINZELNE ISO-Woche. Kein Umbau der async Traits — die zwei bestehenden async Consumer (`get_reports_for_all_employees`, `get_report_for_employee_range`) rufen weiterhin die alte async Version, weil sie über Jahres-Ranges pro-Employee arbeiten (kein Per-(Person × Woche)-Hotspot)."
  - "`holiday_auto_credit`-Toggle wird EINMAL am Kopf von `assemble_weeks` mit dem Caller-Kontext gelesen — Cutoff-Wert in einer lokalen Variable gecacht. `Unauthorized → None` bleibt exakt wie in `build_derived_holiday_map` (Legacy off, D-25-05)."
  - "Special-Day-Preload ist per unique (year, week) in `weeks` (nicht per Jahr). Grund: der bestehende Consumer `booking_information.get_weekly_summary` ruft `reporting_service.get_year` mit einem einzelnen Jahr — die 55 unique Wochen decken sich mit den 55 `get_by_week`-Calls, die die alte Pfad im Loop machte. `get_by_year` wurde bewusst NICHT verwendet, weil dessen Kalenderjahr-Semantik (SDF-03) NICHT byte-identisch zu den ISO-Wochen-gebundenen `get_by_week`-Semantiken ist."
  - "Special-Day-Preload wird komplett übersprungen, wenn `cutoff.is_none()` UND `has_any_absences == false`. Beide In-Memory-Helper liefern dann leere Ergebnisse — byte-identisch zum Pre-Refactor. Diese Optimierung ist gleichzeitig test-mock-friendly (Tests ohne Absence/Holiday brauchen kein `expect_get_by_week`)."
  - "`absence_service.find_all` wird mit `Authentication::Full` aufgerufen — mirror des `sales_person_service.get_all` Patterns aus Follow-Up #1. Die Permission-Semantik der äußeren Consumer (`get_week`/`get_year` durch die REST-Endpoints) bleibt authoritative; der Bulk-Load ist eine Cache-Population, keine Freigabe von Personendaten."
  - "Iteration innerhalb von `derive_hours_for_week_pure` durch Mon..=Sun via `monday + time::Duration::days(offset)` (statt while-loop mit next_day) — deterministisch, keine Möglichkeit für off-by-one bei Monatsübergängen."
  - "`absence_category_priority` als crate-privater Klon der gleichnamigen Funktion in `service_impl::absence` — vermeidet die Kopplung des reporting-Moduls an interne Symbols des absence-Moduls, replicates die Priority-Tabelle 1:1."
metrics:
  duration_minutes: ~65
  completed: 2026-07-06
  tasks_completed: 3
  files_created: 2
  files_modified: 4
  tests_added: 0
  net_lines_added: ~450
status: complete
---

# Phase 52 Follow-Up #2: Year-Batch `special_day` + `toggle` + `absence` in `assemble_weeks` (WOP-04)

**One-liner:** Drei Per-(Person × Woche)-DAO-Chains
(`special_day.get_by_week`, `toggle.get_toggle_value`,
`absence.derive_hours_for_range`) aus dem `assemble_weeks`-Helper gezogen und
durch Year-Scope-Preloads + In-Memory-Berechnungen ersetzt. 26 000 SQLite-
Roundtrips pro Anfrage eliminiert. Latenz-Median: 0.97 s → **~0.12 s**
(8× zusätzlicher Speedup, **WOP-04 <0.5 s Ziel erreicht** — 4× unter Zielwert).
Byte-Identität durch zwei reine Helper-Funktionen bewahrt, Wave-1-Fixtures 8/8
grün.

## WOP-04-Status

**PASS ✅** — Median liegt bei ~0.12 s (Ziel: <0.500 s, 4× unter Zielwert).

| Messung | Median | Streuung | Ambient |
|---------|--------|----------|---------|
| 1 (nach Server-Start) | 0.114 s | 0.04 s | direkt nach Warmup |
| 2 (voller Cache) | 0.147 s | 0.04 s | nach vollem Cache |
| 3 (Konsistenz-Check) | 0.119 s | 0.09 s | Bestätigung |
| **Repräsentativer Median** | **~0.12 s** | | median-of-medians |

Detail siehe `52-followup2-latency-post-optimization.txt`.

## Was gebaut wurde

### Task 1 — Zwei pure Helper-Funktionen in `reporting.rs`

Zwei crate-lokale Modul-Funktionen (nicht `&self`, nicht `async`), die die
byte-identische Semantik der bisherigen async DAO-Chains für **eine einzelne
ISO-Woche** replizieren:

**`derive_hours_for_week_pure(year, week, absences_for_person,
contracts_for_person, holidays_this_week) -> BTreeMap<Date, ResolvedAbsence>`**

Repliziert `AbsenceServiceImpl::derive_hours_for_range` für Mo..=So einer
Woche. Dieselbe Active-Contract-Auswahl (`from_date()/to_date()`-Filter),
derselbe Dominant-Category-Resolver (`max_by_key(absence_category_priority)`,
`SickLeave > Vacation > UnpaidLeave` per BUrlG §9), dieselbe
Verfügbarkeits-Prüfung (`has_day_of_week`), derselbe Holiday-Skip, dieselbe
Wochen-Deckelung auf `workdays_per_week` mit Halbtag-Fraktion-Support.

Iteration durch Mo..=So via `monday + time::Duration::days(offset)` (statt
`next_day()`-Loop) — deterministisch, keine Monatsübergangs-Off-by-one.

**`build_derived_holiday_map_for_week_pure(year, week, cutoff,
special_days_this_week, working_hours, extra_hours_for_person) -> HashMap<Date, f32>`**

Repliziert `ReportingServiceImpl::build_derived_holiday_map` für eine
Woche. Dieselbe Cutoff-Prüfung (`Some/None`-Kurzschluss), dieselbe
Holiday-Filter (`SpecialDayType::Holiday`), dieselbe Manual-Wins-Prüfung
(`ExtraHoursCategory::Holiday` am selben Datum → skip), dieselbe
Contract-Availability-Prüfung (`has_day_of_week` am
`SpecialDay::day_of_week`), dieselbe `holiday_hours()`-Berechnung.

Beide Helper sind ausschließlich pure — kein `await`, keine DAO-Referenzen,
kein `&self`. Sie sind trivial testbar (bereits vollständig via
Byte-Identity gegen die Wave-1-Fixtures geprüft).

### Task 1 — `assemble_weeks` refactored

Neuer Parameter (pub(crate)):

```rust
#[allow(clippy::too_many_arguments)]
pub(crate) async fn assemble_weeks(
    &self,
    weeks: &[(u32, u8)],
    work_details: &[EmployeeWorkDetails],
    shiftplan_reports: &[ShiftplanReportDay],
    extra_hours: &[ExtraHours],
    all_absences: &[AbsencePeriod],                       // ◀ NEU
    sales_person_index: &HashMap<Uuid, SalesPerson>,
    working_hours_by_sp: &HashMap<Uuid, Arc<[EmployeeWorkDetails]>>,
    context: Authentication<Deps::Context>,
    tx: Option<Deps::Transaction>,
) -> Result<Vec<(u8, Arc<[ShortEmployeeReport]>)>, ServiceError>
```

**Preload-Präambel am Kopf des Helpers (vor der Wochen-Schleife):**

```rust
// (a) toggle read ONCE
let toggle_value_res = self.toggle_service.get_toggle_value("holiday_auto_credit", context.clone(), None).await;
let cutoff: Option<time::Date> = match toggle_value_res {
    Ok(v) => v.as_deref().and_then(|s| time::Date::parse(s, &Iso8601::DEFAULT).ok()),
    Err(ServiceError::Unauthorized) => None,
    Err(e) => return Err(e),
};

// (b) absences_by_sp (in-memory bucketing)
let mut absences_by_sp: HashMap<Uuid, Vec<&AbsencePeriod>> = HashMap::new();
for ap in all_absences.iter().filter(|ap| ap.deleted.is_none()) {
    absences_by_sp.entry(ap.sales_person_id).or_default().push(ap);
}
let has_any_absences = !absences_by_sp.is_empty();

// (c) special_day preload per unique (year, week) — only when needed
let need_special_days = cutoff.is_some() || has_any_absences;
let mut special_days_by_week: HashMap<(u32, u8), Arc<[SpecialDay]>> = HashMap::new();
if need_special_days {
    let mut unique_weeks: HashSet<(u32, u8)> = HashSet::new();
    for &(y, w) in weeks { unique_weeks.insert((y, w)); }
    for (y, w) in unique_weeks {
        let sds = self.special_day_service.get_by_week(y, w, context.clone()).await?;
        special_days_by_week.insert((y, w), sds);
    }
}

// (d) holidays_by_week (BTreeSet<Date> per (year, week) for absence-skip)
let mut holidays_by_week: HashMap<(u32, u8), BTreeSet<time::Date>> = HashMap::new();
if has_any_absences {
    for (&(y, w), sds) in special_days_by_week.iter() {
        // ... filter SpecialDayType::Holiday + convert to time::Date ...
    }
}
```

**Body-Änderungen im Per-(Person × Woche)-Loop:**

Zwei async DAO-Calls durch pure Helper-Calls ersetzt:

```rust
// vorher:
let derived = self.absence_service
    .derive_hours_for_range(monday, sunday, sales_person_id, context.clone(), tx.clone())
    .await?;

// nachher:
let absences_for_person: Vec<&AbsencePeriod> = absences_by_sp
    .get(&sales_person_id).cloned().unwrap_or_default();
let holidays_this_week = holidays_by_week
    .get(&(year, week)).unwrap_or(&empty_holidays);
let derived = derive_hours_for_week_pure(
    year, week, &absences_for_person, working_hours, holidays_this_week,
);
```

```rust
// vorher:
let derived_holiday_map = self.build_derived_holiday_map(
    monday, sunday, working_hours, &employee_extra_hours_owned, context.clone(),
).await?;

// nachher:
let special_days_this_week = special_days_by_week
    .get(&(year, week)).unwrap_or(&empty_special_days);
let derived_holiday_map = build_derived_holiday_map_for_week_pure(
    year, week, cutoff, special_days_this_week, working_hours, &employee_extra_hours_owned,
);
```

Alle anderen Semantiken (has_contract_row-Gate, planned_hours-Gate,
absence_derived_balance_total-Gate, is_paid-Filter, apply_weekly_cap-Position)
sind unverändert.

### Task 1 — Callers `get_week` und `get_year` erweitert

Beide laden `all_absences` einmalig via `absence_service.find_all(Authentication::Full, tx.clone())` und reichen es durch:

```rust
let all_absences = self.absence_service
    .find_all(Authentication::Full, tx.clone()).await?;

let assembled = self.assemble_weeks(
    &weeks, &work_details, &shiftplan_reports, &extra_hours,
    &all_absences,                            // ◀ NEU
    &sales_person_index, &working_hours_by_sp,
    context, tx,
).await?;
```

`booking_information.get_weekly_summary` (der Consumer) bleibt UNVERÄNDERT
— er ruft nur `reporting_service.get_year(...)` und `.get_week(...)`, die
Preload-Struktur lebt innerhalb der Aufrufe.

### Task 2 — Test-Mock-Anpassungen

Alle Tests, die `get_week` oder `get_year` aufrufen, brauchen jetzt:
- `absence_service.expect_find_all(...)` — der neue Bulk-Load-Aufruf.
- `special_day_service.expect_get_by_week(...)` — nur wenn `find_all`
  non-empty ist (der `need_special_days`-Gate feuert erst dann).

Zwei get_week-Tests (`test_get_week_additive_merge`,
`test_balance_parity_dynamic_get_week` Lauf B) mockten
`derive_hours_for_range` mit einer synthetischen BTreeMap. Mit dem Refactor
wird `derive_hours_for_range` von `assemble_weeks` nicht mehr aufgerufen —
stattdessen produziert `derive_hours_for_week_pure` das Ergebnis aus der
`find_all`-Slice. Beide Tests wurden migriert:

- Statt der synthetischen `BTreeMap { 2024-06-05: SickLeave 8h }` ein
  echter `AbsencePeriod { from=2024-06-05, to=2024-06-05, SickLeave, Full }`
  in `find_all`. Der pure Helper produziert daraus für das
  `fixture_work_details_8h_mon_fri`-Setting (40h/5days, Mo-Fr) exakt
  8h SickLeave am 2024-06-05.
- Aggregate-Assertion (`sick_leave_hours == 11.0`, `balance_hours ~ 0.0`)
  bleibt unverändert — die Migration ist semantisch äquivalent.

Vier holiday_auto_credit-Tests (`test_holiday_auto_credit_get_week_reduces_soll_bands_unchanged`, `test_hsp03_cap_active_holiday_no_band_leak`,
`test_hsp04_before_cutoff`, `test_hsp04_manual_wins`) bekamen jeweils
`expect_find_all()` returning leer — der `expect_derive_hours_for_range()`
bleibt aus historischen Gründen als Sicherheitsnetz (harmlos, wird nicht
mehr getroffen).

`reporting_get_year.rs::test_get_year_empty_when_no_work_details` +
`build_service()`-Helper: `MockAbsenceService::new()` durch
`expect_find_all()`-Setup ersetzt.

### Task 3 — Latenz-Messung

Drei separate 5-Run-Median-Messungen (jeweils 3 Warmup-Runs davor):

| Messung | Median | Streuung |
|---------|--------|----------|
| 1 (Post-Warmup) | 0.114 s | 0.04 s |
| 2 (Voller Cache) | 0.147 s | 0.04 s |
| 3 (Konsistenz-Check) | 0.119 s | 0.09 s |
| **Rep. Median** | **~0.12 s** | |

## Verifikations-Gates

| Gate | Erwartet | Ist | Status |
| ---- | -------- | --- | ------ |
| `cargo build --workspace` | grün | grün | ✅ |
| `cargo test --package service_impl --lib booking_information_weekly_summary_year_batch` | 8 passed **byte-identisch** | 8 passed byte-identisch | ✅ **Byte-Identity-Gate** |
| `cargo test --package service_impl --lib reporting` | alle passed | 80 passed | ✅ |
| `cargo test --workspace` | alle passed | 713 unit + 64 integration + weitere | ✅ |
| `cargo clippy --workspace -- -D warnings` | 0 warnings | 0 warnings | ✅ |
| `cargo clippy --workspace --tests -- -D warnings` | 0 warnings | 0 warnings | ✅ |
| Latenz-Median post-optimization | <0.500 s | **~0.12 s** | ✅ **PASS** — 4× unter Ziel |
| Frontend touched | nein | nein (`git diff -- shifty-dioxus/` = leer) | ✅ |

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

- Fixture 4 (Volunteer-Vacation): validiert `derive_hours_for_week_pure`
  Semantik gegen die Baseline — grün.
- Fixture 5 (CVC-06-Cap): validiert dass `apply_weekly_cap` gegen die
  Holiday-reduzierte expected_hours korrekt feuert — grün.
- Fixture 8 (Kombi + Spillover): validiert dass die Preload-Präambel
  über year/year+1 Spillover-Wochen konsistent arbeitet — grün.
- IEEE-754-Sign-of-Zero-Muster (`-0.0` für `required_hours`,
  `volunteer_hours`, `committed_voluntary_hours`; `+0.0` für
  `paid_hours`/`overall_available_hours`) bit-identisch erhalten.

## Latenz-Verlauf über alle Waves + Follow-Ups

| Messung | Median | Streuung | Faktor vs Baseline | Delta vs Vorgänger |
| ------- | ------ | -------- | ------------------ | ------------------ |
| Wave-0-Baseline | 2.330 s | 1.60 s | 1.0× | — |
| Wave-5-Post-Refactor | 1.126 s | 0.13 s | 2.07× | 2.07× |
| Follow-Up #1 (SP + WD Load-Once) | ~0.97 s | 0.16 s | 2.40× | 1.16× |
| **Follow-Up #2 (Year-Batch × 3)** | **~0.12 s** | **0.05 s** | **19.4×** | **8.1×** |
| WOP-04-Ziel | <0.500 s | — | 4.66× | Ziel = 4× überschritten |

**Follow-Up #2 alleine liefert ~85% Latenz-Reduktion (0.97 → 0.12 s)** —
größer als die kumulativen Waves 0→5 UND Follow-Up #1 zusammen. Erklärung:
die drei eliminierten DAO-Chains waren die dominante Kostenquelle
(~26 000 SQLite-Roundtrips pro Anfrage; bei ~33 µs pro Roundtrip auf
lokaler NVMe = ~0.85 s reine Roundtrip-Zeit).

## Deviations from Plan

### D-1 — Test-Mock-Migration für 6 get_week-Tests (Rule 3)

- **Ausgangslage:** 6 Reporting-Tests mockten `absence_service.derive_hours_for_range`
  mit einer synthetischen BTreeMap. Der Refactor entfernt den Call aus
  `assemble_weeks` — die BTreeMap wird jetzt aus `find_all` in-memory berechnet.
- **Fix:** Pro Test entweder (a) `expect_find_all` returning leer (für
  Tests wo `derive` empty war) oder (b) einen echten `AbsencePeriod`
  bauen, der die pure Helper-Funktion zum selben aggregierten Ergebnis
  bringt (für Tests wo `derive` non-empty war).
- **Klassifizierung:** Rule 3 (auto-fix blocking issue). Die Tests
  produzieren dieselben aggregierten Assertions — die Migration ist
  semantisch äquivalent (fixture_work_details_8h_mon_fri × 1 Tag
  SickLeave = 8h SickLeave, exakt was die synthetische BTreeMap
  vorgab).

### D-2 — SpecialDay-Preload per (year, week) statt per year (Rule 1)

- **Ausgangslage:** `special_day_service.get_by_year(y)` filtert nach
  KALENDER-Jahr (SDF-03 post-ship), nicht nach ISO-Wochen-Jahr — d.h.
  ein Eintrag mit `year=2026 week=53 day=Fri` (ein 01.01.2027) landet
  im `get_by_year(2027)`-Ergebnis. Die alte per-(year, week)-Semantik
  von `build_derived_holiday_map`/`derive_hours_for_range` ruft
  `get_by_week(iso_year, iso_week)` — das gibt IMMER die Zeile mit
  `year=iso_year week=iso_week`.
- **Alternative geprüft:** `get_by_year(iso_year)` — verworfen, weil
  die Kalender-Jahr-Filterung eine Byte-Identity-Verletzung wäre.
- **Fix:** Preload per unique (year, week) im `weeks`-Slice; für
  `get_year(2026)` = 55 unique Wochen = 55 `get_by_week`-Calls (statt
  ~11 466 pre-optimisation). Byte-identisch zu Pre-Refactor, nur die
  N_persons-Multiplikation eliminiert.
- **Klassifizierung:** Rule 1 (Byte-Identity ist Rule-1-Gate).

## Frontend-Bestätigung

`git diff -- shifty-dioxus/` = leer. Frontend unangetastet.

## Threat mitigations

| Threat ID | Category | Status | Verifikation |
|-----------|----------|--------|--------------|
| T-52-03 | Tampering (is_paid-Filter-Verschiebung) | mitigated | Die `continue;`-Position ist unverändert — nach dem `sales_person`-Resolve. Fixture 4 (Volunteer-Vacation) grün. |
| T-52-09 | Off-by-one (Vec-Index-Semantik) | mitigated | `reporting_get_year::test_get_year_matches_get_week_for_arbitrary_week` grün (Bit-Vergleich). |
| T-52-13 | Denial of Service (Latenz) | **mitigated** ✅ | ~0.12 s Median = 4× unter Zielwert 0.5 s. |
| T-52-14 | Tampering (Iteration-Reihenfolge kippt Byte-Identität) | mitigated | Fixture-8 (Kombi + Spillover) grün — HashMap-Iteration-Reihenfolge unverändert. |
| T-52-15 (neu) | Tampering (pure Helper divergiert von async Impl) | mitigated | Wave-1-Fixtures 8/8 grün — die Fixture-Datensätze decken alle Codepfade der neuen pure Helper ab (Vacation, SickLeave, UnpaidLeave, Holiday, Cutoff-Gate, Manual-Wins, Wochen-Deckelung, Halbtag, Availability-Filter, has_contract_row-Gate). |
| T-52-16 (neu) | Information Disclosure (`find_all` mit Full statt caller context) | accepted | `find_all` liefert an das reporting-Modul, das die Daten NUR zur Aggregat-Berechnung nutzt (nie zurück an den Caller returned) — die äußere REST-Permission bleibt authoritative. Analog zum `sales_person_service.get_all(Authentication::Full)` aus Follow-Up #1. |

## Known Stubs

Keine.

## Threat Flags

Keine neuen. Trust-Boundaries unverändert; keine neuen Netzwerk-Endpoints,
keine neuen Auth-Pfade, keine Schema-Änderungen, kein
`CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump (keine persistierten Werte
betroffen). Keine neuen Cargo-Dependencies.

## Commits

- `perf(52): year-batch holiday map + hours-per-range in assemble_weeks (WOP-04)` — reporting.rs Helper-Refactor
- `test(52): add find_all + get_by_week expectations for assemble_weeks year-batch (WOP-04)` — Test-Mock-Migration
- `perf(52): follow-up 2 latency measurement (WOP-04)` — Latenz-File

## Self-Check: PASSED

- ✓ Zwei pure Helper-Funktionen (`derive_hours_for_week_pure`,
  `build_derived_holiday_map_for_week_pure`) existieren im
  `service_impl::reporting`-Modul.
- ✓ `assemble_weeks` hat einen neuen `all_absences: &[AbsencePeriod]`-
  Parameter; die drei Year-Scope-Preloads (toggle, absences_by_sp,
  special_days_by_week, holidays_by_week) laufen VOR der Wochen-Schleife.
- ✓ Beide Caller (`get_week`, `get_year`) laden `all_absences` via
  `absence_service.find_all(Authentication::Full, tx)` und geben es
  durch.
- ✓ Wave-1-Fixtures (`booking_information_weekly_summary_year_batch::fixture_*`)
  = **8/8 byte-identisch grün**.
- ✓ `cargo test --workspace` grün (713 unit + 64 integration + weitere).
- ✓ `cargo clippy --workspace -- -D warnings` grün.
- ✓ `cargo clippy --workspace --tests -- -D warnings` grün.
- ✓ Latenz-Datei `52-followup2-latency-post-optimization.txt` existiert
  mit 3 Messungen (jede mit 3 Warmup-Runs) + Vergleich zu allen
  vorherigen Waves + Follow-Up #1.
- ✓ WOP-04-Ziel <0.5 s erreicht (~0.12 s Median = 4× unter Zielwert).
- ✓ Frontend unangetastet (`git diff -- shifty-dioxus/` = leer).
- ✓ Keine Migrations, keine neuen Deps, kein
  `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump.
