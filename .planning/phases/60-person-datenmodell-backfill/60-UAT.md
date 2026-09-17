---
status: partial
phase: 60-person-datenmodell-backfill
source: [60-VERIFICATION.md]
started: 2026-09-17T00:00:00Z
updated: 2026-09-17T05:38:32Z
---

## Current Test

[testing paused — 1 item outstanding]

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

result: blocked
blocked_by: other
reason: "User: 'Lass uns weitermachen. Ich prüfe später den Meilenstein.' — Die Query
verlangt Zugriff auf die produktive SQLite-Datei und ist erst vor dem v2.7-Deploy
sinnvoll auszuführen. Bewusst auf den Meilenstein-Check vertagt; bleibt Deploy-Gate."

### 2. WR-01 — Doku-Aussage "UUID v4" für `person.id` korrigieren oder Erzeugung umstellen

expected: Entweder die Doku wird auf "zufällige 16 Bytes, kompatibel zu `Uuid::from_slice`, bewusst ohne RFC-4122-Version/Variant-Bits" korrigiert, oder die Erzeugung wird in SQL auf echte v4-Bits umgestellt.

Betrifft `docs/architecture/03-data-model.md:221` und die deutsche Fassung.

Hintergrund: `randomblob(16)` setzt weder Version-Nibble noch Variant-Bits. Der Test
`person_id_parses_as_uuid` prüft nur Länge und `Uuid::from_slice`-Kompatibilität — das
trifft auf beliebige 16 Bytes zu und deckt die Doku-Behauptung nicht. Redaktionsentscheidung,
kein automatisch entscheidbarer Fakt.

result: pass
resolution: |
  Doku-Weg gewählt (Erzeugung unverändert). Umgesetzt:
  - `docs/architecture/03-data-model.md` — `person.id` beschreibt jetzt 16 Zufallsbytes
    aus `randomblob(16)`, roh gespeichert, über `Uuid::from_slice` gelesen, explizit
    **keine** RFC-4122-UUID (weder Version-Nibble noch Variant-Bits).
  - `docs/architecture/03-data-model_de.md` — synchron mitgezogen (Docs-Freshness-Gate).
  - `service_impl/src/test/person_backfill.rs` — Doc-Kommentar auf
    `person_id_parses_as_uuid` hält fest, was der Test deckt und was nicht, und nennt
    die Migration als Ort einer etwaigen späteren Verschärfung.
  Nicht angefasst: `rebooking_batch.id` (Zeile 149/157 + F14) — diese IDs entstehen
  Rust-seitig und sind echte v4.
  Gates: `cargo test -p service_impl person_backfill` 18/18 grün,
  `cargo clippy --workspace --all-targets -- -D warnings` sauber.

## Summary

total: 2
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 1

## Gaps

[none — der offene Punkt ist ein Deploy-Gate, kein Code-Defekt]
