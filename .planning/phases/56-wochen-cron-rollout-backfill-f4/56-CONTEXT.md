# Phase 56: Wochen-Cron + Rollout-Backfill (F4) - Context

**Gathered:** 2026-07-25
**Status:** Ready for planning
**Ships as:** Regulärer v2.6-Release; endgültiger Milestone-Close nach dieser Phase (letzte Phase von v2.6).

<domain>
## Phase Boundary

Phase 56 liefert den automatischen Wochen-Trigger für die Freiwilligen-Umbuchung
sowie den einmaligen Rollout-Backfill — aufbauend auf dem in Phase 54 gelegten
Datenmodell und dem in Phase 55 gebauten `RebookingReconciliationService`.

**In-Scope (REB-AUTO-01..05):**

- **Wochen-Cron `VoluntaryRebookingScheduler`** (neu; 1:1-Spiegel von
  `PdfExportSchedulerImpl` v2.2). Läuft montags früh, verarbeitet die zuletzt
  abgeschlossene ISO-Woche. Für jeden `cap_planned_hours_to_expected=true`-
  SalesPerson mit `Ist_vorwoche > Soll_vorwoche + committed_voluntary_vorwoche`
  wird der Excess als Rebooking-Pair (`rebooking_batch(kind=auto_cron)`)
  persistiert.
- **Stichtag-Toggle-Wirkung** `voluntary_rebooking_auto_active_from` (Seed aus
  Phase 54, Default `None`): `None` → Cron im Skip-Modus (keine Regression);
  Datum → Rebooking nur für ISO-Wochen ≥ `active_from`.
- **Idempotenz** über UNIQUE-Constraint aus Phase 54 (`INSERT ... ON CONFLICT DO
  NOTHING`); zweiter Lauf über bereits verarbeitete Woche = no-op.
- **HR-Admin-gated Backfill-REST-Endpoint**
  `POST /admin/rebooking/backfill?from=YYYY-Www&dry_run={true|false}` +
  `service_impl::rebooking_reconciliation::{run_for_week, backfill_range}`.
- **Frontend-Admin-Card** (Settings) analog `pdf_export_config`: Stichtag-Toggle-
  Editor + Backfill-Button + Dry-Run-Vorschau.
- **Snapshot-Bump-Entscheidung** dokumentiert (siehe D-01) inkl. Straddling-
  Golden-Snapshot-Fixture als Beweislast.

**Out-of-Scope (schon in 54/55 erledigt oder bewusst ausgeklammert):**

- Datenmodell (`rebooking_batch`, `rebooking_batch_entry`, UNIQUE-Index, Marker-
  Spalte, Toggle-Seed) — Phase 54.
- Manuelle Umbuchung (F3) + HR-Alert-Modal (F5) — Phase 55.
- CLI-Backfill (`clap`) — verworfen zugunsten REST (REQUIREMENTS §Non-Goals).
- Migration alter (vor-v2.6-)ExtraHours in Batches — historische Rows bleiben
  unangetastet.

</domain>

<decisions>
## Implementation Decisions

### Snapshot-Schema-Version (D-F4-SNAPSHOT-01 — TOP-Entscheidung)
- **D-01:** **Bei Snapshot-Version 12 bleiben — KEIN Bump auf 13.** Begründung
  (FEATURES/ARCHITECTURE/STACK-Mehrheit): Rebooking ist balance-neutral (`−N` +
  `+N` in derselben Person/Woche), der Stichtag-Guard schützt die Historie (F4
  läuft nur für Wochen ≥ `active_from`), und ein vorsorglicher Bump ohne neuen
  `value_type` würde jede historische Snapshot-Validierung ohne inhaltlichen
  Grund invalidieren.
- **D-01-BEWEISLAST (verbindlich):** Der „Nein"-Zweig MUSS durch eine
  **Straddling-Golden-Snapshot-Fixture** abgesichert werden (Muster v2.5 Phase 52
  WOP-03, 8 Fixtures): eine Billing-Period wird strikt VOR, strikt NACH und
  TRAVERSIEREND über `active_from` durchgerechnet und muss byte-identisch bleiben.
  **Wenn diese Fixture NICHT byte-identisch liefert → Entscheidung kippt auf
  Bump 12→13** (`CURRENT_SNAPSHOT_SCHEMA_VERSION` in
  `service_impl/src/billing_period_report.rs` + Migration + `docs/features/
  F08-billing-period.md` + `_de.md` synchron). Die Fixture ist damit das
  Gate, nicht die Meinung.

### Race F5-pending ↔ F4-Cron (D-F4-STALE-01)
- **D-02:** **Claim-on-Suggest.** Eine pending HR-Suggestion belegt den UNIQUE-
  Slot sofort (`rebooking_batch(kind=hr_suggestion, state=pending)`); der Cron
  skipt denselben (SalesPerson, ISO-Woche)-Slot via `ON CONFLICT DO NOTHING`.
  Einfachste Idempotenz, löst Pitfall 6 „von beiden Seiten". Kein separater
  Fingerprint-/409-Mechanismus.

### Cron-Cadence + Boot-Sequence (D-F4-CRON-01)
- **D-03:** **Montag 03:00, 6-Feld-Cron-Syntax** (analog v2.2 EXP, zwingend nach
  v2.3.1-Hotfix 6- statt 5-Feld). Verarbeitet die zuletzt abgeschlossene ISO-
  Woche via v2.5-`_iso_year`-Helper (Pitfall 5). **Boot am Ende von `main()`
  nach `pdf_export_scheduler.start()`.** Falls `pdf_export` selbst 03:00 nutzt:
  im Plan auf konfliktfreie Minute/Stunde ausweichen (Plan-Detail, keine
  Neu-Entscheidung nötig).

### Locked-Vorwoche (D-F4-LOCKED-01)
- **D-04:** **Skip + Audit-Row.** Bei `WeekStatus=Locked` überspringt der Cron
  die Woche und persistiert `rebooking_batch(state='skipped_locked')` als Audit-
  Spur (ARCHITECTURE Anti-Pattern 3). Kein Crash, nachvollziehbar, idempotent —
  kein hartes Fehler-Log. `state`-Enum entsprechend erweitern.

### Backfill-Dry-Run-Output (D-F4-BACKFILL-01)
- **D-05:** **JSON pro Batch-Kandidat mit Preview-Werten** (Ist / Soll /
  committed_voluntary / Excess pro SalesPerson+Woche) **+ Pagination** für
  mehrjährige Backfills. `dry_run=true` → kein DB-Write; `dry_run=false` →
  persistiert `rebooking_batch(kind=auto_cron_backfill)`. HR-Admin-Gate
  durchgesetzt. TOs: `BackfillRequestTO`, `BackfillPreviewTO`.

### Chain-Audit-Matrix (D-F4-CHAIN-01)
- **D-06:** **Pro-Chain-Rekonstruktion analog HCFG-02 (v1.7) / SHC-04 (v2.4).**
  Jede Konsumkette, die `extra_hours` liest — Balance (`reporting.rs`), F1-Ist,
  F2-Soll-Nutzung, F4-Self-Guard, `billing_period.value_type`,
  `booking_information::get_weekly_summary` — bekommt eine explizite pre-Stichtag-
  Legacy-Semantik bei `active_from=None` + Golden-Snapshot-Regression. Property-
  Test: `active_from=None` → `cargo test --workspace` byte-identisch mit v2.5-
  Baseline (drei Neutralitäts-Property-Tests: `balance_before==balance_after`,
  `F1_Ist_before==after`, `F2_Soll_before==after`). Analyse-Deliverable für
  Research/Plan, keine Alternative.

### Claude's Discretion
- Exakte Boot-Minute des Cron (nur Konfliktvermeidung mit `pdf_export`).
- Interne Struktur der ISO-Wochen-Arithmetik (v2.5-Helper wiederverwenden).
- `state`-Enum-Benennung (`skipped_locked` als Vorschlag) — solange Audit-
  Semantik + Idempotenz gewahrt.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase-Scope + Requirements
- `.planning/ROADMAP.md` §"Phase 56: Wochen-Cron + Rollout-Backfill (F4)" — Goal,
  Success Criteria, Discuss-Points, Backend-/Frontend-Pfade, Pitfalls-Match.
- `.planning/REQUIREMENTS.md` §"Automatische Wochen-Umbuchung (REB-AUTO)" —
  REB-AUTO-01..05 + Non-Goals (CLI-Backfill, Migration alter ExtraHours).
- `.planning/research/SUMMARY.md` §"Divergenz 2: Snapshot-Schema-Version 12→13" —
  volle Argumentation PRO/CONTRA Bump (Grundlage für D-01).
- `.planning/research/PITFALLS.md` — Pitfalls 2,3,4,5,7,13,16 (F4-relevant).
- `.planning/phases/55-manual-rebooking-hr-alert/55-CONTEXT.md` — F3/F5-Datenfluss
  + `RebookingReconciliationService`-Design, auf dem F4 aufsetzt.

### Snapshot-Contract
- `shifty-backend/CLAUDE.md` §"Billing Period Snapshot Schema Versioning" — die
  Klausel „Change the input set the computation reads from" (Kern von D-01).
- `service_impl/src/billing_period_report.rs::CURRENT_SNAPSHOT_SCHEMA_VERSION`
  (aktuell 12).
- `docs/features/F08-billing-period.md` + `_de.md` — nur bei Bump-Kippfall (D-01)
  zu aktualisieren.

### Präzedenzen (Muster zum Kopieren)
- v2.2 Phase 48 EXP-01/02/03 — `PdfExportSchedulerImpl` + `tokio_cron_scheduler`
  0.15.1 + Single-Row-Toggle-Config-Tabelle: 1:1-Blueprint für D-03.
- v2.3.1 Hotfix — 6-Feld-Cron-Syntax (Pitfall 16).
- v1.7 Phase 25 HCFG-02 `holiday_auto_credit_active_from` + v2.4 Phase 51 SHC-04
  `shortday_slot_clipping_active_from` — Stichtag-Toggle + Chain-Audit-Matrix (D-06).
- v2.5 Phase 52 Follow-up #3 `_iso_year`-Bulk-Methoden — ISO-Wochen-Arithmetik (D-03/Pitfall 5).
- v2.5 Phase 52 WOP-03 Golden-Snapshot-Fixture-Muster (8 Fixtures) — Straddling-
  Fixture für D-01-Beweislast.
- v2.1 Phase 40 WST TOCTOU + WeekLocked-Behandlung — Cron-vs-WeekStatus (D-04).

### MEMORY
- `feedback_stichtag_rollout_legacy_semantics.md` — pro-Chain-Legacy-Semantik-
  Rekonstruktion (D-06).
- `feedback_dioxus_proxy_for_new_backend_endpoints.md` — `Dioxus.toml`-Proxy für
  neuen `/admin/rebooking/*`-Endpoint.

### Docs-Freshness-Gate (bei Trigger-Datei-Änderung mit-aktualisieren)
- `docs/features/F14-rebooking.md` + `_de.md` — F4-Cron + Backfill ergänzen.
- `docs/architecture/02-service-tiers.md` + `diagrams/service-graph-runtime.mmd` —
  neuer Scheduler-Service (main.rs-DI-Änderung).
- `docs/features/F07-reporting-balance.md` + `docs/domain/time-accounting.md` —
  nur falls Balance-Chain-Semantik berührt (D-06).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `service_impl/src/pdf_export_scheduler.rs` + `service/src/…` — 1:1-Spiegel für
  `voluntary_rebooking_scheduler.rs` (Service-Trait + Impl + Boot).
- `service_impl/src/rebooking_reconciliation.rs` (aus Phase 55) — vorhandener
  Doppel-Eintrag-Write-Path; F4 ergänzt nur `run_for_week` + `backfill_range`
  (single write-path für alle Trigger, Pitfall 15).
- v2.5-`_iso_year`-Bulk-Helper — „Vorwoche"-Berechnung.
- UNIQUE-Partial-Index `(sales_person_id, iso_year, iso_week)` (Phase 54) —
  Idempotenz-Backbone.
- `pdf_export_config`-Admin-Card (FE) — Vorlage für Rebooking-Admin-Card.

### Established Patterns
- `gen_service_impl!`-Makro + Service-Tier-Konvention: Scheduler ist Business-
  Logic-Tier (konsumiert `RebookingReconciliationService`), Boot NACH Basic-Tier.
- Stichtag-Toggle-Muster (HCFG-02 / SHC-04): Single-Row-Config + pro-Chain-Guard.
- `[[web.proxy]]` in `Dioxus.toml` für neue Backend-Routes.

### Integration Points
- `shifty_bin/src/main.rs`: DI-Wiring `VoluntaryRebookingSchedulerImpl` + Boot am
  Ende von `main()` nach `pdf_export_scheduler.start()`.
- `rest/src/rebooking.rs` (aus 55): neuer `POST /admin/rebooking/backfill`.
- `rest-types/src/rebooking.rs` (aus 55): `BackfillRequestTO`, `BackfillPreviewTO`.
- `shifty-dioxus/src/loader.rs` + `src/page/settings.rs`: Backfill-Loader + Card.

</code_context>

<specifics>
## Specific Ideas

- Cron-Blueprint strikt 1:1 aus `pdf_export_scheduler.rs` spiegeln — keine
  Neuerfindung des Scheduler-Bootstraps.
- Straddling-Golden-Snapshot-Fixture ist das harte Gate für D-01: byte-identisch
  → 12 bleibt, sonst Bump. Diese Fixture explizit als eigener Test-Task planen.

</specifics>

<deferred>
## Deferred Ideas

None — Diskussion blieb innerhalb der Phase-56-Grenze. Alle 6 Discuss-Points
(D-F4-SNAPSHOT/STALE/CRON/LOCKED/BACKFILL/CHAIN) sind entschieden.

</deferred>

---

*Phase: 56-wochen-cron-rollout-backfill-f4*
*Context gathered: 2026-07-25*
