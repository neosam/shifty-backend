---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 05
subsystem: service
tags: [person-detail, schreibpfad, transaktion, orphan-cleanup, d-63-01, d-63-05, d-63-07]
status: complete
requires:
  - "dao::PermissionDao::create_user/delete_user auf der geteilten Transaktion (63-01)"
  - "dao::person::PersonDao::has_live_facets / delete / find_by_sales_person / find_by_user (63-02)"
  - "dao::person_directory::PersonDirectoryDao::find_directory_entry (63-02)"
  - "service::person_detail::PersonDetailService + PersonFacet + CreatePersonEntry (63-04)"
  - "dao::sales_person::SalesPersonDao::assign_to_user (Phase 61, D-61-03/D-61-04 — unveraendert wiederverwendet)"
provides:
  - "service::person_detail::PersonDetailService::create_combined -> Uuid (PADM-02 / SC-2)"
  - "service::person_detail::PersonDetailService::add_facet -> Uuid (PADM-03 / SC-3)"
  - "service::person_detail::PersonDetailService::remove_facet -> () (PADM-07 / SC-5)"
  - "service::person_detail::PersonDetailService::get_detail_by_sales_person -> PersonEntry (PADM-04 / SC-4)"
  - "service_impl::person_detail::PersonDetailServiceDeps traegt jetzt SalesPersonDao (8 Dependencies)"
affects:
  - "63-06 (REST-Oberflaeche /person: die vier Methoden sind die Handler-Rueckseite; DI-Verdrahtung muss SalesPersonDao mitnehmen)"
  - "63-07 … 63-09 (Frontend: add_facet gibt die RESULTIERENDE person_id zurueck — Aufrufer muessen sie verwenden)"
  - "63-10 (Doku-Aktualisierung der Phase)"
  - "63-11 (Verifikation: bekannte Kante session.user_id ohne Kaskade)"
tech_stack:
  added: []
  patterns:
    - "Eine Transaktion ueber zwei Aggregate: der innere Commit des Basic-Service ist dank Arc::into_inner ein No-Op, solange der orchestrierende Service seinen eigenen Arc haelt"
    - "Serverseitige Auswahl-Validierung vor use_transaction — die Client-Regel ist Bequemlichkeit, die Service-Regel ist die Grenze (ASVS V5)"
    - "Verknuepfen ausschliesslich ueber den bestehenden DAO-Schreibpfad; Merge-Richtung, Konflikt-Erkennung und Person-Absorption bleiben an genau einer Stelle"
    - "Gemeinsamer Zusammenbau-Rumpf (build_entry) auf offener Transaktion statt zweier Ableitungen fuer zwei Lesepfade"
    - "Orphan-Cleanup als Nachlauf in derselben Transaktion, per Rollback-Test belegt statt per Kommentar behauptet"
key_files:
  created: []
  modified:
    - service/src/person_detail.rs
    - service_impl/src/person_detail.rs
    - service_impl/src/test/person_detail.rs
decisions:
  - "add_facet gibt die RESULTIERENDE person_id zurueck, nicht die uebergebene. Beim Zuschalten einer Sales Person auf einen reinen Benutzer-Eintrag gewinnt nach D-61-03 die Sales Person, der Eintrag wechselt seine Identitaet."
  - "Beide Anlege-Pfade validieren ihre Pflichtfelder (Benutzername bzw. Anzeigename) serverseitig und vor jedem Datenbankzugriff; die Anzeigenamen-Pruefung ist eine Ergaenzung gegenueber dem Plan (Rule 2)."
  - "Die Vorgabefarbe einer im Dialog entstandenen Sales Person ist neutrales Weiss (#ffffff), als Modulkonstante. D-63-07: der Dialog fragt keine Farbe ab."
  - "Der gemeinsame Rumpf von get_detail und get_detail_by_sales_person liegt in der privaten Hilfsmethode build_entry und committet nicht — der Aufrufer entscheidet ueber Commit bzw. Verwerfen (P-61-B)."
  - "get_detail_by_sales_person liefert EntityNotFoundGeneric nur fuer die nicht aufloesbare Sales Person; eine aufloesbare, aber unsichtbare Person faellt in EntityNotFound und folgt damit exakt der Sichtbarkeitsregel des Verzeichnisses (D-62-04)."
  - "session.user_id ohne ON DELETE CASCADE bleibt ungefixt (T-63-05-E, accept) und ist im Modulkopf als bekannte Kante fuer Plan 63-11 dokumentiert."
metrics:
  duration_minutes: 21
  tasks_completed: 3
  files_modified: 3
  tests_added: 25
  completed: 2026-09-20
---

# Phase 63 Plan 05: Die drei Auspraegungs-Mutationen — Summary

Anlegen, Zuschalten und Entfernen einer Auspraegung laufen jetzt im
Business-Logic-Tier — jeweils admin-gegatet, jeweils in **einer** Transaktion
ueber beide Aggregate, und mit der traegerlos gewordenen `person`-Zeile als
Nachlauf derselben Transaktion.

## Was gebaut wurde

`service/src/person_detail.rs` traegt vier neue Trait-Methoden (3 → 7
`async fn`), `service_impl/src/person_detail.rs` ihre Implementierung.

| Methode | Rolle |
|---|---|
| `create_combined(&CreatePersonEntry, ..) -> Uuid` | Kombiniertes Anlegen (PADM-02 / SC-2). Rueckgabe: die `person_id` der Detailansicht, in die der Dialog weiterleitet. |
| `add_facet(person_id, facet, display_name, username, ..) -> Uuid` | Fehlende Seite zuschalten (PADM-03 / SC-3). Rueckgabe: die **resultierende** `person_id`. |
| `remove_facet(person_id, facet, ..) -> ()` | Genau eine Seite entfernen (PADM-07 / SC-5), inkl. D-63-01-Aufraeumung. |
| `get_detail_by_sales_person(sales_person_id, ..) -> PersonEntry` | Rueckweg in die Detailansicht nach dem Verknuepfen eines bestehenden Verkaeufers (PADM-04 / SC-4). |

Dazu zwei Modulkonstanten (`PERSON_DETAIL_SERVICE_PROCESS`,
`DEFAULT_BACKGROUND_COLOR`), die private Hilfsmethode `build_entry` und die
achte Dependency `SalesPersonDao` im Deps-Block.

## Wie die Atomaritaet zustande kommt — und wie sie belegt ist

`create_combined` oeffnet genau eine Transaktion und reicht sie als
`Some(tx.clone())` an `SalesPersonService::create` und als `tx.clone()` an
`PermissionDao::create_user`. Der innere Commit des Basic-Service ist dank
`Arc::into_inner` in `TransactionDaoImpl::commit` ein No-Op, solange dieser
Service seinen eigenen Arc noch haelt.

Das ist **nicht** per Kommentar behauptet, sondern per Mutation belegt: wird
`Some(tx.clone())` beim Anlegen der Sales Person durch `None` ersetzt — die
Sales Person bekaeme ihre eigene, sofort committete Transaktion —, faellt Test
C-8 (`create_combined_teilfehler_laesst_keine_haelfte_zurueck`) mit
`left: 0, right: 1`. Die Mutation wurde vollstaendig zurueckgenommen.

## Der `person_id`-Wechsel, ausdruecklich benannt

Beim kombinierten Anlegen entstehen Benutzer und Sales Person von Anfang an auf
derselben Person — es gibt nichts zusammenzufuehren. Beim nachtraeglichen
Zuschalten eines Verkaeufers auf einen reinen Benutzer-Eintrag gibt es diesen
Vorlauf nicht: die neue Sales Person bringt ihre eigene Identitaet mit
(D-61-01), und der bestehende, seit Phase 61 getestete Schreibpfad
`SalesPersonDao::assign_to_user` fuehrt beide zusammen — **die Sales Person
gewinnt** (D-61-03), der Benutzer wandert auf sie, die absorbierte Person wird
geloescht.

**Folge:** In genau diesem Fall wechselt der Eintrag seine `person_id`.
`add_facet` gibt deshalb die resultierende id zurueck, nicht die uebergebene.
Test A-2 prueft beide Seiten explizit (`assert_ne!` auf die Rueckgabe,
`person_exists == false` auf die alte id) und ist per Mutation abgesichert:
gibt die Methode stattdessen die Eingabe zurueck, faellt A-2.

Es wurde **kein** zweiter Verknuepfungs-Schreibpfad und **keine** zweite
409-Uebersetzung gebaut. `DaoError::EntityAlreadyExists` wird an genau einer
Stelle in `ServiceError::EntityAlreadyExists(person_id)` uebersetzt, ohne zu
committen — dieselbe Form wie in `SalesPersonServiceImpl::set_user` (D-61-04).

## Die beiden Loeschsemantiken

| Seite | Semantik | Begruendung |
|---|---|---|
| Login | **Hard Delete** via `PermissionDao::delete_user` | Rollen (`user_role`) und offene Einladungen (`user_invitation`) gehen per `ON DELETE CASCADE` mit. Bewusst keine zusaetzlichen Anwendungs-Deletes, die vom Schema abdriften koennten. |
| Verkaeufer | **Soft Delete** via `SalesPersonService::delete` | Die Zeile bleibt mit `deleted`-Stempel und neuer Version stehen — Schichtplan-Historie und die write-once-Snapshots von `billing_period_sales_person` bleiben unberuehrt. |

`remove_facet` nimmt immer genau einen `PersonFacet`-Wert entgegen; ein
Sammelknopf ist durch D-63-05 ausgeschlossen (Grep-Kriterien: 7 `async fn` im
Trait, 0 Treffer fuer `facets:`).

Danach — und zwingend in derselben Transaktion — laeuft die Traegerprobe und,
bei `false`, das Loeschen der `person`-Zeile (D-63-01, Lesart (b) aus Plan
63-02). Auch das ist per Mutation belegt: wird der Nachlauf entfernt, fallen
R-2 und R-4.

## Neue Tests (25)

`cargo test -p service_impl` faehrt jetzt **958** statt 933 Tests; der Filter
`person_detail` trifft **51** (Modul `person_detail` 38 + `person_detail_dao` 13).

**Task 1 — `create_combined` (8)**

| ID | Test | Traegt |
|---|---|---|
| C-1 | `create_combined_beide_auspraegungen_teilen_eine_person` | SC-2: eine `person`-Zeile fuer beide Objekte, Rueckgabe == diese id |
| C-2 | `create_combined_nur_verkaeufer_legt_keinen_benutzer_an` | „genau die gewaehlten Objekte" |
| C-3 | `create_combined_nur_benutzer_legt_keine_sales_person_an` | dito, Gegenrichtung |
| C-4 | `create_combined_ohne_auswahl_ist_validation_error` | D-63-07 / T-63-05-B, serverseitig |
| C-5 | `create_combined_ohne_benutzernamen_ist_validation_error` | fehlend **und** nur Leerzeichen |
| C-6 | `create_combined_ohne_anzeigenamen_ist_validation_error` | Gegenstueck (Rule-2-Ergaenzung) |
| C-7 | `create_combined_ohne_admin_privileg_ist_forbidden` | T-63-05-A, kein Schreibzugriff |
| C-8 | `create_combined_teilfehler_laesst_keine_haelfte_zurueck` | T-63-05-C, per Mutation belegt |

**Task 2 — `add_facet` (7) + `get_detail_by_sales_person` (3)**

| ID | Test | Traegt |
|---|---|---|
| A-1 | `add_facet_benutzer_auf_verkaeufer_eintrag_laesst_verkaeufer_unveraendert` | SC-3: Name und Version der Sales Person unveraendert |
| A-2 | `add_facet_verkaeufer_auf_benutzer_eintrag_liefert_neue_person_id` | D-61-03, per Mutation belegt |
| A-3 | `add_facet_auf_besetzte_benutzer_seite_ist_entity_already_exists` | D-61-04 |
| A-4 | `add_facet_auf_besetzte_verkaeufer_seite_ist_entity_already_exists` | D-61-04, Gegenrichtung |
| A-5 | `add_facet_ohne_pflichtfeld_ist_validation_error` | beide Seiten |
| A-6 | `add_facet_unbekannte_person_ist_entity_not_found` | Sichtbarkeitsregel |
| A-7 | `add_facet_ohne_admin_privileg_ist_forbidden` | T-63-05-A |
| G-1 | `get_detail_by_sales_person_liefert_dieselbe_zeile_wie_get_detail` | SC-4, `assert_eq!` auf den ganzen `PersonEntry` |
| G-2 | `get_detail_by_sales_person_ohne_aufloesbare_person_ist_entity_not_found_generic` | Fehlerform |
| G-3 | `get_detail_by_sales_person_ohne_admin_privileg_ist_forbidden` | T-63-05-A |

**Task 3 — `remove_facet` (7)**

| ID | Test | Traegt |
|---|---|---|
| R-1 | `remove_facet_benutzer_auf_verknuepftem_eintrag_kaskadiert` | Hard Delete + Kaskade auf `user_role` **und** `user_invitation`; Sales Person und Person bleiben |
| R-2 | `remove_facet_benutzer_ohne_verkaeufer_raeumt_person_auf` | D-63-01, per Mutation belegt |
| R-3 | `remove_facet_verkaeufer_auf_verknuepftem_eintrag_ist_soft_delete` | Stempel + neue Version, Zeile bleibt, Benutzer unangetastet |
| R-4 | `remove_facet_verkaeufer_ohne_benutzer_raeumt_person_auf` | Lesart (b) inkl. haengendem `person_id` |
| R-5 | `remove_facet_auf_fehlende_seite_ist_entity_not_found` | fehlende Seite **und** unbekannte Person |
| R-6 | `remove_facet_ohne_admin_privileg_ist_forbidden` | T-63-05-A |
| R-7 | `remove_facet_nutzt_die_transaktion_des_aufrufers` | Ein-Transaktions-Nachweis per Rollback: weder Loeschung noch Aufraeumung bleibt wirksam |

## TDD Gate Compliance

Der Plan traegt `type: tdd`. Die Gate-Sequenz lief wie in 63-02 und 63-04 als
`feat` pro Task, **nicht** als `test` → `feat`. Grund unveraendert: eine
Trait-Methode ohne Implementierung compiliert nicht, ein RED-Commit waere damit
ein nicht uebersetzbarer Zwischenstand und wuerde das CLAUDE.md-Gate
(`cargo build` / `cargo test` / `cargo clippy` vor jedem Commit) verletzen.

Als Ersatz fuer den entfallenen RED-Schritt wurde jede Task-Suite per
**Mutation** gegen Vakuitaet abgesichert:

| Task | Mutation | Erwartung | Ergebnis |
|---|---|---|---|
| 1 | Sales Person auf eigener Transaktion (`Some(tx.clone())` → `None`) | C-8 faellt | 1 Test rot — bestaetigt |
| 2 | `add_facet` gibt die Eingabe-`person_id` statt der resultierenden zurueck | A-2 faellt | 1 Test rot — bestaetigt |
| 3 | Orphan-Cleanup-Block entfernt | R-2 und R-4 fallen | 2 Tests rot — bestaetigt |

Alle drei Mutationen wurden vollstaendig zurueckgenommen, bevor committet
wurde (`cargo test --workspace` danach gruen).

## Deviations from Plan

### Auto-behoben

**1. [Rule 2 — fehlende Eingabe-Validierung] Anzeigename wird geprueft**

- **Gefunden bei:** Task 1, beim Umsetzen der Validierungs-Schritte.
- **Problem:** Der Plan schreibt nur die Pruefung von `username` vor. Ein
  `wants_sales_person` mit leerem oder nur aus Leerzeichen bestehendem
  `display_name` haette eine namenlose Sales Person erzeugt — im Verzeichnis
  mit leerem Anzeigenamen und damit praktisch unauffindbar, obwohl PADM-06 den
  Anzeigenamen ausdruecklich als „nie leer" fuehrt.
- **Fix:** Symmetrische Pruefung ergaenzt (`ValidationError` mit
  `InvalidValue("display_name")`), in `create_combined` und in `add_facet`.
  Test C-6 bzw. der zweite Zweig von A-5 halten sie fest.
- **Datei:** `service_impl/src/person_detail.rs`
- **Commits:** `719352fc`, `816c6b51`

**2. [Rule 3 — Signatur praezisiert] `add_facet` gibt `Uuid` zurueck**

- **Gefunden bei:** Task 2.
- **Problem:** Die Artifacts-Tabelle des Plans fuehrt `-> Result<Uuid, _>`, der
  Prosa-Abschnitt spricht von „die resultierende id". Zwischenzeitlich war
  `PersonEntry` geschrieben — das haette den Vertrag fuer Plan 63-06 verschoben.
- **Fix:** Auf `Uuid` zurueckgesetzt, wie in der Artifacts-Tabelle. Der
  Detail-Nachladepfad steht mit `get_detail` bzw. `get_detail_by_sales_person`
  ohnehin bereit.
- **Datei:** `service/src/person_detail.rs`
- **Commit:** `816c6b51`

### Sonstiges

- Der gemeinsame Rumpf von `get_detail` wurde wie im Plan vorgesehen in
  `build_entry` gezogen. Dabei aendert sich das Verhalten von `get_detail`
  nicht: der Not-Found-Pfad kehrt weiterhin **vor** `all_user_roles` und vor dem
  Commit zurueck; die Mock-Tests M-1 und M-2 (`.times(0)` / `.times(1)`) halten
  unveraendert.
- `PersonDetailServiceDeps` hat mit `SalesPersonDao` jetzt **acht**
  Dependencies. Die beiden Deps-Impls in der Testdatei und die zwei
  Konstruktionsstellen wurden mitgezogen. **Plan 63-06 muss die DI-Verdrahtung
  in `shifty_bin/src/main.rs` mit dieser achten Dependency bauen.**
- `cargo fmt` steht in der Nix-Dev-Shell weiterhin nicht zur Verfuegung; der
  neue Code wurde von Hand rustfmt-konform geschrieben.

## Auth Gates

Keine.

## Threat Flags

Keine neue Angriffsflaeche ausserhalb des Registers. Abgleich:

| Threat | Disposition | Umsetzung |
|---|---|---|
| T-63-05-A (EoP auf den drei Mutationen) | mitigate | `check_permission("admin", context)` ist in allen vier Methoden die erste Anweisung, vor Validierung und vor `use_transaction`. C-7, A-7, R-6 und G-3 belegen es; jeder prueft zusaetzlich, dass nichts geschrieben wurde. `Authentication::Full` ist erst nach dem Gate erreichbar. |
| T-63-05-B (Auswahl-Regel umgangen) | mitigate | Serverseitige Pruefung vor jedem Schreibzugriff (C-4). |
| T-63-05-C (Teil-Commit) | mitigate | Eine Transaktion ueber beide Aggregate; C-8 provoziert den Teilfehler und ist per Mutation als nicht-vakuum belegt. |
| T-63-05-D (Wettlauf auf das Verknuepfungs-Ziel) | mitigate | Unveraendert der Phase-61-Pfad; `DaoError::EntityAlreadyExists` → `ServiceError::EntityAlreadyExists`, kein Commit. A-3/A-4 halten den Vertrag fest. |
| T-63-05-E (Loeschen bei offener Sitzung) | accept | Vorbestehend, nicht gefixt; im Modulkopf als bekannte Kante fuer Plan 63-11 dokumentiert. |
| T-63-05-F (verwaiste `person`-Zeilen) | mitigate | Traegerprobe plus `delete` in derselben Transaktion (R-2, R-4, R-7). |
| T-63-05-SC (Supply Chain) | mitigate | Keine neue Abhaengigkeit, kein Package-Install. |

## Known Stubs

Keine. Der Service ist weiterhin **nicht** in `shifty_bin/src/main.rs`
verdrahtet und hat noch keinen REST-Handler — beides ist planmaessig Sache von
63-06 und kein Stub im Sinne einer leeren Anzeige.

## Docs-Freshness-Gate

Beruehrte Trigger-Datei: `service/**/*.rs` (`person_detail.rs`).
`shifty_bin/src/main.rs`, `dao/**/*.rs`, `migrations/sqlite/*.sql`,
`service_impl/src/permission.rs`, `service_impl/src/reporting.rs` und
`service_impl/src/billing_period_report.rs` wurden **nicht** beruehrt.

| Beruehrt | → faellig |
|---|---|
| `service/src/person_detail.rs` (vier neue Trait-Methoden) | Passende `docs/features/F*.md` (Personenverwaltung) — Trait-Auszug um die drei Mutationen plus `get_detail_by_sales_person` erweitern, Auth-Gate-Tabelle (`admin` fuer alle vier), Randfaelle: leerer Auswahl-Satz → 422, besetzte Seite → 409, `person_id`-Wechsel beim Zuschalten einer Sales Person, Hard-vs-Soft-Delete-Asymmetrie, Orphan-Cleanup; beide Sprachen |

**Nicht in diesem Plan erledigt** — Plan **63-10** fuehrt die
Doku-Aktualisierung der Phase zentral durch.

## Deferred Issues

Unveraendert gegenueber 63-01/63-02/63-04:

- `service_impl/src/test/person_directory_dao.rs:149` loest unter
  `cargo clippy --all-targets` eine `needless_lifetimes`-Warnung aus
  (pre-existing, Phase 62, ausserhalb des Scopes; feuert nicht unter dem
  CLAUDE.md-Gate und nicht in CI).
- `cargo fmt` fehlt in der Nix-Dev-Shell (`error: no such command: fmt`).
- **Neu:** `session.user_id` ohne `ON DELETE CASCADE` (T-63-05-E, accept) —
  bewusst nicht gefixt, im Modulkopf dokumentiert, gehoert in die Verifikation
  von Plan 63-11.

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo build --workspace` | Exit 0 |
| `cargo test --workspace` | gruen, 0 Failures (958 `service_impl` + 95 Integration + uebrige Suites) |
| `cargo clippy --workspace -- -D warnings` | Exit 0 |
| `cargo test -p service_impl person_detail::create_combined` | 8 passed (≥ 7 gefordert) |
| `cargo test -p service_impl person_detail::add_facet` | 7 passed (≥ 7 gefordert) |
| `cargo test -p service_impl person_detail::get_detail_by_sales_person` | 3 passed |
| `cargo test -p service_impl person_detail` | 51 passed (≥ 28 gefordert) |
| `sed -n '/async fn create_combined/,/^    }/p' … \| grep -c 'ValidationError'` | 3 (≥ 1 gefordert) |
| `sed -n '/async fn create_combined/,/^    }/p' … \| grep -c 'assign_to_user'` | 0 |
| `sed -n '/async fn remove_facet/,/^    }/p' … \| grep -c 'has_live_facets'` | 1 |
| `sed -n '/async fn remove_facet/,/^    }/p' … \| grep -v '^\s*//' \| grep -c 'DELETE FROM user_role\|DELETE FROM user_invitation'` | 0 |
| `grep -c 'async fn ' service/src/person_detail.rs` | 7 |
| `grep -c 'facets:' service/src/person_detail.rs` | 0 |
| `grep -c 'discard_assigned_user' service_impl/src/person_detail.rs` | 0 |
| `grep -c 'CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13' …` | 1 (unveraendert) |
| Keine neue Datei unter `migrations/sqlite/` | bestaetigt (`git status --short migrations/` leer) |

## Commits

| Task | Commit | Typ |
|---|---|---|
| 1 | `719352fc` | `feat(63-05): create_combined — ein Eintrag, zuschaltbare Auspraegungen, eine Transaktion` |
| 2 | `816c6b51` | `feat(63-05): add_facet und get_detail_by_sales_person — fehlende Auspraegung zuschalten` |
| 3 | `2bdaa2be` | `feat(63-05): remove_facet mit D-63-01-Orphan-Cleanup` |

## Self-Check: PASSED

- `service/src/person_detail.rs` — FOUND
- `service_impl/src/person_detail.rs` — FOUND
- `service_impl/src/test/person_detail.rs` — FOUND
- Commit `719352fc` — FOUND
- Commit `816c6b51` — FOUND
- Commit `2bdaa2be` — FOUND
