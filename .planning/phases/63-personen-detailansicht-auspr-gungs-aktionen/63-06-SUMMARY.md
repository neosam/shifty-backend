---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 06
subsystem: rest / di
tags: [person-detail, rest, openapi, di-wiring, dev-proxy, admin-gate, padm-08]
status: complete
requires:
  - "service::person_detail::PersonDetailService — sieben async fn (63-04, 63-05)"
  - "service_impl::person_detail::PersonDetailServiceDeps — acht Dependencies inkl. SalesPersonDao (63-05)"
  - "rest_types::{CreatePersonEntryTO, CreatedPersonEntryTO, PersonFacetTO, AddPersonFacetTO} (63-04)"
  - "rest_types::PersonEntryTO + From<&PersonEntry> (62-02/62-03)"
  - "rest::sales_person::generate_route — POST /sales-person/{id}/user, unveraendert seit Phase 61"
provides:
  - "GET /person/{person_id} -> PersonEntryTO (PADM-05)"
  - "GET /person/by-sales-person/{sales_person_id} -> PersonEntryTO (PADM-04 / SC-4)"
  - "POST /person -> 201 CreatedPersonEntryTO (PADM-02 / SC-2)"
  - "POST /person/{person_id}/facet -> 200 CreatedPersonEntryTO mit der RESULTIERENDEN person_id (PADM-03 / SC-3)"
  - "DELETE /person/{person_id}/facet/{facet} -> 204 (PADM-07 / SC-5)"
  - "GET /person/candidates/sales-persons -> [SalesPersonTO] (PADM-04)"
  - "GET /person/candidates/users -> [UserTO] (PADM-04)"
  - "rest::RestStateDef::PersonDetailService + person_detail_service()"
  - "shifty_bin: PersonDetailServiceDependencies, Typ-Alias, RestStateImpl-Feld und Accessor"
  - "shifty-dioxus/Dioxus.toml: [[web.proxy]] fuer /person"
  - "rest::sales_person ist pub — der Phase-61-Pfad ist fuer Integrationstests erreichbar"
affects:
  - "63-07 (Frontend-API-Schicht: die Aufruf-URLs muessen zeichengleich `/person` tragen)"
  - "63-08 / 63-09 (Detailseite und Dialoge: add_facet-Antwort traegt die resultierende person_id)"
  - "63-10 (Doku-Aktualisierung: shifty_bin/src/main.rs ist Trigger-Datei — DI-Verdrahtung)"
  - "63-11 (Verifikation: der Proxy-Eintrag ist NUR ueber einen laufenden dx serve nachweisbar)"
tech_stack:
  added: []
  patterns:
    - "Sieben Handler, null Autorisierungs-Logik: das Gate liegt vollstaendig im Service, der Handler reicht nur den Session-Kontext durch (D-62-08)"
    - "Praefix als Vier-Stellen-Vertrag: .nest, ApiDoc-Nest, Frontend-URL, [[web.proxy]] — im Modulkopf und in der Test-Konstante namentlich benannt"
    - "Proxy-Reihenfolge ist semantisch: der Dev-Server matcht praefix-literal, `/person` steht deshalb HINTER `/person-directory`"
    - "Facet-Kennung als Pfad-Parameter (`PersonFacetTO` per serde aus dem Pfad), weil DELETE in diesem Repo durchgaengig ohne Body arbeitet"
    - "DAO-Handles geklont statt verschoben, wo ein zweiter Konsument entsteht; fuer den nicht-Arc-Handle eine zweite Instanz ueber den geteilten Pool"
    - "403-Nachweis als Schleife ueber ALLE sieben Operationen statt als Stichprobe — ein vergessenes Gate auf genau einer Route ist der wahrscheinlichste Fehler"
key_files:
  created:
    - rest/src/person_detail.rs
    - shifty_bin/src/integration_test/person_detail.rs
  modified:
    - rest/src/lib.rs
    - shifty_bin/src/main.rs
    - shifty_bin/src/integration_test.rs
    - shifty-dioxus/Dioxus.toml
decisions:
  - "rest::sales_person wird pub-exportiert (nicht geaendert), damit der 409-Regressionstest den unveraenderten Phase-61-Endpoint via tower::oneshot befahren kann — dasselbe Muster wie pub mod person_directory / report / feature_flag."
  - "Der Helfer request_as nestet BEIDE Praefixe (/person und /sales-person) in denselben Router, genau wie rest::start_server — so faehrt der 409-Test gegen dieselbe Router-Form wie die Produktion."
  - "Eine zweite SalesPersonDao-Instanz statt eines Arc-Klons: der Handle oben wird per into() in den SalesPersonService verschoben; der DAO ist zustandslos ueber dem geteilten Pool."
  - "add_person_facet antwortet mit 200 und CreatedPersonEntryTO statt mit 204 — die RESULTIERENDE person_id muss den Aufrufer erreichen (D-61-03), sonst zeigt das Frontend ins Leere."
  - "Neunter Test ergaenzt (kandidatenlisten_blenden_verknuepfte_ziele_aus): ohne ihn waeren die beiden Kandidaten-Routen ausschliesslich negativ (403) abgedeckt."
metrics:
  duration_minutes: 27
  tasks_completed: 3
  files_modified: 6
  tests_added: 9
  completed: 2026-09-20
---

# Phase 63 Plan 06: HTTP-Oberflaeche der Personen-Detailansicht — Summary

Die sieben Operationen des Detail-Aggregats stehen unter `/person`, sind
OpenAPI-dokumentiert, im Dependency-Injection-Graphen verdrahtet, im Dev-Proxy
eingetragen und auf HTTP-Ebene gegen Gate, Roundtrip und Konflikt-Semantik
belegt.

## Was gebaut wurde

| Route | Antwort | Traegt |
|---|---|---|
| `GET /person/candidates/sales-persons` | `[SalesPersonTO]` | PADM-04 |
| `GET /person/candidates/users` | `[UserTO]` | PADM-04 |
| `GET /person/by-sales-person/{sales_person_id}` | `PersonEntryTO` | PADM-04 / SC-4 |
| `POST /person` | 201 + `CreatedPersonEntryTO` | PADM-02 / SC-2 |
| `GET /person/{person_id}` | `PersonEntryTO` | PADM-05 / PADM-06 |
| `POST /person/{person_id}/facet` | 200 + `CreatedPersonEntryTO` | PADM-03 / SC-3 |
| `DELETE /person/{person_id}/facet/{facet}` | 204 | PADM-07 / SC-5 |

Dazu `PersonDetailApiDoc`, der Assoziationstyp `RestStateDef::PersonDetailService`
samt Accessor, die DI-Verdrahtung in `shifty_bin/src/main.rs` und der
`[[web.proxy]]`-Eintrag.

## Der `person_id`-Wechsel erreicht den Aufrufer

`add_facet` gibt seit 63-05 die **resultierende** `person_id` zurueck, nicht die
uebergebene: schaltet man einem reinen Benutzer-Eintrag eine Sales Person zu,
gewinnt nach D-61-03 die Sales Person, und der Eintrag wechselt seine Identitaet.
Der Handler antwortet deshalb mit **200 und `CreatedPersonEntryTO`** statt mit
einem leeren 204. Ein 204 haette die neue id verschluckt, und das Frontend
zeigte danach auf eine geloeschte Person. Die OpenAPI-Beschreibung sagt das
ausdruecklich („use this person_id, not the one from the path").

## Der Proxy-Eintrag und seine Reihenfolge

`/person` ist ein **Praefix** von `/person-directory`, und der
Dioxus-Dev-Server-Proxy matcht praefix-literal. Der neue Eintrag steht deshalb
als **letzter** der beiden in `Dioxus.toml`; umgekehrt finge er die
Verzeichnis-Anfragen mit ab.

Im Axum-Router ist die Reihenfolge dagegen unkritisch — das wurde nicht
behauptet, sondern gemessen: bei laufendem Server liefern
`/person/candidates/users` **und** `/person-directory` jeweils 200
(Abschnitt „Verification" unten).

Der Proxy-Eintrag selbst bleibt der einzige Teil dieses Plans, den **kein**
`cargo test` abdeckt. Er gehoert in den manuellen Durchgang von Plan 63-11.

## Neue Tests (9)

`cargo test -p shifty_bin` faehrt jetzt **104** statt 95 Tests.

| Test | Traegt |
|---|---|
| `admin_bekommt_detail_200` | Alle drei Auspraegungen mit `kind`, `display_name`, `username`, `sales_person_id`; dazu der Rueckweg ueber `by-sales-person` mit `assert_eq!` auf die **ganze** Zeile |
| `unbekannte_person_id_ist_404` | D-62-04 auf beiden Lesepfaden (`EntityNotFound` und `EntityNotFoundGeneric`) |
| `hr_ohne_admin_bekommt_403_auf_jeder_route` | T-63-06-A/B: Schleife ueber **alle sieben** Operationen, je 403, je ohne Namens-Preisgabe im Body — plus Gegenprobe, dass kein Schreibvorgang durchgerutscht ist |
| `kombiniertes_anlegen_erzeugt_beide_objekte` | SC-2: 201, Nachlesen der zurueckgegebenen `person_id`, `kind == Linked`, beide Seiten gefuellt |
| `anlegen_ohne_umschalter_ist_422` | D-63-07 / ASVS V5: der Request umgeht den deaktivierten Knopf und wird trotzdem abgelehnt |
| `facet_zuschalten_wechselt_die_auspraegung` | SC-3: `kind` wechselt auf `Linked`, `person_id` bleibt (Benutzer-Seite), die vorhandene Sales Person bleibt unveraendert |
| `facet_entfernen_raeumt_die_person_auf` | SC-5 / D-63-01: 204, danach 404; der verknuepfte Nachbar bleibt unberuehrt (kein Sammelknopf, D-63-05) |
| `kandidatenlisten_blenden_verknuepfte_ziele_aus` | PADM-04: der unverknuepfte Kandidat erscheint, der verknuepfte nicht — beide Richtungen |
| `bereits_verknuepftes_ziel_wird_mit_409_abgelehnt` | SC-4 / D-61-04 / T-63-06-C: der **unveraenderte** Phase-61-Endpoint lehnt ab, und der Zustand bleibt unveraendert |

### Mutations-Nachweis gegen Vakuitaet

Der 403-Durchlauf ist der wertvollste und zugleich der am leichtesten
vakuierende Test dieses Plans — er bestuende auch dann, wenn er gar keine Route
traefe (404 statt 403 haette er allerdings gemeldet) oder wenn `hilde-hr`
in Wahrheit kein gueltiger HR-Benutzer waere.

| Mutation | Erwartung | Ergebnis |
|---|---|---|
| Gate in `remove_facet` von `"admin"` auf `"hr"` geaendert | der Durchlauf faellt auf der DELETE-Operation | `left: 204, right: 403` — bestaetigt |

Der Ausgang **204** (nicht 403, nicht 404) belegt beides auf einmal: die
Schleife erreicht die DELETE-Route wirklich, und `hilde-hr` ist ein
vollwertiger HR-Benutzer — sonst waere auch das mutierte Gate bei 403
geblieben. Die Mutation wurde vollstaendig zurueckgenommen
(`git checkout -- service_impl/src/person_detail.rs`), `cargo test --workspace`
danach gruen.

## Deviations from Plan

### Auto-behoben

**1. [Rule 3 — blockierend] `rest::sales_person` pub-exportiert**

- **Gefunden bei:** Task 3.
- **Problem:** Der Plan verlangt den 409-Regressionstest „gegen den
  UNVERAENDERTEN Phase-61-Endpoint". `rest/src/lib.rs` fuehrte
  `mod sales_person;` privat — `rest::sales_person::generate_route` war aus
  `shifty_bin` nicht erreichbar, der Test also nicht baubar.
- **Fix:** `pub mod sales_person;` mit erklaerendem Kommentar, exakt das Muster
  von `pub mod person_directory;` / `pub mod report;` / `pub mod feature_flag;`,
  die aus demselben Grund pub sind. Die Prohibition des Plans gilt der **Datei**
  `rest/src/sales_person.rs` — die bleibt unveraendert
  (`git diff --name-only HEAD -- rest/src/sales_person.rs` gibt 0 Zeilen).
- **Datei:** `rest/src/lib.rs`
- **Commit:** `54b65ea2`

**2. [Rule 2 — fehlende Abdeckung] Neunter Test fuer die Kandidatenlisten**

- **Gefunden bei:** Task 3, beim Abgleich der Testliste gegen die Routen.
- **Problem:** Die acht im Plan genannten Tests decken die beiden
  Kandidaten-Routen ausschliesslich **negativ** ab (403 im Durchlauf). Ein
  vertauschtes Paar (`sales-persons` liefert Benutzer und umgekehrt) oder eine
  fehlende Ausblendung waere unbemerkt geblieben — genau die Bequemlichkeits-
  Seite von PADM-04/SC-4.
- **Fix:** `kandidatenlisten_blenden_verknuepfte_ziele_aus` ergaenzt; prueft je
  Liste beide Richtungen (unverknuepftes Ziel erscheint, verknuepftes nicht).
- **Datei:** `shifty_bin/src/integration_test/person_detail.rs`
- **Commit:** `54b65ea2`

### Abweichung von einem Grep-Kriterium (nicht vom Verhalten)

Das Akzeptanzkriterium
`grep -c 'error_handler' rest/src/person_detail.rs` **ist gleich der Zahl der
Handler (7)** ergibt tatsaechlich **8**. Die achte Zeile ist die Import-Zeile
`use crate::{error_handler, Context, RestStateDef};` — dieselbe Form, die
`rest/src/person_directory.rs` als woertliche Vorlage vorgibt (dort: 1 Handler,
`grep -c` = 2).

Der Modulkopf hatte das Literal anfangs zusaetzlich in der Prosa getragen (9
Treffer); dieser Satz wurde umformuliert („die zentrale Fehler-Abbildung von
`rest/src/lib.rs`"), wie es die Phase-Erfahrung aus 63-02/63-04 vorgibt. Den
Import zu entfernen und stattdessen `crate::error_handler(...)` zu schreiben,
haette das Kriterium woertlich erfuellt, aber gegen die ausdrueckliche
Plan-Auflage „Muster von `person_directory.rs`, das ist die woertliche Vorlage"
verstossen — das Verhalten waere identisch geblieben. Die praezise Fassung des
Kriteriums lautet:

`grep -c 'error_handler(' rest/src/person_detail.rs` = **7** — erfuellt.

### Reihenfolge der Gates

Task 1 (`rest`) und Task 2 (`shifty_bin`) sind **zwei Commits**, aber der
Workspace ist zwischen ihnen konstruktionsbedingt rot: Task 1 fuegt
`RestStateDef` einen Assoziationstyp samt Accessor hinzu, Task 2 implementiert
ihn. Ein gruener Zwischenstand ist bei dieser Aufteilung nicht herstellbar.
Deshalb wurden beide Tasks implementiert, das volle CLAUDE.md-Gate
(`cargo build` / `cargo test` / `cargo clippy --workspace -- -D warnings`) auf
dem kombinierten Stand gefahren und **danach** in zwei Commits aufgeteilt. Die
Dateizuordnung je Task ist unveraendert die des Plans.

## Auth Gates

Keine.

## Threat Flags

Keine neue Angriffsflaeche ausserhalb des Registers. Abgleich:

| Threat | Disposition | Umsetzung |
|---|---|---|
| T-63-06-A (EoP auf allen sieben Routen) | mitigate | Kein Handler traegt eine Privileg-Pruefung (`grep -c 'check_permission' rest/src/person_detail.rs` = 0); das Gate liegt als erste Anweisung in jeder Service-Methode. Der 403-Durchlauf faehrt alle sieben ab und ist per Mutation als nicht-vakuum belegt. |
| T-63-06-B (direkter URL-Aufruf unter Umgehung der Navigation) | mitigate | Der Backend-Check ist die autoritative Grenze und haengt nicht an der Navigation — der Testdurchlauf ruft die URLs direkt auf. Der Component-Guard in 63-08 ist Defense-in-Depth. |
| T-63-06-C (Wettlauf beim Verknuepfen) | mitigate | Unveraenderter Phase-61-Pfad mit 409; `bereits_verknuepftes_ziel_wird_mit_409_abgelehnt` haelt ihn fest und prueft zusaetzlich, dass kein Teil-Schreibvorgang zurueckbleibt. |
| T-63-06-D (fehlender Proxy-Eintrag) | mitigate | Eintrag im selben Commit wie die DI-Verdrahtung, Reihenfolge-Kriterium automatisiert. Laufzeit-Nachweis nur per `dx serve` (Plan 63-11). Betrifft ausschliesslich die Entwicklungsumgebung. |
| T-63-06-SC (Supply Chain) | mitigate | Keine neue Abhaengigkeit, kein Package-Install. |

## Known Stubs

Keine. Das Frontend ruft die Routen noch nicht auf — das ist planmaessig Sache
von 63-07 bis 63-09 und kein Stub im Sinne einer leeren Anzeige.

## Docs-Freshness-Gate

Beruehrte Trigger-Datei: **`shifty_bin/src/main.rs` (DI-Verdrahtung)**.
`service/**/*.rs`, `dao/**/*.rs`, `migrations/sqlite/*.sql`,
`service_impl/src/permission.rs`, `service_impl/src/reporting.rs` und
`service_impl/src/billing_period_report.rs` wurden **nicht** beruehrt.

| Beruehrt | → faellig |
|---|---|
| `shifty_bin/src/main.rs` (neuer Business-Logic-Service im DI-Graphen) | `docs/architecture/02-service-tiers.md` — `PersonDetailService` als Business-Logic-Tier mit seinen acht Dependencies aufnehmen, inkl. der Begruendung, warum zwei Basic-Services als Deps zyklenfrei sind; **und** `docs/architecture/diagrams/service-graph-runtime.mmd` — Knoten `PersonDetailService` mit Kanten zu `PersonService`, `SalesPersonService`, `PermissionService`, `TransactionDao` sowie den vier DAOs. Beide Sprachen (`*.md` **und** `*_de.md`). |
| (mittelbar) neue REST-Routen | Passende `docs/features/F*.md` (Personenverwaltung) — Endpoint-Tabelle um die sieben `/person`-Operationen erweitern, Auth-Gate-Spalte durchgaengig `admin`, Status-Code-Semantik 201/200/204/403/404/409/422. |

**Nicht in diesem Plan erledigt** — Plan **63-10** fuehrt die
Doku-Aktualisierung der Phase zentral durch (Auflage aus dem Ausfuehrungs-
Auftrag).

## Deferred Issues

Unveraendert gegenueber 63-01/63-02/63-04/63-05:

- `service_impl/src/test/person_directory_dao.rs:149` loest unter
  `cargo clippy --all-targets` eine `needless_lifetimes`-Warnung aus
  (pre-existing, Phase 62, ausserhalb des Scopes; feuert nicht unter dem
  CLAUDE.md-Gate und nicht in CI).
- `cargo fmt` fehlt in der Nix-Dev-Shell (`error: no such command: fmt`); der
  neue Code wurde von Hand rustfmt-konform geschrieben.
- `session.user_id` ohne `ON DELETE CASCADE` (T-63-05-E, accept) — gehoert in
  die Verifikation von Plan 63-11.
- **Neu (Umgebung, kein Code-Defekt):** Auf Port 3000 lief waehrend der
  Ausfuehrung ein **fremder, aelterer** `shifty_bin` (PID 460885) aus einer
  frueheren Sitzung. Ein `cargo run` konnte deshalb nicht binden, und ein
  `curl` gegen `localhost:3000` befragte den alten Server — dessen OpenAPI
  natuerlich keine `/person`-Pfade kennt. Der Laufzeit-Nachweis wurde deshalb
  auf `SERVER_ADDRESS=127.0.0.1:3099` gefahren. Der Fremdprozess wurde bewusst
  **nicht** beendet. Wer in 63-11 einen 404 auf `/person` sieht, pruefe zuerst
  `ss -ltnp | grep 3000`.

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo build --workspace` | Exit 0 |
| `cargo build -p rest` | Exit 0 |
| `cargo build -p shifty_bin` | Exit 0 |
| `cargo test --workspace` | gruen, 0 Failures (958 `service_impl` + **104** Integration + uebrige Suites) |
| `cargo clippy --workspace -- -D warnings` | Exit 0 |
| `cargo test -p shifty_bin person_detail` | **9 passed** (≥ 8 gefordert) |
| `grep -c 'utoipa::path' rest/src/person_detail.rs` | 7 |
| `grep -c 'error_handler(' rest/src/person_detail.rs` | 7 (siehe Deviations) |
| `grep -c 'check_permission' rest/src/person_detail.rs` | 0 |
| `grep -c 'nest("/person"' rest/src/lib.rs` | 1 |
| `grep -c 'person_detail::PersonDetailApiDoc' rest/src/lib.rs` | 1 |
| `grep -c 'localhost:3000/person"' shifty-dioxus/Dioxus.toml` | 1 |
| `grep -n 'localhost:3000/person' … \| tail -1 \| grep -c 'localhost:3000/person"'` | 1 (generischer Eintrag zuletzt) |
| `awk '… person_directory_service … person_detail_service …'` | `ok` (Business-Logic-Reihenfolge) |
| `grep -c 'fn person_detail_service' shifty_bin/src/main.rs` | 1 |
| `grep -c 'bereits_verknuepftes_ziel_wird_mit_409_abgelehnt' …/person_detail.rs` | 1 |
| `grep -c 'StatusCode::FORBIDDEN' …/person_detail.rs` | 1 (im Schleifen-Assert ueber alle sieben Routen) |
| `git diff --name-only HEAD -- rest/src/sales_person.rs \| wc -l` | 0 (Prohibition eingehalten) |
| Laufender Server (`SERVER_ADDRESS=127.0.0.1:3099`) | `/person/candidates/users` → 200, `/person-directory` → 200 |
| OpenAPI-Dokument des laufenden Servers | alle sieben `/person`-Pfade **und** der Tag `PersonDetail` vorhanden |

## Commits

| Task | Commit | Typ |
|---|---|---|
| 1 | `61e2b79f` | `feat(63-06): sieben /person-Routen mit OpenAPI-Annotation` |
| 2 | `b527aaa9` | `feat(63-06): DI-Verdrahtung des PersonDetailService und Dev-Proxy` |
| 3 | `54b65ea2` | `test(63-06): HTTP-Integrationstests fuer Gate, Roundtrip und 409-Regressionsschutz` |

## Self-Check: PASSED

- `rest/src/person_detail.rs` — FOUND
- `shifty_bin/src/integration_test/person_detail.rs` — FOUND
- `rest/src/lib.rs` — FOUND
- `shifty_bin/src/main.rs` — FOUND
- `shifty-dioxus/Dioxus.toml` — FOUND
- Commit `61e2b79f` — FOUND
- Commit `b527aaa9` — FOUND
- Commit `54b65ea2` — FOUND
