# F14 — Voluntary Rebooking

> **Kurzfassung:** Gedeckelte Mitarbeiter mit `committed_voluntary`
> dürfen nicht in ein dauerhaftes Minus-Konto laufen. F14 liefert das
> Datenmodell-Fundament, die audit-fähige Batch-Struktur und die
> HR-only-Sicht "Voluntary Stats" (F1/F2), die zeigt, wie weit der/die
> Mitarbeiter*in vom Jahres-Freiwillig-Soll entfernt ist. Die gesamte
> End-to-End-Rebooking-Pipeline verteilt sich auf Milestone v2.6,
> Phasen 54..56.

**Cluster ID:** F14
**Status:** shipped (Phase 54 = F1/F2-Baseline; Phase 55 = F3 manuelle
Umbuchung + F5 HR-Alert approve/reject; Phase 56 = F4 Auto-Cron-Scheduler
+ Rollout-Backfill-Endpoint).
**Zuerst eingeführt:** Milestone v2.6, Phase 54 (2026-07-07). F3
(manuelle One-Shot-Umbuchung) und F5 (Claim-on-Suggest-Lifecycle:
suggest / approve / reject) shipped in Phase 55 (2026-07-23). F4
(Auto-Cron-Scheduler + Backfill) shipped in Phase 56 (2026-07-26).
**Zuständige Crates:** `service::{rebooking_batch, voluntary_stats,
rebooking_reconciliation, voluntary_rebooking_scheduler}`,
`service_impl::{rebooking_batch, voluntary_stats, rebooking_reconciliation,
voluntary_rebooking_scheduler, voluntary_rebooking_gate}`,
`dao::rebooking_batch`, `dao_impl_sqlite::rebooking_batch`,
`service_impl::reporting` (vier neue pure fns + die kategoriebasierte
Aggregation mit ihrer einen `source`-abhängigen Ausnahme in
`assemble_weeks` + das `has_pending_rebooking`-Alert-Flag),
`rest::{report, rebooking}` (voluntary-stats + F3/F5-Routes + die F4
`/admin/rebooking/backfill`-Route),
`rest-types::{VoluntaryStatsTO, rebooking}`,
`shifty-dioxus::component::voluntary_stats_row`.

---

## 1. Purpose

Manche Mitarbeiter haben Verträge mit `has_hour_cap = true` (Phase 26)
plus einem `committed_voluntary`-Wert in `employee_work_details`
(Phase 34). Der Cap verhindert, dass der bezahlte Shiftplan-Stunden-
Anteil der Balance den/die Mitarbeiter*in über den Vertrag hinaus
vergütet, aber Freiwillig-Arbeit (Kategorie `Volunteer`) wird additiv
gebucht und zählt sehr wohl in die Balance. Wenn ein/e gedeckelte/r
Mitarbeiter*in gleichzeitig ein bezahltes Minus aufbaut (Krankheit,
Feiertage bei `holiday_auto_credit` off, Absence-Reduktion) und ein
Freiwillig-Plus, müssen die beiden gegeneinander gebucht werden —
sonst bleibt die Balance-Zeile dauerhaft rot, obwohl die Person
faktisch ihren Vertrag plus Freiwillig-Extra geleistet hat.

**Milestone v2.6 liefert eine dreistufige Pipeline:**

1. Zeige dem/der Mitarbeiter*in (F1), wie viele Freiwillig-Stunden pro
   Vertragswoche durchschnittlich anfallen, und (F2), wie weit die
   Person vom Jahres-Soll (`committed_voluntary` pro-rata) entfernt
   ist.
2. HR schlägt ein Rebooking vor (F3): ein Batch, der einige
   `Volunteer`-Stunden in ein gleichwertiges gepaartes
   `Rebooking`-Source-Paar in `extra_hours` umbucht, sodass die
   Balance-Kette den Ausgleich sieht, ohne den Audit-Pfad zu
   verlieren.
3. Sobald genehmigt (F5) — und ab Phase 56 auch automatisch durch
   einen admin-gesteuerten Cron (F4) vorgeschlagen — schreibt der
   Batch die gepaarten `extra_hours`-Zeilen atomar; beide mit
   `extra_hours.source = 'rebooking'`.

## 2. Feature Slices

| Slice | Milestone / Phase | Status | Zweck |
| --- | --- | --- | --- |
| F1 (Ø Freiwillig pro Vertragswoche) | v2.6 Phase 54 | shipped | HR-only Ø — Σ Volunteer / Vertragswochen im ISO-Jahr. |
| F2 (Freiwillig-Soll + Delta) | v2.6 Phase 54 | shipped | HR-only Sicht auf `committed_voluntary`-pro-rata-Soll vs. Ist + Delta. |
| F3 (manuelle One-Shot-Umbuchung) | v2.6 Phase 55 | shipped | HR bucht eine gewählte Stundenzahl in einem Schuss um; Batch landet als `kind = Manual`, `state = Approved`, gepaarte `extra_hours` atomar geschrieben. |
| F4 (Auto-Cron-Scheduler + Backfill) | v2.6 Phase 56 | shipped | `VoluntaryRebookingScheduler` (Montag-03:15-Cron) verarbeitet die Vorwoche für jede gedeckelte Person via `run_for_week` (`kind = AutoCron`), plus der Rollout-Endpoint `POST /admin/rebooking/backfill` (`kind = AutoCronBackfill`). Stichtag-Toggle-gesteuert (`voluntary_rebooking_auto_active_from`); seit Phase 57 auf Balance 0 gedeckelt (REB-AUTO-06). **Snapshot-Schema in Phase 57 auf 12 → 13 gebumpt** (D-57-10; der Phase-56-„kein Bump"-Zweig ist hinfällig — siehe §9.4). Siehe §9. |
| F5 (HR-Alert suggest / approve / reject) | v2.6 Phase 55 | shipped | Claim-on-Suggest (Modell B): `suggest` claimt einen `Pending`-`HrSuggestion`-Batch; `approve` schreibt die gepaarten Zeilen, `reject` transitioniert nur den State. |

**Faustregel:** F3 (manuell) und F5 (HR-Alert) shippen in Phase 55
(siehe §8 unten); F4 (Auto-Cron + Backfill) shippt in Phase 56 (siehe
§9 unten).

## 3. Marker-Filter-Regel ([D-54-DM-02])

`extra_hours` bekommt die additive Spalte `source TEXT NOT NULL
DEFAULT 'manual'`. Die aktiven Domain-Werte sind `manual` und
`rebooking`.

- **`manual`** — jede Zeile, die über die bestehenden UI-Pfade
  geschrieben wird (HR-CRUD, Absence-Convert-Vacation-Writer, Dev-Seed,
  REST-TO → Service-Mapper). Bestandszeilen landen per Column-DEFAULT
  auf `manual`.
- **`rebooking`** — die gepaarten Zeilen, die F3/F4/F5 seit Phase 55
  schreiben. Seit Phase 57 ist der Wert ein reines **Provenienz**-Feld:
  UI-Kennzeichnung, Audit-Queries und spätere Undo-Fähigkeit. Er hat
  keine pauschale Aggregations-Wirkung mehr.

**Reader-Regel (Stand nach D-57-06, VOL-ACCT-04).** Die Fachlogik trägt
die **Kategorie-Semantik**, nicht der Marker:

- `ExtraHoursCategory::as_report_type()` entscheidet über die
  Balance-Relevanz — `ExtraWork` wird zu `WorkingHours` (zählt in die
  bezahlte Balance), `VolunteerWork` zu `Documented` (zählt nicht).
- `ExtraHoursCategory::availability()` entscheidet über die Anwesenheit.

Ein Rebooking-Paar über `h` Stunden hebt damit die Balance um `h` (das
`+h ExtraWork`-Bein) und senkt das Ehrenamt-Ist um `h` (das
`−h VolunteerWork`-Bein) — genau die Wirkung, für die es das Feature
gibt.

**Genau eine `source`-abhängige Ausnahme bleibt.** In `assemble_weeks`
(dem Wochenübersichts-Pfad, `service_impl/src/reporting.rs`) wird die
balance-relevante Summe über `availability()` gebildet, und
`VolunteerWork.availability()` ist `Available`. Dort wird die Kombination
`VolunteerWork` **und** `source = Rebooking` gezielt ausgenommen —
andernfalls würde das `−h`-Bein die Anhebung durch das `+h`-Bein
neutralisieren und das Paar wäre an dieser einen Stelle netto 0. In der
Ehrenamt-Ist-Summe derselben Funktion (`manual_volunteer_hours`) bleibt
dieselbe Zeile bewusst enthalten: ohne sie würde das Ehrenamt-Ist nicht
sinken.

**Vorbestehende Inkonsistenz, bewusst nicht angetastet (RISK-01).**
Gewöhnliche (Nicht-Rebooking-)`VolunteerWork`-Stunden zählen in der
Wochenübersicht in `balance_hours`/`overall_hours`, im Jahresreport aber
nicht — weil die beiden Pfade die Balance unterschiedlich herleiten
(`availability()` vs. `as_report_type()`). Phase 57 hat das **nicht**
angefasst; die obige Site-1-Ausnahme ist genau deshalb auf
`source = Rebooking` beschränkt, damit das Verhalten gewöhnlicher
Freiwilligen-Zeilen unverändert bleibt.

**Historien-Hinweis.** Bis Phase 56 stand an den Aggregations-Eingängen
aller vier Report-Pfade (`get_week`, `get_reports_for_all_employees`,
`hours_per_week`, `overall_extra_work_hours`, D-55-A1) ein unbedingter
Marker-Filter (`source == ExtraHoursSource::Manual`, behalte `Manual`,
schließe `Rebooking` aus). Er war mit „Doppelzählung" begründet und
stammte aus der ursprünglichen Fassung von VOL-ACCT-03, die
Balance-Neutralität forderte. Diese Forderung war selbst der Fehler: der
Filter machte den **gesamten** Übertrag wirkungslos — ein Paar änderte
nirgends etwas. Neu gefasst in Phase 57 (VOL-ACCT-04); an drei Sites ist
der Filter ersatzlos entfallen, an der vierten auf die obige Ausnahme
verengt.

**Audit-Regel:** `rebooking`-Source-Zeilen bleiben in der DB und bleiben
in *Audit*-Queries sichtbar — sie sind die Antwort auf „warum hat sich
die Balance an diesem Datum geändert?" (F5).

**Was neutral bleibt (VOL-ACCT-03, in Phase 57 neu gefasst):** nur das
**Soll**. `voluntary_soll` stammt aus `committed_voluntary` im Vertrag
und liest überhaupt keine `extra_hours`, ein Rebooking-Paar lässt es also
bit-identisch. Das ist die einzige verbliebene Neutralitäts-Garantie,
festgenagelt durch `test_neutrality_f2_soll`
(`service_impl/src/test/rebooking_neutrality.rs`). Die Wirkung auf
Balance und Ist nageln `test_neutrality_balance`,
`test_neutrality_f1_ist`, `rebooking_neutrality_single_manual`
(`service_impl/src/test/rebooking_reconciliation.rs`),
`rebooking_effective_in_weekly_summary` (der Site-1-Wächter) und das
Ende-zu-Ende-Referenz-Szenario in
`service_impl/src/test/rebooking_reference_scenario.rs`
(8 h bezahlt / 12 h Ehrenamt / Balance −2 → 10 / 10 / 0) fest.

## 4. Batch Structure

Zwei SQLite-Tabellen, angelegt in Migration
`20260707000000_create-rebooking-batch.sql`.

### `rebooking_batch` — Parent-Row

| Spalte | Typ | Anmerkung |
| --- | --- | --- |
| `id` | BLOB(16) PK | UUID v4. |
| `sales_person_id` | BLOB(16) FK | Auf welchen Mitarbeiter das Batch bucht. |
| `iso_year` | INT | ISO-Jahr des Reconciliation-Fensters. |
| `iso_week` | INT | ISO-Woche innerhalb `iso_year` (siehe UNIQUE unten). |
| `kind` | TEXT | `Manual` \| `HrSuggestion` \| `AutoCron` \| `AutoCronBackfill` (Phase 55/56 Writer). |
| `state` | TEXT | `Pending` \| `Approved` \| `Rejected` \| `SkippedLocked`. |
| `created`, `approved`, `approved_by` | TEXT | ISO-Zeitstempel + User-Name; `approved*` NULL bis state = Approved. |
| `deleted` | TEXT nullable | Soft-Delete-Marker. |
| `update_process`, `update_version` | Audit-Spalten |

**Constraint [D-54-DM-01]:** partieller UNIQUE-Index
`rebooking_batch_week_unique_idx` auf
`(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL` —
*global über alle Kinds*. Rationale: Claim-on-Suggest — sobald HR
einen Pending-Batch für Woche X öffnet, darf der F4-Cron (Phase 56)
nicht mit einem zweiten AutoCron-Batch für dieselbe Woche
reinlaufen. Der partielle Index (Soft-Delete-aware) ist die
Enforcement-Stelle.

### `rebooking_batch_entry` — Payload pro Slot

| Spalte | Typ | Anmerkung |
| --- | --- | --- |
| `id` | BLOB(16) PK |  |
| `batch_id` | BLOB(16) FK → `rebooking_batch(id)` | Kein CASCADE — Soft-Delete-Muster. |
| `sales_person_id` | BLOB(16) | Denormalisiert für Query-Performance. |
| `hours` | REAL | Absolute Stundenzahl, die das Entry umbuchen will. |
| `balance_before` | REAL | Balance-Snapshot zum Vorschlagszeitpunkt (Audit). |
| `voluntary_actual` | REAL | Tatsächliche Ist-Freiwillig-Stunden zum Vorschlagszeitpunkt. |
| `voluntary_committed` | REAL | Pro-rata-Soll zum Vorschlagszeitpunkt (F2-Zähler). |
| `extra_hours_out_id`, `extra_hours_in_id` | BLOB(16) nullable | FKs in `extra_hours` — gesetzt beim Übergang state → Approved (F3/F5-Writer, Phase 55). |
| `created`, `deleted`, `update_process`, `update_version` | Audit-Spalten |

**Regel:** `extra_hours_out_id` + `extra_hours_in_id` sind `NULL`,
solange `state = Pending`. Sie werden atomar in derselben Transaktion
gesetzt, die `state = Approved` flippt — so garantiert F5, dass die
gepaarten `extra_hours`-Zeilen mit dem Batch-State konsistent sind.

## 5. Services (Phase 54 Baseline)

| Service | Tier | Zweck |
| --- | --- | --- |
| `RebookingBatchService` | Basic | HR-gated CRUD (find_by_id / find_by_sales_person_year_week / create) auf `rebooking_batch` + Entries, plus der Phase-55 F5-Modell-B-Lifecycle: `create_pending_suggestion` (Claim-on-Suggest-Producer — INSERTet einen `HrSuggestion`/`Pending`-Batch und claimt den UNIQUE-Wochen-Slot; belegter Slot → `EntityAlreadyExists`), `find_pending_for_sales_person` (Reader fuer `pending_rebooking_id`, liefert nur `Pending`), `update_state` (race-sichere state-conditional Transition `Pending → Approved\|Rejected`; ein zweiter/bereits entschiedener Wechsel → `EntityAlreadyExists` via `rows_affected()==0`, kein version-Guard; nimmt `approved_by` fuer den Approver-Audit) und `create_entries_for_batch` (haengt Entries an einen **bestehenden** Batch — genutzt von F5-`approve`, um nach dem Pair-Write die Audit-Kette zu schliessen, da `create` einen neuen Batch INSERTet). Deps: `RebookingBatchDao`, `PermissionService`, `ClockService`, `UuidService`, `TransactionDao`. Kein Domain-Service-Dep. |
| `VoluntaryStatsService` | Business-Logic | Read-only F1/F2. Deps: `ExtraHoursService`, `EmployeeWorkDetailsService`, `SalesPersonService`, `PermissionService`, `TransactionDao`. HR-only per API-Level None-Redaktion (nicht 403). |
| `RebookingReconciliationService` | Business-Logic | Phase 55. Der F3/F5-Write-Orchestrator. Besitzt den einzigen internen Pair-Write-Path, geteilt von F3 und F5-approve. Deps: `ExtraHoursService`, `RebookingBatchService`, `ReportingService`, `VoluntaryStatsService`, `PermissionService`, `TransactionDao`. Alle vier Methoden HR-gated (T-55-05). Siehe §8. |

**Konsumenten-Wiring (Phase 55).** `RebookingBatchService` (Basic) hat
seine ersten Konsumenten bekommen: `RebookingReconciliationService`
(F3/F5-Lifecycle) und `ReportingService` (liest
`find_pending_for_sales_person` für das Alert-Banner).
`RebookingReconciliationService` wird DI-**nach** allen Basic-Services
und nach `ReportingService` + `VoluntaryStatsService` in
`shifty_bin/src/main.rs` konstruiert (Reihenfolge: `extra_hours` →
`rebooking_batch` → `reporting` → `voluntary_stats` →
`rebooking_reconciliation`, zuletzt — kein `OnceLock` / Forward-Decl).
`VoluntaryStatsService` wird weiterhin von
`rest/src/report.rs::get_voluntary_stats` konsumiert; die vier
F3/F5-Routes in `rest/src/rebooking.rs` konsumieren
`RebookingReconciliationService`.

**Service-Tier-Hinweis.** Gemäß Konventionen in
`shifty-backend/CLAUDE.md` ist `RebookingBatchService` Basic (nur DAO
+ Permission + Clock + UUID + Transaction). `VoluntaryStatsService`
ist Business-Logic (konsumiert drei andere Domain-Services). Die
Unterscheidung ist im Runtime-Graph verankert — siehe
[`../architecture/diagrams/service-graph-runtime.mmd`](../architecture/diagrams/service-graph-runtime.mmd).

### Aggregations-Modell in `VoluntaryStatsService`

`VoluntaryStatsService` ist dünn. Zwei Verantwortlichkeiten:

**Ist (VOL-STAT-01 / VOL-ACCT-01-Ist):** delegiert an
`ReportingService::get_report_for_employee_range` und liest
`EmployeeReport::volunteer_hours` für den angeforderten Range. Dieses
Aggregat deckt alle drei Quellen ab — manuelle VolunteerWork-ExtraHours,
Shiftplan-Cap-Überlauf (`auto_volunteer_hours`) und
no_contract-Shiftplan-Stunden — konsistent zum OVERALL-"Ehrenamt"-Wert
auf der Employee-Detail-Seite. Weil das Ist delegiert wird, fließt die
Rebooking-Wirkung automatisch mit ein: seit Phase 57 senkt das
`−h VolunteerWork`-Bein `volunteer_hours` um `h` (§3) — dieser Service
hält keinen eigenen Filter.

**Soll + contract-weeks:** zwei Range-basierte pure fns neben
`committed_voluntary_prorata_for_week` (internal per-week Baustein) in
`service_impl/src/reporting.rs`:

```rust
/// F1-Nenner / D-F1-01 — Anzahl ISO-Wochen im Range mit mindestens
/// einem Vertragstag im Range. `expected_hours = 0` zählt MIT.
/// Edge-Weeks zählen als 1 (tages-basierte Verdünnung passiert im
/// Zähler, nicht hier).
///
/// v2.6.1 (D-54.5-02): eine Woche mit mindestens einem Absence-Tag
/// desselben Freiwilligen wird aus dem Zähler ausgeklammert
/// (whole-week-out / ganze-Woche-raus).
pub fn contract_weeks_count_in_range(
    working_hours: &[EmployeeWorkDetails],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
    absences: &[AbsencePeriod],
) -> u32;

/// D-F2-01 — tages-pro-rata für eine einzelne ISO-Woche mit
/// tagesgenau aktiver EmployeeWorkDetails (Mid-Week-Kontraktwechsel).
/// Bleibt als internal per-week Baustein für Debug-Tests.
pub fn committed_voluntary_prorata_for_week(
    working_hours: &[EmployeeWorkDetails], year: u32, week: u8) -> f32;

/// F2-Soll = Σ (committed_voluntary / 7.0) über jeden Range-Tag mit
/// aktivem Vertrag. Edge-Weeks tragen pro-rata für die Tage im Range
/// bei (D-F2-01 bleibt tages-basiert). (Phase 54 Gap-Closure G1 —
/// Range-basiert löst die frühere Full-Year-Variante ab.)
///
/// v2.6.1 (D-54.5-01): jede ISO-Woche, die mit mindestens einem
/// aktiven Absence-Tag desselben Freiwilligen überlappt (Vacation,
/// SickLeave, UnpaidLeave — kategorie-agnostisch), trägt `0` zum
/// Soll bei (whole-week-out / ganze-Woche-raus, nicht pro-rata pro
/// Tag).
pub fn committed_voluntary_target_in_range(
    working_hours: &[EmployeeWorkDetails],
    from_date: ShiftyDate,
    to_date: ShiftyDate,
    absences: &[AbsencePeriod],
) -> f32;
```

**Rationale — Range-basierte Aggregation (Phase 54 Gap G1):** konsistent
mit `ReportingService::get_report_for_employee_range`; Edge-Weeks
tragen pro-rata für die Tage im Range bei. Ohne Cutoff lieferte eine
5h/Woche-Zusage ab Mai ein Full-Year-Ziel, das den tatsächlichen
Report-Zeitraum um ~4x überschoss (~177h vs. realistisch ~54h für
Jan–Juli). Siehe 54-UAT.md Gap G1.

### Absence-bewusstes ganze-Woche-raus (v2.6.1, D-54.5-01 / D-54.5-02 / D-26-03)

Beide Range-basierten pure fns bekommen einen zusätzlichen Parameter
`absences: &[AbsencePeriod]` — die pro-Freiwilligen gefilterte Liste
aktiver (`deleted.is_none()`) Absence-Records — und wenden ein
**whole-week-out** an:

- **Soll (`committed_voluntary_target_in_range`, D-54.5-01):** jede
  ISO-Woche, deren Mo–So-Kalenderbereich mit mindestens einem
  Absence-Tag des Freiwilligen überlappt, trägt `0` zum Soll bei —
  kategorie-agnostisch (Vacation, SickLeave, UnpaidLeave). Nicht
  pro-rata pro Absence-Tag.
- **Contract-Weeks-Nenner (`contract_weeks_count_in_range`,
  D-54.5-02):** derselbe Overlap klammert die Woche aus dem Nenner
  aus. Damit misst `ist_per_contract_week` den Durchschnitt über die
  Wochen, die tatsächlich für Freiwilligenarbeit **verfügbar** waren.
- **Overlap-Helper:** die Prüfung nutzt `period_overlaps_week`
  (`service_impl/src/booking_information.rs:75`) als Single Source of
  Truth, geteilt mit der Weekly-Anzeige (VFA-01 / D-26-03).
- **Rationale — Ist/Soll-Symmetrie:** die Weekly-Anzeige
  (`WeeklySummary.committed_voluntary_hours`) hat Absence-Wochen
  bereits seit v2.6.0 auf 0 gesetzt; `EmployeeReport::volunteer_hours`
  (die Ist-Quelle) ist während Absence-Wochen faktisch auch 0 (kein
  Shiftplan, kein manuelles VolunteerWork). Die Angleichung der
  Soll-Aggregation entfernt die systematische Überschätzung, die das
  Delta wie eine legitime Freiwilligen-Verpflichtungs-Lücke aussehen
  ließ (~15 h pro 3 Absence-Wochen bei einer 5-h/Woche-Zusage).
- **Bewusste Revision zu D-F1-01 für diesen Konsumpfad:** die
  ursprüngliche F1-Regel (`expected_hours = 0` zählt MIT) bleibt
  intakt; Absence-Wochen fallen zusätzlich raus. Die Revision ist auf
  `VoluntaryStatsService` beschränkt; andere Konsumenten von
  `contract_weeks` sind nicht betroffen.
- **Non-HR-Path lädt niemals Absences.** Der `AbsenceService`-Load
  läuft nur im HR-Path; die Non-HR-Redaktion (alle Felder `null`)
  greift vor jedem Datenabruf (`service_non_hr_does_not_load_absences`
  Regressions-Test).

**Changelog:** v2.6.1 — `committed_voluntary_target_in_range` +
`contract_weeks_count_in_range` sind Absence-bewusst (whole-week-out,
D-54.5-01 / D-54.5-02). Siehe Phase `54.5-voluntary-soll-absence-fix`.

**v2.6.1-Nachtrag (Quick-Task 260710) — Voluntary-Erfüllungsgrad:**
`VoluntaryStats` (und der DTO-Spiegel `VoluntaryStatsTO`) bekommt ein
sechstes Feld `ist_per_soll_pct: Option<f32>` = `ist_total /
soll_total * 100` — der Erfüllungsgrad in Prozent. `None`, wenn
`soll_total ≈ 0` (Division-by-zero-Guard: Nicht-Freiwillige oder ein
Range, der komplett in Absence-Wochen fällt). Werte können > 100 %
sein, wenn Ist > Soll (Freiwillige über-erfüllt). Die FE-Zeile wird
ausgeblendet, wenn das Feld `None` ist.

## 6. REST (Phase 54)

| Methode | Pfad | DTO In | DTO Out | Auth |
| --- | --- | --- | --- | --- |
| `GET` | `/report/{id}/voluntary-stats?from_date=YYYY-MM-DD&to_date=YYYY-MM-DD` | — | `VoluntaryStatsTO` | jede authentifizierte Session; HR-only Inhalt — Non-HR erhält alle Felder = `null`. |

`VoluntaryStatsTO` (5 Felder, alle `Option<f32>`/`Option<u32>`,
serde `#[serde(default)]` für Wire-Kompatibilität):

- `ist_per_contract_week` — F1 (Ø Freiwillig / Vertragswoche).
- `ist_total` — F2-Ist (absolute Ehrenamt-Summe im Range).
- `soll_total` — F2-Soll (`committed_voluntary` pro-rata über den Range).
- `delta` — `ist_total − soll_total`.
- `contract_weeks` — F1-Nenner (Audit).
- `ist_per_soll_pct` — Erfüllungsgrad in Prozent (`ist_total /
  soll_total * 100`), `None` wenn `soll_total ≈ 0`.

**Query-Vertrag:** sowohl `from_date` als auch `to_date` sind inklusive
ISO-8601-Daten (`YYYY-MM-DD`). Ungültiges Format oder
`from_date > to_date` → HTTP 400 (Präzedenz `rest/src/toggle.rs`).

**Redaktions-Regel:** die Redaktion passiert **innerhalb**
`VoluntaryStatsService::get_voluntary_stats`, nicht in der REST-
Schicht (Präzedenz VAC-OFFSET-01 v1.8). Non-HR erhält HTTP 200 mit
allen Feldern = `null`. HR sieht die konkreten Werte.

**Prefix-Proxy:** die Route liegt im Axum-Tree unter `/report`. Der
bestehende `[[web.proxy]]`-Eintrag in `shifty-dioxus/Dioxus.toml` für
`/report` deckt sie ab — kein neuer Proxy-Eintrag notwendig.

## 7. Related Features

- **F04 Extra Hours** — die neue Spalte `source` liegt auf der
  `extra_hours`-Tabelle; seit Phase 57 ist sie reine Provenienz
  (siehe §3).
- **F07 Reporting / Balance** — die Read-Aggregate filtern nicht mehr
  nach Provenienz; die Kategorie-Semantik entscheidet
  (`as_report_type()` für die Balance-Relevanz, `availability()` für die
  Anwesenheit), mit der einen `assemble_weeks`-Ausnahme aus §3. Ein
  Rebooking-Paar hebt die Balance um `h` und senkt das Ehrenamt-Ist um
  `h`; neutral bleibt nur das VOL-ACCT-01-Soll (VOL-ACCT-03/04). Siehe
  F07 §2.6.
- **F08 Billing Period Snapshot** — **in Phase 57 auf 12 → 13 gebumpt**
  (D-57-10). Ohne den Marker-Filter erreichen Rebooking-Zeilen die
  persistierten Werte; die `value_type`s `Overall`, `Balance`,
  `ExtraWork` und `Volunteer` verschieben sich für eine Periode mit
  Rebooking-Woche um `±h`. Das ist ein „Change the input set the
  computation reads from" im Sinne der CLAUDE.md-Klausel und damit
  Bump-Pflicht. Der Phase-56-REB-AUTO-05-„kein Bump"-Zweig ist hinfällig
  — siehe §9.4 und F08 §7.4.
- **F13 System-Infrastruktur** — der Toggle
  `voluntary_rebooking_auto_active_from` (in Phase 54 mit
  `enabled = 0`, `value = NULL` geseedet) gattet in Phase 56 den
  F4-Cron. In Phase 54 ist er inaktiv.

## 8. F3 Manuelle Umbuchung + F5 HR-Alert (Phase 55)

Phase 55 hängt die Write-Seite an. Der Orchestrator ist der
Business-Logic-`RebookingReconciliationService`
(`service::rebooking_reconciliation`,
`service_impl::rebooking_reconciliation`).

### 8.1 Single Pair-Write-Path (geteilt von F3 und F5-approve)

Es gibt genau **einen** internen Pair-Writer,
`rebook(direction, hours, week)` — eine private Impl-Methode — genutzt
von F3 (`rebook_manual`) UND F5 (`approve`). Er schreibt das `−N`
(Quell-Kategorie) / `+N` (Ziel-Kategorie) `extra_hours`-Paar, beides mit
`source = 'rebooking'` gestempelt, in der Transaktion des Aufrufers. Das
geteilte Write-Path verhindert F3/F5-Drift (Pitfall 15).
`RebookingDirection` ist explizit (D-55-06): `VolunteerToExtra` (−N
VolunteerWork / +N ExtraWork, der Standard-Minus-Ausgleich) oder
`ExtraToVolunteer` (Korrektur überzahlter Zusagen). F5-`approve` nutzt
fest `VolunteerToExtra`.

### 8.2 F3 — One-Shot-manuelle-Umbuchung (REB-MANUAL-01/02/03)

`rebook_manual(sales_person_id, direction, hours, iso_year, iso_week,
…)` HR-gatet zuerst, **validiert dann den Betrag serverseitig** (`hours >
0`, sonst `ValidationError` → HTTP 422 — der Client-seitige Modal-Guard
ist nicht autoritativ; ein direkter API-Aufruf mit `0`/negativen Stunden
wird abgelehnt, bevor eine Row entsteht), dann in **einer** Transaktion:
schreibt das Paar (§8.1), legt einen `rebooking_batch` mit `kind =
Manual`, `state = Approved` an und einen `rebooking_batch_entry` mit den
beiden Pair-ids. Es gibt **keinen** suggest/pending-Zwischenschritt. Ein
belegter UNIQUE-Wochen-Slot → `EntityAlreadyExists` → HTTP 409.

### 8.3 F5 — Claim-on-Suggest-Lifecycle (Modell B, D-55-B)

Der F5-Lifecycle ist **Claim-on-Suggest** (Modell B): der pending-Batch
wird zum *suggest*-Zeitpunkt erzeugt, nicht erst beim approve.

- **`suggest(sales_person_id, iso_year, iso_week, …)`** — HR-gated;
  claimt den UNIQUE-Wochen-Slot via
  `RebookingBatchService::create_pending_suggestion`, das einen
  `kind = HrSuggestion`, `state = Pending`-Batch INSERTet. Idempotent
  bei bereits offenem pending (liefert dieselbe id); ein **entschiedener**
  Slot → `EntityAlreadyExists` → HTTP 409 (D-55-05, HR-ALERT-04).
  Liefert die backend-computed IST/DANN-Vorschau `RebookingSuggestion`:
  `balance`, `voluntary_ist` (IST), `balance_after = balance +
  proposed_hours`, `voluntary_ist_after = voluntary_ist −
  proposed_hours` (DANN) und `proposed_hours = min(|balance|,
  voluntary_ist)` (D-55-03). Die Suggestion-Struktur trägt **nur** diese
  backend-computed Felder — es gibt kein `voluntary_soll` /
  `voluntary_delta` darauf.
- **`approve(rebooking_batch_id, hours, …)`** — HR-gated. Der **Betrag
  ist backend-autoritativ** (Fat-Backend / Modell B): der Client-Wert
  `hours` wird **verworfen**; `approve` recomputet `proposed_hours =
  min(|balance|, voluntary_ist)` fuer die Batch-Woche mit exakt derselben
  Berechnung wie `suggest` (Report-Balance + VoluntaryStats ueber die
  Mo..=So-Range). Ist der recomputete Betrag `<= 0` (nichts umzubuchen),
  liefert es `ValidationError` → HTTP 422 — **keine** state-Transition,
  **kein** Paar. Sonst transitioniert es den State **zuerst** via
  `RebookingBatchService::update_state` — ein state-conditional UPDATE `…
  WHERE id = ? AND state = 'pending' AND deleted IS NULL` mit
  `rows_affected() == 1`-Guard (Pitfall 12), wobei der anfragende
  HR-Principal als `approved_by` (Audit) durchgereicht wird — und
  schreibt nur bei Erfolg das Paar (§8.1, `VolunteerToExtra`, der
  backend-`proposed`-Betrag) in derselben Transaktion. Ein
  Double-Approve-Verlierer (`rows_affected() == 0`) →
  `EntityAlreadyExists` → HTTP 409. `approve` legt **keinen** zweiten
  Batch an; es transitioniert den bestehenden pending-Batch und haengt
  nach dem Pair-Write via
  `RebookingBatchService::create_entries_for_batch` einen
  `rebooking_batch_entry` an diesen **bestehenden** Batch — mit den
  beiden Pair-ids + den Audit-Snapshots (`hours`, `balance_before`,
  `voluntary_actual`, `voluntary_committed`) — und schliesst so die
  Audit-Kette Batch → Entry → ExtraHours symmetrisch zu F3.
- **`reject(rebooking_batch_id, …)`** — HR-gated; transitioniert den
  State `Pending → Rejected` via denselben conditional UPDATE. **Keine**
  Pair-Rows.

**Claim-on-Suggest / Phase-56-Cron-Konsequenz:** weil der pending-Batch
bereits den UNIQUE-`(sales_person, iso_year, iso_week)`-Slot claimt
(partieller Index, §4), kann der F4-Auto-Cron (Phase 56) keinen zweiten
`AutoCron`-Batch in eine Woche reinlaufen, für die HR einen offenen
Vorschlag hat — `state = pending` blockiert den Cron-Slot.

### 8.4 HR-Alert-Flag auf dem Short-Report (HR-ALERT-01, D-55-01)

`ReportingService::get_reports_for_all_employees` berechnet ein
per-Mitarbeiter-Alert-Flag, exponiert als
`ShortEmployeeReport.has_pending_rebooking` + `pending_rebooking_id`
(gespiegelt auf `ShortEmployeeReportTO`, `#[serde(default)]`,
wire-additiv). Das Alert-Prädikat ist
`balance <= -0.5 ∧ voluntary_ist > 0 ∧ cap_active`
(`alert_predicate`, `service_impl/src/reporting.rs`, grenzgetestet bei
−0.49 / −0.50 / −0.51). Die Banner-Regel (D-55-B): das Flag ist `true`
nur, wenn das Prädikat gilt **und** kein *entschiedener* Batch im Slot
liegt — ein entschiedener (approved oder rejected) Batch unterdrückt das
Banner, der Alert erlischt also nach **beidem**, approve UND reject
(D-55-07). `pending_rebooking_id = Some(pending.id)`, wenn ein offener
pending-Batch existiert. Es gibt **kein Undo** (D-55-04). Um den
Read-Path günstig zu halten, läuft der Batch-Lookup nur für
Prädikat-Kandidaten.

**Klarstellung (D-57-04) — das Banner-Verhalten ist unverändert.** Der
Alert erlischt weiterhin bei Approve, bei Reject und beim belegten
UNIQUE-Wochen-Slot. Er ist ein **Arbeitswerkzeug für HR**, kein Wächter
dagegen, dass der Übertrag erneut wirkungslos wird. Eine Entkopplung von
D-55-07 wurde vom User ausdrücklich verworfen („Solange bis HR auf
annehmen oder ablehnen klickt. Dann hat HR entschieden und es gibt keine
Frage mehr in der Woche."). Die Wächter-Rolle tragen strukturell die
Property-Tests in `service_impl/src/test/rebooking_neutrality.rs` und
`service_impl/src/test/rebooking_reference_scenario.rs` — sie werden rot,
sobald ein Rebooking-Paar die Balance nicht mehr bewegt, und nicht eine
Sichtprüfung des Banners.

### 8.5 REST-Routes (Phase 55, `rest/src/rebooking.rs`)

Alle vier Routes sind HR-gated (am Service) und OpenAPI-dokumentiert;
das Status-Mapping (409 / 403 / 422) liegt zentral im `error_handler`.

| Methode | Pfad | Zweck |
| --- | --- | --- |
| `POST` | `/rebooking/manual` | F3 One-Shot-manuelle-Umbuchung. |
| `POST` | `/rebooking-suggestions` | F5 suggest / claim (bewusste Mutation, Modell B). |
| `POST` | `/rebooking-suggestions/{id}/approve` | F5 approve → schreibt das Paar. |
| `POST` | `/rebooking-suggestions/{id}/reject` | F5 reject → nur State. |

Die optionale read-only `GET /rebooking-suggestions`-Liste wurde
**nicht** implementiert (kein list-pending-Service/DAO existiert in
55-01/02; einen hinzuzufügen wäre eine Rule-4-Architektur-Änderung).

## 9. F4 Auto-Cron + Rollout-Backfill (Phase 56)

Phase 56 setzt die **Automatisierungs-Fläche** auf den Phase-55-Write-Path
obendrauf. Der One-Shot-F3-Write und der F5-approve-Lifecycle brauchen
einen Menschen; F4 bucht dieselben Rebooking-Paare automatisch, Woche für
Woche, für jede gedeckelte Person — plus ein einmaliges
Backfill-Kommando, das beim Rollout historische Wochen nachzieht. Seit
Phase 57 ist die Automatik **auf Balance 0 gedeckelt** (REB-AUTO-06,
§9.1).

### 9.1 Die trigger-agnostische Engine — `run_for_week`

`RebookingReconciliationService::run_for_week(sales_person_id, iso_year,
iso_week, kind, …)` ist der **einzige Write-Path**, geteilt von Cron und
Backfill (Plan 02). Er ist trigger-agnostisch: der Aufrufer übergibt den
`RebookingBatchKind` (`AutoCron` für den Wochen-Cron, `AutoCronBackfill`
für den Backfill), und die Engine macht das gesamte Gating intern, sodass
die beiden Trigger niemals driften können:

- **Excess (seit Phase 57 gedeckelt, REB-AUTO-06)** — die Automatik
  gleicht das **bezahlte Stundenkonto** aus, aber nur so weit, wie der
  Mitarbeiter seine Freiwilligen-Zusage noch erfüllt. Die Balance steigt
  durch die Automatik nie über 0. Formel (`week_excess`,
  `service_impl/src/rebooking_reconciliation.rs`):

  ```
  min( max(ist − committed_voluntary, 0), max(−balance, 0) )
  ```

  Nicht-endliche Eingaben liefern `0.0` (NaN/Inf-Guard auf allen vier
  Parametern). Nichts umzubuchen oder eine Balance ≥ 0 →
  `Skipped { NoExcess }`.

  **Mindest-Buchungsgröße (`MIN_BOOKABLE_HOURS = 0.01`, WR-02).** Der
  Guard lautet `excess < MIN_BOOKABLE_HOURS → Skipped { NoExcess }`, nicht
  `excess <= 0.0`. `balance` ist eine f32-Summe über Shiftplan-,
  ExtraHours- und Carryover-Beiträge; nach einem exakt ausgleichenden
  Übertrag landet sie praktisch nie auf bit-exakt `0.0`, sondern auf einem
  Residuum wie `-1.2e-6`. Ein strikter `> 0.0`-Guard ließe den Write-Pfad
  darauf anlaufen — teuer ist dabei nicht das
  `-0.0000012 / +0.0000012`-Paar, sondern der Batch-Claim: er belegt den
  UNIQUE-Slot `(person, iso_year, iso_week)`, sodass eine spätere legitime
  manuelle Umbuchung oder ein HR-`suggest` für dieselbe Woche in
  `EntityAlreadyExists` (HTTP 409) läuft, obwohl fachlich nie etwas
  gebucht wurde. Dieselbe Schwelle gilt für den Backfill-Preview
  (`would_book`, §9.3), wo die Dry-Run-Headroom-Buchführung das Residuum
  ganz ohne Datenbank-Roundtrip erzeugt.

  **Balance-Quelle (D-57-07):** die kumulierte **year-to-date**-Balance
  ab dem **Montag der ISO-Woche 1** des `iso_year`
  (`ShiftyDate::new(iso_year, 1, DayOfWeek::Monday)`) bis einschließlich
  Sonntag der verarbeiteten Woche, **inklusive** Vorjahres-Carryover
  (`include_carryover = true`) — *nicht* die Balance zum Laufzeitpunkt
  und *nicht* die isolierte Wochen-Range, die `suggest()`/`approve()`
  benutzen. Begründung: ein Backfill über mehrere historische Wochen
  würde sonst denselben Spielraum mehrfach vergeben.

  **Warum ISO-Anker und nicht der 1. Januar (CR-01):**
  `ShiftyDate::first_day_in_year(y)` geht über `from_ymd(y, 1, 1)` und
  speichert das Ergebnis als ISO-Tripel. Fällt der 1. Januar auf Fr/Sa/So
  (2027, 2028, 2021, 2022, 2023), gehört er zum **vorherigen** ISO-Jahr —
  `first_day_in_year(2027)` ist ISO-Woche 53/2026.
  `get_report_for_employee_range` liest `from_date.year() - 1` für den
  Carryover-Lookup und iteriert `hours_per_week` ab
  `from_date.as_shifty_week()`; ein Kalender-Anker lädt damit das falsche
  Carryover-Jahr und zieht eine volle Vorjahres-Woche in die Balance. Da
  der Wert `max(-balance, 0)` und damit den geschriebenen Betrag steuert,
  ist das ein Write-Fehler. Getestet in
  `voluntary_rebooking.rs::run_for_week_balance_range_is_iso_anchored_in_2027`
  und `rebooking_backfill.rs::backfill_balance_range_is_iso_anchored_in_2027`.

  **Abgrenzung zum HR-Vorschlag (D-57-01, Invariante I-5):** der
  HR-Alert-Vorschlag aus §8.3 bleibt bei `proposed_hours =
  min(|balance|, voluntary_ist)` und ist bewusst **nicht** durch die
  Zusage gedeckelt — er darf sie unterschreiten, genau darin liegt der
  Sinn des Features; seine Untergrenze ist das Jahres-Ehrenamt-Ist von 0.
  Zwei Untergrenzen, zwei Zuständigkeiten: die Automatik stoppt bei
  Balance 0, der von einem Menschen bestätigte Vorschlag bei
  Ehrenamt-Ist 0.

  **Referenz-Szenario (D-57-02):** Vertrag 5 h bezahlt + 5 h freiwillig
  pro Woche, zwei Wochen; Woche 1 mit 3 h, Woche 2 mit 17 h → 8 h bezahlt
  / 12 h freiwillig / Balance −2, und ein Übertrag von 2 h landet auf
  10 / 10 / 0. Gegenprobe Übererfüllung: 23 h gearbeitet bei 20 h
  zugesagt überträgt trotzdem nur **2**, nicht 5 — der Deckel bindet.
  Beide Hälften sind ausführbar in
  `service_impl/src/test/rebooking_reference_scenario.rs`, die
  Schreibseite mit denselben Zahlen in
  `voluntary_rebooking.rs::run_for_week_balance_cap_binds`.
- **Idempotenz (REB-AUTO-03):** eine Woche, die bereits einen aktiven
  `rebooking_batch` trägt (früherer Cron-Lauf, manueller F3-Batch oder
  offener pending-HR-Vorschlag), wird geskippt — `Skipped { AlreadyProcessed }`.
  Der partielle UNIQUE-Index `(sales_person, iso_year, iso_week)` (§4)
  ist die Enforcement-Stelle; Claim-on-Suggest (§8.3) bedeutet, dass ein
  offener pending-Batch den Cron-Slot blockiert.
- **Wochen-Sperre (D-04):** eine `WeekStatus::Locked`-ISO-Woche schreibt
  einen `state = SkippedLocked`-**Audit-Batch** (kein Paar) und liefert
  `Skipped { WeekLocked }` — die Balance einer geschlossenen Woche wird
  nie mutiert, aber der Skip bleibt im Audit-Trail sichtbar.
- **Stichtag-Toggle (write-seitiger Self-Guard, D-06):** die Engine liest
  `voluntary_rebooking_auto_active_from` frisch bei jedem Lauf
  (`voluntary_rebooking_gate::should_run_for_week`, der ISO-Wochen-Montag
  ≥ `active_from`-Cutoff, analog zu `shortday_gate`). Toggle nicht gesetzt
  oder Montag der Woche strikt vor `active_from` → `Skipped { ToggleOff }`
  — ein voller No-Op, kein Batch geschrieben.

### 9.2 Der Wochen-Scheduler — `VoluntaryRebookingScheduler`

`VoluntaryRebookingScheduler` (Business-Logic-Tier,
`service::voluntary_rebooking_scheduler` /
`service_impl::voluntary_rebooking_scheduler`) ist ein dünner
`tokio-cron-scheduler`-Job, der `PdfExportSchedulerImpl` minus
`reload_from_db` spiegelt:

- **Hardcoded 6-Feld-Cron `0 15 3 * * 1`** — Montag 03:15
  (`VOLUNTARY_REBOOKING_CRON`), konfliktfrei gegen den
  pdf-export-Montag-06:00-Job (D-03). Sechs Felder sind unter
  `tokio-cron-scheduler` Pflicht (ein 5-Feld-Ausdruck erzeugt still einen
  schlafenden Job — T-56-09).
- **`run_once_now`** berechnet die **vorherige** abgeschlossene ISO-Woche
  (jahresgrenzsicher: `ShiftyDate::from_date(clock.date_now()).as_shifty_week()`
  + ein `previous_week`-Step-Back — nie ein naives `year()`/`week()`),
  iteriert dann jede gedeckelte Person aus
  `EmployeeWorkDetailsService::all_for_week` und delegiert jede an
  `run_for_week(…, AutoCron, …)`. Ein Per-Person-Fehler wird geloggt und
  die Schleife läuft weiter (T-56-07 — keine Panic-Propagation aus der
  Cron-Closure); es loggt eine `{booked} booked, {skipped} skipped,
  {failed} failed`-Zusammenfassung.
- **Kein Cron-String aus dem Toggle kompiliert:** anders als ein
  Stichtag-im-Cron-Ansatz wird der Toggle bei jedem Lauf frisch innerhalb
  `run_for_week` gelesen, sodass ein Flippen von `active_from` am nächsten
  Montag ohne Reload greift. Deshalb hat der Scheduler kein
  `reload_from_db`.
- **DI-Wiring:** minimaler Dep-Set — `RebookingReconciliationService`
  (die Engine), `EmployeeWorkDetailsService` (die Gedeckelte-Personen-
  Quelle), `PermissionService`, `ClockService`, `TransactionDao`. Gebootet
  in `shifty_bin/src/main.rs` am Ende von `main()` **direkt nach**
  `pdf_export_scheduler.start()`. Er hat **keine** REST-Fläche (der
  Backfill-Endpoint ist eine separate Route auf
  `RebookingReconciliationService`), daher ist er ein reines
  `RestStateImpl`-Feld, kein `RestStateDef`-Accessor.

### 9.3 Der Rollout-Backfill-Endpoint

`POST /admin/rebooking/backfill` (`rest/src/rebooking.rs::trigger_backfill`,
genestet unter `/admin/rebooking`) ist ein dünner Delegat an
`RebookingReconciliationService::backfill_range`:

| Query-Param | Bedeutung |
| --- | --- |
| `from` | ISO-Woche `YYYY-Www` (z.B. `2026-W20`); malformed / Woche `0` / Woche > `weeks_in_year` → **HTTP 400** *bevor* der Service läuft (T-56-11 DoS-Bound — keine teure Iteration auf Müll-Input). |
| `dry_run` | `true` = nur Vorschau (kein Write); `false` (oder fehlend) = Apply. Jeder andere Wert → HTTP 400. |
| `page`, `per_page` | Optionale 1-basierte Pagination der Kandidatenliste (Multi-Year-Backfills werden nicht in einer Response abgeladen, D-05). |

- **HR-gated:** `backfill_range` prüft zuerst `HR_PRIVILEGE`
  (`service_impl/src/rebooking_reconciliation.rs`); ein Nicht-HR-Aufrufer
  bekommt **HTTP 403**. Die Route liegt im `/admin/rebooking`-Nest.
- **Single Write-Path:** der Apply-Zweig delegiert jede Kandidaten-Woche
  an dieselbe `run_for_week`-Engine mit `kind = AutoCronBackfill`, sodass
  eine backfillte Woche exakt wie eine Cron-Woche gegated wird
  (Idempotenz, Wochen-Sperre, Toggle) — der einzige Unterschied ist der
  Batch-`kind`, der Audit-Queries erlaubt, einen Rollout-Sweep vom
  laufenden Wochen-Cron zu unterscheiden.
- **Response** ist ein `BackfillPreviewTO` (Kandidaten-Items mit den
  berechneten `voluntary_ist` / `voluntary_soll` / `committed_voluntary` /
  `excess` / `would_book` / `skipped`), identische Shape für dry-run und
  Apply, sodass der HR-Admin vor dem Commit eine Vorschau bekommt.
- **Cutoff-Gate vor den teuren Reads (WR-06).** Die Stichtag-Prüfung
  läuft **pro Woche, vor** den Report- und VoluntaryStats-Reads der
  Kandidaten-Schleife — spiegelbildlich zu `run_for_week`, wo der
  Balance-Read bewusst hinter dem Toggle-Gate steht. Folge für die
  Response: ein Item mit `skip_reason = ToggleOff` trägt `0.0` in
  `voluntary_ist` / `voluntary_soll` / `committed_voluntary` / `excess` —
  diese Kennzahlen werden für eine Woche vor dem Stichtag **nicht
  berechnet**. Grund ist der DoS-Bound T-56-05: ein
  year-to-date-Report ist selbst O(Wochen im Jahr), und ein historischer
  Backfill, dessen `from` vor dem Stichtag liegt — der Normalfall —,
  zahlte sonst pro (Person, Woche) einen vollen Report plus
  VoluntaryStats, nur um anschließend jede Zeile zu verwerfen.
  Festgenagelt von `backfill_toggle_off_week_skips_before_expensive_reads`,
  dessen Report- und Stats-Mocks bei jedem Aufruf paniken.
- **Preview == Apply (Invariante I-7, seit Phase 57).**
  `BackfillPreviewItem.excess` wendet **denselben Balance-Deckel** an wie
  der Schreib-Pfad; der Vorschau-Wert ist also exakt der Betrag, den ein
  `dry_run=false`-Lauf buchen würde. Das ist nicht selbstverständlich:
  der Preview berechnet den Überschuss eigenständig und läuft **nicht**
  über `run_for_week`, braucht also einen eigenen
  year-to-date-Balance-Read (gleiche Range, gleiches
  `include_carryover = true`) — ein `0.0`-Platzhalter dort würde den
  Preview still lügen lassen. Zwei sich widersprechende Tests nageln das
  fest (`backfill_dry_run_writes_nothing` bei Balance −8 → 3.0,
  `backfill_dry_run_preview_respects_balance_cap` bei Balance −2 → 2.0).

  **Headroom-Buchführung über mehrere Wochen (CR-02).** Der Apply-Pfad
  verbraucht den Headroom implizit: jedes geschriebene Paar hebt die
  Balance, und SQLite sieht die Rows innerhalb der geteilten `read_tx`
  der Wochen-Schleife. Der Dry-Run schreibt nichts und sähe deshalb in
  **jeder** Woche denselben unverbrauchten Headroom. `backfill_range`
  führt im Dry-Run daher eine `HashMap<sales_person_id, f32>` mit, die den
  bereits vorgemerkten Übertrag auf die gelesene Balance anrechnet
  (`balance = report.balance_hours + consumed`). Ohne diese Buchführung
  divergieren Preview und Apply linear mit der Anzahl der Wochen — bei
  Balance −10 und je 4 h Überschuss in KW 10/11/12 meldet der Preview 12 h,
  gebucht werden 10 h. Festgenagelt von
  `backfill_dry_run_preview_matches_apply_over_multiple_weeks`, das
  Preview- und Apply-Summe desselben Szenarios gegeneinander stellt.

### 9.4 Snapshot-Schema-Entscheidung (REB-AUTO-05 — in Phase 57 gekippt, D-57-10)

**`CURRENT_SNAPSHOT_SCHEMA_VERSION` wurde in Phase 57 auf 12 → 13
gebumpt.**

Phase 56 hat die REB-AUTO-05-Frage („ist der
F4-Cron-`Rebooking`-Source-`ExtraHours`-Writer ein *Input-Set-Change* im
Sinne der CLAUDE.md-Snapshot-Versioning-Klausel?") mit **nein**
beantwortet und den „kein Bump"-Zweig gepinnt. Diese Antwort stützte sich
vollständig auf die Balance-Neutralität des Rebookings — die
Read-Aggregate filterten `source == Manual`, sodass nie eine
`Rebooking`-Zeile eine Snapshot-Kette erreichte. **Diese Voraussetzung
war selbst der Fehler** (§3): der Filter machte den gesamten Übertrag
wirkungslos. Mit der neu gefassten Filter-Regel (VOL-ACCT-04, D-57-06)
fließen Rebooking-Zeilen jetzt in die persistierten `value_type`s
`Overall`, `Balance`, `ExtraWork` und `Volunteer` ein — `Overall`
verschiebt sich mit, weil `report_delta.overall_hours =
shiftplan_hours_by_week + overall_extra_work_hours` gilt und das
`+h ExtraWork`-Bein den zweiten Summanden hebt. Nicht betroffen:
`ExpectedHours`, `VacationEntitlement`, `VacationDays` und die
Absence-Kategorien — `rebook()` schreibt ausschließlich die Kategorien
`VolunteerWork` und `ExtraWork`, und das Soll stammt aus dem Vertrag.

Nach der CLAUDE.md-Klausel „Change the input set the computation reads
from" ist der Bump damit **Pflicht** und wurde in Phase 57 vollzogen
(D-57-10). Bestehende v12-Rows bleiben unangetastet (D-57-12, kein
Recompute, keine Repair-Migration) — Validatoren vergleichen nur
innerhalb derselben `snapshot_schema_version`, genau dafür existiert das
Versionsfeld.

Der Beweis ist empirisch und läuft jetzt andersherum: die
Straddling-Golden-Snapshot-Fixture
(`service_impl/src/test/billing_period_straddling.rs`) beweist weiterhin,
dass der *Toggle* kein einziges Byte bewegt (der Stichtag steuert nur die
Schreibseite, Invariante I-8), und zusätzlich beweist
`test_straddling_snapshot_rebooking_shifts_values`, dass der *Datensatz*
es tut — dieselbe Periode mit und ohne Rebooking-Paar unterscheidet sich
um genau `±h` auf den vier obigen value_types. Eine Verschiebung von `0`
ist dort der Alarm: sie hieße, dass eine Kette das Paar wieder
herausfiltert. Siehe F08 §7.4 für die Bump-Historie und F07 §2.6.

---

**Fazit.** Phase 54 lieferte die Lese-Seite von F14 (F1/F2,
Audit-Tabellen, Marker-Spalte). Phase 55 hängte die Write-Seite an: F3
manuelle One-Shot-Umbuchung, den F5-Claim-on-Suggest-HR-Alert-Lifecycle
(suggest / approve / reject) und das
`has_pending_rebooking`-Alert-Flag. Phase 56 setzte den
F4-Auto-Cron-Scheduler (wöchentlich Montag 03:15) und den
Rollout-Backfill-Endpoint obendrauf (§9). **Phase 57 hat den Übertrag
tatsächlich wirksam gemacht:** bis dahin hielt ein unbedingter
`source == Manual`-Filter jedes Rebooking-Paar aus den Read-Aggregaten
heraus, eine Umbuchung änderte also nichts. Seit Phase 57 trägt die
Kategorie-Semantik die Fachlogik (VOL-ACCT-04), die Automatik ist auf
Balance 0 gedeckelt, während der von HR bestätigte Vorschlag bewusst
nicht gedeckelt ist (REB-AUTO-06, D-57-01), und das Snapshot-Schema steht
auf **13** (D-57-10). Das Einzige, was über eine Umbuchung hinweg noch
neutral bleibt, ist das VOL-ACCT-01-Soll.

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
