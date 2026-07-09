---
status: complete
phase: 54-data-model-voluntary-stats
source: [54-VERIFICATION.md]
started: 2026-07-07T19:45:00Z
updated: 2026-07-07T20:25:00Z
---

## Current Test

[testing complete]

## Tests

### 1. HR-Roundtrip Browser-Smoke — FE-Zeile sichtbar für HR
expected: |
  Unter dem bestehenden 'Volunteer Work'-Wert erscheinen genau 3 neue Zeilen:
  'Freiwillig Ø / Woche', 'Freiwillig Soll', 'Freiwillig Delta'
  (mit positivem Delta in neutraler Farbe, negativem Delta in rot/text-warn).
why_human: |
  Browser-Rendering-Invariant; `dx serve` + Backend-Start nötig; CDP-Methode
  `get_page_text` + `find` gemäß memory `reference_dioxus_browser_verify_reports`.
result: issue
reported: "Stimmt leider komplett gar nicht. Da hat ein Freiwilliger 177 Freiwillige Sollstunden. Ich kann mir gar nicht erklären, wie das zustande kommt. Außerdem wird Freiwillig und Ehrenamt aktuell vermischt. Das ist das selbe und sollte alles Ehrenamt genannt werden."
severity: blocker

### 2. Non-HR-Roundtrip Browser-Smoke — FE-Zeile NICHT sichtbar
expected: |
  Non-HR-User sieht KEINE Freiwillig/Voluntary/Dobrovoln-Zeile im Employee-Detail-Report.
why_human: |
  Rollenseitige Sichtbarkeit nur im Browser prüfbar; Backend-Redaktion (alle Felder None) +
  Component-Guard (`rsx!{}`) greifen gemeinsam.
result: pass

### 3. cs-Locale Wortlaut Native-Check
expected: |
  Bei Locale=Cs erscheinen 'Dobrovolné prům. / týden', 'Dobrovolné plán', 'Dobrovolné rozdíl'
  (ASSUMED gemäß RESEARCH D.4 — kein nativer Tschechisch-Speaker hat das geprüft).
why_human: |
  i18n-Strings sind ASSUMED; native-Check wurde als Manual-Verify markiert
  (54-05-SUMMARY.md).
result: skipped
reason: "User: Das wird schon passen. Das brauchen wir nicht prüfen. — cs-Strings bleiben ASSUMED und werden bei Terminologie-Fix (Ehrenamt) ohnehin mit angefasst."

## Summary

total: 3
passed: 1
issues: 1
pending: 0
skipped: 1
blocked: 0

## Gaps

- truth: "Freiwillig-Soll-Stunden werden für Freiwillige korrekt berechnet"
  status: failed
  reason: "User reported: Da hat ein Freiwilliger 177 Freiwillige Sollstunden. Ich kann mir gar nicht erklären, wie das zustande kommt."
  severity: blocker
  test: 1
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""

- truth: "Terminologie ist konsistent — Freiwillig und Ehrenamt sind dasselbe und sollten durchgängig 'Ehrenamt' heißen"
  status: failed
  reason: "User reported: Außerdem wird Freiwillig und Ehrenamt aktuell vermischt. Das ist das selbe und sollte alles Ehrenamt genannt werden."
  severity: major
  test: 1
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
