---
phase: 62-personen-verzeichnis-eine-liste-statt-tabs
plan: 03
subsystem: service
tags: [business-logic-service, admin-gate, tdd, aggregate, sorting, dto-mapping]

# Dependency graph
requires:
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-01 — `PersonDirectoryDao` (ungegated, unsortiert) plus `PersonDirectoryRow`/`UserRoleRow`"
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-02 — `PersonEntryTO`/`PersonKindTO` in `rest-types`, noch ohne `From`-Impl"
  - phase: 61-personservice-aufl-sung-von-sales-person-user
    provides: "`PersonService` als Basic-Tier-Vorbild und als Gate-Kontrast (HR vs. admin)"
provides:
  - "`service::person_directory::PersonDirectoryService` — Business-Logic-Trait mit genau einer Methode `list`, mockbar via `#[automock]`"
  - "`service::person_directory::PersonEntry` / `PersonKind` — Domain-Typen des Verzeichnisses"
  - "`service_impl::person_directory::PersonDirectoryServiceImpl` / `PersonDirectoryServiceDeps` — Admin-Gate, Klassifikation, Anzeigename, Rollen-Gruppierung, case-insensitive Sortierung"
  - "`impl From<&PersonEntry> for PersonEntryTO` + `impl From<&PersonKind> for PersonKindTO` — das Transport-Mapping, das 62-05 braucht"
  - "`service_impl::test::person_directory` — acht Tests (S-1..S-8) gegen echte In-Memory-SQLite"
affects: [62-05-rest-di, 62-04-frontend, 62-06-docs]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Gate-Literal im Permission-Mock assertiert statt nur gegreppt — ein falsches Privileg faellt in jedem Test der Datei um"
    - "Klassifikation und Anzeigenamen-Ableitung in einer freien Funktion, damit sie ohne DI-Geruest lesbar bleibt"
    - "Sortierung im Service statt im SQL: locale-unabhaengig und ohne Datenbank testbar"

key-files:
  created:
    - service/src/person_directory.rs
    - service_impl/src/person_directory.rs
    - service_impl/src/test/person_directory.rs
  modified:
    - service/src/lib.rs
    - service_impl/src/lib.rs
    - service_impl/src/test/mod.rs
    - rest-types/src/lib.rs
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md

key-decisions:
  - "Das Gate ist das String-Literal `\"admin\"` (D-62-08), Vorbild `PermissionServiceImpl::get_all_users`. Bewusst NICHT `HR_PRIVILEGE` aus dem Basic-Tier `PersonService` — die Verschaerfung war im Modulkopf von Phase 61 angekuendigt."
  - "Bei fehlendem Privileg wird der Fehler propagiert, nicht redigiert. Von `VoluntaryStatsService` ist nur die Tier- und DI-Form das Vorbild, ausdruecklich nicht seine `None`-Redaktions-Semantik: eine redigierte Antwort waere von einem leeren Verzeichnis ununterscheidbar."
  - "Der Permission-Mock der Testdatei antwortet NUR auf das Literal `\"admin\"` positiv. Damit ist die Gate-Wahl Gegenstand jeder Assertion der Datei und nicht nur eines Grep-Kriteriums — die wirksamere Absicherung gegen RESEARCH.md Pitfall 4."
  - "Sortier-Fixture um `Zacharias` erweitert: `zora`/`Anton`/`mia` allein haette auch eine naive ASCII-Sortierung bestanden. Erst `Zacharias` (0x5A) vor `mia` (0x6D) macht den Test zum echten Nachweis der case-insensitiven Ordnung."
  - "`display_name` wird per `sales_person_name.or(username)` abgeleitet — genau einmal, im Service. Das `From`-Mapping auf `PersonEntryTO` rechnet nichts nach."

patterns-established:
  - "Gate-Wahl als Test-Invariante: statt `grep -c 'check_permission(\"admin\"'` prueft der Permission-Mock das Privileg-Literal und verweigert bei jedem anderen. Ein Copy-Paste-Gate aus einem Nachbar-Service faellt damit sofort und ueberall auf, nicht nur im Gate-Test."
  - "Abwesenheits-Assertion mit Gegenprobe: `list_ohne_admin_privileg_ist_forbidden` prueft zusaetzlich, dass dieselbe Fixture als Admin drei Eintraege liefert — sonst waere `Forbidden` von einem leeren Verzeichnis nicht zu unterscheiden."

# PADM-01 / PADM-06 / PADM-08 bleiben bewusst Pending: alle sieben Plaene dieser
# Phase fuehren sie im Frontmatter, und ihre Aussagen sind nutzersichtbar
# ("ein Admin sieht eine Liste"). Sie gehoeren an den Phasen-Abschluss nach
# 62-07, nicht an diesen reinen Backend-Plan. Siehe 62-01-SUMMARY Deviation 4.
requirements-completed: []
requirements-advanced: [PADM-01, PADM-06, PADM-08]

coverage:
  - id: D1
    description: "Das Aggregat liefert verknuepfte Eintraege, reine Benutzerkonten und reine Verkaeufer nebeneinander, jeder mit gesetzter Auspraegung"
    requirement: "PADM-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_directory.rs#liste_enthaelt_alle_drei_auspraegungen"
        status: pass
    human_judgment: false
  - id: D2
    description: "Der Anzeigename kommt fertig aus dem Service: Verkaeufername wo vorhanden, sonst Benutzername — nie leer"
    requirement: "PADM-06"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_directory.rs#display_name_reines_benutzerkonto_ist_username"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_directory.rs#display_name_verknuepft_ist_verkaeufername"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_directory.rs#username_bei_reinem_verkaeufer_ist_leer"
        status: pass
    human_judgment: false
  - id: D3
    description: "Ein Aufrufer ohne Admin-Privileg bekommt einen Fehler, keine redigierten Daten; das Gate ist die erste Anweisung vor jedem Datenbankzugriff"
    requirement: "PADM-08"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_directory.rs#list_ohne_admin_privileg_ist_forbidden"
        status: pass
      - kind: other
        ref: "sed -n '/async fn list/,/use_transaction/p' service_impl/src/person_directory.rs — check_permission steht vor use_transaction"
        status: pass
    human_judgment: false
  - id: D4
    description: "Die Liste ist case-insensitive nach Anzeigename sortiert"
    requirement: "PADM-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_directory.rs#liste_ist_case_insensitive_nach_display_name_sortiert"
        status: pass
    human_judgment: false
  - id: D5
    description: "Rollen landen beim richtigen Benutzer; ein reiner Verkaeufer hat eine leere Rollenliste"
    requirement: "PADM-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_directory.rs#rollen_werden_dem_richtigen_benutzer_zugeordnet"
        status: pass
    human_judgment: false
  - id: D6
    description: "Soft-geloeschte Verkaeufer erscheinen nicht, inaktive schon (D-62-04) — auf Service-Ebene nachgewiesen"
    requirement: "PADM-01"
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_directory.rs#soft_geloeschter_verkaeufer_erscheint_nicht_inaktiver_schon"
        status: pass
    human_judgment: false
  - id: D7
    description: "Das Transport-Mapping existiert und ist rein feldweise; beide Build-Konfigurationen von rest-types uebersetzen"
    verification:
      - kind: integration
        ref: "nix develop --command bash -c 'SQLX_OFFLINE=true cargo test --workspace'"
        status: pass
      - kind: integration
        ref: "nix develop --command bash -c 'cd shifty-dioxus && cargo build'"
        status: pass
    human_judgment: false

# Metrics
duration: 10min
completed: 2026-09-20
status: complete
---

# Phase 62 Plan 03: Business-Logic-Aggregat `PersonDirectoryService` Summary

**Ein Admin-gegateter Aggregat-Service liefert verknuepfte Personen, reine Benutzerkonten und reine Verkaeufer als eine case-insensitiv sortierte Liste, deren Anzeigename und Auspraegung genau einmal — im Backend — berechnet werden.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-09-20T07:38:05Z
- **Completed:** 2026-09-20T07:48:26Z
- **Tasks:** 2/2
- **Files modified:** 9 (3 neu, 6 geaendert)

## Accomplishments

- **`PersonDirectoryService` als Business-Logic-Trait** mit genau einer Methode
  `list`. Der Modulkopf haelt die Tier-Einordnung fest (das Aggregat fuehrt zwei
  Fach-Objekte zusammen) und begruendet die Gate-Verschaerfung D-62-08 mit dem
  Verweis auf den Modulkopf von `PersonService`, der sie angekuendigt hat.
- **`PersonKind` und `PersonEntry`** als Domain-Typen mit denselben Invarianten
  wie das Transport-Objekt aus 62-02 — so bleibt das Mapping rein feldweise.
- **`PersonDirectoryServiceImpl`** mit genau drei Dependencies und dem
  Admin-Gate als erster Anweisung. Klassifikation und Anzeigenamen-Ableitung
  liegen in der freien Funktion `entry_from_row`, die Sortierung im Service.
- **`impl From<&PersonEntry> for PersonEntryTO`** (plus das Enum-Mapping) in
  `rest-types` hinter `#[cfg(feature = "service-impl")]` — der Baustein, ohne
  den der REST-Handler in 62-05 nicht serialisieren kann.
- **Acht Tests** gegen echte In-Memory-SQLite und den echten DAO; gemockt ist
  nur das Gate.
- **Docs-Freshness-Gate bedient:** `docs/features/F01-employee-management.md`
  und `_de.md` bekommen Sektion 4.6 plus eine Zeile in der Service-Tier-Tabelle.

## Task Commits

| Task | Name | Commit |
| --- | --- | --- |
| 1 (RED) | Trait, Domain-Typen und die vollstaendige Testtafel | `1b38fef9` |
| 2 (GREEN) | `PersonDirectoryServiceImpl` mit Admin-Gate, Klassifikation und Sortierung | `be8fdff8` |

## TDD Gate Compliance

Die Gate-Reihenfolge ist in der Commit-Historie sichtbar:

1. **RED** — `1b38fef9` `test(62-03): add PersonDirectoryService trait, domain
   types and the RED test board`. Der RED-Nachweis ist ein Compile-Fehler, der
   genau auf das noch fehlende Impl-Modul zeigt:

   ```
   error[E0432]: unresolved import `crate::person_directory`
     --> service_impl/src/test/person_directory.rs:56:12
      |
   56 | use crate::person_directory::{PersonDirectoryServiceDeps, PersonDirectoryServiceImpl};
      |            ^^^^^^^^^^^^^^^^ unresolved import
      |
   help: a similar path exists
      |
   56 | use crate::test::person_directory::{PersonDirectoryServiceDeps, PersonDirectoryServiceImpl};
      |            ++++++
   ```

   `cargo build -p service` uebersetzte im selben Zustand fehlerfrei — der
   Trait-Crate stand also bereits, nur die Implementierung fehlte.
2. **GREEN** — `be8fdff8` `feat(62-03): …`, danach 8 passed / 0 failed.
3. **REFACTOR** — entfiel; die GREEN-Implementierung brauchte keine Nacharbeit.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 — Projektvorgabe] Docs-Freshness-Gate bedient (CLAUDE.md)**

- **Found during:** Task 2
- **Issue:** `CLAUDE.md` erklaert das Docs-Gate zur harten Regel: eine Aenderung
  an `service/**/*.rs` (Trait-Signaturen) verlangt die passende
  `docs/features/F*.md`-Sektion — Auth-Gate-Tabelle, Trait-Auszug, Randfaelle —
  im selben Commit, und zwar in **beiden** Sprachen. Der Plan sieht dafuer
  Plan 62-06 vor; das Gate verlangt es aber im selben Commit, und 62-01 hat
  dieselbe Abweichung bereits so aufgeloest.
- **Fix:** Sektion 4.6 `PersonDirectoryService` in
  `docs/features/F01-employee-management.md` und `_de.md` ergaenzt
  (Trait-Auszug, Fat-Backend-Begruendung, Auth-Gate-Tabelle mit D-62-08, drei
  Randfaelle, Transport-Mapping) plus eine Zeile in der Service-Tier-Tabelle
  von §4. 62-06 kann darauf aufbauen statt bei null anzufangen.
- **Files modified:** `docs/features/F01-employee-management.md`,
  `docs/features/F01-employee-management_de.md`
- **Commit:** `be8fdff8`

**2. [Rule 2 — Testschaerfe] Gate-Literal im Permission-Mock assertiert**

- **Found during:** Task 1
- **Issue:** Der Plan sichert die Gate-Wahl ueber zwei Grep-Kriterien ab
  (`check_permission("admin"` vorhanden, `HR_PRIVILEGE` abwesend). Greps sind
  aber blind gegen ein drittes, falsches Privileg — und RESEARCH.md Pitfall 4
  benennt den Copy-Paste aus `service_impl/src/person.rs` als den
  wahrscheinlichsten Fehler dieser Phase.
- **Fix:** `build_perm(admin)` gibt nur dann `Ok(())` zurueck, wenn das
  uebergebene Privileg exakt `"admin"` ist. Ein Service mit falschem Gate faellt
  damit in **allen acht** Tests um, nicht nur im Gate-Test. Die Grep-Kriterien
  des Plans bleiben zusaetzlich erfuellt.
- **Files modified:** `service_impl/src/test/person_directory.rs`
- **Commit:** `1b38fef9`

**3. [Rule 2 — Testschaerfe] Sortier-Fixture um einen Grossbuchstaben-Nachzuegler erweitert**

- **Found during:** Task 1
- **Issue:** Die im Plan vorgegebene Fixture `zora`, `Anton`, `mia` erwartet
  `Anton`, `mia`, `zora`. Diese Reihenfolge liefert eine naive, case-**sensitive**
  ASCII-Sortierung ebenfalls (`A`=0x41 < `m`=0x6D < `z`=0x7A). Der Test haette
  also gegen eine falsche Implementierung gehalten — er waere vakuum gewesen.
- **Fix:** Ein vierter Eintrag `Zacharias` kommt dazu. Erwartet ist jetzt
  `Anton`, `mia`, `Zacharias`, `zora`; eine case-sensitive Sortierung wuerde
  `Zacharias` faelschlich vor `mia` einordnen (0x5A < 0x6D) und der Test faellt.
  Die Aussage des Plans bleibt erhalten und wird schaerfer.
- **Files modified:** `service_impl/src/test/person_directory.rs`
- **Commit:** `1b38fef9`

### Acceptance-Criterion-Abweichungen (dokumentiert, nicht gefixt)

**Task 1, Kriterium `grep -c 'automock' service/src/person_directory.rs` ergibt `1`**
— tatsaechlich ergibt es `2`. Derselbe Authoring-Fehler wie in 62-01 Task 2:
jede Datei, die dem vorgegebenen Muster von `service/src/voluntary_stats.rs`
folgt, enthaelt den String zweimal (`use mockall::automock;` **und**
`#[automock(...)]`). `voluntary_stats.rs` und `person.rs` liefern ebenfalls `2`.
Das Kriterium liesse sich nur durch einen vollqualifizierten Pfad
(`#[mockall::automock(...)]`) ohne `use` erfuellen — also durch Abweichen von
der Konvention, die derselbe Plan verbindlich macht. Die **Absicht** ist
erfuellt und nachgewiesen: `grep -c '#\[automock'` ergibt `1`.

**Task 2, Kriterium `cargo test -p service_impl --lib test::person_directory` meldet `8 passed`**
— der Filter ist ein Praefix-Match und faengt zusaetzlich die sechs
DAO-Integrationstests aus 62-01 (`test::person_directory_dao`), meldet also
`14 passed; 0 failed`. Ersatzkommando mit exaktem Modulpfad:
`cargo test -p service_impl --lib "test::person_directory::"` → `8 passed;
0 failed`. Beide Laeufe sind gruen, das Kriterium ist inhaltlich erfuellt.

## Verification Results

| Gate | Kommando | Ergebnis |
| --- | --- | --- |
| Plan-Tests (Modulpfad exakt) | `cargo test -p service_impl --lib "test::person_directory::"` | 8 passed, 0 failed |
| Plan-Tests (Plan-Filter) | `cargo test -p service_impl --lib test::person_directory -- --nocapture` | 14 passed, 0 failed (inkl. 6 DAO-Tests aus 62-01) |
| PADM-01 | `cargo test … test::person_directory::liste_enthaelt_alle_drei_auspraegungen -- --exact` | 1 passed |
| PADM-06 | `cargo test … test::person_directory::display_name_reines_benutzerkonto_ist_username -- --exact` | 1 passed |
| PADM-08 | `cargo test … test::person_directory::list_ohne_admin_privileg_ist_forbidden -- --exact` | 1 passed |
| Workspace | `cargo test --workspace` | alle gruen (u.a. 904 + 93 Tests), 0 failed |
| Clippy (Hard Gate) | `cargo clippy --workspace -- -D warnings` | ohne Ausgabe |
| Frontend | `cd shifty-dioxus && cargo build` | uebersetzt (rest-types ohne `service-impl`) |
| Gate-Literal | `grep -c 'check_permission("admin"' service_impl/src/person_directory.rs` | 1 |
| Kein HR-Gate | `grep -v '^//' … \| grep -v '^\s*//' \| grep -c 'HR_PRIVILEGE'` | 0 |
| Gate zuerst | `sed -n '/async fn list/,/use_transaction/p' …` | `check_permission` vor `use_transaction` |
| Genau drei Deps | `sed -n '/gen_service_impl! {/,/^}/p' … \| grep -c '='` | 4 (drei Deps + Struct-Zeile) |
| Sortierung im Service | `grep -c 'to_lowercase' service_impl/src/person_directory.rs` | 1 |
| Registrierungen | `grep -c 'pub mod person_directory;'` in `service/src/lib.rs`, `service_impl/src/lib.rs`, `service_impl/src/test/mod.rs` | je 1 |

Alle Kommandos via `nix develop --command bash -c 'SQLX_OFFLINE=true …'`;
`cargo` liegt nicht im Default-PATH.

## Threat Mitigations

| Threat ID | Disposition | Umsetzung |
| --- | --- | --- |
| T-62-01 | mitigate | `check_permission("admin", context)` als erste Anweisung von `list`, vor `use_transaction`. Nachgewiesen durch `list_ohne_admin_privileg_ist_forbidden` **und** durch den Permission-Mock, der nur auf `"admin"` positiv antwortet. Source-Assertion (`sed`) bestaetigt die Reihenfolge. Der HTTP-Beweis folgt in 62-05. |
| T-62-05 | mitigate | Dieselbe Pruefung. Bei fehlendem Privileg wird `ServiceError::Forbidden` propagiert — keine Teil-Antwort, keine Redaktion. Die Gegenprobe im selben Test (als Admin liefert dieselbe Fixture drei Eintraege) schliesst aus, dass `Forbidden` nur ein leeres Verzeichnis kaschiert. |
| T-62-10 | accept | `unreachable!()` im Klassifikations-`match`, mit Kommentar und mit der `person_id` in der Panic-Meldung. Das SQL schliesst den Fall bereits aus (62-01) — doppelter Boden, kein Live-Pfad. |
| T-62-11 | accept | `Authentication<Context>` unveraendert durchgereicht wie in jedem anderen Service des Repos. |
| T-62-SC | accept | Kein einziges neues Paket. `Cargo.toml` ist in keinem der beiden Commits enthalten. |

## Known Stubs

Keine. Der Service ist vollstaendig implementiert; `list` liefert echte Daten
aus dem echten DAO. Der Service hat allerdings **noch keinen Laufzeit-Aufrufer**
— die DI-Verdrahtung in `shifty_bin/src/main.rs` und der REST-Handler kommen in
62-05. Das ist die geplante Wellen-Reihenfolge, kein Stub.

## Threat Flags

Keine neue, ungeplante sicherheitsrelevante Oberflaeche. Der Plan fuehrt das
einzige Auth-Gate des Verzeichnisses ein; es steht im `<threat_model>` und ist
oben als T-62-01 / T-62-05 abgehandelt. Keine REST-Oberflaeche in diesem Plan.

## Notes for Future Phases

- **62-05 verdrahtet, prueft aber nicht doppelt.** Der REST-Handler soll
  bewusst kein zweites Gate setzen — die Trust Boundary liegt an
  `PersonDirectoryService::list`. `PersonDirectoryServiceImpl` gehoert in den
  **Business-Logic**-Block von `shifty_bin/src/main.rs`, nach den Basic
  Services; `PersonDirectoryDaoImpl::new(pool)` ist bisher nirgends
  instanziiert (Hinweis aus 62-01 gilt weiter).
- **62-04 muss nichts ableiten.** `display_name` ist immer gefuellt und `kind`
  ist ein Fakt. Ein Frontend-Fallback (`sales_person_name ?? username`) waere
  eine zweite Quelle derselben Wahrheit und widerspraeche D-62-01/D-62-03.
- **62-06 findet F01 bereits aktualisiert vor** (Sektion 4.6 EN+DE plus
  Service-Tier-Zeile). Offen bleiben dort die REST-Sektion (nach 62-05) und
  `docs/architecture/02-service-tiers.md` samt Runtime-Service-Graph — beide
  haengen an der DI-Verdrahtung, die dieser Plan nicht anfasst.
- **Die Rollen-Map gruppiert ueber den Benutzernamen**, weil `UserRoleRow` den
  Benutzernamen traegt und `user.name` der Primaerschluessel ist. Sollte Phase
  63 Benutzer umbenennbar machen, ist das hier der Punkt, der auf `person_id`
  umgestellt werden muss.

## Self-Check: PASSED

Dateien:

- FOUND: `service/src/person_directory.rs`
- FOUND: `service_impl/src/person_directory.rs`
- FOUND: `service_impl/src/test/person_directory.rs`

Commits:

- FOUND: `1b38fef9`
- FOUND: `be8fdff8`
