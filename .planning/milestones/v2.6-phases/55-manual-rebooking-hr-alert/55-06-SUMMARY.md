---
phase: 55-manual-rebooking-hr-alert
plan: 06
subsystem: docs
tags: [docs-freshness, rebooking, reporting, service-tiers, runtime-graph, f14, f07, en-de-sync]

# Dependency graph
requires:
  - phase: 55-02
    provides: "source==Manual-Neutralitäts-Filter (4 Sites), RebookingReconciliationService (BL), single Pair-write-path, RebookingSuggestion (backend-computed IST/DANN)"
  - phase: 55-03
    provides: "4 REST-Routes, has_pending_rebooking-Alert-Flag, main.rs-DI (RebookingReconciliation + Reporting-->RebookingBatch), RebookingSuggestionTO ohne voluntary_soll/delta"
  - phase: 55-01
    provides: "state-conditional UPDATE (pending→approved|rejected, rows_affected==1), create_pending_suggestion Claim-on-Suggest"
provides:
  - "F14-rebooking.{md,_de.md}: F3/F5-Sektionen (§8) synchron zum Phase-55-Code"
  - "F07-reporting-balance.{md,_de.md}: source==Manual-Neutralitäts-Filter-Note (§2.6)"
  - "02-service-tiers.{md,_de.md}: RebookingReconciliationService als BL-Tier"
  - "service-graph-runtime.mmd: RebookingReconciliation-Knoten + 4 Kanten + Reporting-->RebookingBatch"
affects: [F14-rebooking, F07-reporting-balance, architecture-docs]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Docs-Freshness-Gate nachgezogen: EN+DE synchron, jede Behauptung code-gedeckt (kein Follow-up)"

key-files:
  created:
    - ".planning/phases/55-manual-rebooking-hr-alert/55-06-SUMMARY.md"
  modified:
    - "docs/features/F14-rebooking.md"
    - "docs/features/F14-rebooking_de.md"
    - "docs/features/F07-reporting-balance.md"
    - "docs/features/F07-reporting-balance_de.md"
    - "docs/architecture/02-service-tiers.md"
    - "docs/architecture/02-service-tiers_de.md"
    - "docs/architecture/diagrams/service-graph-runtime.mmd"

key-decisions:
  - "F14 §8 dokumentiert die ECHT implementierte Semantik (nicht die Plan-Artefakte): RebookingSuggestionTO trägt NUR backend-computed Felder (balance/-_after, voluntary_ist/-_after, proposed_hours) — kein voluntary_soll/voluntary_delta (existiert in der 55-02-Domain nicht). Die optionale GET /rebooking-suggestions-Liste wurde als NICHT-implementiert dokumentiert (55-03-Deviation)."
  - "Neutralitäts-Filter als 'keep Manual / exclude Rebooking' an genau 4 reporting.rs-Sites (859/1192/1728/2161) dokumentiert; kein Snapshot-Bump (bleibt 12) — Prohibition strikt gehalten."
  - "Zusätzlich zur Plan-Vorgabe die code-gedeckte Reporting-->RebookingBatch-Kante in den Graph aufgenommen (55-03 verdrahtete sie in main.rs; der Graph-Header erklärt die Doc-Liste als kanonisch → Drift vermieden)."

# Metrics
metrics:
  duration: "~15 min"
  completed: 2026-07-23
  tasks: 2
  files: 7

status: complete
---

# Phase 55 Plan 06: Docs-Freshness-Sync (F14 F3/F5 + F07-Filter + BL-Service-Tier) Summary

**Die technische Doku wurde synchron zum Phase-55-Code nachgezogen (Docs-Freshness-Gate, hart): F14 bekommt seine F3/F5-Sektionen (§8), F07 die source==Manual-Neutralitäts-Filter-Note (§2.6), die Service-Tier-Doku + der Runtime-Graph führen den neuen RebookingReconciliationService — EN und DE synchron, jede Behauptung gegen den Code verifiziert.**

## Accomplishments

- **F14-rebooking.{md,_de.md} — neue §8 (F3/F5-Write-Seite):**
  - Single Pair-write-path `rebook(direction, hours, week)` (private Impl-Methode, geteilt von F3 `rebook_manual` + F5 `approve`, Pitfall 15).
  - F3 `rebook_manual`: One-Shot in EINER Tx (`kind=Manual`, `state=Approved`, Pair + Entry); belegter UNIQUE-Slot → 409.
  - F5 Claim-on-Suggest (Modell B): `suggest` claimt via `create_pending_suggestion` einen `HrSuggestion`/`Pending`-Batch (idempotent; entschiedener Slot → 409), liefert backend-computed IST/DANN + `proposed_hours = min(|balance|, voluntary_ist)`; `approve` transitioniert state ZUERST (state-conditional UPDATE `WHERE … state='pending'`, rows_affected==1), dann Pair; `reject` nur state. Kein zweiter Batch bei approve.
  - Claim-on-Suggest/Phase-56-Cron-Konsequenz (`state=pending` blockiert Cron-Slot).
  - HR-Alert-Flag `has_pending_rebooking` (Predicate `balance<=-0.5 ∧ voluntary_ist>0 ∧ cap`, grenzgetestet -0.49/-0.50/-0.51; Banner-Regel: Predicate UND kein entschiedener Batch; Alert erlischt nach approve UND reject; kein Undo).
  - 4 REST-Routes (POST /rebooking/manual, POST /rebooking-suggestions, POST .../{id}/approve|reject); optionale GET-Liste explizit als NICHT-implementiert vermerkt.
  - Header/Status/Feature-Slices/Marker-Filter-Regel auf Phase-55-aktiv aktualisiert; **kein Snapshot-Bump** (bleibt 12).
- **F07-reporting-balance.{md,_de.md} — §2.6 Rebooking-Neutralitäts-Filter:** `source == ExtraHoursSource::Manual` (keep Manual / exclude Rebooking) an genau 4 Aggregations-Eingängen (`reporting.rs:859/1192/1728/2161`) → ein Gegenpaar `(−h,+h)` ist balance/ist/soll/delta-neutral (VOL-ACCT-03, D-55-A1); Property-Tests referenziert; Reader-Impact-Migration-Note aktualisiert; kein Bump.
- **02-service-tiers.{md,_de.md}:** `RebookingReconciliationService` in der BL-Tier-Liste (Deps ExtraHours/RebookingBatch/Reporting/VoluntaryStats + Permission/Tx; DI-Reihenfolge nach allen Basic-Services; HR-gated; kein Zyklus).
- **service-graph-runtime.mmd:** `RebookingReconciliation`-Knoten (Business-Subgraph) + 4 Domain-Kanten (ExtraHours/RebookingBatch/Reporting/VoluntaryStats) + die code-gedeckte `Reporting-->RebookingBatch`-Kante (55-03-main.rs-DI-Trigger).

## Task Commits

1. **Task 1: F14 F3/F5-Sektionen (EN+DE) + F07-Filter-Note (EN+DE)** — `629a6d50` (docs)
2. **Task 2: RebookingReconciliationService als BL-Tier + Runtime-Graph** — `9477fd1c` (docs)

## Files Modified

- `docs/features/F14-rebooking.md` / `_de.md` — Header/Status/Slices/§3-Marker-Filter aktualisiert + neue §8 (F3/F5)
- `docs/features/F07-reporting-balance.md` / `_de.md` — §2.6 Neutralitäts-Filter-Bullet + Reader-Impact-Note
- `docs/architecture/02-service-tiers.md` / `_de.md` — RebookingReconciliationService in BL-Liste
- `docs/architecture/diagrams/service-graph-runtime.mmd` — Knoten + 5 Kanten

## Code-Verifikation (Docs-Freshness-Gate: keine Behauptung ohne Code-Deckung)

Jede dokumentierte Behauptung wurde gegen den Code geprüft:

- 4 Filter-Sites `eh.source == ExtraHoursSource::Manual`: `service_impl/src/reporting.rs:859, 1192, 1728, 2161` (verifiziert via grep).
- `ExtraHoursSource`-Enum hat exakt 2 Varianten (Manual, Rebooking): `service/src/extra_hours.rs:60-68`.
- Trait `RebookingReconciliationService` (rebook_manual/suggest/approve/reject) + `RebookingSuggestion` (nur balance/-_after, voluntary_ist/-_after, proposed_hours — KEIN soll/delta): `service/src/rebooking_reconciliation.rs`.
- state-conditional UPDATE `WHERE id = ? AND state = 'pending' AND deleted IS NULL` + `rows_affected() != 1`-Guard: `dao_impl_sqlite/src/rebooking_batch.rs:347-366`.
- 4 REST-Routes (/manual, "", /{id}/approve, /{id}/reject) unter Nests /rebooking + /rebooking-suggestions: `rest/src/rebooking.rs` + `rest/src/lib.rs:724-727`.
- Impl-Deps ExtraHours/RebookingBatch/Reporting/VoluntaryStats/Permission/Tx: `service_impl/src/rebooking_reconciliation.rs:40-46`.
- DI-Reihenfolge (extra_hours 1113 → rebooking_batch 1164 → reporting 1210 → voluntary_stats 1236 → rebooking_reconciliation 1256, zuletzt) + Reporting-Dep auf rebooking_batch: `shifty_bin/src/main.rs`.

## Deviations from Plan

**1. [Rule 2 — code-gedeckte Ergänzung] Reporting-->RebookingBatch-Kante zusätzlich in den Graph aufgenommen**
- **Gefunden bei:** Task 2 (Verifikation der main.rs-DI).
- **Issue:** Der Plan listet für den Graph nur den RebookingReconciliation-Knoten + 4 Kanten. 55-03 verdrahtete aber auch `Reporting --> RebookingBatch` in main.rs (für den has_pending_rebooking-Lookup) — der Graph hätte diese code-existierende Kante sonst weiterhin verschwiegen (Drift).
- **Fix:** Die code-gedeckte `Reporting --> RebookingBatch`-Kante ergänzt. Der Graph-Header erklärt die Doc-Liste als kanonisch; die Ergänzung hält sie code-synchron.
- **Datei:** `docs/architecture/diagrams/service-graph-runtime.mmd`
- **Commit:** `9477fd1c`

Ansonsten: Plan exakt wie geschrieben ausgeführt. Prohibitions strikt gehalten — kein Snapshot-Bump-Eintrag in F08/F14 (bleibt 12), kein EN-only/DE-only-Update (beide Sprachen synchron), keine Behauptung ohne Code-Deckung.

## Known Stubs

Keine. Dieser Plan ändert ausschließlich Dokumentation.

## Threat Flags

Keine neue Security-Surface — reine Doku-Synchronisation.

---
*Phase: 55-manual-rebooking-hr-alert*
*Completed: 2026-07-23*

## Self-Check: PASSED
- Alle 7 modifizierten Doku-Dateien auf Disk vorhanden + committet.
- Task-Commits im Git-Log (629a6d50, 9477fd1c).
- Automated verify beider Tasks grün (grep -l pending/source; grep -c RebookingReconciliationService==2, RebookingReconciliation==6).
- EN/DE synchron (F14, F07, 02-service-tiers je Paar aktualisiert).
