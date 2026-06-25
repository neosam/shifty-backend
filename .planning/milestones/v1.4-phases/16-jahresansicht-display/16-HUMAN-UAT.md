---
status: partial
phase: 16-jahresansicht-display
source: [16-VERIFICATION.md, 16-VALIDATION.md]
started: 2026-06-24
updated: 2026-06-24
---

## Current Test

[awaiting human testing]

## Tests

### 1. Visuelle Drei-Farben-Stapelung des Charts
expected: In der Jahresansicht zeigt das Balken-Chart pro Woche drei gestapelte Segmente — Bezahlt (💰), Freiwillig zugesagt (🎯, `var(--good)`) und Freiwillig/Überschuss (🤝) — in der Reihenfolge paid → committed → surplus, ohne Lücken/Überlappung. Tooltip und Legende nennen alle drei Bänder. SSR-Tests pinnen die Styles, aber nicht die gerenderten Pixel.
result: [pending]

### 2. Czech-Strings sprachlich prüfen
expected: Die tschechischen Labels sind sprachlich korrekt: `Dobrovolně přislíbeno` (committed/Band 1), `Dobrovolné` (volunteer/Band 2), `Placené` (paid). Header: `Placené / Dobrovolně přislíbeno / Dobrovolné`. (A3, MEDIUM-confidence — Native-Speaker-Check empfohlen.)
result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
