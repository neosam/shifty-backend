---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
verified: 2026-09-29T00:00:00Z
status: passed
score: 11/11 must-haves verified
behavior_unverified: 0
overrides_applied: 0
---

# Phase 63: Personen-Detailansicht + Ausprägungs-Aktionen — Verification Report

**Phase Goal:** Ein Admin verwaltet eine Person vollständig an einem Ort — anlegen,
fehlende Ausprägung zuschalten, mit einem bestehenden Objekt verknüpfen, entfernen —
ohne zwischen Seiten zu springen und ohne raten zu müssen, was ein Löschen mitnimmt.

**Verifiziert:** 2026-09-29
**Status:** passed
**Re-Verifikation:** Nein — erste Verifikation

## Ausgangslage der Verifikation

Diese Phase hatte einen dokumentierten Defekt, der jedes automatisierte Gate passierte
und erst im menschlichen UAT auffiel: Die SPA-Route lag zunächst auf
`/person/:person_id/` und kollidierte mit dem Backend-Präfix `.nest("/person", ...)`.
Der Dev-Proxy leitet das ganze Segment weiter, wodurch die Detailseite über ihre eigene
URL einen 404 vom Backend statt der SPA bekam. Der Proxy-Beweis aus Plan 63-06/63-11 lief
gegen `/person/<uuid>` **ohne** abschließenden Schrägstrich — das ist der API-Pfad, der
funktioniert — und verbuchte den 200er fälschlich als Erfolg. Die 26 SSR-Rendertests aus
63-08 rendern die Komponente direkt und rühren das Routing nie an.

Der Fix (Commit `69737ac6`) verlegt die Route auf `/person-detail/:person_id/` und fügt
zwei Wächtertests in `router.rs` hinzu (Commit `f289cb0b`), die `[[web.proxy]]`-Einträge
aus `Dioxus.toml` gegen `Route::SITE_MAP` abgleichen. Diese Verifikation hat den Fix,
die Wächtertests und ihre Aussagekraft direkt am Code nachvollzogen — nicht nur die
SUMMARY-Erzählung übernommen.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|---|---|---|
| 1 | SC-1: Eine Komponente zeigt für alle drei Ausprägungen genau die passenden Abschnitte, keine leeren Platzhalter | ✓ VERIFIED | `shifty-dioxus/src/page/person_detail.rs`: `visible_sections(PersonKindTO)` liest ausschließlich `kind`, 26 SSR-Tests (`cargo test … person_detail` → 26 passed) prüfen positiv UND negativ je Ausprägung; UAT-Punkt C „freigegeben" |
| 2 | SC-2: Kombiniertes Anlegen erzeugt genau die gewählten Objekte, atomar über eine Transaktion | ✓ VERIFIED | `service_impl/src/person_detail.rs::create_combined` gatet zuerst auf `admin`, öffnet eine Transaktion, reicht `Some(tx.clone())`/`tx.clone()` an beide Aggregate; Test `create_combined_teilfehler_laesst_keine_haelfte_zurueck` per Mutation als nicht-vakuum belegt (`cargo test -p service_impl person_detail::create_combined` → 8 passed) |
| 3 | SC-3: Fehlende Ausprägung wird über „Login/Person hinzufügen" nachträglich zugeschaltet, Eintrag wechselt sichtbar | ✓ VERIFIED | `add_facet` gibt die **resultierende** `person_id` zurück (Signatur `Result<Uuid, ServiceError>`, Zeile `service/src/person_detail.rs:209`); `cargo test -p service_impl person_detail::add_facet` → 7 passed; UAT-Punkt D „freigegeben" |
| 4 | SC-4: Verknüpfen ausschließlich über Kandidatenliste (kein Freitext), bereits verknüpfte Ziele 409 | ✓ VERIFIED | `sed -n '/fn CandidatePickerDialog/,/^}/p' … \| grep -c TextInput` = 0 (nachvollzogen); Backend-Test `bereits_verknuepftes_ziel_wird_mit_409_abgelehnt` grün in `cargo test -p shifty_bin person_detail` (9 passed); UAT-Punkt E „freigegeben" |
| 5 | SC-5: Lösch-Bestätigungen benennen konkret, was verschwindet/bleibt, unterscheiden Verkäufer- von Benutzer-Seite | ✓ VERIFIED | `delete_confirmation_copy` liefert je Seite Key-Werte; D-63-08-Sätze wörtlich in `i18n/de.rs` (grep bestätigt „Login, Rollen und offene Einladungen…" und „Schichten, Stundenkonto und Abrechnungs-Historie…"); UAT-Punkt F „freigegeben" |
| 6 | `get_detail` liefert NUR die Verzeichniszeile (Variante b: Kern-Aggregat), kein Voll-Aggregat | ✓ VERIFIED | `service_impl/src/person_detail.rs::build_entry` ruft `find_directory_entry` + `all_user_roles` (genau 2 DAO-Aufrufe) und `entry_from_row`; kein Aufruf der vier Abschnitts-Endpoints im Service; Test `get_detail_fragt_jede_dao_methode_genau_einmal` (`.times(1)`) |
| 7 | `add_facet`/`add_person_facet` liefert die RESULTIERENDE `person_id` zurück; REST antwortet 200 + Body, nicht 204 (D-61-03) | ✓ VERIFIED | `rest/src/person_detail.rs:293-318`: `add_person_facet` baut `CreatedPersonEntryTO` und antwortet `Response::builder().status(200)...`; `remove_person_facet` antwortet separat mit 204 (Zeile 350) |
| 8 | Die beiden alten Detailseiten und -routen sind ersatzlos entfernt (D-63-03/D-63-04) | ✓ VERIFIED | `page/user_details.rs`, `page/sales_person_details.rs` existieren nicht (`ls` bestätigt); `grep -rn 'Route::UserDetails\|Route::SalesPersonDetails' shifty-dioxus/src` → 0 Treffer |
| 9 | Zeilen-Löschung in der Personenliste ist entfallen, Löschen lebt nur auf der Detailseite | ✓ VERIFIED | `page/user_management.rs::directory_row_has_no_delete_path` prüft je Ausprägung Abwesenheit von 🗑 und Danger-Tokens; Test grün; UAT-Punkt I „ausdrücklich zugestimmt" |
| 10 | Alle vier Auspräguns-Mutationen (`create_combined`, `add_facet`, `remove_facet`, `get_detail`) gaten im SERVICE auf `admin` als erste Anweisung, nicht im Handler | ✓ VERIFIED | Direkt gelesen in `service_impl/src/person_detail.rs`: alle vier Methoden rufen `self.permission_service.check_permission("admin", context).await?` vor jeder Transaktions-/DB-Operation; `rest/src/person_detail.rs`: `grep -c check_permission` = 0 (kein Handler-Gate) |
| 11 | Der Namensraum-Konflikt zwischen SPA-Route und Backend-Präfix ist behoben und durch einen Regressionstest gesichert | ✓ VERIFIED | `router.rs`: Route liegt auf `/person-detail/:person_id/`; zwei Tests `keine_spa_route_liegt_im_proxy_namensraum_des_backends` und `person_detail_route_liegt_ausserhalb_des_person_praefix` — beide selbst ausgeführt und grün (`cargo test --manifest-path shifty-dioxus/Cargo.toml router::` → 2 passed) |

**Score:** 11/11 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
|---|---|---|---|
| `dao/src/permission.rs`, `dao_impl_sqlite/src/lib.rs` | `PermissionDao::create_user`/`delete_user` auf geteilter Transaktion | ✓ VERIFIED | `type Transaction` vorhanden, `pool.begin()` aus `create_user` entfernt (63-01) |
| `dao/src/person.rs`, `dao_impl_sqlite/src/person.rs` | `delete`, `has_live_facets`, `find_by_sales_person`, `find_by_user` | ✓ VERIFIED | Alle vier Methoden vorhanden und getestet (63-02) |
| `service/src/person_detail.rs`, `service_impl/src/person_detail.rs` | `PersonDetailService`-Trait mit sieben Methoden | ✓ VERIFIED | 7 `async fn` bestätigt, admin-gegatet |
| `rest/src/person_detail.rs`, `rest/src/lib.rs` | sieben `/person`-Handler, `.nest("/person", ...)` | ✓ VERIFIED | Alle sieben Routen vorhanden, `error_handler` durchgängig, kein `check_permission` im Handler |
| `shifty-dioxus/src/api.rs`, `loader.rs`, `service/user_management.rs` | Frontend-Datenschicht (acht Funktionen) | ✓ VERIFIED | Bestätigt in 63-07-SUMMARY, Gates grün nachvollzogen |
| `shifty-dioxus/src/page/person_detail.rs` | Eine Seite, drei reine Helfer, fünf Dialoge | ✓ VERIFIED | `grep -c 'pub fn PersonDetail'` = 1; 26 Tests grün |
| `shifty-dioxus/src/router.rs` | Route `/person-detail/:person_id/` + zwei Wächtertests | ✓ VERIFIED | Route und Tests im Code vorhanden, selbst ausgeführt, grün |
| `docs/features/F01-employee-management.{md,_de.md}` u. a. (63-10) | Docs-Freshness-Gate geschlossen, inkl. Route-Umbenennung (94c13c5d) | ✓ VERIFIED | `06-frontend{,_de}.md` nennt `/person-detail/:person_id/` und begründet die Trennung vom Backend-Präfix |

### Key Link Verification

| From | To | Via | Status | Details |
|---|---|---|---|---|
| `shifty-dioxus/Dioxus.toml` `[[web.proxy]]` | `rest/src/lib.rs` `.nest("/person", ...)` | Vier-Stellen-Vertrag (Praefix) | ✓ WIRED | Proxy-Fingerprint-Nachweis in 63-11-SUMMARY (JSON vs. 8298-Byte-SPA-Fallback), durch Wächtertests dauerhaft abgesichert |
| `page/person_detail.rs` | `service_impl/person_detail.rs` via REST | acht `api.rs`-Funktionen | ✓ WIRED | Vollständige Kette Store → Loader → API → REST → Service, per Test und manuellem UAT bestätigt |
| `router.rs::Route::PersonDetail` | `Dioxus.toml` Proxy-Einträge | Segment-Kollisions-Wächter | ✓ WIRED | Beide Tests lesen `Dioxus.toml` per `include_str!` und `Route::SITE_MAP` zur Compile-/Testzeit — kein manueller Abgleich nötig |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|---|---|---|---|
| Routing-Regressionswächter laufen wirklich und fangen den historischen Defekt | `cargo test --manifest-path shifty-dioxus/Cargo.toml router::` | `2 passed; 0 failed` | ✓ PASS |
| Backend-Testsuite (Workspace) vollständig grün, exakte Zahl aus SUMMARY reproduziert | `cargo test --workspace` | 11+9+33+2+3+3+17+17+958+11+104 = **1168 passed, 0 failed, 3 ignored** | ✓ PASS |
| Frontend-Testsuite grün, exakte Zahl aus SUMMARY reproduziert | `cargo test --manifest-path shifty-dioxus/Cargo.toml` | **858 passed, 0 failed** | ✓ PASS |
| Clippy beide Seiten sauber | `cargo clippy --workspace -- -D warnings` / `cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings` | Exit 0 / Exit 0 | ✓ PASS |
| WASM-Build funktioniert | `cargo build --manifest-path shifty-dioxus/Cargo.toml --target wasm32-unknown-unknown` | Exit 0 | ✓ PASS |
| `person_detail`-Testfamilie Backend | `cargo test -p service_impl person_detail` / `cargo test -p shifty_bin person_detail` | 51 passed / 9 passed | ✓ PASS |

Alle sechs Gates aus dem SUMMARY selbst reproduziert, nicht aus der Prosa übernommen.

### Requirements Coverage

| Requirement | Source Plan(s) | Description | Status | Evidence |
|---|---|---|---|---|
| PADM-02 | 63-01, 63-03, 63-05, 63-06, 63-07, 63-08, 63-09, 63-11 | Kombiniertes Anlegen mit zuschaltbaren Ausprägungen | ✓ SATISFIED | `create_combined`, `create_person_entry`, UAT A/B |
| PADM-03 | 63-03, 63-05, 63-06, 63-07, 63-08, 63-11 | Fehlende Ausprägung nachträglich zuschalten | ✓ SATISFIED | `add_facet`, `MissingFacetCard`, UAT D |
| PADM-04 | 63-03, 63-04, 63-06, 63-07, 63-08, 63-11 | Verknüpfen über Kandidatenliste statt Freitext, 409 bei Konflikt | ✓ SATISFIED | `CandidatePickerDialog`, `/person/candidates/*`, UAT E |
| PADM-05 | 63-02, 63-03, 63-04, 63-06, 63-07, 63-08, 63-09, 63-10, 63-11 | Eine Detailansicht mit ausprägungsabhängigen Abschnitten | ✓ SATISFIED | `PersonDetail`, `visible_sections`, UAT C |
| PADM-07 | 63-01, 63-02, 63-03, 63-05, 63-06, 63-07, 63-08, 63-10, 63-11 | Lösch-Bestätigung benennt konkret Verschwindendes/Bleibendes | ✓ SATISFIED | `delete_confirmation_copy`, `remove_facet`, UAT F |

Alle fünf der Phase zugewiesenen Requirement-IDs (PADM-02, PADM-03, PADM-04, PADM-05,
PADM-07) sind in mindestens einem Plan deklariert und mit Code-Evidenz gedeckt. Kein
Requirement aus `.planning/REQUIREMENTS.md` §Phase 63 ist ohne deklarierenden Plan
(keine ORPHANED-Requirements). `REQUIREMENTS.md` selbst führt alle fünf als „Complete".

### Anti-Patterns Found

Keine. Geprüft wurden alle phasen-relevanten Kerndateien
(`service/src/person_detail.rs`, `service_impl/src/person_detail.rs`,
`rest/src/person_detail.rs`, `shifty-dioxus/src/page/person_detail.rs`,
`shifty-dioxus/src/router.rs`, `dao/src/person*.rs`, `dao_impl_sqlite/src/person*.rs`)
auf `TBD`/`FIXME`/`XXX`/`TODO`/`HACK`/`PLACEHOLDER` sowie auf leere/stub-artige
Rückgaben — keine Treffer, kein Debt-Marker-Gate ausgelöst.

### Bekannte offene Punkte (bewusst kein Gap)

Diese sind laut Aufgabenstellung deliberately open und werden hier nur auf ehrliche
Dokumentation geprüft, nicht als Mangel gewertet:

| # | Punkt | Wo dokumentiert | Befund |
|---|---|---|---|
| DEF-1 | 14 verwaiste i18n-Keys nach dem Rückbau | `63-09-SUMMARY.md`, `F01 §6` | ehrlich benannt, nicht verschwiegen |
| DEF-2 | `SelectedSalesPerson::user_id` nach Rückbau unerreichbar | `63-09-SUMMARY.md` | ehrlich benannt, bewusst nicht dokumentiert um toten Code nicht "am Leben zu halten" |
| DEF-3 | Doppelte Registrierung `Key::CopyToClipboard` | `deferred-items.md`, `63-03/63-07/63-09-SUMMARY.md` | ehrlich benannt |
| DEF-4 | Vorbestehendes `needless_lifetimes` in `person_directory_dao.rs:149` | `deferred-items.md`, mehrere SUMMARYs | ehrlich benannt, feuert nicht unter dem CLAUDE.md-Gate |
| DEF-5 | `session.user_id` ohne `ON DELETE CASCADE` → genereischer 500 | `04-auth.md`, `F12-auth-session.md`, `F01 §4.7`, `63-11-SUMMARY.md` | an drei Doku-Stellen UND im SUMMARY dokumentiert; UAT-Punkt H „als Kante akzeptiert, Beobachtung nicht gesondert belegt" — ehrlich als nicht-beobachtet markiert, nicht als bestätigt behauptet |
| DEF-6 | `/shiftplan/` (SPA) vs. `.nest("/shiftplan", ...)` — vorbestehende, dieselbe Fehlerklasse wie der Phase-63-Defekt, aber NICHT aus dieser Phase | `router.rs` `KNOWN_PREEXISTING`, `63-11-SUMMARY.md`, Commit `a7767219` | ehrlich als „gemessen kaputt" korrigiert (nicht nur „harmlos" behauptet), bewusst nicht in dieser Phase behoben |

Alle sechs Punkte sind an mehreren, auffindbaren Stellen dokumentiert — keiner wurde
still fallengelassen.

### Sonstiger Befund (informativ, kein Gap)

Die Fortschritts-Tabelle in `.planning/ROADMAP.md` (Zeile „63. Personen-Detailansicht +
Ausprägungs-Aktionen | 10/11 | In Progress | —") und die Phasen-Checkbox
(„- [ ] **Phase 63: …**") sind trotz vollständig abgeschlossenem und im UAT bestätigtem
Plan 63-11 nicht aktualisiert. Das ist ein bekanntes Muster in diesem Projekt (Memory:
„GSD-Roadmap-Writer korrumpiert Tabelle" — `update-plan-progress`/`phase.complete`
schreiben gelegentlich in die falsche ROADMAP-Tabelle). Es handelt sich um einen
Dokumentations-Tracking-Fehler, nicht um eine unerfüllte Phasen-Wahrheit — alle elf
Pläne sind in der Plan-Liste mit `[x]` markiert, alle Commits sind vorhanden, alle Gates
sind grün, der UAT ist protokolliert abgeschlossen. Empfehlung: ROADMAP.md-Tabelle und
-Checkbox vor dem nächsten Milestone-Schritt korrigieren.

### Gaps Summary

Keine Gaps. Alle elf Wahrheiten sind durch selbst ausgeführten Code, selbst ausgeführte
Tests und die Gegenprüfung der im SUMMARY behaupteten Testzahlen belegt — nicht nur
durch die SUMMARY-Erzählung. Der dokumentierte historische Defekt (Namensraum-Kollision
zwischen SPA-Route und Backend-Präfix) ist behoben, der Fix ist per Regressionstest
gesichert, und beide Wächtertests wurden in dieser Verifikation selbst ausgeführt und
liefen grün. Alle fünf Requirement-IDs sind gedeckt, keine Requirements sind verwaist.
Die einzige Abweichung ist eine stale ROADMAP-Fortschrittstabelle (informativ, siehe
oben) — kein Code- oder Verhaltensdefekt.

---

_Verifiziert: 2026-09-29_
_Verifier: Claude (gsd-verifier)_
