# Phase 4: Migration & Cutover - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-03
**Phase:** 04-migration-cutover
**Areas discussed:** Migrations-Heuristik, Cutover-Gate-Mechanik, REST-Strategie + Schicksal alter ExtraHours, Carryover-Refresh-Scope + Atomic-Tx-Boundaries

---

## Area 1 — Migrations-Heuristik

### Q1: Werktage-Definition

| Option | Description | Selected |
|--------|-------------|----------|
| Per-Vertrag aus EmployeeWorkDetails.workdays (Recommended) | Per Tag und MA werden die zum Datum gültigen workdays-Bool-Maske gelesen. Konsistent mit Phase-2-derive_hours_for_range. | ✓ |
| Mo-Fr fix (alle Mitarbeiter) | Pauschale Annahme. Falsch für Mitarbeiter mit anderen Modellen. | |
| Production-Data-Profile-driven (Decision deferred) | Verschiebt Decision auf nach SC-1-Profile. | |

**User's choice:** Per-Vertrag aus EmployeeWorkDetails.workdays
**Notes:** D-Phase4-01.

### Q2: Cluster-Algorithmus + Bruchstunden

| Option | Description | Selected |
|--------|-------------|----------|
| Strict: nur volle Tage (amount == contract_hours_at(day)), Rest → Quarantäne (Recommended) | Garantiert Gate-Identität, viel manuelle Nacharbeit, kein Daten-Verlust. | ✓ |
| Tolerant: Range mergen unabhängig vom amount | Gate fail't oft, höhere Migrations-Quote. | |
| Strict + Auto-Quarantine-für-Wochenende-only | Bruchstunden trotzdem migriert. Kompromiss. | |

**User's choice:** Strict
**Notes:** D-Phase4-02. Vertragswechsel-mid-week implizit gelöst (contract_hours_at-Check spaltet Cluster automatisch).

### Q3: Quarantäne-Mechanismus

| Option | Description | Selected |
|--------|-------------|----------|
| Eigene Tabelle `absence_migration_quarantine` (Recommended) | Saubere Separation, eigene Indizes, klare HR-Workflow-Surface. | ✓ |
| Logische Markierung via update_process-String | Kein Schema-Bruch. Risk: Marker überschrieben. | |
| Nur Diff-Report, keine Schema-Persistenz | Kein DB-State; nicht idempotent rekonstruierbar. | |

**User's choice:** Eigene Tabelle
**Notes:** D-Phase4-03.

### Q4: Idempotenz-Key

| Option | Description | Selected |
|--------|-------------|----------|
| Mapping-Tabelle `absence_period_migration_source` (Recommended) | extra_hours_id PK, audit-trail, präzises Rollback möglich. | ✓ |
| Deterministischer logical_id-Hash auf absence_period | Kein neues Schema; Risk Hash-Kollision. | |
| Pre-Check Range-Lookup pro Cluster | Kein Audit-Trail; Risk False-Positive bei manuellen HR-Ranges. | |

**User's choice:** Mapping-Tabelle
**Notes:** D-Phase4-04. Codebase-Befund: extra_hours hat KEIN logical_id-Feld; Idempotenz-Key ist extra_hours.id.

---

## Area 2 — Cutover-Gate-Mechanik

### Q1: Gate-Granularität

| Option | Description | Selected |
|--------|-------------|----------|
| Pro (sp, kategorie, jahr) (Recommended) | Granular genug für Drift-Lokalisierung, grob genug für Rounding-Absorption. Konsistent mit Carryover-Logik. | ✓ |
| Pro (sp, kategorie) global über alle Jahre | Risk Drift-Maskierung über Jahre. | |
| Pro (sp, kategorie, billing_period) | Sehr granular; Risk Coverage-Fragmentierung. | |
| Pro (sp, kategorie, einzelner_extra_hours_eintrag) | Maximal granular; überlappt mit Quarantäne-Logik. | |

**User's choice:** Pro (sp, kategorie, jahr)
**Notes:** D-Phase4-05. Toleranz absolut < 0.01h.

### Q2: Diff-Report-Format

| Option | Description | Selected |
|--------|-------------|----------|
| JSON in `.planning/migration-backup/cutover-gate-{ts}.json` + tracing-Logs (Recommended) | CI-friendly, archivierbar, jj-committable. | ✓ |
| Strukturierter Log nur (kein File) | Nicht archivierbar; Long-Log-greppen schwer. | |
| Beides: JSON + neue DB-Tabellen | Maximal sauber; mehr Schema-Surface. | |

**User's choice:** JSON + tracing-Logs
**Notes:** D-Phase4-06.

### Q3: Run-Modes (CLI vs Server)

| Option | Description | Selected |
|--------|-------------|----------|
| Beides: separater Pre-Flight-Mode + atomarer Commit-Mode (initially Recommended) | Bin-Tool, iteratives Cleanup. | |
| Nur atomarer Run | Bin-Tool, keine Iteration. | |
| Gate als REST-Endpunkt + atomarer Cutover als Bin-Tool | REST + CLI gemischt. | |

**User's choice:** "Wieso sollte das Binary neu aufgerufen werden? Das muss alles funktionieren während der Server läuft."
**Notes:** **User-Korrektur.** Cutover läuft im laufenden Server, kein Bin-Restart. Verschiebt Surface auf REST. Folge-Frage Q3b.

### Q3b: REST-Surface-Schnitt

| Option | Description | Selected |
|--------|-------------|----------|
| Zwei Endpunkte: gate-dry-run + commit (Recommended) | Klare Permission-Trennung, distinct utoipa-Routes. | ✓ |
| Ein Endpunkt mit dry_run-Flag | Schmaler, dafür gefährlicher | |
| Nur Commit; Pre-Flight als Background-Job | Kein on-demand Trigger. | |

**User's choice:** Zwei Endpunkte
**Notes:** D-Phase4-07. Permission: HR (dry-run) und neues `cutover_admin` (commit).

### Q4: Drift-Schutz zwischen Pre-Flight und Commit

| Option | Description | Selected |
|--------|-------------|----------|
| Commit fährt Migration + Gate erneut auf aktuellem State (Recommended) | Pre-Flight ist Komfort; Commit ist Wahrheit. | ✓ |
| Token-Pessimistic-Lock | Komplexer, Lock-Workflow umgehbar. | |
| Lock-Modus: Pre-Flight setzt Block-Flag | Risk vergessener Lock. | |

**User's choice:** Commit fährt Re-Run
**Notes:** D-Phase4-08.

---

## Area 3 — REST-Strategie + Schicksal alter ExtraHours

### Q1: /extra-hours POST-Verhalten nach Cutover

| Option | Description | Selected |
|--------|-------------|----------|
| Hard-403 nur für POST mit Vacation/Sick/UnpaidLeave; DELETE+GET bleiben (Recommended) | Klarste Trennung. | ✓ |
| Read-Compat-Shim: POST intern auf /absence-period umgeleitet | Heuristik wird Read+Write-Surface. | |
| Soft-Deprecate per HTTP-Header | Doppel-Eintragung bleibt möglich. | |

**User's choice:** Hard-403 + flag-gated
**Notes:** **User-Klarstellung danach:** "Backend muss BEIDES unterstützen — alles vor und nach Migration. Während des Betriebs. 403 erst wenn User auf AbsencePeriod umgestellt + Migration getriggert haben." → D-Phase4-09 revidiert: 403-Behavior ist FLAG-GATED. Vor Cutover: 100% unverändert. Nach Cutover: 403 für die 3 Kategorien.

### Q2: Schicksal alter extra_hours-Rows nach Cutover

| Option | Description | Selected |
|--------|-------------|----------|
| Soft-delete in-place + update_process-Marker (Recommended) | Konsistent mit soft-delete-Pattern; reverse-Migration trivial. | ✓ |
| Live behalten | HR-Verwirrung möglich. | |
| Move zu Archiv-Tabelle | 2-step-Operation; höhere Komplexität. | |

**User's choice:** Soft-delete in-place
**Notes:** D-Phase4-10. Soft-Delete passiert INNERHALB der atomaren Cutover-Tx (analog zur Flag-Gating-Klarstellung in Q1). Quarantänierte Rows bleiben aktiv.

### Q3: OpenAPI-Snapshot-Test (SC-6)

| Option | Description | Selected |
|--------|-------------|----------|
| Insta-Snapshot über ApiDoc::openapi() (Recommended) | Industriestandard, voller Coverage. | ✓ |
| Manuelle Pin-Map analog Phase-2 | Kein Crate-Dep; Pflegeaufwand. | |
| Beides: Insta + Pin-Map | Doppelte Pflege. | |

**User's choice:** Insta-Snapshot
**Notes:** D-Phase4-11. **Vorab User-Frage** "Ich versteh nicht — geht es darum den Code zu testen?" → Erklärt: utoipa generiert OpenAPI-JSON; Snapshot vergleicht gegen gefrorenes Pin-File; Diff = Test rot; Mensch akzeptiert via cargo insta review. User dann gewählt.

---

## Area 4 — Carryover-Refresh-Scope + Atomic-Tx-Boundaries

### Q1: Refresh-Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Alle (sp, year)-Tupel im Gate-Scope (Recommended) | Präzise, performant, deckt jeden betroffenen Snapshot. | ✓ |
| Alle (sp, year)-Tupel mit existierenden Carryover-Rows | Broader; unnötiger Refresh. | |
| Nur laufendes Jahr | Bricht SC-2 (per-Jahr-Identität). | |

**User's choice:** Gate-Scope
**Notes:** D-Phase4-12.

### Q2: Pre-Cutover-Backup für Carryover

| Option | Description | Selected |
|--------|-------------|----------|
| Separate Tabelle `employee_yearly_carryover_pre_cutover_backup` (Recommended) | Innerhalb Tx sicher, atomarer Restore. | ✓ |
| JSON-Dump in .planning/migration-backup/ | File-IO-Risiko innerhalb Tx. | |
| Beides: Backup-Tabelle + JSON | Doppelter Aufwand. | |
| Kein dediziertes Backup (Operator macht jj/git-Snapshot) | Risk vergessen; nicht dokumentiert. | |

**User's choice:** Backup-Tabelle
**Notes:** D-Phase4-13.

### Q3: Atomare Tx-Grenzen

| Option | Description | Selected |
|--------|-------------|----------|
| Eine einzige SQLite-Tx über alles (Recommended) | Standard-Pattern; atomar by definition. | ✓ |
| Mehrere Tx mit Compensation-Steps | Bricht Atomarität; mehr Failure-Modes. | |
| Eine Tx + SAVEPOINTS pro Phase | Overkill bei Full-Rollback-Verhalten. | |

**User's choice:** Eine Tx über alles
**Notes:** D-Phase4-14. Via `TransactionDao::use_transaction`-Pattern.

---

## Closing — Hygiene + Scope-Confirm

### Q-Closing-1: Hygiene-Items aus Phase-3-deferred mitnehmen?

| Option | Description | Selected |
|--------|-------------|----------|
| Mitnehmen als Wave-0 (Recommended) | uuid v4-Feature-Fix + localdb-Drift-Doku. | ✓ |
| Nicht mitnehmen | Separater Hygiene-Plan später. | |

**User's choice:** Mitnehmen
**Notes:** D-Phase4-15.

### Q-Closing-2: Weiteres Thema vor CONTEXT.md?

| Option | Description | Selected |
|--------|-------------|----------|
| Nein — 14 Decisions decken Phase 4 ab (Recommended) | CONTEXT.md schreiben. | ✓ |
| Ja — Production-Data-Profile-Inhalt | Verschoben in C-Phase4-05 (Plan-Phase finalisiert). | |
| Ja — UAT-Plan / Verify-Phase-Kriterien | — | |
| Ja — Frontend-Migration-Kommunikation | Out of scope (Frontend-Workstream); als Hinweis in `<specifics>` notiert. | |

**User's choice:** Nein
**Notes:** Plus 1 Hygiene-Decision = 15 Decisions total.

---

## Claude's Discretion

Bewusst der Plan-Phase überlassen (siehe CONTEXT.md `<decisions>` § "Claude's Discretion"):

- **C-Phase4-01:** Migrations-Datei-Anzahl + Reihenfolge (Vorgabe: 4 separate Files).
- **C-Phase4-02:** `CarryoverService::rebuild_for_year` Surface (neuer Helper vs. inline; Service-Tier-Wechsel-Frage).
- **C-Phase4-03:** Cluster-Algorithmus-Implementierung (iterativ in Rust vs. SQL-Window).
- **C-Phase4-04:** Soft-Delete-Modus für migrierte extra_hours (neue Bulk-Methode vs. inline UPDATE).
- **C-Phase4-05:** Production-Data-Profile-Format-Detail (SC-1; Plan-Phase finalisiert Histogramm-Spalten).
- **C-Phase4-06:** Migrations-Heuristik-Vertragslookup-Performance (erst messen, dann optimieren).
- **C-Phase4-07:** REST-Routen-Schnitt für /admin/cutover/*.
- **C-Phase4-08:** Privileg-Surface (neues `cutover_admin` vs. Reuse `feature_flag_admin`).

## Deferred Ideas

(Aus `<deferred>` in CONTEXT.md, nicht in Phase 4 implementiert):

- REST-Endpunkt zum Auflisten der Quarantäne-Rows (HR-Admin-Surface, Folgephase).
- Auto-Cleanup der Quarantäne-Rows (HR-manuell).
- Restore-Endpunkt aus Carryover-Pre-Cutover-Backup (SQL-Pfad reicht für Phase 4).
- Frontend-Migration der /extra-hours-POST-Calls (Frontend-Workstream).
- Bulk-Carryover-Rebuild-Endpoint (Folgephase).
- Read-Compat-Shim für /extra-hours-Vacation-POSTs nach Cutover (bewusst NICHT Phase 4).
- Audit-Trail für feature_flag-Flips (schon Phase 2 deferred).
- REST-Endpunkte für feature_flag mit OpenAPI (Frontend-Admin-Screen).
- Migration weiterer ExtraHours-Kategorien zu range-basiert (out of scope v1).
- CarryoverService-Tier-Wechsel (entstanden aus C-Phase4-02).
- Quarantäne-Reason-i18n (Frontend-Workstream).
