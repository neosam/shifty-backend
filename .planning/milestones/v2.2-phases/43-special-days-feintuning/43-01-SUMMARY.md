---
phase: 43-special-days-feintuning
plan: 1
subsystem: frontend
tags:
  - frontend
  - special-days
  - i18n
  - bugfix
requires:
  - Phase 42 (Card-3 pure-fn extraction pattern D-42-05)
  - Phase 36 / SDF-01 (Backend create-or-replace-in-place für Special-Days)
provides:
  - "sd_year_after_create(date_str) — pure fn, Kalenderjahr-Semantik für Card-3 Jahres-Picker"
  - "Neuer replace-tauglicher Wording für SettingsSpecialDaysDuplicateHint in de/en/cs"
  - "5 neue Unit-Tests im settings::tests-Modul"
affects:
  - Card-3 Post-Create-UX (Jahres-Picker springt beim Jahreswechsel korrekt)
  - Duplicate-Hint Tonalität (informativ statt blockierend)
key_files_created: []
key_files_modified:
  - shifty-dioxus/src/page/settings.rs
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/cs.rs
decisions:
  - "SDF-03: pure fn heißt sd_year_after_create und lebt in settings.rs direkt unter special_day_form_after_create — folgt exakt dem D-42-05-Extraktionsmuster; kein Extract in ein neues Modul."
  - "SDF-03: Fallback ist iso_year (defensiv — Parse-Erfolg ist zu diesem Zeitpunkt bereits garantiert, aber verhindert stille Regression falls sich die parse_date_to_iso_parts-Signatur ändert)."
  - "SDF-04: Der DE-Text ist Referenz-Master (IMP-05); EN/CS ziehen den Ton nach mit lokalen Replace-Cues (replace / nahrazen). Keine neuen i18n-Keys — bestehender SettingsSpecialDaysDuplicateHint bekommt nur neuen Text."
  - "SDF-04-Test nutzt bestehende generate(locale)-API aus i18n/mod.rs statt neue Locale-Init-Helpers zu bauen."
requirements_completed:
  - SDF-03
  - SDF-04
status: complete
---

# Phase 43 Plan 1: Special-Days-Feintuning FE — Summary

Zwei präzise Frontend-Bugfixes am Card-3 Special-Days-Bereich der Einstellungen: (a) Kalenderjahr-Semantik im Post-Create-Sprung des Jahres-Pickers und (b) i18n-Copy des Duplikat-Hinweises von „already exists" auf Replace-Semantik in de/en/cs — beide Fixes sind pure-frontend, ohne Backend-Touch, Snapshot-Bump oder Migration.

## What was built

### SDF-03: Kalenderjahr statt ISO-Wochenjahr im Jahres-Picker-Sprung

**Bug:** Der Post-Create Success-Handler rief `sd_year.set(iso_year)`. Für ein am 2027-01-01 angelegtes Special Day ist `iso_year` = 2026 (ISO-Woche 53 von 2026 nach ISO-8601-Wochenkalender). Der neu angelegte Feiertag verschwand also aus dem 2027er-Jahres-Picker, in dem der User grade stand — er landete stumm in 2026.

**Fix:** Neue `pub(crate) fn sd_year_after_create(date_str: &str) -> Option<u32>` in `shifty-dioxus/src/page/settings.rs` (unmittelbar unter `special_day_form_after_create`, D-42-05-Muster). Nutzt `time::Date::year()` (Kalenderjahr), nicht `to_iso_week_date().0` (ISO-Wochenjahr). Der Success-Handler ruft jetzt `sd_year.set(sd_year_after_create(&date_s).unwrap_or(iso_year))`.

**Kommentar-Rewrite:** Der frühere WR-04-Kommentarblock über `sd_year.set(iso_year)` ist auf SDF-03 aktualisiert und benennt den Jahreswechsel-Grenzfall explizit.

**4 Unit-Tests** (alle grün):
- `sd_year_after_create_mid_year`: `2026-08-15 → Some(2026)`
- `sd_year_after_create_new_year_calendar_vs_iso`: `2027-01-01 → Some(2027)` PLUS Sanity-Assertion `parse_date_to_iso_parts("2027-01-01").unwrap().0 == 2026` (dokumentiert die Divergenz im Testcode)
- `sd_year_after_create_silvester`: `2026-12-31 → Some(2026)` (Symmetriegegenprobe)
- `sd_year_after_create_invalid_returns_none`: `"not-a-date" → None`

### SDF-04: Duplikat-Hinweis-Copy auf Replace-Semantik

**Problem:** Der Text `Key::SettingsSpecialDaysDuplicateHint` klang in allen drei Locales blockierend („already exists" / „existiert bereits" / „již existuje"), obwohl der Backend-Pfad seit SDF-01/Phase 36 in-place-Replace ist (`service_impl/src/special_days.rs:137-163`). User dachten fälschlich, sie könnten den Tag nicht anlegen.

**Fix:** Copy in allen drei Locales umformuliert (DE bleibt Referenz per IMP-05):

| Locale | Alt | Neu |
|--------|-----|-----|
| DE | "An diesem Tag ist bereits ein Sondertag eingetragen." | "An diesem Tag ist bereits ein Sondertag eingetragen — er wird beim Anlegen **ersetzt**." |
| EN | "A special day already exists for this date." | "A special day is already set for this date — creating will **replace** it." |
| CS | "Pro toto datum již existuje zvláštní den." | "Pro toto datum již existuje zvláštní den — vytvořením bude **nahrazen**." |

Der bestehende i18n-Key wurde beibehalten — alle Verwendungsstellen (aktuell nur Row D in `settings.rs:886`) profitieren automatisch.

**1 Presence/Anti-Wording-Unit-Test:**
- `duplicate_hint_copy_signals_replace_semantics`: verifiziert für alle drei Locales via `generate(locale)`, dass der Text nicht leer ist und den erwarteten Replace-Cue enthält (`ersetzt|überschrieben` / `replace` / `nahrazen|přepsán`, case-insensitive).

## Pure functions extracted

- `pub(crate) fn sd_year_after_create(date_str: &str) -> Option<u32>` — Post-create Ziel-Jahr für den Card-3 Jahres-Picker (Kalenderjahr, nicht ISO-Wochenjahr).

## Neue Unit-Tests (in `page::settings::tests`)

1. `sd_year_after_create_mid_year`
2. `sd_year_after_create_new_year_calendar_vs_iso` (Kern-Grenzfall SDF-03)
3. `sd_year_after_create_silvester`
4. `sd_year_after_create_invalid_returns_none`
5. `duplicate_hint_copy_signals_replace_semantics` (SDF-04, alle 3 Locales)

Alle 23 Tests im `page::settings::tests`-Modul grün.

## Verification results

### Frontend gates
```
cd shifty-dioxus
cargo test -p shifty-dioxus page::settings::tests::   # 23 passed, 0 failed
cargo test -p shifty-dioxus -- --skip i18n_impersonation_keys_match_german_reference  # 761 passed, 0 failed
cargo build --target wasm32-unknown-unknown           # Finished dev profile in 46.96s
```

**Vorbestehender Test-Fail (unrelated to this plan):** `i18n_impersonation_keys_match_german_reference` schlägt seit v1.11 (Phase 37-02, Commit `83a0d91`) fehl — dokumentiert in `.planning/todos/pending/2026-07-02-i18n-impersonation-key-test-mismatch.md`, deferred bis Produkt-Copy-Entscheidung des Users. Nicht durch Phase 43-01 verursacht, kein Blocker.

### Backend gates (Cross-Workspace-Sicherheit)
```
cargo build --workspace                                # Finished dev profile in 0.27s
cargo test --workspace                                 # 698 passed, 0 failed across all crates
cargo clippy --workspace -- -D warnings                # Finished (0 warnings, backend hard gate grün)
```

### Frontend clippy
`cargo clippy -p shifty-dioxus --bin shifty-dioxus -- -D warnings` schlägt fehl an ~151 pre-existing Lints (Baseline aus v1.x, Memory `feedback_dioxus_clippy_not_gated`). **Für Files, die dieses Plan berührt:** Filtern auf `page/settings.rs` und `i18n/{de,en,cs}.rs` ergibt genau 1 pre-existing Warning (`page/settings.rs:898`, `manual_range_contains` im 2020–2099-Jahres-Guard) — **nicht durch Phase 43-01 eingeführt**, existierte schon in der Zeile `if y >= 2020 && y <= 2099` vor meinen Edits. Meine 5 neuen Tests + 1 neue pure fn + 3 i18n-Textänderungen ziehen **null neue clippy-Warnings** nach sich.

### Regression-Greps (aus `<verification>`-Block)
```
grep -c "sd_year.set(iso_year)"                       shifty-dioxus/src/page/settings.rs  → 0  ✓
grep -c "A special day already exists for this date"  shifty-dioxus/src/i18n/en.rs        → 0  ✓
grep -c "An diesem Tag ist bereits ein Sondertag eingetragen\.$"  de.rs                   → 0  ✓
grep -c "Pro toto datum již existuje zvláštní den\.$"            cs.rs                   → 0  ✓
grep -c "ersetzt\|überschrieben"                      shifty-dioxus/src/i18n/de.rs        → 1  ✓
grep -c "replace"                                     shifty-dioxus/src/i18n/en.rs        → 1  ✓
grep -c "nahrazen\|přepsán"                          shifty-dioxus/src/i18n/cs.rs        → 1  ✓
```

Alle Guards grün.

## Deviations from Plan

Keine — Plan wurde exakt wie geschrieben ausgeführt. Kleine Format-/Kommentar-Nachzieher:

1. **Docstring-Update `special_day_form_after_create`**: die bestehende Doc referenzierte noch `sd_year.set(iso_year)` (WR-04). Angepasst auf `sd_year.set(sd_year_after_create(...))` (SDF-03), damit der `grep -c "sd_year.set(iso_year)" == 0` Guard aus `<verification>` auch die Doc-Erwähnung mit-erfüllt und die Historie im Code-Kommentar konsistent bleibt. Rein textuelle Änderung, keine Verhaltensänderung.
2. **Test-Kommando `--lib` → `--bin shifty-dioxus`**: `shifty-dioxus` ist ein binary crate ohne library-Target (`cargo test --lib` schlägt fehl mit „no library targets found"). Die aequivalenten Kommandos ohne `--lib` wurden zur Verifikation genutzt (`cargo test -p shifty-dioxus TESTNAME`). Verhalten identisch, nur CLI-Form angepasst.
3. **Frontend-`-D warnings`-Clippy nicht durchgezogen**: Baseline hat ~151 pre-existing Warnings (bekannt, Memory-Referenz `feedback_dioxus_clippy_not_gated`; CI clippt shifty-dioxus nicht). Meine Edits fügen null neue Warnings hinzu (Files-Filter oben). Der Backend-`-D warnings`-Clippy-Gate — der harte Gate laut CLAUDE.md — ist grün.

## Known Stubs

Keine.

## Threat Flags

Keine — Plan berührt keine neuen Trust-Boundaries. `T-43-01` (Info Disclosure in i18n-Copy) und `T-43-02` (Client-side signal tampering) beide `accept` per Threat Register des Plans, keine neuen Vektoren.

## Auth gates

Keine.

## Self-Check: PASSED

- `shifty-dioxus/src/page/settings.rs`: `sd_year_after_create` als `pub(crate) fn` vorhanden (Zeile ~140), Success-Handler-Fix in Zeile ~635, 5 neue Tests im test-Modul.
- `shifty-dioxus/src/i18n/de.rs`, `en.rs`, `cs.rs`: `SettingsSpecialDaysDuplicateHint`-Registration mit Replace-Cue.
- `cargo build --target wasm32-unknown-unknown`: grün.
- `cargo test -p shifty-dioxus page::settings::tests::`: 23 passed / 0 failed.
- `cargo build --workspace` + `cargo test --workspace` + `cargo clippy --workspace -- -D warnings`: alle grün.
- Regression-Greps: alle 7 Guards liefern erwartetes Ergebnis.

---

## Post-Ship-Nachtrag (2026-07-03, vor Release-Tag)

SDF-03-Fix in Phase 43-01 hat NUR `sd_year.set()` nach einem Create auf
Kalenderjahr umgestellt — der DB-Filter blieb ISO-Wochenjahr-basiert. Ein
01.01.2027-Eintrag wurde als `(2026, W53, Fri)` gespeichert und war im
2026er-Filter sichtbar, im 2027er-Filter unsichtbar.

**Nachträglicher Fix (v2.2, post-ship):** `SpecialDayServiceImpl::get_by_year`
lädt jetzt `year` UND `year - 1`, filtert per `ShiftyDate::to_date().year()`
und sortiert nach Kalenderdatum aufsteigend. Neue Tests:
`test_get_by_year_delegates_and_maps` (Basis-Fall auf 2026-W10-Mo angehoben)
+ `test_get_by_year_returns_new_year_day_under_calendar_year` (positive:
01.01.2027 sichtbar bei year=2027 und sortiert VOR 11.01.2027; negative:
nicht sichtbar bei year=2026).

Files: `service_impl/src/special_days.rs`, `service_impl/src/test/special_days.rs`.
Siehe auch `MILESTONES.md` v2.2-Post-Ship-Sektion.
