---
created: 2026-07-02T12:00:00.000Z
title: i18n-Impersonation-Key Test-Mismatch (vorbestehend, prä-v2.1)
area: frontend / i18n
resolves_phase: 46
files:
  - shifty-dioxus/src/i18n/mod.rs:1578
---

# Todo: i18n-Impersonation-Key Test-Mismatch (vorbestehend, prä-v2.1)

**Erfasst:** 2026-07-02 (beim v2.1-Milestone-Close)
**Herkunft:** Phase 37-02 (v1.11), Commit `83a0d91` — NICHT von v2.1 (39–42) eingeführt.
**Status:** deferred — braucht Produkt-Copy-Entscheidung des Users

## Problem

`cargo test -p shifty-dioxus i18n_impersonation_keys_match_german_reference` schlägt fehl
(`shifty-dioxus/src/i18n/mod.rs:1578`):

```
assertion `left == right` failed
  left:  "🥸 Agieren"                 (aktueller De-Wert)
  right: "Als diese Person agieren"   (Test-Referenzwert)
```

Der Test prüft, dass ein De-Impersonation-Label dem hinterlegten „German reference" entspricht.
Aktuell divergieren beide. Es ist unklar, welcher String kanonisch ist:
- Wurde das Label bewusst auf das kürzere „🥸 Agieren" geändert (dann Test-Referenz nachziehen)?
- Oder ist „Als diese Person agieren" gewollt (dann De-Übersetzung zurücksetzen)?

## Warum nicht im v2.1-Run gefixt

Out-of-scope für v2.1 (reiner v1.11-Carry-over). Blindes Ändern von Test **oder** Übersetzung
könnte eine bewusste v1.11-Copy-Entscheidung überschreiben → bewusst dem User überlassen.
Alle vier v2.1-Phasen (39–42) haben diesen Fehler als vorbestehend/unrelated korrekt identifiziert
und ihre eigenen FE-Tests grün gehalten (nur dieser eine Test ist rot).

## Fix (wenn entschieden)

Eine Zeile: entweder die Test-Referenz in `i18n/mod.rs` (~1578) auf „🥸 Agieren" aktualisieren,
oder den De-Key auf „Als diese Person agieren" zurücksetzen — je nach gewünschter UI-Copy.
Danach `cargo test -p shifty-dioxus i18n` grün.
