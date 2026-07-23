# F14 — Voluntary Rebooking

> **Kurzfassung:** Gedeckelte Mitarbeiter mit `committed_voluntary`
> dürfen nicht in ein dauerhaftes Minus-Konto laufen. F14 liefert das
> Datenmodell-Fundament, die audit-fähige Batch-Struktur und die
> HR-only-Sicht "Voluntary Stats" (F1/F2), die zeigt, wie weit der/die
> Mitarbeiter*in vom Jahres-Freiwillig-Soll entfernt ist. Die gesamte
> End-to-End-Rebooking-Pipeline verteilt sich auf Milestone v2.6,
> Phasen 54..56.

**Cluster ID:** F14
**Status:** teilweise shipped (Phase 54 = F1/F2-Baseline; Phase 55 = F3
manuelle Umbuchung + F5 HR-Alert approve/reject); F4 (Auto-Cron) noch
geplant für Phase 56.
**Zuerst eingeführt:** Milestone v2.6, Phase 54 (2026-07-07). F3
(manuelle One-Shot-Umbuchung) und F5 (Claim-on-Suggest-Lifecycle:
suggest / approve / reject) shipped in Phase 55 (2026-07-23). F4
(Auto-Cron) folgt in Phase 56.
**Zuständige Crates:** `service::{rebooking_batch, voluntary_stats,
rebooking_reconciliation}`,
`service_impl::{rebooking_batch, voluntary_stats, rebooking_reconciliation}`,
`dao::rebooking_batch`, `dao_impl_sqlite::rebooking_batch`,
`service_impl::reporting` (vier neue pure fns + der `source ==
Manual`-Neutralitäts-Filter + das `has_pending_rebooking`-Alert-Flag),
`rest::{report, rebooking}` (voluntary-stats + F3/F5-Routes),
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
| F4 (Auto-Cron-Scheduler) | v2.6 Phase 56 | planned | Toggle-gesteuerter Cron erzeugt `AutoCron`-Vorschläge; bumpt Snapshot 12 → 13. |
| F5 (HR-Alert suggest / approve / reject) | v2.6 Phase 55 | shipped | Claim-on-Suggest (Modell B): `suggest` claimt einen `Pending`-`HrSuggestion`-Batch; `approve` schreibt die gepaarten Zeilen, `reject` transitioniert nur den State. |

**Faustregel:** F3 (manuell) und F5 (HR-Alert) shippen in Phase 55
(siehe §8 unten). Nur der F4-Auto-Cron-Writer ist noch auf Phase 56
verschoben.

## 3. Marker-Filter-Regel ([D-54-DM-02])

`extra_hours` bekommt die additive Spalte `source TEXT NOT NULL
DEFAULT 'manual'`. Die aktiven Domain-Werte sind `manual` und
`rebooking`.

- **`manual`** — jede Zeile, die über die bestehenden UI-Pfade
  geschrieben wird (HR-CRUD, Absence-Convert-Vacation-Writer, Dev-Seed,
  REST-TO → Service-Mapper). Bestandszeilen landen per Column-DEFAULT
  auf `manual`.
- **`rebooking`** — reserviert für die gepaarten Zeilen, die F3/F4/F5
  ab Phase 55 emittieren. In Phase 54 setzt kein Writer diesen Wert —
  der Marker existiert ausschließlich als *Reader-Filter-Ziel*.

**Reader-Regel (aktiv seit Phase 55):** jedes Read-Aggregat, das in
Anwesenheit von Rebooking-Paaren balance-neutral bleiben muss, filtert
`source == ExtraHoursSource::Manual` (behalte `Manual`, schließe
`Rebooking` aus). Der Filter greift **einmal am Aggregations-Eingang**
jeder der vier Report-Pfade in `service_impl/src/reporting.rs`
(`get_week`, `get_reports_for_all_employees`, `hours_per_week`,
`overall_extra_work_hours` — `reporting.rs:859, 1192, 1728, 2161`)
statt an jeder einzelnen per-Kategorie-`.filter()`-Site (D-55-A1). Da
`Rebooking`-Source-Zeilen ausschließlich `ExtraWork`/`VolunteerWork`-
Kategorien tragen, hat der Eingangs-Cutoff exakt denselben Effekt für
die vier Balance-Kennzahlen und Null Effekt auf Absence/Holiday-
Aggregate. Ohne ihn würde dieselbe Freiwillig-Stunde doppelt gezählt
(einmal als Original-`Volunteer`, einmal als `Rebooking`-Source-Zeile,
die sie in der bezahlten Kette neutralisiert).

**Audit-Regel:** `rebooking`-Source-Zeilen bleiben in der DB und
bleiben in *Audit*-Queries sichtbar — sie sind die Antwort auf "warum
hat sich die Balance an diesem Datum geändert?" (F5). Sie sind nur
für End-User-Aggregate unsichtbar.

**Balance-Neutralitäts-Garantie (VOL-ACCT-03) — aktiv seit Phase 55:**
mit dem `source == Manual`-Filter verändert das Einfügen eines
gleich-großen Gegenpaares `(−h, +h)`, beides mit `source = 'rebooking'`
gestempelt, `balance`, `ist`, `soll` und `delta` nicht — die
F1/F2-Zahlen bleiben über ein Rebooking-Event stabil, sowohl für eine
einzelne Umbuchung als auch über einen kompletten Roundtrip.
Festgenagelt durch die Property-Tests
`rebooking_neutrality_single_manual` und
`rebooking_neutrality_roundtrip`
(`service_impl/src/test/rebooking_reconciliation.rs`).

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
| `RebookingBatchService` | Basic | HR-gated CRUD (find_by_id / find_by_sales_person_year_week / create) auf `rebooking_batch` + Entries, plus der Phase-55 F5-Modell-B-Lifecycle: `create_pending_suggestion` (Claim-on-Suggest-Producer — INSERTet einen `HrSuggestion`/`Pending`-Batch und claimt den UNIQUE-Wochen-Slot; belegter Slot → `EntityAlreadyExists`), `find_pending_for_sales_person` (Reader fuer `pending_rebooking_id`, liefert nur `Pending`) und `update_state` (race-sichere state-conditional Transition `Pending → Approved\|Rejected`; ein zweiter/bereits entschiedener Wechsel → `EntityAlreadyExists` via `rows_affected()==0`, kein version-Guard). Deps: `RebookingBatchDao`, `PermissionService`, `ClockService`, `UuidService`, `TransactionDao`. Kein Domain-Service-Dep. |
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
auf der Employee-Detail-Seite. Der Rebooking-Neutralitäts-Filter
(`source == 'manual'`) ist in Phase 54 in diesem Service NICHT aktiv; er
greift ab Phase 55 zentral im `ReportingService` und fließt dann
automatisch in diese Kette.

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
- `ist_total` — F2-Ist (absolute Manual-Volunteer-Summe im Range).
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
  `extra_hours`-Tabelle; die vorgelagerten Reader in F07/F08 nutzen
  den Marker-Filter.
- **F07 Reporting / Balance** — die Read-Aggregate filtern seit Phase 55
  `source == Manual` (behalte `Manual`, schließe `Rebooking` aus) an
  vier Eingangs-Sites, sodass ein Rebooking-Paar balance / ist / soll /
  delta-neutral ist (VOL-ACCT-03). Siehe F07 §2.6.
- **F08 Billing Period Snapshot** — **kein Version-Bump in Phase 55.**
  `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt bei 12: Phase 55 fügt weder
  einen persistierten `value_type` hinzu noch eine Berechnung, die einen
  bestehenden ändern würde (das Rebooking-Paar netto-nullt im Aggregat).
  Der Bump 12 → 13 liegt in Phase 56 (REB-AUTO-05, F4-Cron) — siehe
  `REQUIREMENTS.md`.
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
…)` HR-gatet zuerst, dann in **einer** Transaktion: schreibt das Paar
(§8.1), legt einen `rebooking_batch` mit `kind = Manual`, `state =
Approved` an und einen `rebooking_batch_entry` mit den beiden Pair-ids.
Es gibt **keinen** suggest/pending-Zwischenschritt. Ein belegter
UNIQUE-Wochen-Slot → `EntityAlreadyExists` → HTTP 409.

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
- **`approve(rebooking_batch_id, hours, …)`** — HR-gated. Transitioniert
  den State **zuerst** via `RebookingBatchService::update_state` — ein
  state-conditional UPDATE `… WHERE id = ? AND state = 'pending' AND
  deleted IS NULL` mit `rows_affected() == 1`-Guard (Pitfall 12) — und
  schreibt nur bei Erfolg das Paar (§8.1, `VolunteerToExtra`) in
  derselben Transaktion. Ein Double-Approve-Verlierer
  (`rows_affected() == 0`) → `EntityAlreadyExists` → HTTP 409. `approve`
  legt **keinen** zweiten Batch an; es transitioniert den bestehenden
  pending-Batch.
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

---

**Fazit.** Phase 54 lieferte die Lese-Seite von F14 (F1/F2,
Audit-Tabellen, Marker-Spalte). Phase 55 hängt die Write-Seite an: F3
manuelle One-Shot-Umbuchung und der F5-Claim-on-Suggest-HR-Alert-
Lifecycle (suggest / approve / reject), den `source ==
Manual`-Neutralitäts-Filter (VOL-ACCT-03) und das
`has_pending_rebooking`-Alert-Flag — **ohne Snapshot-Schema-Bump**
(bleibt 12). Phase 56 setzt den F4-Auto-Cron obendrauf (dann der Bump
12 → 13).

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
