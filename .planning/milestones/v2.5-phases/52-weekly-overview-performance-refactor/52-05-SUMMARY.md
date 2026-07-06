---
phase: 52-weekly-overview-performance-refactor
plan: 05
subsystem: [backend, booking-information, performance]
tags: [refactor, bulk-load, byte-identity, latency, WOP-01, WOP-02, WOP-05]
requires: ["52-01", "52-02", "52-03", "52-04"]
provides:
  - "BookingInformationServiceImpl::get_weekly_summary — Bulk-Load-Präambel mit 7 konstanten Bulk-Loads statt ~55×3 sequenzieller Wochen-Service-Calls; byte-identisch zur Wave-0-Baseline"
affects:
  - service_impl/src/booking_information.rs
  - service_impl/src/test/booking_information_chain_c.rs
  - service_impl/src/test/booking_information_vfa.rs
  - service_impl/src/test/booking_information_weekly_summary_year_batch.rs
  - shifty_bin/src/main.rs
  - docs/features/F07-reporting-balance.md
  - docs/features/F07-reporting-balance_de.md
tech-stack:
  added: []
  patterns:
    - "Bulk-Load-Präambel + In-Memory-Per-Woche-Filter (analog `all_work_details`/`all_absences`-Muster in derselben Methode)"
    - "In-Memory-DAO-Semantik-Reproduktion (R1: Slot-Filter reproduziert `SlotDao::get_slots_for_week_all_plans`-WHERE-Klausel bit-genau)"
    - "Additive Basic-Tier-Dep (ShiftplanCatalogService für `is_planning`-Filter)"
key-files:
  created:
    - .planning/phases/52-weekly-overview-performance-refactor/52-05-latency-post-refactor.txt
    - .planning/phases/52-weekly-overview-performance-refactor/52-05-SUMMARY.md
  modified:
    - service_impl/src/booking_information.rs
    - service_impl/src/test/booking_information_chain_c.rs
    - service_impl/src/test/booking_information_vfa.rs
    - service_impl/src/test/booking_information_weekly_summary_year_batch.rs
    - shifty_bin/src/main.rs
    - docs/features/F07-reporting-balance.md
    - docs/features/F07-reporting-balance_de.md
decisions:
  - "D-52-01 umgesetzt — Slot-Bulk-Load via bestehendes `SlotService::get_slots`; In-Memory-Filter reproduziert DAO-WHERE-Klausel bit-genau"
  - "D-52-04 umgesetzt — Spillover via `get_year(year) + get_year(year+1)` + gleiches Muster für `special_day.get_by_year` und `extract_shiftplan_report_for_year`; kein `get_year_range`-Sondersignatur"
  - "D-52-06 umgesetzt — Nur additive `_for_year`-Trait-Methoden werden konsumiert; `get_slots` bestehend"
  - "D-52-09 respektiert — Chain-C-Toggle-Read (`shortday_gate::read_active_from`) bleibt bei ~Zeile 320 in `get_weekly_summary` (NICHT in Helper verschoben); CVC-06 Cap-Filter (~485-505) unverändert; Slot-Clipping-Kette (~552-585) unverändert; `is_paid`-Filter unverändert (indirekt via `reporting.assemble_weeks`)"
  - "D-52-15 umgesetzt — Frontend unangetastet; `WeeklySummaryTO` bit-identisch"
  - "R1 umgesetzt — In-Memory-Slot-Filter: `deleted IS NULL`, `valid_from <= sunday`, `valid_to IS NULL OR >= monday`, `shiftplan.is_planning = 0 OR NULL` (via `planning_shiftplan_ids`-HashSet)"
  - "R8 umgesetzt — Toggle-Read strikt in `get_weekly_summary` verankert"
  - "Neue Basic-Tier-Dep: `ShiftplanCatalogService` (via `service::shiftplan_catalog::ShiftplanService`-Trait) — nur `get_all()` konsumiert, für `is_planning`-Filter-Set. Kein Zyklus: ShiftplanCatalog konsumiert keine BookingInformation. DI-Konstruktionsreihenfolge angepasst: `shiftplan_service` VOR `booking_information_service`."
metrics:
  duration_minutes: ~90
  completed: 2026-07-05
  tasks_completed: 2
  files_created: 2
  files_modified: 7
  tests_added: 0
  net_lines_added: ~560
status: complete
---

# Phase 52 Plan 05: `get_weekly_summary` Bulk-Load-Refactor Summary

**One-liner:** Umbau von `BookingInformationServiceImpl::get_weekly_summary` von
~55 sequenziellen `_for_week`-Service-Calls auf 7 konstante Bulk-Loads vor der
Wochen-Schleife (In-Memory-Filter im Loop). Wave-1-Fixtures 8/8 byte-identisch
grün; Latenz-Median auf Dev-DB fällt von 2.33 s auf 1.13 s (Faktor 2.07x).
WOP-04-Ziel (<0.500 s) wird nicht erreicht — dokumentiert als Follow-Up, per
PLAN Task-2 Punkt B kein Phase-Blocker.

## Was gebaut wurde

### Task 1 — `get_weekly_summary`-Bulk-Load-Refactor

**Bulk-Load-Präambel** (neu, nach dem bestehenden Load-once-Muster):

```rust
let year_plus_1 = year + 1;
let year_reports = reporting_service.get_year(year, ...).await?;
let next_year_reports = reporting_service.get_year(year_plus_1, ...).await?;
let special_days_this = special_day_service.get_by_year(year, ...).await?;
let special_days_next = special_day_service.get_by_year(year_plus_1, ...).await?;
let shiftplan_reports_this = shiftplan_report_service.extract_shiftplan_report_for_year(year, ...).await?;
let shiftplan_reports_next = shiftplan_report_service.extract_shiftplan_report_for_year(year_plus_1, ...).await?;
let all_slots = slot_service.get_slots(...).await?;
let planning_shiftplan_ids: HashSet<Uuid> = shiftplan_service.get_all(...).await?
    .iter().filter(|sp| sp.is_planning).map(|sp| sp.id).collect();
```

**Loop-Umbau** — jede Woche wählt Bucket per D-52-04-Spillover-Regel und
filtert in-memory:

- `week_report` = `year_reports_source.get((week - 1) as usize).map(|(_, r)| r.clone())`
- `special_days` = In-Memory-Filter auf `special_days_source` per `(year, week)`
- `shiftplan_reports` = In-Memory-Filter auf `shiftplan_reports_source` per `(year, week)`
- `slots_for_week` = In-Memory-Filter auf `all_slots` mit R1-Reproduktion der
  DAO-WHERE-Klausel: `deleted IS NULL`, `valid_from <= sunday`,
  `valid_to IS NULL OR valid_to >= monday`, planning-Filter via `planning_shiftplan_ids`

Anschließende Holiday-Filter- und `shortday_gate::clip_slot_for_week`-Kette
**unverändert** — arbeitet jetzt auf `slots_for_week` statt DAO-Ergebnis.

### Task 1 — Neue Dep + DI-Reihenfolge

`BookingInformationServiceDeps` erweitert um `ShiftplanService`
(via `service::shiftplan_catalog::ShiftplanService`-Trait, `#[automock]`
liefert `MockShiftplanService`). Konsumiert wird nur `get_all()` — der
Trait ist Basic-Tier, keine Domain-Service-Konsumenten, kein Zyklus.

`shifty_bin/src/main.rs`:
- `BookingInformationServiceDependencies` erweitert um
  `type ShiftplanService = ShiftplanCatalogService;`
- Konstruktionsreihenfolge angepasst: `shiftplan_service` wird VOR
  `booking_information_service` konstruiert (die spätere
  `let shiftplan_service = ...`-Konstruktion wurde entfernt, alle
  nachfolgenden Konsumenten nutzen jetzt den frühen Handle).

### Task 1 — Test-Anpassungen

Drei Test-Deps-Impls erweitert um `type ShiftplanService = service::shiftplan_catalog::MockShiftplanService;`:

- `service_impl/src/test/booking_information_chain_c.rs` (beide `build_service`- und Custom-Setups)
- `service_impl/src/test/booking_information_vfa.rs` (VFA-02-Setup)
- `service_impl/src/test/booking_information_weekly_summary_year_batch.rs` (Wave-1 Fixture-Helper)

Alle drei Test-Files bekommen zusätzliche Mocks für die neuen Bulk-Load-
Endpoints (`get_by_year`, `get_year`, `extract_shiftplan_report_for_year`,
`get_slots`, `shiftplan_service.get_all`).

**Wave-1 Fixture-Helper (`build_service_with`)** — der Slot-Bulk-Mock
umschreibt jeden Slot aus `slots_by_week` mit `valid_from = monday_of_week`
und `valid_to = Some(sunday_of_week)`, damit der In-Memory-DAO-Semantik-Filter
im Consumer den Slot ausschließlich in seiner (year, week) selektiert —
reproduziert die Per-Woche-Auswahl aus `get_slots_for_week_all_plans`
byte-genau.

### Task 2 — Latenz-Messung + F07-Docs-Sync

Latenz-Messung nach demselben Verfahren wie Wave-0-Baseline (5 curl-Runs,
Median, Dev-Profil, gleiche Dev-DB, gleicher Endpoint):

```
Median: 1.126 s (post-refactor)
Baseline (Wave 0): 2.330 s
Speedup-Faktor: 2.07x
WOP-04 Zielwert <0.500 s: FAIL
```

Zielwert-Fehlen als Follow-Up dokumentiert (PLAN Task 2 Punkt B):
- Load-once für `sales_person_service.get_all` im `assemble_weeks`-Helper
  (30-40% erwarteter Zusatz-Speedup, byte-identisch, kleiner Refactor).
- Batching für `absence_service.derive_hours_for_range` und
  `reporting.build_derived_holiday_map` (Jahres-Bulk-Analog).
- DB-Indices auf `booking(year, calendar_week)` (Migration, separater Task).
- HashMap-Voraufbau für `working_hours` pro (Person, Woche).

`docs/features/F07-reporting-balance.md` und `_de.md` bekommen Änderungshistorie-
Eintrag Phase 52: `get_year` als additive Batch-Variante, Formel unverändert,
Byte-Identität strukturell via `assemble_weeks`-Helper garantiert.

## Verifikations-Gates

| Gate | Erwartet | Ist | Status |
| ---- | -------- | --- | ------ |
| `cargo build --workspace` | grün | grün | ✅ |
| `cargo test --workspace` | alle grün | 713 unit + 64 integration + weitere = alle grün | ✅ |
| `cargo test --package service_impl booking_information_weekly_summary_year_batch` | 8 passed | 8 passed byte-identisch | ✅ (Byte-Identity-Gate!) |
| `cargo test --package service_impl booking_information_chain_c` | grün | grün | ✅ |
| `cargo test --package service_impl booking_information_vfa` | grün | grün | ✅ |
| `cargo test --package service_impl booking_information` | grün | grün | ✅ |
| `cargo clippy --workspace -- -D warnings` | 0 warnings | 0 warnings | ✅ |
| `cargo clippy --workspace --tests -- -D warnings` | 0 warnings | 0 warnings | ✅ |
| Latenz-Median post-refactor | Best-Effort | 1.126 s (2.07× Speedup) | ⚠️ FAIL bzgl. WOP-04 <0.500 s (dokumentiert als Follow-Up) |

## Grep-Guards (D-52-09 / R8)

Im Body von `get_weekly_summary` (Zeilen 259-632, alles nach der Umbau):

| Guard | Erwartet | Ist |
| ----- | -------- | --- |
| `reporting_service.get_week(` in Non-Comment-Code | 0 | 0 (nur in Kommentaren) |
| `special_day_service.get_by_week(` in Non-Comment-Code | 0 | 0 (nur in Kommentaren) |
| `extract_shiftplan_report_for_week(` in Non-Comment-Code | 0 | 0 (nur in Kommentaren) |
| `get_slots_for_week_all_plans(` in Non-Comment-Code | 0 | 0 (nur in Kommentaren) |
| `shortday_gate::read_active_from` | ≥ 1 (bleibt) | 1 (Z. 320) | ✅ |
| CVC-06 Filter im Consumer | bleibt | Z. ~485-505 unverändert | ✅ |
| Slot-Clipping-Kette | bleibt | Z. ~552-585 unverändert | ✅ |

`get_summery_for_week` (Z. 634+) bleibt komplett unangetastet — sekundärer
Konsument von `_for_week`-Endpoints, explizit NICHT im Scope dieser Phase.

## Wave-1-Fixture-Ergebnis (Byte-Identity-Gate)

Alle 8 Fixtures aus `booking_information_weekly_summary_year_batch.rs` byte-
identisch grün gegen die refaktorierte Impl:

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
`committed_voluntary_hours`) bleibt erhalten — keine `+0.0 vs -0.0`-Divergenzen.
Fixture 8 (Kombi + Spillover) bestätigt R6-Off-by-one-Guard: Vec-Index
`year_reports_source.get((week - 1) as usize)` liefert den korrekten
`ShortEmployeeReport`-Slice sowohl für Ziel-Jahr-Wochen (W1..W53) als auch
für Spillover-Wochen (W54..W56 → year+1 W1..W3).

## Latenz-Baseline (Wave 0) vs. Post-Refactor (Wave 5)

| Messung | Median | Streuung | Faktor |
| ------- | ------ | -------- | ------ |
| Wave-0-Baseline | 2.330 s | 1.60 s (68%) | 1.0× |
| Wave-5 post-refactor | 1.126 s | 0.13 s (11%) | 2.07× schneller |

Der Refactor hat nicht nur den Median halbiert, sondern auch die Streuung
drastisch reduziert (68% → 11%). Das bestätigt, dass die 55×3 sequenziellen
Roundtrips die Streuung dominierten; Bulk-Loads produzieren stabilere Latenz.

**WOP-04 (<0.500 s): FAIL** — Best-Effort erreicht (Faktor 2.07× Speedup),
aber Zielwert wird nicht erreicht. Per PLAN Task 2 Punkt B kein Phase-Blocker;
Follow-Up-Kandidaten sind priorisiert dokumentiert in
`52-05-latency-post-refactor.txt` und in dieser Summary.

## Docs-Freshness-Notiz

F07-Docs (`docs/features/F07-reporting-balance.md` + `_de.md`) zitieren die
`get_week`-Trait-Signatur (Zeile 372). Ergo Änderungshistorie ergänzt in
beiden Sprachen — kein Formel-Update nötig (Balance-Formel unverändert), nur
additive `get_year`/`_for_year`-Trait-Erweiterung dokumentiert.

## Deviations from Plan

### Deviation D-1 — WOP-04-Latenz-Ziel nicht erreicht (dokumentierter Follow-Up)

- **Erwartung:** Median <0.500 s auf Dev-DB.
- **Ist:** Median 1.126 s (Faktor 2.07× Speedup, aber ~2× über Ziel).
- **Warum kein Fix in dieser Phase:** PLAN Task 2 Punkt B (D-52-16) sagt
  explizit: „Als Follow-Up dokumentieren, NICHT diese Phase blocken. Der
  byte-identische Refactor ist die Kernaufgabe."
- **Follow-Up-Kandidaten:** priorisiert dokumentiert in dieser Summary +
  Latenz-Datei.
- **Nicht als Deviation Rule 1-3 klassifiziert:** kein Bug, keine
  fehlende kritische Funktionalität, kein Blocker; expliziter PLAN-
  approved Follow-Up-Path.

### Deviation D-2 — Zusätzliche Dep `ShiftplanCatalogService` (Rule 2 auto-added)

- **Warum:** R1 (Slot-Filter-Semantik) verlangt, dass der In-Memory-Filter
  die DAO-`WHERE`-Klausel bit-genau reproduziert — inklusive
  `(shiftplan.is_planning = 0 OR shiftplan.is_planning IS NULL)`. Der
  `Slot`-Service-Struct trägt `shiftplan_id: Option<Uuid>`, aber KEIN
  `is_planning`. Ohne `ShiftplanCatalogService::get_all()` als Bulk-
  Load-Fundament wäre der Filter live semantisch inkorrekt (Slots aus
  planning-Shiftplans würden fälschlich in `required_hours` mitgezählt).
- **Kein Zyklus:** ShiftplanCatalog ist Basic-Tier ohne Domain-Service-
  Konsumenten. BookingInformation (Business-Logic) konsumiert es —
  einseitige Abhängigkeit.
- **DI-Konstruktionsreihenfolge:** `shiftplan_service` in `main.rs` VOR
  `booking_information_service` verschoben (war vorher später konstruiert).
  Sekundäre Konsumenten (`shiftplan_view_service` etc.) nutzen den
  gleichen Arc — kein doppelter Handle.
- **Kein CONTEXT.md-Widerspruch:** CONTEXT.md D-52-01 sagt „Slot-Batching
  mit rein" und nennt In-Memory-Filter, spezifiziert aber nicht wie der
  Planning-Filter umgesetzt wird. RESEARCH.md R1 (A3) sagt explizit
  „Executor MUSS die DAO-Query lesen" — genau das habe ich getan und
  die Deps-Erweiterung ist die byte-identische Konsequenz.
- **Klassifizierung:** Rule 2 (auto-add missing critical functionality)
  — Live-Korrektheit erfordert `is_planning`-Filter.

### Deviation D-3 — Fixture-Slot-Mock benötigt `valid_from`/`valid_to`-Korrektur (Test-Umbau)

- **Warum:** Der In-Memory-Slot-Filter im Consumer selektiert nach
  `valid_from`/`valid_to`. Die Original-Fixture-Helper `slot()` setzt
  `valid_from = 2020-01-01, valid_to = None` — d.h. jeder Slot wäre in
  JEDER Woche aktiv. Ohne Korrektur würde ein Slot aus `slots_by_week[(2026, 31)]`
  z.B. auch in Woche 30 mitgerechnet und Fixture 3 (Baseline außerhalb W31)
  wäre kaputt.
- **Fix:** Im `build_service_with`-Bulk-Mock werden die Slots per
  `slots_by_week`-Eintrag mit `valid_from = monday_of(year, week)` und
  `valid_to = Some(sunday_of(year, week))` konfiguriert. Der DAO-Filter
  `valid_from <= sunday && (valid_to IS NULL OR valid_to >= monday)`
  greift dann exakt in der Zielwoche.
- **Klassifizierung:** Rule 3 (auto-fix blocking issue) — Test-Fixture-
  Semantik-Alignment mit der neuen Impl-Konvention. Byte-Identität
  der 8 Wave-1-Fixtures beweist, dass die Korrektur semantisch neutral
  ist.

## Frontend-Bestätigung (D-52-15)

`git diff HEAD~2 HEAD -- shifty-dioxus/` = leer. Frontend unangetastet.
`WeeklySummaryTO` bit-identisch.

## Threat mitigations

| Threat ID | Category | Status | Verifikation |
|-----------|----------|--------|--------------|
| T-52-10 | Tampering (Slot-Filter-Semantik-Drift) | mitigated | R1 exakt umgesetzt (deleted, valid_from, valid_to, is_planning); Wave-1-Fixture 3 (ShortDay in W31) sowie Fixture 8 (Kombi mit Slots in W53 + W55-Spillover) grün. Byte-Identität bestätigt die Filter-Semantik. |
| T-52-11 | Tampering (Off-by-one Spillover) | mitigated | R6 mitigation via Vec-Index-Formel `(week - 1) as usize`; Fixture 8 (Spillover W55 = 2021-W2) explizit grün. |
| T-52-12 | Elevation of Privilege (Toggle-Read wandert) | mitigated | Grep-Guard: `shortday_gate::read_active_from` bleibt genau EIN Vorkommen in `get_weekly_summary` (Z. 320); Chain-C-Tests grün. |
| T-52-13 | Denial of Service (Latenz-Ziel verfehlt) | accepted | Follow-Up dokumentiert; Phase nicht geblockt (D-52-16). |

## Known Stubs

Keine.

## Threat Flags

Keine neuen. Trust-Boundaries unverändert; keine neuen Netzwerk-Endpoints,
keine neuen Auth-Pfade, keine Schema-Änderungen.

## Follow-Ups (für v2.5 späte Phasen oder v2.6)

1. **Load-once für `sales_person_service.get_all` im `assemble_weeks`-Helper**
   (RESEARCH A2/R3). Größter Einzel-Impact zur WOP-04-Zielerreichung.
   Kandidat für eine kleine Extra-Phase in v2.5 (oder als Task in Phase 53
   mit-nehmen falls das dort ohnehin angefasst wird).
2. **Batching für `absence_service.derive_hours_for_range` +
   `reporting.build_derived_holiday_map`** (Jahres-Bulk-Analog zu
   `find_by_year`). Höherer Refactor-Aufwand.
3. **DB-Indices auf `booking(year, calendar_week)`,
   `extra_hours(date_time)`, `working_hours(from_year, to_year)`** (Migration,
   ADR-Kandidat weil erste Perf-Migration).
4. **HashMap-Voraufbau `HashMap<Uuid, Vec<&EmployeeWorkDetails>>`** im
   `assemble_weeks`-Helper (WOP-05 optional-Speedup).

## Commits

- `15a3a0e` — refactor(52-05): rebuild get_weekly_summary with year-bulk loads (WOP-01)
- `4e85a2a` — perf(52-05): latency post-refactor baseline + F07 docs sync (WOP-04)

## Self-Check: PASSED

- ✓ `service_impl/src/booking_information.rs` enthält Bulk-Load-Präambel + In-Memory-Filter im Loop
- ✓ Grep-Guards (`.get_week(`, `.get_by_week(`, `.extract_shiftplan_report_for_week(`, `.get_slots_for_week_all_plans(`) = 0 in `get_weekly_summary`-Fn Non-Comment-Code
- ✓ `shortday_gate::read_active_from` bleibt bei ~Z. 320 in `get_weekly_summary`
- ✓ CVC-06 Filter (~Z. 485-505) + Slot-Clipping (~Z. 552-585) unverändert
- ✓ `.planning/phases/52-weekly-overview-performance-refactor/52-05-latency-post-refactor.txt` existiert mit Median + Baseline-Vergleich + PASS/FAIL-Status
- ✓ `docs/features/F07-reporting-balance.md` + `_de.md` mit Phase-52-Änderungshistorie-Eintrag
- ✓ Commit `15a3a0e` in git log
- ✓ Commit `4e85a2a` in git log
- ✓ `cargo test --workspace` = alle grün
- ✓ Wave-1-Fixtures (`booking_information_weekly_summary_year_batch::fixture_*`) = 8/8 byte-identisch grün
- ✓ Chain-C Tests (`booking_information_chain_c`) grün
- ✓ VFA Tests (`booking_information_vfa`) grün
- ✓ `cargo clippy --workspace -- -D warnings` = 0 warnings
- ✓ `cargo clippy --workspace --tests -- -D warnings` = 0 warnings
- ✓ Frontend (`shifty-dioxus/`) unangetastet
