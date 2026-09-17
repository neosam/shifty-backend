---
phase: 61-personservice-aufl-sung-von-sales-person-user
plan: 09
subsystem: service
tags: [gap-closure, review-wr-04, http-semantics, error-translation, openapi, docs-freshness-gate]

# Dependency graph
requires:
  - phase: 61-04
    provides: "die chirurgische Uebersetzung von DaoError::EntityAlreadyExists nach ServiceError::EntityAlreadyExists im Merge-Zweig von set_user, inklusive ihres Begruendungskommentars — hier woertliches Formatvorbild und jetzt gemeinsamer Block fuer beide Schreibpfade"
  - phase: 61-04
    provides: "discard_assigned_user mit dem rows_affected() != 1-Zweig, der DaoError::EntityAlreadyExists liefert"
provides:
  - "set_user uebersetzt den Split-Konflikt genauso wie den Merge-Konflikt: DELETE /sales-person/{id}/user meldet einen Race-Verlierer als 409 statt als 500"
  - "Zwei Mock-Tests, die die Uebersetzung und ihre Grenze pinnen (set_user_none_uebersetzt_split_konflikt_nach_409, set_user_none_laesst_andere_dao_fehler_als_serverfehler_durch)"
  - "utoipa-Annotation von delete_sales_person_user fuehrt 409"
  - "F01-Statustabelle (EN + DE) fuehrt 409 fuer DELETE /sales-person/{id}/user"
affects: [62-personenverwaltung-liste, 63-personen-detail]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wenn zwei Schreibpfade derselben Service-Methode denselben DaoError erzeugen koennen, gehoert die Uebersetzung VOR beide statt in einen von beiden — ein blankes `?` auf dem einen Pfad macht denselben Zustand auf dem einen Endpoint zum Konflikt und auf dem anderen zum Serverdefekt"
    - "Jede chirurgische Fehleruebersetzung braucht einen Gegentest: ein anderer DaoError muss nachweislich Serverfehler bleiben, sonst tarnt der Fix echte Defekte als fachliche Konflikte"
    - "Ein DAO-Zweig, der nur im Rennen erreichbar ist, wird auf Service-Ebene gemockt statt integriert — Vorab-Lesung und UPDATE laufen in derselben Transaktion, ein Nebenlaeufer kommt nicht dazwischen; der Mock stellt genau den Rueckgabewert her, den der echte Zweig liefert"
    - "Eine geaenderte utoipa-responses-Liste zieht die Statustabelle in docs/features/F*.md nach — beide Sprachen, sonst ist es Drift"

key-files:
  created: []
  modified:
    - service_impl/src/sales_person.rs
    - service_impl/src/test/sales_person.rs
    - rest/src/sales_person.rs
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md

decisions:
  - "Die Variante bleibt auch fuer den Split EntityAlreadyExists. Der Review fragte, ob sie semantisch passt (der Zustand heisst dort eher 'der Benutzer ist zwischenzeitlich weggelaufen'). Antwort: ja — 409 ist fuer einen Race-Verlierer der richtige Status, derselbe, den der Merge-Pfad liefert; eine eigene Variante waere eine Vertragserweiterung ausserhalb des Gap-Scopes. Die Begruendung steht als Satz im Quellkommentar, damit die Frage nicht in jeder Review neu gestellt wird."
  - "Ein Uebersetzungsblock statt zwei: der bestehende Begruendungskommentar wurde aus dem if-let-Rumpf vor beide match-Bloecke gezogen und um den Split-Fall erweitert. Reihenfolge und Transaktionsbehandlung sind unveraendert — insbesondere bleibt der fehlende Commit im Konfliktfall die Rollback-Zusage P-61-B."
  - "Mock statt Integration fuer beide neuen Tests, mit der Begruendung im Doc-Kommentar verankert — sonst liest die naechste Welle das als Abweichung vom Integrations-Stil der Person-Testmodule und 'repariert' es."
  - "WR-01 bleibt aufgeschoben: dass assign_to_user 'nicht gefunden' und einen Invariantenbruch beide als 409 meldet, wurde bewusst NICHT angefasst. Die 404-Zeile in der F01-Tabelle bleibt deshalb stehen."
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 13 — kein persistierter value_type beruehrt."
  - "Docs-Freshness-Gate: F01 (EN + DE) mitgezogen, weil die Statustabelle den erreichbaren 409 fuehren muss. Kein Trigger fuer 02-service-tiers (keine DI-Aenderung), 03-data-model (kein Schema), 04-auth (kein Permission-Pfad)."

metrics:
  duration: "~25 min"
  completed: 2026-09-17
  tasks: 2
  files-modified: 5

status: complete
---

# Phase 61 Plan 09: Gap Closure WR-04 Summary

`DELETE /sales-person/{id}/user` meldete einen Verknuepfungs-Konflikt als HTTP 500,
waehrend `POST /sales-person/{id}/user` dieselbe DAO-Bedingung als HTTP 409 meldete;
jetzt laufen Split und Merge durch denselben Uebersetzungsblock, und die Grenze der
Uebersetzung ist getestet.

## Was war das Problem

`SalesPersonDaoImpl::discard_assigned_user` liefert bei `rows_affected() != 1`
— dem Race-Verlierer beim Loesen einer Verknuepfung — `DaoError::EntityAlreadyExists`.
Im Service lief genau dieser Aufruf ueber ein blankes `?`:

```rust
self.sales_person_dao
    .discard_assigned_user(sales_person_id, tx.clone())
    .await?;
```

`ServiceError` konvertiert `DaoError` per `#[from]` pauschal nach
`DatabaseQueryError`, der zentrale Error-Handler in `rest/src/lib.rs` macht daraus
HTTP 500. Zehn Zeilen weiter wurde derselbe `DaoError` aus `assign_to_user` gezielt
nach `ServiceError::EntityAlreadyExists` → HTTP 409 uebersetzt, mit einem Kommentar,
dessen Begruendung fuer den Split wortgleich gilt.

Die Aussenwirkung: `DELETE /sales-person/{id}/user` ruft `set_user(id, None, ...)`
und faehrt damit **ausschliesslich** den Split. Ein Konflikt auf diesem Endpoint
erschien also garantiert als Serverdefekt — ein HR-Client konnte nicht erkennen,
dass er einen fachlichen Zustand vor sich hat, und wurde zum blinden Wiederholen
eingeladen (T-61-32).

## Was gemacht wurde

### Task 1 — RED (nicht committet)

Zwei Mock-Tests ans Ende von `service_impl/src/test/sales_person.rs`, unter einer
Abschnittsueberschrift, die WR-04 als Herkunft nennt. Die Datei hatte bis hierher
**keinen** einzigen `set_user`-Test.

- `set_user_none_uebersetzt_split_konflikt_nach_409` — `discard_assigned_user`
  liefert `Err(DaoError::EntityAlreadyExists)`; `expect_assign_to_user` wird bewusst
  **nicht** gesetzt, damit mockall einen versehentlichen Merge-Aufruf selbst meldet.
  Erwartung: `Err(ServiceError::EntityAlreadyExists(default_id()))`.
- `set_user_none_laesst_andere_dao_fehler_als_serverfehler_durch` — derselbe Aufbau
  mit einem anderen `DaoError`; Erwartung: **kein** `EntityAlreadyExists`, sondern
  unveraendert `DatabaseQueryError`. Das ist die Grenze der Uebersetzung und war
  bereits vor dem Fix gruen.

**RED-Nachweis (woertlich):**

```
running 2 tests
test test::sales_person::set_user_none_uebersetzt_split_konflikt_nach_409 ... FAILED
test test::sales_person::set_user_none_laesst_andere_dao_fehler_als_serverfehler_durch ... ok

---- test::sales_person::set_user_none_uebersetzt_split_konflikt_nach_409 stdout ----
panicked at service_impl/src/test/sales_person.rs:887:18:
Split-Konflikt wurde nicht als Konflikt gemeldet: Err(DatabaseQueryError(EntityAlreadyExists)).
Dieselbe DAO-Bedingung darf nicht auf POST /sales-person/{id}/user einen 409 und auf
DELETE /sales-person/{id}/user einen 500 ergeben - DELETE faehrt ausschliesslich den Split (WR-04).

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 887 filtered out
```

Der tatsaechliche Wert war `DatabaseQueryError(EntityAlreadyExists)` — der Befund ist
damit reproduziert, nicht nur behauptet: der Konflikt steckte bereits im Fehler, nur
die Huelle machte ihn zum Serverfehler.

Wie in allen Wellen dieser Phase kein eigenstaendiger RED-Commit — das Gate aus
`CLAUDE.md` verlangt gruene Tests vor jedem Commit.

### Task 2 — GREEN (Commit `302838ff`)

**`service_impl/src/sales_person.rs`:** Der Aufruf von `discard_assigned_user` liegt
jetzt in einem `match` mit denselben drei Armen wie `assign_to_user`. Der
Begruendungskommentar wurde aus dem `if let Some(user)`-Rumpf **vor beide Bloecke**
gezogen und um zwei Absaetze erweitert: dass die Uebersetzung fuer beide Schreibpfade
gilt (mit der `set_user(id, None, ..)`-Begruendung), und dass `EntityAlreadyExists`
fuer den Split eine bewusste Entscheidung ist und kein Versehen. Reihenfolge,
Transaktionsbehandlung und der fehlende Commit im Konfliktfall (Rollback-Zusage
P-61-B) sind unveraendert.

**`rest/src/sales_person.rs`:** `#[utoipa::path(delete, ...)]` auf
`delete_sales_person_user` fuehrt jetzt
`(status = 409, description = "The link was changed by someone else before it could be removed")`.
Die Beschreibung ist auf den Split-Fall gemuenzt statt die POST-Formulierung zu
kopieren. 204, 404 und 500 stehen unveraendert. Am Handler selbst hat sich nichts
geaendert — das Status-Mapping faellt zentral in `rest/src/lib.rs`.

**`docs/features/F01-employee-management{,_de}.md`:** Die Statustabelle in §5 fuehrte
fuer `DELETE /sales-person/{id}/user` `404/500`; jetzt `404/409/500`. Siehe
Abweichung 2.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] `DaoError::EntityNotFound` existiert nicht**

- **Found during:** Task 1
- **Issue:** Der Plan schlug fuer den Gegentest `DaoError::EntityNotFound` als
  "anderen DaoError" vor. Diese Variante gibt es in `dao/src/lib.rs` nicht — das
  Enum kennt `DatabaseQueryError`, `UuidError`, `InvalidDayOfWeek`,
  `DateTimeParseError`, `DateTimeFormatError`, `ComponentRangeError`,
  `EnumValueNotFound` und `EntityAlreadyExists`. (`EntityNotFound` gibt es nur auf
  der `ServiceError`-Seite.) Der Test haette nicht kompiliert.
- **Fix:** `DaoError::DatabaseQueryError("simulierter Serverfehler im Split".into())`
  verwendet. Das ist semantisch sogar naeher am Befund: geprueft wird, dass ein
  echter Serverfehler ein Serverfehler bleibt.
- **Files modified:** `service_impl/src/test/sales_person.rs`
- **Commit:** `302838ff`

**2. [Rule 2 - Doku-Drift] F01-Statustabelle fuehrte den jetzt erreichbaren 409 nicht**

- **Found during:** Task 2
- **Issue:** Der Plan verlangt die utoipa-Annotation, erwaehnt aber die
  Statustabelle in `docs/features/F01-employee-management.md` §5 nicht. Dort stand
  fuer `DELETE /sales-person/{id}/user` weiterhin `404/500`. Nach diesem Plan ist
  409 erreichbar — die Tabelle waere ab dem Commit nachweislich falsch gewesen, und
  zwar in genau der Zeile, die dieser Plan veraendert.
- **Fix:** Zeile auf `404/409/500` gehoben, in EN und DE synchron. Die `404`-Zeile
  der **POST**-Route blieb ausdruecklich unberuehrt — das ist WR-01s Baustelle.
- **Files modified:** `docs/features/F01-employee-management.md`,
  `docs/features/F01-employee-management_de.md`
- **Commit:** `302838ff`

### Korrigierte Akzeptanzkriterien

**3. `grep -c 'status = 409' rest/src/sales_person.rs` liefert 2 — falsches Gate**

Das Kriterium unterstellt, die Datei habe vor diesem Plan genau ein `status = 409`
gefuehrt (POST) und danach zwei (POST + DELETE). Tatsaechlich fuehrte sie bereits
**zwei**: `PUT /sales-person/{id}` hat seit jeher `(status = 409, description = "Conflict")`
fuer den Versionskonflikt (Zeile 170). Nach diesem Plan sind es **drei**. Das
Kriterium war also nicht erfuellbar, ohne eine korrekte, unbeteiligte Annotation zu
loeschen.

Korrigiertes Kriterium, gegen das verifiziert wurde: *beide `/{id}/user`-Schreib-
Endpoints fuehren einen 409-Ausgang* — nachgewiesen ueber
`grep -n 'status = 409' rest/src/sales_person.rs` → Zeilen 170 (`PUT /{id}`,
vorbestehend), 284 (`POST /{id}/user`), 318 (`DELETE /{id}/user`, neu).

Dies ist das vierte Mal in dieser Phase, dass ein als Quell-Grep formuliertes
Kriterium ein falsches Gate erzeugt hat. Das Muster: der Planer zaehlt ein Vorkommen
aus dem Kopf, ohne die Datei zu zaehlen.

**4. `grep -A 3 'discard_assigned_user' service_impl/src/sales_person.rs` zeigt einen match-Arm-Block**

Der erste Treffer ist seit Task 2 eine **Kommentarzeile** (der erweiterte
Begruendungstext nennt `discard_assigned_user` namentlich), nicht der Aufruf. Der
zweite Treffer zeigt wie gefordert `.await` gefolgt von `{` und `Ok(()) => {}`. Das
Kriterium ist inhaltlich erfuellt; die Grep-Formulierung ist nur nicht mehr
eindeutig, seit der Bezeichner auch im Kommentar vorkommt.

### WR-01 — bewusst nicht angefasst

`assign_to_user` kollabiert weiterhin "Sales Person existiert nicht",
"`person_id IS NULL`" und "Login existiert nicht" alle auf `EntityAlreadyExists`
→ 409, und die F01-Tabelle fuehrt fuer `POST /sales-person/{id}/user` weiterhin ein
404, das der Code nicht mehr erzeugen kann. Das ist WR-01, steht im Deferral Record
von `61-07-PLAN.md` und braeuchte neue `DaoError`-Faelle plus ein geaendertes
HTTP-Mapping auf einem laut SC3 eingefrorenen Endpoint-Vertrag. Beim Arbeiten in
derselben Funktion war die Versuchung real; sie wurde nicht angefasst.

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo test -p service_impl --lib sales_person` | 111 passed, **0 failed** |
| beide neuen Tests (`--lib set_user_none`) | 2 passed, **0 failed** |
| `cargo test -p service_impl --lib person_merge` | 11 passed, **0 failed** — M-6 (Merge-409) und `relink_konflikt_rollt_den_split_zurueck` (P-61-B) unveraendert gruen |
| `cargo test --workspace` | **0 failed** ueber alle Targets (u.a. 889 + 93 + 33) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit-Code **0** |
| `cargo build -p shifty_bin --no-default-features --features oidc,json_logging` | Exit-Code **0** |
| `git status --porcelain .sqlx` | leer — keine SQL-Query geaendert |
| `git diff --stat Cargo.toml Cargo.lock` | leer — kein Package-Install (T-61-SC) |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION` | unveraendert **13** |
| ASCII-only in neuem Quellcode | `grep -cP '[^\x00-\x7F]'` auf den neuen Bloecken → 0 |

## Threat Model Coverage

| Threat ID | Disposition | Umsetzung |
|---|---|---|
| T-61-32 | mitigate | Gezielte Uebersetzung nur fuer `EntityAlreadyExists`, abgesichert durch `set_user_none_laesst_andere_dao_fehler_als_serverfehler_durch` auf die Gegenrichtung. Beide Richtungen sind jetzt getestet. |
| T-61-33 | mitigate | 409 in der `#[utoipa::path(delete, ...)]`-Annotation ergaenzt; zusaetzlich in der F01-Statustabelle (EN + DE), die der Plan uebersehen hatte. |
| T-61-34 | accept | Unveraendert. Die Rollback-Zusage P-61-B haengt weiter daran, dass der Aufrufer `tx = None` uebergibt (WR-07); alle heutigen Aufrufer in `rest/src/sales_person.rs` tun das. Ins Backlog aufgeschoben. |
| T-61-SC | accept | Kein Package-Install; `Cargo.toml`/`Cargo.lock` unveraendert. |

## Known Stubs

Keine.

## Threat Flags

Keine neue sicherheitsrelevante Oberflaeche. Der Plan aendert eine Fehler-Uebersetzung
und eine OpenAPI-Beschreibung; kein neuer Endpoint, kein neuer Auth-Pfad, kein
Schema-Eingriff.

## Commits

| Task | Commit | Inhalt |
|---|---|---|
| 1 (RED) | — | bewusst nicht committet (Gruen-Gate aus `CLAUDE.md`) |
| 2 (GREEN) | `302838ff` | Service-Uebersetzung, beide Tests, utoipa-Annotation, F01 EN + DE |

## Self-Check: PASSED

Alle behaupteten Dateien existieren auf der Platte (`service_impl/src/sales_person.rs`,
`service_impl/src/test/sales_person.rs`, `rest/src/sales_person.rs`,
`docs/features/F01-employee-management{,_de}.md`, dieses SUMMARY), und der genannte
Commit `302838ff` existiert in der History.

## Tooling-Notiz

`gsd-tools query roadmap.update-plan-progress 61` hat `.planning/ROADMAP.md` zum
**vierten Mal in dieser Phase** beschaedigt: die Fortschrittszeile landete in der
Tabelle "Backend-/Frontend-Abdeckung pro Phase" (Zeile 111) und ueberschrieb die
Phase-61-Zeile mit `| 61 | 9/10 | In Progress| |`. Zeile 111 wurde woertlich
wiederhergestellt; die Fortschrittszeile wurde von Hand in die richtige Tabelle
(Zeile 608, "Progress Table") eingetragen. Die vom Tool korrekt gesetzte
Checkbox fuer `61-09-PLAN.md` (Zeile 380) blieb erhalten.
