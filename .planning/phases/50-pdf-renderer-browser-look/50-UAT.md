---
status: testing
phase: 50-pdf-renderer-browser-look
source: [50-VERIFICATION.md]
started: 2026-07-03T23:35:00Z
updated: 2026-07-03T23:35:00Z
---

## Current Test

number: 1
name: Visueller PDF-Layout-Check via Phase-49-Download-Button (D-50-17)
expected: |
  1. Backend + Frontend laufen (`cargo run` + `dx serve --hot-reload`).
  2. Navigation zur Schichtplan-Seite, Auswahl einer Woche mit Slots (WeekStatus ∈ {Planned, Locked}).
  3. Klick auf den PDF-Download-Button (aus Phase 49) lädt eine PDF-Datei
     `schichtplan-{JJJJ}-KW{NN}.pdf`.
  4. PDF im PDF-Viewer öffnen und visuell prüfen:
     - Landscape A4 (quer)
     - Kopfzeile oben-links: `Schichtplan KW {NN} ({JJJJ})` (bold)
     - Timestamp oben-rechts: `Erstellt am DD.MM.YYYY HH:MM Uhr` (~9pt normal)
     - Wochentag-Spalten: Mo–Sa (6 Spalten) wenn kein Sonntag-Slot,
       Mo–So (7 Spalten) wenn mindestens ein Sonntag-Slot existiert
     - Slots pro Tages-Spalte als sichtbare rechteckige Boxen mit dünnem
       schwarzem Rahmen (~0.3–0.5pt), kein Fill
     - Uhrzeit-Label pro Slot-Box: `08:00 - 12:00` (Format mit Hyphen)
     - Sales-Person-Namen in Slot-Box: alphabetisch case-insensitive,
       ein Name pro Zeile, Plain-Text (kein Chip-Rahmen, keine Farben)
     - Freiwillige (`is_paid == Some(false)`): Namen enden mit ` (freiwillig)`
     - Overflow-Handling bei zu vielen Slots: `+ N weitere` am unteren Rand
       der letzten darstellbaren Slot-Box
awaiting: user response

## Tests

### 1. Visueller PDF-Layout-Check via Phase-49-Download-Button (D-50-17)

expected: |
  PDF-Rendering entspricht sichtbar der Browser-Wochenansicht:
  - Landscape A4
  - Kopfzeile `Schichtplan KW {NN} ({JJJJ})` oben-links
  - Timestamp `Erstellt am DD.MM.YYYY HH:MM Uhr` oben-rechts sichtbar
  - Slots als sichtbare Boxen mit Rahmen in den Tages-Spalten
  - Uhrzeit-Labels `HH:MM - HH:MM` in jeder Slot-Box
  - Sales-Person-Namen alphabetisch in der Slot-Box, Plain-Text
  - Freiwillige mit ` (freiwillig)`-Suffix
  - Sonntag-Spalte nur bei Sonntag-Slot
result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
