# Project Research Summary

**Project:** Shifty — v2.6 Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter
**Domain:** HR / Time-Tracking — voluntary-vs-paid reconciliation on top of shipped payroll-adjacent balance stack
**Researched:** 2026-07-06
**Confidence:** HIGH (all four researcher outputs converge on stack + architecture; two decisions explicitly flagged as open — see Critical Synthesis Points)

## Executive Summary

Shifty v2.6 adds a five-feature reconciliation stack (F1 Ø freiwillig / Vertragswoche, F2 Freiwilliges Konto Soll/Ist, F3 manuelle 1-Click-Umbuchung, F4 wöchentlicher Auto-Cron, F5 HR-Alert + Vorschlags-Modal) on top of primitives shipped in v1.4 (`committed_voluntary`), v1.5 (Employee-Year-Report), v1.7/v2.4 (`ToggleService` Stichtag), v2.1 (AVG-01 read-aggregate + HR-Gate), v2.2 (`tokio_cron_scheduler` + Admin-gated Cron-Config), v2.5 (VAA cap-gated formula + `find_by_iso_year`-Pattern). It is a business-logic / data-model milestone, **not** an infrastructure milestone — zero neue Cargo-Dependencies.

Der empfohlene Kern-Architektur-Move: zwei additive SQLite-Tabellen (`rebooking_batch` Parent + `rebooking_batch_entry` Child), ein `kind`-Diskriminator (`manual` / `hr_suggestion` / `auto_cron` / `auto_cron_backfill`), UNIQUE-Index `(sales_person_id, iso_year, iso_week)` als kombinierter Idempotenz- + TOCTOU- + Doppel-Zählungs-Guard. Neue Services: `RebookingBatchService` (Basic, Entity-Manager), `RebookingReconciliationService` (BL, orchestriert 2× ExtraHours-Doppel-Eintrag + Batch in einer Tx), `VoluntaryStatsService` (BL, HR-gated Read-Aggregat für F1/F2), `VoluntaryRebookingScheduler` (BL, 1:1-Spiegel von `PdfExportSchedulerImpl`). Backfill läuft als HR-gated REST-Endpoint mit `?dry_run=true` — kein `clap`, kein Rebuild für One-Shot-Rollout.

Die Risiken konzentrieren sich auf drei Punkte: (a) **Doppel-Zählung** durch F4-Rebooking-ExtraHours, die in F1-Ist / F2-Soll-Nutz / Balance zurück-lecken — geblockt durch Marker-Spalte + UNIQUE-Constraint + drei Property-Tests (`balance_before == balance_after`, `F1_before == F1_after`, `F2_Soll_before == F2_Soll_after`); (b) **Stichtag-Rollout mit falscher Legacy-Semantik pro Chain** — analog HCFG-02 / SHC-04, jede Konsumkette braucht explizite pre-Stichtag-Semantik + golden-snapshot-Regression; (c) **Snapshot-Schema-Version-Bump 12→13** — hier divergieren die Researcher, siehe „Critical Synthesis Points" unten. Fat Backend / Thin Client bleibt durchgängig — FE rendert nur pre-computed DTO-Felder, keine Arithmetik.

## Critical Synthesis Points (Divergenz zwischen Research-Files — nicht glätten)

### Divergenz 1: Phase-Dekomposition (FEATURES vs. STACK vs. ARCHITECTURE)

Alle drei schlagen unterschiedliche Phasen-Zerlegungen vor. Der Roadmapper entscheidet.

| Vorschlag | Quelle | Phasen-Count | Struktur | Rationale | Trade-off |
|-----------|--------|--------------|----------|-----------|-----------|
| **A (4 Phasen)** | FEATURES.md | 4 | (1) F1+F2 Read-Only + DTO; (2) Migrations + `RebookingService`-Skeleton + F3; (3) F5; (4) F4 + Backfill | Cheapest-first, F4 zuletzt weil highest-blast-radius nach validierter F1..F3+F5-Realität | Migrations spät → risiko dass Schema-Entscheidung durch später auftauchende F4-Anforderung nachgezogen werden muss |
| **B (6 Phasen)** | STACK.md | 6 | Migration/DAO → Service-Core → F3 → F1+F2 parallel → F4 → F5 | Kleinste Waves; DAO+Basic-Service zuerst gibt sauberen Fundament-Sprint; F1+F2 parallelisierbar | Overhead durch viele Phasen-Handoffs; parallele F1+F2 braucht Koordination |
| **C (3 Phasen)** | ARCHITECTURE.md | 3 | (Ph 54) Data-model + F1+F2 Stats; (Ph 55) F3 + F5 ohne Cron; (Ph 56) F4 + Backfill | Jede Phase ist ein UAT-anchorbarer Vertikal-Slice (SPIDR-artig); Data-Model + Stats in einer Phase weil Toggle-Seed + Table-Migration + F1/F2 read-only alle klein sind | Ph 54 vermischt Migrations (Schema-Locked) + Read-Aggregat (Compute-Locked); Reviewer-Load pro Phase höher |

**Konvergenz zwischen A und C:** F4 zuletzt, F1+F2 zuerst, F3+F5 in der Mitte. **B ist der Ausreißer** mit F3 vor F1+F2 — Argument: Rebooking-Primitive vor Read-Aggregat, damit F1/F2 die Rebooking-Marker-Filter direkt konsumieren. Roadmapper sollte in discuss-phase klären, ob F3 wirklich vor F1/F2 muss (STACK-Argument) oder ob die Marker-Filter-Semantik aus Migration+DAO allein reicht (A/C-Annahme).

### Divergenz 2: Snapshot-Schema-Version 12→13 — Bump ja/nein (TOP OPEN DISCUSS-PHASE DECISION)

**FEATURES.md, ARCHITECTURE.md, STACK.md sagen alle drei: „kein Bump, default = 12 bleibt".**

Begründung dort:
- F3/F4/F5 schreiben nur zwei ExtraHours-Zeilen (`VolunteerWork −N` / `ExtraWork +N`) plus 1× `rebooking_batch(_entry)`. ExtraHours fließen bereits in existierende `BillingPeriodValueType`-Terms (`working_hours`, `absense_hours`), also **kein neuer** `value_type`.
- Präzedenz v1.4 CVC-05: derive-on-read wins → kein Persist eines neuen `VoluntaryReconciled`-value_type.
- F1/F2 sind reine Read-Aggregate, schreiben nichts.

**PITFALLS.md widerspricht explizit — Pitfall 2:**

Zitat: „Wenn F4 einen Stichtag-guarded change of `reporting.rs`-input-set (F4 rebookings are visible to `balance`) einführt, triggert das `CLAUDE.md`-Klausel *„Change the input set the computation reads from"* — YES → **Bump 12→13 anyway.**"

Argumente PRO Bump (PITFALLS-Sicht):
1. F4-Cron schreibt neue ExtraHours-Rows, die zwischen Reporting-Woche und Billing-Period-Close in das persistente Snapshot fließen. Alte Snapshots (vor F4-Rollout) unterscheiden sich systematisch von Neu-Rechnungen → Validator kann nicht mehr zwischen echtem Data-Bug und Regel-Change unterscheiden.
2. Präzedenz v1.5 UV/YV: Bump 9→10, weil derived-absences-Merge den Input-Set für `vacation_days` änderte — **ohne** neuen value_type.
3. Präzedenz v1.8 VAC-OFFSET-01: Bump 11→12 für neuen value_type — aber die *Klausel* triggert auch ohne neuen value_type.
4. CLAUDE.md-Kontrakt explizit: *„Change the input set the computation reads from (e.g., starting/stopping to include a category of extra_hours)"* — F4 startet, eine neue *Quelle* von ExtraHours-Rows für existing categories zu schreiben. Das ist ein Input-Set-Change.

Argumente CONTRA Bump (FEATURES/ARCHITECTURE/STACK-Sicht):
1. Die *Kategorien* der ExtraHours-Rows sind unverändert (`VolunteerWork`, `ExtraWork`). Die Formel liest dieselben Kategorien wie vor v2.6.
2. Rebooking ist balance-neutral per Definition (`−N` + `+N` in derselben Person, derselben Woche) — Snapshots vor und nach Rebooking müssen identisch bleiben, sonst ist der Rebooking-Contract gebrochen.
3. Stichtag-Guard verhindert historische Divergenz: F4 läuft nur für Wochen ≥ `active_from`, alte Snapshots sind reproduzierbar.
4. Bump „vorsorglich" ohne neuen value_type invalidiert jede historische Snapshot-Validierung ohne inhaltlichen Grund (STACK-Anti-Pattern).

**Empfehlung an discuss-phase:** Nicht in SUMMARY.md pinnen. Frage explizit auf F4-DISCUSS-Agenda: „Ist die Introduction einer neuen ExtraHours-Quelle (F4-Cron-schreibende Rebooking-Pair-Rows) ein Input-Set-Change im Sinne der CLAUDE.md-Klausel — auch wenn die Kategorien und die Balance-Neutralität invariant bleiben?" Beweislast beim „Nein"-Zweig: Golden-Snapshot-Fixture, die eine Periode strikt vor + strikt nach + straddling F4-Stichtag durchrechnet und byte-identisch bleibt. Wenn die nicht liefert, muss gebumpt werden.

## Key Findings

### Recommended Stack

Zero neue harte Cargo-Dependencies. v2.6 baut auf bereits gepinnter Baseline auf.

**Kern-Technologien (alle reuse):**
- `tokio-cron-scheduler 0.15.1` — F4 Wochen-Cron; identisches Pattern wie `PdfExportSchedulerImpl` (v2.2/v2.3), 6-Feld-Cron, `Arc<Mutex<Option<JobScheduler>>>` + dormant-boot + `reload_from_db`.
- `sqlx 0.8.6` (workspace 0.8.2) — 2 additive Migrationen (`rebooking_batch`, `rebooking_batch_entry`), evtl. 3. für Toggle-Seed. `cargo sqlx prepare --workspace` nach jeder neuen `query!`, `.sqlx/*.json` committen (CI läuft `SQLX_OFFLINE=true`).
- `ExtraHoursService::create` + `TransactionDao` — Doppel-Eintrag-Pair (`−N VolunteerWork` / `+N ExtraWork`) atomar in einer Tx via `use_transaction`. Kein Saga, kein Outbox.
- `ToggleService` (`voluntary_rebooking_auto_active_from`) — Stichtag-Gate analog HCFG-02 (v1.7) und SHC-04 (v2.4). Toggle-off = raw Legacy-Semantik pro Chain (siehe Pitfall 3).
- `time 0.3.36` + `ShiftyDate`/`ShiftyWeek` — ISO-Wochen-Arithmetik (KEIN `chrono`), v2.5-WOP-Follow-up-#3-`_iso_year`-Helper wiederverwenden für F4-„Vorwoche".

**Explizit ausgeschlossen:** kein `clap` (Backfill = HR-gated REST-Endpoint), kein zweites Scheduler-Framework, kein SSE/WebSocket für F5-Alerts (REST-Poll reicht), kein neuer FE-State-Store, kein Job-Queue-Substrat, keine Config-File.

Details: `.planning/research/STACK.md`.

### Expected Features

Fünf Features (F1..F5) plus Backfill-CLI-als-REST-Endpoint. F1+F2 = Read-Aggregate; F3 = manuelle Doppel-Eintrag-Umbuchung; F4 = wöchentlicher Auto-Cron; F5 = HR-Alert + Vorschlags-Modal mit approve/reject.

**Must-have (table stakes):**
- **F1 Ø freiwillig geleistete Stunden / Vertragswoche / Jahr** — HR-only Read-Aggregat, Zähler = Σ `VolunteerWork` ExtraHours pro Jahr, Nenner = Wochen mit gültiger `working_hours`-Zeile (D-F1-01: exakte Definition analog AVG-01 A-22-1 in discuss-phase).
- **F2 Freiwilliges Stundenkonto (Soll/Ist/Delta)** — HR-only. Soll = Σ (`WorkingHours(week).committed_voluntary × Wochen-in-Kraft`), Ist = F1-Zähler. Rendert unter Balance-Block in existierendem `/employees/:id`-Report.
- **F3 Manuelle Umbuchung Freiwillig ↔ Bezahlt (1-Click)** — atomarer Doppel-Eintrag via `RebookingReconciliationService::rebook_manual`; Modal im Employee-Year-Report.
- **F5 HR-Alert bei gedeckeltem Mitarbeiter mit negativem Konto** — persistenter Warn-Banner in Employee-Overview; Predicate: `cap_planned_hours_to_expected = true` AND `balance < 0` AND `voluntary_ist > 0`. Klick öffnet Vorschlags-Modal mit IST / DANN-Spalten (alle Zahlen backend-computed, Fat-Backend-Prinzip). Approve/Reject persistiert als `rebooking_batch(kind=hr_suggestion, state=approved|rejected)`.

**Should-have (Differentiator):**
- **F4 Automatische Umbuchung (Wochen-Cron)** — Vorwoche = letzte abgeschlossene ISO-Woche; für jeden `cap=true`-SalesPerson: wenn `Ist > Soll + committed_voluntary`, umbuche `excess` als Pair-ExtraHours in `rebooking_batch(kind=auto_cron)`. Stichtag-Guard via `voluntary_rebooking_auto_active_from`.
- **F4 Rollout-Backfill (als REST, nicht CLI)** — `POST /rebooking/backfill?from=YYYY-Www&dry_run=true`, HR-Admin-gated, iteriert historische Wochen, schreibt `kind=auto_cron_backfill`. UNIQUE-Constraint aus Pitfall-4 fängt Doppel-Backfill ab.

**Defer (v2.7+):**
- Employee-Self-Service-View des Freiwilligen-Kontos (bleibt HR-only in v2.6).
- Notifications (Email / iCal / Push) auf F4-Completion oder F5-Alert.
- Multi-Role-Approval-Workflow auf F5-Batches (single-step approve/reject genügt).
- Undo/Rollback applied Batches (Schema unterstützt spätere Extension via stable batch.id).
- UI zur Visualisierung der Rebooking-Batch-Historie.
- Alerts für nicht-gedeckelte Mitarbeiter mit Voluntary-Überschuss (nicht zielgruppen-relevant).

**MVP-Fallback wenn Scope schrumpfen muss:** F1 + F2 + F3 als Reduktion — F4/F5 auf v2.7 vertagen, Tabellen dann auch nicht vorab anlegen (leere Tabellen = drift-fuel).

Details: `.planning/research/FEATURES.md`.

### Architecture Approach

Fat Backend / Thin Client (jeder Wert im DTO backend-berechnet), strenge Basic-vs-Business-Logic-Tier-Konvention, zwei additive SQLite-Tabellen mit Marker-Spalte + UNIQUE-Index. Kein neuer Cargo-Dep, keine neue Framework-Familie.

**Major components (neu):**
1. **`RebookingBatchService`** (Basic, Entity-Manager) — CRUD auf `rebooking_batch` + `rebooking_batch_entry`, state-transitions (`pending → approved | rejected | skipped_locked`), pure list/find. Konsumiert nur `RebookingBatchDao` + `PermissionService` + `TransactionDao`.
2. **`RebookingReconciliationService`** (BL) — orchestriert: compute account → decide rebooking → in einer Tx: 2× ExtraHours-Create + Batch-Entry-Write. Einziger Schreib-Pfad für alle drei Trigger (F3 manual, F4 auto-cron, F5 approve). Konsumiert `RebookingBatchService`, `ExtraHoursService`, `ReportingService`, `WorkingHoursService`, `SalesPersonService`.
3. **`VoluntaryStatsService`** (BL, read-only) — HR-gated pure fn `voluntary_hours_per_contract_week(sp_id, year)` + `committed_voluntary_target_for_year(&[EmployeeWorkDetails], year)`. Ist/Soll/Delta für F1+F2. Getrennt vom bereits fetten `ReportingService` (8 Deps).
4. **`VoluntaryRebookingScheduler`** (BL) — 1:1-Spiegel von `PdfExportSchedulerImpl`; `start()` lazy-init, `reload_from_db()`, per-Tick delegiert an `RebookingReconciliationService::run_for_week(prev_iso_year, prev_iso_week)`. Boot am Ende von `main()` nach dem PDF-Scheduler.
5. **`RebookingBatchDao`** (DAO) — SQL gegen die zwei neuen Tabellen. Soft-Delete-Konvention (`WHERE deleted IS NULL`) + optimistic-lock-`version` + `update_process`-Audit.

**Data model (additive):**

```
rebooking_batch (id BLOB PK, kind TEXT, state TEXT, booking_year INT?, booking_week INT?,
                 created TIMESTAMP, approved TIMESTAMP?, approved_by TEXT?,
                 deleted TIMESTAMP?, version BLOB, update_process TEXT)
rebooking_batch_entry (id BLOB PK, batch_id BLOB FK, sales_person_id BLOB, hours REAL,
                       balance_before REAL, voluntary_actual REAL, voluntary_committed REAL,
                       extra_hours_out_id BLOB?, extra_hours_in_id BLOB?,
                       deleted TIMESTAMP?, version BLOB, update_process TEXT)
UNIQUE INDEX (sales_person_id, iso_year, iso_week) WHERE deleted IS NULL
INDEX (state) WHERE deleted IS NULL
INDEX (sales_person_id) WHERE deleted IS NULL
```

**Modified DTOs (alle additive, `#[serde(default)]` guarded per v2.5-Präzedenz):**
- `EmployeeReportTO` +Felder `voluntary_hours_per_contract_week: f32`, `committed_voluntary_target: Option<f32>` (HR-only, None für Self-View), `voluntary_balance: Option<f32>`.
- `ShortEmployeeReportTO` +Felder `has_pending_rebooking: bool`, `pending_rebooking_id: Option<Uuid>` (F5-Banner).
- Neu: `VoluntaryStatsTO`, `RebookingSuggestionTO`, `ManualRebookingRequestTO`, `RebookingBatchTO`, `RebookingBatchEntryTO`.

**Zyklen-Check:** Keine neuen BL→BL-Kanten außer `RebookingReconciliationService → ReportingService` (BL-liest-BL, präzedent-kompatibel mit `BookingInformationService → ReportingService`). Kompiliert unter `gen_service_impl!`-DI-Graph.

**Docs-Freshness-Trigger (CLAUDE.md-Gate):**
- Neu: `docs/features/F14-rebooking.md` + `_de.md` (beide Sprachen synchron im gleichen Commit).
- Update: `docs/features/F07-reporting-balance.md` + `_de.md` (Balance-Chain sieht keine Rebooking-Marker).
- Update: `docs/features/F08-billing-period.md` + `_de.md` (narrative Note zu ExtraWork-aus-Rebooking, plus ggf. Snapshot-Version-Bump-Historie).
- Update: `docs/architecture/02-service-tiers.md` + `docs/architecture/diagrams/service-graph-runtime.mmd` (drei neue BL-Services + Scheduler).
- Update: `docs/architecture/03-data-model.md` + `db-schema-er.mmd` (2 neue Tabellen).

Details: `.planning/research/ARCHITECTURE.md`.

### Critical Pitfalls

1. **Doppel-Zählung durch F4-Rebooking-ExtraHours in F1-Ist / F2-Soll-Nutz / Balance** — F4 schreibt zwei ExtraHours-Zeilen; ohne Marker-Filter zählt F1 die `−N`-Korrektur als „weniger volunteer geleistet", das „verbleibende freiwillige Konto" widens jede Reconciliation. **Prevention:** Marker-Spalte `rebooking_batch_entry_id` (oder `source: 'rebooking'`) auf `extra_hours`, jeder Aggregat-Reader (F1, F2, `reporting.rs::balance`, `booking_information::get_weekly_summary`) filtert explizit. Drei Property-Tests: `balance_before == balance_after`, `F1_Ist_before == F1_Ist_after`, `F2_Soll_before == F2_Soll_after`. Golden-Snapshot-Re-Run über alle 8 WOP-03-Fixtures byte-identisch, wenn keine Rebookings existieren.
2. **Snapshot-Schema-Version 12→13 — Bump ja/nein (TOP OPEN DECISION, siehe Divergenz 2 oben)** — FEATURES/ARCHITECTURE/STACK sagen „nein"; PITFALLS warnt „wahrscheinlich ja per CLAUDE.md-Input-Set-Klausel". Nicht in SUMMARY.md pinnen — in F4-discuss-phase entscheiden, Beweislast beim „Nein"-Zweig ist ein Straddling-Golden-Snapshot.
3. **Stichtag-Rollout mit falscher Legacy-Semantik pro Chain** — analog HCFG-02 (v1.7), SHC-04 (v2.4), D-51-07: pro Konsum-Kette (Balance-Chain, F1-Ist-Statistik, F2-Soll/Nutz-Anzeige, F4-Cron-Self-Guard) explizite pre-Stichtag-Semantik rekonstruieren, nicht blind „None → raw" annehmen. Property-Test: `active_from = None` → byte-identisch mit v2.5-Baseline.
4. **Cron-Idempotenz + TOCTOU zwischen F3-manual / F4-auto / F5-approve** — UNIQUE-Index `(sales_person_id, iso_year, iso_week)` (ohne `kind`!) als kombinierter Idempotenz- + Race-Guard. INSERT ... ON CONFLICT DO NOTHING; zweiter Klick / Cron-Restart / Backfill über bereits-verarbeitete-Wochen bekommt 409 `week-already-reconciled`. State-conditional UPDATE für approve/reject (`WHERE state = 'pending'`, affected-rows == 1).
5. **ISO-Woche vs. Kalender-Woche im Cron-"Vorwoche"** — `Date::iso_week_year()` ≠ `Date::year()` um Jan 1; die exakte Bug-Klasse aus WOP-Follow-up #3 (v2.5). F4 nutzt `ShiftyWeek::previous()` + `_iso_year`-Helper, nicht `chrono::Datelike::year()`. Regressionstest am Jahresrand.

Weitere moderate/minor Pitfalls (F5-Stale-Vorschlag, HR-Only-DTO-Leak, mid-week-Contract-Change, Dioxus-Testability, Dioxus.toml-Proxy, Docs-Drift) siehe `.planning/research/PITFALLS.md` — insgesamt 17 durchnummerierte Pitfalls plus Phase-spezifische Warnung-Tabelle.

## Implications for Roadmap

**Agreed baseline (alle drei Vorschläge konvergieren darauf):**
- F1+F2 = Read-first (niedrigstes Regressions-Risiko), F4 = zuletzt (höchster Blast-Radius, profitiert von validiertem F1..F3+F5 als Ground-Truth).
- Migrations + Marker-Spalte + UNIQUE-Constraint MÜSSEN in der ersten Backend-Phase landen, damit alle nachgelagerten Aggregate-Reads die Filter-Semantik von Anfang an haben.
- Stichtag-Toggle-Seed (`voluntary_rebooking_auto_active_from`) in der ersten F4-berührenden Phase, vor jeder Rebooking-Logik.
- Docs-Freshness (F07 + F08 + neue F14, jeweils EN+DE) im gleichen Commit wie Code-Diff — kein deferred_item.

**Empfohlener Start (Vorschlag C aus ARCHITECTURE, mit Fallback auf A/B je nach discuss-phase-Entscheidung):**

### Phase 54 — Data-model + Voluntary Stats (F1 + F2)
**Rationale:** Migrations + `RebookingBatchDao` + `RebookingBatchService` (Basic-CRUD, noch keine Logik) etablieren die Marker-Spalte + UNIQUE-Constraint für alle folgenden Phasen. F1+F2 lieferen sichtbaren Wert (Statistik im Employee-Year-Report) ohne Schreib-Pfad.
**Delivers:** Zwei additive Migrations, Toggle-Seed, `RebookingBatchService` (Basic), `RebookingBatchDao`, pure fn `committed_voluntary_target_for_year`, `VoluntaryStatsService` (BL, HR-gated), `GET /reporting/employee/{y}/{spid}/voluntary-stats`, FE-Row „Freiwillige Stunden — Ist / Soll / Konto" im Employee-Detail-Page.
**Addresses:** F1, F2.
**Uses:** existierender `ExtraHoursService`, `WorkingHoursService`, `ToggleService`, `sqlx`-Migration-Konvention.
**Avoids:** Pitfall 1 (Marker-Spalte + Filter von Anfang an), Pitfall 8 (HR-Only DTO-Redaction via `Option<f32> = None` für Non-HR), Pitfall 9 (shared pure fn `voluntary_hours_for_person_in_range` als single-source-of-truth), Pitfall 10 (F2-Soll reuses `WorkingHoursService::get_working_hours_for_week`-Semantik statt eigenem code).

### Phase 55 — Manuelle Umbuchung + HR-Alert + Vorschlags-Modal (F3 + F5 ohne Cron)
**Rationale:** F5-Modal braucht F1/F2-Zahlen als „IST"-Spalte; F3-Rebooking-Primitive braucht `RebookingReconciliationService`, der auch F5-Approve konsumiert. Beide zusammen liefern einen manuellen End-to-End-Reconciliation-Flow, verifizierbar vor Cron-Automatisierung.
**Delivers:** `RebookingReconciliationService.rebook_manual` (2× ExtraHours + 1× Batch in einer Tx), `POST /rebooking/manual`, `RebookingReconciliationService.suggest_for_sales_person` (pure decide-logic) + `.approve`, `RebookingBatchService.reject`, `GET /rebooking-suggestions?state=pending`, FE-Alert-Banner in `page/employees.rs` + Vorschlags-Modal mit IST/DANN-Spalten.
**Addresses:** F3, F5.
**Uses:** `ExtraHoursService::create` + `TransactionDao` (existierend), `RebookingBatchService` (aus Phase 54), `PermissionService`.
**Avoids:** Pitfall 1 (atomarer Tx-Doppel-Eintrag über `Authentication::Full`), Pitfall 6 (UNIQUE-Constraint fängt F3/F5-TOCTOU), Pitfall 11 (F5-Alert-Predicate als pure fn + 8-Case-Truth-Table + INT-Sightcheck statt Dioxus-Browser-Test), Pitfall 12 (state-conditional UPDATE für approve/reject), Pitfall 14 (Dioxus.toml-`[[web.proxy]]` für neue Routes), Pitfall 15 (single write-path für alle drei Rebooking-Trigger).

### Phase 56 — Wochen-Cron + Backfill-Endpoint (F4)
**Rationale:** F4 wickelt F3-Logik in Cron-Automatisierung — keine neue Domain-Logik, nur Scheduler + Stichtag-Guard + Backfill. Ganz zuletzt, damit F1..F3+F5 als Ground-Truth für „was hätte F4 hier ausgerechnet?"-Validierung dienen.
**Delivers:** `VoluntaryRebookingScheduler` (analog `PdfExportSchedulerImpl`), Wochen-Cron (Montag 03:00 für Vorwoche), `RebookingReconciliationService.run_for_week`, `POST /admin/rebooking/backfill?from=YYYY-Www&dry_run=true`, FE-Admin-Card analog `pdf_export_config`.
**Addresses:** F4 + Rollout-Backfill.
**Uses:** `tokio-cron-scheduler 0.15.1` (existierend), `ToggleService` (Stichtag), F3/F5-Rebooking-Primitive aus Phase 55.
**Avoids:** Pitfall 2 (Snapshot-Bump-Entscheidung in DISCUSS + Straddling-Fixture), Pitfall 3 (Chain-Audit-Matrix + toggle-off Golden-Snapshot), Pitfall 4 (UNIQUE-Constraint aus Phase 54), Pitfall 5 (ISO-Wochen-Helper aus v2.5 WOP-Follow-up-#3), Pitfall 16 (6-Feld-Cron-Syntax analog v2.2 EXP nach v2.3.1-Fix), Pitfall 13 (Docs F07 + F08 + F14 synced).

### Phase Ordering Rationale
- **Data-model first (Phase 54):** Marker-Spalte + UNIQUE-Constraint MÜSSEN existieren, bevor irgendein Aggregat-Reader (F1, F2, Balance) drauf filtert. Verhindert nachträgliches „Filter überall retro-fitten" (Pitfall 1).
- **Read-only vor Write (F1/F2 vor F3):** Regressionsrisiko null; Numerator-/Denominator-/Chain-Bugs sichtbar in Statistik ohne Schreib-Nebenwirkungen.
- **Manual write vor Cron write (F3+F5 vor F4):** Der manuelle Pfad ist per-Aktion beobachtbar in UAT; sobald F4 automatisch schreibt, ist die Ground-Truth „das ist korrekt so" schwerer zu etablieren.
- **Cron zuletzt:** Höchster Blast-Radius (Snapshot-Impact, Chain-Legacy-Semantik, ISO-Woche, Idempotenz) — profitiert von allen vorherigen Verifikationen als Ground-Truth.

**Fallback bei Scope-Schrumpfung:** MVP = F1 + F2 + F3 (Phase 54 + F3-Teil aus Phase 55). F4 + F5 auf v2.7 vertagen; Tabellen dann NICHT vorab anlegen (leere Tabellen = drift-fuel — FEATURES.md-Empfehlung).

### Research Flags

Phasen mit likely deeper-research-Bedarf während planning:
- **Phase 54:** Denominator-Definition für F1 (AVG-01 A-22-1: „Wochen mit contract, absence-adjusted" vs. simpler „contract-weeks in Jahr") + Mid-Week-Vertragswechsel-Semantik für F2-Soll (Option A pro-rata / B latest-active / C split-week) sind offen. `/gsd-plan-phase --research-phase 54` empfohlen.
- **Phase 56:** Snapshot-Bump-Entscheidung (Divergenz 2), Chain-Audit-Matrix (welche Konsumketten lesen ExtraHours, welche Legacy-Semantik pro Chain), F5-Stale-Vorschlag-Strategie (fingerprint vs. claim-on-suggest). `/gsd-plan-phase --research-phase 56` dringend empfohlen.

Phasen mit standard patterns (skip research-phase):
- **Phase 55:** Manuelle Umbuchung + Alert-Banner + Modal folgen etablierten Präzedenzen (v1.5 UV-Modal, v1.8 VAC-OFFSET-Editor, v2.5 VAA-Banner). Pure-Predicate-Tests + INT-Sightcheck sind gesetzte Konvention (D-25-06, D-49-13).

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Alle Reuse-Punkte an im Repo bereits gelieferten Präzedenzen (v1.0/1.4/1.7/1.8/2.1/2.2/2.4/2.5); crates.io-Verifikation für `tokio-cron-scheduler 0.15.1` + `clap 4.6.1`. |
| Features | HIGH | Alle Upstream-Services + DTOs direkt aus Repo gelesen, gegen PROJECT.md + v2.5-REQUIREMENTS cross-checked. |
| Architecture | HIGH | Jede Design-Entscheidung an einem konkreten shipped-precedent gegroundet (`PdfExportSchedulerImpl`, `BillingPeriodService`/`BillingPeriodReportService`, `BookingInformationService`, `committed_voluntary_for_calendar_week`). Kein invention-cost. |
| Pitfalls | HIGH | 17 durchnummerierte Pitfalls, jeder mit konkretem Repo-Präzedenzfall + benannter Requirement + Success-Criterion. Nur Pitfall 2 (Snapshot-Bump) bleibt bewusst offen. |

**Overall confidence:** HIGH — mit einer expliziten Ausnahme (Snapshot-Bump-Entscheidung, siehe Divergenz 2).

### Gaps to Address

- **Snapshot-Schema-Version 12→13 Bump ja/nein** (Divergenz 2 oben): in F4-discuss-phase pinnen. Beweislast beim „Nein"-Zweig = Straddling-Golden-Snapshot-Fixture.
- **F1-Denominator-Definition** (D-F1-01): „Wochen mit gültiger `working_hours`-Zeile absence-adjusted" (AVG-01 A-22-1) vs. simpler „contract-weeks in Jahr". In Phase 54 discuss-phase pinnen.
- **F2-Soll-Berechnung bei Mid-Week-Vertragswechsel** (D-F2-XX): Optionen A/B/C, Empfehlung: reuse `WorkingHoursService::get_working_hours_for_week`-Semantik.
- **F5-Stale-Vorschlag-Strategie**: Fingerprint (`state_fingerprint` auf Suggestion + 409 bei Divergenz) vs. Claim-on-Suggest (`rebooking_batch(kind=hr_suggestion, state=pending)` beansprucht die UNIQUE-Slot sofort, F4 skipt). Empfehlung: Claim-on-Suggest — löst zusätzlich Pitfall 6 „von der anderen Seite".
- **F4-Cron-Cadence + Uhrzeit**: ARCHITECTURE schlägt Montag 03:00 vor. Konflikt mit `PdfExportScheduler` (welche Uhrzeit?) in discuss-phase klären.
- **UNIQUE-Constraint-Shape**: `(sales_person_id, iso_year, iso_week)` (globale Wochen-Sperre für alle Kinds) vs. `(kind, sales_person_id, iso_year, iso_week)` (HR-Suggestion kann Auto-Cron ersetzen). Empfehlung: erstere, klarer Fehler-Fall + einfachere Idempotenz.
- **F5-Alert-Edge-Case bei balance ≈ 0**: F5 alertet ab „negativer Balance" — Threshold-Frage: `balance < 0` (strikt) oder `balance <= -0.5h` (Rundungs-Toleranz)? Pure-fn-Predicate-Test-Matrix in Phase 55 discuss-phase.

## Sources

### Primary (HIGH confidence)
- Repo-internal: `.planning/PROJECT.md` §„Current Milestone: v2.6" (F1..F5-Charter, Fat-Backend-Prinzip, Snapshot-Version=12).
- Repo-internal: `shifty-backend/CLAUDE.md` (Service-Tier-Konvention, Snapshot-Bump-Kontrakt, Docs-Freshness-Gate, `gen_service_impl!`-Muster, sqlx-prepare-Gate).
- Repo-internal: `.planning/milestones/v1.4-REQUIREMENTS.md` (committed_voluntary, CVC-05 no-bump precedent, CVC-06 no-leak precedent).
- Repo-internal: `.planning/milestones/v1.5-REQUIREMENTS.md` (Employee-Year-Report YV-01..03, STAT-01/02, A-22-1 Formel, UV-04/05 double-count Snapshot 9→10).
- Repo-internal: `.planning/milestones/v1.7-REQUIREMENTS.md` (HCFG-01/02/03 Stichtag-Präzedenz, HSNAP-01 Snapshot-Bump 10→11, HOL-03/VFA-02 Asymmetry-Regression-Guard).
- Repo-internal: `.planning/milestones/v1.8-REQUIREMENTS.md` (VAC-OFFSET-01 HR-Only-API-Level-Redaction, Snapshot 11→12).
- Repo-internal: `.planning/milestones/v2.1-REQUIREMENTS.md` (AVG-01 Read-Aggregate + HR-Gate + `is_dynamic`-Filter, WeekStatus TOCTOU-Präzedenz).
- Repo-internal: `.planning/milestones/v2.2-REQUIREMENTS.md` (EXP-01/02/03 `tokio_cron_scheduler` + Admin-Gate, RPT-01..03 Compute-Pattern, v2.3.1 Cron-Syntax-Hotfix 5→6-Feld).
- Repo-internal: `.planning/milestones/v2.4-REQUIREMENTS.md` (SHC-04 `shortday_slot_clipping_active_from` Stichtag mit Chain-A'/B/C/D-Matrix).
- Repo-internal: `.planning/milestones/v2.5-REQUIREMENTS.md` (VAA-01..04 D-53-02 cap-gated Formel, `find_by_iso_year`-Pattern, WOP-Follow-up #3 `_iso_year`-Helper, additive DTO + `#[serde(default)]`).
- Repo-internal: `service_impl/src/pdf_export_scheduler.rs` (Blueprint für `VoluntaryRebookingScheduler`).
- Repo-internal: `service_impl/src/reporting.rs` (Standort für neue pure fn `committed_voluntary_target_for_year`; Präzedenz-Helper).
- Repo-internal: `service_impl/src/extra_hours.rs` (`Authentication::Full` internal-caller pattern).
- Repo-internal: `service_impl/src/booking_information.rs` (BL-liest-BL-Präzedenz).
- Repo-internal: `service/src/extra_hours.rs` (`ExtraHoursCategory::VolunteerWork` + `::ExtraWork` als Pair-Ziel).
- Repo-internal: `service_impl/src/billing_period_report.rs::CURRENT_SNAPSHOT_SCHEMA_VERSION` (aktuell 12).
- Repo-internal: `Cargo.lock` (verified `tokio-cron-scheduler 0.15.1`, `sqlx 0.8.6`, `time 0.3.36`, `mockall 0.13`).

### Secondary (HIGH confidence, MEMORY.md-verankert)
- MEMORY `feedback_atomic_repoint_no_double_count.md`.
- MEMORY `feedback_stichtag_rollout_legacy_semantics.md`.
- MEMORY `reference_toggle_service_full_context_bypass.md`.
- MEMORY `feedback_docs_always_current_no_followup.md`.
- MEMORY `feedback_dioxus_proxy_for_new_backend_endpoints.md`.
- MEMORY `reference_dioxus_browser_test_date_inputs.md` + `reference_dioxus_browser_verify_reports.md`.
- MEMORY `feedback_verify_backend_roundtrip_e2e.md`.
- MEMORY `feedback_fat_backend_thin_client.md`.
- MEMORY `reference_sqlx_prepare_after_new_query.md`.

### Tertiary (LOW confidence)
- Keine Web-only-Behauptungen ohne Repo-Präzedenz.

---
*Research completed: 2026-07-06*
*Ready for roadmap: yes (mit zwei explizit offenen DISCUSS-Decisions: Snapshot-Bump + Phase-Dekomposition-Wahl)*
