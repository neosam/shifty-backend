---
phase: 61-personservice-aufl-sung-von-sales-person-user
plan: 08
subsystem: dao
tags: [gap-closure, review-cr-02, invariant, sqlite, sqlx-offline-cache, docs-freshness-gate]

# Dependency graph
requires:
  - phase: 61-02
    provides: "service_impl/src/test/person_creation.rs mit dem In-Memory-Harness (setup_pool, tx_of, commit, person_id_of_user, person_row_count) und den Invarianten N-1 bis N-8"
  - phase: 61-06
    provides: "docs/architecture/03-data-model{,_de}.md §Die dauerhaft pruefbare Invariante — der Abschnitt, der hier um den Loeschpfad erweitert wurde"
  - phase: 61-07
    provides: "Anweisung -- A7) der DROP-Migration, deren Loeschbedingung hier wortgleich in den Laufzeitpfad uebernommen wurde"
provides:
  - "BasicDaoImpl::clear_all loescht ausschliesslich traegerlose person-Zeilen; user-Zeilen ueberleben mit aufloesbarer person_id"
  - "Zwei Regressionstests N-9/N-10, die beide Haelften der Invariante nach clear_all pinnen"
  - "Aktualisierter .sqlx-Offline-Cache fuer die geaenderte query!-Makro-Query"
  - "Datenmodell-Doku (EN + DE), die clear_all ausdruecklich als Loeschpfad mit derselben Pflicht benennt"
affects: [62-personenverwaltung-liste, 63-personen-detail]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Eine Invariante ohne FK-Rueckhalt muss auf ALLEN Pfaden von Hand gehalten werden — Loeschpfade eingeschlossen; sie sind der blinde Fleck, weil die Doku nur Anlegepfade als Verpflichtete nannte"
    - "Eine Loeschbedingung, die in Migration und Laufzeitpfad dieselbe Invariante herstellt, wird woertlich gleich formuliert — der Abgleich ist dann ein Textvergleich, keine Analyse"
    - "Eine Aenderung am Text einer sqlx::query!-Makro-Query invalidiert den .sqlx-Offline-Cache; Regenerierung braucht DATABASE_URL und explizit KEIN SQLX_OFFLINE=true"

key-files:
  created: []
  modified:
    - dao_impl_sqlite/src/lib.rs
    - service_impl/src/test/person_creation.rs
    - .sqlx/query-37ab13c0dc5445ef2673f8a8eb4584a6c3e96ce06e654ab1acbaa5b66c97139f.json
    - .sqlx/query-f83453231dd44b800571f35ea3c6babbba2237c8f835294bf0d8bb86de677d3b.json
    - docs/architecture/03-data-model.md
    - docs/architecture/03-data-model_de.md

decisions:
  - "Die Loeschbedingung ist woertlich Anweisung -- A7) der DROP-Migration (NOT EXISTS auf sales_person und user). Migration und Laufzeitpfad stellen dieselbe Invariante her und sollen deshalb denselben Text tragen."
  - "Die Position der person-Anweisung in der Kette bleibt unveraendert (nach sales_person, vor special_day) — die Reihenfolge ist tragend, weil erst das Leeren von sales_person die zugehoerigen Personen traegerlos macht."
  - "Der .sqlx-Cache wurde gegen eine Wegwerf-SQLite ausserhalb des Repos regeneriert (Muster aus 61-05), nicht gegen localdb.sqlite3 — die Dev-DB bleibt unberuehrt, insbesondere ihr in 61-07 von Hand nachgezogener Ledger-Eintrag."
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 13 — kein persistierter value_type beruehrt."
  - "Docs-Freshness-Gate: nur 03-data-model{,_de}.md geaendert. Das ER-Diagramm bleibt unberuehrt (kein Schema-Element), F01/F12 ebenfalls (keine Trait-Signatur, kein Permission-Pfad)."

metrics:
  duration: "~20 min"
  completed: 2026-09-17
  tasks: 2
  files-modified: 6

status: complete
---

# Phase 61 Plan 08: Gap Closure CR-02 Summary

`BasicDaoImpl::clear_all` loeschte bis hierher pauschal alle `person`-Zeilen und
liess die `user`-Zeilen stehen — nach jedem Aufruf zeigte jeder Login, auch
`DEVUSER` und `admin`, auf eine nicht mehr existierende Person; jetzt trifft die
Loeschung nur noch traegerlose Zeilen, wortgleich zur Anweisung `-- A7)` der
DROP-Migration.

## Was gebaut wurde

### Task 1 + Task 2 (ein Commit, `79898073`)

**`dao_impl_sqlite/src/lib.rs` — `BasicDaoImpl::clear_all`:**

| vorher | nachher |
|---|---|
| `DELETE FROM person;` | `DELETE FROM person WHERE NOT EXISTS (SELECT 1 FROM sales_person sp WHERE sp.person_id = person.id) AND NOT EXISTS (SELECT 1 FROM user u WHERE u.person_id = person.id);` |

Position in der Anweisungskette unveraendert: nach `DELETE FROM sales_person;`,
vor `DELETE FROM special_day;`. Die uebrigen sechs Anweisungen sind unberuehrt.

Der Kommentar begruendete vorher ausschliesslich die Reihenfolge gegenueber
`sales_person` und erwaehnte `user` mit keinem Wort — genau die Luecke, aus der
CR-02 entstand. Der neue Kommentar nennt die vier Punkte aus dem Plan:
Traegerlosigkeit als einziges Loeschkriterium; das Ueberleben der Logins und
ihrer Identitaet (D-61-01); die fehlende `FOREIGN KEY`-Deklaration auf
`user.person_id` als Grund, warum die Datenbank das nicht abfaengt; und die
Herkunft der Bedingung als zweite Haelfte der Invariante aus
`docs/architecture/03-data-model.md` samt Verweis auf `-- A7)`.

**`service_impl/src/test/person_creation.rs`:** 8 → 10 Tests, Modulkopf um die
Invarianten-Eintraege N-9 und N-10 erweitert (Herkunft CR-02 benannt).

| Test | Pinnt |
|---|---|
| `clear_all_laesst_keinen_benutzer_ohne_person_zurueck` (N-9) | `sales_person` ist leer; `person_id_of_user("alice")` ist `Some`; genau eine `person`-Zeile zu diesem Wert |
| `clear_all_raeumt_traegerlose_personen_auf` (N-10) | die Invarianten-Query aus der Doku liefert 0; `COUNT(person) == COUNT(user)` |

Beide Tests tragen die Kommentarzeile, dass `clear_all` keine Transaktion nimmt
und direkt auf dem Pool arbeitet — die Fixture muss vorher committet sein. Jede
Assertion nennt CR-02, die verletzte Invariantenhaelfte und die fehlende
FK-Deklaration.

**`.sqlx`:** ein Eintrag getauscht (`query-f8345323…` entfaellt,
`query-37ab13c0…` neu). Regeneriert mit
`cargo sqlx prepare --workspace -- --all-targets` gegen eine Wegwerf-SQLite im
Scratchpad, in die zuvor die volle Migrationskette eingespielt wurde.

**Doku (EN + DE, im selben Commit):** Der Invarianten-Abschnitt endete mit „Wer
einen dritten Anlage-Pfad schreibt, muss das von Hand einhalten". Dahinter steht
jetzt ein Absatz, der `clear_all` ausdruecklich als **Loesch**-Pfad einordnet,
der dieselbe Pflicht von der anderen Seite erfuellt, und der festhaelt, dass der
einzige Weg dorthin `POST /dev/...` hinter `#[cfg(feature = "mock_auth")]` ist —
Entwicklungs- und Testbetrieb, nicht Produktion.

## RED-Nachweis (Task 1, woertlich)

Gegen das **unveraenderte** `clear_all`, vor Task 2:

```
running 10 tests
test test::person_creation::clear_all_raeumt_traegerlose_personen_auf ... FAILED
test test::person_creation::clear_all_laesst_keinen_benutzer_ohne_person_zurueck ... FAILED
[... die acht bestehenden Tests N-1 bis N-8: ok ...]

---- test::person_creation::clear_all_laesst_keinen_benutzer_ohne_person_zurueck stdout ----
panicked at service_impl/src/test/person_creation.rs:410:5:
assertion `left == right` failed: CR-02: jede person-Zeile hat mindestens einen
Traeger, und umgekehrt muss die person_id einer ueberlebenden user-Zeile
aufloesbar bleiben; ein pauschales DELETE FROM person hinterlaesst hier einen
identitaetslosen Benutzer, und die Datenbank meldet das nicht, weil
user.person_id keine FOREIGN KEY-Deklaration traegt
  left: 1
 right: 0

---- test::person_creation::clear_all_raeumt_traegerlose_personen_auf stdout ----
panicked at service_impl/src/test/person_creation.rs:456:5:
assertion `left == right` failed: nach clear_all sind die Benutzer die einzigen
verbliebenen Traeger: die Person der Sales Person ist weg, die des Benutzers
steht noch
  left: 1
 right: 0

test result: FAILED. 8 passed; 2 failed; 0 ignored; 0 measured; 877 filtered out
```

Kein `error[E0…]` in der Ausgabe — der Lauf kompilierte. Kein Commit aus
Task 1; die Aenderung lag als uncommitted working copy vor (`jj status`).

GREEN nach Task 2: `test result: ok. 10 passed; 0 failed`.

## Deviations from Plan

### 1. [Akzeptanzkriterium korrigiert] N-10 war im RED-Lauf ebenfalls rot

Das Task-1-Kriterium lautete: „`clear_all_raeumt_traegerlose_personen_auf` steht
auf `ok` — die heutige Implementierung loescht zu viel, nicht zu wenig; dieser
Test ist die Gegenprobe gegen eine Ueberkorrektur und darf schon gruen sein."

Tatsaechlich war N-10 rot. Der Grund liegt im Plan selbst: sein `<behavior>`
verlangt fuer N-10 **zwei** Zusicherungen — die Invarianten-Query aus der Doku
**und** `COUNT(person) == COUNT(user)`. Die erste war gegen das alte `clear_all`
trivial erfuellt (nach `DELETE FROM person` gibt es ueberhaupt keine
`person`-Zeile mehr, also auch keine traegerlose). Die zweite konnte gar nicht
halten: `person` war leer (0), `user` hatte eine Zeile (1). Das
Akzeptanzkriterium war also mit dem `<behavior>`-Text desselben Tasks
unvereinbar.

Aufgeloest zugunsten des `<behavior>`-Texts: beide Zusicherungen bleiben stehen,
das Kriterium ist falsch und wird hiermit korrigiert. Der Zweck von N-10 — die
Gegenprobe gegen eine Ueberkorrektur, bei der das `DELETE FROM person`
ersatzlos entfaellt — bleibt dabei vollstaendig erhalten: liesse man die
Personen der geleerten Sales Persons stehen, schluege die erste Zusicherung an.
Der Test deckt jetzt beide Fehlrichtungen ab statt nur einer. Das ist dieselbe
Klasse von Befund wie Deviation 1 in `61-07-SUMMARY.md`.

Kein Produktionsbezeichner wurde angefasst, um ein Kriterium zu erfuellen.

### 2. [Rule 3 - Blocker] `cargo sqlx prepare` gegen eine Wegwerf-DB statt `localdb.sqlite3`

Der Plan liess beide Wege offen. Gewaehlt wurde die Wegwerf-SQLite im
Scratchpad (`sqlx database create` + `sqlx migrate run --source
migrations/sqlite`, zuletzt angewendet: `20260917000000 drop-sales-person-user`),
damit die in 61-07 von Hand nachgezogene Ledger-Checksumme in
`localdb.sqlite3` unberuehrt bleibt. `git status --porcelain .sqlx` zeigt danach
genau eine geloeschte und eine neue Datei — das erwartete Ergebnis.

Sonst: keine weiteren Auto-Fixes nach Regel 1-3, keine architektonische Frage
(Regel 4), keine Authentifizierungs-Gates.

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo test -p service_impl --lib person_creation` | `ok. 10 passed; 0 failed` (vorher 8) |
| `cargo test --workspace` | alle Suiten `0 failed` (887 + 11 + 93 + 0/0/0/0/0/0) |
| Integrations-Harnesses in `shifty_bin`, die `clear_all` vor dem Seeden rufen | in den 93 enthalten, `0 failed` |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0 |
| `SQLX_OFFLINE=true cargo build --workspace` | Exit 0 — der Offline-Cache deckt die neue Query |
| `cargo build -p shifty_bin --no-default-features --features oidc,json_logging` | Exit 0 |
| `git status --porcelain .sqlx` | `D query-f8345323….json`, `?? query-37ab13c0….json` — genau ein Tausch |
| `git diff --name-only docs/` | genau `03-data-model.md` und `03-data-model_de.md` |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION` | unveraendert 13 (`billing_period_report.rs:140`) |
| `git diff --stat Cargo.toml Cargo.lock` | leer (T-61-SC) |
| Ein Commit fuer DAO + Tests + `.sqlx` + beide Doku-Dateien | `79898073` |

## Commits

| Commit | Inhalt |
|---|---|
| `79898073` | `fix(61-08)`: `clear_all`-Loeschbedingung + Kommentar, N-9/N-10, `.sqlx`-Tausch, `03-data-model{,_de}.md` |

Task 1 hat plangemaess **nicht** committet — ein eigenstaendiger RED-Commit
haette das harte Gate aus `CLAUDE.md` (`cargo test` gruen vor jedem Commit)
verletzt. Der RED-Nachweis steht stattdessen oben.

## Known Stubs

Keine.

## Threat Flags

Keine neue sicherheitsrelevante Oberflaeche. Die beiden `mitigate`-Dispositionen
des Plans sind umgesetzt: T-61-29 (`clear_all` hinterlaesst keine verwaisten
`user`-Verweise) ist durch N-9 und N-10 belegt, T-61-31 (Doku nennt nur
Anlegepfade als Verpflichtete) durch den neuen Absatz in beiden Sprachen.
T-61-30 bleibt `accept` — das `mock_auth`-Feature-Gate ist unveraendert, der
Produktionsbuild (`--features oidc,json_logging`) ist als Gate gruen.

## Fuer die naechsten Wellen

- 61-09 und 61-10 schliessen WR-04 und WR-06. `AUTHENTICATION.md` (WR-06) fuehrt
  weiterhin `CREATE TABLE sales_person_user` — nach dieser Welle unveraendert
  offen.
- Der Deferral Record aus `61-07-PLAN.md` (WR-01/02/03/07, IN-01…IN-07) bleibt
  unveraendert gueltig; WR-02 gehoert nach Phase 62.
- Merkposten fuer Phase 62/63: die Personenverwaltung darf sich darauf
  verlassen, dass `PersonService::get(user.person_id)` nach jedem Pfad dieser
  Phase — Anlegen, Verknuepfen, Loesen, Migration und jetzt Leeren — aufloest.

## Self-Check: PASSED

Alle sechs beruehrten Dateien existieren im erwarteten Zustand auf der Platte;
Commit `79898073` ist im Log auffindbar.
