---
quick_id: 260702-jql
title: Special-Day Duplikat-Hinweis nach Create ausblenden, erst bei Feld-Änderung wieder
mode: quick
wave: 1
autonomous: true
files_modified:
  - shifty-dioxus/src/page/settings.rs
must_haves:
  truths:
    - "Nach erfolgreichem Special-Day-Create ist der Inline-Hinweis 'existiert bereits' NICHT sichtbar, obwohl die Felder gefüllt bleiben und das Datum sich selbst matcht."
    - "Sobald der User eines der drei Formularfelder (Datum/Typ/Zeit) ändert, erscheint der Duplikat-Hinweis wieder (falls dann tatsächlich ein Duplikat vorliegt)."
    - "Die Sichtbarkeitsregel ist als reine, unit-getestete Funktion gekapselt."
  artifacts:
    - "should_show_duplicate_hint(is_duplicate, suppressed) -> bool (pure fn) + Unit-Tests in settings.rs"
    - "sd_dup_hint_suppressed-Signal in der SettingsPage-Komponente"
  key_links:
    - "Create-Success-Arm (settings.rs ~596-601) setzt sd_dup_hint_suppressed = true"
    - "on_change-Handler der 3 Felder (settings.rs ~770/795/815) setzen sd_dup_hint_suppressed = false"
    - "Render-Gate (settings.rs ~833) nutzt should_show_duplicate_hint(sd_is_duplicate, sd_dup_hint_suppressed())"
---

# Quick Task 260702-jql: Special-Day Duplikat-Hinweis nach Create unterdrücken

## Kontext / Problem

Seit Phase 42 (D-42-01) bleiben die Formularfelder nach erfolgreichem Anlegen gefüllt, damit
der „Anlegen"-Button aktiv bleibt. Nebeneffekt: der gerade angelegte Feiertag matcht sich
selbst → der Inline-Hinweis „existiert bereits" (`sd_is_duplicate`, gerendert bei
`settings.rs:833`) erscheint sofort nach dem Create. Gewünscht: Hinweis nach Create
unterdrücken, erst bei der nächsten echten Feld-Änderung wieder anzeigen. (Leichte, bewusste
Umkehr von D-42-03.)

## Task 1 (tdd): Reine Sichtbarkeits-Funktion + Unit-Tests

<read_first>
- shifty-dioxus/src/page/settings.rs (Zeilen ~64-180 bestehende pure Helper + Tests wie
  is_duplicate_special_day / sd_type_to_select_value; Zeile ~522-523 sd_is_duplicate; Zeile ~833 render)
- shifty-dioxus/CLAUDE.md (FE-Konventionen)
</read_first>

<action>
Extrahiere die Sichtbarkeitsregel des Duplikat-Hinweises als reine Funktion (analog zu den
bestehenden pure Helpern), z.B.:
`pub(crate) fn should_show_duplicate_hint(is_duplicate: bool, suppressed: bool) -> bool { is_duplicate && !suppressed }`
Schreibe im bestehenden `#[cfg(test)]`-Block Unit-Tests RED-first, dann GREEN:
- (is_duplicate=true, suppressed=false) → true
- (is_duplicate=true, suppressed=true) → false  (direkt nach Create)
- (is_duplicate=false, suppressed=false) → false
- (is_duplicate=false, suppressed=true) → false
</action>

<acceptance_criteria>
- `settings.rs` enthält `should_show_duplicate_hint(` als pub(crate)-fn
- Neue Unit-Tests decken alle 4 Kombinationen ab und sind grün: `cargo test -p shifty-dioxus duplicate_hint`
</acceptance_criteria>

<verify>
<automated>cd /home/neosam/programming/rust/projects/shifty/shifty-backend && nix develop -c cargo test -p shifty-dioxus duplicate_hint</automated>
</verify>

<done>Reine Funktion + 4 Unit-Tests grün.</done>

## Task 2 (execute): Suppress-Signal verdrahten

<read_first>
- shifty-dioxus/src/page/settings.rs (Signal-Deklarationen im Komponenten-Body; Create-Success-Arm
  ~596-601; on_change-Handler ~770 (Datum), ~789-795 (Typ), ~814-815 (Zeit); Render ~833)
</read_first>

<action>
- Neues Signal im SettingsPage-Body deklarieren: `let mut sd_dup_hint_suppressed = use_signal(|| false);`
  (bei den anderen sd_*-Signalen).
- Im Create-Success-Arm (nach dem `special_day_form_after_create`-Retain, ~596-601):
  `sd_dup_hint_suppressed.set(true);`
- In den `on_change`-Handlern der drei Felder (`sd_date_str` ~770, `sd_type` ~795, `sd_time_str` ~815):
  jeweils `sd_dup_hint_suppressed.set(false);` ergänzen (vor/nach dem bestehenden `.set(...)`).
- Render-Gate bei ~833 von `if sd_is_duplicate` auf
  `if should_show_duplicate_hint(sd_is_duplicate, sd_dup_hint_suppressed())` ändern.
- Controlled-Select (D-06/D-08) NICHT anfassen — Felder bleiben gefüllt, nur die Hinweis-Sichtbarkeit
  wird gesteuert. `sd_save_result`/„Gespeichert" unverändert (D-42-04).
</action>

<acceptance_criteria>
- Nach Create ist der Hinweis unterdrückt (Signal true); nach Feld-Änderung wieder aktiv (Signal false)
- `cargo build --target wasm32-unknown-unknown` (shifty-dioxus) warnungsfrei
- `cargo test -p shifty-dioxus` grün (nur der bekannte pre-existing i18n_impersonation-Fehler bleibt)
- Backend `cargo clippy --workspace -- -D warnings` unberührt grün (Backend nicht angefasst)
</acceptance_criteria>

<verify>
<automated>cd /home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus && nix develop -c cargo build --target wasm32-unknown-unknown</automated>
</verify>

<done>Signal verdrahtet, Render-Gate nutzt die pure fn, WASM-Build warnungsfrei.</done>

## Scope Guard
FE-only (`settings.rs`). Kein Backend, keine API/TO-Änderung, kein Snapshot-Bump, keine Migration,
keine neuen Deps, i18n unverändert. Kein Leeren der Felder (D-06/D-08 intakt).
