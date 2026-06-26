---
status: partial
phase: 24-paid-capacity-enforcement-config
source: [24-VERIFICATION.md]
started: 2026-06-27T00:00:00.000Z
updated: 2026-06-27T16:30:00.000Z
---

## Current Test

Browser-UAT (Chrome, lokale Dev-Server) durchgeführt 2026-06-27. Dabei wurden zwei echte Bugs
gefunden und gefixt (siehe ## Gaps). Items 2/3/4 visuell bestätigt; Item 1 (#1) bewusst nicht
im Browser getestet (User-Entscheidung „#2 fertig, #1 weglassen") — Backend-Logik unit-getestet.

## Tests

### 1. Block-error placement (D-24-05 visual)
expected: User-Entscheidung 2026-06-27: Die Hard-Block-Meldung (`BookingBlockedPaidLimit`) soll **global oberhalb der Tabelle** stehen (nicht per-Slot, nicht in der WeekView).
result: implemented — `block_error`-Banner aus dem WeekView-Wrapper in den globalen RSX-Bereich oberhalb der Tabelle verschoben (direkt nach der Overage-Sektion, vor `ShiftplanTabBar`), als Fehler-Banner (`bg-bad-soft border-bad text-bad`). Native- + WASM-Build grün. Strukturell auf derselben Render-Ebene wie die Overage-Sektion (die nachweislich oberhalb der Tabelle rendert). Live-409-Visual NICHT erfasst: das synthetische Self-Booking-Setup (Nicht-shiftplanner) wurde schon am Permission-Gate mit `Unauthorized` abgewiesen (Mock-Session-/Zuweisungs-Detail), bevor der 409-Pfad erreicht wurde. Backend-409-Logik bleibt durch 4 Unit-Tests abgedeckt.

### 2. Settings-Toggle-Interaktion (D-24-06)
expected: Auf `/settings/` flippt der Paid-Limit-Toggle (hart/weich), `aria-pressed` ändert sich, „Saved."-Bestätigung erscheint; bei Fehler Revert + „Could not save setting."
result: PASS (nach Fix von Bug A). Klick → Label „Hard (enforced)", `aria-pressed=true`, „Saved."; Backend-Toggle=true. Zurück-Flip → „Soft (warnings only)", `aria-pressed=false`, „Saved."; Backend=false. Fehler-UX („Could not save setting." + Revert) ebenfalls bestätigt (vor dem Fix bzw. bei fehlendem toggle_admin).

### 3. Overage-Sektion rollenübergreifend sichtbar (D-24-03)
expected: Die persistente Overage-Warn-Sektion über dem Wochenplan erscheint für ALLE Rollen, sobald Slots über dem bezahlten Limit liegen; bei keiner Overage unsichtbar.
result: PASS (nach Fix von Bug B). Sektion rendert: „⚠️ Paid employee limit exceeded this week" / „Monday 10:00–11:00: 2/1 paid" (Platzhalter-Substitution korrekt). „Alle Rollen" nach Fix per API verifiziert: als Nicht-HR `shiftplanner` ist `current_paid_count=2` (vorher 0), per-Booking `is_paid` bleibt gegated (kein Leak).

### 4. Direkter URL-Zugriff Nicht-Admin auf /settings/
expected: Ein Nicht-Admin sieht „Not authorized." statt der Toggle-UI (Component-Guard).
result: PASS. Als `shiftplanner` (Nicht-Admin) zeigt `/settings/` „Not authorized."; als Admin die Toggle-UI.

## Summary

total: 4
passed: 3
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps

### Bug A — `/toggle` fehlte in der Dev-Proxy-Allowlist (GEFIXT)
status: resolved
Plan 24-04 führte die Backend-Pfad-Familie `/toggle` ein, ergänzte aber `shifty-dioxus/Dioxus.toml`
nicht. Folge im Dev: `GET /toggle/.../enabled` lieferte die SPA-`index.html` (→ Default „Soft"),
`PUT .../enable` → HTTP 405 (→ „Could not save setting."). Der Settings-Toggle war im Dev funktionslos.
fix: `[[web.proxy]] backend = "http://localhost:3000/toggle"` in Dioxus.toml ergänzt. Verifiziert:
PUT über :8080 jetzt 204; Toggle-Flip im Browser erfolgreich. (Erfordert dx-Neustart.)

### Bug B — Overage-Sektion nur für HR sichtbar, nicht „alle Rollen" (D-24-03) (GEFIXT)
status: resolved
`service_impl/src/shiftplan.rs` berechnete `current_paid_count` aus dem HR-gegateten
`sales_person.is_paid` (in `sales_person.rs` für Nicht-HR auf `None` gestrippt). Damit war der Count
für Nicht-HR-Rollen immer 0 → Overage-Sektion erschien nie (Widerspruch zu D-24-03 „alle Rollen").
fix: `current_paid_count` wird in der Shiftplan-View jetzt aus un-gegatetem
`get_all_paid(Authentication::Full)` gezählt; per-Booking `is_paid` bleibt gegated (kein Leak,
*wer* bezahlt ist). Regressionstest `test_current_paid_count_correct_for_non_hr_caller` ergänzt.
Backend-Gates grün (build/test/clippy). API-verifiziert: non-HR `current_paid_count=2` (vorher 0).

### Offen — #1 Inline-Block-Platzierung (nicht-blockierend)
status: open
Die 409-Inline-Meldung rendert unter der gesamten WeekView statt an der Slot-Zelle (siehe Test 1).
Nicht im Browser getestet; Entscheidung über Nachbesserung der Platzierung steht aus.
