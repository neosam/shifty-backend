---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 02
subsystem: dao
tags: [person, directory, soft-delete, orphan-cleanup, sqlite, d-63-01]
status: complete
requires:
  - "dao::person::PersonDao (Phase 61 — find_by_id/create als Muster fuer Row-Layout und TryFrom)"
  - "dao::person_directory::PersonDirectoryDao::list_directory_entries (Phase 62 — Join-Semantik, D-62-05/WR-02)"
  - "migrations/sqlite/20260916000000_create-person-backfill.sql (Beleg: person_id ohne Foreign Key)"
provides:
  - "PersonDao::delete(id, tx) — harter DELETE FROM person, idempotent"
  - "PersonDao::has_live_facets(person_id, tx) -> bool — D-63-01-Traegerprobe, Lesart (b)"
  - "PersonDao::find_by_sales_person(sales_person_id, tx) -> Option<PersonEntity>"
  - "PersonDao::find_by_user(username, tx) -> Option<PersonEntity>"
  - "PersonDirectoryDao::find_directory_entry(person_id, tx) -> Option<PersonDirectoryRow>"
affects:
  - "63-04 (Detailansicht-Lesepfad: entry_from_row wird unveraendert auf der Einzelzeile wiederverwendet)"
  - "63-05 (kombinierter Anlege-Pfad: Rueckwaerts-Lookups liefern die gewinnende Identitaet nach dem Merge, D-61-03)"
  - "63-0x remove_facet / Orphan-Cleanup (has_live_facets + delete)"
tech_stack:
  added: []
  patterns:
    - "Traegerprobe als EIN Statement mit zwei EXISTS-Teilausdruecken statt zwei Roundtrips — 'lebend' hat genau eine Definition an genau einer Stelle"
    - "Einzelsatz-Lesepfad als woertliche Kopie der Listen-Query plus WHERE-Einschraenkung, damit Liste und Detail nicht divergieren koennen"
    - "Rueckwaerts-Lookup bewertet nicht, er loest auf: find_by_sales_person liefert auch fuer soft-geloeschte Sales Persons; die Bewertung macht ausschliesslich has_live_facets"
key_files:
  created:
    - service_impl/src/test/person_detail_dao.rs
  modified:
    - dao/src/person.rs
    - dao/src/person_directory.rs
    - dao_impl_sqlite/src/person.rs
    - dao_impl_sqlite/src/person_directory.rs
    - service_impl/src/test/mod.rs
    - .sqlx/ (5 neue Offline-Cache-Eintraege)
decisions:
  - "D-63-01 wird als Lesart (b) festgeschrieben: eine Person ist traegerlos, wenn keine user-Zeile und keine sales_person-Zeile mit deleted IS NULL mehr auf sie zeigt. Eine soft-geloeschte Sales Person zaehlt als weg."
  - "Der daraus folgende haengende person_id auf der soft-geloeschten sales_person-Zeile ist ausdruecklich akzeptiert und an drei Stellen dokumentiert (Trait-Doc, Impl-Kommentar, Test D-4) — kein Bug fuer einen spaeteren Aufraeum-Pass."
  - "Die Phase-61-Klausel 'PersonDao bleibt auf genau vier Methoden beschraenkt' ist aufgeloest. Gemeint war 'keine Vorratshaltung', nicht ein Zahlenlimit; jede der vier neuen Methoden hat einen in Phase 63 gebauten Aufrufer."
  - "find_directory_entry kopiert die Listen-Query woertlich statt sie zu abstrahieren — der Divergenz-Schutz ist ein Kommentar in beiden Queries plus Test E-6, nicht eine gemeinsame Hilfsfunktion (SQL-Makros von sqlx lassen sich nicht sinnvoll teilen)."
metrics:
  duration_minutes: 42
  tasks_completed: 3
  files_modified: 6
  tests_added: 13
  completed: 2026-09-20
---

# Phase 63 Plan 02: DAO-Lese- und Loeschprimitive fuer Detailansicht und Orphan-Cleanup — Summary

`PersonDao` kann eine Person jetzt loeschen, ihre lebenden Traeger zaehlen und sich von
beiden Traeger-Seiten her aufloesen lassen; `PersonDirectoryDao` liefert zu einer
`person_id` genau die Rohzeile, aus der Phase 62 bereits die Listenzeile baut.

## Was gebaut wurde

**`dao/src/person.rs` + `dao_impl_sqlite/src/person.rs`** — vier neue Trait-Methoden,
alle auf der geteilten Transaktion:

- `delete(id, tx)` — `DELETE FROM person WHERE id = ?`. Hart, nicht soft: die Tabelle
  hat keine `deleted`-Spalte (D-60-04, gesperrt). Die Zeilenzahl wird bewusst nicht
  geprueft, damit ein zweiter Aufruf ein No-Op ohne Fehler bleibt.
- `has_live_facets(person_id, tx) -> bool` — **ein** Statement mit zwei
  `EXISTS`-Teilausdruecken (`user` beliebig, `sales_person` mit `deleted IS NULL`),
  nicht zwei Roundtrips. So hat „lebend" genau eine Definition an genau einer Stelle.
- `find_by_sales_person` / `find_by_user` — `INNER JOIN` von `person` auf die jeweilige
  Traeger-Seite ueber `person_id`, `fetch_optional` plus
  `.as_ref().map(PersonEntity::try_from).transpose()` nach dem Muster von `find_by_id`.

Der Modulkopf haelt fest, dass die Phase-61-Klausel („genau vier Methoden") Vorratshaltung
verbieten sollte, nicht belegten Bedarf, und nennt pro neuer Methode den Aufrufer aus
Phase 63.

**`dao/src/person_directory.rs` + `dao_impl_sqlite/src/person_directory.rs`** —
`find_directory_entry(person_id, tx) -> Option<PersonDirectoryRow>`. Dieselbe
Drei-Wege-Outer-Join-Query wie `list_directory_entries`, ergaenzt um `p.id = ?` und mit
`fetch_optional`. Der Soft-Delete-Filter bleibt in der `ON`-Klausel; die Traeger-Bedingung
(`sp.id IS NOT NULL OR u.name IS NOT NULL`) bleibt erhalten, damit eine traegerlos
gewordene Person hier genauso unsichtbar ist wie in der Liste.

**`service_impl/src/test/person_detail_dao.rs`** — 13 Integrationstests gegen eine echte
in-memory-SQLite mit den echten DAO-Impls, ohne Mock. Der Modulkopf wiederholt die
gewaehlte D-63-01-Lesart, damit die Entscheidung im Testcode steht und nicht nur im Plan.

## Die entschiedene D-63-01-Semantik

RESEARCH.md hatte die Lesart offen gelassen (Pitfall 1 / Assumption A1). Dieser Plan
schreibt **Lesart (b)** fest: eine Person ist traegerlos, wenn keine `user`-Zeile und
keine `sales_person`-Zeile mit `deleted IS NULL` mehr auf sie zeigt.

Der Preis ist ausdruecklich benannt und an drei Stellen dokumentiert (Trait-Doc,
Impl-Kommentar, Test `delete_funktioniert_bei_haengendem_person_id`): eine soft-geloeschte
Sales Person behaelt einen `person_id`, der nach dem Loeschen auf nichts mehr aufloest.
Das ist gefahrlos, weil `sales_person.person_id` keinen Foreign Key auf `person(id)` hat
(Phase-60-Migration: ADD COLUMN plus UNIQUE-Index) und weil
`billing_period_sales_person` auf `sales_person.id` referenziert, nie auf `person.id`.
Historische Snapshots bleiben unberuehrt.

## Abweichungen vom Plan

### Auto-behoben

**1. [Rule 2 — fehlende kritische Testabdeckung] Test E-6 ergaenzt**

- **Gefunden bei:** Task 3, im Mutationstest nach dem ersten gruenen Lauf.
- **Problem:** Die im Plan vorgegebene Testliste (E-1 bis E-5, L-1 bis L-4, D-1, D-4, R-1)
  haelt vollstaendig gegen eine Implementierung, die `sp.deleted IS NULL` in die
  `WHERE`- statt in die `ON`-Klausel verschiebt. Verifiziert per Mutation: Query
  umgestellt → 12 von 12 Tests weiterhin gruen. Genau diese Verschiebung ist aber die
  Prohibition des Plans („sonst degeneriert der LEFT JOIN zum INNER JOIN") und der Bug,
  den Phase 62 gefixt hat. Der Grund fuer die Luecke: `sp.deleted IS NULL` ist
  NULL-sicher und damit auch in `WHERE` fuer nicht gematchte Zeilen wahr — die
  Degeneration schlaegt nur bei einer Person durch, deren Sales Person **existiert und
  soft-geloescht ist** und die zusaetzlich einen Benutzer hat. Diese Konstellation fehlte.
- **Fix:** Test `find_directory_entry_soft_geloeschte_sales_person_laesst_benutzer_sichtbar`
  (E-6) ergaenzt — das Gegenstueck zu V-3 in `person_directory_dao.rs`. Gegen die
  mutierte Query faellt er (RED verifiziert), gegen die korrekte haelt er.
- **Dateien:** `service_impl/src/test/person_detail_dao.rs`
- **Commit:** 6110636e

**2. [Rule 2 — Scope-Grenze dokumentiert] `deferred-items.md` ergaenzt**

- Die vorbestehende `clippy::needless_lifetimes`-Warnung in
  `service_impl/src/test/person_directory_dao.rs:149` (fremde Datei, feuert nur unter
  `--all-targets`, nicht unter dem CLAUDE.md-Gate und nicht in CI) sowie das fehlende
  `cargo fmt` in der Nix-Dev-Shell sind in `deferred-items.md` protokolliert statt
  behoben.

### Sonstiges

- `.sqlx`-Offline-Cache musste fuer die fuenf neuen `query!`/`query_as!`-Makros per
  `cargo sqlx prepare --workspace` erzeugt werden (5 neue Dateien, keine geloescht);
  die Cache-Eintraege sind Teil der jeweiligen Task-Commits.
- Ein Impl-Kommentar wurde umformuliert, weil er die Zeichenfolge `UPDATE person`
  enthielt und damit das Acceptance-Kriterium von Task 1 (grep-Zaehlung == 0) falsch
  positiv ausgeloest haette. Inhaltlich unveraendert.

## TDD-Gate-Compliance

Der Plan traegt `type: tdd`, ordnet die Tests aber vollstaendig dem dritten Task zu,
waehrend Tasks 1 und 2 Trait- plus Impl-Arbeit sind — eine Trait-Methode ohne
Implementierung compiliert nicht, ein RED-Commit vor Task 1 waere also nicht
konstruierbar gewesen. Die Gate-Reihenfolge ist darum als
`feat` → `feat` → `test` gelaufen, nicht als `test` → `feat`.

Als Ersatz fuer den entfallenen RED-Schritt wurde die Testsuite per **Mutationstest**
gegen Vakuitaet abgesichert:

| Mutation | Erwartung | Ergebnis |
|---|---|---|
| `AND deleted IS NULL` aus `has_live_facets` entfernt | L-3 und D-4 fallen | 2 Tests rot — bestaetigt |
| `sp.deleted IS NULL` von `ON` nach `WHERE` verschoben | mindestens ein Test faellt | **0 Tests rot — Luecke**, siehe Abweichung 1 |
| dieselbe Mutation nach Ergaenzung von E-6 | E-6 faellt | 1 Test rot — bestaetigt (echtes RED) |

Beide Mutationen wurden anschliessend vollstaendig zurueckgenommen
(`git diff` gegen HEAD leer), bevor committet wurde.

## Verifikation

| Gate | Ergebnis |
|---|---|
| `cargo build` (Workspace) | Exit 0 |
| `cargo test --workspace` | alle gruen, 920 Tests in `service_impl` (13 neue), 95 Integrationstests |
| `cargo test -p service_impl person_detail_dao` | 13 passed, 0 failed |
| `cargo clippy --workspace -- -D warnings` | Exit 0 |
| Keine neue Datei unter `migrations/sqlite/` | bestaetigt (`git diff HEAD~3 HEAD -- migrations/` leer, weiterhin 67 Dateien) |
| `grep -c 'async fn ' dao/src/person.rs` | 8 (vier bestehende + vier neue) |
| `sed -n '/fn delete/,/^    }/p' … \| grep -c 'UPDATE person'` | 0 |
| `sed -n '/fn find_directory_entry/,/^    }/p' … \| grep -c 'sp.deleted IS NULL'` | 2 |

## Docs-Freshness-Gate

Beruehrte Trigger-Dateien: `dao/**/*.rs` (`person.rs`, `person_directory.rs`).
Faellige Doku-Sektion laut CLAUDE.md-Tabelle: **Datenmodell-Sektion der passenden
`docs/features/F*.md`** (Personenverwaltung) — plus die deutsche/englische
Schwesterdatei. Nicht in diesem Plan ausgefuehrt: Plan **63-10** fuehrt die
Doku-Aktualisierung der Phase zentral durch. `shifty_bin/src/main.rs`,
`migrations/sqlite/*.sql` und `service_impl/src/permission.rs` wurden **nicht**
beruehrt.

## Threat-Model-Abgleich

| Threat | Disposition | Umsetzung |
|---|---|---|
| T-63-02-A (`delete` tampering) | mitigate | Der DAO loescht nur die genannte id und bewertet nicht; die Berechtigung faellt im Business-Logic-Service (63-05) nach `has_live_facets` und Admin-Gate. Kein zweiter Aufrufer angelegt. |
| T-63-02-B (`find_directory_entry` disclosure) | mitigate | Ungegated wie jeder DAO im Repo; kein REST-Handler ruft DAOs direkt auf. |
| T-63-02-C (verwaiste `person`-Zeilen) | mitigate | `has_live_facets` + `delete` sind genau der Abbaupfad; Test D-4 belegt ihn. |
| T-63-02-SC (Supply Chain) | mitigate | Keine neue Abhaengigkeit, kein Package-Install. |

Keine neue Angriffsflaeche ausserhalb des Registers: keine Endpunkte, keine Auth-Pfade,
keine Schema-Aenderung.

## Self-Check: PASSED

- `service_impl/src/test/person_detail_dao.rs` — FOUND
- `dao/src/person.rs`, `dao/src/person_directory.rs` — FOUND
- `dao_impl_sqlite/src/person.rs`, `dao_impl_sqlite/src/person_directory.rs` — FOUND
- Commit `483fb661` — FOUND
- Commit `cd6f2627` — FOUND
- Commit `6110636e` — FOUND
