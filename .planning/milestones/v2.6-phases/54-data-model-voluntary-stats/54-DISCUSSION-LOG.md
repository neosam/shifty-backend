# Phase 54: Data-Model + Voluntary Statistics (F1 + F2) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-06
**Phase:** 54-data-model-voluntary-stats
**Areas discussed:** D-F1-01 F1-Denominator, D-F2-01 Mid-Week-Vertragswechsel F2-Soll, D-54-DM-01 UNIQUE-Constraint-Shape, D-54-DM-02 Marker-Approach auf extra_hours

---

## D-F1-01 — F1-Denominator-Definition

| Option | Description | Selected |
|--------|-------------|----------|
| (a) strikt „contract-weeks in year" | Wochen mit gültiger `working_hours`-Row mit `expected_hours > 0` (Research-Empfehlung; Konsistenz mit v1.4 CVC-05) | |
| (b) AVG-01-A-22-1 absence-adjusted | Wochen mit Contract minus komplett-Absence-Wochen (analog v2.1 AVG-01, konsistent mit direkt danebenliegender Ø-Anwesenheit) | |
| **User-Variante:** „Wochen mit Arbeitsvertrag, `expected_hours` darf 0 sein" | Weder (a) noch (b) — Vertragsexistenz-Test statt Volumen-Filter; 0-h-Verträge zählen mit | ✓ |

**User's choice:** „nur die Wochen in denen es einen Arbeitsvertrag gab. expected hours kann da sich [sic: „auch"] 0 sein"

**Notes:** Explizit weiter als Research-Empfehlung. Rationale: F1 misst „Ø
freiwillig pro Vertragswoche" — sobald Kontrakt existiert, ist die Person unter
Vertrag, auch wenn Kontraktvolumen 0 ist. 0-h-Vertragler, der 4 h freiwillig
macht, hat 4h/Woche, nicht undefined. Absence-Adjust ausgeschlossen (F1 misst
geleistet, nicht anwesend — Doppelkorrektur).

---

## D-F2-01 — Mid-Week-Vertragswechsel-Semantik für VOL-ACCT-01-Soll

| Option | Description | Selected |
|--------|-------------|----------|
| **A pro-rata** | anteilig je nach Tagen unter jedem Kontrakt (neuer Aggregator nötig) | ✓ |
| B latest-active | zum Wochenende gültiger `committed_voluntary` gilt für ganze Woche (Research-Empfehlung, reuse `get_working_hours_for_week`) | |
| C split-week | beide Werte separat zählen (DTO-Auswirkung) | |

**User's choice:** A pro-rata

**Notes:** Bewusste Wahl gegen Research-Empfehlung. Rationale (Claude): F2-Soll ist
kumulative Konten-Zusage, keine Punkt-Messung. Pro-rata ist ehrlicher — ein
Mittwochs-Kontraktwechsel mit halbierter Voluntary-Zusage soll ab Mittwoch anteilig
wirken, nicht rückwirkend für die ganze Woche „umgeschrieben" werden. Konsequenz:
Planer muss neuen Aggregator (`committed_voluntary_prorata_for_week` oder inline
in `committed_voluntary_target_for_year`) bauen. Assistant hat bei Antwort um
Bestätigung gebeten wegen Kleinbuchstaben-Case-Ambiguity; keine Korrektur des Users
erhalten → A pro-rata festgehalten.

---

## D-54-DM-01 — UNIQUE-Constraint-Shape auf `rebooking_batch`

| Option | Description | Selected |
|--------|-------------|----------|
| **(i)** `(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL` | Globale Wochen-Sperre über alle Kinds (Research-Empfehlung; Claim-on-Suggest fällt aus Constraint direkt raus) | ✓ |
| (ii) `(kind, sales_person_id, iso_year, iso_week)` | HR-Suggestion + Auto-Cron dürfen koexistieren; Konflikt-Logik im Service | |

**User's choice:** (i)

**Notes:** Deckt sich mit Research + Pitfall 4 (kombinierter Idempotenz- + TOCTOU-
+ Doppel-Zählungs-Guard). `INSERT ... ON CONFLICT DO NOTHING` als no-op für
Cron-Restart / Backfill über bereits verarbeitete Wochen. Rejected-Slot-Freigabe-
Semantik ist Phase-55-Detail.

---

## D-54-DM-02 — Marker-Approach auf `extra_hours`

| Option | Description | Selected |
|--------|-------------|----------|
| **(x) STRING-Enum** | Neue Spalte `source TEXT NOT NULL DEFAULT 'manual'`, Werte `'manual' \| 'rebooking'`; kein NULL-JOIN in Balance-/F1-/F2-Chain | ✓ |
| (y) FK auf `rebooking_batch_entry` | `rebooking_batch_entry_id BLOB NULL REFERENCES rebooking_batch_entry(id)`; explizite Beziehung, aber NULL-JOIN in jeder Chain | |

**User's choice:** (x)

**Notes:** Reverse-Lookup „welche ExtraHours gehört zu welchem Batch" bleibt via
`rebooking_batch_entry.extra_hours_out_id / _in_id` (BLOB-FK) verfügbar — kein
Datenverlust. Migration additiv per `ALTER TABLE ADD COLUMN ... DEFAULT 'manual'`;
sqlx-prepare-Gate danach.

---

## Claude's Discretion

- **REST-Route-Design für F1/F2:** additive Response auf existierenden Employee-Year-
  Report-Endpoint ODER neuer dedizierter Endpoint. Beide präzedent-kompatibel.
- **i18n-Wording final** („Ist / Soll / Δ" vs. „Ist-Ø freiwillig pro Woche /
  Zugesagt / Konto"). Planer/UI-Phase präzisiert.
- **FE-Row-Layout** (eine Zeile mit 3 Werten vs. 3 separate Zeilen).
- **Toggle-Seed-Migration** eigene Datei vs. inline.

## Deferred Ideas

- Snapshot-Schema-Version-Bump 12→13 (REB-AUTO-05, Phase-56-discuss-phase).
- F5-Reject-Wochen-Slot-Freigabe-Semantik (Phase 55).
- F5-Stale-Vorschlag-Strategie (Phase 55/56).
- F4-Cron-Cadence + Uhrzeit (Phase 56).
- UI Voluntary-Konto-Historie / Batch-Timeline (v2.7+).
- Employee-Self-Service-View (v2.7+).
- Multi-Role-Approval / Notifications / Undo (v2.7+).

## Zusätzliche User-Wünsche

Keine — User hat keine Sonderwünsche außer den vier Discuss-Points geäußert.
