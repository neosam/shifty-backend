# Phase 9 UAT-Feedback: Dialog→Banner Refactor

**Datum:** 2026-06-12
**Art:** Post-UAT-Anpassung (kein neuer Plan, UAT-getriebenes Refactoring)
**Commit:** `vvtlnpus` (0f2374ed)

## Zusammenfassung

Nach der UAT-Abnahme von Phase 9 (Booking-Conflict-Dialog) hat der Nutzer festgestellt, dass ein
modaler Bestaetigungs-Dialog den Arbeitsfluss unterbricht. Anforderung: Konflikte sollen als
**nicht-blockierender, abbrechbarer Warn-Banner** oben im Shiftplan angezeigt werden.
Buchungen werden immer angelegt und niemals zurueckgerollt.

## Geänderte Dateien

| Datei | Änderung |
|---|---|
| `shifty-dioxus/src/page/shiftplan.rs` | Dialog-Mechanismus durch Banner-Block ersetzt |
| `shifty-dioxus/src/i18n/mod.rs` | `BookingWarningDialogConfirm`/`Cancel`→`BookingWarningDismiss` |
| `shifty-dioxus/src/i18n/en.rs` | Neue Dismiss-Übersetzung (en) |
| `shifty-dioxus/src/i18n/de.rs` | Neue Dismiss-Übersetzung (de) |
| `shifty-dioxus/src/i18n/cs.rs` | Neue Dismiss-Übersetzung (cs) |

## Entfernte Logik

- `ShiftPlanAction::RollbackBooking(Uuid)` — Enum-Variant und Coroutine-Arm
- `pending_warnings: Signal<WarningsList>` und `pending_rollback_id: Signal<Option<Uuid>>`
- Dialog-Render-Block (Dialog, DialogVariant, Rollback-Buttons)
- `BookingWarningDialogConfirm` und `BookingWarningDialogCancel` i18n-Keys in allen 3 Locales

## Hinzugefügte Logik

- `booking_warnings: Signal<WarningsList>` — einzelnes Signal, ersetzt beide alten Signals
- `AddUserToSlot`: ruft immer `update_shiftplan()`, setzt `booking_warnings` bei Konflikten
- `NextWeek`/`PreviousWeek`: leeren `booking_warnings` beim Wochenwechsel
- Dismissibler Warn-Banner oberhalb des Shiftplan-Inhalts (unterhalb der Konflikte-Sektion)
- `BookingWarningDismiss` i18n-Key (en: "Dismiss warning", de: "Hinweis schließen", cs: "Zavřít upozornění")

## Tests

- `booking_dialog_warning_list_suppresses_internal_header` → ersetzt durch:
  - `booking_banner_present_when_warnings_non_empty` — Banner + Header sichtbar
  - `booking_banner_absent_when_warnings_empty` — kein Banner-Output
  - `booking_banner_source_contract` — Quelltext-Invarianten (kein RollbackBooking, kein DialogVariant, etc.)
- `i18n_booking_warning_keys_present_in_all_locales` — aktualisiert (Confirm/Cancel → Dismiss)
- `i18n_booking_warning_keys_match_german_reference` — aktualisiert

## Gates

- `cargo test`: 556 Tests, alle grün
- `cargo build --target wasm32-unknown-unknown`: Exit 0, nur Vor-UAT-Warnungen
- `cargo check --workspace` (Backend): Finished, clean

## Abweichungen von Phase-9-Original

| Entscheidung | Original (D-04) | Nach UAT |
|---|---|---|
| Buchungsverhalten bei Konflikt | Optimistic-Create + Rollback | Immer behalten, keine Rollback-Pfad |
| UI-Muster | Modaler Bestätigungs-Dialog | Dismissibler Top-Banner |
| Rollback-API-Aufruf | Ja (delete_booking) | Entfernt |
