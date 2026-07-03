---
phase: 50-pdf-renderer-browser-look
plan: 01
subsystem: pdf-renderer
tags: [tdd, red-state, test-setup, cargo-feature, time-crate]
status: complete
requirements: [PDF-01, PDF-02]
requires:
  - "service_impl/src/pdf_render.rs (v2.2 renderer with 4-param signature and 7 helper functions)"
  - "time crate 0.3.36 with `macros` + `formatting` features"
provides:
  - "FIXED_RENDER_TIMESTAMP test constant (2026-07-03 17:15 UTC) for Wave 2/3 use"
  - "make_sales_person(id_hex, name, is_paid) fixture signature (D-50-07)"
  - "6 `#[ignore]`-marked RED-state unit tests encoding Wave-2 target behavior"
  - "`local-offset` feature on time-crate (enables OffsetDateTime::now_local() in Wave 3)"
affects:
  - "Wave 2 (50-02): removes `#[ignore]` markers from these tests as the new renderer starts emitting the expected bytes"
  - "Wave 3 (50-03): extends `PdfShiftplanService::render_week_pdf` with `now_local()`; consumes the `local-offset` feature"
tech-stack:
  added:
    - "cargo feature `local-offset` on existing `time` dep (no new crate)"
  patterns:
    - "Wave-0 TDD RED-state: `#[ignore]`-skeletons carry `let _ = <const>;` bindings to keep target-behavior constants alive against `dead_code` linting until later waves consume them at runtime"
key-files:
  created: []
  modified:
    - "service_impl/Cargo.toml (time-dep features list: +`local-offset`)"
    - "service_impl/src/pdf_render.rs (mod test: +const FIXED_RENDER_TIMESTAMP, extended fixture, 3 tests removed, 6 tests added)"
decisions:
  - "D-50-13 enforced: `deterministic_bytes_for_same_input` removed — byte-determinism contract from v2.2 is explicitly dropped (timestamp varies per render call)."
  - "D-50-15 enforced: `sales_persons_sorted_by_id` and `build_sales_person_row_lists_bookings_time_ranges` removed — the global-id-sort + row-layout patterns are obsolete under the new slot-box-centric layout."
  - "D-50-14 anchored: `FIXED_RENDER_TIMESTAMP` = `2026-07-03 17:15 UTC` as `time::macros::datetime!` constant in mod test."
  - "D-50-07 fixture: `make_sales_person` now takes `is_paid: Option<bool>` — enables Volunteer-Suffix test without an extra fixture."
metrics:
  duration_seconds: N/A
  tasks_completed: 3
  files_created: 1
  files_modified: 3
  commits: 3
  tests_passing: 7
  tests_ignored: 6
  tests_failing: 0
  completed_date: 2026-07-03
---

# Phase 50 Plan 01: Wave-0 TDD-Setup for PDF-Renderer Rewrite — Summary

TDD RED-state anchor for the Phase 50 PDF-renderer rewrite: cargo feature `local-offset` activated on the `time` dep, `FIXED_RENDER_TIMESTAMP` test constant introduced, `make_sales_person` fixture extended by `is_paid`, three v2.2-legacy tests removed, and six new `#[ignore]`-marked unit tests added that encode the target Wave-2 behavior.

## What Was Built

### Task 1 — Feature-flag + fixture-Erweiterung (commit `8981b98`)

- `service_impl/Cargo.toml`: added `"local-offset"` to the `time`-dep feature list, so `OffsetDateTime::now_local()` compiles in Wave 3 (Pitfall 1 / RESEARCH §Standard Stack, D-50-12).
- `service_impl/src/pdf_render.rs` (`#[cfg(test)] mod test`):
  - Added `FIXED_RENDER_TIMESTAMP: time::OffsetDateTime = time::macros::datetime!(2026-07-03 17:15 UTC)` as `const` (D-50-14). Initially decorated with `#[allow(dead_code)]` because Task 1 does not yet reference it — Task 3 removes the attribute once the six new tests carry `let _ = FIXED_RENDER_TIMESTAMP;` bindings.
  - Extended fixture signature from `make_sales_person(id_hex, name)` to `make_sales_person(id_hex, name, is_paid: Option<bool>)` (D-50-07).
  - Updated three existing call sites (`all_active_sales_persons_appear`, `sales_persons_sorted_by_id`, `build_sales_person_row_lists_bookings_time_ranges`) to pass `Some(true)` explicitly for backward compat.
- `Cargo.lock` — incidental refresh from `cargo build` (transitive dep of time-crate feature activation), committed with Task 1.

### Task 2 — Drop 3 obsolete v2.2 tests (commit `3e7e42a`)

Removed the following tests from `mod test`, each replaced by a short "why gone / where moved" note kept as a comment so future readers understand the deletion:

1. `deterministic_bytes_for_same_input` — v2.2 byte-determinism guard. Explicitly obsoleted by D-50-13 because the render timestamp now varies per call.
2. `sales_persons_sorted_by_id` — v2.2 global sort assertion. Obsoleted by D-50-06 (names are alphabetical WITHIN a slot box, not globally). Wave 2 replaces it with `names_within_slot_alphabetical`.
3. `build_sales_person_row_lists_bookings_time_ranges` — v2.2 per-day row-layout test. Obsoleted by D-50-15 (row layout replaced by slot-box rendering).

Retained (per D-50-15 portable set):
- `empty_week_yields_valid_pdf_signature`
- `header_contains_year_and_week`
- `all_active_sales_persons_appear`
- `build_page_header_produces_expected_text`
- `build_day_column_headers_yields_seven_short_labels`
- `normalize_pdf_id_removes_variable_id_array`
- `find_all_subsequences_locates_multiple_occurrences`

All test helpers (`normalize_pdf_id`, `find_subsequence`, `find_all_subsequences`, `encode_ascii_to_pdf_hex`, `make_slot`, `make_booking`, `empty_week`) kept intact for Wave 2 reuse.

### Task 3 — 6 ignored RED-state skeletons (commit `1b04fa0`)

Added six `#[ignore = "Wave 2: erst nach Renderer-Rewrite grün — siehe 50-02-PLAN.md"]`-marked tests at the end of `mod test`. Each contains a concrete assertion (no `todo!()` / `unimplemented!()`) against the target Wave-2 behavior, calls the current Wave-1 4-param renderer (so the test compiles), and references `FIXED_RENDER_TIMESTAMP` via `let _ = FIXED_RENDER_TIMESTAMP;`.

| Test | Requirement | Encodes |
|---|---|---|
| `render_includes_timestamp_string` | PDF-02, D-50-16 | Fixed-timestamp string "Erstellt am 03.07.2026 17:15 Uhr" must appear in the content stream (hex-encoded). |
| `slot_boxes_sorted_by_start_time` | PDF-01, D-50-02, D-50-16 | Slots on the same day must render in ascending start-time order regardless of input Vec order. |
| `names_within_slot_alphabetical` | PDF-01, D-50-06, D-50-16 | Names inside a single slot box must be alphabetical (case-insensitive) regardless of booking-insertion order. |
| `unpaid_marker_suffix` | PDF-01, D-50-07, D-50-16 | Volunteer entries (`is_paid == Some(false)`) get " (freiwillig)" suffix appended to the name. |
| `sunday_column_hidden_when_no_sunday_slots` | PDF-01, D-50-08, D-50-16 | "So" column header must NOT appear when no Sunday slot exists. |
| `sunday_column_shown_when_at_least_one_sunday_slot` | PDF-01, D-50-08, D-50-16 | "So" column header MUST appear when at least one Sunday slot exists. |

Also removed the `#[allow(dead_code)]` attribute from `FIXED_RENDER_TIMESTAMP` — the new tests now reference it.

The 7th D-50-16 test (`now_local_fallback_to_utc_on_indeterminate_offset`) is deferred to Wave 3 (`50-03-PLAN.md`) because it belongs to service level (`pdf_shiftplan.rs`, D-50-12), not renderer level.

## Deviations from Plan

**None** — plan executed exactly as written, with one minor guardrail choice made during Task 1:

- **Micro-adjustment (not a deviation):** the plan text says "kompiliert nicht mit `dead_code`-Warning, weil in Wave-2-Tests genutzt". Between Task 1 and Task 3 the constant is declared but has no references, which would fail Clippy under `-D warnings`. Task 1 therefore temporarily decorates the const with `#[allow(dead_code)]`; Task 3 removes the attribute the moment it also adds the `let _ = FIXED_RENDER_TIMESTAMP;` bindings. Net effect after Task 3 matches the plan's target state (attribute absent, const referenced). Recorded here for traceability; no user permission needed (Rule 3 — blocking issue autofix).

## Verification

| Gate | Result |
|---|---|
| `cargo build -p service_impl` | ✅ green |
| `cargo test -p service_impl pdf_render --lib` | ✅ 7 passed, 6 ignored, 0 failed |
| `cargo clippy --workspace -- -D warnings` | ✅ green |
| `grep -c '#\[ignore = "Wave 2' service_impl/src/pdf_render.rs` | 6 |
| `grep -c 'todo!()\|unimplemented!()' service_impl/src/pdf_render.rs` | 0 |
| `grep -c 'const FIXED_RENDER_TIMESTAMP' service_impl/src/pdf_render.rs` | 1 |
| `grep -c 'fn make_sales_person(id_hex: u128, name: &str, is_paid: Option<bool>)'` | 1 |
| `grep -c 'local-offset' service_impl/Cargo.toml` | 1 |
| Deleted tests re-added? | no (0 matches for the 3 removed fn names) |
| 7 portable tests intact? | yes (all 7 fn names still present) |

## Commits

| # | Hash | Type | Description |
|---|---|---|---|
| 1 | `8981b98` | feat | Activate `local-offset` feature + extend `make_sales_person` fixture (Task 1) |
| 2 | `3e7e42a` | refactor | Drop 3 obsolete v2.2 renderer tests (Task 2) |
| 3 | `1b04fa0` | test | Add 6 ignored renderer RED-state skeletons (Task 3) |

## Known Stubs

**None.** The six `#[ignore]`-marked tests are not stubs — they carry real assertions against the target Wave-2 behavior. The `#[ignore]` marker is a documented staging device (D-50-16 / plan objective §"Wave-1-kompatibel") that Wave 2 (50-02-PLAN.md) removes as it lands the new renderer.

## Threat Flags

**None.** This plan touches only test-scaffolding and Cargo features — no new network endpoints, no auth paths, no file access, no schema changes.

## Hand-off to Wave 2 (50-02)

Wave 2 will:
1. Rewrite `render_shiftplan_week_pdf` for slot-box layout, timestamp-inclusion, name-sorting, unpaid-suffix, dynamic Sunday column.
2. Extend the renderer signature from 4 params to 5 (add `now: OffsetDateTime`) per D-50-11.
3. Update ALL Wave-1 test call sites (portable + new) to pass the 5th parameter (`FIXED_RENDER_TIMESTAMP` in mod test).
4. Remove the `#[ignore = "Wave 2: ..."]` markers from the six skeleton tests as each assertion becomes reachable in green.
5. Keep the portable test set green throughout.

The `local-offset` feature is Wave-3-ready — no further Cargo.toml changes needed until then.

## Self-Check: PASSED

- `service_impl/Cargo.toml` — FOUND, contains `local-offset` ✓
- `service_impl/src/pdf_render.rs` — FOUND, contains `FIXED_RENDER_TIMESTAMP` + 3-arg fixture + 6 `#[ignore]` tests, 0 removed-test names ✓
- Commit `8981b98` — FOUND in git log ✓
- Commit `3e7e42a` — FOUND in git log ✓
- Commit `1b04fa0` — FOUND in git log ✓
