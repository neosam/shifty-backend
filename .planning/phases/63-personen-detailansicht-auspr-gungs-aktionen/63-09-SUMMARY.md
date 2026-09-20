---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 09
subsystem: frontend (Rueckbau — Routen, Werkzeugleiste, Zeilen-Aktionen)
tags: [teardown, d-63-03, d-63-04, d-63-07, padm-02, padm-05, padm-07, sc-1, uat-62-befund]
status: complete
requires:
  - "63-07: die neun UserManagementAction-Varianten und die fuenf Store-Felder"
  - "63-08: page::person_detail::{PersonDetail, CreatePersonEntryDialog}, Route::PersonDetail"
  - "63-03: Key::CreatePersonEntry in de/en/cs"
  - "rest_types::CreatePersonEntryTO"
provides:
  - "genau EINE Personen-Detailroute (/person/:person_id/) — die beiden alten sind ersatzlos weg"
  - "genau EINEN Anlege-Einstieg in der Werkzeugleiste der Personenliste"
  - "genau EINEN Bearbeiten-Zielpfad fuer alle drei Auspraegungen"
  - "genau EINEN Loeschort (die Detailseite) — die Zeilen-Loeschung ist entfallen"
  - "vier neue/umgedrehte Tests in page/user_management.rs, eine umgestellte Zusicherung in component/top_bar.rs"
affects:
  - "63-10 (Doku: docs/architecture/06-frontend.md + _de.md — Routen-, Seiten- und Aktions-Uebersicht; Faelligkeit aus 63-07/63-08 waechst)"
  - "63-11 (Human-UAT: die Entscheidung „keine Zeilen-Loeschung\" ist dort ausdruecklich zu bestaetigen)"
tech_stack:
  added: []
  patterns:
    - "Rueckbau als eigener Plan mit eigenen Nachweis-Kriterien: der Beweis ist ein leerer Grep, nicht ein gruener Test"
    - "Weiterleitung nach dem Anlegen ueber einen Store-Uebergang statt ueber einen Rueckgabewert: die Koroutine kann nichts zurueckgeben, also markiert ein Flag den eigenen Anlegevorgang — und der vorige Detail-Eintrag wird vorher geraeumt, damit kein Ergebnis eines frueheren Besuchs dafuer gehalten wird"
    - "`peek` statt `read` fuer das Anlege-Flag im Effekt: das Setzen des Flags darf den Effekt nicht ausloesen, sonst liefe er mit dem Stand vor dem Anlegen"
key_files:
  created: []
  modified:
    - shifty-dioxus/src/page/user_management.rs
    - shifty-dioxus/src/service/user_management.rs
    - shifty-dioxus/src/component/top_bar.rs
    - shifty-dioxus/src/router.rs
    - shifty-dioxus/src/page/mod.rs
    - shifty-dioxus/src/page/person_detail.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/api.rs
  deleted:
    - shifty-dioxus/src/page/user_details.rs
    - shifty-dioxus/src/page/sales_person_details.rs
decisions:
  - "Task-Reihenfolge umgedreht (3 → 2 → 1). Die Plan-Reihenfolge erzeugt einen nicht kompilierenden Zwischenstand: Task 1 verlangt `cargo build` Exit 0 UND einen leeren Grep auf die alten Routen, waehrend zwei andere Dateien sie noch referenzieren. In Plan-Reihenfolge haette Task 1 die Arbeit von Task 2 und 3 mit absorbieren muessen."
  - "Nicht nur `CreateNewSalesPerson` entfernt, sondern auch `AddUser`, `DeleteUser`, `UpdateSalesPersonUser` und `RemoveSalesPersonUser`: sie alle verlieren mit diesem Rueckbau ihren letzten Absender. Die Auflage lautete, die Totcode-Fallout NICHT mit `#[allow(dead_code)]` zuzudecken."
  - "Die 14 durch den Rueckbau verwaisten i18n-Keys bleiben registriert und werden hier nur benannt. Sie sind Uebersetzungsdaten in drei Sprachen und stehen unter dem Key-Mengen-Test; ein Loeschen waere ein Eingriff in eine geteilte Datei ohne funktionalen Gewinn."
  - "`SelectedSalesPerson::user_id` und der Benutzer-Abgleich in `save_sales_person()` bleiben stehen, obwohl sie nach diesem Plan nie mehr abweichen koennen. Der Compiler sieht sie nicht als tot, und ein Umbau des Speicher-Pfades waere echte Drift. Als Deferred Item notiert."
metrics:
  duration_minutes: 25
  tasks_completed: 3
  files_modified: 8
  files_deleted: 2
  tests_added: 3
  tests_removed: 4
  completed: 2026-09-21
---

# Phase 63 Plan 09: Der Rueckbau — Summary

Ein Anlege-Einstieg, ein Bearbeiten-Ziel, ein Loeschort, eine Detailroute.
Dieser Plan hat nichts gebaut, sondern 1115 Zeilen entfernt und 297
geschrieben — und der Nachweis dafuer sind vier leere Greps, nicht vier gruene
Tests.

## Was verschwunden ist

| Art | Symbol | Grund |
|---|---|---|
| Route | `/user_details/:user_id/`, `/sales_person_details/:sales_person_id/` | D-63-04 — eine Detailroute fuer alle drei Auspraegungen |
| Seite | `page/user_details.rs` (333 Z.), `page/sales_person_details.rs` (339 Z.) | vollstaendig in `page/person_detail.rs` aufgegangen (63-08) |
| Toolbar | Sekundaer-Link „Neuen Verkaeufer erstellen", Primaer-Knopf „Neuen Benutzer hinzufuegen" | D-63-03 — der UAT-Befund im Wortlaut: „Das checkt niemand." |
| Dialog | `AddUserDialog`, `DeleteUserConfirmDialog` samt Props und vier Rendertests | ersetzt durch `CreatePersonEntryDialog` bzw. ersatzlos |
| Zeilen-Aktion | Danger-Knopf 🗑 in der Aktionsspalte | ihr generischer Bestaetigungstext kann die Zusage aus PADM-07 nicht halten |
| Action | `CreateNewSalesPerson`, `AddUser`, `DeleteUser`, `UpdateSalesPersonUser`, `RemoveSalesPersonUser` | alle fuenf verlieren hier ihren letzten Absender |
| Service-Fn | `add_user`, `delete_user` (service), `add_user`, `delete_user`, `load_sales_person_by_user` (loader), `add_user`, `delete_user`, `get_sales_person_by_user` (api) | dito |
| Marker | das letzte `#[allow(dead_code)]` aus 63-07 (`CreatePersonEntry`) | die Werkzeugleiste ist jetzt sein Absender |

An jeder entfernten Stelle steht ein Kommentar, der sagt **warum** sie
entfallen ist und **wohin** ihre Aufgabe gewandert ist — nicht nur, dass sie
weg ist. Die Bezeichner der beiden alten Routen kommen dabei nirgends mehr vor
(Auflage aus D-63-04).

## Was dazugekommen ist

Genau ein Primaer-Knopf `Key::CreatePersonEntry` in der Werkzeugleiste. Sein
Klick oeffnet den `CreatePersonEntryDialog` aus 63-08; die Seite haelt die vier
Eingabewerte als Signale und schickt beim Bestaetigen
`UserManagementAction::CreatePersonEntry`.

**Die Weiterleitung nach dem Anlegen (D-63-07)** ist der einzige nicht-triviale
Teil dieses Plans. Eine Dioxus-Koroutine kann nichts zurueckgeben — der
Anlege-Pfad des Service laedt die vom Backend gelieferte `person_id` nach und
legt die Zeile in `store.person_detail` ab. Die Seite wartet auf genau diesen
Uebergang. Zwei Fallen, beide mit Kommentar an Ort und Stelle entschaerft:

1. **Ein Eintrag aus einem frueheren Besuch.** `person_detail` ueberlebt das
   Verlassen der Detailseite. Ohne Vorkehrung haette der erste Effekt-Lauf
   nach dem Klick die alte Zeile fuer das Ergebnis gehalten und auf die
   falsche Person weitergeleitet. Der Klick-Handler raeumt das Feld deshalb,
   **bevor** er das Flag setzt.
2. **Das Flag selbst als Ausloeser.** Wird `create_pending` mit `read`
   gelesen, loest sein Setzen den Effekt aus — mit dem Stand vor dem Anlegen.
   Es wird deshalb mit `peek` gelesen; ausgeloest wird der Effekt
   ausschliesslich von Store-Aenderungen.

Der Klick-Handler wiederholt die Sperr-Bedingung des Dialogs (mindestens ein
Umschalter; Benutzername, solange die Login-Seite an ist). Ein Kommentar sagt,
dass beides Bequemlichkeit ist und verbindlich der Service prueft (ASVS V5).

## Der Bearbeiten-Link: der sichtbare Beleg fuer Success Criterion 1

Phase 62 musste hier noch verzweigen — Verkaeufer-Seite vorhanden ⇒
Verkaeufer-Detailroute, sonst Benutzer-Detailroute (D-62-07, „Phase 62 legt
keine Detail-Route an"). Die **Zeile** musste also wissen, welche Haelfte einer
Person die richtige Seite ist. Jetzt kennt sie nur noch die Identitaet des
Eintrags; welche Abschnitte erscheinen, entscheidet erst die Zielseite anhand
des Backend-`kind`.

`directory_edit_link_targets_one_route_for_all_kinds` haelt das fest: je
Auspraegung einzeln gerendert, je genau **ein** `href` auf
`Route::PersonDetail` mit der `person_id` des Eintrags.

## Der Loeschort

| | Vorher | Jetzt |
|---|---|---|
| Orte | Zeile der Liste **und** (ab 63-08) Detailseite | nur Detailseite |
| Text in der Zeile | „Soll Benutzer X wirklich geloescht werden? Das kann nicht rueckgaengig gemacht werden." | — |
| Text auf der Detailseite | je Auspraegung getrennt, mit Letzte-Auspraegung-Zusatz (D-63-08) | unveraendert |
| Verkaeufer-Zeile | konnte gar nicht geloescht werden (nur Zeilen mit Login trugen den Knopf) | beide Seiten auf der Detailseite, getrennt bestaetigt |

Der alte Zeilen-Text nennt weder die Rollen und offenen Einladungen, die beim
Login-Loeschen mit verschwinden, noch hat er ein Soft-Delete-Gegenstueck. Er
kann die Zusage aus PADM-07 also nicht halten. Die Alternative — dieselben
zwei D-63-08-Textbausteine ein zweites Mal in einen Zeilen-Dialog kopieren —
haette zwei Loeschwege ohne erkennbaren kanonischen ergeben.

Zwei Tests sichern die Abwesenheit, in beide Richtungen:
`directory_row_has_no_delete_path` prueft je Auspraegung, dass weder das
Piktogramm 🗑 noch die Danger-Tokens (`text-bad`, `border-bad`) im Markup
stehen; `directory_actions_only_for_user_side` behaelt seine
Impersonations-Zusicherung und hat ihre Loesch-Haelfte verloren.

## Deviations from Plan

### Auto-behoben

**1. [Rule 3 — blockierend] Task-Reihenfolge umgedreht: 3 → 2 → 1**

- **Gefunden bei:** Planung des ersten Commits, vor der ersten Aenderung.
- **Problem:** Task 1 verlangt gleichzeitig `cargo build` Exit 0 **und**
  `grep -rn 'Route::UserDetails\|Route::SalesPersonDetails' shifty-dioxus/src
  | wc -l` = 0. Beide alten Routen werden aber zum Zeitpunkt von Task 1 noch
  von `component/top_bar.rs` (Task 2) und `page/user_management.rs` (Task 3)
  referenziert. In Plan-Reihenfolge haette Task 1 die Arbeit beider
  Folge-Tasks mit absorbieren muessen — ein Commit statt drei, und die
  Nachweis-Kriterien von Task 2 und 3 waeren bereits vor ihrer Ausfuehrung
  erfuellt gewesen.
- **Fix:** Task 3 zuerst (die Liste hoert auf, die alten Routen zu nennen),
  dann Task 2 (die obere Leiste hoert auf), dann Task 1 (die Routen und ihre
  Seiten fallen). Jeder der drei Commits baut, testet und clippyt gruen —
  genau die Atomaritaet, die die Plan-Reihenfolge verhindert haette.
- **Dateien:** alle acht.
- **Commits:** `043fffe2`, `34423b47`, `d1ec86fc`

**2. [Rule 3 — blockierend] Fuenf Aktionen statt einer entfernt, plus acht Hilfsfunktionen**

- **Gefunden bei:** Task 3 und Task 1, jeweils beim `cargo build`.
- **Problem:** Der Plan nennt ausdruecklich nur `CreateNewSalesPerson`
  („die Aktions-Variante, die eine leere Verkaeufer-Vorlage in den Store
  legte"). Tatsaechlich verlieren fuenf Varianten ihren letzten Absender:
  - `AddUser` und `DeleteUser` hingen an den beiden entfernten Dialogen der
    Listenseite;
  - `CreateNewSalesPerson` am Anlege-Link mit leerer Kennung;
  - `UpdateSalesPersonUser` und `RemoveSalesPersonUser` an den
    Verknuepfungs-Bedienelementen der abgeloesten Verkaeufer-Seite.

  Mit ihnen fallen `service::{add_user, delete_user}`,
  `loader::{add_user, delete_user, load_sales_person_by_user}` und
  `api::{add_user, delete_user, get_sales_person_by_user}` — rustc meldete
  sie nacheinander als „never used".
- **Verworfene Alternative:** `#[allow(dead_code)]` darueber. Der
  Ausfuehrungsauftrag verbietet das ausdruecklich („delete what actually
  became unreachable"), und es waere ausgerechnet in dem Plan geschehen, der
  das letzte solche Marker aus 63-07 entfernt.
- **Fachliche Deckung:** Jede der fuenf ist ersetzt, nicht verloren.
  Angelegt wird ueber `create_person_entry` (PADM-02 — eine Transaktion ueber
  beide Auspraegungen statt zweier Einstiege mit je einer Haelfte), entfernt
  ueber `remove_person_facet` (PADM-07 — mit ausprägungs-bewusster
  Bestaetigung und dem Orphan-Cleanup aus D-63-01), verknuepft und geloest
  ueber `LinkCandidate`/`UnlinkFacet` (PADM-04/PADM-07 — sofort statt als
  Nebenwirkung eines Speichern-Knopfes). Das clientseitige „erst
  entknuepfen, dann loeschen" des alten `delete_user` war ein Zwei-Schritt
  ohne Rollback; derselbe Ablauf laeuft jetzt serverseitig in einer
  Transaktion.
- **Dateien:** `service/user_management.rs`, `loader.rs`, `api.rs`
- **Commits:** `043fffe2`, `d1ec86fc`

### Abweichungen von der Plan-Prosa (nicht vom Verhalten)

**`component/top_bar.rs` fuehrt keine Breadcrumb-Leiste.** Task 2 nennt „die
Breadcrumb-Zweige der drei Routen" in den Zeilen 870-900. An dieser Stelle
steht in Wahrheit der Unit-Test `is_active_for_user_management_matches_all_
three_variants`; `grep -in breadcrumb` ueber die gesamte Codebasis gibt null
Treffer. Die zwei Fundstellen der alten Routen in dieser Datei waren also der
Aktiv-Zweig (Zeile 94/95) und seine Zusicherung (Zeile 883/889) — beide sind
umgestellt. Der Test heisst jetzt `..._matches_list_and_detail` und prueft
Liste, eine Detailroute und eine Negativ-Richtung. Das Kriterium
`grep -c 'Route::PersonDetail' top_bar.rs >= 2` ist damit ohne Kunstgriff
erfuellt (Aktiv-Zweig + Zusicherung = 2).

Die zweite Haelfte der Task-2-Auflage — „den Anzeigenamen aus dem
Personen-Store, falls er dort verfuegbar ist" — entfaellt mit dem Befund: es
gibt kein Element, das einen Titel anzeigen koennte. Die neue Route markiert
den Personenverwaltungs-Punkt als aktiv, mehr traegt die Leiste nicht. Keine
Ableitungsregel fuer einen Namen gebaut (PADM-06) — es gab nichts, wofuer.

**Vier Tests entfernt, drei hinzugefuegt (netto 857 → 856).** Die vier
Rendertests von `AddUserDialog` und `DeleteUserConfirmDialog` sind mit ihren
Dialogen entfallen; ein Kommentar an ihrer Stelle sagt, wohin ihre Aufgabe
gewandert ist (der Anlege-Dialog wird seit 63-08 in `page/person_detail.rs`
geprueft, der Loesch-Dialog existiert nicht mehr und wird in umgekehrter
Richtung von `directory_row_has_no_delete_path` abgedeckt). Neu sind
`directory_row_has_no_delete_path`,
`directory_toolbar_has_exactly_one_create_entry_point` und
`directory_edit_link_targets_one_route_for_all_kinds`.

**Kein Test schreibt die alten Pfade.** Die Prohibition verlangt
`grep -rn 'user_details/\|sales_person_details/' shifty-dioxus/src` = 0 —
**einschliesslich** des Testmoduls. Die neuen Tests weisen ihre Sache deshalb
positiv nach (genau ein `href` auf die neue Route), nie durch die Abwesenheit
eines alten Pfad-Literals.

## Auth Gates

Keine.

## Threat Flags

Keine neue Angriffsflaeche. Abgleich mit dem Register des Plans:

| Threat | Disposition | Umsetzung |
|---|---|---|
| T-63-09-A (Repudiation ueber den schwaecheren Loeschweg) | mitigate | Der Zeilen-Knopf ist weg; `directory_row_has_no_delete_path` prueft je Auspraegung auf Piktogramm **und** Danger-Tokens. Der verbleibende Weg traegt die Texte aus D-63-08. Prohibitions-Grep auf den entfallenen Dialog gibt 0. |
| T-63-09-B (EoP ueber die guard-lose alte Route) | mitigate | Beide alten Seiten hatten keinen Component-Guard und sind geloescht; `page/person_detail.rs` traegt ihn (63-08). Der Rueckbau schliesst die Luecke, statt sie nur zu verdecken. |
| T-63-09-C (Lesezeichen auf die alten Pfade) | accept | Unveraendert bewusst: keine Weiterleitungs-Huellen (D-63-04). Ein Kommentar an der Routen-Variante haelt fest, dass das Nachruesten moeglich bleibt — damit es keine stille Schuld ist. |
| T-63-09-D (zurueckgebliebene tote Aktionen) | mitigate | Fuenf Varianten und acht Funktionen entfernt; `cargo clippy --all-targets -- -D warnings` ist gruen, ohne ein einziges neues `#[allow(dead_code)]`. |
| T-63-09-SC (Supply Chain) | mitigate | Keine neue Abhaengigkeit, kein Package-Install. `git diff HEAD~3 HEAD` beruehrt keine `Cargo.toml` und kein Lockfile. |

## Known Stubs

Keine. Dieser Plan hat nichts angelegt, das noch auf eine Verdrahtung wartet —
im Gegenteil, er hat die letzte offene Verdrahtung aus 63-08 geschlossen
(`CreatePersonEntryDialog` hat jetzt seinen Oeffner, und
`UserManagementAction::CreatePersonEntry` seinen Absender).

## Verwaiste i18n-Keys (nicht geloescht, nur benannt)

Vierzehn Keys verlieren mit diesem Rueckbau ihren Renderer. Sie bleiben in
`de.rs`/`en.rs`/`cs.rs` registriert: sie sind Uebersetzungsdaten in drei
Sprachen, stehen unter dem Key-Mengen-Test aus 63-03, und ein Loeschen waere
ein Eingriff in eine geteilte Datei ohne funktionalen Gewinn. Der Rueckbau der
Uebersetzungsdaten gehoert, wenn ueberhaupt, in einen eigenen Durchgang.

| Herkunft | Keys |
|---|---|
| Werkzeugleiste | `CreateNewSalesPerson`, `AddNewUser` |
| Anlege-Dialog (alt) | `CreateUser` |
| Zeilen-Loeschung | `DeleteUser`, `DeleteUserConfirmTitle`, `DeleteUserConfirmBody` |
| `user_details.rs` | `UserDetails`, `ManageRolesAndPermissions` |
| `sales_person_details.rs` | `SalesPersonDetails`, `CreateNewSalesPersonTitle`, `EditSalesPersonInformation`, `SalesPersonSavedSuccessfully`, `UserAccount`, `ConnectUserAccount` |

Die Codebasis fuehrte vor diesem Plan bereits rund 45 renderer-lose Keys
(u. a. die `PersonKind*`-Trias und `ColumnLinked*` aus Phase 62). Die Zahl ist
also kein neues Phaenomen — aber jetzt benannt.

## Docs-Freshness-Gate

Beruehrte Trigger-Dateien laut CLAUDE.md-Tabelle: **keine.**
`shifty_bin/src/main.rs`, `service/**`, `dao/**`, `migrations/sqlite/*.sql`,
`service_impl/src/permission.rs`, `service_impl/src/reporting.rs` und
`service_impl/src/billing_period_report.rs` wurden nicht angefasst.

Zusaetzlich faellig (Frontend):

| Beruehrt | → faellig |
|---|---|
| `shifty-dioxus/src/router.rs` (zwei Routen entfallen) | `docs/architecture/06-frontend.md` **und** `_de.md` — Routen-Uebersicht: `/user_details/` und `/sales_person_details/` streichen, `/person/:person_id/` als einziges Detail-Ziel fuehren, die bewusste Abwesenheit von Weiterleitungen erwaehnen |
| geloeschte `page/user_details.rs`, `page/sales_person_details.rs` | `docs/architecture/06-frontend.md` + `_de.md` — Seiten-Uebersicht |
| `shifty-dioxus/src/service/user_management.rs` (fuenf Aktionen weg) | `docs/architecture/06-frontend.md` + `_de.md` — soweit dort Aktionen gelistet sind; ergaenzt die von 63-07 vermerkte Faelligkeit |
| `shifty-dioxus/src/{loader,api}.rs` (drei API-Primitive weg) | keine eigene Sektion; die Backend-Endpoints sind unveraendert |

**Nicht in diesem Plan erledigt** — Plan **63-10** fuehrt die
Doku-Aktualisierung der Phase zentral durch (Auflage aus dem
Ausfuehrungsauftrag).

## Deferred Issues

Unveraendert aus 63-08 uebernommen:

- `service_impl/src/test/person_directory_dao.rs:149` loest unter
  `cargo clippy --all-targets` eine `needless_lifetimes`-Warnung aus
  (pre-existing, Phase 62, ausserhalb des Scopes).
- `cargo fmt` fehlt in der Nix-Dev-Shell; der Code wurde von Hand
  rustfmt-konform geschrieben.
- `Key::CopyToClipboard` ist in `i18n/de.rs` und `i18n/en.rs` je zweimal
  registriert (aus 63-03 bekannt).
- Der `[[web.proxy]]`-Eintrag fuer `/person` bleibt der einzige Teil der Kette
  ohne `cargo test`-Abdeckung — Laufzeit-Nachweis nur per `dx serve`
  (Plan 63-11).
- Die Reihenfolge der `visible_sections_*`-Tests am Ende von
  `page/person_detail.rs` ist ein Kriterium-Artefakt, kein fachlicher Zwang.
- **Umgebung, kein Code-Defekt:** auf Port 3000 laeuft weiterhin ein fremder,
  aelterer `shifty_bin` (PID 460885). Bewusst nicht beendet.

Neu:

- **Vestigial, bewusst stehen gelassen:** `SelectedSalesPerson::user_id` wird
  von `load_sales_person` gefuellt, aber seit diesem Plan von niemandem mehr
  veraendert — die beiden Aktionen, die das taten, sind entfallen. Der
  Benutzer-Abgleich in `save_sales_person()` (`(Some,Some)`, `(Some,None)`,
  `(None,Some)`) kann damit nie mehr einen anderen Zweig als `_ => {}`
  nehmen. Der Compiler sieht das nicht; ein Umbau des Speicher-Pfades waere
  in einem Rueckbau-Plan echte Drift. Kandidat fuer eine spaetere
  Aufraeum-Runde.
- **Vierzehn verwaiste i18n-Keys** (Tabelle oben) — bewusst nicht geloescht.
- **Bewusste Loeschungen im Diff:** `git diff --diff-filter=D HEAD~3 HEAD`
  meldet genau zwei Dateien, `page/user_details.rs` und
  `page/sales_person_details.rs`. Beide sind das erklaerte Ziel von Task 1
  (D-63-04); keine unbeabsichtigte Loeschung.

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo build --manifest-path shifty-dioxus/Cargo.toml` | Exit 0, **0 Warnungen** |
| `cargo test --manifest-path shifty-dioxus/Cargo.toml` | **856 passed**, 0 failed (857 − 4 + 3) |
| `cargo test … user_management` | 19 passed |
| `cargo test … directory_row_encodes_kind_without_badge` | **1 passed** (Prohibition: unveraendert gruen) |
| `cargo clippy --manifest-path shifty-dioxus/Cargo.toml -- -D warnings` | Exit 0 |
| `cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings` | Exit 0 |
| `cargo build --manifest-path shifty-dioxus/Cargo.toml --target wasm32-unknown-unknown` | Exit 0 |
| `cargo build --workspace` | Exit 0 |
| `cargo test --workspace` | alle gruen (u. a. 958 + 104 + 33) |
| `cargo clippy --workspace -- -D warnings` | Exit 0 |
| `test -f shifty-dioxus/src/page/user_details.rs` | **1** (nicht vorhanden) |
| `test -f shifty-dioxus/src/page/sales_person_details.rs` | **1** (nicht vorhanden) |
| `grep -rn 'Route::UserDetails\|Route::SalesPersonDetails' shifty-dioxus/src \| wc -l` | **0** |
| `grep -rn 'user_details/\|sales_person_details/' shifty-dioxus/src \| wc -l` | **0** (D-63-04-Nachweis) |
| `grep -c 'PersonDetail' shifty-dioxus/src/router.rs` | 2 (gefordert: ≥ 2) |
| `grep -c 'Route::UserDetails\|Route::SalesPersonDetails' component/top_bar.rs` | **0** |
| `grep -c 'Route::PersonDetail' component/top_bar.rs` | 2 (gefordert: ≥ 2) |
| `grep -c 'CreateNewSalesPerson\|AddNewUser' page/user_management.rs` | **0** |
| `grep -c 'DeleteUserConfirmDialog' page/user_management.rs` | **0** |
| `grep -c 'Route::PersonDetail' page/user_management.rs` | 3 (gefordert: ≥ 1) |
| `grep -c 'allow(dead_code)' service/user_management.rs` | **0** (vorher 1) |

## Commits

| Task | Commit | Typ |
|---|---|---|
| 3 | `043fffe2` | `feat(63-09): ein Anlege-Einstieg, ein Bearbeiten-Ziel, kein Zeilen-Löschweg` |
| 2 | `34423b47` | `refactor(63-09): obere Leiste kennt nur noch die eine Personen-Detailroute` |
| 1 | `d1ec86fc` | `refactor(63-09): die beiden alten Detailrouten und ihre Seiten ersatzlos entfernt` |

## Self-Check: PASSED

- `shifty-dioxus/src/page/user_management.rs` — FOUND
- `shifty-dioxus/src/service/user_management.rs` — FOUND
- `shifty-dioxus/src/component/top_bar.rs` — FOUND
- `shifty-dioxus/src/router.rs` — FOUND
- `shifty-dioxus/src/page/mod.rs` — FOUND
- `shifty-dioxus/src/page/user_details.rs` — GONE (erwuenscht)
- `shifty-dioxus/src/page/sales_person_details.rs` — GONE (erwuenscht)
- Commit `043fffe2` — FOUND
- Commit `34423b47` — FOUND
- Commit `d1ec86fc` — FOUND
