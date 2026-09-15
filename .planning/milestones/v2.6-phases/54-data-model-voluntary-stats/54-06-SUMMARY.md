---
phase: 54-data-model-voluntary-stats
plan: 06
subsystem: [docs, docs-freshness-gate]
tags: [docs, docs-freshness, feature-doc, F14, voluntary-rebooking, phase-54, wave-4]
status: complete
requirements: []
dependency_graph:
  requires:
    - 54-01 (data-model — rebooking_batch tables + extra_hours.source)
    - 54-02 (RebookingBatchService — Basic-Tier)
    - 54-03 (VoluntaryStatsService — BL-Tier + 4 pure fns)
    - 54-04 (REST endpoint /report/{id}/voluntary-stats + VoluntaryStatsTO)
    - 54-05 (Frontend voluntary_stats_row + i18n)
  provides:
    - docs/features/F14-rebooking.md (EN, new)
    - docs/features/F14-rebooking_de.md (DE, new)
    - Feature index (README.md + README_de.md) with F14 entry
    - Phase-54 notes in F07 (source-marker) + F08 (snapshot-12 non-bump)
    - Rebooking + VoluntaryStats in 02-service-tiers docs + runtime graph
    - Rebooking tables + source column in 03-data-model + ER diagram
  affects:
    - Milestone-close audit (no docs drift for Phase 54)
tech-stack:
  added: []
  patterns:
    - Same-commit rule for docs updates (CLAUDE.md Docs-Freshness-Gate + MEMORY feedback_docs_always_current_no_followup)
    - Sprach-Parallelitaet EN + DE (same H1/H2/table structure)
    - No snapshot-schema-version bump (F08 non-bump confirmation note)
    - Filename ASCII-safe (MEMORY feedback_no_umlauts_in_paths)
key-files:
  created:
    - docs/features/F14-rebooking.md
    - docs/features/F14-rebooking_de.md
  modified:
    - docs/features/README.md
    - docs/features/README_de.md
    - docs/features/F07-reporting-balance.md
    - docs/features/F07-reporting-balance_de.md
    - docs/features/F08-billing-period.md
    - docs/features/F08-billing-period_de.md
    - docs/architecture/02-service-tiers.md
    - docs/architecture/02-service-tiers_de.md
    - docs/architecture/03-data-model.md
    - docs/architecture/03-data-model_de.md
    - docs/architecture/diagrams/service-graph-runtime.mmd
    - docs/architecture/diagrams/db-schema-er.mmd
    - .planning/phases/54-data-model-voluntary-stats/54-VALIDATION.md
decisions:
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 12 (kein Bump in Phase 54); F08 dokumentiert die Non-Bump-Rationale explizit; 12->13 ist REB-AUTO-05 (Phase 56)."
  - "Ein einziges neues Feature-Cluster (F14) statt separater Docs pro Slice — F1..F5 werden als Feature-Slices IN F14 gefuehrt, Status je Slice (shipped vs. planned)."
  - "F14-Filename ASCII-only; Textinhalt darf Umlaute enthalten (analog zu bestehenden _de.md)."
  - "Service-Graph-Nodes ohne eingehende Kanten fuer RebookingBatchService (kein Konsument in Phase 54) — Kommentar erklaert Phase-55-Konsument."
  - "F07-Note liegt im Migrations-Abschnitt (nicht im Balance-Chain-Text) — bewahrt die 'Reader'-Perspektive und verlinkt zu F14."
metrics:
  duration: ~11 min
  completed: 2026-07-07
  tasks: 5
  files_created: 2
  files_modified: 12
  commits: 5
must_haves:
  truths_verified:
    - "docs/features/F14-rebooking.md und F14-rebooking_de.md existieren mit identischer H1/H2-Struktur (Purpose, Feature Slices, Marker-Filter Rule, Batch Structure, Services, REST, Related Features)."
    - "[D-54-DM-01] docs/architecture/03-data-model.md und _de.md dokumentieren beide neuen Tabellen inkl. globalem partial UNIQUE-Index."
    - "docs/architecture/02-service-tiers.md + _de.md listen RebookingBatchService (Basic) und VoluntaryStatsService (BL) mit korrekten Deps."
    - "Beide Diagramme (db-schema-er.mmd + service-graph-runtime.mmd) zeigen die neuen Nodes + Kanten."
  artifacts_verified:
    - "docs/features/F14-rebooking.md (NEU) — Purpose + F1..F5 Slices + Marker-Filter Rule D-54-DM-02 + Batch Structure + Services (Basic/BL) + REST + Related Features"
    - "docs/features/F14-rebooking_de.md (NEU) — strukturell identisch (H1/H2/Tabellen)"
    - "docs/features/README.md + README_de.md: F14-Zeile nach F13 ergaenzt"
    - "F07 EN+DE: narrative Note zu source-Marker im Migrationen-Abschnitt + F14-Cross-Ref"
    - "F08 EN+DE: 'Milestone v2.6 Phase 54 — non-bump confirmation' Absatz nach v12-Historie + F14-Cross-Ref"
    - "02-service-tiers EN+DE: RebookingBatchService in Basic-Liste, VoluntaryStatsService in BL-Liste"
    - "03-data-model EN+DE: rebooking_batch + rebooking_batch_entry + extra_hours.source + Toggle-Seed dokumentiert; Migration-History-Zeile 2026-07"
    - "service-graph-runtime.mmd: 2 neue Nodes + 3 Edges (VoluntaryStats -> ExtraHours/WorkingHours/SalesPerson); RebookingBatch ohne eingehende Kante (kein Konsument in Phase 54)"
    - "db-schema-er.mmd: 2 neue Entities (REBOOKING_BATCH + REBOOKING_BATCH_ENTRY), extra_hours.source Feld ergaenzt, 4 Relations (sales_person->batch, batch->entry, extra_hours <- entry out/in)"
---

# Phase 54 Plan 06: Docs-Freshness-Gate Summary

**Alle triggernden Docs-Files gemaess CLAUDE.md-Trigger-Tabelle EN + DE synchron aktualisiert: neues Feature F14 (Voluntary Rebooking), Notes in F07/F08 zu source-Marker und Snapshot-12-Non-Bump, Service-Tier-Registrierung + Runtime-Graph, Datenmodell + ER-Diagramm; kein snapshot-schema-version-Bump.**

## Was wurde erledigt

Phase 54 Plan 06 schliesst den Docs-Freshness-Gate fuer Phase 54 (Milestone v2.6 Baseline "Voluntary-Stats + Rebooking-Datenmodell") ab. 14 Docs-Files sind syncron mit dem Wave-1..3-Code:

### Task 1 — F14 Feature-Doku (EN + DE) + Index

- **`docs/features/F14-rebooking.md`** (NEU, 168 Zeilen): Struktur an F13-Vorlage angelehnt.
  - H2 "Purpose" — Business-Kontext (gedeckelte Mitarbeiter, Voluntary-Ausgleich).
  - H2 "Feature Slices" — F1..F5 mit Milestone/Phase/Status/Zweck-Tabelle.
  - H2 "Marker-Filter Rule ([D-54-DM-02])" — `manual`/`rebooking` + Reader/Audit-Regeln, VOL-ACCT-03 Balance-Neutralitaets-Garantie.
  - H2 "Batch Structure" — rebooking_batch + rebooking_batch_entry Column-Listen inkl. [D-54-DM-01] partial-UNIQUE-Index-Regel.
  - H2 "Services (Phase 54 baseline)" — Tabelle Basic vs. BL + Pure-Fns-Signaturen aus `service_impl::reporting`.
  - H2 "REST (Phase 54)" — Endpoint + DTO-Felder + Redaktions-Regel.
  - H2 "Related Features" — F04/F07/F08/F13 Cross-Refs.
- **`docs/features/F14-rebooking_de.md`** (NEU, strukturell identisch, deutsch).
- **`docs/features/README.md` + `README_de.md`**: F14-Zeile nach F13.

**Neue F14-H1/H2-Struktur (EN):**

```
# F14 — Voluntary Rebooking
## 1. Purpose
## 2. Feature Slices
## 3. Marker-Filter Rule ([D-54-DM-02])
## 4. Batch Structure
### `rebooking_batch` — parent row
### `rebooking_batch_entry` — per-slot payload
## 5. Services (Phase 54 baseline)
### Pure functions in `service_impl::reporting`
## 6. REST (Phase 54)
## 7. Related Features
```

### Task 2 — F07 + F08 Notes (EN + DE)

- **F07 (EN + DE)** — narrative Note im "Migrations that directly affect the reporting read"-Abschnitt zu `20260707000001_add-source-column-to-extra-hours.sql`: erklaert `source`-Marker + Reader-Impact (`source = 'manual'` ab Phase 55) + F14-Cross-Ref.
- **F08 (EN + DE)** — neuer Absatz "Milestone v2.6 Phase 54 — non-bump confirmation" nach dem v12-Bump-Historie-Absatz. Bestaetigt `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12`, erklaert warum kein Bump (kein neuer `value_type`, keine Berechnungsaenderung), verweist auf REB-AUTO-05 (Phase 56) fuer den 12 -> 13-Bump. Voluntary-Stats explizit als "live-computed HR-only read view, kein Snapshot" markiert.

### Task 3 — 02-service-tiers + Runtime-Graph

- **02-service-tiers.md + _de.md**: `RebookingBatchService` in Basic-Liste (alphabetisch nach `BookingService`), `VoluntaryStatsService` in BL-Liste (alphabetisch nach `BillingPeriodReportService`). Deps + F14-Cross-Ref inline.
- **service-graph-runtime.mmd Diff-Snippet:**

```mermaid
%% NEU in Basic-Cluster:
        RebookingBatch[Rebooking Batch Service<br/>Phase 54, v2.6]

%% NEU in Business-Cluster:
        VoluntaryStats[Voluntary Stats Service<br/>Phase 54, v2.6]

%% NEUE Kanten (am Ende):
    %% Voluntary Stats Service (Phase 54 — BL-Tier, HR-only F1/F2)
    VoluntaryStats --> ExtraHours
    VoluntaryStats --> WorkingHours
    VoluntaryStats --> SalesPerson

    %% Rebooking Batch Service (Phase 54 — Basic-Tier, no domain-service dep;
    %% first consumer arrives in Phase 55 as RebookingReconciliationService)
```

### Task 4 — 03-data-model + ER-Diagramm

- **03-data-model.md + _de.md**: neuer Abschnitt "Phase 54 (v2.6) — Rebooking data-model additions" mit rebooking_batch Column-Tabelle + rebooking_batch_entry Column-Tabelle + `extra_hours.source` Beschreibung + Toggle-Seed. [D-54-DM-01] und [D-54-DM-02] explizit dokumentiert. Migration-History-Tabelle um Zeile 2026-07 (v2.6 Phase 54) erweitert. Rebooking als Core-Aggregate ergaenzt.
- **db-schema-er.mmd**: `extra_hours` bekommt Feld `text source`. Zwei neue Entities `REBOOKING_BATCH` + `REBOOKING_BATCH_ENTRY` mit vollstaendigen Spalten. Vier neue Relations:

```mermaid
SALES_PERSON ||--o{ REBOOKING_BATCH : "reconciles-voluntary"
REBOOKING_BATCH ||--o{ REBOOKING_BATCH_ENTRY : "contains"
EXTRA_HOURS ||--o| REBOOKING_BATCH_ENTRY : "paired-out (nullable)"
EXTRA_HOURS ||--o| REBOOKING_BATCH_ENTRY : "paired-in  (nullable)"
```

### Task 5 — Docs-Freshness-Sanity + VALIDATION-Update

Sanity-grep-Ergebnisse (Ist-Werte):

| Command | Sollwert | Istwert |
| --- | --- | --- |
| `grep -rl 'F14' docs/features/ \| wc -l` | ≥ 8 | **8** |
| `grep -rl 'RebookingBatchService' docs/architecture/ \| wc -l` | ≥ 2 | **2** |
| `grep -rq 'voluntary-stats' docs/features/` | true | **true** |

54-VALIDATION.md Row `54-06-01` von ⬜ pending / ❌ W0 auf ✅ green / ✅ mit erweiterter automated-Query.

**Docs-Freshness-Gate — CLAUDE.md Trigger-Coverage:**

| Trigger (aus CLAUDE.md) | Behandelt in |
| --- | --- |
| `migrations/sqlite/*.sql` | Task 4 (03-data-model + db-schema-er.mmd) |
| `service/**/*.rs` (neues Trait) | Task 3 (02-service-tiers + service-graph-runtime.mmd) |
| `dao/**/*.rs` (neues Trait) | Task 4 (implizit via Tabellen-Doku) |
| `service_impl/billing_period_report.rs` | Nicht angefasst in Phase 54 → F08 Note dokumentiert Non-Bump (Task 2) |

## Verifikationsergebnisse

| Gate | Ergebnis |
| ---- | -------- |
| `cargo test --workspace` | **898 passed / 0 failed** |
| `cargo clippy --workspace -- -D warnings` | green |
| Sanity-grep F14 in docs/features/ | 8 Files (≥ 8) |
| Sanity-grep RebookingBatchService in docs/architecture/ | 2 Files (≥ 2) |
| Sanity-grep voluntary-stats in docs/features/ | true |
| Sprach-Parallelitaet EN/DE (H1/H2 identisch) | verified |
| ASCII-safe Filenames | verified (nur `_de` Suffix) |

## Commits (5 Tasks atomar)

| Commit | Task | Files |
| ------ | ---- | ----- |
| `d6f21e9` | Task 1: F14 EN + DE + Index | 4 (2 new, 2 modified) |
| `b1e0a10` | Task 2: F07 + F08 Notes | 4 modified |
| `1a67666` | Task 3: 02-service-tiers + Runtime-Graph | 3 modified |
| `328a31b` | Task 4: 03-data-model + ER-Diagramm | 3 modified |
| `5114662` | Task 5: VALIDATION-Row auf green | 1 modified |

## Deviations from Plan

Keine. Plan-Text 1:1 umgesetzt. Sanity-grep-Kommandos aus Task 5 wurden zusaetzlich in die 54-VALIDATION.md-Zeile als `automated`-Query eingebettet (leichte Praezisierung gegenueber der urspruenglichen `test -f`-Kette).

## Known Stubs

Keine. Alle Docs-Referenzen sind grep-verifiziert und die genannten Traits/Services/Tabellen existieren im Code.

## Threat Flags

Keine. Docs-only-Aenderungen — keine neue Netzwerk-/Auth-/Trust-Boundary-Surface.

## Naechste Schritte

Phase 54 ist damit vollstaendig (6/6 Plaene). Phase 55 (F3 HR-Suggest + F5 Approval-UI) und Phase 56 (F4 Auto-Cron + Snapshot-Bump 12->13) koennen auf dem baseline-Docs-Stand aufsetzen. Beide werden ihre eigenen F14-Note-Absaetze (F14 wird "growing living doc") und ggf. eigene F08-Bump-Zeile ergaenzen.

## Self-Check: PASSED

**Files exist:**
- `docs/features/F14-rebooking.md`: FOUND
- `docs/features/F14-rebooking_de.md`: FOUND
- `docs/features/README.md`: FOUND (F14 entry present)
- `docs/features/README_de.md`: FOUND (F14 entry present)
- `docs/features/F07-reporting-balance.md`: FOUND (F14 cross-ref present)
- `docs/features/F07-reporting-balance_de.md`: FOUND (F14 cross-ref present)
- `docs/features/F08-billing-period.md`: FOUND (Phase 54 non-bump note present)
- `docs/features/F08-billing-period_de.md`: FOUND (Phase 54 non-bump note present)
- `docs/architecture/02-service-tiers.md`: FOUND (RebookingBatchService + VoluntaryStatsService present)
- `docs/architecture/02-service-tiers_de.md`: FOUND
- `docs/architecture/03-data-model.md`: FOUND (Phase 54 section present)
- `docs/architecture/03-data-model_de.md`: FOUND
- `docs/architecture/diagrams/service-graph-runtime.mmd`: FOUND (new nodes + edges present)
- `docs/architecture/diagrams/db-schema-er.mmd`: FOUND (new entities + source column present)

**Commits exist:**
- `d6f21e9`: FOUND (Task 1)
- `b1e0a10`: FOUND (Task 2)
- `1a67666`: FOUND (Task 3)
- `328a31b`: FOUND (Task 4)
- `5114662`: FOUND (Task 5)

**Gates:**
- `cargo test --workspace`: 898 passed / 0 failed
- `cargo clippy --workspace -- -D warnings`: green
- Sanity-grep F14 ≥ 8: PASSED (8 files)
- Sanity-grep RebookingBatchService ≥ 2: PASSED (2 files)
- Sanity-grep voluntary-stats: PASSED (present)
