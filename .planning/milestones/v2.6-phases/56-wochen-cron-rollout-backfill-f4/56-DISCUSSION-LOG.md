# Phase 56: Wochen-Cron + Rollout-Backfill (F4) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-25
**Phase:** 56-wochen-cron-rollout-backfill-f4
**Areas discussed:** Snapshot-Bump, Stale/Race, Cron-Timing, Locked-Woche, Backfill-Output, Chain-Audit

**Format note:** Auf Wunsch des Users als reine Chat-Liste präsentiert (kein
AskUserQuestion-Tool). User hat alle Empfehlungen in einem Rutsch bestätigt ("alle ✅").

---

## Snapshot-Bump 12→13 (D-F4-SNAPSHOT-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Bei 12 bleiben | Research-Mehrheit; balance-neutral; Stichtag-Guard schützt Historie; Beweislast = Straddling-Golden-Snapshot byte-identisch | ✓ |
| Bump auf 13 | PITFALLS-Sicht; neue ExtraHours-Quelle = Input-Set-Change; Migration + Docs-Sync | |
| Erst Fixture, dann entscheiden | Datengetrieben, verschiebt Commit in Execute-Phase | |

**User's choice:** Bei 12 bleiben (✅).
**Notes:** Fixture bleibt verbindliches Gate — kippt auf Bump, falls nicht byte-identisch (in CONTEXT als D-01-BEWEISLAST festgehalten).

---

## Stale-Vorschlag / Race (D-F4-STALE-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Claim-on-Suggest | pending belegt UNIQUE-Slot sofort, Cron skipt; einfachste Idempotenz, löst Pitfall 6 beidseitig | ✓ |
| Fingerprint + 409 | state_fingerprint, 409 bei Divergenz; mehr Kontrolle, mehr Komplexität | |

**User's choice:** Claim-on-Suggest (✅).
**Notes:** —

---

## Cron-Cadence + Uhrzeit (D-F4-CRON-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Montag 03:00, nach pdf_export | 6-Feld-Cron analog EXP; Boot am Ende main() nach pdf_export_scheduler.start() | ✓ |
| Andere Uhrzeit/Tag | freie Angabe | |

**User's choice:** Montag 03:00 (✅).
**Notes:** Konfliktvermeidung mit pdf_export-Uhrzeit als Plan-Detail an Claude delegiert.

---

## Locked-Vorwoche (D-F4-LOCKED-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Skip + Audit-Row (state='skipped_locked') | ARCHITECTURE Anti-Pattern 3; nachvollziehbar, idempotent | ✓ |
| Hartes Fehler-Log | keine Batch-Row, lauter | |

**User's choice:** Skip + Audit-Row (✅).
**Notes:** —

---

## Backfill-Dry-Run-Output (D-F4-BACKFILL-01)

| Option | Description | Selected |
|--------|-------------|----------|
| JSON pro Kandidat + Pagination | detaillierte Preview (Ist/Soll/Excess), mehrjahres-tauglich | ✓ |
| Kompakte Sammel-Summary | nur Aggregat | |

**User's choice:** JSON pro Kandidat + Pagination (✅).
**Notes:** —

---

## Chain-Audit-Matrix (D-F4-CHAIN-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Pro-Chain-Rekonstruktion analog HCFG-02/SHC-04 | explizite pre-Stichtag-Semantik + Golden-Snapshot-Regression pro Chain | ✓ |
| Abweichung | Chain streichen/ergänzen | |

**User's choice:** Pro-Chain-Rekonstruktion (✅).
**Notes:** Analyse-Deliverable für Research/Plan, keine echte Alternative — nur bestätigt.

---

## Claude's Discretion

- Exakte Boot-Minute des Cron (nur Konfliktvermeidung mit pdf_export).
- Interne ISO-Wochen-Arithmetik (v2.5-Helper wiederverwenden).
- `state`-Enum-Benennung (`skipped_locked` als Vorschlag).

## Deferred Ideas

None — Diskussion blieb innerhalb der Phase-56-Grenze.
