---
phase: 61-personservice-aufl-sung-von-sales-person-user
plan: 10
subsystem: docs
tags: [gap-closure, review-wr-06, docs-freshness-gate, schema-doku, repo-sweep]

# Dependency graph
requires:
  - phase: 61-05
    provides: "migrations/sqlite/20260917000000_drop-sales-person-user.sql — die Migration, auf die der Ersatz-Absatz verweist"
  - phase: 61-06
    provides: "docs/architecture/03-data-model{,_de}.md §Die dauerhaft pruefbare Invariante und docs/architecture/04-auth{,_de}.md §Self-access gate for sales-person data — die kanonischen Ziele der neuen Querverweise"
  - phase: 61-07
    provides: "Block A der DROP-Migration inklusive A7 (Traeger-Aufraeumung) — Grundlage der Invarianten-Formulierung im Ersatz-Absatz"
  - phase: 61-08
    provides: "BasicDaoImpl::clear_all mit der traegerlos-Bedingung — Grundlage der Aussage, dass DEVUSER/admin ihre Identitaet ueber einen Dev-Reset behalten"
provides:
  - "AUTHENTICATION.md zeigt user mit person_id blob(16) und person als eigenstaendige Tabelle plus beide vollstaendigen UNIQUE-Indizes"
  - "Verweis-Banner am Dateianfang: bei Widerspruch gewinnen die zweisprachig gepflegten Dokumente unter docs/architecture/"
  - "Querverweise auf 04-auth.md#self-access-gate-for-sales-person-data und 03-data-model.md"
  - "Abschnitt zur Login-Auto-Anlage dokumentiert die Identitaetsvergabe in create_user (D-61-01); Codeblock an shifty_bin/src/main.rs angeglichen"
  - "Bestandsaufnahme des Repo-Sweeps mit Einordnung jeder verbliebenen Fundstelle"
affects: [62-personenverwaltung-liste, 63-personen-detail]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Ein Dokument ausserhalb von docs/ das dieselbe Sache beschreibt braucht eine explizite Vorrangregel im Dokument selbst — sonst ist beim naechsten Drift unentscheidbar, welche Seite gewinnt"
    - "Ein Doku-Sweep-Kriterium darf nicht 'String kommt nicht mehr vor' lauten, wenn legitime Bezeichner denselben String tragen; das pruefbare Kriterium ist 'kein Dokument beschreibt es als aktuelles Schema'"
    - "Eine Doku-Schemaaussage wird gegen PRAGMA table_info der echten DB abgeglichen, nicht gegen den Migrationstext allein — der Migrationstext sagt nicht, in welcher Reihenfolge die Spalten am Ende stehen"

key-files:
  created: []
  modified:
    - AUTHENTICATION.md

decisions:
  - "D-61-10a: AUTHENTICATION.md bleibt bestehen und wird nicht durch docs/architecture/04-auth.md ersetzt (die zweite im Review angebotene Option). Die Datei traegt OIDC-Setup, Middleware-Reihenfolge, curl-Beispiele und Troubleshooting, das die kanonischen Dokumente nicht fuehren. Stattdessen legt ein Verweis-Banner die Vorrangregel fest."
  - "D-61-10b: Es wird kein AUTHENTICATION_de.md angelegt. `ls AUTHENTICATION*` listet genau eine Datei; die EN/DE-Regel aus CLAUDE.md gilt fuer bestehende Paare, eine neue 600-Zeilen-Uebersetzung waere neuer Doku-Scope statt Gap-Closure."
  - "D-61-10c: Das Sweep-Kriterium aus dem Plan (`wc -l` == 0) ist sachlich nicht erreichbar und wurde durch ein tragfaehiges ersetzt — siehe Abweichung 1."
  - "Der alte Tabellenname kommt im neuen Text nirgends woertlich vor (0 Treffer in AUTHENTICATION.md), damit die Pruefung sich nicht selbst invalidiert (Muster aus 61-05, Deviation 1 + 2)."
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 13 — kein persistierter value_type beruehrt, kein Code angefasst."
  - "Docs-Freshness-Gate: keine Trigger-Datei beruehrt (kein main.rs, kein service/**, kein dao/**, keine Migration, kein permission.rs). Die Aenderung ist die Gegenrichtung — eine fruehere Migration zieht hier ihre Doku nach."

metrics:
  duration: "~30 min"
  completed: 2026-09-17
  tasks: 2
  files-modified: 1

status: complete
---

# Phase 61 Plan 10: Gap Closure WR-06 Summary

`AUTHENTICATION.md` fuehrte die in Phase 61 aufgeloeste 1:1-Verknuepfungstabelle
weiter als `CREATE TABLE` und zeigte `user` ohne `person_id`; der Schema-Abschnitt
beschreibt jetzt das, was die Migrationskette tatsaechlich hinterlaesst, und ein
Verweis-Banner legt fest, dass bei Widerspruch die zweisprachig gepflegten
Dokumente unter `docs/architecture/` gewinnen.

## Was gemacht wurde

**Task 1 — Schema-Abschnitt auf den Stand nach Phase 61.**

- `CREATE TABLE user` zeigt jetzt `person_id blob(16)` mit einem Kommentar, der
  drei Dinge festhaelt: die Spalte kam per `ALTER TABLE ... ADD COLUMN` in
  `20260916000000_create-person-backfill.sql` dazu (deshalb steht sie in der
  echten Datenbank am Ende), sie traegt keine `FOREIGN KEY`-Deklaration (SQLite
  kann bei `ADD COLUMN` keine setzen), und die 1:1-Garantie kommt aus dem
  separaten vollstaendigen Unique-Index `user_person_id_unique_idx`.
- `CREATE TABLE person` ist neu im Abschnitt, mit den vier Spalten woertlich aus
  der Migration (`id blob(16) NOT NULL PRIMARY KEY`, `created TEXT NOT NULL`,
  `update_process TEXT NOT NULL`, `update_version blob(16) NOT NULL`) und der
  Einordnung nach D-60-04: reine Identitaet plus Audit-Spalten, keine Namens-
  und keine `deleted`-Spalte.
- Beide vollstaendigen Unique-Indizes (`sales_person_person_id_unique_idx`,
  `user_person_id_unique_idx`) stehen mit dem D-60-01-Hinweis dabei, dass sie
  bewusst nicht partiell sind.
- Der Block der frueheren Verknuepfungstabelle ist entfernt und durch den
  Abschnitt „Linking a login to its sales-person record" ersetzt: bis v2.7 eine
  eigene 1:1-Tabelle, seit Phase 61 `person_id` auf beiden Seiten, aufgeloest
  durch `migrations/sqlite/20260917000000_drop-sales-person-user.sql`, keine
  Down-Migration, Wiederherstellungspfad ist die Dateikopie vor dem Deploy.
  Verweis auf `docs/architecture/03-data-model.md` fuer die Details.
- Querverweis auf
  `docs/architecture/04-auth.md#self-access-gate-for-sales-person-data` mit der
  Ein-Satz-Begruendung (SQL-Join statt `Option`-Vergleich; `NULL = NULL` ist in
  SQL nie wahr, `None == None` in Rust schon — ein Options-Vergleich waere
  fail-open). Die kanonische Fassung bleibt in `04-auth.md`.
- Verweis-Banner direkt unter der Ueberschrift mit der Vorrangregel.

**Task 2 — Login-Auto-Anlage und Bestandsaufnahme.**

- Der `create_admin_user`-Codeblock wich von `shifty_bin/src/main.rs:1635-1664`
  ab und wurde angeglichen (Details unter „Abgleiche").
- Darunter neu: `PermissionDaoImpl::create_user` oeffnet seit Phase 61 eine
  methodeninterne Transaktion und schreibt erst die `person`-Zeile, dann die
  `user`-Zeile mit deren `person_id` (D-61-01); beide committen zusammen oder
  gar nicht. Ohne diese Vergabe fiele jeder nach dem Deploy angelegte Login aus
  der person-basierten Selbst-Zugriffs-Pruefung — der Phase-60-Backfill lief
  einmalig ueber den Bestand. Gilt fuer `DEVUSER` und `admin` ebenso wie fuer
  `POST /api/permission/user` (derselbe DAO-Pfad, verifiziert in
  `service_impl/src/permission.rs:121-133`). Ergaenzt um den Hinweis, dass die
  beiden Start-Logins ihre Identitaet ueber einen Dev-Reset behalten, weil
  `clear_all` nur traegerlose `person`-Zeilen loescht (Plan 61-08).

Alles auf Englisch, im Stil der umgebenden Datei.

## Abgleiche (durchgefuehrt, Ergebnis woertlich)

**1. Doku-Schema gegen die echte Datenbank** — `PRAGMA table_info` gegen
`localdb.sqlite3`:

```
user:    0|name|TEXT|1||1   1|update_timestamp|TEXT|0||0
         2|update_process|TEXT|1||0   3|person_id|blob(16)|0||0
person:  0|id|blob(16)|1||1   1|created|TEXT|1||0
         2|update_process|TEXT|1||0   3|update_version|blob(16)|1||0
```

Deckt sich mit dem dokumentierten Spaltensatz. `person_id` ist tatsaechlich die
letzte Spalte von `user` und nullable (`notnull = 0`) — genau so steht es jetzt
in der Doku. Die beiden Indizes existieren mit den dokumentierten Namen
(`sales_person_person_id_unique_idx`, `user_person_id_unique_idx`), und
`SELECT count(*) FROM sqlite_master WHERE name = '<alte Tabelle>'` liefert `0` —
die Tabelle ist in der Dev-DB wirklich weg.

**2. `create_admin_user`-Codeblock gegen `shifty_bin/src/main.rs`** — der Block
wich an drei Stellen ab und wurde angeglichen:

| Doku vorher | Code heute |
|---|---|
| `.expect(&format!("Expected being able to create the {}", username))` | `.unwrap_or_else(\|_\| panic!("Expected being able to create the {}", username))` |
| `.expect(&format!("Expected being able to make {} an admin", username))` | `.unwrap_or_else(\|_\| panic!(...))` |
| kein `use dao::PermissionDao;`, kein Einleitungskommentar | beides vorhanden |

Die beiden Aufrufe (`DEVUSER`, `admin`) stimmten und blieben unveraendert. Der
Abgleich stand als Abnahmekriterium ausdruecklich im Plan — er hat hier zu einer
Aenderung gefuehrt, nicht zu einem No-Op.

**3. Deutsche Entsprechung** — `ls AUTHENTICATION*` listet genau eine Datei:
`AUTHENTICATION.md`. Es gibt kein `AUTHENTICATION_de.md`. Die EN/DE-Regel aus
`CLAUDE.md` greift damit nicht, und es wurde bewusst keine deutsche Fassung
angelegt (D-61-10b): eine neue Uebersetzung von ~660 Zeilen waere neuer
Doku-Scope, nicht Gap-Closure. Das ist eine begruendete Feststellung, kein
Versaeumnis.

**4. Repo-Sweep** — siehe eigener Abschnitt unten.

## Repo-Sweep: Ergebnis und Einordnung

`grep -rn "sales_person_user" --include="*.md" --include="*.rs" .` ohne
`.planning/` liefert **58 Treffer in 7 Dateigruppen** — nicht `0`, wie der Plan
erwartete. Jede Gruppe wurde einzeln angesehen:

| Fundstelle | Treffer | Einordnung |
|---|---|---|
| `AUTHENTICATION.md` | **0** (vorher 1) | Ziel dieses Plans — erledigt |
| `shifty_bin/src/person_link_migration_test.rs` | 24 | **Absicht.** Der Test faehrt die Migrationskette bis *vor* die DROP-Migration und prueft die Phase-60-Invarianten gegen die damals noch bestehende Tabelle. Ohne den Namen waere der Nachweis nicht herstellbar; die Begruendung steht im Modul-Doc-Kommentar (P-61-C). |
| `rest/src/sales_person.rs` | 9 | **Bezeichner, kein Tabellenverweis.** Handler- und Routen-Namen `get_/set_/delete_sales_person_user` fuer `/{id}/user`. |
| `shifty-dioxus/src/{page,service}/user_management.rs` | 6 | **Bezeichner.** Store-Feld `sales_person_user_links` und Loader `load_all_sales_person_user_links`. |
| `shifty-dioxus/openspec/**` | 17 | Dieselben Bezeichner in den Specs der stillgelegten OpenSpec-Ablage. |
| `openspec/changes/archive/2026-04-28-dev-seed-endpoint/**` | 1 | Archiv der stillgelegten OpenSpec-Ablage. |
| `openspec/specs/dev-seed/spec.md:79` | 1 | **Echter Drift-Befund, ausserhalb dieses Gap-Scopes** — siehe „Weiterhin offen". |
| `migrations/sqlite/*.sql` | — | Bewusst nicht im Sweep. sqlx fuehrt eine Checksumme je Migrationsdatei; eine nachtraegliche Textaenderung wuerde den Start jeder Bestands-Datenbank blockieren (T-61-37, disposition `accept`). |

Die tragfaehige Aussage ist damit: **kein Dokument im Repository beschreibt die
aufgeloeste Tabelle als aktuelles Schema** — mit der einen benannten Ausnahme in
der stillgelegten OpenSpec-Ablage.

## Abweichungen vom Plan

### 1. [Regel 1 — falsche Annahme im Abnahmekriterium] Sweep-Kriterium ersetzt

- **Gefunden bei:** Task 2
- **Problem:** Der Plan forderte
  `grep -rn "sales_person_user" --include="*.md" --include="*.rs" . | grep -v "^./.planning" | wc -l` → `0`.
  Das ist aus zwei Gruenden nicht erreichbar: (a) der Filter `^./.planning`
  greift nicht, weil `grep -r` die Pfade hier ohne `./`-Praefix ausgibt — der
  im Plan notierte Erwartungswert wurde offenbar nie gegen die Realitaet
  gefahren; (b) auch mit korrektem Filter bleiben 58 legitime Treffer stehen
  (Bezeichner, Migrations-Historientest, stillgelegte OpenSpec-Ablage).
- **Fix:** Kriterium ersetzt durch „kein Dokument beschreibt die Tabelle als
  aktuelles Schema", jede verbliebene Fundstelle einzeln eingeordnet (Tabelle
  oben). Kein Bezeichner wurde umbenannt — das waere eine Code-Aenderung in
  einem reinen Doku-Plan und haette den Sweep zum Selbstzweck gemacht.
- **Dateien:** keine zusaetzlichen

### 2. [Regel 2 — fehlende korrekte Angabe] Codeblock war veraltet

- **Gefunden bei:** Task 2
- **Problem:** Der Plan sagte „wenn er stimmt, bleibt er, wie er ist". Er stimmte
  nicht (siehe Abgleich 2) — `.expect(&format!(...))` ist im Code laengst
  `.unwrap_or_else(|_| panic!(...))`, vermutlich als Clippy-Folge
  (`expect_fun_call`).
- **Fix:** Block woertlich an `shifty_bin/src/main.rs:1635-1664` angeglichen.
- **Dateien:** `AUTHENTICATION.md`

### 3. [Scope-Grenze eingehalten] Drei weitere Drift-Befunde nicht mitrepariert

Beim Sweep gefunden, aber ausserhalb `files_modified: [AUTHENTICATION.md]` —
in `.planning/STATE.md` unter „Doku-Drift-Befunde aus dem Phase-61-Sweep"
eingetragen statt still behoben oder still uebergangen. Details unten.

## Weiterhin offen (bewusst, nicht uebersehen)

1. **arc42 D3, zweiter Drift-Befund zu `AUTHENTICATION.md`** — die Datei
   beschreibt weiter einen `MockContext`-Context-Typ hinter
   `#[cfg(feature = "mock_auth")]`. Verifiziert: `MockContext` existiert im Code
   nicht mehr (`grep -rn "MockContext" --include="*.rs" .` → 0 Treffer);
   `rest/src/session.rs:12` fuehrt `pub type Context = Option<Arc<str>>` **ohne**
   `cfg`-Unterscheidung, und `service_impl/src/lib.rs:64` verwendet denselben
   Typ. Der Befund war im Plan ausdruecklich als nicht in Scope gefuehrt
   (`61-10-PLAN.md` §Deferrals) und bleibt offen; `docs/arc42/11-risks-and-technical-debt.md`
   Zeile D3 fuehrt ihn weiter. **Er ist nicht stillschweigend mitrepariert und
   nicht stillschweigend ignoriert worden.**
2. **`openspec/specs/dev-seed/spec.md:79`** behauptet, der Dev-Seed-Reset loesche
   Zeilen aus der aufgeloesten Verknuepfungstabelle. Seit Phase 61 stimmt das
   nicht mehr (`BasicDaoImpl::clear_all` kennt sie nicht, die Tabelle existiert
   nicht). Der ganze `openspec/`-Baum ist laut arc42 D3 ohnehin als stillgelegtes
   Artefakt zum Loeschen oder Als-Archiv-Markieren vorgemerkt — eine Reparatur
   waere Scope-Ausweitung in eine tote Ablage.
3. **ROADMAP-Progress-Tabelle, Zeile Phase 60** steht weiter auf
   `0/? | Not started`, obwohl die Phase abgeschlossen ist. Vorbestehende Drift,
   nicht durch diese Welle verursacht; nur die Zeile fuer Phase 61 wurde
   angefasst.
4. **WR-02** geht an Phase 62; **WR-01, WR-03, WR-07** und **IN-01 bis IN-07**
   gehen ins Backlog (Deferral Record in `61-07-PLAN.md`). **CR-02
   Pre-Flight-Query** bleibt Deploy-Gate.

## Gates

| Gate | Ergebnis |
|---|---|
| `cargo test --workspace` | **0 failed** (1082 passed, 3 ignored — Doc-Tests) |
| `cargo clippy --workspace --all-targets -- -D warnings` | **Exit-Code 0** |
| `git diff --name-only` (Task-Commit) | nur `AUTHENTICATION.md` |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION` | unveraendert **13** (`service_impl/src/billing_period_report.rs:140`) |

Die Cargo-Gates sind hier Regressionsnachweis, nicht Zielnachweis — der Plan
aendert keinen Rust-Code. Sie liefen trotzdem, weil dies der letzte Plan der
Phase ist und eine Welle nicht mit rotem Workspace endet.

## Abnahmekriterien

| Kriterium | Ergebnis |
|---|---|
| `person_id` in `AUTHENTICATION.md` | 10 Treffer |
| `CREATE TABLE person` mit vier Spalten | vorhanden |
| `user_person_id_unique_idx` | 2 Treffer |
| `20260917000000_drop-sales-person-user.sql` | 1 Treffer |
| `04-auth.md#self-access-gate-for-sales-person-data` | 1 Treffer |
| `docs/architecture/03-data-model.md` | 2 Treffer |
| alter Tabellenname in `AUTHENTICATION.md` | **0** Treffer |
| Spaltensatz gegen `PRAGMA table_info` | deckungsgleich |
| `ls AUTHENTICATION*` | genau eine Datei |

## Self-Check: PASSED

- `AUTHENTICATION.md` — FOUND
- `.planning/phases/61-personservice-aufl-sung-von-sales-person-user/61-10-SUMMARY.md` — FOUND
- Commit `dc43e472` — FOUND (`docs(61-10): AUTHENTICATION.md auf den Schema-Stand nach Phase 61 bringen`)
