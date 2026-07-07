# Roadmap: Shifty Backend

## Milestones

- 🚧 **v2.6** — Phasen 54–56 (planning 2026-07-06) — Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter (this file)
- ✅ **v2.5** — Phasen 52–53 (shipped 2026-07-06) — Weekly-Overview Performance & Freiwilligen-Abwesenheiten ([`milestones/v2.5-ROADMAP.md`](milestones/v2.5-ROADMAP.md))
- ✅ **v2.4** — Phase 51 (shipped 2026-07-05) — Kurzer-Tag-Slot-Kürzung ([`milestones/v2.4-ROADMAP.md`](milestones/v2.4-ROADMAP.md))
- ✅ **v2.3** — Phasen 49–50 (shipped 2026-07-04) — PDF-Export: Browser-Look & Download-Button ([`milestones/v2.3-ROADMAP.md`](milestones/v2.3-ROADMAP.md))
- ✅ **v2.2** — Phasen 43–48 (shipped 2026-07-03) — Aufräumen, WebDAV-Export & Wochentag-Muster ([`milestones/v2.2-ROADMAP.md`](milestones/v2.2-ROADMAP.md))
- ✅ **v2.1** — Phasen 39–42 (shipped 2026-07-02) — Schichtplan- & Reporting-Erweiterungen ([`milestones/v2.1-ROADMAP.md`](milestones/v2.1-ROADMAP.md))
- ✅ **v1.11** — Phasen 36–38 (shipped 2026-07-01) — Stabilisierung & UX-Politur ([`milestones/v1.11-ROADMAP.md`](milestones/v1.11-ROADMAP.md))
- ✅ **v1.10** — Phasen 33–35 (shipped 2026-06-30) — Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz ([`milestones/v1.10-ROADMAP.md`](milestones/v1.10-ROADMAP.md))
- ✅ **v1.9** — Phasen 29–32 (shipped 2026-06-29) — Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation ([`milestones/v1.9-ROADMAP.md`](milestones/v1.9-ROADMAP.md))
- ✅ **v1.8** — Phasen 27–28 (shipped 2026-06-29) — Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) ([`milestones/v1.8-ROADMAP.md`](milestones/v1.8-ROADMAP.md))
- ✅ **v1.7** — Phasen 25–26 (shipped 2026-06-29) — Automatische Feiertage & Freiwilligen-Abwesenheit ([`milestones/v1.7-ROADMAP.md`](milestones/v1.7-ROADMAP.md))
- ✅ **v1.6** — Phase 24 (shipped 2026-06-27) — Paid-Capacity-Durchsetzung & Konfiguration ([`milestones/v1.6-ROADMAP.md`](milestones/v1.6-ROADMAP.md))
- ✅ **v1.5** — Phasen 18–23 (shipped 2026-06-27) — Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen ([`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md))
- ✅ **v1.4** — Phasen 14–17 (shipped 2026-06-25) — Committed Voluntary Capacity ([`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md))
- ✅ **v1.3** — Phasen 8–13 (closed 2026-06-22) — Frontend Abwesenheiten + UI-Closure-Restanten ([`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md))
- ✅ **v1.2** — Phasen 6–7 (shipped 2026-05-07) — Frontend rest-types Konsolidierung ([`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md))
- ✅ **v1.1** — Phase 5 (shipped 2026-05-04) — Slot Capacity & Constraints ([`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md))
- ✅ **v1.0** — Phasen 1–4 (shipped 2026-05-03) — Range-Based Absence Management ([`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md))

Vollständiger historischer Index: [`MILESTONES.md`](MILESTONES.md).

## Milestone v2.6 — Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter (Active)

**Goal:** Gedeckelte Mitarbeiter (`cap_planned_hours_to_expected=true`), die freiwillig
mitarbeiten, sollen ihre Freiwilligen-Stunden automatisch oder halbautomatisch als
Ausgleich gegen ein negatives Stundenkonto einsetzen können. HR sieht ein Soll/Ist-Konto
für Freiwilligkeit, wird proaktiv über Ausgleichsbedarf informiert und kann Umbuchungen
bestätigen; die Software erledigt die Doppelbuchung.

**Requirements-Quelle:** `.planning/REQUIREMENTS.md` (17 REQ-IDs in 5 Kategorien).
**Research-Quelle:** `.planning/research/SUMMARY.md` (Konvergenz aus 4 Researcher-Outputs; 2 explizit
offene Discuss-Decisions bleiben in den Phasen zu pinnen).

### Phase-Dekomposition-Entscheidung

Research kennt drei Vorschläge (SUMMARY §Divergenz 1): A (4 Phasen, FEATURES),
B (6 Phasen, STACK), C (3 Phasen, ARCHITECTURE). Roadmap wählt **C** aus zwei Gründen:

1. **Vertikal-Slice-Prinzip:** Jede Phase C ist ein UAT-anchorbarer End-to-End-Slice
   (BE + FE + REST + i18n) mit sichtbarem User-Wert. Präzedenz v1.10 Phase 33
   (Special-Days-UI-BE+FE in einer Phase), v2.1 Phase 39 (WeekStatus Basic Service
   inkl. Migration + REST + FE-Badge in einer Phase), v2.2 Phase 48 (EXP-WebDAV mit
   Migration + Scheduler + Admin-Card in einer Phase) — alle diese Phasen haben
   Migration + Basic-Service + REST + FE erfolgreich in einer Phase geliefert.

2. **Data-model-first-Regel bleibt intakt:** Die Marker-Spalte + UNIQUE-Constraint
   MÜSSEN in Phase 54 landen, bevor F1/F2/F3/F4/F5-Aggregate lesen — sonst muss der
   Marker-Filter retroaktiv nachgezogen werden (Pitfall 1). Phase-54-Split würde
   die Migration nur um eine Phase verzögern, ohne den Kopplungs-Blocker aufzulösen.

**Fallback-Regel (wenn Phase 54 in der discuss-phase als „zu groß" bewertet wird):**
Split in 54-A (Migrations + `RebookingBatchDao` + `RebookingBatchService`-Skeleton)

+ 54-B (F1+F2 Read-Aggregat + FE-Row), Phase 55→56 wird zu 56→57. Kein hartes

Änderungssignal jetzt — Phasen 55 und 56 in ARCHITECTURE-C sind bereits kompakter
als Phase 54.

### Snapshot-Schema-Version 12→13

**Nicht in dieser Roadmap vorwegnehmen.** REB-AUTO-05 pinnt die Entscheidung explizit
auf Phase-56-discuss-phase (F4). Kriterium: Ist die neue Rebooking-ExtraHours-Quelle
ein „Input-Set-Change" im Sinne CLAUDE.md-Klausel? Beweislast beim „Nein"-Zweig
= Straddling-Golden-Snapshot-Fixture, byte-identisch über pre/post/traversierend
`active_from`. Wenn die nicht liefert → Bump 12→13.

### Stichtag-Toggle-Seed

**Landet in Phase 54** (Data-Model-Phase). Grund: der Toggle-Key
`voluntary_rebooking_auto_active_from` gehört zum Data-Model-Fundament, das alle
späteren Phasen konsumieren. Präzedenz: v2.4 Migration
`20260704000001_seed-shortday-slot-clipping-toggle.sql` (Toggle-Seed lief mit der
Data-Model-Migration derselben Phase). Default `None` = Feature dormant, keine
Regression bei Rollout.

### Docs-Freshness-Gate

Neu anzulegen (beide Sprachen synchron, gleicher Commit wie Code-Diff):

- **`docs/features/F14-rebooking.md` + `F14-rebooking_de.md`** — Rebooking-Domäne
  (F1..F5, Batch-Struktur, Marker-Filter-Regel).

Zu aktualisieren:

- **`docs/features/F07-reporting-balance.md` + `_de.md`** — Balance-Chain sieht
  Rebooking-Marker-Filter (keine Doppel-Zählung).

- **`docs/features/F08-billing-period.md` + `_de.md`** — narrative Note zu
  ExtraWork-aus-Rebooking; Snapshot-Version-Bump-Historie nur wenn REB-AUTO-05
  „Ja" ergibt.

- **`docs/architecture/02-service-tiers.md`** + `diagrams/service-graph-runtime.mmd`
  — drei neue BL-Services (`RebookingReconciliationService`, `VoluntaryStatsService`,
  `VoluntaryRebookingScheduler`) + ein Basic (`RebookingBatchService`).

- **`docs/architecture/03-data-model.md`** + `diagrams/db-schema-er.mmd` — 2 neue
  Tabellen `rebooking_batch` + `rebooking_batch_entry`.

Kein Follow-up — MEMORY `feedback_docs_always_current_no_followup.md`.

### Phases

- [x] **Phase 54: Data-Model + Voluntary Statistics (F1 + F2)** — Migrationen + (completed 2026-07-07)
      RebookingBatch-Basic-Service + VoluntaryStatsService (BL, HR-gated) + FE-Row
      im Employee-Detail-Report

- [ ] **Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5)** —
      RebookingReconciliationService.rebook_manual + suggest/approve/reject +
      FE-Alert-Banner + Vorschlags-Modal

- [ ] **Phase 56: Wochen-Cron + Rollout-Backfill (F4)** —
      VoluntaryRebookingScheduler (Wochen-Cron) + Stichtag-Toggle-Wirkung +
      HR-gated Backfill-REST-Endpoint + Admin-Card (FE)

### Phase Details

#### Phase 54: Data-Model + Voluntary Statistics (F1 + F2)

**Goal:** HR sieht im Mitarbeiter-Jahresreport zusätzlich zur bestehenden Ø-Anwesenheit
(AVG-01) das Ist/Soll/Delta des Freiwilligen-Kontos; die Datenmodell-Basis für alle
späteren Rebooking-Trigger (F3/F4/F5) steht mit Marker-Spalte + UNIQUE-Constraint bereit.

**Depends on:** Nothing (first phase of milestone; sits on shipped v2.5 baseline).

**Requirements:** VOL-STAT-01, VOL-STAT-02, VOL-ACCT-01, VOL-ACCT-02, VOL-ACCT-03

**Success Criteria** (what must be TRUE):

  1. HR sieht im Employee-Year-Report (`/employees/:id/:year`) unter der bestehenden
     „Freiwillige Stunden"-Zeile drei zusätzliche Werte: Ø freiwillige Stunden pro
     Vertragswoche (VOL-STAT-01-Ist), zugesagte Soll-Summe (VOL-ACCT-01-Soll) und
     Delta (Ist − Soll).

  2. Nicht-HR-Rollen (Sales-Person-Self-View, Shiftplanner, User) sehen die
     Ist-/Soll-/Delta-Felder NICHT (API-Level-Redaction: DTO liefert `Option<f32> =
     None`). Der bestehende „Freiwillige Stunden"-Ist-Wert bleibt für alle Rollen
     sichtbar wie heute.

  3. Backend hat die additive Datenmodell-Basis (2 neue Tabellen `rebooking_batch`
     + `rebooking_batch_entry`, UNIQUE-Constraint `(sales_person_id, iso_year,
     iso_week) WHERE deleted IS NULL`, Marker-Spalte auf `extra_hours` für
     spätere Rebooking-Filter) sowie den Toggle-Seed
     `voluntary_rebooking_auto_active_from` (Default `None`) für F4-Stichtag-Guard.

  4. Property-Test „Rebooking-Neutralität für Read-Aggregate": eine fingierte
     Rebooking-Pair-ExtraHours-Row (Marker gesetzt) verändert weder VOL-STAT-01-Ist
     noch VOL-ACCT-01-Soll (VOL-ACCT-03 als CI-Guard erfüllt).

  5. Präzedenz-Docs (`docs/features/F14-rebooking.md` + `_de.md` neu;
     `docs/architecture/02-service-tiers.md` + Runtime-Graph; `03-data-model.md`

     + ER-Diagramm) sind im selben Commit synchron aktualisiert.

**Discuss-Points (in phase-54-discuss-phase zu pinnen):**

- **D-F1-01**: Denominator-Definition für VOL-STAT-01. Option (a) „strikt
  contract-weeks in year" (schlicht) vs. Option (b) „AVG-01-A-22-1 absence-adjusted"
  (analog Phase 41). Research empfiehlt (a) für Konsistenz mit v1.4-CVC-05-Pattern
  („committed_voluntary ist per-Woche-Zusage"), aber Entscheidung offen.

- **D-F2-01**: Mid-Week-Vertragswechsel-Semantik für VOL-ACCT-01-Soll. Optionen
  A pro-rata / B latest-active / C split-week. Research empfiehlt „reuse
  `WorkingHoursService::get_working_hours_for_week`-Semantik" (deckt sich mit
  v1.4-CVC-Precedent).

- **D-54-DM-01**: UNIQUE-Constraint-Shape — `(sales_person_id, iso_year, iso_week)`
  (globale Wochen-Sperre für alle Kinds) vs. `(kind, sales_person_id, iso_year,
  iso_week)`. Research empfiehlt erstere Variante (klarerer Fehler-Fall + Idempotenz

  + Claim-on-Suggest-Strategie).
- **D-54-DM-02**: Marker-Approach auf `extra_hours` — `source: TEXT NOT NULL
  DEFAULT 'manual'` (STRING-Enum) vs. `rebooking_batch_entry_id: Option<BLOB>` (FK
  in eine neue Tabelle). Research neutral, FK dokumentiert die Beziehung expliziter.

**Präzedenzen (Muster zum Kopieren):**

- v2.1 Phase 39 (WeekStatus Basic-Tier + Migration + partial UNIQUE): Basic-Service
  + Migration + REST + FE in einer Phase.
- v2.2 Phase 47 (RPT-01/02/03 pure-fn `weekday_attendance_distribution`): pure-fn
  Read-Aggregat + BL-Service + HR-Gate + FE-Row.

- v1.8 Phase 28 (VAC-OFFSET-01 API-Level-Redaction): `Option<f32> = None` als
  HR-Only-DTO-Pattern.

- v1.4 CVC-05: „kein Snapshot-Bump für Achse-B-only Read-Aggregat" (`committed_voluntary`
  neben `expected_hours` als separates Feld) — Analog-Argument für F2-Soll.

**Pitfalls-Match:** 1 (Doppel-Zählung — Marker-Spalte + Filter von Anfang an),
8 (HR-Only DTO-Redaction), 9 (shared pure-fn), 10 (F2-Soll reuses existing
`get_working_hours_for_week`), 13 (Docs-Freshness synchron im gleichen Commit).

**Backend-Pfade:** `service/src/rebooking_batch.rs` (neu),
`service_impl/src/rebooking_batch.rs` (neu),
`dao/src/rebooking_batch.rs` (neu),
`dao_impl_sqlite/src/rebooking_batch.rs` (neu),
`service/src/voluntary_stats.rs` (neu), `service_impl/src/voluntary_stats.rs` (neu),
`service_impl/src/reporting.rs` (neue pure fn
`committed_voluntary_target_for_year`), `rest/src/reporting.rs` (Endpoint-Ergänzung
oder neues Modul `rest/src/voluntary_stats.rs`), `rest-types/src/employee_report.rs`
(EmployeeReportTO-Ergänzung + neue Struct `VoluntaryStatsTO`),
`shifty_bin/src/main.rs` (DI-Wiring), `migrations/sqlite/` (2 neue Migrationen).

**Frontend-Pfade:** `shifty-dioxus/src/loader.rs` (neu `load_voluntary_stats`),
`shifty-dioxus/src/state/employee_report.rs` (VoluntaryStats-Feld),
`shifty-dioxus/src/component/voluntary_stats_row.rs` (neu, unter
„Freiwillige Stunden" im Employee-Detail-Page),
`shifty-dioxus/src/page/employee_details.rs` (Row-Einbindung, HR-gated via
existierendem Role-Gate), `shifty-dioxus/Dioxus.toml` (Proxy für neuen Endpoint —
MEMORY `feedback_dioxus_proxy_for_new_backend_endpoints.md`),
`shifty-dioxus/i18n/{de,en,cs}/*.ftl` (Row-Labels).

**Plans:** 6/6 plans complete

- [x] 54-01-PLAN.md — Migrations + DAO-Skelett (rebooking_batch/entry, extra_hours.source, toggle-seed) ✅ 2026-07-07 (SUMMARY: 54-01-SUMMARY.md, 880 tests green)
- [x] 54-02-PLAN.md — RebookingBatchService (Basic-Tier, HR-gated CRUD)
- [x] 54-03-PLAN.md — VoluntaryStatsService (BL) + pure fns F1/F2 + Property-Test VOL-ACCT-03
- [x] 54-04-PLAN.md — REST-Endpoint GET /report/{id}/voluntary-stats + VoluntaryStatsTO
- [x] 54-05-PLAN.md — Frontend Row + Loader + i18n de/en/cs
- [x] 54-06-PLAN.md — Docs-Freshness (F14 neu, F07/F08/02-service-tiers/03-data-model + Diagramme)

**UI hint**: yes

---

#### Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5)

**Goal:** HR kann im Mitarbeiter-Jahresreport per 1-Klick eine Umbuchung
Freiwillig ↔ Bezahlt anlegen; für gedeckelte Mitarbeiter mit negativem Konto und
verfügbaren Freiwilligenstunden erscheint in der Employee-Overview eine dauerhafte
Warnzeile, deren Klick ein IST/DANN-Vorschlags-Modal öffnet, in dem HR approve
oder reject.

**Depends on:** Phase 54 (RebookingBatchService, Marker-Spalte, VoluntaryStats-DTO,
`committed_voluntary_target_for_year`).

**Requirements:** REB-MANUAL-01, REB-MANUAL-02, REB-MANUAL-03, HR-ALERT-01,
HR-ALERT-02, HR-ALERT-03, HR-ALERT-04

**Success Criteria** (what must be TRUE):

  1. HR kann im Employee-Year-Report per 1-Klick eine Umbuchung
     `VolunteerWork → ExtraWork` oder `ExtraWork → VolunteerWork` anlegen; ein
     Vorschau-Modal zeigt Menge, Richtung, Woche vor Bestätigung; nach
     Bestätigung sind zwei ExtraHours-Rows (Marker gesetzt) plus ein
     `rebooking_batch(kind=manual)`-Eintrag persistiert und im Report sichtbar.

  2. Für gedeckelte SalesPersons mit `balance < 0` UND `voluntary_ist > 0` zeigt
     die Employee-Overview eine dauerhafte Warnzeile pro Person; die Warnzeile
     verschwindet, sobald das Konto ausgeglichen oder der Vorschlag abgelehnt ist.

  3. Klick auf die Warnzeile öffnet ein Modal mit IST- und DANN-Spalten für
     Stundenkonto (Balance), Freiwillige Ist, Freiwilliges Soll, Freiwilliges
     Delta — alle DANN-Werte backend-computed; kein FE-Arithmetik.

  4. HR-Approve triggert dieselbe Doppel-Eintrag-Semantik wie die manuelle
     Umbuchung, persistiert `rebooking_batch(kind=hr_suggestion, state=approved)`;
     HR-Reject persistiert `state=rejected` (bleibt als Audit sichtbar).
     Concurrency: state-conditional UPDATE (`WHERE state='pending'`,
     affected-rows == 1) verhindert Double-Approve-Race.

  5. Rebooking-Neutralität für Read-Aggregate hält empirisch: Property-Test
     „Manuelles Rebooking-Roundtrip" beweist `balance_before == balance_after`,
     `VOL-STAT-01-Ist_before == _after`, `VOL-ACCT-01-Soll_before == _after`
     (VOL-ACCT-03-Guard, Pitfall 1).

**Discuss-Points (in phase-55-discuss-phase zu pinnen):**

- **D-F5-01**: F5-Alert-Threshold — `balance < 0` (strikt) vs. `balance <= -0.5h`
  (Rundungs-Toleranz)? Pure-fn-Predicate-Test-Matrix als Konsequenz.

- **D-F5-02**: Alert-Payload-Shape — im existierenden `ShortEmployeeReport`-DTO
  additiv (`has_pending_rebooking: bool`, `pending_rebooking_id: Option<Uuid>`)
  vs. separate Sammel-API `GET /rebooking-suggestions/summary`. Research
  empfiehlt DTO-Additiv (weniger Round-Trips).

- **D-F5-03**: F5-Modal-„proposed_rebooking_hours"-Berechnung. Empfehlung:
  `min(|balance|, voluntary_ist − 0)` — deckt sich mit v2.5-VAA-cap-gated-Formel.

- **D-55-UNDO-01**: Undo-Verhalten nach Approve — Research + REQUIREMENTS.md
  sagen „kein Undo, defer to v2.7+" (Anti-Feature REB-UNDO-01). Muss explizit
  bestätigt werden (User-Frage).

**Präzedenzen (Muster zum Kopieren):**

- v1.8 Phase 27 (AbsenceModal/AbsenceFilterBar Gruppierungs-Helfer): Modal-Muster.
- v1.5 Phase 22 (STAT-01/02): HR-only-Statistik-Row.
- v2.5 Phase 53 (VAA-01..04 `sales_person_absences`): Fat-Backend-Prinzip mit
  `#[serde(default)]`-additiver DTO-Erweiterung.

- v2.1 Phase 40 (state-conditional UPDATE + affected-rows-Check für
  Concurrency): Pattern für approve/reject-Race.

- v1.6 Phase 24 (Overage-Warn-Sektion in Weekview): dauerhafte inline Warnzeile
  über HR-gated Read-Aggregat.

**Pitfalls-Match:** 1 (atomarer Tx-Doppel-Eintrag), 6 (UNIQUE-Constraint fängt
F3/F5-TOCTOU), 11 (F5-Alert-Predicate als pure fn + Truth-Table + INT-Sightcheck
statt Dioxus-Browser-Test — MEMORY `reference_dioxus_browser_test_date_inputs.md`),
12 (state-conditional UPDATE), 14 (Dioxus.toml-Proxy für neue Routes —
MEMORY `feedback_dioxus_proxy_for_new_backend_endpoints.md`), 15 (single write-path
für alle drei Rebooking-Trigger).

**Backend-Pfade:** `service/src/rebooking_reconciliation.rs` (neu),
`service_impl/src/rebooking_reconciliation.rs` (neu, BL orchestriert 2×ExtraHours

+ 1×Batch-Entry in einer Tx), `service_impl/src/rebooking_batch.rs` (state-transition

`pending → approved | rejected`), `rest/src/rebooking.rs` (neu: `POST /rebooking/manual`,
`GET /rebooking-suggestions`, `POST .../{id}/approve`, `POST .../{id}/reject`),
`rest-types/src/rebooking.rs` (neu: `ManualRebookingRequestTO`, `RebookingSuggestionTO`,
`RebookingBatchTO`), `rest-types/src/employee_report.rs`
(`ShortEmployeeReportTO`-Erweiterung `has_pending_rebooking` +
`pending_rebooking_id`), `shifty_bin/src/main.rs` (DI-Wiring).

**Frontend-Pfade:** `shifty-dioxus/src/loader.rs`
(`submit_manual_rebooking`, `load_rebooking_suggestions_pending`,
`approve_rebooking_suggestion`, `reject_rebooking_suggestion`),
`shifty-dioxus/src/state/rebooking.rs` (neu; thin `From<&…TO>`-Mapper, keine
Arithmetik), `shifty-dioxus/src/component/rebooking_alert_banner.rs` (neu),
`shifty-dioxus/src/component/rebooking_suggestion_modal.rs` (neu; IST/DANN-Table
mit Approve/Reject),
`shifty-dioxus/src/component/manual_rebooking_modal.rs` (neu; F3-Vorschau + Submit),
`shifty-dioxus/src/page/employees.rs` (Banner-Einbindung),
`shifty-dioxus/src/page/employee_details.rs` (F3-Button in Freiwilligen-Zeile),
`shifty-dioxus/Dioxus.toml` (Proxy `/rebooking`, `/rebooking-suggestions`),
`shifty-dioxus/i18n/{de,en,cs}/*.ftl` (Modal-Labels, Approve/Reject, Banner-Text).

**Plans:** TBD

**UI hint**: yes

---

#### Phase 56: Wochen-Cron + Rollout-Backfill (F4)

**Goal:** Ein Wochen-Cron-Job verarbeitet automatisch die Vorwoche (letzte
abgeschlossene ISO-Woche) und schreibt für jeden gedeckelten SalesPerson mit
Freiwilligen-Überschuss die entsprechende Rebooking-Pair-ExtraHours; ein
admin-konfigurierbarer Stichtag-Toggle schützt historische Balance-Views; ein
HR-Admin-gated Backfill-REST-Endpoint erlaubt einmalige rückwirkende Verarbeitung
mit Dry-Run-Vorschau.

**Depends on:** Phase 55 (RebookingReconciliationService mit
`rebook_manual`-Semantik + Suggest-Decide-Logik).

**Requirements:** REB-AUTO-01, REB-AUTO-02, REB-AUTO-03, REB-AUTO-04, REB-AUTO-05

**Success Criteria** (what must be TRUE):

  1. Ein Wochen-Cron (`VoluntaryRebookingScheduler`, analog `PdfExportSchedulerImpl`
     v2.2) läuft montags früh und verarbeitet die zuletzt abgeschlossene ISO-Woche;
     für jeden `cap_planned_hours_to_expected=true`-SalesPerson mit
     `Ist_vorwoche > Soll_vorwoche + committed_voluntary_vorwoche` wird der Excess
     als Rebooking-Pair (`rebooking_batch(kind=auto_cron)`) persistiert.

  2. Der Admin-Toggle `voluntary_rebooking_auto_active_from` (Präzedenz HCFG-02
     v1.7 + SHC-04 v2.4) wirkt pro Konsumkette korrekt: bei `None` (Default) läuft
     Cron im Skip-Modus (keine Regression); mit Datum wirkt Rebooking nur für
     ISO-Wochen ≥ `active_from`. Property-Test: `active_from = None` → Backend
     `cargo test --workspace` byte-identisch mit v2.5-Baseline.

  3. Cron ist idempotent — UNIQUE-Constraint aus Phase 54 fängt Doppel-Läufe ab
     (`INSERT ... ON CONFLICT DO NOTHING`); zweiter Lauf über bereits verarbeitete
     Woche ist no-op. Claim-on-Suggest (HR-ALERT-04) hält: `state=pending`-Suggestion
     blockiert den Cron-Slot für dieselbe (SalesPerson, ISO-Woche).

  4. HR-Admin kann via `POST /admin/rebooking/backfill?from=YYYY-Www&dry_run=true`
     einen einmaligen rückwirkenden Lauf starten; `dry_run=true` zeigt geplante
     Änderungen ohne DB-Write; `dry_run=false` persistiert
     `rebooking_batch(kind=auto_cron_backfill)`. HR-Admin-Gate durchgesetzt.
     Admin-Card im FE (Settings) analog `pdf_export_config` liefert Bedienoberfläche.

  5. Snapshot-Version-Entscheidung (12 oder 13) ist in Phase-56-discuss-phase
     dokumentiert; falls „12 bleibt": Straddling-Golden-Snapshot-Fixture
     (pre/post/traversierend `active_from`) beweist byte-Identität. Falls „13":
     Bump in `CURRENT_SNAPSHOT_SCHEMA_VERSION` + Migration + `docs/features/
     F08-billing-period.md` + `_de.md` synchron.

**Discuss-Points (in phase-56-discuss-phase zu pinnen):**

- **D-F4-SNAPSHOT-01 (TOP OPEN DECISION):** Snapshot-Schema-Version-Bump 12→13
  ja oder nein. Kriterium (CLAUDE.md-Klausel): „Change the input set the
  computation reads from" — trifft F4-Cron-schreibende Rebooking-ExtraHours-Rows
  zu, auch wenn Kategorien invariant bleiben und Balance-Neutralität pro-Batch
  gilt? Research SUMMARY-Divergenz 2 zeigt PITFALLS vs. FEATURES/ARCHITECTURE/
  STACK gespalten. Beweislast beim „Nein"-Zweig = Straddling-Golden-Snapshot,
  byte-identisch. Wenn diese nicht liefert → Bump.

- **D-F4-CRON-01:** Cron-Cadence + Uhrzeit (Vorschlag Montag 03:00 analog EXP).
  Konflikt mit `PdfExportScheduler`-Uhrzeit? Reihenfolge im Boot-Sequence?

- **D-F4-CHAIN-01:** Chain-Audit-Matrix — welche Konsumketten lesen ExtraHours
  (Balance / F1-Ist / F2-Soll-Nutz / F4-Self-Guard / billing_period.value_type),
  welche Legacy-Semantik pro Chain bei `active_from=None`? Analog HCFG-02 /
  SHC-04 pro-Chain-Rekonstruktion (MEMORY
  `feedback_stichtag_rollout_legacy_semantics.md`).

- **D-F4-STALE-01:** F5-Stale-Vorschlag-Strategie nach F4-Race — Claim-on-Suggest
  (Empfehlung) vs. Fingerprint mit 409. Research empfiehlt Claim-on-Suggest
  wegen einfacherer Idempotenz.

- **D-F4-BACKFILL-01:** Dry-Run-Output-Format (JSON pro Batch-Kandidat mit
  Preview-Werten vs. Sammel-Summary); Pagination bei mehrjährigen Backfills.

- **D-F4-LOCKED-01:** Verhalten bei WeekStatus=Locked — Cron skipt + persistiert
  `rebooking_batch(state='skipped_locked')` (Anti-Pattern 3 aus ARCHITECTURE) vs.
  hartes Fehler-Log.

**Präzedenzen (Muster zum Kopieren):**

- v2.2 Phase 48 (EXP-01/02/03 `PdfExportSchedulerImpl` + `tokio_cron_scheduler`
  0.15.1 + Single-Row-Toggle-Config-Tabelle): 1:1-Blueprint für
  `VoluntaryRebookingScheduler`.

- v2.3.1 Hotfix (Cron-Syntax 5- vs. 6-Feld-Fix): unbedingt 6-Feld-Cron nutzen.
- v1.7 Phase 25 (HCFG-02 `holiday_auto_credit_active_from`) + v2.4 Phase 51
  (SHC-04 `shortday_slot_clipping_active_from`): Stichtag-Toggle mit Chain-Audit-
  Matrix.

- v2.5 Phase 52 Follow-up #3 (`_iso_year`-Bulk-Methoden): ISO-Wochen-Arithmetik
  für „Vorwoche" korrekt handhaben (Pitfall 5).

- v2.5 Phase 52 (WOP-03 Golden-Snapshot-Fixture-Muster, 8 Fixtures):
  Straddling-Fixture für D-F4-SNAPSHOT-01-Beweislast.

- v2.1 Phase 40 (WST TOCTOU + WeekLocked-Behandlung): Cron-vs-WeekStatus-Interaktion.

**Pitfalls-Match:** 2 (Snapshot-Bump-Entscheidung mit Straddling-Fixture),
3 (Chain-Audit-Matrix + toggle-off Golden-Snapshot), 4 (UNIQUE-Constraint aus
Phase 54 + Claim-on-Suggest), 5 (ISO-Wochen-Helper), 7 (Stale-Vorschlag),
13 (Docs F07 + F08 + F14 synced), 16 (6-Feld-Cron-Syntax analog v2.2 EXP +
v2.3.1-Hotfix).

**Backend-Pfade:** `service/src/voluntary_rebooking_scheduler.rs` (neu),
`service_impl/src/voluntary_rebooking_scheduler.rs` (neu; 1:1-Spiegel von
`pdf_export_scheduler.rs`), `service_impl/src/rebooking_reconciliation.rs`
(neue Methoden `run_for_week`, `backfill_range`), `rest/src/rebooking.rs`
(Endpoint `POST /admin/rebooking/backfill`), `rest-types/src/rebooking.rs`
(`BackfillRequestTO`, `BackfillPreviewTO`),
`service_impl/src/billing_period_report.rs` (nur wenn D-F4-SNAPSHOT-01 = „ja":
`CURRENT_SNAPSHOT_SCHEMA_VERSION` Bump auf 13 + Migration), `shifty_bin/src/main.rs`
(Scheduler-Boot am Ende von `main()` nach `pdf_export_scheduler.start()`).

**Frontend-Pfade:** `shifty-dioxus/src/loader.rs`
(`trigger_rebooking_backfill_dry_run`, `trigger_rebooking_backfill_apply`),
`shifty-dioxus/src/page/settings.rs` (neue Admin-Card unter „Automatische
Freiwilligen-Umbuchung"; Stichtag-Toggle-Editor + Backfill-Button + Dry-Run-
Vorschau; Präzedenz `pdf_export_config`-Card),
`shifty-dioxus/Dioxus.toml` (Proxy `/admin/rebooking`),
`shifty-dioxus/i18n/{de,en,cs}/*.ftl` (Card-Labels, Backfill-Button,
Dry-Run-Vorschau-Header).

**Plans:** TBD

**UI hint**: yes

---

### Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 54. Data-Model + Voluntary Statistics (F1 + F2) | 6/6 | Complete   | 2026-07-07 |
| 55. Manuelle Umbuchung + HR-Alert-Modal (F3 + F5) | 0/? | Not started | - |
| 56. Wochen-Cron + Rollout-Backfill (F4) | 0/? | Not started | - |

Requirements-Coverage: 17/17 mapped (5 → P54, 7 → P55, 5 → P56).

## Backlog

Ungeplante / off-theme Arbeit, die NICHT zum aktiven Milestone gehört. Vor Ausführung
in einen Milestone promoten oder per `/gsd-plan-phase 999.1` direkt planen.

- [ ] **Phase 999.1: Breaking/Major Dependency-Migration** (Backend + Frontend, Maintenance) — Alle direkten Deps mit verfügbaren Major-Releases über beide Cargo-Workspaces (Backend-Root + `shifty-dioxus/`, 9 Member-Crates) auf den neuen Major heben (Cargo.toml-Constraint-Edits + Code-/API-Migration).

  **Goal:** Reproduzierbares Breaking-Update-Tooling etabliert und alle tragbaren Major-Bumps migriert, mit grünen Gates über beide Workspaces — ohne die heiklen Pins (dioxus 0.6.x) ungefragt anzufassen.

  **Context:** Quick-Task `260627-vgo` hat die **semver-kompatible** Baseline bereits geliefert (nur Cargo.lock, alle Gates grün). Offen ist NUR der Breaking/Major-Teil, der dort eskaliert wurde, weil die gepinnte **stable cargo 1.95.0** kein `cargo update --breaking` kann (nightly-only) und weder `cargo-edit` (`cargo upgrade`) noch `cargo-outdated` noch `+nightly` verfügbar sind.

  **Scope / grobe Wave-Struktur:**

  - Task 1 — Toolchain-Enabler: nightly-Toolchain bzw. `cargo-edit`/`cargo-outdated` ins `flake.nix` aufnehmen, sodass `cargo update --breaking` oder `cargo upgrade --incompatible` reproduzierbar laufen.
  - Task 2 — Major-Bump-Inventar: welche direkten Deps, welcher Sprung, Changelog-/Breaking-Risiko (beide Workspaces).
  - Task 3 — iterativ pro Major migrieren mit Gates: Backend `cargo build` + `cargo clippy --workspace -- -D warnings` + `cargo test --workspace`; Frontend `cargo build --target wasm32-unknown-unknown` (nix-shell -p openssl pkg-config lld) + `cargo test`.

  **Constraints:**

  - **dioxus-Major** (0.6.x-Pin) NUR mit expliziter User-Freigabe — dx-CLI-0.7-Inkompatibilität dokumentiert (App startet nicht + Design gestrippt).
  - `flake.lock` Nix-Inputs sind NICHT Teil dieser Phase (separater Maintenance-Job).
  - jj-Repo: User committet manuell, keine git-Fallbacks.

  **Depends on:** Quick-Task `260627-vgo` (compatible baseline) ✅
  **Plans:** 1/1 plans complete

---

*Last updated: 2026-07-06 — v2.6 Roadmap frisch geschrieben (Phasen 54–56, 17/17 Requirements gemappt, ARCHITECTURE-C 3-Phasen-Baseline). Backlog 999.1 unverändert erhalten.*
