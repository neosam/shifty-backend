# F14 — Voluntary Rebooking

> **Kurzfassung:** Gedeckelte Mitarbeiter mit `committed_voluntary`
> dürfen nicht in ein dauerhaftes Minus-Konto laufen. F14 liefert das
> Datenmodell-Fundament, die audit-fähige Batch-Struktur und die
> HR-only-Sicht "Voluntary Stats" (F1/F2), die zeigt, wie weit der/die
> Mitarbeiter*in vom Jahres-Freiwillig-Soll entfernt ist. Die gesamte
> End-to-End-Rebooking-Pipeline verteilt sich auf Milestone v2.6,
> Phasen 54..56.

**Cluster ID:** F14
**Status:** teilweise shipped (Phase 54 = F1/F2-Baseline)
**Zuerst eingeführt:** Milestone v2.6, Phase 54 (2026-07-07). F3
folgt in Phase 55 (HR-Suggest-Write-Path); F4 (Auto-Cron) und F5
(Approval-UI) in Phase 56.
**Zuständige Crates:** `service::{rebooking_batch, voluntary_stats}`,
`service_impl::{rebooking_batch, voluntary_stats}`,
`dao::rebooking_batch`, `dao_impl_sqlite::rebooking_batch`,
`service_impl::reporting` (vier neue pure fns),
`rest::report` (voluntary-stats Route),
`rest-types::VoluntaryStatsTO`, `shifty-dioxus::component::voluntary_stats_row`.

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
| F3 (HR-Suggest → Pending-Batch) | v2.6 Phase 55 | planned | HR schlägt gepaartes Rebooking vor; Batch landet als `state = Pending`. |
| F4 (Auto-Cron-Scheduler) | v2.6 Phase 56 | planned | Toggle-gesteuerter Cron erzeugt `AutoCron`-Vorschläge; bumpt Snapshot 12 → 13. |
| F5 (Approval / UI) | v2.6 Phase 55 | planned | HR prüft Pending-Batches — approve schreibt die gepaarten Zeilen atomar oder reject. |

**Faustregel für Phase 54:** Alles, was ein *Reader* braucht,
funktioniert heute. Alles, was ein *Writer* anfasst (F3/F4/F5), ist
auf Phase 55/56 verschoben.

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

**Reader-Regel (geplant für Phase 55):** jedes Aggregat, das in
Anwesenheit zukünftiger Rebooking-Paare balance-neutral bleiben muss,
wird `source = 'manual'` filtern. In Phase 54 ist der Filter noch NICHT
aktiv — das Voluntary-Stats-Ist-Aggregat liest
`EmployeeReport::volunteer_hours` aus dem `ReportingService` und erbt
das, was diese zentrale Kette filtert. Wenn Phase 55 den
`source == 'manual'`-Cutoff im `ReportingService` selbst einbaut, wird
diese Kette den automatisch mitnehmen; sonst würde dieselbe
Freiwillig-Stunde doppelt gezählt (einmal als Original-`Volunteer`,
einmal als `Rebooking`-Source-Zeile, die sie in der bezahlten Kette
neutralisiert).

**Audit-Regel:** `rebooking`-Source-Zeilen bleiben in der DB und
bleiben in *Audit*-Queries sichtbar — sie sind die Antwort auf "warum
hat sich die Balance an diesem Datum geändert?" (F5). Sie sind nur
für End-User-Aggregate unsichtbar.

**Balance-Neutralitäts-Garantie (VOL-ACCT-03) — geplant für Phase 55:**
sobald `source == 'manual'`-Filter zentral im `ReportingService` greift
(Phase 55), verändert das Einfügen eines gleich-gerichteten Gegenpaares
`(+h, -h)`, beides mit `source = 'rebooking'` gestempelt, den
`EmployeeReport::volunteer_hours` nicht — die F1/F2-Zahlen bleiben über
ein Rebooking-Event stabil, weil die Voluntary-Stats-Kette direkt
`EmployeeReport::volunteer_hours` konsumiert. Der Property-Test ist auf
Phase 55 verschoben (zusammen mit dem ersten Live-Rebooking-Writer).

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
| `RebookingBatchService` | Basic | HR-gated CRUD (find_by_id / find_by_sales_person_year_week / create) auf `rebooking_batch` + Entries. Deps: `RebookingBatchDao`, `PermissionService`, `ClockService`, `UuidService`, `TransactionDao`. Kein Domain-Service-Dep. |
| `VoluntaryStatsService` | Business-Logic | Read-only F1/F2. Deps: `ExtraHoursService`, `EmployeeWorkDetailsService`, `SalesPersonService`, `PermissionService`, `TransactionDao`. HR-only per API-Level None-Redaktion (nicht 403). |

**Konsumenten-Wiring (Phase 54):** `RebookingBatchService` hat noch
keinen Konsumenten im Code — er ist DI-verdrahtet in
`shifty_bin/src/main.rs`, damit sich Phase 55's
`RebookingReconciliationService` ohne DI-Refactor anhängen kann.
`VoluntaryStatsService` wird genau einmal konsumiert, vom REST-Handler
`rest/src/report.rs::get_voluntary_stats`.

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
- **F07 Reporting / Balance** — die Balance-Kette filtert ab Phase 55
  `source = 'manual'` (sobald ein `Rebooking`-Writer existiert).
  Phase 54 führt den Marker ein, aber keinen Writer — sämtliche
  Bestandszeilen gehen weiterhin identisch in die Balance ein.
- **F08 Billing Period Snapshot** — kein Version-Bump in Phase 54.
  `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt bei 12, weil Phase 54
  weder einen persistierten `value_type` hinzufügt noch eine
  Berechnung ändert. Der Bump 12 → 13 liegt in Phase 56
  (REB-AUTO-05, F4-Cron) — siehe `REQUIREMENTS.md`.
- **F13 System-Infrastruktur** — der Toggle
  `voluntary_rebooking_auto_active_from` (in Phase 54 mit
  `enabled = 0`, `value = NULL` geseedet) gattet in Phase 56 den
  F4-Cron. In Phase 54 ist er inaktiv.

## 8. Manuelle Umbuchung (F3)

*Eingeführt Phase 55 (v2.6).* HR-getriggerter One-Shot-Pair-Write, der
einige `Volunteer`-Stunden in ein Balance-neutralisierendes `+/-`-Paar
umsetzt — direkt im Zustand `Approved`, ohne Pending-Zwischenschritt.

### 8.1 Trigger

HR navigiert zu `/employees/{id}` (Employee-Details-Seite) und öffnet
den Menu-Punkt *"Manuelle Umbuchung"* in der TopBar / im Action-Menu
([D-55-06]). Explizite Design-Wahl: KEIN Button in der reinen
Lese-Zeile „Voluntary-Stats" — die TopBar hält die Read-Ansicht sauber
und erzwingt, dass die Richtungswahl eine bewusste Modal-Aktion ist.

### 8.2 Modal-Aufbau

Die Komponente `manual_rebooking_modal` zeigt vier Eingaben plus eine
Vorschau:

- **ISO-Woche** — `iso_year` (`u32`) + `iso_week` (`u8`), Default:
  aktuelle KW. HR darf jede Woche wählen, auch retrospektiv
  ([D-55-05]).
- **Richtung** — Radio zwischen `VolunteerToExtra` und
  `ExtraToVolunteer` ([D-55-06] — im TopBar-Trigger gibt es keinen
  Delta-Vorzeichen-Kontext, deshalb MUSS das Modal die Richtung
  explizit anbieten).
- **Stunden** — positives `f32`; validiert per REB-MANUAL-03 (siehe
  8.5).
- **Vorschau** — spiegelt die Backend-berechneten Pair-Payloads vor
  dem Submit.

### 8.3 Submit-Flow

Das Modal POSTet `ManualRebookingRequestTO` an
`POST /rebooking/manual` (im Axum-Tree unter `/rebooking`; Dev-Proxy-
Eintrag in Plan 55-02 nachgezogen). Der REST-Handler ruft
`RebookingReconciliationService::rebook_manual`, das:

1. **HR-gate** als erster `await` — Non-HR bekommt `Forbidden` ohne
   DAO-Round-Trip.
2. **Öffnet Transaktion** (`TransactionDao::use_transaction`).
3. **Baut die Paar-Payloads** über einen `Direction`-getriebenen Helper
   (`build_pair_payloads`) — das Enum eliminiert die Vorzeichen-Bug-
   Klasse, die ein rohes signiertes `hours`-Argument öffnen würde.
4. **Schreibt zwei `ExtraHours`-Zeilen** mit Marker
   `ExtraHoursSource::Rebooking` und Kategorien je Richtung:
   - `VolunteerToExtra` → `-h Volunteer`, `+h ExtraWork`.
   - `ExtraToVolunteer` → `-h ExtraWork`, `+h Volunteer`.
5. **Erzeugt einen `rebooking_batch`** (`kind=Manual, state=Approved`;
   `approved` + `approved_by` werden sofort aus dem Auth-Context
   gesetzt).
6. **Erzeugt einen `rebooking_batch_entry`** mit den beiden
   ExtraHours-Row-IDs als `extra_hours_out_id` / `extra_hours_in_id` —
   die §4-Invariante (FKs sind `NULL` gdw. `state = Pending`) hält
   auch im Manual-Pfad, weil der Write atomar ist und direkt in
   `Approved` landet.

### 8.4 State-Machine

Manuelle Batches durchlaufen niemals `Pending` — sie werden direkt als
`Approved` geboren. Der einzig erlaubte Übergang ist der, den der
Writer bei der Anlage vornimmt: `⟂ → Approved`. [D-55-04] pinnt das
als One-Shot: kein Undo, keine `Approved → Rejected`-Rückwärtsrichtung,
kein Delete. Das Anti-Feature REB-UNDO-01 in `REQUIREMENTS.md` ist der
normative Anker.

### 8.5 Fehler

| HTTP | Body                                                           | Wann |
|------|----------------------------------------------------------------|------|
| 400  | `{"error":"RebookingErrorHoursMustBePositive"}`                 | `hours <= 0` oder nicht endlich (REB-MANUAL-03; validiert in `rest/src/rebooking.rs::post_manual` vor dem Service-Aufruf). |
| 409  | `{"error":"RebookingErrorSlotTaken"}`                           | UNIQUE-Slot `(sales_person_id, iso_year, iso_week)` bereits belegt durch *irgendeinen* Batch (Manual / HrSuggestion / AutoCron), unabhängig vom State. Die BL propagiert `ServiceError::EntityAlreadyExists`; der REST-Handler mapped manuell auf den i18n-Key, damit die Batch-UUID nicht in den Wire-Body leakt (T-4-Mitigation). |
| 403  | plain `Forbidden`                                              | Non-HR-Aufrufer (BL-HR-Gate). |

Bei `RebookingErrorSlotTaken` bleibt das Modal offen; HR kann die
Woche ändern und neu senden. Kein Auto-Overwrite — das würde eine
frühere manuelle Korrektur still überschreiben.

### 8.6 Zustand der Read-Aggregate nach Erfolg

Zwei `ExtraHours`-Zeilen und ein Batch+Entry sind persistiert. Das
Paar trägt `source == Rebooking`, ist damit unsichtbar für
`EmployeeReport::volunteer_hours`, `balance_hours` und `overall_hours`
— [VOL-ACCT-03] hält durch Konstruktion. Der zentrale
`source != Rebooking`-Filter lebt im `ReportingService` (Wave-1-Owner)
und deckt jeden nachgelagerten Konsumenten ab (`VoluntaryStatsService`,
`BookingInformationService`, F1/F2-Zeilen, Balance-Zeile). Property-
Test 55-03 liefert die End-to-End-Garantie.

---

## 9. HR-Alert + Vorschlags-Modal (F5)

*Eingeführt Phase 55 (v2.6).* Zwei-Schritt-, Zwei-Akteur-Variante von
§8: HR-Alert markiert die Person automatisch, HR löst den Alert per
Klick auf Approve oder Reject im Vorschlags-Modal auf.

### 9.1 Alert-Predicate ([D-55-01])

Pure fn in `service/src/rebooking_reconciliation.rs`:

```rust
pub fn alert_predicate(balance: f32, voluntary_ist: f32, cap_active: bool) -> bool {
    cap_active && balance <= -0.5 && voluntary_ist > 0.0
}
```

- **`cap_active`** — der/die Zielperson hat `has_hour_cap = true`. Ohne
  Cap reconciliert die Balance-Kette schon zum Jahresende; der Alert
  wäre bedeutungslos.
- **`balance <= -0.5`** — Float-Noise-Tolerance. Strikte `< 0`-Regel
  triggerte auf `-0.0001`-Rundungsartefakte; die Halb-Stunden-Schwelle
  passt zur UI-Granularität (eine Nachkommastelle) und zeigt nur
  echte Lücken.
- **`voluntary_ist > 0.0`** — nichts umzubuchen, wenn die Person null
  Freiwilligen-Stunden im Range hat.

Der Truth-Table-Test liegt in
`service_impl/src/test/rebooking_reconciliation.rs::predicate_truth_table`
und pinnt das Rand-Tripel `balance = -0.49 / -0.5 / -0.51` explizit.

### 9.2 Backend-Ripple ([D-55-02])

`ShortEmployeeReportTO` (in `rest-types`) wird additiv erweitert:

```rust
#[serde(default)]
pub has_pending_rebooking: bool,
#[serde(default)]
pub pending_rebooking_id: Option<Uuid>,
```

Wire-Kompatibilität bleibt via `#[serde(default)]` gewahrt (Präzedenz
VAA-04). Der Wert wird in
`ReportingService::enrich_reports_with_pending_rebooking` per
*Predicate-first*-Muster gesetzt:

1. Berechne `alert_predicate(balance, voluntary_ist, cap_active)` aus
   den bereits assemblierten ShortEmployeeReport-Aggregaten.
2. Nur bei `true` wird
   `RebookingBatchService::list_pending_for_sales_person(Some(sp_id))`
   via des neuen DAO-Verbs `find_pending_for_sales_person` abgefragt.
3. Existiert ein `state = Pending`-Batch, wird
   `has_pending_rebooking = true` + `pending_rebooking_id = Some(batch_id)`
   gestempelt.

Zwei Guardrails:

- **HR-Gate** — Enrichment bricht vor dem DAO-Call ab, wenn der
  Caller Non-HR ist (`check_permission(HR_PRIVILEGE)`). Non-HR
  erhält die Defaults (`false` / `None`).
- **Authentication::Full-Skip** — interne Aggregat-Aufrufer
  (`BookingInformationService`, `VoluntaryStatsService`, PDF-
  Scheduler) übergeben `Authentication::Full` und brauchen das
  Alert-Flag nicht; die Enrichment-Funktion returned für sie
  frühzeitig, damit die ~40 `get_week`-Test-Setup-Call-Counts intakt
  bleiben ([D-55-EXEC-03]).

### 9.3 Alert-UI

`shifty-dioxus/src/component/rebooking_alert_banner.rs` (angelegt in
Plan 55-04) rendert ein **nicht-blockierendes Inline-Banner** in der
Employees-Übersichts-Liste — eine Zeile pro Person mit
`has_pending_rebooking = true`. Kein Modal-on-load, kein Confirmation-
Dialog — folgt bewusst der projektweiten MEMORY-Regel
[`feedback_warnings_inline_not_dialog`](../../.planning/PROJECT.md).
Klick auf das Banner öffnet das Suggestion-Modal für
`pending_rebooking_id`.

### 9.4 Vorschlags-Modal — IST / DANN ([D-55-03])

`shifty-dioxus/src/component/rebooking_suggestion_modal.rs` zeigt die
`RebookingSuggestionTO`-Payload, die `GET /rebooking-suggestions` (bzw.
der Einzel-Fetch `/{id}`) liefert. Zwei Spalten nebeneinander:

| Feld                 | IST (`_before`) — vor Approve | DANN (`_after`) — nach Approve |
|----------------------|-------------------------------|--------------------------------|
| Balance              | `balance_before`              | `balance_after`                |
| Voluntary Ist        | `voluntary_ist_before`        | `voluntary_ist_after`          |
| Voluntary Soll       | `voluntary_soll_before`       | `voluntary_soll_after`         |
| Voluntary Delta      | `voluntary_delta_before`      | `voluntary_delta_after`        |
| Vorgeschlagene Std.  | —                             | `proposed_hours`               |

**Jede** dieser Zahlen ist Backend-berechnet ([D-55-03], MEMORY
`feedback_fat_backend_thin_client`). Das FE subtrahiert nicht, kappt
nicht, verrechnet nicht — es rendert nur. Insbesondere
`voluntary_delta_before` und `voluntary_delta_after` sind **eigene
Backend-Felder**, keine FE-Ableitung — das FE rechnet nicht `ist - soll`.

`proposed_hours = min(|balance|, voluntary_ist).max(0.0)` — die
zentrale Pure-fn `proposed_rebooking_hours` aus
`service/src/rebooking_reconciliation.rs`.

Das Modal exposet exakt zwei Aktionen: **Approve** und **Reject**.

### 9.5 State-Machine

```
                    suggest_for_week          approve_suggestion
                   ┌──────────────┐          ┌───────────────────┐
        ⟂  ─────►  │  Pending     │  ─────►  │     Approved      │
                   │  (Claim held)│          │  (Paar geschrieben)│
                   └──────────────┘          └───────────────────┘
                            │                            
                            │ reject_suggestion          
                            ▼                            
                   ┌──────────────┐                      
                   │  Rejected    │                      
                   │  (Slot held) │                      
                   └──────────────┘                      
```

**`suggest_for_week`** — erzeugt `rebooking_batch`
`kind=HrSuggestion, state=Pending`. Die zwei Entry-FKs
(`extra_hours_out_id`, `extra_hours_in_id`) sind `NULL` gemäß §4-
Invariante — **es werden KEINE `ExtraHours`-Zeilen im Pending
geschrieben**. Der UNIQUE-Slot-Claim wird direkt über den partiellen
Index [D-54-DM-01] gehalten; das ist Claim-on-Suggest ([HR-ALERT-04])
— verhindert Doppelvorschläge für dieselbe Person / dieselbe ISO-Woche
und blockiert auch den F4-AutoCron (Phase 56) vor einem Race.

**`approve_suggestion`** — state-conditional `UPDATE ... WHERE state='pending'`
(rows-affected == 1 gewinnt den Race, sonst `BatchAlreadyResolved`).
Bei Erfolg werden die gepaarten `ExtraHours`-Zeilen **atomar in
derselben Transaktion** geschrieben, die den State flippt — damit
gilt die §4-FK-Invariante genau dann als erfüllt, wenn der State auf
`Approved` steht.

**`reject_suggestion`** — state-conditional `UPDATE ... WHERE state='pending'`
auf `Rejected`. **Kein `ExtraHours`-Write**. Der UNIQUE-Slot bleibt
belegt bis zur nächsten ISO-Woche ([D-55-07]) — dieselbe Person kann
in derselben Woche keinen neuen Vorschlag bekommen, und HR muss
denselben Reject nicht zweimal drücken.

### 9.6 Alert-Terminierung ([D-55-07])

Beide terminalen States beenden den Alert:

- **Approve** — der Paar-Write hebt `balance` über die -0.5h-Schwelle;
  der Predicate wird `false`.
- **Reject** — der Slot ist von einem Nicht-Pending-Batch belegt; die
  Query `find_pending_for_sales_person` liefert nichts; der DAO-Gate
  stempelt `has_pending_rebooking = false`.

Konsistent zur `REQUIREMENTS.md`-HR-ALERT-03-Formulierung *„sichtbar
vermerkt"* — der Rejected-Batch bleibt im Audit-Trail, nur nicht im
Banner.

### 9.7 Kein Undo ([D-55-04], Anti-Feature REB-UNDO-01)

Sowohl `Approve` als auch `Reject` sind One-Shot. Es gibt keinen
`Approved → Rejected`- oder `Rejected → Approved`-Übergang, kein
Delete, keinen Undo-Endpoint. Ein Rebooking rückgängig zu machen
erfordert von HR ein frisches Manual-Rebooking (F3) *in
Gegenrichtung* — ein eigenes Audit-Event.

### 9.8 Race-Semantik ([HR-ALERT-03], T-55-01)

Zwei HR-User klicken gleichzeitig Approve — beide treffen das
state-conditional UPDATE. Genau einer bekommt `rows_affected = 1`; der
andere bekommt `rows_affected = 0`, was die BL auf
`ServiceError::BatchAlreadyResolved` mapped. Der REST-Layer bildet das
auf `HTTP 409 { "error": "RebookingErrorAlreadyResolved" }` ab — der
FE-i18n-Vertrag ist stabil und hängt nicht an SQL-Details (T-4 Leak-
Mitigation). Gleiches Protokoll gilt für den Approve/Reject-Cross-
Race.

---

**Fazit.** Phase 54 liefert die Lese-Seite von F14: HR sieht F1/F2
im Employee-Report, die Audit-Tabellen stehen, und die Marker-Spalte
sagt den zukünftigen Writern, wo die zukünftigen Rebooking-Zeilen
landen werden. Phase 55 hängt die zwei User-facing-Writer (F3 Manual
+ F5 HR-Alert/Vorschlag) auf diese Basis, ohne weitere Schema-
Änderung — die gesamte Pair-Write-Invariante, das Pending-Claim-on-
Suggest und die State-Machine leben in `RebookingReconciliationService`
(Business-Logic-Schicht) mit Backend-berechneten IST/DANN und ohne
Undo. Phase 56 hängt den F4-AutoCron oben drauf.

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.

_Letztes Update: 2026-07-10 — Phase 55 F3+F5-Sektionen angehängt._
