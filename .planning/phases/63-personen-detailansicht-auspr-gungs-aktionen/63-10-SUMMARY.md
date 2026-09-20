---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 10
subsystem: documentation (Docs-Freshness-Gate der Phase)
tags: [docs, docs-freshness-gate, d-63-01, d-63-05, d-61-03, padm-05, padm-07, en-de-parity]
status: complete
requires:
  - "63-01: PermissionDao mit Transaction-Assoziationstyp, PermissionServiceImpl mit TransactionDao"
  - "63-02: PersonDao::{delete, has_live_facets, find_by_sales_person, find_by_user}, PersonDirectoryDao::find_directory_entry"
  - "63-04/63-05: service::person_detail::PersonDetailService mit sieben Methoden"
  - "63-06: DI-Verdrahtung in shifty_bin/src/main.rs, die sieben /person-Routen"
  - "63-07/63-08/63-09: shifty-dioxus — acht /person-Aufrufe, page/person_detail.rs, Rueckbau der zwei alten Routen/Seiten"
provides:
  - "docs/features/F01-employee-management.{md,_de.md} — neue Sektionen 4.7 (PersonDetailService) und 5.5 (REST /person), erweiterte Datenmodell-, Frontend-, Randfall- und Historien-Sektionen"
  - "docs/architecture/02-service-tiers.{md,_de.md} — PersonDetailService als Business-Logic mit acht Dependencies"
  - "docs/architecture/diagrams/service-graph-runtime.mmd — Knoten PersonDetail plus zwei Domain-Service-Kanten"
  - "docs/architecture/04-auth.{md,_de.md} — die transaktionsfaehigen PermissionDao-Schreibpfade und die unveraenderte PermissionService-Oberflaeche"
  - "docs/architecture/05-transactions.{md,_de.md} — PermissionDao als bewusst gemischtes Trait; Commit-Semantik per Arc::into_inner belegt"
  - "docs/architecture/06-frontend.{md,_de.md} — Routen-/Seiten-Inventar, Konvention 'Entscheidungslogik in reinen Funktionen', die acht /person-Aufrufe und die 404/409/422-Unterscheidung"
  - "docs/architecture/03-data-model.{md,_de.md} — Lebenszyklus der person-Zeile (D-63-01)"
  - "docs/features/F12-auth-session.{md,_de.md} — korrigierte create_user-Signatur, TX-Verhalten, Dependencies, Seiten-Inventar"
  - "docs/features/F10-templates-communication.{md,_de.md} — Invitation-Panel zeigt auf die existierende Seite"
affects:
  - "63-11 (Human-UAT: die Doku beschreibt jetzt die Randfaelle, die dort zu bestaetigen sind — insbesondere der person_id-Wechsel und die Hard-vs-Soft-Delete-Asymmetrie)"
  - "Milestone-Close v2.7 (das harte Docs-Gate aus PROJECT.md ist fuer Phase 63 geschlossen)"
tech_stack:
  added: []
  patterns:
    - "Doku-Behauptung nur mit Code-Deckung: jede genannte Methode, Route, Datei und Zeilennummer per Grep gegen die Codebasis geprueft, bevor sie stehen bleibt"
    - "Dangling-Reference-Jagd ueber den ganzen docs/-Baum statt nur in den files_modified — ein Rueckbau macht Doku in fremden Features falsch"
    - "[To verify]-Marker aufloesen, wenn die Phase die Antwort ohnehin belegt hat (Commit-Semantik per Arc::into_inner)"
key_files:
  created: []
  modified:
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
    - docs/architecture/02-service-tiers.md
    - docs/architecture/02-service-tiers_de.md
    - docs/architecture/diagrams/service-graph-runtime.mmd
    - docs/architecture/03-data-model.md
    - docs/architecture/03-data-model_de.md
    - docs/architecture/04-auth.md
    - docs/architecture/04-auth_de.md
    - docs/architecture/05-transactions.md
    - docs/architecture/05-transactions_de.md
    - docs/architecture/06-frontend.md
    - docs/architecture/06-frontend_de.md
    - docs/features/F12-auth-session.md
    - docs/features/F12-auth-session_de.md
    - docs/features/F10-templates-communication.md
    - docs/features/F10-templates-communication_de.md
decisions:
  - "Die Frontend-Doku hatte bis heute gar kein Routen-/Seiten-Inventar. Statt nur 'die zwei alten Routen streichen' (es gab nichts zu streichen) ist eines angelegt worden — sonst waere das Akzeptanzkriterium 'grep person/ != 0' formal erfuellbar, aber die eigentliche Auflage aus 63-08/63-09 nicht."
  - "03-data-model wurde erweitert, obwohl der Plan sie nicht in files_modified fuehrt. Ihr key_links-Eintrag verlangt ausdruecklich 'einen Satz zur Prosa des person-Lebenszyklus' — und die vorhandene Aussage 'in v2.7 wird keine Person geloescht' war seit 63-05 schlicht falsch."
  - "F10 wurde korrigiert, obwohl sie nicht zur Phase gehoert: sie zeigte an vier Stellen auf page/user_details.rs, eine Datei, die 63-09 geloescht hat. Eine Doku, die auf ein nicht existierendes Symbol zeigt, ist die Fehlerklasse, die dieses Gate verhindern soll."
  - "Der [To verify]-Marker in 05-transactions zur Commit-Semantik ist aufgeloest statt stehen gelassen worden. Genau diese Invariante (innerer Commit ist No-op, solange ein anderer Halter den Arc haelt) ist die Voraussetzung fuer die Ein-Transaktions-Zusage aus SC-2; sie ungeprueft zu lassen waere hier besonders schief gewesen."
  - "Der Phase-61-Kommentar im mmd wurde praezisiert statt ueberschrieben: 'keine Kante' ist zu 'keine AUSGEHENDE Kante' geworden, weil PersonDetail --> Person die erste eingehende ist — und die Richtung ist genau die erlaubte."
  - "Die 14 verwaisten i18n-Keys sind in F01 als 'registriert, aber unbenutzt' benannt statt verschwiegen. Drei davon (SalesPersonDetails, UserAccount, ConnectUserAccount) sind stichprobenhaft per Grep als unbenutzt belegt."
metrics:
  duration_minutes: 58
  tasks_completed: 3
  files_modified: 17
  tests_added: 0
  completed: 2026-09-21
---

# Phase 63 Plan 10: Docs-Freshness-Gate der Phase schliessen — Summary

Siebzehn Doku-Dateien auf den Stand der Phase gebracht, paarweise EN und DE — und
dabei drei Aussagen gefunden, die durch Phase 63 nicht unvollstaendig, sondern
**falsch** geworden waren.

## Was die drei Tasks gemacht haben

**Task 1 — F01 (`41384e9c`).** Zwei neue Sektionen und vier erweiterte:

| Sektion | Inhalt |
|---|---|
| §3 `PersonDao` | vier neue Methoden; der Lebenszyklus der `person`-Zeile mit D-63-01-Lesart (b) und dem ausdruecklich akzeptierten haengenden `person_id` |
| §3 `PersonDirectoryDao` | `find_directory_entry` und warum `None` fuer unbekannt *und* traegerlos die richtige Antwort ist |
| §4.7 **neu** | Trait-Auszug (sieben Methoden), Detail-Aggregat samt N+1-Begruendung, Anlege-Pfad, Zuschalt-Asymmetrie, Loesch-Semantik-Tabelle, Kandidaten-Lesepfade, Auth-Gate-Tabelle (7× `admin`), Tier-Begruendung mit acht Deps, die Session-Kante |
| §5.5 **neu** | die sieben `/person`-Operationen mit Statuscodes und der 404/409/422-Unterscheidung |
| §6 | `page/person_detail.rs` statt der zwei entfernten Seiten, der `/person`-Proxy-Eintrag samt Reihenfolge-Falle, die i18n-Wanderung |
| §7, §9 | vier neue Randfaelle, Phase-62/63-Absatz in der Historie |

**Task 2 — Tiers, Graph, Auth, Transaktionen (`779efb25`).** Neun Dateien. Die
inhaltliche Kernaussage steht dreimal, weil sie dreimal jemanden abholt: das
Gate liegt auf `admin`, die `PermissionService`-Oberflaeche ist eingefroren, und
der Transaktions-Assoziationstyp sitzt deshalb eine Ebene tiefer auf dem DAO.

**Task 3 — Frontend und Konsistenz-Durchlauf (`74a97f6e`).** Routen-/Seiten-
Inventar, Testbarkeits-Konvention, die acht `/person`-Aufrufe, der
Verkaeufer-Loeschpfad D-63-06 — plus die Korrektur in F10.

**Nachzug — Datenmodell (`716c077b`).** Siehe Deviations.

## Drei Aussagen, die falsch waren (nicht nur unvollstaendig)

Das ist der eigentliche Ertrag dieses Plans — eine luechenhafte Doku kostet Zeit,
eine falsche kostet eine Fehlentscheidung:

1. **`docs/architecture/03-data-model.md`:** „In v2.7 wird keine Person
   geloescht." Seit 63-05 loescht `remove_facet` die `person`-Zeile hart, wenn
   der letzte lebende Traeger geht. Wer das Schema-Dokument liest, um zu
   entscheiden, ob er einen FK ergaenzen darf, haette hier genau die falsche
   Grundlage gehabt.
2. **`docs/features/F12-auth-session.md` §3:** „Die Trait-Signatur
   `create_user(&self, user: &UserEntity, process: &str)` ist unveraendert — die
   Transaktion ist Interna, kein neuer Parameter." Seit 63-01 sind es fuenf
   Parameter inklusive `person_id` und `tx`, und die `pool.begin()` ist weg.
3. **`docs/features/F10-templates-communication.md` §6.2:** vier Verweise auf
   `page/user_details.rs` — eine Datei, die 63-09 geloescht hat.

Dazu zwei Halbwahrheiten: F12 §4.6 („weder `PermissionService` noch
`SessionService` verwenden `Transaction`-Parameter" — fuer den Service stimmt
es, fuer den DAO seit 63-01 nicht) und §4.7 (`PermissionServiceImpl` hat drei
Dependencies, nicht zwei).

## Abgleich gegen die neun Vorgaenger-SUMMARYs

Jeder „faellig fuer 63-10"-Eintrag, einzeln quittiert:

| Aus | Faelligkeit | Status |
|---|---|---|
| 63-01 | `04-auth` + `F12` — geaenderte `PermissionDao`-Signaturen | erledigt (Task 2) |
| 63-01 | `02-service-tiers` + `service-graph-runtime.mmd` — DI | erledigt (Task 2) |
| 63-02 | Datenmodell-Sektion der Personen-Feature-Doku | erledigt (Task 1, F01 §3) |
| 63-03 | — keine Trigger-Datei, keine Faelligkeit vermerkt | nichts zu tun; die 32 neuen i18n-Keys sind in F01 §6 dennoch benannt |
| 63-04 | F01 — Trait-Auszug, Auth-Gate (`admin`), Randfall „traegerlose Person → 404" | erledigt (Task 1, §4.7) |
| 63-05 | F01 — drei Mutationen + `get_detail_by_sales_person`, `admin` fuer alle vier, fuenf Randfaelle | erledigt (Task 1, §4.7 + §7) |
| 63-06 | `02-service-tiers` + `.mmd` — acht Deps, Zyklenfreiheit | erledigt (Task 2) |
| 63-06 | F01 — Endpoint-Tabelle, sieben Operationen, Statuscodes | erledigt (Task 1, §5.5) |
| 63-07 | `06-frontend` — acht `/person`-Aufrufe, D-63-06, 404/409/422 | erledigt (Task 3) |
| 63-08 | `06-frontend` — neue Route, Seiten-/Komponenten-Uebersicht, reine Helfer als Testmuster | erledigt (Task 3) |
| 63-09 | `06-frontend` — zwei Routen streichen, `/person/:person_id/` als einziges Ziel, Abwesenheit von Weiterleitungen | erledigt (Task 3) |
| 63-09 | `06-frontend` — geloeschte Seiten, fuenf entfallene Aktionen | erledigt (Task 3); die Aktions-Liste steht in F01 §6, nicht in `06-frontend` — dort gab es noch nie eine Aktions-Uebersicht, und eine anzulegen waere ein neues, sofort driftendes Inventar |

**Ausserhalb des Scopes gelassen, mit Grund:** die in 63-07/63-09 vermerkte
„Aktions-Uebersicht" in `06-frontend` (siehe letzte Zeile). Die neun bzw. jetzt
sechzehn `UserManagementAction`-Varianten sind in F01 §6 an der Seite
dokumentiert, die sie sendet — ein zweites Inventar im Architektur-Dokument
haette dieselbe Wahrheit doppelt gefuehrt, und die Erfahrung dieses Plans ist,
dass genau solche Doppelungen falsch werden.

## Konsistenz-Durchlauf (Task 3)

Jede Behauptung gegen die Quelle gegrept, nicht gegen die SUMMARYs:

| Geprueft | Ergebnis |
|---|---|
| sieben `PersonDetailService`-Methoden in `service/src/person_detail.rs` | alle gefunden |
| vier neue `PersonDao`-Methoden + `find_directory_entry` | je 1 Treffer |
| fuenf REST-Pfade in `rest/src/person_detail.rs`, `nest("/person"` in `lib.rs` | alle gefunden |
| acht `api.rs`-Funktionen | alle gefunden |
| sechzehn in F01 §6 genannte `UserManagementAction`-Varianten | alle gefunden |
| drei reine Helfer in `page/person_detail.rs` | alle gefunden |
| `classify_link_error` in `loader.rs`, `CreatePersonEntryDialog` | gefunden |
| fuenfzehn im neuen Routen-Inventar genannte Seiten-Dateien | alle vorhanden |
| `Dioxus.toml` Zeile 118 (`/person-directory`) / Zeile 130 (`/person`) | bestaetigt, Reihenfolge stimmt |
| `user_management.rs` Zeilenzahl (Doku sagte 878) | **845** — korrigiert |
| `SalesPersonDetails`, `UserAccount`, `ConnectUserAccount` ausserhalb von `i18n/` | 0 Treffer — als verwaist benannt |
| `person_table_has_no_deleted_column` | existiert (`shifty_bin/src/person_link_migration_test.rs:1052`) |
| `Route::UserDetails\|Route::SalesPersonDetails` in `docs/` | **0** |
| `page/user_details.rs` in `docs/` ausserhalb historischer Erwaehnung | **0** |
| EN/DE-Paarigkeit ueber alle geaenderten `docs/**/*.md` | 0 unpaarige Dateien |
| Mermaid-Struktur (`.mmd`) | 3 `subgraph` / 3 `end`, keine undefinierte Knoten-Referenz, `PersonDetail` definiert |

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo build --workspace` | Exit 0 |
| `cargo clippy --workspace -- -D warnings` | Exit 0 |
| `cargo test --workspace` | gruen, 0 Failures (958 `service_impl` + 104 Integration + uebrige Suites) |
| `grep -c 'get_detail\|create_combined\|add_facet\|remove_facet' F01*.md` | 18 / 18 (gefordert: ≥ 4) |
| `grep -c 'PersonDetail' 02-service-tiers{,_de}.md, .mmd` | 1 / 1 / 4 — kein File ohne Treffer |
| `grep -ci 'transaktion\|transaction' 04-auth_de.md` | 13 (gefordert: ≥ 1) |
| `grep -c 'create_user' 05-transactions{,_de}.md` | 1 / 1 — kein File ohne Treffer |
| `grep -c 'person/' 06-frontend{,_de}.md` | 2 / 2 — kein File ohne Treffer |

Dieser Plan aendert keinen Code; die drei gruenen Gates belegen, dass der Stand
unveraendert gruen ist.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] `docs/architecture/03-data-model.{md,_de.md}` behauptete, es werde keine Person geloescht**
- **Gefunden waehrend:** Task 3 (Konsistenz-Durchlauf)
- **Problem:** Die `person`-Sektion schreibt „No person is deleted in v2.7"
  und leitet daraus ab, eine `deleted`-Spalte sei unnoetig. Seit 63-05 loescht
  `remove_facet` die Zeile hart. Die Datei steht nicht in `files_modified`,
  aber der `key_links`-Eintrag des Plans verlangt ausdruecklich einen Satz zum
  Lebenszyklus der `person`-Zeile.
- **Fix:** Der Punkt trennt jetzt „nie *soft*-geloescht" (unveraendert, per
  Schema-Test festgezurrt) von „Hard Delete seit Phase 63, ohne
  Schema-Aenderung", benennt die Lesart von „lebender Traeger" und den
  akzeptierten haengenden Zeiger, und verlinkt auf F01 §3.
- **Dateien:** `docs/architecture/03-data-model.md`, `..._de.md`
- **Commit:** `716c077b`

**2. [Rule 1 - Bug] `docs/features/F10-templates-communication.{md,_de.md}` zeigte auf eine geloeschte Datei**
- **Gefunden waehrend:** Task 3 (Konsistenz-Durchlauf ueber den ganzen `docs/`-Baum)
- **Problem:** Vier Verweise auf `shifty-dioxus/src/page/user_details.rs`
  (inklusive zweier Zeilennummern-Angaben), geloescht in 63-09. F10 gehoert
  nicht zu dieser Phase — aber der Rueckbau hat sie falsch gemacht.
- **Fix:** Verweise auf `page/person_detail.rs` umgestellt, Zeilennummern durch
  Symbolnamen ersetzt (`DetailIntent::RevokeInvitation` /
  `RevokeInvitationSession`, der `InvitationStatus::Redeemed`-Zweig), und der
  Umzug als Satz festgehalten, damit die Historie nicht verloren geht.
- **Verifikation:** `grep -rn "page/user_details" docs/` → nur noch die eine
  historische Erwaehnung des Umzugs selbst.
- **Commit:** `74a97f6e`

**3. [Rule 2 - fehlende kritische Doku] `06-frontend` hatte kein Routen-/Seiten-Inventar**
- **Gefunden waehrend:** Task 3
- **Problem:** Die Auflage aus 63-08/63-09 lautet „die zwei abgeloesten Routen
  aus dem Inventar entfernen". Es gab kein Inventar — `grep 'Route::UserDetails'`
  gab schon vorher 0. Die Auflage waere formal erfuellt gewesen, ohne dass ein
  Leser je erfaehrt, dass es genau eine Detailroute gibt.
- **Fix:** Tabelle ueber alle vierzehn Routen mit ihrer Seiten-Datei angelegt,
  `/person/:person_id/` als einzige Detailroute markiert, die ersatzlose
  Entfernung und die bewusste Abwesenheit einer Weiterleitung erklaert.
- **Verifikation:** alle fuenfzehn genannten Seiten-Dateien existieren.
- **Commit:** `74a97f6e`

**4. [Rule 1 - Bug] Falsche Zeilenzahl und falsche i18n-Key-Zuordnung**
- **Gefunden waehrend:** Task 1 und Task 3
- **Problem:** F12 nannte `user_management.rs` mit 878 Zeilen (tatsaechlich
  845). Der erste Entwurf der F01-i18n-Sektion behauptete, `UserAccount`,
  `ConnectUserAccount` und `SalesPersonDetails` seien nach `person_detail.rs`
  gewandert; ein Grep zeigte 0 Verwendungen — sie gehoeren zu den vierzehn
  verwaisten Keys aus 63-09.
- **Fix:** Zahl korrigiert; die drei Keys ausdruecklich als „registriert, aber
  unbenutzt" gefuehrt statt faelschlich als umgezogen.
- **Commits:** `41384e9c`, `779efb25`

**5. [Rule 2] `[To verify]`-Marker zur Commit-Semantik aufgeloest**
- **Gefunden waehrend:** Task 2
- **Problem:** `05-transactions` fuehrt als „kritische Invariante, ohne die das
  Pattern kaputt ist", dass ein Commit auf durchgereichter TX kein echtes
  `COMMIT` fahren darf — als `[To verify]`. Genau diese Invariante traegt die
  SC-2-Zusage von 63-05.
- **Fix:** `TransactionDaoImpl::commit` gelesen (`Arc::into_inner`), Code-Auszug
  und Erklaerung eingesetzt, Marker entfernt — in beiden Sprachen.
- **Commit:** `779efb25`

---

**Total deviations:** 5 auto-fixed (3× Rule 1, 2× Rule 2). Vier davon sind
Korrekturen an Doku, die durch diese Phase falsch geworden ist; eine ist eine
Beleg-Nachreichung. Kein Scope Creep: keine Code-Datei angefasst, kein neues
Konzept eingefuehrt.

## Issues Encountered

**Der Plan rechnete mit dreizehn Dateien, es sind siebzehn geworden.** Zwei
Paare kamen dazu (`03-data-model`, `F10`), beide weil der Konsistenz-Durchlauf
sie als falsch entlarvt hat. Das ist kein Planungsfehler, sondern genau das,
wofuer Task 3 die Anweisung „findet sich eine Behauptung ohne Deckung, wird die
Doku korrigiert" mitbringt.

**`06-frontend` war duenner als angenommen.** Der Plan (und die Faelligkeiten
aus drei Vorgaenger-Plaenen) setzen eine „Routen-Uebersicht" und ein
„Seiten-Inventar" voraus, die es nicht gab. Siehe Deviation 3.

## Deferred Issues

Unveraendert aus 63-09 uebernommen, keine davon in diesem Plan adressierbar
(alle betreffen Code, nicht Doku) — aber jetzt alle in der Doku *benannt*, wo
sie hingehoeren:

- `service_impl/src/test/person_directory_dao.rs:149` — `needless_lifetimes`
  unter `cargo clippy --all-targets` (pre-existing, Phase 62; feuert nicht
  unter dem CLAUDE.md-Gate).
- `cargo fmt` fehlt in der Nix-Dev-Shell.
- `Key::CopyToClipboard` doppelt registriert in `i18n/de.rs` und `i18n/en.rs`
  (aus 63-03).
- Vierzehn verwaiste i18n-Keys — **jetzt in F01 §6 als solche benannt**, damit
  ein spaeterer Aufraeum-Pass weiss, dass sie bewusst stehen.
- `SelectedSalesPerson::user_id` und der tote Benutzer-Abgleich in
  `save_sales_person()` — bewusst nicht dokumentiert: eine Doku ueber toten
  Code haelt ihn am Leben.
- `session.user_id` ohne `ON DELETE CASCADE` (T-63-05-E, accept) — **jetzt an
  drei Stellen dokumentiert** (F01 §4.7, `04-auth`, F12 §4.6), weil ein
  generischer 500 ohne Erklaerung im UAT wie ein Defekt aussieht.
- Der `[[web.proxy]]`-Eintrag fuer `/person` bleibt ohne `cargo test`-Abdeckung
  — Laufzeit-Nachweis nur per `dx serve` (Plan 63-11). Die Reihenfolge-Falle
  (`/person` als Praefix von `/person-directory`) steht jetzt in F01 §6 **und**
  in `06-frontend`.

## Threat-Model-Abgleich

| Threat | Disposition | Umsetzung |
|---|---|---|
| T-63-10-A (Auth-Gate-Zeile falsch → Absenken des echten Gates) | mitigate | Alle sieben Gate-Zeilen der F01-Tabelle gegen `grep check_permission service_impl/src/person_detail.rs` geprueft: sieben Treffer, alle `"admin"`, jeweils erste Anweisung. Zusaetzlich steht in F01 §4.7 und in `02-service-tiers` ausdruecklich, dass das strenger ist als die HR-Gates darunter — eine spaetere „Vereinheitlichung" nach unten waere damit sichtbar ein Widerspruch. |
| T-63-10-B (undokumentierte Loesch-Semantik) | mitigate | Hard vs. Soft als Tabelle in F01 §4.7 (beide Sprachen), plus Kaskade, plus Person-Aufraeumung, plus der Satz „es gibt keinen Sammelknopf" mit Begruendung. Zusaetzlich als Randfall in §7 und als Lebenszyklus in `03-data-model`. |
| T-63-10-C (EN/DE-Drift) | mitigate | Paarigkeits-Pruefung ueber `git diff --name-only` programmatisch ausgefuehrt: 0 unpaarige Dateien. Ueberschriften-Struktur von F01 und `06-frontend` zeilenweise gegenuebergestellt. |
| T-63-10-SC (Supply Chain) | mitigate | Keine Abhaengigkeit, kein Package-Install, kein Code-Diff. |

Keine neue Angriffsflaeche: keine Endpunkte, keine Auth-Pfade, keine
Schema-Aenderung.

## Self-Check: PASSED

- `docs/features/F01-employee-management.md` / `_de.md` — FOUND
- `docs/architecture/02-service-tiers.md` / `_de.md` — FOUND
- `docs/architecture/03-data-model.md` / `_de.md` — FOUND
- `docs/architecture/04-auth.md` / `_de.md` — FOUND
- `docs/architecture/05-transactions.md` / `_de.md` — FOUND
- `docs/architecture/06-frontend.md` / `_de.md` — FOUND
- `docs/architecture/diagrams/service-graph-runtime.mmd` — FOUND
- `docs/features/F12-auth-session.md` / `_de.md` — FOUND
- `docs/features/F10-templates-communication.md` / `_de.md` — FOUND
- Commit `41384e9c` — FOUND
- Commit `779efb25` — FOUND
- Commit `74a97f6e` — FOUND
- Commit `716c077b` — FOUND

## Commits

| Task | Commit | Typ |
|---|---|---|
| 1 | `41384e9c` | `docs(63-10): F01 um Detail-Aggregat, Anlege-Pfad und Loesch-Semantik fortschreiben` |
| 2 | `779efb25` | `docs(63-10): Service-Tiers, Runtime-Graph, Auth und Transaktions-Konvention nachziehen` |
| 3 | `74a97f6e` | `docs(63-10): Frontend-Doku und Konsistenz-Durchlauf` |
| 3 (Nachzug) | `716c077b` | `docs(63-10): Lebenszyklus der person-Zeile im Datenmodell nachziehen` |
