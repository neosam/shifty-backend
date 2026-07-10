---
phase: 54-data-model-voluntary-stats
plan: 08
subsystem: [frontend, i18n, voluntary-stats]
tags: [gap-closure, frontend, dioxus, wasm, i18n, voluntary-stats, terminology-rename]
requires: [54-07]
provides: [gap-g1-fe-anteil, gap-g2-i18n-vereinheitlichung]
affects: [shifty-dioxus/src/api.rs, shifty-dioxus/src/loader.rs, shifty-dioxus/src/service/employee.rs, shifty-dioxus/src/i18n/de.rs, shifty-dioxus/src/i18n/en.rs, shifty-dioxus/src/i18n/mod.rs, shifty-dioxus/src/component/employee_view.rs, shifty-dioxus/src/page/weekly_overview.rs]
tech-stack:
  added: []
  patterns:
    - "time::Date::from_iso_week_date für ISO-Sonntag-Berechnung (kein neuer Dep, `time` war bereits Dep)"
    - "String-Wert-Rename ohne Rust-Symbol-Change (Key-Enum, TO-Felder bleiben)"
key-files:
  created: []
  modified:
    - "shifty-dioxus/src/api.rs (get_voluntary_stats: +until_week param, URL nutzt from_date+to_date; neue Helper-Fn compute_voluntary_stats_to_date)"
    - "shifty-dioxus/src/loader.rs (load_voluntary_stats: +until_week param)"
    - "shifty-dioxus/src/service/employee.rs (load_employee_data reicht until_week durch)"
    - "shifty-dioxus/src/i18n/de.rs (9 String-Werte Freiwillig → Ehrenamt)"
    - "shifty-dioxus/src/i18n/en.rs (5 String-Werte Volunteer/Work → Voluntary/work)"
    - "shifty-dioxus/src/i18n/mod.rs (3 Reference-Tests aktualisiert)"
    - "shifty-dioxus/src/component/employee_view.rs (4 SSR-Tests: Volunteer → Voluntary)"
    - "shifty-dioxus/src/page/weekly_overview.rs (page_header_uses_three_band_key Test)"
decisions:
  - "Rust-Symbole (Key-Enum-Varianten wie Key::PaidVolunteer, Key::CategoryVolunteerWork, DB-Spalten committed_voluntary, TO-Feld-Namen) BLEIBEN unverändert — nur String-Werte umbenannt. Rationale: Minimaler Blast-Radius, keine API-Break, keine DB-Migration."
  - "cs.rs unangetastet — User-approved Deferral aus 54-UAT.md Test 3 (deferred_items.54-08-cs-rename)."
  - "Range wird aus (year, until_week) im FE abgeleitet, kein neuer Date-Picker — konsistent mit Employee-Report-Kontext."
  - "compute_voluntary_stats_to_date fällt bei ungültiger ISO-Woche (until_week > weeks_in_year) auf YYYY-12-31 zurück; Backend validiert die Range zusätzlich (Plan 54-07 Task 4)."
metrics:
  duration_minutes: 6
  completed_date: "2026-07-10"
  tasks_completed: 4
  files_modified: 8
  commits: 2
status: complete
---

# Phase 54 Plan 08: Gap-Closure G1 FE-Anteil + G2 i18n-Vereinheitlichung Summary

FE zieht auf die neue Backend-Range-Signatur nach (`?from_date=…&to_date=…` statt `?year=…`) und vereinheitlicht die Voluntary-Terminologie (de: Freiwillig → Ehrenamt; en: Volunteer → Voluntary) in allen user-facing i18n-Strings — Rust-Symbole und cs.rs bleiben unverändert.

## Overview

**Ausgangslage:** Plan 54-07 hat den Backend-Endpoint `GET /report/{id}/voluntary-stats` auf `?from_date=YYYY-MM-DD&to_date=YYYY-MM-DD` umgestellt und antwortet bei altem `?year=YYYY` mit HTTP 400. Ohne FE-Update wäre die Voluntary-Stats-Row auf dem Employee-Report leer. Parallel dazu hatte der User in UAT.md moniert, dass "Freiwillig" und "Ehrenamt" im deutschen UI vermischt sind und im englischen UI "Volunteer" statt einheitlich "Voluntary" auftaucht.

**Ergebnis:** FE ruft die Backend-Range mit `from = YYYY-01-01`, `to = Sonntag der (year, until_week)` auf; alle user-facing Strings sind konsistent auf "Ehrenamt …" (DE) und "Voluntary" (EN) umgestellt.

## Task 1 — FE URL-Umstellung (Gap G1)

### `shifty-dioxus/src/api.rs` — `get_voluntary_stats`-Diff

**Signatur:** `+until_week: u8` als zusätzlicher Parameter.

**Vorher:**
```rust
pub async fn get_voluntary_stats(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
) -> Result<VoluntaryStatsTO, reqwest::Error> {
    let url = format!(
        "{}/report/{}/voluntary-stats?year={}",
        config.backend, sales_person_id, year
    );
    ...
}
```

**Nachher:**
```rust
pub async fn get_voluntary_stats(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
    until_week: u8,
) -> Result<VoluntaryStatsTO, reqwest::Error> {
    let from_date = format!("{:04}-01-01", year);
    let to_date = compute_voluntary_stats_to_date(year, until_week);
    let url = format!(
        "{}/report/{}/voluntary-stats?from_date={}&to_date={}",
        config.backend, sales_person_id, from_date, to_date
    );
    ...
}

/// Berechnet das `to_date` (Sonntag der ISO-Woche `until_week` im Jahr `year`)
/// im Format `YYYY-MM-DD`. Fallback bei ungueltiger Woche: `YYYY-12-31`.
fn compute_voluntary_stats_to_date(year: u32, until_week: u8) -> String {
    if let Ok(sunday) =
        time::Date::from_iso_week_date(year as i32, until_week, time::Weekday::Sunday)
    {
        format!(
            "{:04}-{:02}-{:02}",
            sunday.year(),
            sunday.month() as u8,
            sunday.day()
        )
    } else {
        format!("{:04}-12-31", year)
    }
}
```

`time` war bereits Dep in `shifty-dioxus/Cargo.toml`; `time::Date` wird an anderen Stellen der Datei bereits genutzt (Zeile 1074, 1075, 1343). Kein neuer `use time`-Import nötig (qualifiziert verwendet).

### `shifty-dioxus/src/loader.rs` — `load_voluntary_stats`

- `+until_week: u8` in Signatur, an `api::get_voluntary_stats` durchgereicht.
- Doc-Kommentar aktualisiert (`?from_date=…&to_date=…` in Erklärung).

### `shifty-dioxus/src/service/employee.rs` — `load_employee_data`

Aufruf angepasst:
```rust
let voluntary_stats =
    loader::load_voluntary_stats(CONFIG.read().clone(), sales_person_id, year, until_week)
        .await
        .unwrap_or_default();
```

`until_week` war schon Function-Arg (Zeile 109) — kein neuer User-Input, kein Date-Picker, keine State-Änderung.

## Task 2 + 3 — i18n-Rename (Gap G2)

### `shifty-dioxus/src/i18n/de.rs` — 9 String-Renames

| Key | Alt | Neu |
|---|---|---|
| `PaidVolunteer` | `"Bezahlt / Freiwillig"` | `"Bezahlt / Ehrenamt"` |
| `Committed` | `"Freiwillig zugesagt"` | `"Ehrenamt zugesagt"` |
| `PaidCommittedVolunteer` | `"Bezahlt / Freiwillig zugesagt / Freiwillig"` | `"Bezahlt / Ehrenamt zugesagt / Ehrenamt"` |
| `CommittedVoluntaryLabel` | `"Freiwillige Zusage (h)"` | `"Ehrenamt-Zusage (h)"` |
| `CommittedVoluntaryHelp` | `"Zugesagte freiwillige Stunden."` | `"Zugesagte Ehrenamt-Stunden."` |
| `Volunteer` | `"Freiwillig"` | `"Ehrenamt"` |
| `AbsenceGroupVolunteers` | `"Freiwillige"` | `"Ehrenamtliche"` |
| `VoluntaryHoursIstPerWeek` | `"Freiwillig Ø / Woche"` | `"Ehrenamt Ø / Woche"` |
| `VoluntaryHoursSoll` | `"Freiwillig Soll"` | `"Ehrenamt Soll"` |
| `VoluntaryHoursDelta` | `"Freiwillig Delta"` | `"Ehrenamt Delta"` |

`CategoryVolunteerWork` blieb bei `"Ehrenamt"` (war bereits konsistent, kein Rename nötig).

Übrig gebliebenes `Freiwillig` in de.rs Zeile 1332 ist ein **Rust-Kommentar** (`// Phase 54: HR-only Freiwillig-Stunden-Konto`) — laut Plan bleiben Kommentare unangetastet.

### `shifty-dioxus/src/i18n/en.rs` — 5 String-Renames

| Key | Alt | Neu |
|---|---|---|
| `PaidVolunteer` | `"Paid / Volunteer"` | `"Paid / Voluntary"` |
| `PaidCommittedVolunteer` | `"Paid / Voluntary committed / Volunteer"` | `"Paid / Voluntary committed / Voluntary"` |
| `CategoryVolunteerWork` | `"Volunteer Work"` | `"Voluntary work"` |
| `CommittedVoluntaryLabel` | `"Voluntary Commitment (h)"` | `"Voluntary commitment (h)"` |
| `Volunteer` | `"Volunteer"` | `"Voluntary"` |
| `AbsenceGroupVolunteers` | `"Volunteers"` | `"Voluntary staff"` |

`Key::Committed = "Voluntary committed"` blieb (bereits konsistent). Ebenso `VoluntaryHoursIstPerWeek/Soll/Delta` und `CommittedVoluntaryHelp` (waren schon "Voluntary…").

### `shifty-dioxus/src/i18n/cs.rs` — UNANGEFASST

User-approved Deferral (54-UAT.md Test 3 skipped). `git diff HEAD~2 HEAD -- shifty-dioxus/src/i18n/cs.rs` = 0 Zeilen.

### Test-Anpassungen (Reference-/SSR-Tests hardcoden Copies)

Diese Tests haben die alten String-Werte assertiert und musste nachgezogen werden — vgl. Deviations-Sektion, mechanisch:

- **`shifty-dioxus/src/i18n/mod.rs`** — 3 Reference-Tests aktualisiert:
  - `i18n_committed_keys_match_german_reference` — "Freiwillig zugesagt" → "Ehrenamt zugesagt", three-band-copy analog.
  - `i18n_committed_keys_match_english_reference` — "Paid / Voluntary committed / Volunteer" → "Paid / Voluntary committed / Voluntary".
  - `i18n_phase17_keys_match_german_reference` — "Freiwillige Zusage (h)" → "Ehrenamt-Zusage (h)".
  - `i18n_phase17_keys_match_english_reference` — "Voluntary Commitment (h)" → "Voluntary commitment (h)".
  - `i18n_contract_help_keys_match_german_reference` — "Zugesagte freiwillige Stunden." → "Zugesagte Ehrenamt-Stunden.".

- **`shifty-dioxus/src/component/employee_view.rs`** — 4 SSR-Tests (week_list_shows/no_volunteer_when_gt_zero/zero und week_detail_panel_shows/no_volunteer_when_gt_zero/zero) — assertion-Substring `"Volunteer"` → `"Voluntary"`.

- **`shifty-dioxus/src/page/weekly_overview.rs`** — `page_header_uses_three_band_key` — Substring "Paid / Voluntary committed / Volunteer" → "Paid / Voluntary committed / Voluntary".

Rust-Symbole (`Key::PaidVolunteer`, `Key::AbsenceGroupVolunteers`, DB-Spalten `committed_voluntary`, TO-Feld-Namen etc.) **komplett unverändert**.

## Grep-Guards (alle 0)

```
grep -c '"[^"]*Freiwillig' shifty-dioxus/src/i18n/de.rs                        → 0
grep -c '"[^"]*Volunteer\b\|"[^"]*Volunteer Work' shifty-dioxus/src/i18n/en.rs → 0
git diff HEAD~2 HEAD -- shifty-dioxus/src/i18n/cs.rs | wc -l                    → 0
grep -B 2 -A 8 'fn get_voluntary_stats' shifty-dioxus/src/api.rs | grep -c '?year=' → 0
```

Übrig gebliebenes `Freiwillig` in `de.rs:1332` ist ein Rust-Kommentar (nicht in einem `"…"`-Literal), daher greift `grep -c '"[^"]*Freiwillig'` = 0. Andere `Volunteer`-Vorkommen in `en.rs` sind alle in Kontexten wie `Key::PaidVolunteer` (Rust-Symbol) oder `AbsenceGroupVolunteers` (Rust-Symbol) — der Regex `"[^"]*Volunteer\b` triggert nur auf String-Werte.

## Verification-Log

Alle 4 Gates grün (aus dieser Session, nach dem letzten Commit `00de0d8`):

```
1. cd shifty-dioxus && cargo build --target wasm32-unknown-unknown
   → Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.24s

2. cd shifty-dioxus && cargo test -p shifty-dioxus
   → test result: ok. 806 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

3. cd shifty-dioxus && cargo clippy -p shifty-dioxus -- -D warnings
   → Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.14s   (0 warnings)

4. SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings
   → Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s    (0 warnings, Regression-Check grün)
```

**Interpretation FE-Clippy:** In dieser Session lief `cargo clippy -p shifty-dioxus -- -D warnings` sauber ohne Warnings — die aus MEMORY (`reference_dioxus_clippy_not_gated`) bekannten ~198 pre-existing Lints kamen nicht hoch. Vermutlich wurden sie in einer Zwischen-Phase (43+45?) aufgeräumt und die MEMORY-Note ist bei diesem Repo-Stand veraltet. Kein Blocker.

## Manual-Verify-Anweisung

**Backend + FE starten:**

```bash
# Terminal 1: Backend
cd shifty-backend && cargo run

# Terminal 2: Tailwind Watch
cd shifty-backend/shifty-dioxus && npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch

# Terminal 3: Dioxus Dev-Server
cd shifty-backend/shifty-dioxus && dx serve --hot-reload
```

**HR-User im DE-Locale:**

1. Zum Employee-Detail-Report des Test-Freiwilligen mit `committed_voluntary=5.0/week` (Contract seit KW 18/2026 = 2026-04-27) navigieren.
2. **Voluntary-Stats-Row** muss 3 Zeilen mit **"Ehrenamt …"-Labels** zeigen:
   - "Ehrenamt Ø / Woche"
   - "Ehrenamt Soll"
   - "Ehrenamt Delta"
3. **Erwartete Ehrenamt-Soll-Range (bei Ansicht in KW 28/2026):**
   - Range: `from_date=2026-01-01`, `to_date=2026-07-12` (Sonntag der KW 28/2026).
   - Contract-aktive Tage im Range = 27.4. bis 12.7. = 77 Tage.
   - `soll_total ≈ 5.0 × 77 / 7 ≈ 55h` (NICHT 177h!).
   - Der genaue Wert hängt vom Backend `contract_weeks`-Zähler ab (Plan 54-05); Plan-Referenzwert ≈ 53.6h für until_week=28.
4. **Weekly-Overview-Header** muss lauten: `"Bezahlt / Ehrenamt zugesagt / Ehrenamt"`.

**Locale-Wechsel EN:**

- Voluntary-Stats-Row zeigt "Voluntary avg / week", "Voluntary target", "Voluntary delta".
- Weekly-Overview-Header: `"Paid / Voluntary committed / Voluntary"`.
- Keine "Volunteer"-Strings mehr sichtbar in User-facing UI (bis auf hardcodierte Test-Fixtures, die nicht ins Live-UI gehen).

**Non-HR User:**

- Backend-Redaktion greift, Voluntary-Stats-Row bleibt komplett leer (unverändert seit Plan 54-03/07).

**Screenshot-Verifikation:** MEMORY-Muster `reference_dioxus_screenshots_html2canvas` + `reference_dioxus_browser_verify_reports` (`get_page_text + find` für Zahlen, `html2canvas` für Layout).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Test-Update] SSR- und Reference-Tests hardcoden alte Copies**

- **Found during:** Task 2/3 verification (`cargo test -p shifty-dioxus`).
- **Issue:** 8 Test-Failures, weil die i18n-Reference-Tests in `mod.rs` und die SSR-Tests in `employee_view.rs` / `weekly_overview.rs` die alten String-Werte ("Freiwillig zugesagt", "Volunteer", "Voluntary Commitment (h)", "Paid / Voluntary committed / Volunteer" etc.) fest im Test-Body assertiert haben.
- **Fix:** Assertions auf die neuen Strings umgestellt, jeweils mit Kommentar `// Phase 54-08 Gap G2 i18n unification: …`. Kein Rust-Symbol-Change, nur Assertion-Substrings.
- **Files modified:** shifty-dioxus/src/i18n/mod.rs, shifty-dioxus/src/component/employee_view.rs, shifty-dioxus/src/page/weekly_overview.rs.
- **Commit:** `00de0d8` (zusammen mit dem i18n-Rename, weil die Tests logisch mit dem Rename kollidieren und der Fix mechanisch ist).
- **Rationale:** Rule 1 (Bug/Test-Regression) — Reference- und SSR-Tests sind Guards für exakt die Copies, die wir umbenennen; sie MÜSSEN mit-ziehen, sonst blockieren sie das Gate. Der Plan hat Punkt 4-Anmerkung dazu ("Falls Task 2 oder Task 3 einen unerwarteten SSR-Test brechen … Test-Erwartung an neuen String anpassen") — genau dieser Fall.

**2. [Rule 3 - Missing docstring update] `loader.rs`-Doc-Kommentar zeigte alte URL**

- **Found during:** Task 1 review.
- **Issue:** Der Doc-Kommentar von `load_voluntary_stats` verwies noch auf `GET /report/{id}/voluntary-stats?year=YYYY`.
- **Fix:** Doc-Kommentar auf `?from_date=YYYY-MM-DD&to_date=YYYY-MM-DD` aktualisiert.
- **Commit:** `a4f72e5`.

### FE-Clippy MEMORY-Note pre-existing Lints

**MEMORY-Note:** `reference_dioxus_clippy_not_gated` besagt, dass shifty-dioxus ~198 pre-existing Clippy-Lints hat, und dass FE-Clippy aus dem Dioxus-Shell wegen E0514 kaputt sei.

**Observation in dieser Session:** `cd shifty-dioxus && cargo clippy -p shifty-dioxus -- -D warnings` lief clean durch (10.14s, 0 Warnings, exit 0), ohne Environment-Manipulation. Vermutlich wurden die pre-existing Lints in Phase 43/45 (oder später) bereits aufgeräumt und die MEMORY-Note ist am aktuellen Repo-Stand veraltet. Nicht in scope für diesen Plan.

### Deferred (unverändert aus Plan-Frontmatter)

- **`54-08-cs-rename`** — `shifty-dioxus/src/i18n/cs.rs` Terminologie-Rename bleibt für User-approved Deferral (54-UAT.md Round 1 Test 3). Keine Aktion in diesem Plan.

## Commits

```
00de0d8 refactor(54-08): unify i18n de/en to Ehrenamt / voluntary (Gap G2)
a4f72e5 feat(54-08): voluntary-stats FE — call backend with date range (Gap G1)
```

Auf `HEAD` seit dieser Session (nach `ddfd3dc docs(54-07): SUMMARY …` und `7aefad3 feat(54-07): …`).

## Threat-Model-Check

Der Plan-`<threat_model>` deklariert für `T-54-08-01` (from_date/to_date-Tampering) und `T-54-08-02` (i18n Info-Disclosure) beide **accept** — kein Mitigation-Aufwand. `T-54-08-SC` (Supply-Chain) → **mitigate**: keine neuen Cargo-Deps hinzugefügt (`time` war bereits Dep).

Keine neuen Threat-Flags. Diese Änderung erweitert keine Trust-Boundary — sie ändert nur URL-Query-String-Format (Backend-validiert seit Plan 54-07) und user-facing Text-Copies (kein XSS-Vektor, Dioxus rendert als Text).

## Self-Check: PASSED

- [x] `shifty-dioxus/src/api.rs` modifiziert, `compute_voluntary_stats_to_date` vorhanden, Signatur mit `until_week: u8`.
- [x] `shifty-dioxus/src/loader.rs` — `load_voluntary_stats` reicht `until_week` durch.
- [x] `shifty-dioxus/src/service/employee.rs` Zeile 152 übergibt `until_week`.
- [x] `shifty-dioxus/src/i18n/de.rs` — 0 String-Werte mit "Freiwillig" (grep-guard = 0).
- [x] `shifty-dioxus/src/i18n/en.rs` — 0 String-Werte mit "Volunteer\b" oder "Volunteer Work" (grep-guard = 0).
- [x] `shifty-dioxus/src/i18n/cs.rs` unverändert (`git diff` leer).
- [x] 4 Gates grün: WASM-Build, FE-Test (806 passed), FE-Clippy, BE-Clippy.
- [x] Commit `a4f72e5` (Gap G1) und `00de0d8` (Gap G2) existieren.
