---
status: testing
phase: 54-data-model-voluntary-stats
source: [54-VERIFICATION.md]
started: 2026-07-07T19:45:00Z
updated: 2026-07-07T19:45:00Z
---

## Current Test

number: 1
name: HR-Roundtrip Browser-Smoke — FE-Zeile sichtbar für HR
expected: |
  Unter dem bestehenden 'Volunteer Work'-Wert erscheinen genau 3 neue Zeilen:
  'Freiwillig Ø / Woche', 'Freiwillig Soll', 'Freiwillig Delta'
  (mit positivem Delta in neutraler Farbe, negativem Delta in rot/text-warn).
awaiting: user response

## Tests

### 1. HR-Roundtrip Browser-Smoke — FE-Zeile sichtbar für HR
expected: |
  Unter dem bestehenden 'Volunteer Work'-Wert erscheinen genau 3 neue Zeilen:
  'Freiwillig Ø / Woche', 'Freiwillig Soll', 'Freiwillig Delta'
  (mit positivem Delta in neutraler Farbe, negativem Delta in rot/text-warn).
why_human: |
  Browser-Rendering-Invariant; `dx serve` + Backend-Start nötig; CDP-Methode
  `get_page_text` + `find` gemäß memory `reference_dioxus_browser_verify_reports`.
result: [pending]

### 2. Non-HR-Roundtrip Browser-Smoke — FE-Zeile NICHT sichtbar
expected: |
  Non-HR-User sieht KEINE Freiwillig/Voluntary/Dobrovoln-Zeile im Employee-Detail-Report.
why_human: |
  Rollenseitige Sichtbarkeit nur im Browser prüfbar; Backend-Redaktion (alle Felder None) +
  Component-Guard (`rsx!{}`) greifen gemeinsam.
result: [pending]

### 3. cs-Locale Wortlaut Native-Check
expected: |
  Bei Locale=Cs erscheinen 'Dobrovolné prům. / týden', 'Dobrovolné plán', 'Dobrovolné rozdíl'
  (ASSUMED gemäß RESEARCH D.4 — kein nativer Tschechisch-Speaker hat das geprüft).
why_human: |
  i18n-Strings sind ASSUMED; native-Check wurde als Manual-Verify markiert
  (54-05-SUMMARY.md).
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
