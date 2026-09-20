---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 04
subsystem: service / rest-types
tags: [person-detail, business-logic-tier, admin-gate, fat-backend, candidates, dto]
status: complete
requires:
  - "dao::person_directory::PersonDirectoryDao::find_directory_entry (63-02)"
  - "dao::PermissionDao::Transaction (63-01) — als Dependency fuer den Schreibpfad in 63-05"
  - "service_impl::person_directory::entry_from_row (Phase 62 — Klassifikation und Anzeigename)"
  - "service::person::PersonService::unlinked_sales_persons / unlinked_users (Phase 61, bis heute ohne Aufrufer)"
provides:
  - "rest_types::CreatePersonEntryTO / CreatedPersonEntryTO / PersonFacetTO / AddPersonFacetTO"
  - "service::person_detail::PersonFacet / CreatePersonEntry (Domain-Typen)"
  - "service::person_detail::PersonDetailService — get_detail, unlinked_sales_person_candidates, unlinked_user_candidates"
  - "service_impl::person_detail::PersonDetailServiceImpl + PersonDetailServiceDeps (7 Dependencies)"
  - "service_impl::person_directory::entry_from_row ist pub(crate)"
affects:
  - "63-05 (Schreibpfad: kombinierter Anlege-Pfad, add_facet, remove_facet — der Deps-Block steht bereits)"
  - "63-06 (REST-Oberflaeche /person: get_detail und die beiden Kandidatenlisten sind die Lesepfade)"
  - "63-0x DI-Verdrahtung in shifty_bin/src/main.rs (noch nicht erfolgt)"
tech_stack:
  added: []
  patterns:
    - "Detail-Lesepfad als Wiederverwendung der Listen-Assembly: entry_from_row wird pub(crate) statt einer zweiten Ableitung (PADM-06)"
    - "Kern-Aggregat plus gezielte Nachladepfade statt Voll-Aggregat — vier Abschnitts-Endpoints bleiben in Betrieb, kein N+1 pro Abschnitt"
    - "Authentication::Full erst NACH dem eigenen, strengeren Admin-Gate an den HR-gegateten Basic-Service durchgereicht"
    - "Deps-Block traegt bereits die vier Schreibpfad-Dependencies, damit die DI-Verdrahtung nur einmal angefasst wird"
key_files:
  created:
    - service/src/person_detail.rs
    - service_impl/src/person_detail.rs
    - service_impl/src/test/person_detail.rs
  modified:
    - rest-types/src/lib.rs
    - service/src/lib.rs
    - service_impl/src/lib.rs
    - service_impl/src/person_directory.rs
    - service_impl/src/test/mod.rs
decisions:
  - "Zuschnitt des Detail-Aggregats: Variante (b) — Kern-Aggregat plus gezielte Nachladepfade. GET /person/{id} liefert genau die Verzeichniszeile; Stammdaten, Schichtplan-Zuordnung, Rollen und Einladungen laden ihre bestehenden Endpoints."
  - "Der Detail-Lesepfad leitet kind und display_name NICHT selbst ab, sondern ruft entry_from_row auf; die Funktion wird dafuer pub(crate)."
  - "Eine unbekannte UND eine traegerlos gewordene person_id ergeben beide EntityNotFound — die Detailansicht folgt exakt der Sichtbarkeitsregel der Liste (D-62-04)."
  - "Die beiden Kandidatenmethoden rufen den HR-gegateten PersonService mit Authentication::Full auf, erst nach dem eigenen Admin-Gate; ein Durchreichen des Nutzerkontexts wuerde einen Admin ohne HR-Rolle grundlos abweisen."
  - "PersonDao, PermissionDao, SalesPersonService stehen bereits im Deps-Block, obwohl der Lesepfad sie nicht braucht — sie sind die Dependencies des Schreibpfads aus 63-05, und die DI-Verdrahtung soll nur einmal angefasst werden."
metrics:
  duration_minutes: 44
  tasks_completed: 3
  files_modified: 8
  tests_added: 17
  completed: 2026-09-20
---

# Phase 63 Plan 04: Lesepfad der Personen-Detailansicht — Summary

Zu einer `person_id` liefert das Backend jetzt dieselbe fertig klassifizierte Zeile wie
die Liste aus Phase 62 — mit demselben `kind`, demselben `display_name` und derselben
freien Funktion, die beide ableitet; dazu die beiden Kandidatenlisten des Pickers, beide
admin-gegatet.

## Was gebaut wurde

**`rest-types/src/lib.rs`** — vier Transport-Objekte hinter dem Phase-62-Verzeichnisblock,
alle mit `ToSchema`:

| TO | Rolle |
|---|---|
| `CreatePersonEntryTO` | Eingabe des Anlege-Dialogs: `display_name`, `wants_user`, `wants_sales_person`, `username` (D-63-07) |
| `CreatedPersonEntryTO` | Antwort mit genau einem Feld `person_id` — das Ziel der Weiterleitung |
| `PersonFacetTO` | `user` / `sales_person`, `snake_case` wie `PersonKindTO` |
| `AddPersonFacetTO` | eine Form fuer „Login hinzufuegen" und „Person hinzufuegen" (PADM-03) |

Dazu drei `#[cfg(feature = "service-impl")] impl From`-Bloecke (`&PersonFacetTO -> PersonFacet`,
`&PersonFacet -> PersonFacetTO`, `&CreatePersonEntryTO -> CreatePersonEntry`) — rein
feldweise. Insbesondere prueft das Mapping **nicht**, ob mindestens ein Umschalter an ist;
diese Regel gehoert in den Service, der sie unabhaengig vom Client durchsetzt (ASVS V5).

**`service/src/person_detail.rs`** — das Trait `PersonDetailService` mit drei Lesemethoden
(`get_detail`, `unlinked_sales_person_candidates`, `unlinked_user_candidates`), den
Domain-Typen `PersonFacet` und `CreatePersonEntry` sowie `MockPersonDetailService` per
`#[automock]`. Der Modulkopf traegt die Tier-Klassifizierung, die Gate-Entscheidung und
die unten begruendete Zuschnitt-Entscheidung.

**`service_impl/src/person_detail.rs`** — `PersonDetailServiceImpl` mit sieben
Dependencies. `get_detail` gatet auf `"admin"` als erste Anweisung, oeffnet dann die
Transaktion, liest die Einzelzeile, gibt bei `None` **vor dem Commit**
`ServiceError::EntityNotFound(person_id)` zurueck (die Transaktion wird verworfen, P-61-B),
liest sonst die Rollen und baut die Antwort mit `entry_from_row`. Beide Kandidatenmethoden
gaten ebenfalls zuerst und delegieren dann an `PersonService`.

**`service_impl/src/person_directory.rs`** — `entry_from_row` ist `pub(crate)`; der
Doc-Kommentar nennt den neuen Aufrufer und den Grund (PADM-06: genau eine Definition von
Anzeigename und Auspraegung).

## Die entschiedene Zuschnitt-Frage

`63-CONTEXT.md` liess den Zuschnitt des Detail-Aggregats ausdruecklich offen. Dieser Plan
entscheidet **Variante (b): Kern-Aggregat plus gezielte Nachladepfade.**

`get_detail` liefert genau die Verzeichniszeile. Die vier Abschnitte der Seite laden ihre
je eigenen, bereits in Betrieb befindlichen Endpoints (`GET /sales-person/{id}`,
`GET /sales-person-shiftplan/...`, `GET /permission/user/{user}/roles`,
`GET /user-invitation/...`). Begruendung:

1. Die CONTEXT-Auflage „kein N+1" ist eingehalten — jeder Abschnitt ist ein begrenzter
   Aufruf fuer genau EINE Person, nie ein Aufruf je Listenzeile. Der Mock-Test M-2 haelt
   mit `.times(1)` fest, dass `get_detail` selbst genau zwei DAO-Aufrufe macht.
2. Die Auflage „keine Auspraegungs-Ableitung im Frontend" ist eingehalten — `kind` und
   `display_name` kommen fertig aus dem Backend, aus derselben Funktion wie in der Liste.
3. Ein Voll-Aggregat wuerde vier laufende Endpoints duplizieren und vier neue TO-Formen
   samt Tests erzeugen, ohne dass die Seite dadurch weniger Requests braucht.

## Key Decisions

| Entscheidung | Warum |
|---|---|
| `entry_from_row` wird `pub(crate)` statt einer zweiten Ableitung | Eine zweite Herleitung von `kind`/`display_name` fuer die Detailansicht waere genau die Doppelwahrheit, die PADM-06 verbietet. Grep-Kriterium: 0 Treffer fuer `fn entry_from_row` in `person_detail.rs`, 1 Treffer fuer `pub(crate) fn entry_from_row` in `person_directory.rs`. |
| Traegerlos gewordene Person = `EntityNotFound` | Der DAO liefert fuer beide Faelle `None`, weil die Detail-Query dieselbe Traeger-Bedingung traegt wie die Liste (63-02, D-62-04). Eine halbleere Antwort waere fuer das Frontend von einem echten Eintrag ununterscheidbar. |
| `Authentication::Full` an den HR-gegateten `PersonService` | Das strengere Admin-Gate ist bereits passiert. Den Nutzerkontext durchzureichen wuerde einen Admin ohne HR-Rolle grundlos abweisen. `Full` ist erst NACH dem Gate erreichbar und wird nie aus einem Request abgeleitet (T-63-04-B). |
| Rollen-Bulk-Antwort auf den eigenen Benutzer gefiltert | `all_user_roles` liest alle Zuordnungen; fuer eine einzelne Person wird nur der eigene Benutzer in die Map uebernommen. Reine Optimierung — `entry_from_row` schlaegt ohnehin nur unter `row.username` nach (per Mutation belegt, siehe unten). |
| Vier Schreibpfad-Dependencies schon jetzt im Deps-Block | `PersonDao`, `PermissionDao`, `SalesPersonService` und der Schreib-Anteil von `PersonService` braucht erst 63-05. Sie stehen hier, damit `shifty_bin/src/main.rs` nur einmal angefasst wird. Test `sales_person_service_dependency_ist_verdrahtet` haelt fest, dass die Verdrahtung konstruierbar ist — sonst faellt der Fehler erst in einem fremden Plan auf. |

## Neue Tests

**`rest-types` (4 Wire-Tests, im bestehenden Modul `test_person_directory_to_wire`)**

| Test | Was er fixiert |
|---|---|
| `person_facet_to_serialisiert_snake_case` | `"user"` / `"sales_person"` |
| `create_person_entry_to_roundtrip` | verlustfrei mit UND ohne `username` |
| `created_person_entry_to_roundtrip` | genau ein Feld, `person_id` |
| `add_person_facet_to_roundtrip` | beide Richtungen, `facet` als `snake_case` |

**`service_impl` (13 Tests, `service_impl/src/test/person_detail.rs`)**

| ID | Test | Traegt |
|---|---|---|
| M-1 | `get_detail_ohne_admin_privileg_ist_forbidden` | T-63-04-A — weder `use_transaction` noch ein DAO wird aufgerufen (`.times(0)`) |
| M-2 | `get_detail_fragt_jede_dao_methode_genau_einmal` | Prohibition „kein N+1" (`.times(1)` auf beide DAO-Methoden) |
| M-3 | `unlinked_sales_person_candidates_ohne_admin_privileg_ist_forbidden` | T-63-04-B — `PersonService` wird nicht angefasst |
| M-4 | `unlinked_user_candidates_ohne_admin_privileg_ist_forbidden` | T-63-04-B |
| I-1 | `get_detail_verknuepfte_person_ist_linked` | PADM-05/PADM-06, D-62-03 |
| I-2 | `get_detail_nur_benutzer_ist_user_only` | `sales_person_id == None`, alle fuenf Verkaeufer-Felder leer |
| I-3 | `get_detail_nur_verkaeufer_ist_sales_person_only` | `username == None`, leere Rollen |
| I-4 | `get_detail_unbekannte_person_ist_entity_not_found` | keine halbleere Antwort |
| I-5 | `get_detail_traegerlose_person_ist_entity_not_found` | D-62-04 — dieselbe Unsichtbarkeit wie in der Liste |
| I-6 | `get_detail_traegt_nur_die_eigenen_rollen` | D-62-06 |
| I-7 | `unlinked_sales_person_candidates_blendet_verknuepfte_aus` | PADM-04 / SC-4 |
| I-8 | `unlinked_user_candidates_blendet_verknuepfte_aus` | PADM-04 / SC-4 |
| — | `sales_person_service_dependency_ist_verdrahtet` | die Vorrats-Dependency ist konstruierbar |

`cargo test -p service_impl` faehrt jetzt 933 statt 920 Tests, `cargo test -p rest-types`
12 statt 8.

## TDD Gate Compliance

Der Plan traegt `type: tdd`. Die Gate-Sequenz ist pro Task unterschiedlich gelaufen:

- **Task 1: echtes RED→GREEN.** Die vier Wire-Tests wurden zuerst geschrieben;
  `cargo test -p rest-types` scheiterte mit 14 Compile-Fehlern (`use of undeclared type
  PersonFacetTO` etc.). Danach die vier TOs, danach gruen. Committet wurde einmal, als
  `feat` — ein `test(...)`-Commit haette einen nicht uebersetzbaren Zwischenstand
  eingefroren und damit das CLAUDE.md-Gate verletzt.
- **Task 2** ist reine Trait-Arbeit ohne Verhalten; ein RED-Test dagegen waere
  nicht konstruierbar (die Verifikation des Plans ist folgerichtig ein Build).
- **Task 3: kein RED, ersetzt durch Mutationstests.** Eine Trait-Implementierung ohne
  Implementierung compiliert nicht, also war der RED-Schritt auch hier nicht
  konstruierbar. Die Suite wurde stattdessen per Mutation gegen Vakuitaet abgesichert:

| Mutation | Erwartung | Ergebnis |
|---|---|---|
| Gate hinter `use_transaction` geschoben | M-1 faellt | 1 Test rot — bestaetigt |
| `"admin"` durch `"hr"` ersetzt | breiter Ausfall | 9 von 13 rot — bestaetigt |
| Rollen-Filter entfernt | — | **0 rot**, siehe Abweichung 1 |
| alle gelesenen Rollen an die Zeile gehaengt | I-6 faellt | 1 Test rot — bestaetigt |

Alle Mutationen wurden vollstaendig zurueckgenommen, bevor committet wurde.

## Deviations from Plan

### Auto-behoben

**1. [Rule 1 — falsche Testbehauptung] I-6-Doc-Kommentar korrigiert**

- **Gefunden bei:** Task 3, im Mutationstest.
- **Problem:** Der Doc-Kommentar von I-6 behauptete, ein Service, der die
  Rollen-Bulk-Antwort ungefiltert uebernaehme, wuerde fremde Rollen ausliefern. Die
  Mutation „Filter entfernt" liess aber alle 13 Tests gruen: `entry_from_row` schlaegt
  nur unter `row.username` nach, der Filter ist reine Optimierung, keine
  Sicherheitsgrenze. Der Kommentar war also eine falsche Behauptung ueber den Testwert.
- **Fix:** Kommentar auf die Mutation umgeschrieben, die I-6 tatsaechlich faengt — den
  naheliegenden Kurzschluss „es ist ja nur eine Person, also alle gelesenen Rollen an sie
  haengen". Diese Mutation laesst I-6 fallen (verifiziert). Der Filter bleibt als
  Optimierung erhalten.
- **Datei:** `service_impl/src/test/person_detail.rs`
- **Commit:** 9af907fa

**2. [Rule 3 — Akzeptanzkriterium] Zwei Grep-Falschpositive im Kommentar entfernt**

- **Gefunden bei:** Task 3, beim Pruefen der Akzeptanzkriterien.
- **Problem:** Zwei Kriterien greppen den Quelltext und trafen auf Prosa statt Code:
  `grep -c 'HR_PRIVILEGE' service_impl/src/person_detail.rs` gab 1 aus (der Modulkopf
  erwaehnte die Konstante, um sie auszuschliessen), und die Zeilennummer-Probe
  „`check_permission` vor `use_transaction`" traf `use_transaction` zuerst, weil das Wort
  im Kommentar ueber dem Gate stand.
- **Fix:** Beide Erwaehnungen umformuliert („das HR-Privileg", „vor dem Oeffnen der
  Transaktion"). Inhaltlich unveraendert; beide Kriterien sind jetzt erfuellt (0 bzw.
  Zeile 11 vor Zeile 14). Derselbe Vorfall wie in 63-02.
- **Datei:** `service_impl/src/person_detail.rs`
- **Commit:** 9af907fa

### Sonstiges

- Im Deps-Block des Integrationstests werden `PermissionDao` und `SalesPersonService` mit
  **echten** Implementierungen verdrahtet statt mit Mocks. Grund: `MockPermissionDao` und
  `MockSalesPersonService` sind per `#[automock(type Transaction = MockTransaction)]` auf
  `MockTransaction` festgelegt und erfuellen `…<Transaction = TransactionImpl>` nicht. Die
  echten Impls zu bauen war der kleinere Eingriff als den Deps-Block aufzuweichen.
- `get_detail` kehrt bei `None` sofort zurueck, **bevor** `all_user_roles` laeuft. Der Plan
  nennt die Reihenfolge „`find_directory_entry`, `all_user_roles`, Commit"; die
  Auflage „vor dem Commit" ist eingehalten und der Fehlerpfad spart eine ueberfluessige
  Abfrage.

## Auth Gates

Keine.

## Threat Flags

Keine neue Angriffsflaeche ausserhalb des Registers. Abgleich:

| Threat | Disposition | Umsetzung |
|---|---|---|
| T-63-04-A (EoP auf `get_detail`) | mitigate | `check_permission("admin", context)` ist die erste Anweisung; M-1 belegt mit `.times(0)`, dass weder Transaktion noch DAO angefasst werden. Grep: 0 Treffer fuer das HR-Literal. |
| T-63-04-B (Disclosure ueber die Kandidatenlisten) | mitigate | Beide Methoden gaten auf `admin`, bevor `PersonService` aufgerufen wird; `Authentication::Full` ist erst danach erreichbar. M-3/M-4 belegen es. |
| T-63-04-C (Admin sieht fremde Person) | accept | Unveraendert akzeptiert (PADM-08); abschnittsweises Rollen-Gating bleibt PADM-F-02. |
| T-63-04-D (clientseitig geschickte abgeleitete Daten) | mitigate | Die drei Eingabe-TOs tragen ausschliesslich Rohwerte; `kind` und `display_name` werden nie entgegengenommen. |
| T-63-04-SC (Supply Chain) | mitigate | Keine neue Abhaengigkeit, kein Package-Install. |

## Known Stubs

Keine. Der Service ist noch **nicht** in `shifty_bin/src/main.rs` verdrahtet und hat noch
keinen REST-Handler — beides ist planmaessig Sache von 63-05/63-06 und kein Stub im Sinne
einer leeren Anzeige.

## Docs-Freshness-Gate

Beruehrte Trigger-Dateien: `service/**/*.rs` (`person_detail.rs` neu, `lib.rs`).
`shifty_bin/src/main.rs`, `migrations/sqlite/*.sql`, `dao/**/*.rs`,
`service_impl/src/permission.rs` und `service_impl/src/billing_period_report.rs` wurden
**nicht** beruehrt.

| Beruehrt | → faellig |
|---|---|
| `service/src/person_detail.rs` (neues Trait, neue Domain-Typen) | Passende `docs/features/F*.md` (Personenverwaltung) — Trait-Auszug, Auth-Gate-Tabelle (`admin`, nicht `hr`), Randfall „traegerlose Person → 404"; beide Sprachen |

**Nicht in diesem Plan erledigt** — Plan **63-10** fuehrt die Doku-Aktualisierung der
Phase zentral durch. Die konkrete Aenderung: ein neuer Business-Logic-Service
`PersonDetailService` mit drei admin-gegateten Lesemethoden, und
`service_impl::person_directory::entry_from_row` hat ab jetzt zwei Aufrufer.

## Deferred Issues

Unveraendert gegenueber 63-01/63-02: `service_impl/src/test/person_directory_dao.rs:149`
loest unter `cargo clippy --all-targets` eine `needless_lifetimes`-Warnung aus
(pre-existing, Phase 62, ausserhalb des Scopes). `cargo fmt` steht in der Nix-Dev-Shell
weiterhin nicht zur Verfuegung (`error: no such command: fmt`), der neue Code wurde
darum von Hand rustfmt-konform geschrieben.

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo build --workspace` | Exit 0 |
| `cargo test --workspace` | gruen, 0 Failures (933 `service_impl` + 95 Integration + uebrige Suites) |
| `cargo clippy --workspace -- -D warnings` | Exit 0 |
| `cargo test -p rest-types` | 12 passed (vorher 8) |
| `cargo test -p service_impl person_detail::` | 13 passed |
| `grep -c 'pub struct CreatePersonEntryTO\|pub struct CreatedPersonEntryTO\|pub enum PersonFacetTO\|pub struct AddPersonFacetTO' rest-types/src/lib.rs` | 4 |
| `grep -c 'ToSchema' rest-types/src/lib.rs` | 78 (vorher 74) |
| `grep -c 'pub mod person_detail' service/src/lib.rs` | 1 |
| `grep -c 'async fn ' service/src/person_detail.rs` | 3 |
| `grep -c 'admin' service/src/person_detail.rs` | 2 |
| `grep -c 'HR_PRIVILEGE' service_impl/src/person_detail.rs` | 0 |
| `grep -c 'fn entry_from_row\|PersonKind::Linked' service_impl/src/person_detail.rs` | 0 |
| `grep -c 'pub(crate) fn entry_from_row' service_impl/src/person_directory.rs` | 1 |
| Gate vor Transaktion in `get_detail` | `check_permission` Zeile 11, `use_transaction` Zeile 14 |
| `grep -c 'CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13' service_impl/src/billing_period_report.rs` | 1 (unveraendert) |

## Self-Check: PASSED

- `service/src/person_detail.rs` — FOUND
- `service_impl/src/person_detail.rs` — FOUND
- `service_impl/src/test/person_detail.rs` — FOUND
- Commit `8064ea2c` — FOUND
- Commit `16e7d011` — FOUND
- Commit `9af907fa` — FOUND

## Commits

| Task | Commit | Typ |
|---|---|---|
| 1 | `8064ea2c` | `feat(63-04): Transport-Objekte fuer Anlege-Dialog und Auspraegungs-Aktionen` |
| 2 | `16e7d011` | `feat(63-04): PersonDetailService-Trait mit den drei Lesemethoden` |
| 3 | `9af907fa` | `feat(63-04): PersonDetailServiceImpl — admin-gegateter Lesepfad plus Tests` |
