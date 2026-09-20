---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 07
subsystem: frontend (api / loader / store)
tags: [person-detail, frontend-datenschicht, conflict-409, not-found-404, d-63-06, padm-02, padm-03, padm-04, padm-05, padm-07]
status: complete
requires:
  - "rest: die sieben /person-Routen aus Plan 63-06 (Praefix, Bodies, Statuscodes)"
  - "rest_types::{PersonEntryTO, CreatePersonEntryTO, CreatedPersonEntryTO, PersonFacetTO, AddPersonFacetTO} (63-04)"
  - "rest::sales_person: DELETE /sales-person/{id} und POST /sales-person/{id}/user — beide unveraendert seit Phase 61"
  - "shifty-dioxus/Dioxus.toml: [[web.proxy]] fuer /person (63-06)"
provides:
  - "api::{get_person_detail, get_person_by_sales_person, post_person_entry, post_person_facet, delete_person_facet, get_unlinked_sales_persons, get_unlinked_users, delete_sales_person}"
  - "loader::{load_person_detail, load_person_by_sales_person, create_person_entry, add_person_facet, remove_person_facet, load_unlinked_sales_persons, load_unlinked_users, delete_sales_person}"
  - "UserManagementStore::{person_detail, person_detail_missing, unlinked_sales_person_candidates, unlinked_user_candidates, person_link_conflict}"
  - "UserManagementAction::{LoadPersonDetail, CreatePersonEntry, AddPersonFacet, RemovePersonFacet, LoadUnlinkedSalesPersons, LoadUnlinkedUsers, LinkCandidate, UnlinkFacet, DeleteSalesPerson}"
  - "rest_types::SalesPersonTO/UserTO tragen PartialEq (UserTO zusaetzlich Clone/Eq)"
affects:
  - "63-08 (Detailseite: findet Store-Felder und Aktionen fertig vor; entfernt die neun #[allow(dead_code)] Variante fuer Variante)"
  - "63-09 (Dialoge: CreatePersonEntry signalisiert ueber den Uebergang None -> Some in person_detail)"
  - "63-10 (Doku: docs/architecture/06-frontend.md wird faellig — siehe Docs-Freshness-Gate unten)"
  - "63-11 (Verifikation: der Dev-Proxy-Eintrag bleibt nur per dx serve nachweisbar)"
tech_stack:
  added: []
  patterns:
    - "Statuscode-Uebersetzung in api.rs, wo die Antwort noch offen ist und der 422-Body gelesen werden kann — dieselbe Form wie die Absence- und Rebooking-Pfade"
    - "404 als Zustand statt als Fehler: Option im Loader-Rueckgabetyp, boolesches Store-Feld getrennt von 'noch nicht geladen'"
    - "Nachladen ueber die vom Backend zurueckgegebene person_id, nie ueber die uebergebene (D-61-03)"
    - "Ein-Schuss-Konflikt-Flag, das von jedem erfolgreichen Kandidaten-Laden zurueckgesetzt wird — die Konflikt-Behandlung laedt deshalb zuerst und setzt danach"
    - "Quell-Assertion als dauerhafte Absicherung einer Architektur-Auflage, per Mutation gegen Vakuitaet belegt"
key_files:
  created: []
  modified:
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/service/user_management.rs
    - rest-types/src/lib.rs
decisions:
  - "Die 409/422-Uebersetzung der beiden neuen Schreibpfade liegt in api.rs, nicht in loader.rs: dort ist die Response noch offen und der 422-Body lesbar. Der Plan verortete das Muster in loader.rs; die Vorlage, auf die er verweist (Absence-Pfade), steht tatsaechlich in api.rs."
  - "save_user_for_sales_person bekommt die Uebersetzung dagegen im Loader (classify_link_error), weil api::post_user_to_sales_person ein unveraenderter Phase-61-Pfad mit reqwest::Error-Signatur ist und mehrere Bestandsaufrufer hat."
  - "Die Conflict/Validation-Texte dieses Loader-Pfades sind bewusst NICHT leer: laeuft ein 409 einmal nicht in die seitenlokale Behandlung, sondern ins zentrale Overlay, stuende dort sonst eine leere Zeile."
  - "CreatePersonEntry kommt ohne sechstes Store-Feld aus: der Arm leert person_detail und fuellt es mit dem frisch geladenen Eintrag der zurueckgegebenen person_id. Der Dialog beobachtet den Uebergang None -> Some (D-63-07)."
  - "SalesPersonTO und UserTO bekommen PartialEq (UserTO zusaetzlich Clone/Eq), damit UserManagementStore sie in seinen Rc<[..]>-Feldern halten kann — der Store leitet Clone + PartialEq ab."
  - "Die neun Action-Varianten tragen bis Plan 63-08 je ein einzelnes #[allow(dead_code)] statt einer Markierung auf der ganzen Aufzaehlung, damit eine kuenftig wirklich tote Variante weiterhin auffaellt."
metrics:
  duration_minutes: 34
  tasks_completed: 3
  files_modified: 4
  tests_added: 2
  completed: 2026-09-20
---

# Phase 63 Plan 07: Datenschicht der Personen-Detailansicht — Summary

Acht HTTP-Funktionen, acht Loader-Wrapper, fuenf Store-Felder und neun
Coroutine-Aktionen. Kein Markup — die Komponente folgt in Plan 63-08 und findet
alle Netzwerk- und Zustandswege fertig vor.

## Was gebaut wurde

| Ebene | Symbol | Traegt |
|---|---|---|
| `api.rs` | `get_person_detail` | PADM-05 |
| `api.rs` | `get_person_by_sales_person` | PADM-04 / SC-4 |
| `api.rs` | `post_person_entry` | PADM-02 / SC-2 |
| `api.rs` | `post_person_facet` | PADM-03 / SC-3 |
| `api.rs` | `delete_person_facet` | PADM-07 / SC-5 |
| `api.rs` | `get_unlinked_sales_persons` / `get_unlinked_users` | PADM-04 |
| `api.rs` | `delete_sales_person` | PADM-07 / **D-63-06** |
| `loader.rs` | die acht gleichnamigen Wrapper | — |
| `user_management.rs` | fuenf Store-Felder, neun Aktionen, zehn Hilfsfunktionen | — |

Dazu der Vier-Stellen-Vertrag des Praefixes `/person` als Blockkommentar ueber
den neuen API-Funktionen — dieselbe Form, die `get_person_directory` schon
traegt.

## Zwei Dinge, die es bis heute nicht gab

**Der Verkaeufer-Loeschpfad.** `DELETE /sales-person/{id}` existiert im Backend
seit Langem und ist HR-gegatet. Das Frontend hatte keinen Aufrufer — nur den
Benutzer-Loeschpfad. Ohne diesen Aufruf bliebe PADM-07 fuer die Verkaeufer-Seite
eine leere Zusage. Die Semantik ist ein Soft Delete; das steht als Doc-Kommentar
an `loader::delete_sales_person`, damit die Bestaetigung in 63-08 nicht das
Falsche verspricht.

**Die beiden Kandidatenlisten.** `unlinked_sales_persons` / `unlinked_users`
existierten seit Phase 61 ohne Verdrahtung und seit 63-06 mit Route. Hier
bekommen sie ihren ersten Konsumenten.

## Der `person_id`-Wechsel wird durchgereicht

`POST /person/{id}/facet` antwortet mit 200 und `CreatedPersonEntryTO`, und die
darin enthaltene `person_id` ist die **resultierende**, nicht die uebergebene:
schaltet man einem reinen Benutzer-Eintrag eine Sales Person zu, gewinnt nach
D-61-03 die Sales Person, und der Eintrag wechselt seine Identitaet.

Beide Stellen, an denen das auftritt, sind entsprechend gebaut:

- `add_person_facet` laedt ueber `created.person_id` nach, nicht ueber das
  Argument.
- `link_candidate` laedt nach erfolgreichem Verknuepfen ueber
  `load_person_by_sales_person(sales_person_id)` — der Weg, der die Identitaet
  gar nicht erst voraussetzt. Genau dafuer existiert dieser Lesepfad.

## Drei unterscheidbare Antworten statt einem generischen Fehler

| Statuscode | Ankunft im Frontend | Wirkung |
|---|---|---|
| 404 auf dem Detail-Lesepfad | `Ok(None)` | `person_detail_missing = true` — die Seite zeigt „existiert nicht mehr" statt eines Ladespinners (UI-SPEC, not-found state) |
| 409 beim Verknuepfen | `ShiftyError::Conflict` | Kandidatenliste neu laden, danach `person_link_conflict = true` (SC-4 / D-61-04) |
| 422 beim Anlegen | `ShiftyError::Validation(body)` | Rohtext fuer die Dialog-Meldung (D-63-07 / ASVS V5) |

Die Reihenfolge im Konflikt-Arm ist nicht beliebig: jedes **erfolgreiche**
Laden einer Kandidatenliste setzt `person_link_conflict` zurueck. Wuerde das
Flag vor dem Nachladen gesetzt, loeschte der Ladevorgang es sofort wieder. Der
Arm laedt deshalb zuerst und setzt danach; beides steht als Begruendung am
Store-Feld und an `link_candidate`.

## Neue Tests (2)

`cargo test --manifest-path shifty-dioxus/Cargo.toml` faehrt jetzt **834** statt
832 Tests.

| Test | Traegt |
|---|---|
| `load_person_detail_ist_ein_request` | Variante (b) des Detail-Aggregats: genau ein `.await` im Lade-Arm. Ein zweiter waere der Anfang eines fetten Client-Aggregats. |
| `kein_arm_loescht_beide_seiten_auf_einmal` | D-63-05: kein Arm der Koroutine fuehrt Verkaeufer- und Benutzer-Loeschung in einem Zug aus. Die Pruefung laeuft ueber den Koroutinen-Rumpf, nicht ueber die ganze Datei — die Hilfsfunktionen oben tragen beide Aufrufe naturgemaess, aber getrennt. |

### Mutations-Nachweis gegen Vakuitaet

Beides sind Quell-Assertions und damit die Testform, die am leichtesten
vakuiert (ein umbenannter Marker, und der Test prueft nichts mehr). Deshalb
gemessen statt behauptet:

| Mutation | Erwartung | Ergebnis |
|---|---|---|
| Zweites `load_person_directory().await` in den `LoadPersonDetail`-Arm gesetzt | `load_person_detail_ist_ein_request` faellt | FAILED — `left == right` auf der `.await`-Zahl |
| `delete_user("x".into()).await` in den `DeleteSalesPerson`-Arm gesetzt | `kein_arm_loescht_beide_seiten_auf_einmal` faellt | FAILED — Sammelknopf erkannt |

Beide Mutationen wurden vollstaendig zurueckgenommen; die drei Tests des Moduls
sind danach wieder gruen.

## Deviations from Plan

### Auto-behoben

**1. [Rule 3 — blockierend] `PartialEq` fuer `SalesPersonTO` und `UserTO`**

- **Gefunden bei:** Task 3.
- **Problem:** `UserManagementStore` leitet `Clone` **und** `PartialEq` ab. Die
  beiden Kandidaten-Felder (`Rc<[SalesPersonTO]>`, `Rc<[UserTO]>`) sind damit
  nur haltbar, wenn die TOs `PartialEq` tragen — `SalesPersonTO` trug nur
  `Clone`, `UserTO` gar keines von beiden. Der Store liess sich nicht
  uebersetzen.
- **Fix:** `PartialEq` an `SalesPersonTO`, `Clone + PartialEq + Eq` an `UserTO`.
  Rein additiv; `PersonEntryTO` traegt `PartialEq` aus demselben Grund seit
  Phase 62. Die Alternative — die Kandidaten als Frontend-Typen (`SalesPerson`,
  `User`) zu halten — haette die im Plan festgeschriebenen Signaturen von
  `api::get_unlinked_*` und `loader::load_unlinked_*` gebrochen.
- **Datei:** `rest-types/src/lib.rs`
- **Commit:** `d3dfa529`

**2. [Rule 3 — blockierend] `#[allow(dead_code)]` auf den neun Action-Varianten**

- **Gefunden bei:** Task 3.
- **Problem:** Die Varianten bekommen ihren Absender erst mit der Detailseite in
  Plan 63-08. Bis dahin meldet `dead_code` „multiple variants are never
  constructed", und das Plan-Gate `cargo clippy -- -D warnings` faellt.
- **Fix:** Je Variante eine eigene Markierung mit Begruendung, ausdruecklich
  **nicht** eine auf der ganzen Aufzaehlung — sonst bliebe eine kuenftig wirklich
  tote Variante unbemerkt. 63-08 entfernt sie Variante fuer Variante.
- **Datei:** `shifty-dioxus/src/service/user_management.rs`
- **Commit:** `d3dfa529`

### Abweichung von der Plan-Prosa (nicht vom Verhalten)

**Ort der 409/422-Uebersetzung.** Der Plan verlangt sie fuer `add_person_facet`
und `create_person_entry` in `loader.rs` und verweist als Vorlage auf „das
vorhandene Muster … in dieser Datei bei den Absence-Pfaden". Dieses Muster steht
tatsaechlich in **`api.rs`** (`create_absence_period`, `update_absence_period`,
und ebenso alle Rebooking-Pfade) — `loader.rs` enthielt vor diesem Plan keine
einzige Statuscode-Auswertung, nur einen Doc-Kommentar, der eine erwaehnt.

Die Uebersetzung liegt deshalb dort, wo sie funktioniert: in `api.rs`, solange
die `Response` noch offen ist und der 422-Body per `response.text()` gelesen
werden kann. Eine Verlagerung in den Loader haette den Body verworfen, weil
`error_for_status_ref()` ihn nicht mitfuehrt — die Validierungsmeldung des
Anlege-Dialogs waere leer geblieben.

Im Loader steht die Uebersetzung dort, wo sie dort hingehoert:
`classify_link_error` uebersetzt den 409 des **unveraenderten** Phase-61-Pfades
`POST /sales-person/{id}/user`, dessen `api::`-Funktion `reqwest::Error`
zurueckgibt und mehrere Bestandsaufrufer hat. Das Kriterium
`grep -c 'ShiftyError::Conflict' shifty-dioxus/src/loader.rs` ist damit erfuellt
(1 vorher → 3 nachher).

**Nebenwirkung, bewusst in Kauf genommen:** `save_sales_person` (Bestandspfad
der Verkaeufer-Detailseite) sieht denselben 409 jetzt als `Conflict` statt als
`Reqwest`. Weil die Variante als `#[error("{0}")]` gerendert wird, haette ein
leerer String dort eine leere Overlay-Zeile erzeugt. Die Texte sind deshalb
nicht leer, sondern nennen Statuscode und Ursache.

### Reihenfolge der Gates

Wie in Plan 63-06: die drei Tasks sind **drei Commits**, aber ein gruenes
`clippy -- -D warnings` ist zwischen ihnen konstruktionsbedingt nicht
herstellbar. Task 1 legt acht Funktionen ohne Aufrufer an (→ `dead_code`),
Task 2 kapselt sie und erzeugt dieselbe Lage eine Ebene hoeher. Erst nach Task 3
ist die Kette geschlossen. Alle drei Tasks wurden deshalb implementiert, das
volle Gate auf dem kombinierten Stand gefahren und **danach** in drei Commits
aufgeteilt. Die Dateizuordnung je Task ist unveraendert die des Plans
(`rest-types/src/lib.rs` haengt an Task 3, weil erst der Store die Derives
braucht).

## Auth Gates

Keine.

## Threat Flags

Keine neue Angriffsflaeche ausserhalb des Registers. Abgleich:

| Threat | Disposition | Umsetzung |
|---|---|---|
| T-63-07-A (EoP auf allen neuen Aufrufen) | mitigate | Kein Gate im Client — `grep -c 'is_admin\|check_permission'` ueber die drei geaenderten Frontend-Dateien = 0. Jede Route ist serverseitig auf `admin` gegatet (63-04/63-05), auf HTTP-Ebene in 63-06 ueber alle sieben Operationen belegt. `DELETE /sales-person/{id}` ist HR-gegatet. Der Component-Guard aus 63-08 bleibt Defense-in-Depth. |
| T-63-07-B (Konflikt wird geschluckt) | mitigate | `classify_link_error` hebt den 409 aus dem Transportfehler heraus; `link_candidate` behandelt ihn als eigenen Zweig, laedt die Kandidatenliste neu und setzt das Flag. Kein `_ => Err(err)` faengt ihn vorher ab. |
| T-63-07-C (Kandidatenlisten im Store) | accept | Unveraendert: die Listen tragen Verkaeufer- und Benutzernamen, also genau das, was die admin-gegatete Seite ohnehin zeigt. |
| T-63-07-D (Verkaeufer-Loeschpfad erstmals im Client) | mitigate | Der Pfad ist aus dem Store nur ueber `UserManagementAction::DeleteSalesPerson` erreichbar; die Bestaetigung nach D-63-08 baut Plan 63-08. Die Soft-Delete-Semantik ist am Loader dokumentiert, damit die Bestaetigung das Richtige benennt. |
| T-63-07-SC (Supply Chain) | mitigate | Keine neue Abhaengigkeit, kein Package-Install. `git diff --stat` beruehrt keine `Cargo.toml`. |

## Known Stubs

Keine. Die neun Action-Varianten haben bis Plan 63-08 keinen Absender — das ist
die geplante Wellen-Aufteilung und kein Stub im Sinne einer leeren Anzeige. Sie
tragen keine Platzhalter-Werte, sondern schlicht noch keinen Aufrufer.

## Docs-Freshness-Gate

Beruehrte Trigger-Dateien laut CLAUDE.md-Tabelle: **keine.**
`shifty_bin/src/main.rs`, `service/**`, `dao/**`, `migrations/sqlite/*.sql`,
`service_impl/src/permission.rs`, `service_impl/src/reporting.rs` und
`service_impl/src/billing_period_report.rs` wurden nicht angefasst.

Zusaetzlich faellig gemaess der Auflage des Ausfuehrungs-Auftrags:

| Beruehrt | → faellig |
|---|---|
| `shifty-dioxus/src/{api,loader,service/user_management}.rs` (neue Frontend-Datenschicht der Detailansicht) | `docs/architecture/06-frontend.md` **und** `docs/architecture/06-frontend_de.md` — die acht `/person`-Aufrufe in die Endpoint-/Loader-Uebersicht aufnehmen; den Verkaeufer-Loeschpfad als neuen Frontend-Aufrufer benennen (D-63-06); die drei unterscheidbaren Antworten (404 als Zustand, 409 als Konflikt, 422 als Validierung) in der Fehlerbehandlungs-Sektion ergaenzen. |
| `rest-types/src/lib.rs` (`PartialEq` auf `SalesPersonTO`/`UserTO`) | Keine Doku-Sektion — reine Ableitungs-Ergaenzung ohne Konventions- oder Schema-Wirkung. |

**Nicht in diesem Plan erledigt** — Plan **63-10** fuehrt die Doku-Aktualisierung
der Phase zentral durch (Auflage aus dem Ausfuehrungs-Auftrag).

## Deferred Issues

Unveraendert gegenueber 63-06:

- `service_impl/src/test/person_directory_dao.rs:149` loest unter
  `cargo clippy --all-targets` eine `needless_lifetimes`-Warnung aus
  (pre-existing, Phase 62, ausserhalb des Scopes; feuert nicht unter dem
  CLAUDE.md-Gate und nicht in CI).
- `cargo fmt` fehlt in der Nix-Dev-Shell; der neue Code wurde von Hand
  rustfmt-konform geschrieben.
- `Key::CopyToClipboard` ist in `i18n/de.rs` und `i18n/en.rs` je zweimal
  registriert (zweites `add_text` ueberschreibt still das erste). Aus 63-03
  bekannt, ausserhalb des Scopes dieses Plans.
- Der `[[web.proxy]]`-Eintrag fuer `/person` bleibt der einzige Teil der
  Frontend-Kette, den kein `cargo test` abdeckt — Laufzeit-Nachweis nur per
  `dx serve` (Plan 63-11).
- **Umgebung, kein Code-Defekt:** auf Port 3000 laeuft weiterhin ein fremder,
  aelterer `shifty_bin` (PID 460885) aus einer frueheren Sitzung. Er wurde
  bewusst nicht beendet. Wer in 63-11 einen 404 auf `/person` sieht, pruefe
  zuerst `ss -ltnp | grep 3000`.

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo build --manifest-path shifty-dioxus/Cargo.toml` | Exit 0 |
| `cargo test --manifest-path shifty-dioxus/Cargo.toml` | **834 passed**, 0 failed |
| `cargo test --manifest-path shifty-dioxus/Cargo.toml service::user_management` | 3 passed |
| `cargo clippy --manifest-path shifty-dioxus/Cargo.toml -- -D warnings` | Exit 0 |
| `cargo build --manifest-path shifty-dioxus/Cargo.toml --target wasm32-unknown-unknown` | Exit 0 |
| `cargo build --workspace` | Exit 0 |
| `cargo clippy --workspace -- -D warnings` | Exit 0 |
| `cargo test --workspace` | gruen (958 `service_impl` + 104 Integration + uebrige Suites) |
| `grep -c 'pub async fn get_person_detail\|…\|pub async fn delete_sales_person' api.rs` | **8** (gefordert: 8) |
| `grep -c 'sales-person/{}' api.rs` | 7 (vorher 5 → +2) |
| `grep -c 'pub async fn load_person_detail\|…\|pub async fn delete_sales_person' loader.rs` | **8** (gefordert: 8) |
| `sed -n '/pub async fn load_person_detail/,/^}/p' loader.rs \| grep -c 'Option<PersonEntryTO>'` | 1 (gefordert: ≥ 1) |
| `grep -c 'ShiftyError::Conflict' loader.rs` | 3 (vorher 1 → +2) |
| `grep -c 'LoadPersonDetail\|…\|DeleteSalesPerson' user_management.rs` | **25** (gefordert: ≥ 18) |
| `grep -c 'person_detail_missing' user_management.rs` | 6 (gefordert: ≥ 2) |
| `grep -ci 'DeletePersonEntry\|delete_person_entry' user_management.rs` | **0** |
| Prohibition „keine Auspraegungs-Ableitung": `grep -c 'PersonKindTO::…' api\|loader\|user_management` | **0** Dateien mit Treffern |
| Prohibition „kein Sammelknopf": `grep -ci 'DeletePersonEntry\|delete_person_entry' ueber alle drei` | **0** Dateien mit Treffern |

## Commits

| Task | Commit | Typ |
|---|---|---|
| 1 | `987018a6` | `feat(63-07): acht HTTP-Funktionen fuer die Personen-Detailansicht` |
| 2 | `5e8368c4` | `feat(63-07): Loader-Wrapper mit Konflikt- und Nicht-gefunden-Uebersetzung` |
| 3 | `d3dfa529` | `feat(63-07): Store-Felder, neun Aktionen und ihre Coroutine-Arme` |

## Self-Check: PASSED

- `shifty-dioxus/src/api.rs` — FOUND
- `shifty-dioxus/src/loader.rs` — FOUND
- `shifty-dioxus/src/service/user_management.rs` — FOUND
- `rest-types/src/lib.rs` — FOUND
- `63-07-SUMMARY.md` — FOUND
- Commit `987018a6` — FOUND
- Commit `5e8368c4` — FOUND
- Commit `d3dfa529` — FOUND
