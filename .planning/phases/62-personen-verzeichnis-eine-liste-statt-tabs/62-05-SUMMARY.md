---
phase: 62-personen-verzeichnis-eine-liste-statt-tabs
plan: 05
subsystem: rest
tags: [rest-endpoint, openapi, dependency-injection, dev-proxy, admin-gate, tdd, service-tiers]

# Dependency graph
requires:
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-01 — `PersonDirectoryDaoImpl`, bis hierher ohne Laufzeit-Aufrufer"
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-02 — `PersonEntryTO`/`PersonKindTO` als Wire-Format"
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-03 — `PersonDirectoryService` (Admin-Gate) + `From<&PersonEntry> for PersonEntryTO`"
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-04 — die Frontend-URL `{backend}/person-directory`, gegen die der Praefix zeichengleich sein muss"
provides:
  - "`rest::person_directory::generate_route` / `get_person_directory` / `PersonDirectoryApiDoc` — der Endpoint `GET /person-directory`"
  - "`RestStateDef::PersonDirectoryService` + `person_directory_service()` — der Zugriffspfad des Handlers"
  - "`shifty_bin`-DI: `type PersonDirectoryDao`, `PersonDirectoryServiceDependencies`, `type PersonDirectoryService`, Feld + Konstruktion in `RestStateImpl`"
  - "`[[web.proxy]]`-Eintrag fuer `/person-directory` in `shifty-dioxus/Dioxus.toml`"
  - "`shifty_bin/src/integration_test/person_directory.rs` — zwei HTTP-Tests, beide Gate-Seiten"
affects: [62-06-docs, 62-07-verifikation, 63-personen-anlage]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Route und Dev-Proxy-Eintrag zwingend im selben Commit — der Proxy-Eintrag ist Pflichtinhalt, kein Nachtrag"
    - "Routen-Praefix als benannte Konstante im Integrationstest, damit die Zeichengleichheit an einer Stelle haengt"
    - "Gate nur im Service; der Handler reicht den Kontext durch und laesst `error_handler` auf 403 abbilden"

key-files:
  created:
    - rest/src/person_directory.rs
    - shifty_bin/src/integration_test/person_directory.rs
  modified:
    - rest/src/lib.rs
    - shifty_bin/src/main.rs
    - shifty_bin/src/integration_test.rs
    - shifty-dioxus/Dioxus.toml

key-decisions:
  - "Der Handler enthaelt keine eigene Rechtepruefung (T-62-13). Die Trust Boundary liegt an `PersonDirectoryService::list`; eine zweite Gate-Stelle waere eine zweite Wahrheit, und die schwaechere davon waere die Luecke."
  - "Der Praefix `/person-directory` liegt im Integrationstest als Konstante `DIRECTORY_PATH` vor und wird sowohl fuer `.nest(...)` als auch fuer die Request-URI benutzt — ein Tippfehler faellt damit in beiden Tests um."
  - "Der Business-Logic-Service wird nach `voluntary_stats_service` konstruiert, also klar im BL-Abschnitt und damit strikt nach `person_service` (Zeile 1371 vs. 1278). Kein `OnceLock`, keine Forward-Declaration."
  - "`admin_bekommt_200` prueft zusaetzlich die Gesamtzahl 4 statt nur 3: `TestSetup::new()` legt `DEVUSER` an, und seit D-61-01 vergibt `create_user` sofort eine `person`-Identitaet — `DEVUSER` ist damit selbst eine legitime `UserOnly`-Zeile. Die Zahl mitzupruefen schliesst aus, dass der Endpoint Zeilen unterschlaegt oder dupliziert."
  - "Die Tier-Doku (`docs/architecture/02-service-tiers*.md`, Runtime-Graph) bleibt bewusst bei Plan 62-06, der sie als Task 2 vollstaendig und mit Baseline-Kriterien fuehrt und dafuer ausdruecklich den Stand *nach* 62-05 liest."

patterns-established:
  - "Route-Praefix-Vertrag ueber vier Stellen (Router-Nest, OpenAPI-Nest, Frontend-URL, Dev-Proxy) wird per `grep -rho … | sort -u | wc -l == 1` als *eine* Schreibweise nachgewiesen statt per vier Einzel-Greps."
  - "Negativtest prueft nicht nur den Statuscode, sondern auch, dass der Antwort-Body keinen der geseedeten Namen traegt — damit ist ein 200-mit-Redaktion kein stiller Bestehensfall."

# PADM-01 / PADM-08 bleiben Pending: alle sieben Plaene dieser Phase fuehren sie
# im Frontmatter, und ihre Aussagen sind nutzersichtbar. Sie gehoeren an den
# Phasen-Abschluss nach 62-07, nicht an diesen Backend-Plan (Praezedenz
# 62-01/62-03/62-04).
requirements-completed: []
requirements-advanced: [PADM-01, PADM-08]

coverage:
  - id: D1
    description: "Ein Admin bekommt vom Aggregat-Endpoint 200 mit allen drei Auspraegungen, jede mit nicht-leerem Anzeigenamen"
    requirement: "PADM-01"
    verification:
      - kind: integration
        ref: "shifty_bin/src/integration_test/person_directory.rs#admin_bekommt_200"
        status: pass
    human_judgment: false
  - id: D2
    description: "Ein HR-Nutzer ohne Admin-Rolle bekommt 403 und keine Daten — kein 200 mit leerer Liste, keine Redaktion"
    requirement: "PADM-08"
    verification:
      - kind: integration
        ref: "shifty_bin/src/integration_test/person_directory.rs#hr_ohne_admin_bekommt_403"
        status: pass
      - kind: other
        ref: "grep -c 'check_permission' rest/src/person_directory.rs == 0 — keine zweite Gate-Implementierung"
        status: pass
    human_judgment: false
  - id: D3
    description: "Der Endpoint ist in der OpenAPI-Doku registriert (CLAUDE.md §REST API)"
    verification:
      - kind: other
        ref: "grep -c 'utoipa::path' rest/src/person_directory.rs == 1; grep -c 'pub struct PersonDirectoryApiDoc' == 1; grep -c 'PersonDirectoryApiDoc' rest/src/lib.rs == 1"
        status: pass
    human_judgment: false
  - id: D4
    description: "Der Routen-Praefix steht im selben Commit im Dev-Proxy; alle vier Stellen tragen genau eine Schreibweise"
    verification:
      - kind: other
        ref: "git show --name-only --format= e0d8e8d3 listet rest/src/lib.rs UND shifty-dioxus/Dioxus.toml"
        status: pass
      - kind: other
        ref: "grep -rho 'person-directory' rest/src/lib.rs shifty-dioxus/Dioxus.toml shifty-dioxus/src/api.rs | sort -u | wc -l == 1"
        status: pass
    human_judgment: false
  - id: D5
    description: "Der Business-Logic-Service wird nach dem Basic-Block konstruiert, ohne Forward-Declaration-Tricks"
    verification:
      - kind: other
        ref: "grep -n 'let person_directory_service' == 1371 > grep -n 'let person_service' == 1278; kein nicht-kommentiertes 'OnceLock' in shifty_bin/src/main.rs"
        status: pass
    human_judgment: false
  - id: D6
    description: "Die Produktionsvariante (oidc) und die gesamte Workspace-Suite bleiben gruen; Clippy als harte Schranke"
    verification:
      - kind: integration
        ref: "cargo test --workspace / cargo clippy --workspace -- -D warnings / cargo build --no-default-features --features oidc,json_logging"
        status: pass
    human_judgment: false

# Metrics
duration: 7min
completed: 2026-09-20
status: complete
---

# Phase 62 Plan 05: Aggregat-Endpoint `GET /person-directory` Summary

**Der Admin-gegatete Verzeichnis-Endpoint ist verdrahtet, in der OpenAPI-Doku
registriert und — im selben Commit wie die Route — im Dev-Proxy eingetragen;
beide Seiten des Gates sind als HTTP-Test bewiesen.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-09-20T08:18:51Z
- **Completed:** 2026-09-20T08:26:09Z
- **Tasks:** 2/2
- **Files modified:** 6 (2 neu, 4 geaendert)

## Accomplishments

- **`rest/src/person_directory.rs`** mit `generate_route`, dem Handler
  `get_person_directory` und `PersonDirectoryApiDoc`. Der Handler ruft
  `person_directory_service().list(context.into(), None)`, mappt feldweise auf
  `Arc<[PersonEntryTO]>` und antwortet mit 200 — die Form von
  `rest/src/permission.rs::get_all_users`. **Keine eigene Rechtepruefung.**
- **`rest/src/lib.rs`**: `pub mod person_directory;` (mit der begruendenden
  Ausnahme-Notiz wie bei `pub mod report;`), der assoziierte Typ
  `PersonDirectoryService` samt Accessor direkt nach dem `PersonService`-Block,
  der `ApiDoc`-Nest-Eintrag und der Router-Nest.
- **DI in `shifty_bin/src/main.rs`** vollstaendig: DAO-Typalias,
  `PersonDirectoryServiceDependencies`, `type PersonDirectoryService`, das
  `RestStateImpl`-Feld, der Accessor, die DAO-Konstruktion bei den uebrigen
  DAOs und die Service-Konstruktion im Business-Logic-Abschnitt (direkt nach
  `voluntary_stats_service`). Damit hat `PersonDirectoryDaoImpl` aus 62-01
  erstmals einen Laufzeit-Aufrufer.
- **Dev-Proxy-Eintrag** `http://localhost:3000/person-directory` in
  `shifty-dioxus/Dioxus.toml` — im selben Commit `e0d8e8d3` wie
  `rest/src/lib.rs`. Das ist der Fehler, den dieses Repo dreimal geshipt hat
  (v1.6 `/toggle`, v1.8 `/vacation-entitlement-offset`, v1.10 `/special-days`).
- **Zwei HTTP-Integrationstests** ueber `tower::ServiceExt::oneshot` gegen den
  echten `RestStateImpl`: Admin → 200 mit allen drei Auspraegungen, HR-only →
  403 ohne jede Namenspreisgabe.

## Task Commits

| Task | Name | Commit |
| --- | --- | --- |
| 1 (RED) | HTTP-Integrationstests fuer beide Gate-Seiten | `54af1a57` |
| 2 (GREEN) | REST-Modul, OpenAPI, DI in `main.rs` und Dev-Proxy — ein Commit | `e0d8e8d3` |

## TDD Gate Compliance

Die Gate-Reihenfolge ist in der Commit-Historie sichtbar:

1. **RED** — `54af1a57` `test(62-05): add failing HTTP tests for both gate sides
   of the person directory`. Der RED-Nachweis ist ein Compile-Fehler, der genau
   auf das noch fehlende REST-Modul zeigt:

   ```
   error[E0432]: unresolved import `rest::person_directory`
     --> shifty_bin/src/integration_test/person_directory.rs:38:11
      |
   38 | use rest::person_directory::generate_route;
      |           ^^^^^^^^^^^^^^^^ could not find `person_directory` in `rest`
   ```

2. **GREEN** — `e0d8e8d3` `feat(62-05): add the admin-gated GET
   /person-directory endpoint`, danach `2 passed; 0 failed`.
3. **REFACTOR** — entfiel; die GREEN-Implementierung brauchte keine Nacharbeit.

## Deviations from Plan

### Auto-fixed Issues

Keine. Es war kein Bug, keine fehlende kritische Funktionalitaet und kein
Blocker zu beheben — der Plan lief wie geschrieben durch.

### Acceptance-Criterion-Abweichungen (dokumentiert, nicht gefixt)

**Task 2, Kriterium „`grep -c 'nest(\"/person-directory\"' rest/src/lib.rs`
ergibt `2`"** — ergibt tatsaechlich `1`. Der `ApiDoc`-Nest-Eintrag schreibt sich
im Repo durchgaengig `(path = "/person-directory", api = …)` und nicht
`nest("/person-directory"`; das Muster kommt nur im Router-Aufbau vor. Der Plan
sieht diesen Fall selbst vor und nennt das Ersatzkriterium: mindestens `1` fuer
den Router **und** zusaetzlich `grep -c '"/person-directory", api ='
rest/src/lib.rs` ergibt `1`. Beide sind erfuellt (`1` und `1`).

**Task 2, Kriterium „`git show --name-only HEAD | grep -c 'Dioxus.toml'` ergibt
`1`"** — ergibt `2`. `git show --name-only` druckt auch den Commit-Body, und der
nennt `shifty-dioxus/Dioxus.toml` in der Begruendungszeile. Das Kriterium ist
gegen sich selbst blind. Korrigiertes Kommando, das nur die Dateiliste liest:
`git show --name-only --format= e0d8e8d3` → genau vier Zeilen:

```
rest/src/lib.rs
rest/src/person_directory.rs
shifty-dioxus/Dioxus.toml
shifty_bin/src/main.rs
```

Die **Absicht** — Route und Proxy-Eintrag im selben Commit — ist damit
nachgewiesen und sogar schaerfer belegt als vom Kriterium verlangt.

**Task 2, Kriterium „`grep -c 'OnceLock' shifty_bin/src/main.rs` unveraendert,
erwartet `0`"** — der Ausgangswert vor diesem Plan war `4`, nicht `0`; nach dem
Plan sind es `6`. Alle sechs Treffer sind **Prosa in Kommentaren** („kein
`OnceLock`, keine Forward-Declaration"), zwei davon habe ich selbst als
Tier-Begruendung erganzt. Die tatsaechliche Aussage des Kriteriums ist
nachgewiesen: `grep 'OnceLock' shifty_bin/src/main.rs | grep -v '^\s*//' | wc -l`
ergibt `0` — es gibt keine einzige nicht-kommentierte Verwendung.

**Task 1, Behavior „der Body … enthaelt genau drei Eintraege"** — die Antwort
traegt vier. `TestSetup::new()` ruft `create_admin_user` und legt `DEVUSER` an;
seit D-61-01 vergibt `PermissionDaoImpl::create_user` jedem Benutzer sofort eine
`person`-Identitaet, und `clear_all()` loescht `user`-Zeilen ausdruecklich nicht.
`DEVUSER` ist damit selbst eine korrekte Verzeichniszeile der Auspraegung
`UserOnly` — eine Liste mit genau drei Eintraegen waere ein Fehler des
Endpoints, kein Erfolg. Der Test prueft deshalb die drei geseedeten Personen
einzeln (Identitaet, Auspraegung, Anzeigename) **und** zusaetzlich
`entries.len() == 4` mit einer Meldung, die `DEVUSER` benennt. Die Aussage des
Plans bleibt erhalten und wird schaerfer: sie schliesst jetzt auch
Unterschlagung und Duplikation aus. Der Modulkopf der Testdatei haelt die
Begruendung fest.

### Docs-Freshness-Gate — bewusst bei 62-06 belassen

`CLAUDE.md` bindet eine Aenderung an `shifty_bin/src/main.rs` (DI-Verdrahtung)
an `docs/architecture/02-service-tiers.md` plus
`docs/architecture/diagrams/service-graph-runtime.mmd`. Diese Aktualisierung
ist **Task 2 von Plan 62-06**, der im selben Phasen-PR laeuft, die drei Dateien
in `files_modified` fuehrt und in seinem `read_first` ausdruecklich
„`shifty_bin/src/main.rs` (Stand nach 62-05)" liest — 62-06 haengt also an
genau diesem Plan. Seine Akzeptanzkriterien messen zudem gegen Baselines
(„`grep -ci 'basic' …` ist groesser als vor dem Plan"); eine Vorwegnahme hier
haette diese Baselines entwertet und 62-06 zum No-Op gemacht. Anders als bei
62-03 (wo das Gate eine `service/**/*.rs`-Aenderung betraf und die
F01-Sektionen sonst gar keinen Plan gehabt haetten) ist das Ziel hier
lueckenlos beplant. Die Auslassung ist damit eine terminierte Zuordnung
innerhalb desselben PR, kein Drift.

## Verification Results

Alle Kommandos via `nix develop --command bash -c 'SQLX_OFFLINE=true …'`;
`cargo` liegt nicht im Default-PATH.

| Gate | Kommando | Ergebnis |
| --- | --- | --- |
| Plan-Tests | `cargo test -p shifty_bin --bin shifty_bin integration_test::person_directory -- --nocapture` | 2 passed, 0 failed |
| PADM-08 positiv | `… integration_test::person_directory::admin_bekommt_200` | passed |
| PADM-08 negativ | `… integration_test::person_directory::hr_ohne_admin_bekommt_403` | passed |
| Workspace | `cargo test --workspace` | alle gruen (u.a. 904 + 95), 0 failed |
| Clippy (Hard Gate) | `cargo clippy --workspace -- -D warnings` | ohne Befund |
| OIDC-Produktionsvariante | `cargo build --no-default-features --features oidc,json_logging` | `Finished dev profile` |
| Kein zweites Gate | `grep -c 'check_permission' rest/src/person_directory.rs` | 0 |
| OpenAPI-Pfad | `grep -c 'utoipa::path' rest/src/person_directory.rs` | 1 |
| OpenAPI-Struct | `grep -c 'pub struct PersonDirectoryApiDoc' rest/src/person_directory.rs` | 1 |
| OpenAPI-Registrierung | `grep -c 'PersonDirectoryApiDoc' rest/src/lib.rs` | 1 |
| Router-Nest | `grep -c 'nest("/person-directory"' rest/src/lib.rs` | 1 |
| ApiDoc-Nest | `grep -c '"/person-directory", api =' rest/src/lib.rs` | 1 |
| Dev-Proxy | `grep -c 'localhost:3000/person-directory' shifty-dioxus/Dioxus.toml` | 1 |
| Praefix-Konsistenz (4 Stellen) | `grep -rho 'person-directory' rest/src/lib.rs shifty-dioxus/Dioxus.toml shifty-dioxus/src/api.rs \| sort -u \| wc -l` | 1 |
| Tier-Reihenfolge | `grep -n 'let person_service'` = 1278, `grep -n 'let person_directory_service'` = 1371 | 1371 > 1278 |
| Keine Forward-Decl | `grep 'OnceLock' shifty_bin/src/main.rs \| grep -v '^\s*//' \| wc -l` | 0 |
| Testtafel | `grep -c '#\[tokio::test\]' shifty_bin/src/integration_test/person_directory.rs` | 2 |
| Modul registriert | `grep -c 'mod person_directory;' shifty_bin/src/integration_test.rs` | 1 |
| HR-Rolle ohne Admin | `grep -c 'add_user_role' …` = 1, `grep 'add_user_role' … \| grep -c '"admin"'` = 0 | erfuellt |
| Keine Loeschungen | `git diff --diff-filter=D --name-only HEAD~1 HEAD \| wc -l` | 0 |

## Threat Mitigations

| Threat ID | Disposition | Umsetzung |
| --- | --- | --- |
| T-62-01 | mitigate | `hr_ohne_admin_bekommt_403` fuehrt den HTTP-Beweis, den 62-03 nur auf Service-Ebene fuehren konnte. Der Benutzer traegt ausschliesslich die Rolle `hr`, die laut Migration `20240426150045_user-roles.sql` genau das Privileg `hr` bindet und kein `admin` — ein aus Phase 61 kopiertes HR-Gate faellt damit sofort um. Gegenprobe: `admin_bekommt_200` liefert mit derselben Fixture vier Zeilen. |
| T-62-05 | mitigate | Dasselbe Gate. Der Negativtest prueft zusaetzlich, dass der Antwort-Body **keinen** der vier geseedeten Namen (`Anna Verknuepft`, `anna-linked`, `bertram-useronly`, `Clara Nurverkauf`) enthaelt. Ein 200 mit leerer Liste oder redigierten Feldern waere damit ein Fehlschlag, kein stilles Bestehen. |
| T-62-13 | mitigate | Der Handler prueft bewusst nicht selbst; `grep -c 'check_permission' rest/src/person_directory.rs` ergibt `0`. Der Modulkopf haelt die Begruendung fest (zwei Gate-Stellen sind zwei Wahrheiten). |
| T-62-14 | mitigate | Der `[[web.proxy]]`-Eintrag liegt in Commit `e0d8e8d3` — demselben, der die Route anlegt. Die Praefix-Konsistenz ueber alle vier Stellen ist per `sort -u \| wc -l == 1` belegt. Der Laufzeit-Smoke gegen `dx serve` ist Inhalt von 62-06. |
| T-62-SC | accept | Keine neue Abhaengigkeit. `Cargo.toml` ist in keinem der beiden Commits enthalten. |

## Known Stubs

Keine. Der Endpoint liefert echte Daten aus dem echten Service und dem echten
DAO; `PersonDirectoryDaoImpl` hat mit diesem Plan seinen Laufzeit-Aufrufer
bekommen. Der vollstaendige Pfad Browser → Dev-Proxy → Endpoint → Service → DAO
ist damit geschlossen; sein Nachweis im laufenden `dx serve` ist planmaessig
Gegenstand von 62-06.

## Threat Flags

Der Plan legt genau eine neue HTTP-Oberflaeche an (`GET /person-directory`).
Sie steht im `<threat_model>` des Plans und ist oben als T-62-01 / T-62-05 /
T-62-13 abgehandelt. Keine weitere, ungeplante sicherheitsrelevante Oberflaeche:
keine Schreibpfade, keine Schema-Aenderung, kein neuer Dateizugriff.

## Notes for Future Phases

- **62-06 findet den Stand vor, den sein `read_first` erwartet.** Die
  Konstruktion steht in `shifty_bin/src/main.rs` ab Zeile 1371, im
  Business-Logic-Abschnitt direkt nach `voluntary_stats_service`; die
  Dependency-Liste im `gen_service_impl!`-Block von
  `service_impl/src/person_directory.rs` ist unveraendert dreistellig
  (`PersonDirectoryDao`, `PermissionService`, `TransactionDao`). Fuer den
  Laufzeit-Graphen gilt: der Knoten hat **keine** Domain-Service-Kante.
- **Der Praefix ist jetzt ein Vier-Stellen-Vertrag.** Wer ihn aendert, muss
  `rest/src/lib.rs` (zweimal), `shifty-dioxus/src/api.rs` und
  `shifty-dioxus/Dioxus.toml` gemeinsam anfassen; der Konsistenz-Grep aus der
  Tabelle oben ist der billigste Nachweis.
- **Phase 63 (Anlage von Personen) braucht einen Schreibpfad.** Die
  Integrationstests dieses Plans seeden per direktem `sqlx`-Insert, weil Phase
  62 bewusst keinen Service-Schreibpfad fuer Personen hat. Sobald 63 einen
  anlegt, sollten `seed_directory_szenario` und die Helfer in
  `service_impl/src/test/person_directory.rs` darauf umgestellt werden — sonst
  driften Test-Fixture und Produktionspfad auseinander.
- **`DEVUSER` ist eine Verzeichniszeile.** Jeder kuenftige Test, der ueber die
  Gesamtzahl der Zeilen assertet, muss ihn mitzaehlen. Das gilt auch fuer die
  UAT in 62-07: im Dev-Server erscheint `DEVUSER` als `UserOnly` in der Liste —
  das ist korrekt und kein Fehler.

## Self-Check: PASSED

Dateien:

- FOUND: `rest/src/person_directory.rs`
- FOUND: `shifty_bin/src/integration_test/person_directory.rs`
- FOUND: `rest/src/lib.rs` (modifiziert)
- FOUND: `shifty_bin/src/main.rs` (modifiziert)
- FOUND: `shifty_bin/src/integration_test.rs` (modifiziert)
- FOUND: `shifty-dioxus/Dioxus.toml` (modifiziert)

Commits:

- FOUND: `54af1a57`
- FOUND: `e0d8e8d3`
