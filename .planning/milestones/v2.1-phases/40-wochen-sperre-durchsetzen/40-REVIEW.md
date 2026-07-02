---
phase: 40-wochen-sperre-durchsetzen
reviewed: 2026-07-02T05:26:55Z
depth: deep
files_reviewed: 15
files_reviewed_list:
  - rest/src/booking.rs
  - rest/src/lib.rs
  - rest/src/shiftplan_edit.rs
  - service/src/lib.rs
  - service/src/shiftplan_edit.rs
  - service_impl/src/shiftplan_edit.rs
  - service_impl/src/test/mod.rs
  - service_impl/src/test/shiftplan_edit.rs
  - service_impl/src/test/shiftplan_edit_lock.rs
  - shifty-dioxus/src/i18n/cs.rs
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/mod.rs
  - shifty-dioxus/src/page/shiftplan.rs
  - shifty_bin/src/main.rs
findings:
  critical: 1
  warning: 2
  info: 1
  total: 4
status: issues_found
---

# Phase 40: Code Review — Wochen-Sperre-Enforcement

**Reviewed:** 2026-07-02T05:26:55Z
**Depth:** deep
**Files Reviewed:** 15
**Status:** issues_found

## Zusammenfassung

Phase 40 implementiert das Wochen-Sperr-Gate auf allen 6 Schreibpfaden von
`ShiftplanEditService`. Der strukturelle Ansatz (TOCTOU-sicherer In-Transaction-
Read, get-before-lock-check-before-delete-Reihenfolge in `delete_booking`,
HTTP-423-Mapping) ist korrekt. Der 17-Test-Matrix-Ansatz ist solide. Es gibt
jedoch einen sicherheitskritischen Bypas-Privileg-Mismatch in einem der sechs
Pfade sowie zwei Qualitätsmängel.

---

## Critical Issues

### CR-01: Bypass-Privileg-Mismatch in `book_slot_with_conflict_check`

**File:** `service_impl/src/shiftplan_edit.rs:593-601`

**Issue:** Der Lock-Gate in `book_slot_with_conflict_check` wird übersprungen,
wenn der Aufrufer das `shiftplanner`-Privileg hält (`SHIFTPLANNER_PRIVILEGE =
"shiftplanner"`). Alle anderen fünf Schreibpfade (`modify_slot`,
`modify_slot_single_week`, `remove_slot`, `copy_week`, `delete_booking`)
delegieren an `assert_week_not_locked`, das intern `shiftplan.edit` als Bypass-
Bedingung prüft.

Das sind zwei **separate Datenbankrollen**: `shiftplanner` (`role_privilege`:
shiftplanner→shiftplanner) und `shiftplan.edit` (`role_privilege`:
shiftplan.edit→shiftplan.edit), vergeben durch separate Migrationen
(`20240614075633` bzw. `20241118165756`). Ein Nutzer kann eine Rolle ohne die
andere haben.

**Konkrete Verletzung:**
- Design D-40-02: „Bypass: Hält der Aufrufer `shiftplan.edit`, darf er immer
  schreiben" — der Bypass soll `shiftplan.edit` sein.
- `assert_week_not_locked`-Docstring: „Der Bypass prüft `shiftplan.edit`
  (NICHT `SHIFTPLANNER_PRIVILEGE`)" — explizit dokumentierte Abgrenzung.
- Frontend `is_shift_editor = has_privilege("shiftplan.edit")` — Buttons werden
  für `shiftplan.edit`-Halter in gesperrten Wochen sichtbar gelassen; für reine
  `shiftplanner`-Halter ausgeblendet.

**Folge:**
Ein Nutzer mit nur `shiftplanner` (ohne `shiftplan.edit`):
- Sieht im Frontend **keine** +/–-Buttons in einer gesperrten Woche (FE blendet
  aus, da `is_shift_editor = false`)
- Kann `POST /shiftplan-edit/booking` dennoch direkt aufrufen und **schreibt
  in der gesperrten Woche** (Bypass über `is_shiftplanner`)
- ABER `DELETE /booking/{id}` → `delete_booking` → `assert_week_not_locked` →
  prüft `shiftplan.edit` → **blockiert**

Ein reiner `shiftplanner` kann Buchungen **erstellen**, aber nicht **löschen**
in gesperrten Wochen — eine inkonsistente und unintendierte Asymmetrie.

Die Tests in `shiftplan_edit_lock.rs` decken diesen Defekt nicht auf, weil
`build_dependencies(true, false)` **alle** Permissions (sowohl `shiftplanner`
als auch `shiftplan.edit`) einheitlich genehmigt — der Unterschied zwischen
den beiden Rollen ist nicht testbar mit dem aktuellen Mock.

**Fix:**

In `book_slot_with_conflict_check` die Outer-Bypass-Bedingung auf `shiftplan.edit`
umstellen statt `shiftplanner` zu verwenden:

```rust
// Vorher (Zeile 593-601):
let is_shiftplanner = sp_perm.is_ok();
sp_perm.or(self_perm)?;

if !is_shiftplanner {
    self.assert_week_not_locked(...)
        .await?;
}

// Nachher: Lock-Gate an assert_week_not_locked delegieren (wie alle anderen Pfade).
// assert_week_not_locked prüft intern shiftplan.edit als Bypass.
sp_perm.or(self_perm)?;

self.assert_week_not_locked(
    booking.year,
    booking.calendar_week as u8,
    context.clone(),
    tx.clone(),
)
.await?;
```

Damit wird die Bypass-Logik einheitlich auf `shiftplan.edit` vereinheitlicht
(consistent mit D-40-02, allen anderen Pfaden und dem FE). Die bestehende
`is_shiftplanner`-Variable wird dann nur noch für den Paid-Limit-Hard-Block
(weiter unten, Zeile 624) benötigt — dort bleibt sie korrekt.

Zugehörige Tests: T-40-07 und T-40-08 müssen überarbeitet werden, da
`build_dependencies(false, true)` (Self-Booker, kein Shiftplanner) weiterhin
WeekLocked liefern muss, und ein neuer Testfall für den
„shiftplan.edit-Bypass in book_slot" hinzugefügt werden sollte.

---

## Warnings

### WR-01: T-40-03 fehlt im Test-Matrix-File

**File:** `service_impl/src/test/shiftplan_edit_lock.rs:1`

**Issue:** Der Datei-Kommentar lautet „IDs aus 40-VALIDATION.md (T-40-01..17)"
— es werden 17 Tests erwartet. Die Datei enthält aber nur 16: T-40-03 fehlt
vollständig. Die Nummerierung springt direkt von T-40-02 nach T-40-04.

T-40-03 wäre dem Muster nach `modify_slot_single_week + Locked + non-editor →
Forbidden` (symmetrisch zu T-40-01 für `modify_slot`). Dieser Case ist
zwar semantisch durch T-40-01 impliziert (beide Methoden fordern `shiftplan.edit`
vor dem Gate), aber der explizit versprochene Test im Validierungsdokument fehlt
und hinterlässt eine Lücke in der Rückverfolgbarkeit zum 40-VALIDATION.md.

**Fix:** T-40-03 analog zu T-40-01 hinzufügen:

```rust
/// T-40-03: modify_slot_single_week + Locked + non-editor → Err(Forbidden).
#[tokio::test]
async fn t_40_03_modify_slot_single_week_locked_non_editor_forbidden() {
    let mut deps = build_dependencies(false, false);
    set_week_status(&mut deps, WeekStatus::Locked);
    let service = deps.build_service();
    let result = service
        .modify_slot_single_week(&monday_slot(), 2026, 26, ().auth(), None)
        .await;
    test_forbidden(&result);
}
```

---

### WR-02: `WeekLockedError`-i18n-Key definiert aber nie im UI gerendert

**File:** `shifty-dioxus/src/page/shiftplan.rs:499-516`

**Issue:** Der i18n-Key `Key::WeekLockedError` ist in allen drei Locales
korrekt übersetzt (D-40-05 erfüllt) und wird im Locales-Test
`i18n_week_status_keys_present_in_all_locales` geprüft. Er wird jedoch an
**keiner Stelle** in der Anwendung tatsächlich gerendert.

Wenn der Server HTTP 423 zurückgibt (z.B. weil der Wochen-Status zwischen
Laden der Seite und Klick auf + geändert wurde), landet der Fehler im
generischen Else-Zweig des `AddUserToSlot`-Handlers:

```rust
Err(e) => {
    crate::error::error_handler(e);  // loggt nur in die Konsole
    update_shiftplan();
}
```

`error_handler` prüft nur auf 401 (Page-Reload); für alle anderen Status-Codes
(inkl. 423) passiert sichtbar nichts — kein Toast, kein Banner, keine
Nutzer-Rückmeldung.

Gleiches gilt für `RemoveUserFromSlot`, das `result_handler()` nutzt, welches
ebenfalls lediglich in die Konsole loggt.

**Praktischer Impact:** Da der FE-Button für Nicht-Editoren in gesperrten
Wochen ausgeblendet wird, ist ein 423 im Normalfall nicht erreichbar. Jedoch:
1. Es gibt ein Zeitfenster (Woche wird nach Laden der Seite gesperrt), in dem
   der User noch die Buttons sieht.
2. Direkte API-Calls (ohne FE) erhalten eine 423 ohne Feedback-Kontext auf der
   FE-Seite.

**Fix:** Im `AddUserToSlot`-Handler explizit auf 423 reagieren und
`Key::WeekLockedError` als sichtbares Banner setzen — analog zur 409-Behandlung
für `PaidLimitExceeded`:

```rust
Err(crate::error::ShiftyError::Reqwest(ref e))
    if e.status() == Some(reqwest::StatusCode::LOCKED) =>
{
    // 423 = Woche gesperrt — Buchung abgelehnt
    // Optional: block_error.set(Some(slot_id)) oder eigenes Signal für
    // WeekLocked-Banner. Mindestens: update_shiftplan() damit der
    // Button-Mode korrekt auf None springt nach Status-Refresh.
    week_status_service.send(WeekStatusAction::Load { year: *year.read(), week: *week.read() });
    update_shiftplan();
}
```

---

## Info

### IN-01: `DELETE /booking/{id}` ohne OpenAPI 423-Dokumentation

**File:** `rest/src/booking.rs:157-173`

**Issue:** Der `delete_booking`-Handler ruft jetzt
`shiftplan_edit_service().delete_booking()` auf, das HTTP 423 zurückgeben kann.
Der Handler hat keine `#[utoipa::path]`-Annotation und erscheint nicht im
OpenAPI-Schema. Die 423-Response ist damit für diesen Endpoint nicht
dokumentiert.

Hinweis: `edit_slot`, `delete_slot` und `add_vacation` in
`rest/src/shiftplan_edit.rs` haben ebenfalls keine utoipa-Annotationen — das
ist ein pre-existing Pattern, nicht spezifisch für Phase 40. Jedoch fügt Phase
40 einen neuen 423-Pfad durch diesen undokumentierten Endpoint ein.

**Fix:** Sofern der Endpoint in den OpenAPI-Scope aufgenommen werden soll,
`#[utoipa::path]` hinzufügen mit einer 423-Response-Zeile:

```rust
// In rest/src/booking.rs
#[utoipa::path(
    delete,
    path = "/{id}",
    tags = ["Booking"],
    responses(
        (status = 200, description = "Booking deleted"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Booking not found"),
        (status = 423, description = "Week is locked — changes are not possible"),
    ),
)]
pub async fn delete_booking<RestState: RestStateDef>(...) -> Response { ... }
```

---

## Nicht-Befunde (Sicherheits-Checkliste)

Die folgenden sicherheitskritischen Punkte aus dem Review-Auftrag wurden
überprüft und sind **korrekt implementiert**:

- **Alle 6 Schreibpfade haben den Gate-Aufruf:** `modify_slot` (Z. 69),
  `modify_slot_single_week` (Z. 226), `remove_slot` (Z. 166),
  `book_slot_with_conflict_check` (Z. 593–601), `copy_week_with_conflict_check`
  (Z. 804), `delete_booking` (Z. 867–873). ✓

- **TOCTOU-Schutz:** `assert_week_not_locked` nimmt `tx: Deps::Transaction`
  (bereits materialisiert) entgegen — der Lock-Read läuft immer in derselben
  Transaktion wie der Write. Kein separater Connection-Read. ✓

- **copy_week prüft Ziel-Woche:** `assert_week_not_locked(to_year,
  to_calendar_week, ...)` — Quelle wird nicht gesperrt. ✓

- **delete_booking-Reihenfolge:** `get` → `assert_week_not_locked` → `delete`.
  Das `get` liest year/week des Bookings VOR dem Gate-Check; danach erst Delete.
  T-40-15 und T-40-17 belegen diese Reihenfolge explizit. ✓

- **HTTP 423 (nicht 409):** `ServiceError::WeekLocked` → Status 423 in
  `rest/src/lib.rs:263-268`. Match ist exhaustiv (Rust-Compiler erzwingt alle
  Varianten). ✓

- **Kein Scope-Creep auf Absence/Unavailable:** `service_impl/src/absence.rs`
  und `service_impl/src/booking.rs` haben keine `assert_week_not_locked`-Aufrufe
  oder `WeekStatusService`-Dependency. ✓

- **DELETE-Handler routet durch ShiftplanEditService:** `rest/src/booking.rs:
  delete_booking` ruft `shiftplan_edit_service().delete_booking(...)` — nicht
  mehr direkt `booking_service().delete()`. ✓

---

_Reviewed: 2026-07-02T05:26:55Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
