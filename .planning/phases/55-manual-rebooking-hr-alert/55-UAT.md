---
status: testing
phase: 55-manual-rebooking-hr-alert
source: [55-VERIFICATION.md]
started: 2026-07-24T00:00:00Z
updated: 2026-07-24T00:00:00Z
---

## Current Test

number: 1
name: F5 End-to-End — HR öffnet Banner → Suggestion-Modal → Approve → Banner verschwindet
expected: |
  Als HR die Employee-Overview aufrufen und einen gedeckelten Mitarbeiter mit negativem
  Stundenkonto und verfügbaren Freiwilligenstunden finden (Warn-Banner sichtbar,
  has_pending_rebooking=true). Klick auf den Banner öffnet das IST/DANN-Vorschlags-Modal.
  Nach Approve schließt das Modal, die Overview refetcht automatisch, und der Banner für
  diesen Mitarbeiter verschwindet — kein manuelles Dismiss nötig. Analog bei Reject.
awaiting: user response

## Tests

### 1. F5 End-to-End — Banner → Approve/Reject → Banner verschwindet
expected: Nach Approve/Reject zeigt die Employee-Overview keinen Banner mehr für diesen Mitarbeiter; das Modal schließt automatisch; der Overview-Refetch läuft ohne manuelles Dismiss (D-55-07, reaktive Laufzeit-Invariante — nur im Browser beweisbar).
result: [pending]

### 2. F3 Manuelle Umbuchung — TopBar-Trigger → Modal → Report zeigt Umbuchung
expected: Auf der Employee-Details-Seite öffnet der HR-gated F3-Trigger in der Action-Zone (nicht in der Voluntary-Stats-Zeile) das Manual-Modal. Nach erfolgreichem Submit (Richtung/Menge/Woche) schließt das Modal und der Report refetcht; die Umbuchung ist als rebooking_batch(kind=manual) sichtbar. Bei belegtem Slot (409) bleibt das Modal offen mit inline-Fehler.
result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
