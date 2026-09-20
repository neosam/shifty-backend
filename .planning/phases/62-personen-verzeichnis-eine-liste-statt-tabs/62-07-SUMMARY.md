---
phase: 62-personen-verzeichnis-eine-liste-statt-tabs
plan: 07
subsystem: testing
tags: [dev-proxy, smoke-test, runtime-verification, uat-checkpoint, grep-artifact, mock-auth]

# Dependency graph
requires:
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-04 — Frontend-Umbau, `load_directory_ist_ein_request`, Ziel des Fat-Backend-Greps"
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-05 — `GET /person-directory`, `.nest(...)`-Praefix, `[[web.proxy]]`-Eintrag"
provides:
  - "Dev-Proxy-Smoke-Protokoll: Port 3000 und Port 8080 liefern denselben Statuscode und denselben Body"
  - "Praefix-Konsistenz-Nachweis ueber vier Fundstellen mit Datei und Zeilennummer"
  - "Fat-Backend-Grep-Artefakt (Kommando + woertliche Ausgabe)"
  - "Ein-Request-Nachweis (Unit-Test gruen) plus ein validierter Laufzeit-Zaehler im Backend-Log"
affects: [62-verifikation, 63-personen-anlage]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Doppel-Curl-Smoke: identischer Statuscode gegen Backend-Port und Dev-Proxy-Port als harter Gate fuer die `[[web.proxy]]`-Liste"
    - "Handler-Span im Backend-Log als exakter Request-Zaehler: die DAO-Join-Query erscheint genau einmal pro Handler-Aufruf"

key-files:
  created:
    - ".planning/phases/62-personen-verzeichnis-eine-liste-statt-tabs/62-07-SUMMARY.md"
  modified: []

key-decisions:
  - "Der Ein-Request-Laufzeitbeleg wird ueber die DAO-Join-Query im Backend-Log gezaehlt statt ueber ein HTTP-Access-Log — `tower_http::trace` ist in diesem Build nicht aktiv (0 Treffer), die Handler-Span-Zeile dagegen ist 1:1 zur Request-Zahl (validiert: 4 Requests → 4 Zeilen, +1 Request → 5 Zeilen)"
  - "Der Fat-Backend-Grep wurde zusaetzlich auf den gesamten FE-Datenpfad ausgeweitet; `display_name` kommt im Frontend ausschliesslich in `page/user_management.rs` vor, die Grep-Zielwahl des Plans ist damit vollstaendig"

patterns-established:
  - "Proxy-Smoke vor UAT: der Statuscode-Vergleich 3000 vs. 8080 laeuft, bevor ein Mensch den Browser oeffnet — ein fehlender Proxy-Eintrag wird so nicht als UI-Fehler fehldiagnostiziert"

requirements-completed: []  # bewusst leer — PADM-01/06/08 haengen am offenen Checkpoint (Task 2)

coverage:
  - id: D1
    description: "Der Routen-Praefix ist ueber alle vier Fundstellen zeichengleich"
    requirement: "PADM-01"
    verification:
      - kind: automated_ui
        ref: "grep -rho \"person-directory\" rest/src/lib.rs shifty-dioxus/Dioxus.toml shifty-dioxus/src/api.rs | sort -u | wc -l  →  1"
        status: pass
    human_judgment: false
  - id: D2
    description: "Der Dev-Proxy reicht den neuen Praefix weiter — Port 8080 antwortet wie Port 3000"
    requirement: "PADM-01"
    verification:
      - kind: integration
        ref: "curl -s -o /dev/null -w '%{http_code}' http://localhost:3000/person-directory → 200 ; http://localhost:8080/person-directory → 200 ; Bodies byte-gleich"
        status: pass
    human_judgment: false
  - id: D3
    description: "Genau ein Aggregat-Request fuellt die Liste (Code-Ebene)"
    requirement: "PADM-06"
    verification:
      - kind: unit
        ref: "shifty-dioxus: service::user_management::tests::load_directory_ist_ein_request -- --exact"
        status: pass
    human_judgment: false
  - id: D4
    description: "Im Frontend existiert keine Namens-Fallback-Regel"
    requirement: "PADM-08"
    verification:
      - kind: other
        ref: "grep -n 'display_name\\|username' shifty-dioxus/src/page/user_management.rs | grep -c 'unwrap_or\\|\\.or(' → 0"
        status: pass
    human_judgment: false
  - id: D5
    description: "Ein Admin sieht drei Auspraegungen nebeneinander mit sichtbarem Badge; Suche ueber beide Namen; genau ein Request im Netzwerk-Tab; Direktaufruf ohne Admin-Rolle wird abgewiesen"
    requirement: "PADM-01"
    verification: []
    human_judgment: true
    rationale: "Gerenderte Seite, Badge-Unterscheidbarkeit, Suchverhalten und der Ablehnungstext sind visuelle Aussagen; ausserdem muessen die drei Auspraegungen erst ueber die Oberflaeche angelegt werden — die lokale Datenbank enthaelt nur zwei `user_only`-Eintraege."

# Metrics
duration: ~25 min (Task 1)
completed: 2026-09-20
status: checkpoint-pending
---

# Phase 62 Plan 07: Laufzeit-Verifikation und UAT — Zwischenstand (Task 1)

Der Dev-Proxy reicht `/person-directory` nachweislich weiter (200 auf Port 3000
**und** auf Port 8080, byte-gleicher Body); Praefix-Konsistenz, Ein-Request-Test
und das Fat-Backend-Grep-Artefakt liegen vor. Die visuelle Bestaetigung (Task 2)
steht aus.

**Dieser Plan ist NICHT abgeschlossen.** Task 2 ist ein blockierender
`checkpoint:human-verify`. Kein Punkt dieses Dokuments behauptet, dass ein
Mensch die gerenderte Seite gesehen hat.

## Task 1 — abgeschlossen

### a) Praefix-Konsistenz statisch belegt

Verify-Kommando des Plans:

```
$ grep -rho "person-directory" rest/src/lib.rs shifty-dioxus/Dioxus.toml shifty-dioxus/src/api.rs | sort -u | wc -l
1
```

Genau eine Schreibweise. Die vier Fundstellen:

| # | Rolle | Datei:Zeile | Inhalt |
|---|---|---|---|
| 1 | Router-Nest | `rest/src/lib.rs:706` | `.nest("/person-directory", person_directory::generate_route())` |
| 2 | OpenAPI-Nest | `rest/src/lib.rs:640` | `(path = "/person-directory", api = person_directory::PersonDirectoryApiDoc),` |
| 3 | Frontend-URL | `shifty-dioxus/src/api.rs:881` | `let url = format!("{}/person-directory", config.backend);` |
| 4 | Dev-Proxy | `shifty-dioxus/Dioxus.toml:118` | `backend = "http://localhost:3000/person-directory"` |

### b) Backend-Smoke (Port 3000)

Backend gestartet mit
`nix develop --command bash -c 'SQLX_OFFLINE=true DATABASE_URL=sqlite:./localdb.sqlite3 cargo run'`
(Standard-Features, `mock_auth`, Entwicklungs-Admin `DEVUSER`).

```
$ curl -s -o /dev/null -w '%{http_code}' http://localhost:3000/person-directory
200
```

Body (vollstaendig, lokale Datenbank `localdb.sqlite3`):

```json
[{"person_id":"836f3408-d276-7972-eaea-7102df9b7903","kind":"user_only","display_name":"admin","username":"admin","sales_person_id":null,"background_color":null,"is_paid":null,"inactive":null,"roles":["admin"]},{"person_id":"e2030e48-9c16-8b99-f4d8-6ce46eee4ef7","kind":"user_only","display_name":"DEVUSER","username":"DEVUSER","sales_person_id":null,"background_color":null,"is_paid":null,"inactive":null,"roles":["admin"]}]
```

Zwei Eintraege, beide mit gesetztem Auspraegungs-Feld (`kind: "user_only"`) und
nicht-leerem `display_name` (`"admin"`, `"DEVUSER"`). Dass `DEVUSER` selbst als
`user_only` erscheint, ist korrekt: seit D-61-01 bekommt jeder neue Benutzer eine
`person`-Identitaet.

**Datenlage-Hinweis an Task 2:** Die lokale Datenbank enthaelt **nur** die
Auspraegung `user_only`. Es gibt weder einen `sales_person_only`- noch einen
verknuepften Eintrag. Schritt 1 des Checkpoints (Datensaetze ueber die
Oberflaeche anlegen) ist daher **nicht optional** — ohne ihn sind die Schritte
3, 4, 7 und 8 nicht pruefbar.

### c) Dev-Proxy-Smoke (Port 8080) — der eigentliche Zweck dieses Plans

Frontend-Dev-Server gestartet mit
`nix develop --command bash -c 'cd shifty-dioxus && dx serve'`; Port 8080 antwortete
nach ca. 10 s (`curl http://localhost:8080/` → `200`).

```
$ curl -s -o /dev/null -w '%{http_code}' http://localhost:3000/person-directory
200
$ curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/person-directory
200
```

| Ziel | Statuscode |
|---|---|
| `http://localhost:3000/person-directory` (Backend direkt) | **200** |
| `http://localhost:8080/person-directory` (ueber Dev-Proxy) | **200** |

Identisch, und **kein** 404 und **kein** 405. Der Body ueber den Proxy war
byte-gleich mit dem Body gegen Port 3000. Der 3× wiederholte Proxy-Incident
tritt in dieser Phase nicht auf.

### d) Ein-Request-Nachweis

Primaernachweis (Unit-Test aus 62-04):

```
$ nix develop --command bash -c 'cd shifty-dioxus && cargo test service::user_management::tests::load_directory_ist_ein_request -- --exact'
running 1 test
test service::user_management::tests::load_directory_ist_ein_request ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 834 filtered out; finished in 0.00s
```

Laufzeit-Beleg: Das Backend fuehrt **kein** HTTP-Access-Log
(`grep -c 'tower_http::trace\|started processing request\|finished processing request'`
auf dem Backend-Log ergibt `0`). Stattdessen traegt jede `sqlx`-Logzeile des
Handlers den Span `in rest::person_directory::get_person_directory with context:
Some("DEVUSER")`, und die DAO-Drei-Wege-Outer-Join-Query erscheint **genau einmal
pro Handler-Aufruf**. Das wurde als Zaehler validiert:

```
# nach 4 gesendeten Requests
$ grep -c 'FROM person p' backend.log
4
# ein weiterer Request
$ curl -s -o /dev/null http://localhost:8080/person-directory
$ grep -c 'FROM person p' backend.log
5
```

Damit ist der Zaehler 1:1 zur Request-Zahl. Die Zahl der Treffer **waehrend eines
einzelnen Seitenaufrufs im Browser** konnte ohne Browser nicht erhoben werden und
wird — wie im Plan vorgesehen — dem visuellen Netzwerk-Tab-Nachweis aus Task 2
zugeschlagen. Der Mensch kann dort denselben Zaehler als zweiten, unabhaengigen
Beleg benutzen (Kommando in der Checkpoint-Anleitung).

### e) Fat-Backend-Grep-Artefakt (Success Criterion 3)

Kommando und Ausgabe woertlich:

```
$ grep -n 'display_name\|username' shifty-dioxus/src/page/user_management.rs | grep -c 'unwrap_or\|\.or('
0
```

Die vollstaendige Ausgabe des inneren Greps (alle Zeilen, die `display_name` oder
`username` nennen) — keine davon enthaelt `unwrap_or` oder `.or(`:

```
115:                on_request_delete: move |username: ImStr| delete_user_confirm.set(Some(username)),
117:                on_impersonate: move |username: ImStr| {
118:                    impersonate_service.send(ImpersonateAction::Start(username))
141:            on_confirm: move |username: ImStr| {
142:                user_management_service.send(UserManagementAction::DeleteUser(username));
208:            let Some(username) = props.target.clone() else {
215:        .replace("{username}", username.as_str());
219:    let username_for_confirm = username.clone();
236:                    on_click: move |_| on_confirm.call(username_for_confirm.clone()),
274:            let hit_title = matches_search(&entry.display_name, &term);
276:                .username
349:                        let title = entry.display_name.clone();
350:                        let login: Option<ImStr> = entry.username.as_deref().map(ImStr::from);
528:        display_name: &str,
529:        username: Option<&str>,
535:            display_name: Arc::from(display_name),
536:            username: username.map(Arc::from),
599:        assert!(html.contains("lmueller"), "username cell missing: {html}");
608:    fn directory_user_only_row_has_display_name() {
620:        assert!(html.contains("font-mono"), "username cell missing: {html}");
805:    fn delete_user_confirm_dialog_open_interpolates_username_and_has_danger_button() {
809:        assert!(html.contains("alex"), "username not interpolated: {html}");
```

Zeile 349 ist die Kernaussage: `let title = entry.display_name.clone();` — der
Anzeigename wird unveraendert vom Backend uebernommen, ohne Ableitungsregel.

**Zusatz-Grep (ueber den Planauftrag hinaus, zur Absicherung der Absicht):** Die
Fallback-Suche wurde auf den gesamten FE-Datenpfad ausgeweitet:

```
$ for f in shifty-dioxus/src/service/user_management.rs shifty-dioxus/src/state/user_management.rs shifty-dioxus/src/api.rs; do grep -n 'display_name\|username' "$f" | grep -c 'unwrap_or\|\.or(' ; done
0
0
0
$ grep -rln 'display_name' shifty-dioxus/src/
shifty-dioxus/src/page/user_management.rs
```

`display_name` kommt im gesamten Frontend ausschliesslich in
`page/user_management.rs` vor — die Zielwahl des Plan-Greps ist damit
vollstaendig, nicht nur stichprobenhaft.

### Zusatz-Grep zur Tab-Freiheit

```
$ grep -rl 'UserManagementTabBar' shifty-dioxus/src/ | wc -l
0
```

Kein Restverweis auf die Tab-Leiste.

### f) Aufraeumen

Beide Server sind beendet:

```
$ ss -tln | grep -E ':(3000|8080) '
OK: kein Prozess mehr auf 3000 oder 8080
$ pgrep -af 'dx serve|shifty_bin'
OK: keine Serverprozesse mehr
```

Kein Schritt von b) bis e) ist fehlgeschlagen; es gibt aus Task 1 keinen Gap
gegen 62-04 oder 62-05.

## Task 2 — OFFEN (checkpoint:human-verify, blockierend)

**Status: nicht durchgefuehrt.** Die Schritte 3 bis 9 der Checkpoint-Anleitung
verlangen eine gerenderte Seite im Browser und ein menschliches Urteil. Sie sind
hier **nicht** bestaetigt:

| Schritt | Kriterium | Status |
|---|---|---|
| 3 | Keine Tab-Leiste, drei Auspraegungen in einer Liste, Badge je Zeile | **offen** |
| 4 | Suche greift ueber Verkaeufer- **und** Benutzernamen | **offen** |
| 5 | Reines Benutzerkonto zeigt sichtbaren Namen, kein Platzhalter | **offen** |
| 6 | Netzwerk-Tab: genau **ein** Request auf `/person-directory`, Status 200 | **offen** (Code-Ebene gruen, Laufzeit-Zahl unbeziffert) |
| 7 | Impersonation/Loeschen fehlen beim reinen Verkaeufer (D-62-06) | **offen** |
| 8 | Bearbeiten-Link zielt auspraegungsabhaengig (D-62-07) | **offen** |
| 9 | Direktaufruf ohne Admin-Rolle wird abgewiesen; Ablehnungstext woertlich | **offen** |

Offene Pflichtangaben, die nach der Freigabe hier nachzutragen sind:
die im Netzwerk-Tab beobachtete Request-Zahl (bezifferte Zahl) und der beim
Direktaufruf ohne Admin-Rolle angezeigte Text (woertlich).

## Deviations from Plan

**1. [Rule 3 — blockierendes Werkzeug] Laufzeit-Zaehler statt HTTP-Access-Log**
- **Gefunden bei:** Task 1, Schritt d
- **Problem:** Der Plan verlangt „die Zahl der Treffer auf den neuen Pfad im
  Zugriffs-Log des Backends". Der Build fuehrt kein `tower_http::trace`-Layer;
  ein Zugriffs-Log existiert nicht (0 Treffer).
- **Vorgehen:** Der Plan sieht diesen Fall ausdruecklich vor („falls das Log das
  nicht hergibt"). Statt nur auf Task 2 zu verweisen, wurde ein aequivalenter und
  validierter Zaehler etabliert (DAO-Join-Query-Zeile pro Handler-Aufruf) und dem
  Checkpoint als zweiter, unabhaengiger Beleg mitgegeben.
- **Dateien geaendert:** keine

**2. [Erweiterung, kein Regelverstoss] Fat-Backend-Grep auf den gesamten FE-Datenpfad ausgeweitet**
- **Gefunden bei:** Task 1, Schritt e
- **Anlass:** Der Plan-Grep prueft nur `page/user_management.rs`. Eine
  Fallback-Regel koennte ebenso im Service- oder API-Layer sitzen.
- **Vorgehen:** Zusaetzlicher Grep ueber `service/`, `state/` und `api.rs` sowie
  `grep -rln 'display_name' shifty-dioxus/src/`. Ergebnis: `display_name` existiert
  im Frontend nur an der vom Plan benannten Stelle. Die Zielwahl des Plans ist
  damit belegt statt angenommen.
- **Dateien geaendert:** keine

Keine Quelldatei wurde veraendert — `files_modified` des Plans ist leer und bleibt
es (T-62-18).

## Self-Check

- `.planning/phases/62-personen-verzeichnis-eine-liste-statt-tabs/62-07-SUMMARY.md` — FOUND
- Keine Quelldateien erstellt oder geaendert (planmaessig)

## Self-Check: PASSED (fuer Task 1; Task 2 offen)
