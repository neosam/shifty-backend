---
phase: 62-personen-verzeichnis-eine-liste-statt-tabs
plan: 02
subsystem: api
tags: [rest-types, dto, serde, utoipa, openapi, i18n, dioxus, person-directory]

# Dependency graph
requires:
  - phase: 61-personservice-aufl-sung-von-sales-person-user
    provides: "`person` als Traeger-Identitaet (D-61-01/02) — `person_id` ist die Zeilen-Identitaet des Verzeichnisses"
provides:
  - "`rest_types::PersonEntryTO` — flaches Neun-Feld-Listen-DTO des Personen-Verzeichnisses"
  - "`rest_types::PersonKindTO` — explizites Auspraegungs-Enum mit snake_case-Drahtform (D-62-01)"
  - "Sechs i18n-Keys (`ColumnUsername`, `PersonKindLinked`, `PersonKindSalesPersonOnly`, `PersonKindUserOnly`, `PersonDirectorySearchEmptyHeading`, `PersonDirectorySearchEmptyBody`) in de/en/cs"
affects: [62-03 Business-Logic-Service (From-Impl), 62-04 Frontend-Umbau (Konsum von TO + Keys), 62-05 REST + OpenAPI]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Zweckgebautes Listen-DTO statt Wiederverwendung von `UserTO`/`SalesPersonTO` (Praezedenz `SalesPersonAbsenceTO`)"
    - "Auspraegung als explizites Enum-Feld statt clientseitiger Ableitung aus leeren Options"
    - "Wire-Form-Test (`snake_case` + Roundtrip) direkt neben dem DTO, ohne `service-impl`-Feature"

key-files:
  created: []
  modified:
    - rest-types/src/lib.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs

key-decisions:
  - "Die beiden neuen Wire-Tests liegen in `mod test_person_directory_to_wire` — die Plan-Akzeptanzbefehle mit `-- --exact` ohne Modulpfad filtern daher 0 Tests; verifiziert wurde mit vollem Pfad `test_person_directory_to_wire::<test>`."
  - "`PersonKindTO` ist `Copy` — das Enum ist ein Unit-Enum und wird im Frontend pro Zeile weitergereicht; `Copy` vermeidet unnoetige Clones im rsx!-Pfad."
  - "Die sechs i18n-Keys sind als eigener kommentierter Block (`// Personen-Verzeichnis (Phase 62)`) unter den bestehenden User-Management-Keys abgelegt, nicht dazwischen gemischt — der Phasenbezug bleibt lesbar."

patterns-established:
  - "Wire-Kontrakt vor Domain-Typ: DTO landet ohne `From`-Impl in Welle 1, damit Frontend- und Business-Logic-Plan parallel starten koennen; das `From` folgt in 62-03 in derselben Datei."
  - "Neue i18n-Keys werden im selben Commit in den Paritaets-Test aufgenommen — eine vergessene Sprache faellt mechanisch auf."

requirements-completed: [PADM-01, PADM-06]

coverage:
  - id: D1
    description: "`PersonEntryTO` traegt die Auspraegung als explizites `kind`-Enum und einen nicht-optionalen `display_name` (D-62-01, D-62-03)"
    requirement: "PADM-06"
    verification:
      - kind: unit
        ref: "rest-types/src/lib.rs#test_person_directory_to_wire::person_entry_to_roundtrip"
        status: pass
      - kind: other
        ref: "grep -c 'pub display_name: Arc<str>,' rest-types/src/lib.rs == 1"
        status: pass
    human_judgment: false
  - id: D2
    description: "Drahtform des Auspraegungs-Enums ist snake_case (`linked` / `sales_person_only` / `user_only`)"
    requirement: "PADM-01"
    verification:
      - kind: unit
        ref: "rest-types/src/lib.rs#test_person_directory_to_wire::person_kind_to_serialisiert_snake_case"
        status: pass
    human_judgment: false
  - id: D3
    description: "Beide Build-Konfigurationen von `rest-types` uebersetzen (mit und ohne `service-impl`)"
    verification:
      - kind: integration
        ref: "nix develop --command bash -c 'SQLX_OFFLINE=true cargo build --workspace'"
        status: pass
      - kind: integration
        ref: "nix develop --command bash -c 'cd shifty-dioxus && cargo build'"
        status: pass
    human_judgment: false
  - id: D4
    description: "Sechs neue Beschriftungs-Keys existieren in de, en und cs und sind vom Paritaets-Test abgedeckt"
    requirement: "PADM-01"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/i18n/mod.rs#i18n::tests::i18n_user_management_keys_present_in_all_locales"
        status: pass
    human_judgment: false
  - id: D5
    description: "Die tatsaechlich gerenderten Texte (Badge-Beschriftungen, Leer-Zustand) lesen sich in allen drei Sprachen natuerlich und passen in die Spaltenbreite"
    verification: []
    human_judgment: true
    rationale: "Sprachliche Qualitaet und Layout-Passung sind erst im gerenderten UI beurteilbar; die Keys werden erst in 62-04 gerendert."

# Metrics
duration: 13min
completed: 2026-09-20
status: complete
---

# Phase 62 Plan 02: DTO + i18n Summary

**`PersonEntryTO` als zweckgebautes Neun-Feld-Listen-DTO mit explizitem `PersonKindTO`-Auspraegungs-Enum (snake_case-Drahtform) plus sechs neue Personen-Verzeichnis-Beschriftungen in de/en/cs.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-09-20T07:26:17Z
- **Completed:** 2026-09-20T07:39:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `PersonKindTO` als Unit-Enum (`Linked` / `SalesPersonOnly` / `UserOnly`) mit
  `#[serde(rename_all = "snake_case")]` — die Auspraegung ist damit ein
  Backend-Fakt und keine Frontend-Heuristik ueber leere `Option`s (D-62-01,
  T-62-09).
- `PersonEntryTO` mit genau neun Feldern: `person_id`, `kind`, `display_name`
  (nicht optional), `username`, `sales_person_id`, `background_color`,
  `is_paid`, `inactive`, `roles`. Keine internen Audit-/Versionsfelder und keine
  Loesch-Zeitstempel — die Feldliste ist die Mitigation zu T-62-03.
- Feld-Doc-Kommentare halten die Invarianten fest (wann `username` leer ist,
  wann die Verkaeufer-Felder gefuellt sind, dass `display_name` im Backend
  abgeleitet wird).
- Zwei Wire-Tests fixieren die Drahtform des Enums und den verlustfreien
  Serde-Roundtrip des Structs — beide ohne `service-impl`-Feature.
- Sechs i18n-Keys in allen drei Sprachdateien, alle im bestehenden
  Paritaets-Test `i18n_user_management_keys_present_in_all_locales` verankert.

## Task Commits

Each task was committed atomically:

1. **Task 1: PersonEntryTO und PersonKindTO in rest-types** - `48d0aed9` (feat)
2. **Task 2: Sechs neue i18n-Keys in de, en und cs** - `f60255e1` (feat)

## Files Created/Modified

- `rest-types/src/lib.rs` - `PersonKindTO` + `PersonEntryTO` am Dateiende, dazu
  `mod test_person_directory_to_wire` mit zwei Wire-Tests (+130 Zeilen).
- `shifty-dioxus/src/i18n/mod.rs` - sechs `Key`-Varianten mit Doc-Kommentar zum
  Verwendungsort; die sechs Keys zusaetzlich im Paritaets-Test.
- `shifty-dioxus/src/i18n/de.rs` / `en.rs` / `cs.rs` - je sechs `add_text`-Eintraege
  als eigener `// Personen-Verzeichnis (Phase 62)`-Block.

## Decisions Made

- **Kein `From`-Impl in diesem Plan.** Der Domain-Typ
  `service::person_directory::PersonEntry` existiert noch nicht; Plan 62-03
  ergaenzt `#[cfg(feature = "service-impl")] impl From<&PersonEntry> for PersonEntryTO`
  in derselben Datei. Genau dieser Zuschnitt erlaubt es, dass 62-03 und 62-04
  parallel laufen.
- **`PersonKindTO` ist `Copy`.** Ein Unit-Enum ohne Nutzlast; das Frontend reicht
  den Wert pro Zeile in `rsx!` weiter, `Copy` spart dort Clones.
- **Neuer Key `ColumnUsername` statt Wiederverwendung von `ColumnLinkedUser`.**
  Seit D-62-03 zeigt jede Zeile einen Benutzernamen-Wert; „Verknuepfter Benutzer"
  waere fuer eine `UserOnly`-Zeile semantisch falsch (UI-SPEC Copywriting
  Contract).
- **Denial-Meldung des Admin-Guards bleibt hartkodiertes Englisch.** Bewusste,
  in 62-UI-SPEC Open Question 4 dokumentierte Luecke des kopierten
  Guard-Musters; diese Phase schliesst sie nicht.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Akzeptanzbefehl mit `-- --exact` ohne Modulpfad filtert 0 Tests**

- **Found during:** Task 1 (PersonEntryTO und PersonKindTO in rest-types)
- **Issue:** Der Plan schreibt die Tests in einen `#[cfg(test)] mod`-Block vor,
  gibt als Verifikation aber
  `cargo test -p rest-types person_kind_to_serialisiert_snake_case -- --exact`
  an. `--exact` matcht den **vollen** Testnamen inklusive Modulpfad; der Befehl
  laeuft damit zwar gruen durch, fuehrt aber `0 tests` aus („8 filtered out") und
  beweist nichts. Beide Plan-Vorgaben sind so nicht gleichzeitig erfuellbar.
- **Fix:** Verifikation mit vollem Pfad gefahren —
  `cargo test -p rest-types test_person_directory_to_wire::person_kind_to_serialisiert_snake_case -- --exact`
  (1 passed) und analog fuer `person_entry_to_roundtrip` (1 passed). Zusaetzlich
  die volle Crate-Suite: `cargo test -p rest-types` → 8 passed / 0 failed. Die
  Tests selbst wurden **nicht** aus dem Modul herausgezogen, weil der
  Plan-Action-Text den `mod`-Block ausdruecklich vorschreibt und ein
  Top-Level-`#[test]` in `lib.rs` gegen die Dateikonvention verstiesse.
- **Files modified:** keine (reine Verifikations-Korrektur)
- **Verification:** siehe oben — beide Tests einzeln gruen, Crate-Suite gruen.
- **Committed in:** `48d0aed9` (Task-1-Commit, Tests enthalten)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Rein verifikatorisch. Kein Code-Zuschnitt geaendert, kein
Scope-Creep. Fuer Folgephasen: Akzeptanzbefehle mit `-- --exact` muessen den
Modulpfad tragen.

## Issues Encountered

- **`rustfmt` fehlt in der DevShell** (bekannt, im Plan vermerkt). Alle Einfuegungen
  wurden von Hand rustfmt-konform formatiert: Zeilen bis 100 Zeichen bleiben
  einzeilig, die beiden langen `PersonDirectorySearchEmptyBody`-Eintraege in de
  und cs (und en) wurden auf die mehrzeilige Argumentform umgebrochen. Beide
  Clippy-Gates laufen ohne Ausgabe durch.
- **`.planning/ROADMAP.md` war bei Plan-Start bereits modifiziert** — die
  `1/7 | In Progress`-Zeile stammt aus Plan 62-01 und war dort nicht mit
  committet worden. Sie wurde nicht verworfen, sondern im Doku-Commit dieses
  Plans auf `2/7` fortgeschrieben.

- **PADM-01/PADM-06 bleiben in `REQUIREMENTS.md` auf `Pending`.** Beide Requirements
  werden von allen sieben Plaenen dieser Phase getragen; `requirements mark-complete`
  wurde bewusst **nicht** aufgerufen, damit der Abschluss am Phasen-Close haengt und
  nicht an einem einzelnen Plan. Das `requirements-completed`-Feld im Frontmatter oben
  spiegelt das `requirements`-Feld des Plans, nicht den Zustand in `REQUIREMENTS.md`.
- **`roadmap update-plan-progress` hat erneut die Nyquist-Validierungstabelle
  ueberschrieben** (`| 62 | ja | ja | — |` → `| 62 | 2/7 | In Progress| |`, bekannter
  Tooling-Defekt). Die Zeile wurde von Hand zurueckgesetzt und die korrekte
  Phasen-Fortschrittszeile manuell auf `2/7 | In Progress` gesetzt.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- **62-03 (Business-Logic-Service)** kann `PersonEntryTO`/`PersonKindTO`
  unveraendert als Zielform des `From<&PersonEntry>`-Mappings annehmen. Die
  Feldreihenfolge und die Invarianten stehen in den Doc-Kommentaren direkt am
  Typ.
- **62-04 (Frontend-Umbau)** kann sofort gegen die sechs Keys und gegen
  `PersonKindTO` bauen — beide sind ohne `service-impl` verfuegbar, das Frontend
  uebersetzt bereits.
- **62-05 (REST + OpenAPI)** erbt `ToSchema` auf beiden Typen; der endgueltige
  Beweis der OpenAPI-Faehigkeit ist der dortige `cargo build -p rest`.
- Keine Blocker.

---
*Phase: 62-personen-verzeichnis-eine-liste-statt-tabs*
*Completed: 2026-09-20*

## Self-Check: PASSED

Alle fuenf geaenderten Quelldateien und die SUMMARY existieren auf der Platte;
beide Task-Commits (`48d0aed9`, `f60255e1`) sind in `git log` auffindbar.
