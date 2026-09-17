---
status: testing
phase: 60-person-datenmodell-backfill
source: [60-VERIFICATION.md]
started: 2026-09-17T00:00:00Z
updated: 2026-09-17T00:00:00Z
---

## Current Test

number: 1
name: CR-02 — Pre-Flight-Query gegen die Produktions-Datenbank vor dem v2.7-Deploy
expected: |
  Entweder die Query liefert 0 (die Migration startet sicher durch), oder es wird
  bewusst entschieden, die Daten vorher zu bereinigen bzw. das Risiko zu tragen und
  den Rollback-Pfad (Datei-Backup der SQLite-Datei) bereitzuhalten.
awaiting: user response

## Tests

### 1. CR-02 — Pre-Flight-Query gegen die Produktions-Datenbank vor dem v2.7-Deploy

expected: Query liefert 0, oder bewusste Entscheidung zu Bereinigung/Risiko mit bereitgehaltenem Datei-Backup.

Auszuführen gegen die produktive SQLite-Datei:

```sql
SELECT COUNT(*) FROM sales_person_user spu
 WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.id = spu.sales_person_id)
    OR NOT EXISTS (SELECT 1 FROM user u WHERE u.name = spu.user_id);
```

Hintergrund: Die in Plan 60-04 verschärfte Selbst-Assertion bricht die Migration ab,
wenn verwaiste `sales_person_user`-Zeilen existieren — mit einer nicht-diagnostischen
`CHECK constraint failed`-Meldung, die den Serverstart blockiert. Über die laufende
Anwendung sind solche Zeilen nicht erzeugbar (sqlx setzt `PRAGMA foreign_keys = ON`,
`sales_person` wird nur soft-gelöscht). Restrisiko sind Altbestände aus manuellen
Eingriffen, Restores oder älteren Tools. Kein Test kann die reale Datenlage prüfen.

result: [pending]

### 2. WR-01 — Doku-Aussage "UUID v4" für `person.id` korrigieren oder Erzeugung umstellen

expected: Entweder die Doku wird auf "zufällige 16 Bytes, kompatibel zu `Uuid::from_slice`, bewusst ohne RFC-4122-Version/Variant-Bits" korrigiert, oder die Erzeugung wird in SQL auf echte v4-Bits umgestellt.

Betrifft `docs/architecture/03-data-model.md:221` und die deutsche Fassung.

Hintergrund: `randomblob(16)` setzt weder Version-Nibble noch Variant-Bits. Der Test
`person_id_parses_as_uuid` prüft nur Länge und `Uuid::from_slice`-Kompatibilität — das
trifft auf beliebige 16 Bytes zu und deckt die Doku-Behauptung nicht. Redaktionsentscheidung,
kein automatisch entscheidbarer Fakt.

result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
