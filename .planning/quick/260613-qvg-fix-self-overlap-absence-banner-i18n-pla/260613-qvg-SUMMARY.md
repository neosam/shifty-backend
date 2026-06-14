---
quick_id: 260613-qvg
title: Fix self-overlap absence banner i18n placeholder substitution
status: complete
date: 2026-06-13
---

# Quick Task 260613-qvg — Summary

## Was war kaputt

Der Self-Overlap-Fehlerbanner beim Anlegen/Bearbeiten von Abwesenheiten zeigte
die wörtlichen i18n-Platzhalter:

> `{category}-Eintrag von {from} bis {to} überschneidet sich. Bitte Zeitraum oder Kategorie anpassen.`

**Root Cause:** `SelfOverlapBanner` (`shifty-dioxus/src/page/absences.rs`) gab das
Template `Key::AbsenceErrorSelfOverlapBody` per `i18n.t(...)` roh aus. Der bereits
vorhandene Substitutions-Mechanismus `I18n::t_m(key, HashMap)` (`i18n/i18n.rs:67`)
wurde nicht genutzt, und die konkreten Werte wurden nicht in den Banner durchgereicht.

## Fix (echte Substitution)

- **`SelfOverlapBannerProps`** um `category: AbsenceCategory`, `from: Option<time::Date>`,
  `to: Option<time::Date>` erweitert.
- **`SelfOverlapBanner`**: Kategorie → i18n-Key (Vacation/SickLeave/UnpaidLeave) gemappt
  und lokalisiert; `from`/`to` via `i18n.format_date()` locale-korrekt formatiert; Body via
  `i18n.t_m(Key::AbsenceErrorSelfOverlapBody, {category, from, to})` gerendert. Unparseable
  Datumswerte fallen auf `"?"` zurück. Raw-Payload bleibt als Zusatzkontext darunter.
- **Call-Site** (AbsenceModal): `category: *category.read()`, `from: parsed_from`,
  `to: parsed_to` ergänzt (parsed_from/parsed_to sind `Copy`, bereits vorhanden).
- **Import:** `std::collections::HashMap` ergänzt.

## Tests

- Neuer Regressionstest `i18n_self_overlap_body_substitutes_placeholders_in_all_locales`
  (`shifty-dioxus/src/i18n/mod.rs`): prüft über En/De/Cs, dass das Template noch
  Platzhalter hat (kein vacuous pass) und dass `t_m` sie vollständig ersetzt — keine
  `{`/`}` mehr, category/from/to eingesetzt.

## Verifikation

- `cargo test` (shifty-dioxus): **596 passed, 0 failed**.
- `nix develop -c cargo build --target wasm32-unknown-unknown`: **Finished** (kompiliert sauber).
  (Direkter Host-Build scheitert an fehlendem `lld`-Linker — reines Umgebungs-Gap, via
  `nix develop` behoben.)

## Geänderte Dateien

- `shifty-dioxus/src/page/absences.rs` — Props, Banner-Substitution, Call-Site, HashMap-Import
- `shifty-dioxus/src/i18n/mod.rs` — Regressionstest

## VCS

Noch nicht committet — jj-Repo, Commits erfolgen manuell durch den User.
