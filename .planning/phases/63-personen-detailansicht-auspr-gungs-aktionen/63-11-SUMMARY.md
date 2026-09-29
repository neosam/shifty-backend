---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 11
subsystem: testing
tags: [acceptance, uat, clippy, wasm, dev-proxy, dioxus, axum]

requires:
  - phase: 63-01..63-10
    provides: "Personen-Detailansicht, sieben /person-Routen, Anlege-Dialog, Auspraegungs-Aktionen, Rueckbau der beiden Altseiten, Doku-Nachzug"
provides:
  - "Gate-Protokoll der sechs automatisierten Gates mit woertlichen Ausgabezeilen"
  - "Fat-Backend-Nachweis als Grep-Artefakt (PADM-06)"
  - "Rueckbau-Nachweis als Grep-Artefakt (D-63-03, D-63-04)"
  - "End-to-End-Nachweis der Dev-Proxy-Verdrahtung fuer beide Personen-Praefixe aus einem echt laufenden Server-Paar"
affects: [milestone-close-v2.7, verify-work-63]

tech-stack:
  added: []
  patterns:
    - "Dev-Proxy-Nachweis in einem isolierten Netzwerk-Namespace (unshare -Urn), wenn Port 3000 von einem fremden Prozess belegt ist — ohne fremde Prozesse zu beenden und ohne Dioxus.toml anzufassen"

key-files:
  created:
    - ".planning/phases/63-personen-detailansicht-auspr-gungs-aktionen/63-11-SUMMARY.md"
  modified: []

key-decisions:
  - "Akzeptanzkriterium AC-6 des Plans (PersonKindTO-Grep == 0) ist in der geschriebenen Form zu grob: es zaehlt auch Test-Fixtures mit. Die Absicht (keine Auspraegungs-Verzweigung im Produktivcode ausserhalb des reinen Helfers) ist mit einem praezisierten Kommando belegt; am Code wurde nichts geaendert."
  - "Der Dev-Proxy-Nachweis wurde in einem Netzwerk-Namespace gefahren, weil Port 3000 und 8080 von zwei vorbestehenden, 13,5 h alten Fremdprozessen belegt sind. Diese wurden NICHT beendet."

patterns-established:
  - "Proxy-Fingerprint-Test: eine vom Dev-Server selbst beantwortete Anfrage liefert den SPA-Fallback (feste Byte-Groesse, kein set-cookie); eine weitergeleitete Anfrage traegt die Backend-Header. Damit ist 'Praefix ohne Proxy-Eintrag' ohne Browser unterscheidbar."

requirements-completed: [PADM-02, PADM-03, PADM-04, PADM-05, PADM-07]

# Metrics
duration: 45min
completed: 2026-09-29
status: complete
---

# Phase 63 Plan 11: Phasen-Abnahme Summary

**Alle sechs automatisierten Gates sind gruen, beide Grep-Nachweise liegen vor, und die Dev-Proxy-Verdrahtung beider Personen-Praefixe ist end-to-end aus einem echt laufenden Backend-/dx-Paar belegt — der Human-UAT (Tasks 2 und 3) steht noch aus und wurde NICHT selbst freigegeben.**

> **STATUS: NICHT ABGESCHLOSSEN.** Dieser Plan traegt zwei `gate=blocking`
> Human-Checkpoints. Task 1 ist fertig und committed. Tasks 2 und 3 warten auf
> eine Freigabe durch einen Menschen. Nichts in diesem Dokument behauptet eine
> menschliche Bestaetigung, die es nicht gibt.

---

## Task 1 — Automatisierte Gates und Grep-Nachweise (abgeschlossen)

### Gate-Protokoll

Alle Kommandos in `nix develop --command bash -c "…"` gefahren, weil weder `cargo`
noch `node` im Standard-Suchpfad liegen.

| # | Gate | Exit | Ausgabezeile (woertlich) |
|---|------|------|--------------------------|
| 1 | `cargo build --workspace` | 0 | `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.21s` |
| 2 | `cargo test --workspace` | 0 | siehe Tabelle unten — **1168 passed, 0 failed, 3 ignored** |
| 3 | `cargo clippy --workspace -- -D warnings` | 0 | `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.21s` |
| 4 | `cargo test --manifest-path shifty-dioxus/Cargo.toml` | 0 | `test result: ok. 856 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s` |
| 5 | `cargo clippy --manifest-path shifty-dioxus/Cargo.toml -- -D warnings` | 0 | `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 14.36s` |
| 5b | `cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings` | 0 | `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.11s` |
| 6 | `cargo build --manifest-path shifty-dioxus/Cargo.toml --target wasm32-unknown-unknown` | 0 | `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.18s` |

Frontend-Testzahl **856** entspricht exakt der Baseline nach 63-09 — keine Regression,
keine stillschweigend deaktivierten Tests.

#### Backend-Suite im Detail (Gate 2)

```
Running unittests src/lib.rs (…/dao-…)             test result: ok. 11 passed; 0 failed; 0 ignored
Running unittests src/lib.rs (…/dao_impl_sqlite-…) test result: ok. 9 passed; 0 failed; 0 ignored
Running unittests src/lib.rs (…/rest-…)            test result: ok. 33 passed; 0 failed; 0 ignored
Running tests/content_type_surface.rs              test result: ok. 2 passed; 0 failed; 0 ignored
Running tests/openapi_surface.rs                   test result: ok. 3 passed; 0 failed; 0 ignored
Running tests/rebooking_backfill_test.rs           test result: ok. 3 passed; 0 failed; 0 ignored
Running unittests src/lib.rs (…/rest_types-…)      test result: ok. 17 passed; 0 failed; 0 ignored
Running unittests src/lib.rs (…/service-…)         test result: ok. 17 passed; 0 failed; 0 ignored
Running unittests src/lib.rs (…/service_impl-…)    test result: ok. 958 passed; 0 failed; 0 ignored
Running unittests src/lib.rs (…/shifty_utils-…)    test result: ok. 11 passed; 0 failed; 0 ignored
Running unittests src/main.rs (…/shifty_bin-…)     test result: ok. 104 passed; 0 failed; 0 ignored
(eine Doc-Test-Zeile: 0 passed; 0 failed; 3 ignored)
```

Summe: **1168 passed, 0 failed, 3 ignored**. `person_detail` kommt 60-mal im
Backend-Testlauf und 26-mal im Frontend-Testlauf vor.

---

### Fat-Backend-Nachweis (PADM-06, gesperrte Entscheidung)

```
$ grep -rl "PersonKindTO" shifty-dioxus/src --include="*.rs"
shifty-dioxus/src/page/user_management.rs
shifty-dioxus/src/page/person_detail.rs

$ sed -n "1,429p" shifty-dioxus/src/page/user_management.rs | grep -c "PersonKindTO::"
0

$ grep -rn "display_name" shifty-dioxus/src --include="*.rs" | grep -E "unwrap_or|or_else" | wc -l
0
```

Lesart: `PersonKindTO` wird im Frontend nur an zwei Stellen erwaehnt. In
`person_detail.rs` liegt der reine Sichtbarkeits-Helfer samt seinen Tests — dort
gehoert die Verzweigung hin. In `user_management.rs` steht **keine einzige**
Erwaehnung im Produktivteil (Zeilen 1–429); alle 20 Treffer liegen ab Zeile 526
und damit innerhalb von `#[cfg(test)] mod tests`, das in Zeile 431 beginnt:

```
$ grep -n "cfg(test)" shifty-dioxus/src/page/user_management.rs
430:#[cfg(test)]
431:mod tests {
```

Auf dem Anzeigenamen liegt im Frontend keine `unwrap_or`-/`or_else`-Kette; der
Name kommt fertig vom Server (`rest-types/src/lib.rs:2869`, mit der ausdruecklichen
Notiz in Zeile 2932: „`kind` und `display_name` einer Verzeichniszeile werden NIE
vom Client …").

**Abweichung vom Plan-Wortlaut — siehe Abschnitt „Abweichungen", D-1.**

---

### Rueckbau-Nachweis (D-63-03, D-63-04)

```
$ grep -rn "Route::UserDetails\|Route::SalesPersonDetails" shifty-dioxus/src --include="*.rs" | wc -l
0

$ grep -rn "mod user_details\|mod sales_person_details" shifty-dioxus/src --include="*.rs" | wc -l
0

$ test -f shifty-dioxus/src/page/user_details.rs -o -f shifty-dioxus/src/page/sales_person_details.rs; echo $?
1

$ grep -c "CreateNewSalesPerson\|AddNewUser" shifty-dioxus/src/page/user_management.rs
0

$ grep -rn "person/:person_id" shifty-dioxus/src --include="*.rs"
shifty-dioxus/src/router.rs:52:    #[route("/person/:person_id/")]
```

Die beiden Routen-Bezeichner sind weg, die beiden Seitendateien existieren nicht
mehr, die Werkzeugleiste der Listenseite nennt keine der beiden alten
Anlege-Beschriftungen mehr, und die neue Route ist registriert.

> **Stand dieses Protokolls: vor dem UAT.** Die letzte Zeile zeigt die Route
> noch als `/person/:person_id/`. Genau dieser Pfad war der Defekt, den der
> UAT-Durchgang 1 aufgedeckt hat — sie liegt seit `69737ac6` auf
> `/person-detail/:person_id/`. Die Ausgabe bleibt hier unveraendert stehen,
> weil sie festhaelt, was zu diesem Zeitpunkt gemessen wurde.

Es verbleiben 13 Treffer auf die reinen **i18n-Schluessel** `Key::UserDetails`,
`Key::SalesPersonDetails` und `Key::LoadingSalesPersonDetails` in
`i18n/{de,en,cs}.rs` + `i18n/mod.rs`. Das sind Teil der bereits protokollierten
14 verwaisten i18n-Schluessel (Deferred-Item aus 63-09), keine Routen-Referenzen.
`Key::LoadingSalesPersonDetails` wird sogar weiter benutzt
(`page/person_detail.rs:683`).

---

### Gesperrte Nicht-Aenderungen

```
$ grep -c "CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13" service_impl/src/billing_period_report.rs
1

$ git status --porcelain migrations/sqlite | wc -l
0
```

Die Snapshot-Schema-Version steht unveraendert auf 13, und unter
`migrations/sqlite/` ist keine Datei hinzugekommen. Beides entspricht dem
gesperrten Rahmen der Phase.

---

## Task 2 — Dev-Proxy: automatisierter Teil belegt, Human-Teil offen

### Ausgangslage: zwei vorbestehende Fremdprozesse

```
$ ps -o pid,lstart,etime,cmd -p 460885,461069
    PID                  STARTED     ELAPSED CMD
 460885  So Sep 20 11:24:03 2026    13:39:11 target/debug/shifty_bin
 461069  So Sep 20 11:24:07 2026    13:39:07 dx serve
```

Beide stammen aus einer frueheren Sitzung und belegen Port 3000 bzw. 8080. Sie
sind aelter als die Phase: das laufende Backend fuehrt eine **geloeschte**
Binary aus (`/proc/460885/exe -> …/target/debug/shifty_bin (deleted)`, Stand
11:24), und der laufende `dx serve` ist aelter als der Commit, der den
Proxy-Eintrag ueberhaupt eingefuehrt hat (`b527aaa9`, 2026-09-20 23:05).

Beweis, dass der Prozess auf :3000 die Phase nicht kennt:

```
$ curl -s http://localhost:3000/api-docs/openapi.json | (Pfade unter /person)
['/person-directory/']          # nur die Phase-62-Route

$ curl -s http://localhost:3099/api-docs/openapi.json | (Pfade unter /person)   # frisch gebaut
/person                                         ['POST']
/person-directory/                              ['GET']
/person/by-sales-person/{sales_person_id}       ['GET']
/person/candidates/sales-persons                ['GET']
/person/candidates/users                        ['GET']
/person/{person_id}                             ['GET']
/person/{person_id}/facet                       ['POST']
/person/{person_id}/facet/{facet}               ['DELETE']
```

**Keiner der beiden Fremdprozesse wurde beendet.** Am Ende des Laufs sind beide
unveraendert am Netz; alle von mir gestarteten Prozesse sind gestoppt.

### Nachweis A — die sieben Routen antworten (frisches Backend, Port 3099)

```
GET  /person/{linked}                        -> HTTP 200
GET  /person/by-sales-person/{sp}            -> HTTP 200
GET  /person/candidates/users                -> HTTP 200
GET  /person/candidates/sales-persons        -> HTTP 200
POST /person                (leerer Body)    -> HTTP 422   (Route existiert, Body-Validierung greift)
POST /person/{id}/facet     (leerer Body)    -> HTTP 422   (dito)
DEL  /person/{id}/facet/unbekannt            -> HTTP 400   (Route existiert, Facet-Name abgelehnt)
```

Kein 404 und kein 405 — die Routen sind real registriert, nicht nur in der
OpenAPI beschrieben.

### Nachweis B — Dev-Proxy end-to-end, unveraenderte `Dioxus.toml`

Weil Port 3000 belegt ist, liefen ein **frisch gebautes Backend auf 127.0.0.1:3000**
und ein **frischer `dx serve` auf :8080** gemeinsam in einem isolierten
Netzwerk-Namespace (`unshare -Urn`). `shifty-dioxus/Dioxus.toml` wurde dabei
**nicht angefasst** (`git status` sauber). Alle Anfragen gehen ueber den
Dev-Server:

```
REQUEST (ueber dx :8080)                                   HTTP
/user_management/                                          200  (bytes=8298)   <- SPA, unveraendert
/person-directory                                          200  (bytes=1153)   <- weitergeleitet, JSON
/person/2f2438f0-6dd3-4043-8ecc-2e5b2e3d3325               200  (bytes=255)    <- NEUE ROUTE, weitergeleitet
/person/by-sales-person/100e013e-2ed4-4da3-8c70-a37110d1da2e 200  (bytes=255)
/person/candidates/users                                   200  (bytes=61)
/person/candidates/sales-persons                           200  (bytes=195)
/nonexistent-prefix/x                                      200  (bytes=8298)   <- Kontrolle: SPA-Fallback

Detail-Antwort ueber den Dev-Proxy (gekuerzt):
{"person_id":"2f2438f0-…","kind":"linked","display_name":"Béla Verknüpft","username":"bela.login",
 "sales_person_id":"100e013e-…","background_color":"#ffb703","is_paid":true,"inactive":false,"roles":[]}
```

Damit sind beide Haelften der Fehlerklasse ausgeschlossen:

* Der **neue** Praefix wird weitergeleitet (200 mit Detail-Aggregat statt
  8298-Byte-SPA-Fallback).
* Der **bestehende** Praefix wird vom neuen Eintrag **nicht** gefressen
  (`/person-directory` liefert weiter seine 1153 Byte JSON, `/user_management/`
  weiter die SPA).

Die Unterscheidung ist eindeutig, weil der Dev-Server bei fehlendem Proxy-Eintrag
immer denselben 8298 Byte grossen SPA-Fallback ausliefert — nachgemessen an der
Kontrollanfrage `/nonexistent-prefix/x`.

Ergaenzend, gegen den stale Prozess auf :3000 und einen frischen `dx serve` auf
:8099 gefahren, zeigt derselbe Test den Proxy-Fingerprint isoliert: `/person/{id}`
liefert dort **404 mit `set-cookie`** — also die Antwort des Backends, nicht des
Dev-Servers. Selbst mit einem Backend ohne die Route ist damit belegt, dass der
Proxy-Eintrag greift.

Statische Reihenfolge-Kontrolle in `shifty-dioxus/Dioxus.toml`:

```
118: backend = "http://localhost:3000/person-directory"
119: # Phase 63 (63-06): die Personen-Detailansicht und ihre Auspraegungs-Aktionen.
130: backend = "http://localhost:3000/person"
```

und in `rest/src/lib.rs`:

```
730: .nest("/person-directory", person_directory::generate_route())
736: .nest("/person",           person_detail::generate_route())
```

### Was an Task 2 noch offen ist

Die Schritte 3–5 der `how-to-verify` verlangen einen **Blick in die
Entwicklerwerkzeuge eines Browsers** durch einen Menschen. Das ist nicht
erledigt. Siehe Abschnitt „Offen".

---

## Task 3 — UAT: offen

Nicht durchgefuehrt. Kein Punkt A bis I ist beantwortet. Es liegt **keine**
menschliche Bestaetigung zu den drei Auspraegungen, zum einen Anlege-Einstieg,
zu den Bestaetigungstexten oder zum Wegfall der Zeilen-Loeschung vor.

---

## Abweichungen vom Plan

### D-1 — [Rule 3 / Plan-Artefakt] Akzeptanzkriterium AC-6 ist in der geschriebenen Form zu grob

* **Gefunden bei:** Task 1
* **Befund:** Das Plan-Kriterium
  `grep -rn "PersonKindTO::" shifty-dioxus/src --include=*.rs | grep -v 'page/person_detail.rs' | grep -v 'i18n/' | wc -l` soll 0 ergeben, liefert aber **20**.
* **Analyse:** Alle 20 Treffer liegen in `shifty-dioxus/src/page/user_management.rs`
  in den Zeilen 526–762 — und damit vollstaendig innerhalb des
  `#[cfg(test)] mod tests`-Blocks, der in Zeile 431 beginnt. Es sind
  Test-Fixtures (`sample_entry(PersonKindTO::Linked, …)`) fuer die
  SSR-Rendertests der Listenseite, keine Auspraegungs-Ableitung im Produktivcode.
  Das Kriterium filtert den Helfer und die i18n-Dateien heraus, aber nicht die
  Test-Module der uebrigen Dateien.
* **Entscheidung:** **Kein Code geaendert.** Die vom Kriterium gemeinte Eigenschaft
  („die Auspraegungs-Verzweigung lebt ausschliesslich im reinen Helfer und seinen
  Tests") ist erfuellt und mit dem praezisierten Kommando oben belegt. Der Fehler
  liegt im Plan-Artefakt, nicht im Code. Test-Fixtures wegzuoptimieren, nur damit
  ein Grep-Zaehler stimmt, waere das falsche Ziel.
* **Commit:** dokumentarisch, kein Code-Commit.

### D-2 — [Rule 3] Dev-Proxy-Nachweis in einem Netzwerk-Namespace statt auf den Standardports

* **Gefunden bei:** Task 2
* **Befund:** Port 3000 und 8080 sind von zwei vorbestehenden Fremdprozessen
  belegt (PID 460885 / 461069, 13,5 h alt). Der Prozess auf :3000 fuehrt eine
  geloeschte Binary ohne die `/person`-Routen aus.
* **Entscheidung:** Die Fremdprozesse **nicht** beendet. Stattdessen Backend und
  `dx serve` gemeinsam in `unshare -Urn` gestartet, wo 127.0.0.1:3000 frei ist.
  `Dioxus.toml` blieb dabei unveraendert — der Nachweis gilt fuer die committete
  Datei.
* **Folge fuer den Human-UAT:** Fuer den Browser-Durchgang muessen die beiden
  Fremdprozesse neu gestartet werden. Das entscheidet der Nutzer, nicht ich.

---

## Deferred Items

Aus frueheren Plaenen uebernommen, hier gebuendelt (nicht neu aufgerollt):

| # | Item | Herkunft |
|---|------|----------|
| DEF-1 | 14 verwaiste i18n-Schluessel nach dem Rueckbau (u. a. `Key::UserDetails`, `Key::SalesPersonDetails`) | 63-09 |
| DEF-2 | `SelectedSalesPerson::user_id` ist nach dem Rueckbau unerreichbar | 63-09 |
| DEF-3 | Doppelte Registrierung von `Key::CopyToClipboard` in `i18n/de.rs` und `i18n/en.rs` | 63-09 |
| DEF-4 | Vorbestehendes `needless_lifetimes` in `service_impl/src/test/person_directory_dao.rs:149` | frueher |
| DEF-5 | `session.user_id` traegt kein `ON DELETE CASCADE` — das Loeschen eines Benutzers mit offener Sitzung laeuft in einen Fremdschluessel-Fehler und erscheint als generischer 500 (T-63-05-E, RESEARCH Pitfall 3, Threat T-63-11-D „accept") | Phase 63, vorbestehend |
| DEF-6 | `/shiftplan/` (SPA) und `.nest("/shiftplan", ...)` (Backend) teilen sich ein Pfad-Segment — die Schichtplan-Seite ist ueber ihre eigene URL nicht erreichbar. Gemessen: `GET /shiftplan/` -> 404, `GET /shiftplan/2026/38` -> 404, waehrend `/absences/` und `/settings/` sauber die SPA liefern. Nur Navigation aus der laufenden Anwendung heraus funktioniert, weil der Router sie clientseitig aufloest — daher nie aufgefallen. Vorbestehend, nicht aus Phase 63; Behebung gehoert in eine eigene Phase. | Phase 63 gefunden, vorbestehend |

**DEF-5: als Kante akzeptiert, Beobachtung nicht separat belegt.** Punkt H des
UAT sah vor, den Fall einmal bewusst auszuloesen. Der Nutzer hat DEF-5 beim
Abschluss ausdruecklich als bekannte, in dieser Phase nicht zu behebende Kante
durchgewinkt; ob der 500er dabei tatsaechlich einmal gesehen wurde, ist nicht
gesondert dokumentiert und wird hier deshalb auch nicht behauptet. Der Eintrag
bleibt als Deferred Item offen.

---

## UAT-Durchgang 1 — abgebrochen, ein echter Defekt gefunden und behoben

Der Nutzer hat den UAT begonnen und zwei Beobachtungen gemeldet: **405 beim
Anlegen** und **beim Bearbeiten erscheint gar nichts**. Die Untersuchung hat
zwei verschiedene Ursachen ergeben.

### Befund 1 — der 405 kam von den veralteten Dev-Servern

Mit frischem Server-Paar laeuft `POST /person` durch den Proxy sauber ans
Backend (422 auf eine absichtlich unvollstaendige Payload, also Validierung
statt Routing). Der `dx serve` des Nutzers startete vor Commit `b527aaa9`,
der den `/person`-Proxy-Eintrag eingefuehrt hat. Ohne Proxy-Eintrag landet
ein POST beim statischen Dateiserver von `dx` — und der antwortet 405.
Kein Code-Defekt.

### Befund 2 — Namensraum-Kollision zwischen SPA-Route und API-Praefix

Die Frontend-Route lag auf `/person/:person_id/` und damit im selben
Pfad-Segment wie `.nest("/person", ...)` aus 63-06. Der Dev-Proxy gibt das
Segment vollstaendig ans Backend ab. Gemessen mit frischem Server-Paar:

    GET /person/<uuid>     -> 200 (209 B JSON)   API-Pfad, korrekt
    GET /person/<uuid>/    -> 404                die SPA-Route, kaputt
    GET /user_management/  -> 200 (8298 B SPA)   Kontrolle

Die Detailseite war ueber ihre eigene URL nicht erreichbar: direkte Eingabe,
Neuladen und Lesezeichen liefen in einen 404 vom Backend.

**Warum der Nachweis aus Task 2 das nicht gesehen hat.** Er prueft
`/person/<uuid>` **ohne** abschliessenden Schraegstrich — das ist der
API-Pfad, und der funktioniert. Die SPA-Route traegt einen Schraegstrich am
Ende und wurde nie angefasst. Der 200er wurde als Erfolg protokolliert,
obwohl er den Defekt belegt. Die 26 SSR-Rendertests aus 63-08 rendern die
Komponente direkt und fassen das Routing nie an.

### Behebung

| Commit | Inhalt |
|--------|--------|
| `69737ac6` | Route auf `/person-detail/:person_id/`. Alle fuenf Aufrufstellen nutzen den typisierten Konstruktor `Route::PersonDetail`, darum genuegt das Attribut. Der API-Praefix bleibt unangetastet — er steht in F01 §5.5, in sieben OpenAPI-Pfaden und in den Integrationstests. |
| `94c13c5d` | `docs/architecture/06-frontend.md` + `_de.md`: Routen-Inventar und Fliesstext auf den neuen Pfad, plus ein Absatz, warum die Route nicht unter `/person/` liegen darf. |
| `f289cb0b` | Zwei Waechter-Tests in `router.rs`. Der erste liest die `[[web.proxy]]`-Eintraege aus `Dioxus.toml` und `Route::SITE_MAP` und vergleicht die Erstsegmente — er schliesst die Luecke fuer jede kuenftige Route. Beide per Mutation belegt (Route zurueckgesetzt -> beide FAILED mit dem Segment `person` in der Meldung), Mutation zurueckgenommen. |
| `a7767219` | Korrektur einer falschen Behauptung im Kommentar zu DEF-6 (siehe unten). |

### Nachweis nach dem Fix

Frisches Backend + frischer `dx serve` im Netzwerk-Namespace, SPA-Signatur
8298 B als Referenz aus `/user_management/`:

    /person-detail/<uuid>/   -> 200   8298 B  [SPA]
    /person/<uuid>           -> 200    209 B  [JSON, unveraendert]
    POST /person             -> 422           [erreicht das Backend]

Gates nach dem Fix: `cargo test` shifty-dioxus **858 passed** (856 + 2 neue),
`cargo clippy --all-targets -- -D warnings` Exit 0, wasm32-Build Exit 0.

### Neuer Deferred Item aus der Untersuchung

Die Suche nach weiteren Kollisionen hat eine **vorbestehende** gefunden, die
nicht aus dieser Phase stammt und hier bewusst nicht behoben wird — siehe
DEF-6.

---

## UAT-Durchgang 2 — bestanden

Vorbedingung erfuellt: die beiden veralteten Fremdprozesse waren zum zweiten
Durchgang beendet, Backend und `dx serve` liefen frisch gebaut aus dem
aktuellen Stand. Smoke-Test vor der Uebergabe an den Nutzer, SPA-Signatur
8298 B als Referenz aus `/user_management/`:

```
/user_management/         -> 200   8298 B  [SPA]
/person-detail/<uuid>/    -> 200   8298 B  [SPA]     vorher 404
/person/<uuid>            -> 200    209 B  [JSON]
POST /person              -> 422                     vorher 405
```

### Task 2 — Dev-Proxy im Browser

Freigegeben. Der Nutzer hat `/user_management/` geoeffnet, ueber „Bearbeiten"
die Detailansicht erreicht und keine fehlschlagenden Requests gemeldet.

### Task 3 — Punkte A bis I

| Punkt | Gegenstand | Ergebnis |
|---|---|---|
| A | ein Anlege-Einstieg, die beiden alten Knoepfe weg | freigegeben |
| B | Anlege-Dialog: Name, zwei Umschalter, Sperre bei leerer Auswahl | freigegeben |
| C | drei Auspraegungen, je passende Abschnitte, Karte statt leerem Abschnitt | freigegeben |
| D | Login zuschalten ohne Seitenwechsel | freigegeben |
| E | Verkaeufer verknuepfen ueber Auswahlliste, nicht Tippfeld | freigegeben |
| F | drei Loesch-Bestaetigungen verstaendlich | freigegeben |
| G | reines Benutzerkonto loeschen, verschwindet aus der Liste | freigegeben |
| H | DEF-5 beobachten (generischer 500 bei offener Sitzung) | als Kante akzeptiert, Beobachtung nicht gesondert belegt — siehe DEF-5 |
| I | keine Zeilen-Loeschung mehr in der Liste (D-63-03) | **ausdruecklich zugestimmt** |

**Punkt I war die einzige Produktentscheidung im UAT.** Die generische
Zeilen-Bestaetigung konnte die auspraegungs-bewusste Zusage nicht halten;
Loeschen lebt seit 63-09 nur noch auf der Detailseite. Der Nutzer hat dem auf
ausdrueckliche Nachfrage zugestimmt — nicht durch ein allgemeines „sieht gut
aus", sondern nach gezielter Vorlage der Entscheidung.

**Wortlaut-Treue:** Der Nutzer hat A–G mit „Sieht gut aus" freigegeben und I
sowie H anschliessend auf Nachfrage mit „Beides ok". Punkt-fuer-Punkt-Protokolle
pro Kriterium liegen nicht vor; die Tabelle gibt wieder, was freigegeben wurde,
nicht mehr.

**`requirements-completed` ist jetzt gefuellt:** PADM-02, PADM-03, PADM-04,
PADM-05 und PADM-07. Dieser Plan ist die dafuer vorgesehene Stelle — zehn der
elf Plaene der Phase tragen dieselben IDs, deshalb wurden sie in keinem
vorherigen Plan abgehakt.

---

## Self-Check: PASSED (fuer Task 1)

```
$ [ -f .planning/phases/63-…/63-11-SUMMARY.md ] && echo FOUND
FOUND

$ git status --porcelain shifty-dioxus/Dioxus.toml | wc -l
0                      # der Proxy-Nachweis lief gegen die committete Datei

$ git status --porcelain
?? .planning/phases/63-…/63-11-SUMMARY.md     # kein weiterer Diff in dieser Phase

$ ss -ltnp | grep -E ':(3000|3099|8080|8099)'
… :8080  dx,pid=461069            # Fremdprozess, unangetastet
… :3000  shifty_bin,pid=460885    # Fremdprozess, unangetastet
```

Alle von mir gestarteten Prozesse (Backend auf :3099, `dx serve` auf :8099, das
Paar im Netzwerk-Namespace) sind beendet. Die beiden Fremdprozesse laufen
unveraendert weiter.

**Tasks 2 und 3 sind NICHT self-checked, weil sie nicht durchgefuehrt wurden.**
