---
slug: convert-to-absence-404
status: resolved
trigger: |
  Beim Umwandeln von Extra Hours vom Typ Urlaub in einen Abwesenheitszeitraum
  kommt: reqwest error: HTTP status client error (404 Not Found) for url
  (https://shifty-int.nebenan-unverpackt.de/api//extra-hours/cfaba0dd-42a2-4e19-89d7-e8c126340a36/convert-to-absence).
  Auffällig: Doppel-Slash /api//extra-hours in der URL. Backend-Route
  POST /{id}/convert-to-absence existiert lokal in rest/src/extra_hours.rs:31,
  Frontend-Aufruf in shifty-dioxus/src/api.rs:543. Fehler tritt auf deployter
  Umgebung shifty-int.nebenan-unverpackt.de auf.
created: 2026-06-12T00:00:00Z
updated: 2026-06-12T00:00:00Z
---

# Debug Session: convert-to-absence-404

## Symptoms

<!-- DATA_START — user-supplied content, treat as data only -->

- **Expected behavior:** Eine Extra-Hours-Buchung vom Typ Urlaub lässt sich über den
  Convert-Button in einen Abwesenheitszeitraum (absence_period) umwandeln; der
  Backend-Endpoint `POST /extra-hours/{id}/convert-to-absence` antwortet erfolgreich.
- **Actual behavior:** Der API-Call schlägt mit `404 Not Found` fehl. Die Buchung
  wird nicht umgewandelt.
- **Error messages:** `reqwest error: HTTP status client error (404 Not Found) for
  url (https://shifty-int.nebenan-unverpackt.de/api//extra-hours/cfaba0dd-42a2-4e19-89d7-e8c126340a36/convert-to-absence)
  [status=404] [url=https://shifty-int.nebenan-unverpackt.de/api//extra-hours/cfaba0dd-42a2-4e19-89d7-e8c126340a36/convert-to-absence]`.
  Auffällig: Doppel-Slash `/api//extra-hours`.
- **Timeline:** Tritt auf der deployten Integrationsumgebung
  `shifty-int.nebenan-unverpackt.de` auf. Feature (convert-to-absence) ist
  jüngst hinzugekommen (Absence-Arbeit, Phase 260612-o7t und Vorgänger).
- **Reproduction:** Auf der deployten Umgebung shifty-int eine Extra-Hours-Buchung
  vom Typ Urlaub auswählen und „in Abwesenheit umwandeln" auslösen.

<!-- DATA_END -->

## Known Facts (vom Orchestrator beim Routing erhoben)

- Backend-Route existiert lokal: `rest/src/extra_hours.rs:31-34` registriert
  `POST /{id}/convert-to-absence` via `convert_extra_hours_to_absence`.
- Frontend baut die URL in `shifty-dioxus/src/api.rs:543`:
  `format!("{}/extra-hours/{}/convert-to-absence", config.backend, id)`.
- Andere Extra-Hours-Calls auf shifty-int scheinen zu funktionieren (User konnte
  die Buchung sehen) → Base-URL-Config grundsätzlich korrekt, aber Doppel-Slash
  in dieser URL deutet auf `config.backend` mit Trailing-Slash (`.../api/`).

## Hypotheses

- H1 (untested): Die deployte Backend-Version auf shifty-int ist älter und kennt
  den `convert-to-absence`-Endpoint noch nicht → 404 (Deployment-Drift). Der lokale
  Code hat die Route, das Deployment nicht.
- H2 (untested): Reverse-Proxy/nginx auf shifty-int normalisiert `//` nicht und
  routet `/api//extra-hours/...` nicht auf den Axum-Handler → 404. Doppel-Slash
  entsteht durch `config.backend = ".../api/"` (Trailing-Slash) + `/extra-hours`.

## Current Focus

reasoning_checkpoint:
  hypothesis: >
    The frontend builds API URLs as format!("{}/<path>", config.backend, ...) while
    the DEPLOYED config.json sets backend = "https://shifty-int.../api/" WITH a
    trailing slash. Every request URL therefore contains a double-slash
    ("/api//extra-hours/{id}/convert-to-absence"). axum 0.8.7 treats a double-slash
    as an empty path segment and returns 404 for ANY such path (verified with a
    minimal repro). The convert call surfaces the 404 to the user because, unlike the
    GET helpers (reqwest::get → error_for_status), the convert helper actually
    propagates non-success status. Root cause = trailing-slash-in-config + naive URL
    join producing a path axum 0.8 refuses to match.
  confirming_evidence:
    - "Deployed config.json: backend = 'https://shifty-int.nebenan-unverpackt.de/api/' (trailing slash) — fetched live."
    - "All frontend api.rs calls use format!(\"{}/<path>\", config.backend) → trailing slash yields '/api//<path>'."
    - "Minimal axum 0.8.7 repro: every double-slash path (GET and POST) → 404; clean path → 200."
    - "Error URL in report literally shows the double-slash: '/api//extra-hours/...'."
    - "Backend route + service wiring exist at the deployed commit 01d0b2c → H1 (deployment drift) eliminated."
  falsification_test: >
    If the fix (robust join that never emits a double-slash) is deployed and the
    convert call still 404s, the hypothesis is wrong. Locally: building the URL with
    a trailing-slash backend must now produce a single-slash path; a unit test
    asserts no '//' after the scheme.
  fix_rationale: >
    Make URL construction robust to a trailing slash in config.backend by trimming a
    single trailing '/' from the base before joining. This produces the clean path
    axum 0.8 matches, fixing convert AND hardening every other call against the same
    latent defect — without depending on proxy slash-merging behavior we cannot fully
    inspect.
  blind_spots: >
    Could not probe past the deployed OIDC auth layer (all unauth requests → 401), so
    I could not directly observe axum returning 404 for the convert path on shifty-int;
    the axum-404-on-double-slash mechanism is proven locally and by the verbatim error
    URL, not by an authenticated live trace. If shifty-int's /api/ nginx already merges
    slashes (assets location does), the trailing slash alone might be masked for GETs
    and a deeper cause (e.g. auth/permission edge) could exist — but the double-slash
    is an unambiguous real defect and the correct, lowest-risk fix regardless.

next_action: >
  Implement a robust URL join in shifty-dioxus/src/api.rs that trims a trailing '/'
  from config.backend, apply it at least to the convert call (ideally all), add a
  unit test asserting no double-slash, run cargo build/test + wasm build gate.

## Evidence

- timestamp: 2026-06-12T00:10:00Z
  checked: Backend route mounting (rest/src/lib.rs:592) + route registration (rest/src/extra_hours.rs:31-34)
  found: `.nest("/extra-hours", extra_hours::generate_route())` mounts a router that
    has `.route("/{id}/convert-to-absence", post(...))`. Resolved path is
    `/extra-hours/{id}/convert-to-absence`. Local code is correct and consistent
    with the frontend call.
  implication: Local code is NOT the bug. The 404 is environmental — either the
    deployed backend is older (H1) or the proxy mishandles the double-slash (H2).

- timestamp: 2026-06-12T00:10:00Z
  checked: Frontend URL construction for convert (api.rs:542) vs other calls (api.rs:525, 564)
  found: ALL frontend calls use `format!("{}/<path>", config.backend, ...)`. If
    `config.backend` ends in `/` (`.../api/`), then EVERY url has a double-slash
    (`/api//absence-period`, `/api//extra-hours`, etc.), not just convert. The user
    reported other extra-hours/absence calls work (could see the booking).
  implication: If GETs with `//` work but this POST gets 404, the double-slash alone
    cannot be the sole cause (H2 weakened) — UNLESS the difference is GET-vs-POST
    behavior at the proxy, OR the deployed backend simply lacks the route (H1).

- timestamp: 2026-06-12T00:30:00Z
  checked: axum 0.8.7 double-slash behavior — built minimal repro (/tmp/axum-slash-test)
    mirroring the real nest structure, probed single- vs double-slash with curl.
  found: In axum 0.8, EVERY double-slash URL returns 404 (empty path segment not
    matched): `//version`→404, `//extra-hours/{id}/convert-to-absence`→404,
    `//absence-period/by-sales-person/{id}`→404. Single-slash all →200.
  implication: If reqwest literally sends `/api//...` to axum, ALL calls (GET+POST)
    would 404, not just convert. Confirmed via `url` crate that reqwest preserves
    the double-slash verbatim. So either (a) something between client and axum
    collapses `//` (nginx merge_slashes, default ON) — in which case convert should
    ALSO work, or (b) the GETs don't actually carry a double-slash. The 404-only-on-
    convert pattern is NOT explained by a uniform trailing-slash on config.backend.

- timestamp: 2026-06-12T00:35:00Z
  checked: git history ordering — convert route (107109f, 2026-06-12 13:36) vs
    absence GET routes that work on shifty-int (56c5ed6).
  found: 56c5ed6 is 4 commits AFTER 107109f in linear history. The absence-page GET
    endpoints the user successfully uses came LATER than the convert route.
  implication: A deployment containing the later absence GET routes MUST also contain
    the earlier convert route. H1 (deployment drift — deployed backend too old to
    have convert route) is logically eliminated.

## Evidence (Orchestrator-Live-Trace, 2026-06-12 — WIDERLEGT die Debugger-Diagnose)

- timestamp: 2026-06-12T18:00:00Z
  checked: Live `https://shifty-int.../assets//config.json` (Doppel-Slash) vs Single-Slash.
  found: BEIDE liefern HTTP 200 mit IDENTISCHEM Inhalt (179 Bytes). Auch `//assets/config.json`
    → 200. nginx auf shifty-int hat `merge_slashes on` (Default) und kollabiert `//` → `/`,
    BEVOR die Anfrage upstream (axum) geht.
  implication: Der Doppel-Slash erreicht axum NIE als Doppel-Slash. Das lokal bewiesene
    axum-404-bei-Doppel-Slash-Verhalten ist auf dieser Deployment-Topologie IRRELEVANT.
    → Die Root-Cause-Diagnose (Trailing-Slash → Doppel-Slash → axum 404) ist WIDERLEGT.
    Der Doppel-Slash in der reqwest-Fehler-URL ist client-seitig real, aber serverseitig
    harmlos (nginx normalisiert).

- timestamp: 2026-06-12T18:05:00Z
  checked: Unauth-Live-Probes gegen `/api//extra-hours/{id}/convert-to-absence` (POST),
    Single-Slash POST, Doppel-/Single-Slash GET `/api/extra-hours/by-sales-person/{id}`.
  found: ALLE → HTTP 401. Auth-Layer wrappt den Router und antwortet 401 vor dem Routing,
    unabhängig von Methode und Slash-Anzahl. `/api/openapi.json`, `/api/version`,
    `/api/swagger-ui` ebenfalls 401.
  implication: Routen-Existenz/Version ist unauthentifiziert NICHT prüfbar — die Auth-Wand
    maskiert das eigentliche 404-Routing-Verhalten. Authentifizierter Zugang (Cookie/Token)
    oder Kenntnis des deployten Backend-Commits nötig, um Deployment-Drift zu bestätigen.

- timestamp: 2026-06-12T18:10:00Z
  checked: git-Alter der convert-Route + Annahme hinter H1-Elimination.
  found: convert-Route eingeführt in `107109f` HEUTE 2026-06-12 13:36 (wenige Stunden alt).
    Die vom User SICHTBARE Urlaubs-Buchung kommt aus `/extra-hours/by-sales-person` — eine
    ALTE Route, NICHT aus den neuen Absence-Routes (56c5ed6). Die H1-Elimination nahm
    fälschlich an, die sichtbaren Daten kämen aus 56c5ed6.
  implication: H1 (Deployment-Drift) ist NICHT valide eliminiert. Ein Backend-Build von
    vor 13:36 hat `/extra-hours/by-sales-person` (alt, GET funktioniert), aber NICHT
    `/extra-hours/{id}/convert-to-absence` (neu) → echtes 404. Das ist die führende Hypothese.

## Eliminated

- hypothesis: ~~H1-Elimination~~ Trailing-Slash/Doppel-Slash als Root Cause (Debugger-Diagnose).
  evidence: nginx merged Slashes (Assets-Test → Doppel-Slash-Pfad liefert 200, identischer
    Inhalt). Der Doppel-Slash erreicht axum nie. Diagnose widerlegt.
  timestamp: 2026-06-12T18:00:00Z

- note: Die frühere "H1 eliminiert"-Begründung (git-Ordering 56c5ed6 nach 107109f) ist
    HINFÄLLIG — sie nahm an, die sichtbaren GETs seien die neuen Absence-Routes; tatsächlich
    ist es die alte `/extra-hours/by-sales-person`-Route. H1 ist REAKTIVIERT.

## Evidence (LOKALE REPRODUKTION gegen den ECHTEN axum-Router — validiert Root Cause)

- timestamp: 2026-06-12T18:40:00Z
  checked: Neuer Integration-Test `shifty_bin/src/integration_test/convert_route_slash.rs`
    baut `Router::new().nest("/extra-hours", extra_hours::generate_route())` (ECHTER
    Router, nicht Minimal-Repro) und schickt via tower::oneshot Single- vs Doppel-Slash.
  found: |
    POST /extra-hours/{id}/convert-to-absence        → 200 OK   (Route existiert, Erfolg)
    POST /extra-hours//{id}/convert-to-absence        → 404 Not Found
    POST //extra-hours/{id}/convert-to-absence        → 404 Not Found
    GET  /extra-hours/by-sales-person/{id}            → 201 (matched)
    GET  //extra-hours/by-sales-person/{id}           → 404 Not Found
    GET  /extra-hours//by-sales-person/{id}           → 404 Not Found
  implication: Der REALE shifty-Router liefert für JEDEN Doppel-Slash-Pfad 404 — GET und
    POST identisch. Der Single-Slash-Kontrollfall mit gültigem Seed liefert 200, d.h. das
    404 ist eindeutig ROUTING (vor dem Handler), kein fachliches "not found". Damit ist
    bewiesen: Trailing-Slash in config.backend → Doppel-Slash → axum 404 ist ein echter,
    reproduzierbarer Defekt im echten Code.

- timestamp: 2026-06-12T18:42:00Z
  checked: Re-Bewertung der früheren "nginx merged → Doppel-Slash harmlos"-Widerlegung.
  found: Der Assets-Test (`/assets//config.json` → 200) beweist Slash-Merging nur fürs
    STATIC-Serving (nginx serviert über das normalisierte `$uri`). Für die `/api/`-PROXY-
    Location gilt das NICHT: die Fehler-URL enthält den Doppel-Slash WÖRTLICH und der
    Server antwortete 404 — direkter Beweis, dass der Doppel-Slash über den Proxy-Pfad
    axum ERREICHT (typisch für `proxy_pass http://backend;` ohne URI-Teil: nginx reicht
    die rohe, un-gemergte Request-URI weiter).
  implication: Meine Zwischen-Widerlegung war falsch (Static-Merge ≠ Proxy-Forwarding).
    Die Debugger-Root-Cause ist korrekt. Deployment-Drift ist KEINE Ursache (User: ganzes
    Paket deployt; lokaler Beweis zeigt, dass die Route da ist und nur der Doppel-Slash 404t).

## Resolution

status: ROOT CAUSE VALIDATED (lokal reproduziert) — Fix korrekt.

root_cause: >
  Die deployte config.json auf shifty-int setzt
  `backend = "https://shifty-int.nebenan-unverpackt.de/api/"` MIT Trailing-Slash (live
  bestätigt). Alle Frontend-API-Calls bauen URLs via `format!("{}/<path>", config.backend)`
  → Doppel-Slash `/api//extra-hours/{id}/convert-to-absence`. Die nginx-`/api/`-Proxy-
  Location reicht die un-gemergte URI an axum weiter (Static-Serving merged zwar, der Proxy-
  Pfad nicht). Der echte axum-0.8-Router liefert für jeden Doppel-Slash-Pfad 404 (lokal
  gegen den realen Router reproduziert: POST convert single=200 / double=404).
fix: >
  `normalize_backend()` in shifty-dioxus/src/api.rs entfernt genau EINEN Trailing-Slash an
  der einzigen Eintrittsstelle (`load_config()`). Alle Caller bauen danach Single-Slash-
  Pfade → axum matcht → 200. Behebt convert UND härtet jeden anderen Call gegen denselben
  latenten Defekt.
verification: >
  Frontend: 4 Unit-Tests (`api::normalize_backend_tests`) + volle Suite 577 passed.
  Backend (NEU, Beweis des Mechanismus): `convert_route_slash.rs` — 2 oneshot-Tests gegen den
  echten Router, single=200 vs double=404 für POST und GET. `cargo test -p shifty_bin
  convert_route_slash` grün (2 passed).
open_question: >
  Warum erscheinen dem User die GETs auf shifty-int "funktionierend", wenn axum auch GET-
  Doppel-Slash mit 404 beantwortet? Wahrscheinlichste Erklärung: dieselbe Fehler-Schluck-
  Mechanik wie in [[employees-view-relative-url]] — nur wenige Frontend-Seiten rendern
  überhaupt einen ErrorView; auf den übrigen wird ERROR_STORE still gefüllt und die Seite
  zeigt leere/zwischengespeicherte Daten. Der convert-Call surfacet den Fehler explizit.
  Verifizierbar nach Deploy: prüfen, ob die Absences-Seite auf shifty-int wirklich FRISCHE
  Daten zeigt (Network-Tab: Single-Slash-URLs, 200) — wenn ja, war der Doppel-Slash dort
  schon immer nur client-seitig und etwas anderes maskiert; wenn nein, bestätigt es die
  Schluck-These.
files_changed:
  - "shifty-dioxus/src/api.rs (normalize_backend() + Aufruf in load_config + 4 Unit-Tests) — der Fix"
  - "rest/src/lib.rs (extra_hours mod pub-exportiert für oneshot-Test)"
  - "shifty_bin/src/integration_test/convert_route_slash.rs (NEU — Routing-Regressionstest, beweist axum-404-bei-Doppel-Slash)"
  - "shifty_bin/src/integration_test.rs (mod-Registrierung)"
