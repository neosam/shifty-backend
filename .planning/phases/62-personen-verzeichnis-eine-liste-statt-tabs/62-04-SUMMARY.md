---
phase: 62-personen-verzeichnis-eine-liste-statt-tabs
plan: 04
subsystem: frontend
tags: [dioxus, rsx, ssr-test, tailwind-tokens, admin-guard, aggregate-loader, n+1-removal]

# Dependency graph
requires:
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "`rest_types::PersonEntryTO` / `PersonKindTO` und die sechs i18n-Keys aus Plan 62-02"
provides:
  - "`shifty-dioxus::api::get_person_directory` — ein GET auf `{backend}/person-directory`"
  - "`shifty-dioxus::loader::load_person_directory` — reicht das Aggregat unveraendert durch"
  - "`UserManagementStore.directory` + `UserManagementAction::LoadPersonDirectory`"
  - "`component::atoms::person_kind_badge::{PersonKindBadge, person_kind_badge_class, person_kind_label_key}`"
  - "`page::user_management::{is_admin, NotAuthorizedView, PersonDirectoryTable}`"
affects:
  - "62-05 REST + Dev-Proxy — der Route-Praefix muss zeichengleich `/person-directory` sein"
  - "62-06 Browser-Nachweis — rendert diese Seite gegen den echten Endpoint"
  - "63 Kandidaten-Picker — erbt die konservierten `load_all_users`/`get_all_users`-Pfade"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Aggregat-Loader statt N+1: ein Store-Feld, eine Action, ein `.await` im Coroutine-Arm"
    - "Quell-Assertion als Regressionsschutz fuer die Request-Anzahl (`include_str!` auf das eigene Modul)"
    - "Auspraegung als Backend-Enum; das Frontend `match`t nur, es leitet nie ab"
    - "Guard-Praedikat als freie Funktion + eigene Ablehnungs-Komponente, damit beides ohne Dioxus-Laufzeit bzw. per SSR pruefbar ist"
    - "Begruendete `#[allow(dead_code)] // reason: …`-Konservierung statt Loeschung fuer Pfade, die eine Folgephase wieder braucht"

key-files:
  created:
    - shifty-dioxus/src/component/atoms/person_kind_badge.rs
  modified:
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/service/user_management.rs
    - shifty-dioxus/src/page/user_management.rs
    - shifty-dioxus/src/component/atoms/mod.rs
    - shifty-dioxus/src/component/mod.rs
    - shifty-dioxus/src/state/user_management.rs
  deleted:
    - shifty-dioxus/src/component/user_management_tab_bar.rs

key-decisions:
  - "Der Ein-Request-Nachweis ist eine Quell-Assertion und zaehlt `.await` als Text — darum darf kein Kommentar im geprueften Arm das Literal `.await` enthalten. Der Arm-Kommentar umschreibt es bewusst."
  - "Die `Typ`-Spalte traegt Auspraegungs-Badge, Paid/Volunteer-Pille und Inaktiv-Badge gemeinsam; damit bleibt es bei vier Inhaltsspalten plus kopfloser Aktionsspalte wie im Plan-Action-Text vorgegeben."
  - "`state::user_management::User` wurde zusaetzlich zu `loader::load_all_users`/`api::get_all_users` als begruendetes dead_code konserviert — mit dem Loader fiel auch sein einziger Konstruktionspfad weg."
  - "`is_some_and` statt `map(...).unwrap_or(false)` im Suchfilter: der Fat-Backend-Grep verbietet `unwrap_or` in jeder Zeile, die `username` beruehrt."

patterns-established:
  - "Quell-Assertions muessen den Testblock vom Produktionscode abschneiden (`src[..src.find(\"#[cfg(test)]\")]`), sonst findet `find` das Suchmuster im Testliteral selbst."
  - "Akzeptanz-`sed`-Ranges brechen an Kommentaren, die den Endmarker enthalten — Kommentartext im geprueften Bereich muss den Marker meiden."

requirements-completed: []

coverage:
  - id: D1
    description: "Genau ein Aggregat-Aufruf fuellt die Liste; kein Nachladen pro Zeile (Success Criterion 5)"
    requirement: "PADM-01"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/service/user_management.rs#tests::load_directory_ist_ein_request"
        status: pass
      - kind: other
        ref: "grep -c 'join_all' shifty-dioxus/src/service/user_management.rs == 0"
        status: pass
      - kind: other
        ref: "grep -rl 'LoadAllUsers|LoadAllSalesPersons' shifty-dioxus/src/ | wc -l == 0"
        status: pass
    human_judgment: false
  - id: D2
    description: "Eine Liste ohne Tab-Leiste; alle drei Auspraegungen stehen nebeneinander und tragen ein unterscheidbares Badge"
    requirement: "PADM-01"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/page/user_management.rs#tests::directory_row_renders_kind_badge"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/component/atoms/person_kind_badge.rs#tests::person_kind_badge_class_ist_statisch_je_variante"
        status: pass
      - kind: other
        ref: "test ! -f shifty-dioxus/src/component/user_management_tab_bar.rs && grep -rc 'UserManagementTabBar' shifty-dioxus/src/ | grep -v ':0' | wc -l == 0"
        status: pass
    human_judgment: false
  - id: D3
    description: "Ein Suchfeld findet einen Eintrag ueber Anzeigename ODER Benutzername (Success Criterion 2)"
    requirement: "PADM-01"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/page/user_management.rs#tests::directory_search_matches_both_names"
        status: pass
      - kind: other
        ref: "grep -c 'use_signal(String::new)' shifty-dioxus/src/page/user_management.rs == 1"
        status: pass
    human_judgment: false
  - id: D4
    description: "Im Frontend existiert keine Namens-Fallback-Regel; die Auspraegung kommt ausschliesslich aus dem Backend-Enum (Success Criterion 3, T-62-09)"
    requirement: "PADM-06"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/page/user_management.rs#tests::directory_user_only_row_has_display_name"
        status: pass
      - kind: other
        ref: "grep -n 'display_name|username' shifty-dioxus/src/page/user_management.rs | grep -c 'unwrap_or|\\.or(' == 0"
        status: pass
      - kind: other
        ref: "awk '/#\\[cfg\\(test\\)\\]/{exit} /PersonKindTO::/{c++} END{print c+0}' shifty-dioxus/src/page/user_management.rs == 0"
        status: pass
    human_judgment: false
  - id: D5
    description: "Ein Nutzer ohne Admin-Rolle sieht bei direktem Routenaufruf die Ablehnung statt der Liste (T-62-02, D-62-08)"
    requirement: "PADM-08"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/page/user_management.rs#tests::non_admin_sees_not_authorized"
        status: pass
      - kind: other
        ref: "sed -n '/pub fn UserManagementPage/,/use_effect/p' shifty-dioxus/src/page/user_management.rs | grep -c 'is_admin' >= 1"
        status: pass
    human_judgment: false
  - id: D6
    description: "Anlegen, Loeschen, Impersonation und die Rollen-Spalte funktionieren weiter und sind auf Eintraege mit Benutzer-Auspraegung beschraenkt (D-62-06)"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/page/user_management.rs#tests::directory_actions_only_for_user_side"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/page/user_management.rs#tests::add_user_dialog_open_renders_field_and_buttons"
        status: pass
      - kind: unit
        ref: "shifty-dioxus/src/page/user_management.rs#tests::delete_user_confirm_dialog_open_interpolates_username_and_has_danger_button"
        status: pass
    human_judgment: false
  - id: D7
    description: "Das Frontend uebersetzt warnungsfrei und ist clippy-sauber"
    verification:
      - kind: integration
        ref: "nix develop --command bash -c 'cd shifty-dioxus && cargo clippy --all-targets -- -D warnings'"
        status: pass
      - kind: integration
        ref: "nix develop --command bash -c 'cd shifty-dioxus && cargo test' — 835 passed / 0 failed"
        status: pass
      - kind: integration
        ref: "nix develop --command bash -c 'SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings'"
        status: pass
    human_judgment: false
  - id: D8
    description: "Die gerenderte Seite sieht im Browser richtig aus: Badge-Farben, Spaltenbreiten, Toolbar-Hierarchie und die drei Sprachen"
    verification: []
    human_judgment: true
    rationale: "Der laufende Browser-Nachweis braucht den REST-Endpoint aus 62-05 und ist ausdruecklich in 62-06/62-07 verortet; dieser Plan testet ausschliesslich per SSR mit injizierten Eintraegen."

# Metrics
duration: 22min
completed: 2026-09-20
status: complete
---

# Phase 62 Plan 04: Frontend-Umbau Summary

**Ein Aggregat-Request ersetzt das 2+N+2M-Ladeverhalten der User-Management-Seite; `/user_management/` zeigt statt zweier Tabs eine admin-gegatete, durchsuchbare Liste mit Auspraegungs-Badge.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-09-20T07:51:30Z
- **Completed:** 2026-09-20T08:13:22Z
- **Tasks:** 3 (davon 2 im TDD-Zyklus)
- **Files created:** 1
- **Files modified:** 7
- **Files deleted:** 1

## Accomplishments

- **Aggregat-Datenpfad.** `api::get_person_directory` ruft
  `{backend}/person-directory`, `loader::load_person_directory` reicht das
  Ergebnis **unveraendert** durch (keine Sortierung, keine Umformung — das
  waere die Logik-Dopplung, die das Fat-Backend-Prinzip verbietet), und
  `UserManagementAction::LoadPersonDirectory` schreibt es in das neue
  Store-Feld `directory`.
- **Ein-Request-Nachweis als Test.** `load_directory_ist_ein_request` schneidet
  den Coroutine-Arm aus der eigenen Quelle aus und assertiert genau ein
  `.await` darin — zeilenzahl-unabhaengiger Regressionsschutz fuer Success
  Criterion 5.
- **`PersonKindBadge`-Atom.** Drei statische Token-Klassenstrings
  (accent / warn / neutral-dashed), keine `format!`-Konstruktion, kein
  Roh-Hexwert. Alle drei Varianten recyceln Tokens, die auf dieser Seite
  ohnehin vorkommen (62-UI-SPEC Open Question 2).
- **Component-Level-Admin-Guard.** `is_admin` als freie Funktion (ohne
  Dioxus-Laufzeit pruefbar) plus `NotAuthorizedView`; der fruehe `return`
  steht vor jedem Effekt-Hook und vor jedem Datenzugriff (T-62-02, D-62-08).
- **Eine Liste statt zweier Tabs.** `PersonDirectoryTable` rendert Werkzeugleiste
  (ein Suchfeld, Sekundaer-CTA „Neuen Verkaeufer anlegen", Primaer-CTA „Neuen
  Benutzer anlegen") und eine Tabelle mit vier Inhaltsspalten plus kopfloser
  Aktionsspalte. Zeilen-Anatomie nach 62-UI-SPEC: Farbpunkt nur bei
  vorhandener Verkaeufer-Seite, Anzeigename als Zeilentitel, Auspraegungs-Badge
  neben Paid/Volunteer-Pille und Inaktiv-Badge, Benutzername in `font-mono`,
  Rollen als Chips, Gedankenstrich-Key bei leeren Zellen.
- **Aktions-Gating und Verlinkung.** Impersonation und Loeschen erscheinen nur
  bei vorhandenem Benutzernamen (D-62-06); der Bearbeiten-Link zeigt bei
  vorhandener Verkaeufer-Seite auf die Verkaeufer-Detailroute, sonst auf die
  Benutzer-Detailroute (D-62-07).
- **N+1-Abriss.** Fuenf Store-Felder, fuenf Lade-/Nachlade-Funktionen, zwei
  Hilfsfunktionen samt fuenf Unit-Tests, zwei Actions und ihre Coroutine-Arme
  sowie die Tab-Leisten-Datei sind entfernt. Die drei Auffrisch-Pfade (Benutzer
  anlegen, Benutzer loeschen, Verkaeufer speichern-und-navigieren) rufen jetzt
  das Aggregat.

## Task Commits

Each task was committed atomically:

1. **Task 1 (RED): Ein-Request-Nachweis** - `c7f9cd30` (test)
2. **Task 1 (GREEN): Aggregat-Datenpfad** - `2855b30d` (feat)
3. **Task 2 (RED): Listen-, Badge- und Guard-Tests** - `4389427f` (test)
4. **Task 2 (GREEN): Seitenumbau + Tab-Leiste entfernt** - `a97d328c` (feat)
5. **Task 3: N+1-Abriss und Clippy-Hard-Gate** - `5f4af01c` (refactor)

## Files Created/Modified

- `shifty-dioxus/src/component/atoms/person_kind_badge.rs` **(neu)** —
  `person_kind_badge_class`, `person_kind_label_key`, Komponente
  `PersonKindBadge`, fuenf Unit-Tests inkl. Legacy-Klassen-Gate.
- `shifty-dioxus/src/api.rs` — `get_person_directory` mit Zeilenkommentar zur
  Zeichengleichheit des Route-Praefix; `get_all_users` als begruendetes
  dead_code annotiert.
- `shifty-dioxus/src/loader.rs` — `load_person_directory` mit
  Fat-Backend-Begruendung im Doc-Kommentar; `load_all_users` annotiert.
- `shifty-dioxus/src/service/user_management.rs` — Store-Feld `directory`,
  `load_person_directory`, Action + Coroutine-Arm, Testmodul auf die eine
  Quell-Assertion reduziert; alle N+1-Pfade entfernt (680 → 496 Zeilen).
- `shifty-dioxus/src/page/user_management.rs` — vollstaendig umgebaut:
  Guard, `PersonDirectoryTable`, neuer SSR-Harness `render_directory`, Fixture
  `sample_entry`, sieben neue Tests; beide Tab-Komponenten und ihre Tests
  entfernt (878 → 841 Zeilen).
- `shifty-dioxus/src/component/atoms/mod.rs`, `shifty-dioxus/src/component/mod.rs`
  — `PersonKindBadge` registriert und re-exportiert, Tab-Leisten-Eintraege
  entfernt.
- `shifty-dioxus/src/state/user_management.rs` — `User` als begruendetes
  dead_code konserviert.
- `shifty-dioxus/src/component/user_management_tab_bar.rs` **(geloescht)** —
  die Seite war der einzige Konsument.

## Decisions Made

- **Quell-Assertionen muessen den Testblock abschneiden.** Der erste Entwurf von
  `load_directory_ist_ein_request` suchte den Marker in der gesamten Datei und
  fand ihn im eigenen Testliteral. Geloest wie im etablierten
  `no_legacy_classes_in_source`-Muster: nur `src[..src.find("#[cfg(test)]")]`
  durchsuchen.
- **Kommentartext zaehlt bei Quell-Assertionen mit.** Der Arm-Kommentar
  enthielt zunaechst das Literal `.await` und verdoppelte damit den Zaehlwert.
  Der Kommentar umschreibt es jetzt bewusst und sagt auch, warum.
- **`is_some_and` statt `map(...).unwrap_or(false)`** im Such-Praedikat — der
  Fat-Backend-Grep verbietet `unwrap_or` in jeder Zeile, die `username`
  beruehrt, und `is_some_and` ist hier ohnehin die idiomatischere Form.
- **Die `Typ`-Spalte sammelt Badge und Pillen.** Der Plan-Action-Text gibt vier
  Spaltenkoepfe plus kopflose Aktionsspalte vor; die Paid/Volunteer-Pille und
  das Inaktiv-Badge sassen schon im alten Verkaeufer-Tab in der
  `ColumnType`-Spalte und bleiben dort.
- **`state::user_management::User` zusaetzlich konserviert.** Der Plan nannte
  nur `loader.rs`/`api.rs`; mit dem Sammel-Loader fiel aber auch der einzige
  Konstruktionspfad des Zustandstyps weg. Annotiert statt geloescht, mit
  derselben Phase-62/63-Begruendung.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Akzeptanz-`sed`-Range brach am eigenen Kommentar**

- **Found during:** Task 2
- **Issue:** Das Kriterium
  `sed -n '/pub fn UserManagementPage/,/use_effect/p' … | grep -c 'is_admin'`
  ergab `0`, obwohl der Guard korrekt vor dem Effekt-Hook stand. Ursache: der
  Guard-Kommentar enthielt selbst das Wort `use_effect` und beendete den
  `sed`-Bereich vor der `is_admin`-Zeile.
- **Fix:** Kommentar auf „vor jedem Effekt-Hook" umformuliert. Kein
  Verhaltens- oder Strukturwechsel; der Guard stand die ganze Zeit an der
  richtigen Stelle.
- **Files modified:** `shifty-dioxus/src/page/user_management.rs`
- **Verification:** Kriterium ergibt jetzt `1`.
- **Committed in:** `a97d328c`

**2. [Rule 1 - Bug] Quell-Assertion fand sich selbst**

- **Found during:** Task 1 (RED)
- **Issue:** `src.find(MARKER)` traf das Marker-Literal im Testblock, bevor der
  Produktionscode existierte — die Fehlermeldung zeigte Testcode statt eines
  Coroutine-Arms. Nach der Implementierung waere der Test zwar zufaellig
  richtig gelaufen, die Assertion aber nicht mehr eindeutig verankert.
- **Fix:** Suche auf den Nicht-Test-Teil der Datei begrenzt (Muster aus
  `no_legacy_classes_in_source`).
- **Files modified:** `shifty-dioxus/src/service/user_management.rs`
- **Verification:** RED-Lauf meldet jetzt „LoadPersonDirectory arm missing"
  statt Testcode; GREEN-Lauf zaehlt genau einen Aufruf.
- **Committed in:** `c7f9cd30`

**3. [Rule 2 - Missing] Badge-Test war trivial gruen**

- **Found during:** Task 2 (RED)
- **Issue:** `html.contains(cls)` ist fuer einen leeren Klassenstring immer
  wahr — `directory_row_renders_kind_badge` lief gegen den Stub gruen und
  haette einen leeren Badge nie gemeldet.
- **Fix:** Zusaetzliche Assertion `!cls.is_empty()` mit Begruendung im
  Meldungstext.
- **Files modified:** `shifty-dioxus/src/page/user_management.rs`
- **Verification:** Test faellt gegen den Stub, laeuft gegen die Implementierung.
- **Committed in:** `4389427f`

**4. [Rule 2 - Missing] `state::user_management::User` wurde aufruferlos**

- **Found during:** Task 3
- **Issue:** Der Plan nannte fuer die Dead-Code-Konservierung nur `loader.rs`
  und `api.rs`. Mit dem Wegfall von `service::load_all_users` verlor auch der
  Zustandstyp `User` seinen letzten Konstruktionspfad → `cargo build`-Warnung,
  die das Clippy-Hard-Gate gerissen haette.
- **Fix:** `#[allow(dead_code)] // reason: …` mit derselben Phase-62/63-
  Begruendung auf den Typ gesetzt (sein `From<&UserTO>`-Impl bleibt dadurch
  ebenfalls erreichbar).
- **Files modified:** `shifty-dioxus/src/state/user_management.rs`
- **Verification:** `cargo build` warnungsfrei, Clippy ohne Ausgabe.
- **Committed in:** `5f4af01c`

**5. [Rule 3 - Blocking] `use super::*` im Restbestand des Testmoduls**

- **Found during:** Task 3
- **Issue:** Nach dem Entfernen der fuenf `partition_results`-Tests brauchte
  das verbliebene Testmodul den Glob-Import nicht mehr → `unused_imports` unter
  `-D warnings`.
- **Fix:** Import entfernt.
- **Files modified:** `shifty-dioxus/src/service/user_management.rs`
- **Verification:** Clippy-Hard-Gate ohne Ausgabe.
- **Committed in:** `5f4af01c`

---

**Total deviations:** 5 auto-fixed (2 blocking, 2 missing, 1 bug)
**Impact on plan:** Kein Scope-Wechsel. Zwei Befunde (1, 2) betreffen die
Mechanik der grep-/Quell-basierten Nachweise, zwei (4, 5) sind direkte Folgen
des im Plan vorgesehenen Abrisses, einer (3) haertet einen Test, der sonst
nichts bewiesen haette.

## Clippy-Baseline

Vor Task 1 aufgenommen:
`nix develop --command bash -c 'cd shifty-dioxus && cargo clippy --all-targets'`
→ **null Warnungen**. Dieser Plan hat also keine vorbestehenden Warnungen
geerbt und hinterlaesst ebenfalls keine.

## Dead-Code-Bilanz (Task 3)

**Entfernt** (Store + Service):
- Store-Felder `users`, `sales_persons`, `sales_person_user_links`,
  `user_sales_person_links`, `user_role_assignments`
- `load_all_users`, `load_all_sales_persons`,
  `load_all_sales_person_user_links`, `load_all_user_sales_person_links`,
  `load_all_user_roles`
- `partition_results`, `report_errors` und ihre fuenf Unit-Tests
- Actions `LoadAllUsers`, `LoadAllSalesPersons` samt Coroutine-Armen
- Importe `std::collections::HashMap`, `futures::future::join_all`,
  `state::User`
- Datei `component/user_management_tab_bar.rs` samt `pub mod` und `pub use`

**Neu annotiert: 3** (alle mit `// reason:` auf derselben Zeile)
- `shifty-dioxus/src/api.rs` → `get_all_users`
- `shifty-dioxus/src/loader.rs` → `load_all_users`
- `shifty-dioxus/src/state/user_management.rs` → `struct User`

Das Akzeptanzkriterium greppt `loader.rs` und `api.rs`; dort stehen die
geforderten 2 begruendeten Annotationen.

## Issues Encountered

- **`rustfmt` fehlt in der DevShell** (bekannt). Alle Einfuegungen wurden von
  Hand rustfmt-konform formatiert; beide Clippy-Gates laufen ohne Ausgabe.
- **`GET /person-directory` existiert zur Laufzeit noch nicht** — der Endpoint
  entsteht in Plan 62-05. Das ist kein Blocker: alle Tests injizieren die
  Eintraege direkt in `PersonDirectoryTable`, kein Test faehrt Netzwerk.
- **Der Admin-Guard liegt vor den Hook-Aufrufen** und folgt damit exakt dem
  Praezedenzfall `page/settings.rs`. Das ist streng genommen ein
  bedingter Hook-Pfad; die Seite rendert in beiden Faellen einen stabilen Baum,
  und das Muster ist im Projekt etabliert (D-62-08 schreibt die woertliche
  Uebernahme vor).

## Known Stubs

Keine. Alle gerenderten Zellen haengen an Feldern von `PersonEntryTO`; es gibt
keine hartkodierte Leerliste und keinen Platzhaltertext.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- **62-05 (REST + DI + Proxy)** muss den Route-Praefix **zeichengleich**
  `/person-directory` setzen — in `.nest(...)` und im `[[web.proxy]]`-Eintrag
  der `Dioxus.toml`. Der Frontend-Aufruf steht in `api.rs` mit Zeilenkommentar.
  Die erwartete Antwortform ist `[PersonEntryTO]` (JSON-Array).
- **62-06 (Browser-Nachweis)** kann direkt gegen `/user_management/` fahren,
  sobald 62-05 gelandet ist; die Seite braucht keine weiteren Frontend-Arbeiten.
- **62-07** traegt die Laufzeit-Verifikation der Badge-Farben, Spaltenbreiten
  und der drei Sprachen (offener Human-Judgment-Punkt D8 hier und D5 aus 62-02).
- **PADM-01/06/08 bleiben in `REQUIREMENTS.md` auf `Pending`** — alle sieben
  Plaene der Phase tragen sie; der Abschluss haengt am Phasen-Close, nicht an
  einem einzelnen Plan.
- Keine Blocker.

---
*Phase: 62-personen-verzeichnis-eine-liste-statt-tabs*
*Completed: 2026-09-20*

## Self-Check: PASSED

Alle im SUMMARY genannten Dateien existieren auf der Platte (die
Tab-Leisten-Datei ist bestaetigt geloescht), und alle fuenf Task-Commits
(`c7f9cd30`, `2855b30d`, `4389427f`, `a97d328c`, `5f4af01c`) sind in der
Git-Historie auffindbar.
