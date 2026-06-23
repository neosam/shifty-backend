---
phase: 09-booking-flow-reverse-warnings-copy-week
plan: "01"
subsystem: ui
tags: [dioxus, i18n, wasm, components, booking, warnings]

requires:
  - phase: 08-absences-frontend
    provides: WarningsList newtype-PartialEq-Pattern, SSR-Snapshot-Tests, Per-Locale-Reference-Matcher-Tests, Dialog-Primitive

provides:
  - "component/warning_list.rs: WarningsList newtype + WarningList component mit 3 Booking-Varianten + person_name + suppress_header"
  - "api::book_slot_with_conflict_check -> BookingCreateResultTO (POST /shiftplan-edit/booking)"
  - "loader::register_user_to_slot_with_conflict_check -> (Uuid, Vec<WarningTO>)"
  - "7 neue i18n Keys (BookingWarning*) in en/de/cs"
  - "Alle SSR-Snapshot- und i18n-Parity-Tests für Phase-9-Warning-Foundation grün"

affects:
  - 09-02-PLAN

tech-stack:
  added: []
  patterns:
    - "suppress_header-Prop auf WarningList verhindert Doppel-Header im Dialog-Kontext"
    - "person_name: Option<ImStr>-Prop ermöglicht Personennamen-Interpolation in Booking-Warnungen"
    - "booking-path WarningList aus component/ statt inline in page/ — geteilte single source of truth"

key-files:
  created:
    - shifty-dioxus/src/component/warning_list.rs
  modified:
    - shifty-dioxus/src/component/mod.rs
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/page/absences.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs

key-decisions:
  - "WarningList aus absences.rs nach component/warning_list.rs extrahiert — single source of truth für beide Surfaces (D-09)"
  - "suppress_header: bool Prop (default false) vermeidet Doppel-Header wenn Dialog.title die Überschrift liefert"
  - "person_name: Option<ImStr> als Prop übergeben statt im Component selbst aufgelöst"
  - "book_slot_with_conflict_check als neue Funktion neben add_booking (kein Umbenennen) — add_booking bleibt Regression-Lock"
  - "Booking-path-Header nutzt BookingWarningDialogHeaderSingular/Plural, nicht die Absence-Warning-Keys"

requirements-completed: [FUI-A-05]

duration: 45min
completed: 2026-06-12
---

# Phase 09 Plan 01: Booking-Warning Foundation Summary

**Shared WarningList-Komponente (3 Booking-Varianten, person_name, suppress_header), conflict-aware API/Loader-Funktionen und 7 i18n-Keys in en/de/cs als Foundation für Plan 02 (Shiftplan-Dialog-Integration)**

## Performance

- **Duration:** ca. 45 min
- **Started:** 2026-06-12T05:45:00Z
- **Completed:** 2026-06-12T07:05:00Z
- **Tasks:** 3 (alle abgeschlossen)
- **Files modified:** 8

## Accomplishments

- `component/warning_list.rs` erstellt: `WarningsList`-Newtype + `WarningList`-Komponente mit allen 5 `WarningTO`-Varianten (3 neue Booking-Varianten + 2 bestehende Absence-Varianten), `person_name`- und `suppress_header`-Props; 5 SSR-Tests grün
- 7 neue `BookingWarning*`-i18n-Keys in allen 3 Locales (en/de/cs); Parity-Test + Reference-Matcher-Test grün; `{person}`-Platzhalter in de.rs verifiziert (Pitfall-1-Guard)
- `api::book_slot_with_conflict_check` und `loader::register_user_to_slot_with_conflict_check` fertig; `add_booking`/`register_user_to_slot` unverändert (Regression-Lock)
- `absences.rs` importiert `WarningList`/`WarningsList` jetzt aus `component::warning_list` — kein Inline-Duplikat mehr
- Vollständige Frontend-Test-Suite: 551 Tests grün; WASM-Build-Gate `exit 0`

## Task Commits

Jede Task wurde atomar committed:

1. **Task 1: i18n Keys** - `713ac3c7` (feat)
2. **Task 2: component/warning_list.rs** - `4d9540d8` (feat)
3. **Task 3: api + loader + absences.rs Import** - `5062800b` (feat)

## Files Created/Modified

- `shifty-dioxus/src/component/warning_list.rs` — Neue Datei: WarningsList-Newtype + WarningList-Komponente + 5 SSR-Tests
- `shifty-dioxus/src/component/mod.rs` — `pub mod warning_list` + `pub use warning_list::{WarningList, WarningsList}`
- `shifty-dioxus/src/api.rs` — `BookingCreateResultTO`/`WarningTO` importiert; `book_slot_with_conflict_check` hinzugefügt
- `shifty-dioxus/src/loader.rs` — `register_user_to_slot_with_conflict_check` hinzugefügt
- `shifty-dioxus/src/page/absences.rs` — Inline-Definitionen entfernt; `use crate::component::warning_list::{WarningList, WarningsList}` hinzugefügt
- `shifty-dioxus/src/i18n/mod.rs` — 7 neue Key-Varianten + 2 neue Tests
- `shifty-dioxus/src/i18n/de.rs` — 7 deutsche Übersetzungen (alle Locale::De)
- `shifty-dioxus/src/i18n/en.rs` — 7 englische Übersetzungen
- `shifty-dioxus/src/i18n/cs.rs` — 7 tschechische Übersetzungen

## Decisions Made

- `suppress_header: bool`-Prop (default `false`): Der Booking-Dialog in `shiftplan.rs` (Plan 02) übergibt `suppress_header: true`, weil `Dialog { title }` bereits die Überschrift rendert. `absences.rs`-Konsumenten behalten `false` (internen Header weiterhin anzeigen).
- `person_name: Option<ImStr>` als Prop statt Side-Join im Component: Einfacher, Absence-path-Konsumenten übergeben `None` (Fallback `"–"`).
- Header-Keys: `BookingWarningDialogHeaderSingular/Plural` statt Wiederverwendung der `AbsenceWarningHeader*`-Keys — semantisch klarer, Dialog-spezifischer Wortlaut.
- `book_slot_with_conflict_check` als neue separate Funktion: `add_booking` (altes `/booking`-Endpoint) bleibt als Regression-Lock unverändert.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] I18N import aus service::i18n statt crate::i18n**
- **Found during:** Task 2 (warning_list.rs erstellen)
- **Issue:** `use crate::i18n::I18N` schlägt fehl — `I18N` ist in `crate::service::i18n`, nicht in `crate::i18n`
- **Fix:** Import auf `use crate::service::i18n::I18N` korrigiert
- **Files modified:** shifty-dioxus/src/component/warning_list.rs
- **Verification:** `cargo test warning_list` exit 0

**2. [Rule 1 - Bug] ImStr hat kein Deref<Target=str>, as_deref() funktioniert nicht**
- **Found during:** Task 2 (warning_list.rs)
- **Issue:** `Option<ImStr>` unterstützt `as_deref()` nicht, da `ImStr` kein `Deref<Target=str>` implementiert
- **Fix:** `person_name.as_ref().map(|n| n.as_str().to_string()).unwrap_or_else(|| "–".to_string())`
- **Files modified:** shifty-dioxus/src/component/warning_list.rs
- **Verification:** Kompiliert und Tests grün

---

**Total deviations:** 2 auto-fixed (beide Rule 1 - Bug)
**Impact on plan:** Beide Fixes nötig für Korrektheit. Kein Scope-Creep.

## Issues Encountered

- WASM-Build-Gate: `lld`-Linker auf NixOS nicht im PATH — `nix develop --command cargo build --target wasm32-unknown-unknown` löst das (per CLAUDE.local.md dokumentiert)

## Next Phase Readiness

Plan 02 kann direkt auf den fertigen Interfaces aufbauen:
- `api::book_slot_with_conflict_check` + `loader::register_user_to_slot_with_conflict_check` — fertig
- `component::{WarningList, WarningsList}` mit `person_name` + `suppress_header` — fertig
- 7 i18n-Keys (Header Singular/Plural, Confirm, Cancel, 3 Item-Texte) — fertig
- Kein Blocker für Plan 02

## Known Stubs

Keine — alle implementierten Funktionen sind vollständig verdrahtet. Die neuen Loader/API-Funktionen werden erst in Plan 02 in `shiftplan.rs` konsumiert, sind aber fertig implementiert.

## Threat Flags

Keine neuen unbekannten Security-Surfaces. Die Threat-Analyse in Plan-01-PLAN.md deckt alle berührten Boundaries ab.

---

## Self-Check: PASSED

- `shifty-dioxus/src/component/warning_list.rs` — FOUND
- `shifty-dioxus/src/api.rs` enthält `book_slot_with_conflict_check` — FOUND
- `shifty-dioxus/src/loader.rs` enthält `register_user_to_slot_with_conflict_check` — FOUND
- Commit `713ac3c7` (i18n keys) — FOUND
- Commit `4d9540d8` (warning_list.rs) — FOUND
- Commit `5062800b` (api + loader + absences.rs) — FOUND
- 551 tests grün — VERIFIED
- WASM build exit 0 — VERIFIED

---
*Phase: 09-booking-flow-reverse-warnings-copy-week*
*Completed: 2026-06-12*
