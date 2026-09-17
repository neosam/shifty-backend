---
phase: 60-person-datenmodell-backfill
plan: 04
gap_closure: true
subsystem: database
tags: [sqlite, sqlx, migration, backfill, iso8601, foreign-keys, person]

# Dependency graph
requires:
  - phase: 60-person-datenmodell-backfill
    provides: Migration 20260916000000_create-person-backfill.sql, 16 Integrationstests, Doku-Sektionen zu person
provides:
  - person.created wird in einem Format geschrieben, das die repo-weite Parser-Kette (Iso8601::DATE_TIME) liest
  - Die Selbst-Assertion der Migration prüft die Deckungs-Invariante, nicht nur die Zählarithmetik
  - Korrigierte Doku-Aussage zur FK-Durchsetzung (sqlx setzt PRAGMA foreign_keys = ON per Verbindung)
affects: [61-person-dao, 62, 63]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "SQLite-TEXT-Zeitstempel in Migrationen: strftime('%Y-%m-%dT%H:%M:%S.000000000Z','now'), nie datetime('now')"
    - "Selbst-Assertionen in Migrationen prüfen die Invariante selbst, nicht eine Arithmetik, die sie nur unter ungeprüften Annahmen impliziert"
    - "Fixtures, die FK-verletzende Zustände brauchen, schalten PRAGMA foreign_keys auf der acquirierten Verbindung kurz ab und wieder an"

key-files:
  created: []
  modified:
    - migrations/sqlite/20260916000000_create-person-backfill.sql
    - service_impl/src/test/person_backfill.rs
    - docs/architecture/03-data-model.md
    - docs/architecture/03-data-model_de.md
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
    - docs/ops/database.md
    - docs/ops/database_de.md
    - .planning/phases/60-person-datenmodell-backfill/60-CONTEXT.md

key-decisions:
  - "Zeitstempel-Format empirisch gewählt: strftime('%Y-%m-%dT%H:%M:%S.000000000Z','now') ist der einzige getestete Kandidat, der in BEIDEN Zweigen der Repo-Parser-Kette parst (OffsetDateTime und PrimitiveDateTime)"
  - "Die Zählarithmetik bleibt in der Assertion stehen — sie ist als Dedup-Nachweis für Block 3e weiterhin sinnvoll, nur nicht als Deckungsnachweis"
  - "Der CR-01-Heilungsvorschlag des Reviews (3b/3d umschreiben) wurde NICHT umgesetzt — Scope war Assertion-Härtung, nicht Verhaltensänderung des Backfills"
  - "D-60-06 bleibt als Scope-Entscheidung stehen; nur die falsche Prämisse wurde mit einem datierten Vermerk korrigiert, die Entscheidung selbst nicht umgeschrieben"

patterns-established:
  - "Doku-Korrekturen zu widerlegten Befunden bekommen einen datierten Korrektur-Block statt einer stillen Umformulierung — der Irrtum bleibt nachvollziehbar"
  - "Ein Quelltext-Grep ist kein Beleg für Treiber-Defaults: was sqlx per Connection setzt, steht nicht in der Codebase"

requirements-completed: []

coverage:
  - id: D1
    description: "person.created ist mit der repo-weiten Parser-Kette (OffsetDateTime::parse mit Iso8601::DATE_TIME, Rückfall PrimitiveDateTime::parse) lesbar"
    requirement: "PERS-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#person_created_is_iso8601_parsable"
        status: pass
      - kind: manual_procedural
        ref: "cargo run gegen localdb.sqlite3 mit Bestandsdaten; SELECT created FROM person → 2026-09-17T04:24:28.000000000Z"
        status: pass
    human_judgment: false
  - id: D2
    description: "Die Selbst-Assertion der Migration bricht ab, wenn nach dem Backfill eine sales_person- oder user-Zeile ohne person_id zurückbleibt"
    requirement: "PERS-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#dangling_link_aborts_backfill"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#migration_aborts_on_count_mismatch"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#apply_twice_is_noop"
        status: pass
    human_judgment: false
  - id: D3
    description: "CREATE TEMP TABLE IF NOT EXISTS — ein Replay nach fehlgeschlagenem Lauf scheitert nicht mehr an 'table already exists'"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_backfill.rs#apply_twice_is_noop"
        status: pass
    human_judgment: true
    rationale: "Der eigentliche Pfad — zweiter Lauf auf DERSELBEN Verbindung nach einem Fehlschlag — ist von keinem Test exerziert. Der sqlx-Pool reicht die Verbindung im Fehlerfall zurück, ein Test müsste sie explizit festhalten. Der Guard ist korrekt, aber nur durch Inspektion belegt."
  - id: D4
    description: "Doku-Korrektur: FK-Durchsetzung ist aktiv (sqlx-Default), in 03-data-model, F01 und ops/database jeweils EN + DE"
    verification:
      - kind: manual_procedural
        ref: "cargo run Startup-Log zeigt als erste Anweisung: PRAGMA foreign_keys = ON; verwaister sales_person_user-Insert → FOREIGN KEY constraint failed (787); PRAGMA foreign_key_list listet beide Deklarationen"
        status: pass
    human_judgment: true
    rationale: "Doku-Prosa in sechs Dateien plus ein Korrektur-Vermerk in 60-CONTEXT.md. Die technische Behauptung ist empirisch belegt und re-checkbar, aber ob EN/DE inhaltlich deckungsgleich und die Formulierung angemessen ist, kann kein Test beurteilen."

# Metrics
duration: 40min
completed: 2026-09-17
status: complete
---

# Phase 60 Plan 04: Gap-Closure zum Code-Review Summary

**ISO-8601-Zeitstempel im Backfill statt `datetime('now')`, eine Selbst-Assertion, die die Deckungs-Invariante statt nur die Zählarithmetik prüft, und die Rücknahme der falschen Doku-Behauptung, FK-Durchsetzung sei in diesem Repo aus.**

## Performance

- **Duration:** ~40 min
- **Started:** 2026-09-17T04:05:00Z
- **Completed:** 2026-09-17T04:30:00Z
- **Tasks:** 3 Items, 6 Commits
- **Files modified:** 9

## Accomplishments

- **CR-03 geschlossen.** `person.created` wird nicht mehr als `2026-09-17 04:10:07` (Leerzeichen) geschrieben, sondern als `2026-09-17T04:24:28.000000000Z`. Das Format wurde **empirisch** bestimmt, nicht geraten: eine Wegwerf-Probe hat sechs Kandidaten gegen `OffsetDateTime::parse` **und** `PrimitiveDateTime::parse` mit `Iso8601::DATE_TIME` gefahren. Ergebnis: `datetime('now')` scheitert in beiden Zweigen, `strftime('%Y-%m-%dT%H:%M:%S','now')` parst nur als `PrimitiveDateTime`, und `strftime('%Y-%m-%dT%H:%M:%S.000000000Z','now')` parst in **beiden**. Der Rust-Schreibpfad selbst produziert `2026-09-17T04:08:52.818427581` (ohne `Z`) — die gewählte Variante ist also strikt robuster als das, was die Applikation schreibt, und sachlich korrekt, weil sqlites `'now'` UTC ist.
- **CR-01 geschlossen.** Block 4 prüft jetzt zusätzlich, dass keine `sales_person`- und keine `user`-Zeile ohne `person_id` zurückbleibt. Ein Regressionstest baut die verwaiste Link-Zeile direkt per SQL und belegt in derselben Assertion-Kette, *warum* die Zählprobe allein nicht reicht: sie geht mit `1 == 1 + 1 - 1` auf, während ein `user` ohne Identität dasteht.
- **WR-03 und WR-02 mit erledigt.** `CREATE TEMP TABLE IF NOT EXISTS` verhindert, dass ein Replay nach einem Fehlschlag an "table already exists" scheitert und damit den echten Fehler maskiert. `migration_aborts_on_count_mismatch` prüft jetzt `sqlx::Error::Database` **und** den Fehlertext `check constraint failed`, statt jeden beliebigen Datenbankfehler zu akzeptieren.
- **Die Doku-Unwahrheit zurückgenommen.** Die Phase-60-Behauptung "Shifty setzt `PRAGMA foreign_keys` nirgends, die FK-Deklarationen sind dekorativ" ist widerlegt. Dreifach belegt: sqlx-Quelle (`sqlx-sqlite-0.8.6/src/options/mod.rs:185`), Test-Harness (`PRAGMA foreign_keys` → `1`, verwaister Insert → Code 787, `PRAGMA foreign_key_list` listet beide Deklarationen trotz fehlendem Komma), und — am deutlichsten — das **Startup-Log des echten Servers**, in dem `PRAGMA foreign_keys = ON` die allererste ausgeführte Anweisung ist.
- **End-to-end validiert.** Die korrigierte Migration wurde nicht nur gegen In-Memory-Fixtures gefahren, sondern gegen die echte lokale `localdb.sqlite3` mit Bestandsdaten (2 `user`-Zeilen): Server startet, `person`-Zeilen tragen das korrigierte Format, beide neuen Assertion-Bedingungen halten.

## Task Commits

1. **Item 1 RED — `created`-Format festzurren** — `c08ead73` (test)
2. **Item 1 GREEN — Migration auf `strftime` umgestellt** — `2779270a` (fix)
3. **Item 3a — `migration_aborts_on_count_mismatch` verschärft** — `193a8e8e` (test)
4. **Item 3b RED — Deckungslücke bei verwaistem Link belegt** — `5bb986c2` (test)
5. **Item 3c GREEN — Assertion + `IF NOT EXISTS`** — `45f568a9` (fix)
6. **Item 2 — Doku-Korrektur FK-Durchsetzung (EN + DE) + CONTEXT-Vermerk** — `fe4717c6` (docs)

## Files Created/Modified

- `migrations/sqlite/20260916000000_create-person-backfill.sql` — `strftime`-Zeitstempel in Block 3e, erweiterte Selbst-Assertion in Block 4, `CREATE TEMP TABLE IF NOT EXISTS`; jede Änderung mit ihrer Begründung als Kommentar an der Stelle verankert
- `service_impl/src/test/person_backfill.rs` — zwei neue Tests (I-13, I-14), ein Fixture-Helfer `seed_dangling_link`, verschärfter Negativtest, Modul-Header um beide Invarianten ergänzt
- `docs/architecture/03-data-model.md` / `_de.md` — FK-Sektion neu geschrieben mit datiertem Korrektur-Block; Sektion "Enforcement: index, not foreign key" gibt jetzt den echten Grund an (`person_id` trägt gar keine FK-Deklaration)
- `docs/features/F01-employee-management.md` / `_de.md` — Beziehungs-Sektion korrigiert
- `docs/ops/database.md` / `_de.md` — `**[To verify]**`-Marker mit der verifizierten Antwort aufgelöst, inklusive Nachprüf-Queries und betrieblicher Konsequenz
- `.planning/phases/60-person-datenmodell-backfill/60-CONTEXT.md` — datierter Korrektur-Vermerk an D-60-06, der Deferred-Eintrag "`PRAGMA foreign_keys = ON` aktivieren" als hinfällig markiert

## Decisions Made

- **Zeitstempel-Kandidat empirisch entschieden, nicht aus dem Review übernommen.** Der Review schlug `strftime('%Y-%m-%dT%H:%M:%S','now')` vor. Das parst, aber nur über den `PrimitiveDateTime`-Rückfallzweig. Die gewählte Variante mit `.000000000Z` parst in beiden Zweigen und ist damit robuster gegen eine spätere Vereinfachung der DAO-Leseseite.
- **Der CR-01-Heilungsvorschlag wurde bewusst nicht umgesetzt.** Der Review schlug vor, Block 3b und 3d umzuschreiben, damit verwaiste Links geheilt statt erkannt werden. Das ändert das Backfill-**Verhalten** und gehört zu CR-02, das ausdrücklich nicht im Scope war. Umgesetzt wurde nur, was der Auftrag sagt: die Assertion soll prüfen, was sie behauptet. Konsequenz: ein verwaister Link bricht die Migration jetzt ab, statt lautlos eine Lücke zu committen — fail-closed statt fail-silent.
- **`IF NOT EXISTS` statt des im Review vorgeschlagenen `DROP TABLE IF EXISTS temp....`.** Beide Varianten funktionieren; `IF NOT EXISTS` ist die kleinere Änderung und ausreichend, weil die Tabelle beim Wiedereintritt immer leer ist (die einzige schreibende Anweisung ist der `INSERT`, und der schreibt im Fehlerfall nichts).
- **D-60-06 nicht umgeschrieben, nur mit Vermerk versehen.** Die Entscheidung — den Ablehnungstest über den Index zu führen — bleibt sachlich richtig, weil `person_id` gar keine FK-Deklaration trägt. Falsch war nur die Begründung. Der Vermerk trennt das explizit, damit spätere Phasen weder die Entscheidung noch den Irrtum blind übernehmen.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Lokale `localdb.sqlite3` nach Migrations-Edit nicht mehr startfähig**

- **Found during:** Item 1, Gate-Lauf `cargo run`
- **Issue:** Die Migration `20260916000000` war lokal bereits angewandt. Eine Änderung am Dateiinhalt ändert die sqlx-Prüfsumme, der Start bricht mit `VersionMismatch(20260916000000)` ab. Das war zu erwarten und vom Auftrag ausdrücklich gedeckt ("editing it in place is therefore correct and intended"), musste aber aufgelöst werden, um das `cargo run`-Gate aus CLAUDE.md überhaupt fahren zu können.
- **Fix:** Datei-Backup nach `localdb.sqlite3.pre-60-04.bak` (gitignored via `localdb.sqlite3*`), dann Ledger-Zeile gelöscht und die Phase-60-Artefakte per SQL zurückgedreht (`DROP INDEX`/`DROP TABLE person`/`ALTER TABLE … DROP COLUMN person_id`), anschließend Migration über `cargo run` frisch angewandt.
- **Files modified:** keine versionierten Dateien — nur der lokale, gitignorierte Entwicklungsstand
- **Verification:** Server startet, `SELECT created FROM person` liefert `2026-09-17T04:24:28.000000000Z`, die erweiterte Zählprobe hält (`count-check ok=1`)
- **Committed in:** nicht committet (gitignored)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Kein Scope-Zuwachs. Die Auflösung betrifft ausschließlich den lokalen Entwicklungsstand, keine versionierte Datei.

## Issues Encountered

- **Die Fixture für Item 3 war zunächst nicht baubar** — und zwar aus genau dem Grund, den Item 2 behandelt. Der erste Versuch, eine verwaiste `sales_person_user`-Zeile zu seeden, wäre an der aktiven FK-Durchsetzung gescheitert. Das hat Item 2 unabhängig bestätigt, bevor ich die Doku angefasst habe, und ist im Test-Helfer `seed_dangling_link` samt Begründung dokumentiert: das Pragma wird auf der acquirierten Verbindung kurz abgeschaltet und wieder an. Der Test hält damit auch fest, dass der geprüfte Zustand über die Applikation **nicht** erreichbar ist.

## Ehrliche Einordnung

Drei Punkte, bei denen ich lieber deutlich bin als bündig:

1. **Item 3 war Härtung, kein Bugfix.** Der Auftrag sagt das bereits, und die Umsetzung bestätigt es: der Zustand ist über die Applikation nicht erzeugbar. Der Wert liegt darin, dass die Assertion jetzt prüft, was sie behauptet — nicht darin, dass etwas repariert wurde. Das steht so auch im Migrations-Kommentar und im Test-Doc, damit es später nicht als "da war mal ein Bug" fehlgelesen wird.
2. **D3 (`IF NOT EXISTS`) ist nur durch Inspektion belegt.** Der Pfad, den der Guard adressiert — zweiter Lauf auf derselben Verbindung nach einem Fehlschlag —, wird von keinem Test exerziert, weil der sqlx-Pool die Verbindung im Fehlerfall zurückreicht. Ein Test müsste sie explizit festhalten. Ich habe das nicht gebaut; im `coverage`-Block ist D3 deshalb `human_judgment: true`.
3. **Die Rollback-Zusage bleibt ungetestet.** Der Review hält in WR-02 fest, dass `run_backfill_section` ohne äußere Transaktion läuft; die Transaktionsklammer kommt im Produktionspfad allein von `sqlx::migrate`. Daran habe ich nichts geändert — es lag außerhalb der drei Items. Ich habe die Formulierung im neuen Test-Kommentar aber so gewählt, dass sie nichts Falsches behauptet, und nutze den Umstand sogar aktiv (der Test liest nach dem Abbruch den geschriebenen Zwischenstand aus, um die Lücke zu belegen).

## Out of Scope — für den Backlog

- **`20251016154210_add-user-invitation-table.sql` hat denselben Zeitstempel-Defekt.** Auf Anweisung nicht angefasst. Die Datei schreibt Zeitstempel im `datetime('now')`-Format mit Leerzeichen, und `dao_impl_sqlite/src/user_invitation.rs:45-47` liest sie mit derselben `Iso8601::DATE_TIME`-Kette, die das Leerzeichen ablehnt. Das ist bestehende Schuld, nicht von Phase 60 eingeführt — aber es ist derselbe Fehlermodus und gehört auf den Backlog. Anders als bei `person` gibt es hier bereits eine Leseseite im Code, der Defekt ist also potenziell schon heute wirksam.
- **CR-02** (verwaiste Link-Zeile in der Gegenrichtung blockiert den Serverstart; erreichbar über `delete_user`) — nicht im Scope dieses Plans. Mit der jetzt verschärften Assertion ist die Abbruchbedingung sogar breiter geworden. Die vom Review vorgeschlagene Pre-Flight-Abfrage für das Deploy-Runbook fehlt weiterhin. **Vor dem Milestone-Deploy adressieren** — die Phase-60-Migration läuft beim Serverstart.
- **WR-01** (Doku sagt "UUID v4", `randomblob(16)` setzt keine Version-/Variant-Bits), **WR-04** (`user_update_timestamp`-Trigger feuert beim Backfill, Doku sagt "unberührt"), **WR-05** (kein Struktur-Guard auf dem Markerbereich), **WR-06**, **IN-01** bis **IN-05** — unverändert offen.

## User Setup Required

Keine externe Konfiguration nötig.

**Lokaler Hinweis:** `localdb.sqlite3` wurde neu aufgebaut, der vorherige Stand liegt als `localdb.sqlite3.pre-60-04.bak` im Projektverzeichnis (gitignored). Wer die Migration auf einem anderen lokalen Stand hatte, braucht denselben Schritt — die Prüfsumme der bearbeiteten Migrationsdatei stimmt nicht mehr mit dem sqlx-Ledger überein. Auf ein Deployment wirkt sich das nicht aus: die Migration wurde nirgends ausgerollt.

## Next Phase Readiness

- **Phase 61 (`PersonDao`) ist entblockt.** `person.created` ist jetzt mit exakt der Parser-Kette lesbar, die `billing_period.rs` und `user_invitation.rs` verwenden. Der Test `person_created_is_iso8601_parsable` hält das fest, damit eine spätere Änderung an der Migration nicht unbemerkt zurückfällt.
- **Vor dem Milestone-Deploy offen:** CR-02 (Pre-Flight-Abfrage für verwaiste `sales_person_user`-Zeilen). Die Assertion bricht jetzt breiter ab als vorher — das ist gewollt fail-closed, macht eine Pre-Flight-Prüfung vor dem Deploy aber wertvoller, nicht weniger.
- **Gates:** `cargo build`, `cargo test --workspace` (1029 passed, 0 failed) und `cargo clippy --workspace --all-targets -- -D warnings` liefen vor jedem GREEN-/docs-Commit vollständig grün. Die beiden RED-Commits (`c08ead73`, `5bb986c2`) sind naturgemäß rot gegen `cargo test` — für sie wurden `cargo build` und `cargo clippy` gefahren und der Fehlschlag jeweils auf die richtige Ursache geprüft (Parse-Fehler bzw. `Ok(())` trotz erfüllter Vorbedingung), nicht auf eine leere Fixture.
- **`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13** — kein persistierter `value_type` wurde berührt, `billing_period_report.rs` nicht angefasst.

---
*Phase: 60-person-datenmodell-backfill*
*Plan: 04 (Gap-Closure)*
*Completed: 2026-09-17*

## Self-Check: PASSED

Alle 9 geänderten/erzeugten Dateien existieren auf der Platte, alle 6 Commit-Hashes
sind in `git log` auffindbar, und `git diff --diff-filter=D 0ee3106a..HEAD` meldet
keine gelöschten Dateien.
