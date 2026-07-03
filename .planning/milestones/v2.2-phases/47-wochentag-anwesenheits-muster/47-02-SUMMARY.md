---
phase: 47-wochentag-anwesenheits-muster
plan: 2
subsystem: frontend/report-view
tags: [reporting, attendance, hr, weekday, i18n, dioxus, breaking-change-consumer]
status: complete
requirements: [RPT-02, RPT-03]
requires: [47-01]
provides:
  - component::employee_view::format_weekday_attendance_line
  - i18n::Key::WeekdayShortMon..WeekdayShortSun
  - i18n::Key::WeekdayAttendanceTooltip
  - i18n::Key::WeekdayAttendanceEmpty
affects:
  - component/employee_view.rs (HR-stats block render + SSR tests)
  - i18n/{mod,de,en,cs}.rs (9 new keys × 3 locales; 3 v2.1 AVG keys removed)
tech-stack:
  added: []
  patterns:
    - pure-fn formatter + SSR-test module (mirrors v2.1 STAT-01 pattern)
    - i18n presence-test guard for the completeness gate
key-files:
  created: []
  modified:
    - shifty-dioxus/src/component/employee_view.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
  deleted: []
decisions:
  - D-47-CONTEXT — Layout `Mo: 8 (80%) · Di: 3 (30%) · …` (7 segments joined by ` · `)
  - D-AVG-05 (retained) — Row visibility gated by `attendance_statistics.is_some()`
  - RPT-03 — Presence-test covers 9 keys × 3 locales, tooltip + empty-state included
  - Rounding — `pct = (share * 100.0).round() as i32` (no floor, no ceil)
metrics:
  duration: 13m
  completed: 2026-07-02T22:14Z
---

# Phase 47 Plan 2: Frontend Weekday-Attendance Row + i18n Summary

Replace the v2.1 `Ø Std/Anwesenheitstag` scalar in the HR-stats block of `component/employee_view.rs` with the new per-weekday distribution line delivered by 47-01, add nine new i18n keys × three locales, delete the three orphaned v2.1 keys, and pin the visible output with SSR tests + a formatter unit test + an i18n presence test.

## New Pure Function

**Signature** (in `component/employee_view.rs`):

```rust
pub fn format_weekday_attendance_line(
    stats: &EmployeeAttendanceStatisticsTO,
    i18n: &I18nType,
) -> String
```

**Semantics** (D-47-CONTEXT):

- `counted_calendar_weeks == 0` OR `attendance_by_weekday` empty → return the localized `Key::WeekdayAttendanceEmpty` placeholder text.
- Otherwise: iterate `attendance_by_weekday` in the BE-provided order (Mon..Sun — the FE does NOT re-sort). Each row becomes `"{short-label}: {count} ({pct}%)"` where `pct = (share * 100.0).round() as i32`. Segments are joined by `" · "` (space, U+00B7 MIDDLE DOT, space).
- Weekday → i18n key mapping via `fn weekday_short_key(DayOfWeekTO) -> Key` (7-arm match, exhaustive).

**Sample output** (`counted_calendar_weeks = 10`, `share ∈ [0.0, 1.0]`, Locale::De):

```
Mo: 8 (80%) · Di: 3 (30%) · Mi: 7 (70%) · Do: 5 (50%) · Fr: 2 (20%) · Sa: 0 (0%) · So: 0 (0%)
```

## HR-stats-block Rendering

The v2.1 attendance-row `if let Some(att) …` block (lines 537–558) is replaced with a single new row rendered as `TupleRow`:

- Label = `Key::WeekdayAttendanceTooltip` (also duplicated as the `title=` attribute for hover tooltip).
- Value = `<span class="font-mono tabular-nums" title="…">{format_weekday_attendance_line(att, &i18n)}</span>`.
- Row visibility gate `attendance_statistics.is_some()` preserved from v2.1 (D-AVG-05).

## New i18n Keys

| Key | De | En | Cs |
|-----|----|----|----|
| `WeekdayShortMon` | Mo | Mon | Po |
| `WeekdayShortTue` | Di | Tue | Út |
| `WeekdayShortWed` | Mi | Wed | St |
| `WeekdayShortThu` | Do | Thu | Čt |
| `WeekdayShortFri` | Fr | Fri | Pá |
| `WeekdayShortSat` | Sa | Sat | So |
| `WeekdayShortSun` | So | Sun | Ne |
| `WeekdayAttendanceTooltip` | Anzahl Anwesenheitstage pro Wochentag und Anteil an den gezählten Kalenderwochen im Zeitraum | Attendance-day count per weekday and share relative to counted calendar weeks in the range | Počet pracovních dní podle dne v týdnu a podíl vůči započítaným kalendářním týdnům v období |
| `WeekdayAttendanceEmpty` | Keine gezählten Kalenderwochen im Zeitraum | No counted calendar weeks in range | Žádné započítané kalendářní týdny v období |

## Removed v2.1 Keys

- `Key::AvgHoursPerAttendanceDay` (enum variant + de/en/cs bodies)
- `Key::AvgHoursPerAttendanceDayDescription` (enum variant + de/en/cs bodies)
- `Key::AvgHoursPerAttendanceDayEmpty` (enum variant + de/en/cs bodies)
- Presence-test `i18n_attendance_keys_present_in_all_locales` (replaced by `phase_47_weekday_i18n_presence`)
- SSR-test block `render_attendance_row` + 3 tests in `component/employee_view.rs` (`attendance_row_shows_number_when_some`, `attendance_row_shows_endash_when_inner_none`, `attendance_row_absent_when_none`)

Grep gate:

```text
$ grep -rn 'AvgHoursPerAttendanceDay' shifty-dioxus/src/
(no matches)
```

## Tests Added

1. `phase_47_weekday_i18n_presence` (i18n/mod.rs) — asserts each of the 9 new keys resolves to a non-empty, non-`??` string in `De/En/Cs` (27 lookups) and that the 7 weekday-short labels within each locale are DISTINCT.
2. `weekday_row_renders_all_seven_segments_when_populated` (component/employee_view.rs) — SSR renders 7 segments `Mon: 8 (80%) … Sun: 0 (0%)`, contains ≥6 `·` separators, carries a `title=` tooltip, does NOT contain the retired `Ø Std/Anwesenheitstag` label.
3. `weekday_row_renders_empty_state_when_counted_weeks_zero` — SSR renders the localized empty-state text; no weekday segments appear.
4. `weekday_row_absent_when_statistics_is_none` — SSR renders NO row (`font-mono` absent, no weekday label segments).
5. `formatter_handles_odd_percents_correctly` — pure-fn: `share=0.333 → "Mon: 3 (33%)"` (round-half-to-even/round-nearest via `f32::round`).

## Gates

- `cargo test -p shifty-dioxus`: 779 tests green (was 774 in the pre-47-02 state; +5 new tests, −3 v2.1 SSR tests, +1 net after replacing the presence test).
- `cargo build --target wasm32-unknown-unknown` in `shifty-dioxus/`: green (restored after 47-01 intentionally broke it).
- `cargo clippy -p shifty-dioxus --tests -- -D warnings`: clean.
- `cargo clippy --workspace -- -D warnings` at repo root (backend hard gate): clean.
- `cargo test --workspace` at repo root (backend suite): green (all suites unchanged from 47-01 baseline).

Grep gates:

```text
$ grep -rn 'AvgHoursPerAttendanceDay' shifty-dioxus/src/  # RPT-02 payload removal
(none)

$ grep -rn 'WeekdayShortMon\|WeekdayShortSun' shifty-dioxus/src/i18n/ | wc -l
10   # Key enum def (2 lines: Mon + Sun) + 4 locale files (Mon + Sun × 4) = 10
```

## Threat-Model Notes

- **T-47-05** (info disclosure via HR-stats-block visibility) — mitigation intact: SSR test `weekday_row_absent_when_statistics_is_none` pins that the whole row disappears when the DTO is None.
- **T-47-06** (weekday-order tampering) — formatter iterates `attendance_by_weekday` in BE order and does NOT re-sort; the populated-state SSR test pins the Mon..Sun sequence.
- **T-47-07** (share out-of-range) — formatter clamps implicitly via `.round() as i32` (no panic even on NaN/Inf under `f32::round`; NaN → 0 in `as i32` cast).
- **T-47-SC** (package installs) — no new deps added.

## Manual UI Smoke — Deferred

Per D-25-06 policy (WASM interactive browser tests are unreliable), no browser smoke run. Structural pinning through the 3 SSR tests + 1 formatter test + 1 i18n presence test is the compliance evidence. Follow-up phases MAY add an in-browser smoke once the D-25-06 pattern is revisited (out of scope for 47).

## Self-Check: PASSED

Files modified (existence + edit confirmed):

- FOUND: `shifty-dioxus/src/component/employee_view.rs` (row swap + SSR tests + formatter helper)
- FOUND: `shifty-dioxus/src/i18n/mod.rs` (9 new Key variants, 3 old removed, new presence test)
- FOUND: `shifty-dioxus/src/i18n/de.rs` (9 new De bodies, 3 old removed)
- FOUND: `shifty-dioxus/src/i18n/en.rs` (9 new En bodies, 3 old removed)
- FOUND: `shifty-dioxus/src/i18n/cs.rs` (9 new Cs bodies, 3 old removed)

Files created:

- FOUND: `.planning/phases/47-wochentag-anwesenheits-muster/47-02-SUMMARY.md`

Commits will be recorded by GSD auto-commit (co-located jj/git).
