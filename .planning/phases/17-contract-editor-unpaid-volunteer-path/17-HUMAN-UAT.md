---
status: passed
phase: 17-contract-editor-unpaid-volunteer-path
source: [17-VERIFICATION.md]
started: 2026-06-24
updated: 2026-06-24
verified_by: claude-chrome-browser-test
---

## Current Test

[all complete]

## Tests

### 1. Contract-Editor Live-Round-Trip (CVC-09)
expected: Modal für eine gedeckelte Person öffnen, „Freiwillige Zusage (h)" auf 3.5 setzen, speichern, erneut öffnen → Wert ist 3.5 (nicht 0).
result: passed — Tom Bauer, Cap aktiviert → Feld „Voluntary Commitment (h)" erscheint mit 0; auf 3.5 gesetzt, gespeichert (Modal schloss ohne Fehler), erneut geöffnet → Wert ist 3.5. Backend-Round-Trip durch beide TryFrom-Richtungen bestätigt.

### 2. D-01 rein-freiwilliger Branch (CVC-09)
expected: Person mit `cap=false, expected_hours=0` → das committed-Feld ist sichtbar.
result: passed — Cap deaktiviert + Expected Hours auf 0 gesetzt → „Voluntary Commitment (h)" bleibt sichtbar. Gegenprobe: cap=false + expected_hours=10 → Feld korrekt versteckt.

### 3. „alle"-Toggle Mitarbeiteransicht (CVC-10)
expected: Default zeigt nur bezahlte Mitarbeiter; Toggle „alle" deckt zusätzlich rein unbezahlte nicht-inaktive Freiwillige auf; inaktive Personen bleiben immer ausgeblendet.
result: passed — Default zeigt 3 bezahlte (Anna Müller, Max Schmidt, Sarah Fischer); „all"-Toggle deckt zusätzlich Tom Bauer (Volunteer, 0.0/0, is_paid=false) auf. Inaktiv-Ausschluss durch Unit-Test `filter_inactive_always_hidden` abgesichert.

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
