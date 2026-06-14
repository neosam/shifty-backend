---
quick_id: 260613-qvg
title: Fix self-overlap absence banner i18n placeholder substitution
status: planned
date: 2026-06-13
---

# Quick Task 260613-qvg: Self-Overlap-Banner i18n-Substitution

## Problem

Beim Anlegen/Bearbeiten von Abwesenheiten zeigt der Self-Overlap-Fehlerbanner
die wörtlichen i18n-Platzhalter statt Werte:

> `{category}-Eintrag von {from} bis {to} überschneidet sich. Bitte Zeitraum oder Kategorie anpassen.`

**Root Cause:** Das Template `Key::AbsenceErrorSelfOverlapBody` enthält Platzhalter
`{category}/{from}/{to}`, aber `SelfOverlapBanner` (shifty-dioxus/src/page/absences.rs:695)
gibt es per `i18n.t(...)` **roh** aus. Der Substitutions-Mechanismus
`I18n::t_m(key, HashMap)` (shifty-dioxus/src/i18n/i18n.rs:67) existiert bereits, wird hier
aber nicht genutzt.

## Ansatz (gewählt vom User): Echte Substitution

category, from_date, to_date aus den Formular-Signalen des AbsenceModal in den
SelfOverlapBanner durchreichen und die Platzhalter via `t_m` füllen. Kategorie über
i18n-Key lokalisieren, Datum via `i18n.format_date()` locale-korrekt formatieren.

## Tasks

### Task 1 — SelfOverlapBanner substituiert Platzhalter
- **files:** `shifty-dioxus/src/page/absences.rs`
- **action:**
  1. `SelfOverlapBannerProps` um `category: AbsenceCategory`, `from: Option<time::Date>`,
     `to: Option<time::Date>` erweitern.
  2. In `SelfOverlapBanner` Kategorie → i18n-Key (Vacation/SickLeave/UnpaidLeave) mappen,
     `from`/`to` via `i18n.format_date()` formatieren, und Body via
     `i18n.t_m(Key::AbsenceErrorSelfOverlapBody, {category, from, to})` rendern.
  3. Call-Site (~absences.rs:1088) um `category: *category.read()`, `from: parsed_from`,
     `to: parsed_to` ergänzen.
- **verify:** `cargo build --target wasm32-unknown-unknown` in shifty-dioxus/ grün.
- **done:** Banner zeigt lokalisierte Kategorie + formatierte Daten, keine `{}`-Platzhalter mehr.

### Task 2 — Unit-Test für Substitution
- **files:** `shifty-dioxus/src/i18n/mod.rs` (tests-Modul)
- **action:** Test über alle Locales (En/De/Cs): `t_m(Key::AbsenceErrorSelfOverlapBody, …)`
  enthält keinen `{`/`}`-Platzhalter mehr und setzt category/from/to ein.
- **verify:** `cargo test` in shifty-dioxus/ grün.
- **done:** Regressionstest schützt gegen erneutes rohes Template.

## must_haves

- truths:
  - Banner-Text enthält keine wörtlichen `{category}`/`{from}`/`{to}` mehr.
  - Kategorie wird lokalisiert (i18n-Key, nicht Enum-Rohwert).
  - Datum wird via `format_date()` locale-korrekt formatiert.
- artifacts:
  - Erweiterte `SelfOverlapBannerProps` + `t_m`-Aufruf in absences.rs.
  - i18n-Unit-Test in mod.rs.
- key_links:
  - shifty-dioxus/src/page/absences.rs (SelfOverlapBanner + Call-Site)
  - shifty-dioxus/src/i18n/i18n.rs (t_m)
  - shifty-dioxus/src/i18n/mod.rs (Test)
