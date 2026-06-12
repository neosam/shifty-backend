---
phase: 09-booking-flow-reverse-warnings-copy-week
plan: "02"
subsystem: ui
tags: [dioxus, wasm, booking, conflict-dialog, rollback, copy-week-cleanup, i18n]

requires:
  - phase: 09-booking-flow-reverse-warnings-copy-week
    plan: "01"
    provides: "WarningsList, WarningList(suppress_header), api::book_slot_with_conflict_check, loader::register_user_to_slot_with_conflict_check, 7 i18n Keys"

provides:
  - "shiftplan.rs: RollbackBooking(Uuid)-Action + AddUserToSlot conflict-aware flow"
  - "shiftplan.rs: BookingWarningDialog (optimistic-create + rollback, alle Close-Pfade gesichert)"
  - "shiftplan.rs: WarningList mit suppress_header: true (kein Doppel-Header)"
  - "Dead Copy-Week frontend code vollständig entfernt (api, loader, shiftplan, i18n)"
  - "no_copy_week_in_frontend_source: Reintroduktions-Guard in i18n/mod.rs"
  - "ROADMAP + REQUIREMENTS doc-synced (FUI-A-05 Done, FUI-A-06 Dropped bestätigt)"

affects:
  - phase: 09-booking-flow-reverse-warnings-copy-week
    notes: "Phase vollständig abgeschlossen; UAT-Checkpoint folgt"

tech-stack:
  added: []
  patterns:
    - "RollbackBooking(Uuid) als Daten-Carrier — booking_id im Action statt Signal (Pitfall 5 Guard)"
    - "SHIFTPLAN_REFRESH.write() += 1 für sync-fähiges Reload aus RSX-Handler ohne Coroutine"
    - "pending_warnings + pending_rollback_id Signals für Dialog-State (nicht in Coroutine gehalten)"
    - "Alle Dialog-Close-Pfade (X/ESC/Backdrop) via on_close → RollbackBooking (Pitfall 2 Guard)"
    - "Split-String-Literals in self-test um grep-Selbst-Match zu vermeiden"

key-files:
  created: []
  modified:
    - shifty-dioxus/src/page/shiftplan.rs
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/cs.rs
    - .planning/ROADMAP.md
    - .planning/REQUIREMENTS.md

key-decisions:
  - "SHIFTPLAN_REFRESH statt update_shiftplan()-Closure für Trotzdem-buchen-Reload — Closure ist im Coroutine-Scope, RSX-Handler braucht Sync-Pfad"
  - "ShiftyError::Reqwest(e) Wrap für error_handler im RollbackBooking-Handler — api::remove_booking gibt reqwest::Error, error_handler erwartet ShiftyError"
  - "Split-String-Literals in no_copy_week_in_frontend_source — vermeidet grep-Selbst-Match bei include_str!-Scan"
  - "booking_dialog_warning_list_suppresses_internal_header SSR-Test in shiftplan.rs statt warning_list.rs — deckt den konkreten Verwendungskontext ab"

requirements-completed: [FUI-A-05]

duration: 50min
completed: 2026-06-12
---

# Phase 09 Plan 02: Shiftplan-Wiring + Copy-Week-Cleanup Summary

**AddUserToSlot auf conflict-aware `POST /shiftplan-edit/booking` umgestellt; BookingWarningDialog mit RollbackBooking-Semantik integriert (optimistic-create + alle Close-Pfade als Rollback-Guard); toter Copy-Week-Frontend-Code vollständig entfernt; ROADMAP/REQUIREMENTS doc-synced; alle Regression-Gates grün**

## Performance

- **Duration:** ca. 50 min
- **Started:** 2026-06-12T08:00:00Z
- **Completed:** 2026-06-12T08:50:00Z
- **Tasks:** 3 (alle abgeschlossen; Task 4 = Checkpoint)
- **Files modified:** 9

## Accomplishments

- `shiftplan.rs`: `AddUserToSlot` nutzt `register_user_to_slot_with_conflict_check`; leere warnings → direkter Reload, non-empty warnings → Dialog; 403 silent-swallow (D-13), 422 surfaced
- `RollbackBooking(Uuid)` Action: booking_id als Daten-Carrier; DELETE via `api::remove_booking` + `error_handler` + Reload auch bei Fehler (D-04)
- `BookingWarningDialog`: `Dialog { title, footer }` mit Singular/Plural-Header, `WarningList { suppress_header: true }` (kein Doppel-Header), alle Close-Pfade → `cr.send(RollbackBooking(rollback_id))`, "Trotzdem buchen" cleared pending + `SHIFTPLAN_REFRESH.write() += 1`
- Dead Code entfernt: `api::copy_week`, `loader::copy_from_previous_week`, `ShiftPlanAction::CopyFromPreviousWeek`, `Key::ShiftplanTakeLastWeek` (en/de/cs)
- `no_copy_week_in_frontend_source` Guard in `i18n/mod.rs` mit split-Strings
- 2 neue Tests: `booking_dialog_warning_list_suppresses_internal_header` (SSR) + `no_copy_week_in_frontend_source`
- REQUIREMENTS: FUI-A-05 → `[x] Done`; ROADMAP: Phase 9 → `[x]`, 2/2 Complete
- 553 Tests grün; WASM-Build-Gate exit 0; `cargo check --workspace` clean

## Task Commits

1. **Task 1: AddUserToSlot + RollbackBooking + Dialog** — `403f5845`
2. **Task 2: Copy-Week dead code + self-test** — `760c1fe1`
3. **Task 3: Doc-sync + Regression-Gates** — `8dce15da`

## Files Created/Modified

- `shifty-dioxus/src/page/shiftplan.rs` — RollbackBooking-Action, Dialog-State Signals, neuer AddUserToSlot-Handler, RollbackBooking-Handler, BookingWarningDialog RSX, SSR-Test; CopyFromPreviousWeek-Variant/Handler entfernt; _take_last_week_str entfernt
- `shifty-dioxus/src/api.rs` — `copy_week` entfernt
- `shifty-dioxus/src/loader.rs` — `copy_from_previous_week` entfernt
- `shifty-dioxus/src/i18n/mod.rs` — `ShiftplanTakeLastWeek` entfernt; `no_copy_week_in_frontend_source`-Test hinzugefügt
- `shifty-dioxus/src/i18n/en.rs` — `ShiftplanTakeLastWeek`-Übersetzung entfernt
- `shifty-dioxus/src/i18n/de.rs` — `ShiftplanTakeLastWeek`-Übersetzung entfernt
- `shifty-dioxus/src/i18n/cs.rs` — `ShiftplanTakeLastWeek`-Übersetzung entfernt
- `.planning/ROADMAP.md` — Phase 9 + 09-02-PLAN als `[x]`; Progress-Tabelle 2/2 Complete
- `.planning/REQUIREMENTS.md` — FUI-A-05 `[x] Done`; Coverage-Tabelle aktualisiert

## Decisions Made

- **SHIFTPLAN_REFRESH als Reload-Trigger:** `update_shiftplan()` ist im Coroutine-Scope definiert und nicht aus RSX-Handlern aufzurufen. `*SHIFTPLAN_REFRESH.write() += 1` ist das bestehende Pattern für Sync-Reloads aus RSX-Handlers.
- **ShiftyError::Reqwest(e)-Wrap:** `api::remove_booking` gibt `reqwest::Error` zurück; `error_handler` erwartet `ShiftyError`. Wrap nötig, analog zu anderen Stellen.
- **Split-Strings im self-test:** `include_str!` + `.contains()` prüft den vollen Dateitext inkl. Test-Code. Ohne Split würde der Test sich selbst matchen.
- **SSR-Test in shiftplan.rs:** Der Test prüft den genauen Verwendungskontext (`suppress_header: true` im Dialog-Kontext). Die Komponente selbst hat eigene Tests in `warning_list.rs`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] reqwest::Error statt ShiftyError für error_handler**
- **Found during:** Task 1 (RollbackBooking-Handler)
- **Issue:** `api::remove_booking` gibt `reqwest::Error` zurück; `crate::error::error_handler` erwartet `ShiftyError`
- **Fix:** `crate::error::error_handler(crate::error::ShiftyError::Reqwest(e))` wrap
- **Files modified:** `shiftplan.rs`
- **Verification:** `cargo check` clean

**2. [Rule 1 - Bug] BookingOnAbsenceDay falsche Felder im SSR-Test**
- **Found during:** Task 1 (SSR-Test schreiben)
- **Issue:** Test enthielt `sales_person_id`-Feld, das in `rest_types::WarningTO::BookingOnAbsenceDay` nicht existiert; korrekte Felder sind `booking_id, date, absence_id, category`
- **Fix:** Test-Struct-Literal korrigiert
- **Files modified:** `shiftplan.rs`
- **Verification:** `cargo test` exit 0

**3. [Rule 1 - Bug] Self-matching im no_copy_week-Test**
- **Found during:** Task 2 (Reintroduktions-Guard)
- **Issue:** `include_str!("mod.rs").contains("ShiftplanTakeLastWeek")` trifft auf die Assertion-Strings im Test selbst
- **Fix:** Split-String-Literals (`["Shiftplan", "TakeLastWeek"].concat()`) und Assertion-Texte ohne Symbol-Strings
- **Files modified:** `i18n/mod.rs`
- **Verification:** `cargo test no_copy_week_in_frontend_source` exit 0

---

**Total deviations:** 3 auto-fixed (alle Rule 1 - Bug)
**Impact:** Alle nötig für Korrektheit. Kein Scope-Creep.

## Threat Surface Scan

Keine neuen unbekannten Security-Surfaces. Die Threat-Analyse in 09-02-PLAN.md deckt alle berührten Boundaries ab (T-09-04 bis T-09-07).

## Known Stubs

Keine. Alle implementierten Funktionen sind vollständig verdrahtet.

---

## Self-Check: PASSED

- `shifty-dioxus/src/page/shiftplan.rs` enthält `RollbackBooking` — FOUND (4 Treffer)
- `shifty-dioxus/src/page/shiftplan.rs` enthält `register_user_to_slot_with_conflict_check` — FOUND (1 Treffer)
- `shifty-dioxus/src/page/shiftplan.rs` enthält `suppress_header: true` — FOUND (1 Treffer)
- `shifty-dioxus/src/api.rs` enthält KEIN `fn copy_week` — VERIFIED (0 Treffer)
- `shifty-dioxus/src/loader.rs` enthält KEIN `copy_from_previous_week` — VERIFIED (0 Treffer)
- `shifty-dioxus/src/i18n/mod.rs` enthält KEIN `ShiftplanTakeLastWeek` (non-test) — VERIFIED (0 Treffer)
- `rest/src/shiftplan_edit.rs` enthält `copy_week_with_conflict_check` — FOUND (Backend untouched)
- Commit `403f5845` (Task 1) — FOUND
- Commit `760c1fe1` (Task 2) — FOUND
- Commit `8dce15da` (Task 3) — FOUND
- 553 Tests grün — VERIFIED
- WASM build exit 0 — VERIFIED
- `cargo check --workspace` clean — VERIFIED

---
*Phase: 09-booking-flow-reverse-warnings-copy-week*
*Completed: 2026-06-12*
