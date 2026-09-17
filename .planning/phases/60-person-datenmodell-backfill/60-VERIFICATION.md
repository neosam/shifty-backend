---
phase: 60-person-datenmodell-backfill
verified: 2026-09-17T00:00:00Z
status: human_needed
score: 5/5 Roadmap-Success-Criteria verifiziert (Code + Tests); 2 offene operative/dokumentarische Punkte als human_verification
behavior_unverified: 0
overrides_applied: 0
human_verification:
  - test: "CR-02 (Code-Review) vor dem Milestone-v2.7-Deploy bewusst adressieren: Pre-Flight-Query gegen die Produktions-`localdb.sqlite3` fahren — `SELECT COUNT(*) FROM sales_person_user spu WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.id = spu.sales_person_id) OR NOT EXISTS (SELECT 1 FROM user u WHERE u.name = spu.user_id);` — und bei einem Treffer > 0 vor dem Deploy bereinigen oder den Rollback-Pfad (Datei-Backup) bereithalten."
    expected: "Entweder die Query liefert 0 (Migration startet sicher durch), oder ein Mensch trifft bewusst die Entscheidung, das Risiko einzugehen bzw. die Daten vorher zu bereinigen. Diese Entscheidung kann kein Test treffen, weil sie von der realen Produktionsdatenlage abhängt, die in keiner Fixture vorliegt."
    why_human: "Der über `delete_user` (Hard-Delete ohne Link-Aufräumung) erreichbare Pfad ist in Produktion nicht ausgeschlossen; die Migration bricht in diesem Fall den Serverstart mit einer nicht-diagnostischen `CHECK constraint failed`-Meldung ab. Das ist von Plan 60-04 bewusst nicht behoben worden (Scope war Assertion-Härtung, nicht Backfill-Verhalten) und steht in der 60-04-SUMMARY selbst als 'Vor dem Milestone-Deploy adressieren'. Kein Test kann die reale Datenlage einer fremden Produktions-DB prüfen."
  - test: "Dokumentationsaussage 'UUID v4' für `person.id` gegen den tatsächlichen Erzeugungsmechanismus abgleichen (`docs/architecture/03-data-model.md:221`, `_de.md` analog)."
    expected: "Entweder die Doku wird auf 'zufällige 16 Bytes, kompatibel zu `Uuid::from_slice`, bewusst keine RFC-4122-Version/Variant-Bits' korrigiert, oder die Erzeugung wird in SQL auf echte v4-Bits umgestellt."
    why_human: "`randomblob(16)` setzt keine RFC-4122-Version-Nibble und keine Variant-Bits; `person_id_parses_as_uuid` prüft nur Länge und `Uuid::from_slice`-Kompatibilität, was für beliebige 16 Bytes zutrifft und die Doku-Behauptung nicht deckt (WR-01 aus 60-REVIEW.md, von Plan 60-04 explizit unadressiert gelassen). Das ist eine Redaktionsentscheidung, kein automatisch entscheidbarer Fakt."
---

# Phase 60: Person-Datenmodell + Backfill Verification Report

**Phase Goal:** `person`-Tabelle, nullable `person_id`-FKs mit wiederhergestelltem UNIQUE,
idempotenter Backfill inkl. soft-gelöschter Sales Persons (Backend-only).

**Verified:** 2026-09-17
**Status:** human_needed
**Re-verification:** Nein — Erstverifikation

## Goal Achievement

Alle fünf Roadmap-Success-Criteria (ROADMAP.md §„Phase 60: Person-Datenmodell +
Backfill") wurden gegen die tatsächliche Migrationsdatei und das Testmodul
geprüft — nicht gegen die SUMMARY-Behauptungen. Ergebnis: alle fünf sind mit
belastbaren, nicht-vakuumen Tests hinterlegt und laufen grün gegen den
aktuellen Code-Stand (`18 passed, 0 failed` in `person_backfill`,
`cargo test --workspace` 1029/0 laut vorheriger Orchestrator-Verifikation,
hier durch den gezielten `-p service_impl person_backfill`-Lauf erneut
bestätigt). Zwei Punkte bleiben offen und werden nicht stillschweigend
übergangen, sondern als menschliche Entscheidung eskaliert (siehe „Human
Verification Required").

### Observable Truths

| # | Truth (Roadmap-SC) | Status | Evidence |
|---|---|---|---|
| SC1 | Nach `sqlx migrate run` trägt jede `sales_person`-Zeile (inkl. soft-gelöschter) und jede `user`-Zeile ein nicht-leeres `person_id`; jedes `sales_person_user`-Paar zeigt auf dieselbe `person.id`; Zählprobe hält. | ✓ VERIFIED | Migration Zeilen 61–129 (Blöcke 3a–3e); Tests `full_identity_coverage`, `count_identity_holds`, `soft_deleted_sales_person_gets_person_id` — alle grün. Zusätzlich `dangling_link_aborts_backfill` (Plan 60-04) beweist, dass eine Verletzung dieser Invariante die Migration jetzt tatsächlich abbricht, statt lautlos zu committen (CR-01 aus 60-REVIEW.md geschlossen: Block 4 prüft seit Plan 60-04 drei Bedingungen — beide NULL-Zählungen plus die Arithmetik — statt nur die Arithmetik). |
| SC2 | Ein zweiter Lauf des Backfills ändert weder `COUNT(person)` noch eine Zuordnung (echter Vorher/Nachher-Diff). | ✓ VERIFIED | `apply_twice_is_noop`: führt den markierten Abschnitt zweimal aus, vergleicht einen vollständigen `Snapshot` (Zeilenzahl + jede `(id, person_id)`-Paarung) vor/nach dem zweiten Lauf und trägt eine Vorbedingungs-Assertion (`person_count > 0`), die einen vakuum-grünen Vergleich zweier leerer Zustände ausschließt. Kein bloßes „läuft durch". |
| SC3 | Soft-gelöschte Sales Person mit `billing_period_sales_person`-Zeile bekommt `person_id`. | ✓ VERIFIED | `soft_deleted_sales_person_gets_person_id` seedet exakt diesen Fall und prüft `person_id.is_some()` plus Fortbestand der Snapshot-Zeile. Migration setzt bewusst keinen `WHERE deleted IS NULL`-Filter (Blöcke 3a/3c), mit erklärendem Kommentar gegen einen künftigen „Aufräum"-Pass. |
| SC4 | Ein direkter SQL-Schreibversuch auf doppelte `person_id` wird von der Datenbank abgelehnt (nicht Applikationsprüfung). | ✓ VERIFIED | `duplicate_person_id_rejected_by_db_sales_person` / `..._user`: direkter `UPDATE`, `Result` nicht entpackt, Assertion auf `sqlx::Error::Database` **und** Fehlertext `unique` (nicht nur `is_err()`); zusätzlich Nachweis, dass die zweite Zeile nichts halb geschrieben bekommen hat. Läuft bewusst gegen den UNIQUE-Index, nicht gegen einen Fremdschlüssel (D-60-06) — korrekt begründet, siehe Dok-Abschnitt unten. |
| SC5 | `sales_person.id`/`user.name` bitgleich, Snapshots lösen unverändert auf, Workspace-Suite grün, `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13. | ✓ VERIFIED | `legacy_keys_are_bit_identical_after_backfill` vergleicht Byte-/Zeichenwerte vor/nach dem Backfill und prüft die Snapshot-Auflösung erneut per Join. `grep -c 'CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13' service_impl/src/billing_period_report.rs` → 1; Datei nicht im Phasen-Diff. Workspace-Suite lief laut Orchestrator-Vorverifikation grün (1029/0); hier zusätzlich der gezielte `person_backfill`-Lauf bestätigt (18/18). |

**Score:** 5/5 Roadmap-Truths verifiziert. Zusätzlich 2 Punkte als
Mensch-Entscheidung eskaliert (siehe unten) — sie sind keine
Roadmap-Success-Criteria, aber reale, im Review benannte und in der
60-04-SUMMARY selbst als offen bezeichnete Risiken.

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `migrations/sqlite/20260916000000_create-person-backfill.sql` | Schema-DDL `person`, `ADD COLUMN person_id` auf beiden Tabellen, zwei vollständige UNIQUE-Indizes, markierter Backfill-Abschnitt, Selbst-Assertion | ✓ VERIFIED | Existiert, 180 Zeilen, alle Bestandteile per Lesen der Datei bestätigt (nicht nur gegrept). `person` hat exakt vier Spalten (`id`, `created`, `update_process`, `update_version`), keine `deleted`-Spalte. Beide Indizes ohne `WHERE`-Klausel (Zeilen 46–50). Block 4 (Zeilen 131–178) prüft seit Plan 60-04 drei Bedingungen statt nur die Zählarithmetik. |
| `service_impl/src/test/person_backfill.rs` | 16 Testfunktionen (Plan 60-01 + 60-02) | ✓ VERIFIED (übertroffen) | 18 `#[tokio::test]`-Funktionen gezählt (8 aus 60-01, 8 aus 60-02, 2 aus 60-04 für CR-01/CR-03). Alle 18 liefen im Verifikationslauf grün. |
| `service_impl/src/test/mod.rs` (Registrierung) | `#[cfg(test)] pub mod person_backfill;` | ✓ VERIFIED | Zeile 121–128 bestätigt. |
| Doku-Abschnitte (`03-data-model.md`/`_de`, `F01`/`_de`, `db-schema-er.mmd`, `ops/database.md`/`_de`) | Person-Sektion + FK-Korrektur | ✓ VERIFIED (mit Einschränkung) | Alle Dateien existieren und enthalten die Person-Sektion; die FK-Korrektur (Plan 60-04) ist inhaltlich präzise und dreifach belegt (sqlx-Quelle, Test-Harness, Server-Startup-Log). Einschränkung: die „UUID v4"-Behauptung (Zeile 221 in `03-data-model.md`, DE analog) ist nachweislich falsch (WR-01, unten) und der Backfill-Absatz beschreibt weiterhin nur die Zähl-Assertion, nicht die seit Plan 60-04 erweiterte Deckungsprüfung in Block 4 — beides siehe „Human Verification Required". |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| Block 3c (`sales_person.person_id`) | Block 3d (`user.person_id`) | korrelierte Subquery über `sales_person_user` | ✓ WIRED | `UPDATE user SET person_id = (SELECT sp.person_id FROM sales_person_user spu JOIN sales_person sp ...)`. Genau **ein** `randomblob(16)`-Aufruf pro Paar bestätigt: `grep -c 'randomblob(16)'` == 4 (3a, 3b, 3c, `update_version` in 3e) — Block 3d würfelt nicht selbst. `full_identity_coverage` bestätigt Bytegleichheit beider Seiten. |
| Migrationsdatei-Marker (`-- BACKFILL START/END`) | Testmodul (`backfill_section()`) | `include_str!` + `split_once` | ✓ WIRED | Marker existieren exakt je einmal; `backfill_section()` panic't bei fehlendem Marker mit klarer Meldung statt still zu scheitern. |
| Block 4 (Selbst-Assertion) | Migrations-Transaktion | `CREATE TEMP TABLE ... CHECK (ok = 1)` | ✓ WIRED | `migration_aborts_on_count_mismatch` und `dangling_link_aborts_backfill` bestätigen den tatsächlichen Abbruch mit `sqlx::Error::Database` **und** dem Fehlertext `check constraint failed` (nicht nur `is_err()` — WR-02 aus dem Review wurde geschlossen). |
| Test-Fixture (`seed_dangling_link`) | FK-Durchsetzung (sqlx-Default) | `PRAGMA foreign_keys = OFF/ON` auf derselben Connection | ✓ WIRED | Bestätigt, dass der von CR-01 beschriebene Zustand über die Applikation nicht erreichbar ist, weil FK-Durchsetzung aktiv ist — konsistent mit der korrigierten D-60-06-Begründung. |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|---|---|---|---|---|
| PERS-01 | 60-01, 60-04 | Jede Sales Person/jeder Benutzer genau eine Person; verknüpfte Paare teilen sich dieselbe Person | ✓ SATISFIED | `full_identity_coverage`, `count_identity_holds`, `dangling_link_aborts_backfill` |
| PERS-02 | 60-01 | Backfill wiederholbar, keine Doppelzuordnung | ✓ SATISFIED | `apply_twice_is_noop` |
| PERS-03 | 60-01 | Soft-gelöschte Sales Persons erfasst | ✓ SATISFIED | `soft_deleted_sales_person_gets_person_id` |
| PERS-04 | 60-02 | DB erzwingt 1:1-Garantie | ✓ SATISFIED | `duplicate_person_id_rejected_by_db_sales_person`, `..._user` |
| PERS-06 | 60-02 | Bestehende Referenzen unverändert, Snapshot-Version unverändert | ✓ SATISFIED | `legacy_keys_are_bit_identical_after_backfill`, Grep-Lock auf `CURRENT_SNAPSHOT_SCHEMA_VERSION` |

Cross-Check gegen `REQUIREMENTS.md`: Zeilen 84–89 und 113 listen exakt
PERS-01, PERS-02, PERS-03, PERS-04, PERS-06 als „Phase 60 / Complete" —
deckungsgleich mit den in den PLAN-Frontmatters deklarierten Requirement-IDs.
PERS-05 ist korrekt als „Phase 61 / Pending" ausgewiesen und taucht in keinem
Phase-60-Plan auf — **kein orphaned requirement**.

### Anti-Patterns Found

Keine Debt-Marker (`TBD`/`FIXME`/`XXX`) und keine `TODO`/`HACK`/`PLACEHOLDER`-
Kommentare in der Migrationsdatei oder im Testmodul gefunden. Keine leeren
Handler, keine hartkodierten leeren Rückgabewerte im Liefergegenstand dieser
Phase (reines SQL + Testcode).

**Historisch behobene Vakuum-Tests (positiv zu vermerken, kein aktueller
Befund):** Der Review (60-REVIEW.md) hatte zwei Instanzen echter
Vakuum-Assertions dokumentiert — `sales_person_index_is_not_partial`/
`user_index_is_not_partial` prüften ursprünglich `contains("UNIQUE")`, was
durch den Indexnamen selbst (`..._unique_idx`) immer wahr gewesen wäre; und
`migration_aborts_on_count_mismatch` prüfte nur `is_err()`. Beide wurden vor
dieser Verifikation bereits behoben — im aktuellen Code stehen
`starts_with("CREATE UNIQUE INDEX")` (Präfix-Vergleich) bzw. eine
zusätzliche Prüfung auf `sqlx::Error::Database` und den Fehlertext
`check constraint failed`. Diese Verifikation hat den aktuellen Code
gelesen, nicht die SUMMARY-Behauptung übernommen — beide Fixes sind an Ort
und Stelle bestätigt (Zeilen 1059–1067 bzw. 471–480 in
`person_backfill.rs`).

## Honesty-Check der SUMMARYs

Alle vier SUMMARYs (60-01 bis 60-04) wurden gegen den Code geprüft. Keine
Diskrepanz zwischen SUMMARY-Behauptung und Code-Realität gefunden — im
Gegenteil: Die SUMMARYs listen ihre eigenen Deviations, offenen Punkte
(„Ehrliche Einordnung" in 60-04) und nicht behobenen Review-Findings (WR-01,
WR-04 bis WR-06, IN-01 bis IN-05, CR-02) explizit und ungeschönt auf, statt
sie zu verschweigen. Diese Offenheit wurde stichprobenartig verifiziert:

- Die Behauptung „16 Integrationstests" (60-02) und „18 Tests nach Plan
  60-04" wurde nachgezählt: 18 `#[tokio::test]`-Funktionen im aktuellen
  Stand — korrekt.
- Die Behauptung, CR-02 sei bewusst nicht behoben, wurde verifiziert: kein
  Test und keine SQL-Änderung adressiert die umgekehrte Verwaisungsrichtung
  (`sales_person_user.user_id` zeigt ins Leere).
- Die Behauptung, WR-01 (UUID-v4-Doku-Fehler) sei offen geblieben, wurde
  verifiziert: `docs/architecture/03-data-model.md:221` sagt weiterhin
  „UUID v4", `randomblob(16)` setzt keine Version-/Variant-Bits.

Fazit: **keine Überzeichnung gefunden.** Diese Phase ist ein positives
Gegenbeispiel zum in der Prompt beschriebenen Standardfall „SUMMARY sagt
mehr, als der Code hält".

## Human Verification Required

### 1. CR-02 — Verwaiste `sales_person_user`-Zeile kann den Server-Start blockieren

**Test:** Vor dem Milestone-v2.7-Deploy gegen eine Kopie der
Produktions-`localdb.sqlite3` ausführen:
```sql
SELECT COUNT(*) FROM sales_person_user spu
WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.id = spu.sales_person_id)
   OR NOT EXISTS (SELECT 1 FROM user u WHERE u.name = spu.user_id);
```
**Expected:** 0. Falls > 0: vor dem Deploy bereinigen oder das Rollback-Risiko
(Datei-Backup laut D-60-09) bewusst in Kauf nehmen.
**Why human:** `delete_user` (Hard-Delete ohne Link-Aufräumung,
`rest/src/permission.rs:90` → `dao_impl_sqlite/src/lib.rs:112`) ist ein
bestehender, erreichbarer Admin-Pfad. Trifft dieser Zustand in Produktion
zu, bricht die jetzt verschärfte Selbst-Assertion (Plan 60-04) den
Serverstart mit einer nicht-diagnostischen Fehlermeldung ab — bewusst
fail-closed, aber ohne Pre-Flight-Prüfung. Kein Test kann die reale
Produktionsdatenlage einer fremden DB prüfen; 60-04-SUMMARY selbst hält
das als offenen Punkt „Vor dem Milestone-Deploy adressieren" fest.

### 2. WR-01 — Doku behauptet „UUID v4", Backfill erzeugt Zufallsbytes ohne RFC-4122-Bits

**Test:** `docs/architecture/03-data-model.md:221` und
`03-data-model_de.md` gegen `randomblob(16)` in der Migration abgleichen.
**Expected:** Entweder Doku-Korrektur auf „zufällige 16 Bytes, kompatibel
zu `Uuid::from_slice`, bewusst keine RFC-4122-Version" (EN + DE), oder eine
SQL-Änderung, die echte v4-Bits setzt.
**Why human:** Redaktionsentscheidung zwischen zwei technisch validen
Optionen; `person_id_parses_as_uuid` prüft nur Länge und
`Uuid::from_slice`-Kompatibilität und deckt die Doku-Behauptung nicht.
Niedrige Priorität (kein aktiver Bug, aber ein OpenAPI-`format: uuid`-
Validator könnte die Werte später ablehnen).

## Gaps Summary

Keine Gaps im Sinne von „gaps_found" (kein Truth FAILED, kein Artifact
MISSING/STUB, kein Key-Link NOT_WIRED, keine Debt-Marker). Alle fünf
Roadmap-Success-Criteria sind mit belastbaren, wiederholt lauffähigen Tests
hinterlegt (18/18 grün, `cargo clippy --workspace --all-targets -- -D
warnings` sauber). Der Status ist dennoch `human_needed`, weil zwei reale,
im Code-Review benannte und in den SUMMARYs selbst offen gehaltene Punkte
(CR-02 Produktions-Risiko, WR-01 Doku-Fehler) eine menschliche Entscheidung
brauchen, die kein Test treffen kann — beide sind Eskalationen, keine
Blocker für den Phase-Goal-Abschluss selbst.

**Zusätzlich als dauerhaft offen, aber bewusst und dokumentiert nicht
behoben** (keine Eskalation nötig, da explizit als Backlog markiert):
WR-04 (Trigger-Nebenwirkung auf `user.update_timestamp` — Doku sagt
„unberührt", stimmt für Legacy-Schlüssel, nicht für die Audit-Spalte),
WR-05 (kein Struktur-Guard gegen Schrumpfen des Marker-Bereichs), WR-06
(teilweise durch I-13/I-14 aus Plan 60-04 mitbehoben, `update_process`
bleibt ungetestet), IN-01 bis IN-05 (Detail-Findings, niedrige Priorität).
Diese stehen bereits explizit in der 60-04-SUMMARY unter „Out of Scope" —
sie werden hier nicht erneut als Gap gezählt, sondern nur zur
Vollständigkeit referenziert.

---

*Verified: 2026-09-17*
*Verifier: Claude (gsd-verifier)*
