---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 01
subsystem: auth / dao
tags: [transaction, permission-dao, refactor, atomicity, person]
status: complete
requires:
  - "dao::Transaction / dao::TransactionDao (bestehendes Arc::into_inner-Commit-Muster)"
  - "dao::person::PersonDao (fuer den Person-Reuse-Nachweis)"
provides:
  - "dao::PermissionDao::Transaction — Assoziationstyp auf dem DAO-Trait"
  - "PermissionDao::create_user(user, person_id: Option<Uuid>, process, tx)"
  - "PermissionDao::delete_user(username, tx)"
  - "PermissionServiceDeps::TransactionDao"
affects:
  - "63-05 (kombinierter Anlege-Pfad: Benutzer + Verkaeufer auf DIESELBE Person)"
  - "63-0x Orphan-Cleanup der person-Zeile (D-63-01)"
tech_stack:
  added: []
  patterns:
    - "Gemischtes DAO-Trait: nur die zwei von Phase 63 orchestrierten Schreibpfade nehmen die geteilte Transaktion, alle uebrigen Methoden bleiben pool-basiert"
    - "Transaktions-Assoziationstyp auf dem DAO- statt auf dem Service-Trait, um den workspace-weiten Ripple ueber ~20 PermissionService-Konsumenten zu vermeiden"
key_files:
  created: []
  modified:
    - dao/src/permission.rs
    - dao_impl_sqlite/src/lib.rs
    - service_impl/src/permission.rs
    - service_impl/src/user_invitation.rs
    - service_impl/src/test/permission_test.rs
    - service_impl/src/test/person_creation.rs
    - shifty_bin/src/main.rs
    - shifty_bin/src/integration_test/person_directory.rs
    - shifty_bin/src/integration_test/impersonation_audit.rs
decisions:
  - "Der Transaktions-Assoziationstyp sitzt auf dao::PermissionDao, nicht auf service::PermissionService — das Service-Trait bleibt eingefroren (0 Treffer fuer 'type Transaction')."
  - "PermissionDao bleibt bewusst ein gemischtes Trait: nur create_user und delete_user werden transaktionsfaehig, keine Vorratshaltung fuer die uebrigen 15 Methoden."
  - "PermissionServiceImpl holt sich die Transaktion intern und committet sie selbst — das Admin-Gate bleibt die erste Anweisung, vor use_transaction (T-63-01-B)."
  - "N-8 (create_user_ist_atomar) wurde umgeschrieben statt geloescht: der Atomaritaets-Nachweis wandert eine Ebene hoeher zur aufrufenden Transaktion."
metrics:
  duration_minutes: 38
  tasks_completed: 3
  files_modified: 9
  tests_added: 3
  completed: 2026-09-20
---

# Phase 63 Plan 01: PermissionDao auf die geteilte Transaktion heben — Summary

`PermissionDao::create_user`/`delete_user` nehmen jetzt die Transaktion des Aufrufers
statt einer unsichtbaren lokalen `pool.begin()`, und `create_user` kann einen
vorgegebenen `person_id` wiederverwenden — damit ist die SC-2-Atomaritaet und der
D-63-01-Orphan-Cleanup fuer Phase 63 ueberhaupt erst konstruierbar.

## Was gebaut wurde

**`dao/src/permission.rs`** — `PermissionDao` traegt `type Transaction: crate::Transaction`
(`#[automock(type Transaction = crate::MockTransaction;)]`). Die beiden Schreibpfade
haben neue Signaturen:

- `create_user(&self, user: &UserEntity, person_id: Option<Uuid>, process: &str, tx: Self::Transaction)`
- `delete_user(&self, username: &str, tx: Self::Transaction)`

Alle uebrigen 15 Methoden bleiben unveraendert pool-basiert. Die Begruendung fuer das
bewusst gemischte Trait und fuer die Platzierung des Assoziationstyps auf der DAO-
statt der Service-Ebene steht als Doc-Kommentar direkt ueber `type Transaction`.

**`dao_impl_sqlite/src/lib.rs`** — `impl dao::PermissionDao for PermissionDaoImpl` hat
`type Transaction = TransactionImpl`. In `create_user` sind `self.pool.begin()` und
`tx.commit()` entfallen; beide `query!`-Aufrufe laufen auf `tx.tx.lock().await.as_mut()`.
Der `person_id`-Parameter steuert per `match`, ob das `INSERT INTO person` ueberhaupt
ausgefuehrt wird. `delete_user` schreibt ebenfalls auf die uebergebene Transaktion statt
direkt auf den Pool. Der D-61-01-Kommentarblock ist erhalten und um den Hinweis ergaenzt,
dass die Atomaritaet jetzt von der uebergebenen Transaktion getragen wird.

**`service_impl/src/permission.rs`** — `PermissionServiceImpl` bekommt `TransactionDao`
als dritte Dependency; `create_user`/`delete_user` holen sich per `use_transaction(None)`
eine Transaktion und committen sie selbst. Das Aussenverhalten von
`service::PermissionService` ist unveraendert — kein `type Transaction`, kein
`tx`-Parameter, keine Anpassung bei den rund zwanzig Konsumenten.

**`service_impl/src/user_invitation.rs`** — reicht seine bereits offene `tx` an
`create_user` durch. Die Einloesung einer Einladung ist damit wieder wirklich atomar;
vorher committete `create_user` intern mitten in der Einloesung.

**`shifty_bin/src/main.rs`** — `PermissionServiceDependencies` bekommt
`type TransactionDao = TransactionDao`, die Konstruktion `transaction_dao.clone()`
(die Reihenfolge passte bereits). `create_admin_user` baut sich einen
`TransactionDaoImpl`, holt eine Transaktion, legt `DEVUSER` an und committet;
`add_user_role` bleibt pool-basiert.

**Tests** — `service_impl/src/test/permission_test.rs` bekommt einen
`MockTransactionDao` in den Deps; die Erwartungen fuer `create_user`/`delete_user`
sind auf die neuen Signaturen angepasst. Drei neue Nachweise in
`service_impl/src/test/person_creation.rs` (siehe unten).

## Key Decisions

| Entscheidung | Warum |
|---|---|
| `type Transaction` auf `dao::PermissionDao`, nicht auf `service::PermissionService` | Das Service-Trait fuehren rund zwanzig Services als Dependency; ein Assoziationstyp dort waere ein workspace-weiter Ripple ohne Gegenwert. Auf DAO-Ebene sind es zwei Konsumenten plus Test-/Bootstrap-Aufrufstellen. |
| Gemischtes DAO-Trait (nur 2 von 17 Methoden transaktionsfaehig) | Vorratshaltung ist ausdruecklich unerwuenscht (Research Pitfall 5). Nur die zwei Schreibpfade, die Phase 63 orchestriert, brauchen die geteilte Transaktion. |
| Admin-Gate bleibt vor `use_transaction` | T-63-01-B: der Refactor darf die Reihenfolge nicht umstellen, sonst oeffnet ein nicht-autorisierter Aufruf eine DB-Transaktion. |
| N-8 umgeschrieben statt entfernt | Der Atomaritaets-Nachweis war an die methodeninterne Transaktion gebunden. Er wandert eine Ebene hoeher: der fehlgeschlagene Aufruf wird vom Aufrufer verworfen und darf nichts zuruecklassen. Semantik identisch, Besitzer verschoben. |

## Neue Tests (Task 3)

| Test | Traegt | Was er beweist |
|---|---|---|
| `create_user_rollback_leaves_no_row` (N-11) | SC-2 | Rollback der aufrufenden Transaktion nimmt **beide** Zeilen mit — `user` UND `person`. Ein Rueckfall auf `pool.begin()` waere nur an der `person`-Seite sichtbar, deshalb pruefen beide Assertions. |
| `delete_user_rollback_leaves_row` (N-12) | SC-2 / D-63-01 | Rollback nimmt das `DELETE FROM user` zurueck. Vorher schrieb `delete_user` direkt auf den Pool und war unwiderruflich, bevor der Orphan-Cleanup ueberhaupt lief. |
| `create_user_with_given_person_id_reuses_it` (N-13) | Voraussetzung fuer den kombinierten Anlege-Pfad | Der Benutzer traegt genau die uebergebene Person **und** die Gesamtzahl der `person`-Zeilen bleibt unveraendert — nur die zweite Zusicherung schliesst eine still zusaetzlich angelegte Person aus. |

`cargo test -p service_impl person_creation` faehrt jetzt 13 statt 10 Tests.

## Deviations from Plan

**1. [Rule 3 - Blocking] N-8 `create_user_ist_atomar` musste umgeschrieben werden**

- **Gefunden bei:** Task 2
- **Problem:** Der Test bewies die Atomaritaet ueber die *methodeninterne* Transaktion
  (zweiter `create_user` mit demselben Namen scheitert am Primaerschluessel, `person`-Anzahl
  bleibt unveraendert). Nach dem Refactor gibt es diese interne Transaktion nicht mehr;
  SQLite bricht bei einem fehlgeschlagenen Statement die umgebende Transaktion nicht
  automatisch ab, also haette der Test die jetzt beim Aufrufer liegende Verantwortung
  nicht mehr getroffen.
- **Fix:** Erste Anlage committen, zweite auf einer eigenen Transaktion ausfuehren, Fehler
  erwarten, Transaktion rollbacken, danach dieselben Assertions. Der Nachweis ist derselbe,
  er wird nur eine Ebene hoeher gefuehrt — im Test und im Modulkopf dokumentiert.
- **Datei:** `service_impl/src/test/person_creation.rs`
- **Commit:** abf9e5f0

**2. [Prozess] Task 1 und Task 2 wurden gemeinsam verifiziert, aber getrennt committet**

Task 1 aendert Trait-Signaturen; der Workspace kompiliert erst nach Task 2 wieder. Um
das CLAUDE.md-Gate (`cargo build` / `cargo test` / `cargo clippy --workspace -- -D warnings`)
nicht mit einem nicht uebersetzbaren Zwischenstand zu verletzen, wurden beide Tasks
implementiert, gemeinsam verifiziert und danach in zwei Commits getrennt. Commit
`48d9027a` ist damit ein reiner Signatur-Zwischenstand und baut allein nicht — die
scoped Akzeptanz aus dem Plan (`cargo build -p dao -p dao_impl_sqlite`) war zum
Commit-Zeitpunkt gruen.

## Auth Gates

Keine.

## Threat Flags

Keine neue Angriffsflaeche. `T-63-01-B` (Elevation of Privilege) bleibt mitigiert: das
`check_permission("admin", context)`-Gate ist in beiden Methoden weiterhin die erste
Anweisung, vor `use_transaction` und vor jedem DB-Zugriff.

## Known Stubs

Keine.

## Docs-Freshness-Gate

Dieser Plan beruehrt drei Trigger-Dateien der CLAUDE.md-Tabelle:

| Beruehrt | → faellig |
|---|---|
| `dao/src/permission.rs` (DAO-Trait-Signaturen) | `docs/features/F12-auth-session.md` (+ `_de`) — Datenmodell-/Schreibpfad-Sektion |
| `service_impl/src/permission.rs` (Auth) | `docs/architecture/04-auth.md` + `docs/features/F12-auth-session.md` (beide Sprachen) |
| `shifty_bin/src/main.rs` (DI-Verdrahtung) | `docs/architecture/02-service-tiers.md` + `docs/architecture/diagrams/service-graph-runtime.mmd` |

**Nicht in diesem Plan erledigt** — Plan 63-10 fuehrt die Doku-Aktualisierung der Phase
zentral. Die konkrete Aenderung: `PermissionServiceImpl` hat jetzt drei statt zwei
Dependencies (neu: `TransactionDao`), und `PermissionDao` ist ein gemischtes Trait mit
zwei transaktionsfaehigen Schreibpfaden.

## Deferred Issues

Siehe `deferred-items.md`: `service_impl/src/test/person_directory_dao.rs:149` loest unter
`cargo clippy --all-targets` eine `needless_lifetimes`-Warnung aus. Pre-existing aus
Phase 62, ausserhalb des Scopes. Das in CLAUDE.md festgelegte Gate
`cargo clippy --workspace -- -D warnings` ist gruen.

## TDD Gate Compliance

Der Plan ist als `type: tdd` gefuehrt. Die Gate-Sequenz ist **nicht** in der Reinform
RED→GREEN gelaufen, und zwar aus einem strukturellen Grund: Tasks 1 und 2 sind ein
Signatur-Refactor an bestehendem, bereits getestetem Verhalten — ein RED-Test haette
sich gegen eine Signatur richten muessen, die es noch nicht gab, also nicht kompiliert.
Der `test(...)`-Commit (`d1995e32`) liegt darum **nach** den beiden `refactor(...)`-Commits.

Was den Verlust an Beweiskraft abfaengt: N-11 und N-12 wuerden gegen den Zustand vor
diesem Plan tatsaechlich fehlschlagen (die lokale `pool.begin()` bzw. der Pool-Schreibpfad
ueberleben ein Rollback), sind also keine tautologisch gruenen Tests. Die bestehende
Suite (904 `service_impl`-Tests, 95 Integrationstests) war waehrend des gesamten
Refactors die Regressionsschranke.

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo build --workspace --all-targets` | gruen |
| `cargo test --workspace` | gruen, 0 Failures (904 + 95 + uebrige Suites) |
| `cargo clippy --workspace -- -D warnings` | gruen |
| `cargo build -p shifty_bin` | gruen |
| `cargo test -p service_impl person_creation` | 13 passed (vorher 10) |
| `cargo test -p shifty_bin devuser_admin_holds_every_privilege` | 1 passed — der Dev-Admin-Bootstrap laeuft ueber den geaenderten Pfad |
| `grep -c 'type Transaction' service/src/permission.rs` | 0 — Service-Trait eingefroren |
| `grep -c 'type Transaction' dao/src/permission.rs` | 2 |
| `pool.begin` in `create_user` | 0 |

## Self-Check: PASSED

Alle in `key_files.modified` genannten Dateien existieren, alle drei Commits sind in
`git log` auffindbar (`48d9027a`, `abf9e5f0`, `d1995e32`).

## Commits

| Task | Commit | Typ |
|---|---|---|
| 1 | `48d9027a` | `refactor(63-01): PermissionDao auf die geteilte Transaktion heben` |
| 2 | `abf9e5f0` | `refactor(63-01): Aufrufstellen nachziehen, PermissionService bekommt TransactionDao` |
| 3 | `d1995e32` | `test(63-01): Rollback- und Person-Reuse-Nachweise fuer create_user/delete_user` |
