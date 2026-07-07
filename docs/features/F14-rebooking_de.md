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

**Reader-Regel:** jedes Aggregat, das in Anwesenheit zukünftiger
Rebooking-Paare balance-neutral bleiben muss, filtert
`source = 'manual'`. Erster Konsument ist Plan 54-03's
`voluntary_ist_total_for_year(extra_hours, year)`, das die Ist-
Freiwillig-Stunden für F1/F2 aufsummiert und das künftige
`rebooking`-Rauschen ausschließen muss; sonst würde dieselbe
Freiwillig-Stunde doppelt gezählt (einmal als Original-`Volunteer`,
einmal als `Rebooking`-Source-Zeile, die sie in der bezahlten Kette
neutralisiert).

**Audit-Regel:** `rebooking`-Source-Zeilen bleiben in der DB und
bleiben in *Audit*-Queries sichtbar — sie sind die Antwort auf "warum
hat sich die Balance an diesem Datum geändert?" (F5). Sie sind nur
für End-User-Aggregate unsichtbar.

**Balance-Neutralitäts-Garantie (VOL-ACCT-03):** der Property-Test in
`service_impl/src/test/voluntary_stats.rs` zeigt, dass das Einfügen
eines gleich-gerichteten Gegenpaares `(+h, -h)`, beides mit
`source = 'rebooking'` gestempelt, `voluntary_ist_total_for_year(..)`
nicht verändert — die F1/F2-Zahlen bleiben über ein Rebooking-Event
stabil.

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

### Pure Functions in `service_impl::reporting`

`VoluntaryStatsService` ist dünn. Die Mathematik liegt in vier pure
fns neben `committed_voluntary_for_calendar_week` in
`service_impl/src/reporting.rs`:

```rust
/// VOL-STAT-01 / VOL-ACCT-01-Ist — Manual-only Summe der Volunteer-
/// Stunden im ISO-Jahr. Filtert source = Manual + Soft-Deletes.
pub fn voluntary_ist_total_for_year(extra_hours: &[ExtraHours], year: u32) -> f32;

/// F1-Nenner / D-F1-01 — Vertragswochen-Zählung; expected_hours = 0 zählt MIT.
pub fn contract_weeks_count(working_hours: &[EmployeeWorkDetails], year: u32) -> u32;

/// D-F2-01 — tages-pro-rata für eine einzelne ISO-Woche mit
/// tagesgenau aktiver EmployeeWorkDetails (Mid-Week-Kontraktwechsel).
pub fn committed_voluntary_prorata_for_week(
    working_hours: &[EmployeeWorkDetails], year: u32, week: u8) -> f32;

/// F2-Soll = Σ über alle ISO-Wochen des Jahres.
pub fn committed_voluntary_target_for_year(
    working_hours: &[EmployeeWorkDetails], year: u32) -> f32;
```

## 6. REST (Phase 54)

| Methode | Pfad | DTO In | DTO Out | Auth |
| --- | --- | --- | --- | --- |
| `GET` | `/report/{id}/voluntary-stats?year=YYYY` | — | `VoluntaryStatsTO` | jede authentifizierte Session; HR-only Inhalt — Non-HR erhält alle Felder = `null`. |

`VoluntaryStatsTO` (5 Felder, alle `Option<f32>`/`Option<u32>`,
serde `#[serde(default)]` für Wire-Kompatibilität):

- `ist_per_contract_week` — F1 (Ø Freiwillig / Vertragswoche).
- `ist_total` — F2-Ist (absolute Manual-Volunteer-Summe für das Jahr).
- `soll_total` — F2-Soll (`committed_voluntary` pro-rata).
- `delta` — `ist_total − soll_total`.
- `contract_weeks` — F1-Nenner (Audit).

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

---

**Fazit.** Phase 54 liefert die Lese-Seite von F14: HR sieht F1/F2
im Employee-Report, die Audit-Tabellen stehen, und die Marker-Spalte
sagt den zukünftigen Writern, wo die zukünftigen Rebooking-Zeilen
landen werden. Milestones v2.6 Phase 55 + Phase 56 hängen die Writer
und den Cron ohne weitere Schema-Änderung darauf.

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
