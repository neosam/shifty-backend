# Project Research Summary

**Project:** Shifty v2.1 — Schichtplan- & Reporting-Erweiterungen
**Domain:** Rust/Axum/SQLite/Dioxus shift-planning app — feature additions to an existing monorepo
**Researched:** 2026-07-01
**Confidence:** HIGH (all research derived from direct codebase inspection; no speculative web-research findings)

## Executive Summary

Shifty v2.1 adds two new backend-driven features (WST-01 calendar-week status/locking and AVG-01 average attendance for flexible employees) plus one isolated frontend bugfix (SDF-Desync: "Anlegen" button stays disabled after special-day create). All four research streams independently confirm that **zero new crate dependencies are needed** — every pattern required by both features is already proven in the codebase across 38+ prior phases. The implementation is entirely additive: new migration, new DAO/service/REST modules for WST-01; new method on an existing service + new REST endpoint for AVG-01; a one-line signal-reset removal for SDF-Desync.

The central architectural decision for WST-01 is correct tier placement of the lock gate: the new `WeekStatusService` is a Basic-tier entity manager, but the lock gate itself belongs in `ShiftplanEditService` (business-logic tier) because it is a cross-entity invariant. A new `ShiftplanEditService::delete_booking` method is required to close the only genuine non-shiftplanner write bypass (`DELETE /booking/{id}` currently routes directly to `BookingService::delete`, bypassing all business-logic gates). The gate must execute inside the same transaction as the write to prevent TOCTOU races.

The central open question for AVG-01 is definitional: the existing A-22-1 formula (`average_worked_hours_per_week`) excludes weeks where any absence category is present and worked hours are zero, but AVG-01's stated intent is vacation-only exclusion. These are not the same formula. The discuss-phase must resolve eight open decision points (D-AVG-01 through D-AVG-08) before implementation begins. AVG-01 is confirmed as a pure read-aggregate (no new `BillingPeriodValueType`), so `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays at 12 — provided the discuss-phase does not reverse this decision.

## Key Findings

### Recommended Stack

No new packages. The full Rust workspace (SQLx 0.8.2, Axum 0.8.7, utoipa 5, `time` 0.3.36, Dioxus 0.6.1, serde 1.0, mockall 0.13, uuid 1.8.0, async-trait, thiserror 2.0) already contains every primitive needed. The critical post-implementation gate is `cargo sqlx prepare --workspace` (run in `nix develop`) after any new `query!`/`query_as!` macro — this step must be committed before autonomous phases complete, or CI fails silently.

**Core technologies (reused, no version changes):**
- **SQLx 0.8.2** — compile-time SQLite queries; new `week_status` table uses the `week_message` migration as an exact template.
- **`time` 0.3.36** — ISO week arithmetic (`Date::from_iso_week_date`, `Date::to_iso_week_date`) already used throughout; no `chrono` needed for new features.
- **Axum 0.8.7 + utoipa 5** — REST layer; new `rest::week_status` module and new endpoint in `rest::report`; all with `#[utoipa::path]` annotations.
- **Dioxus 0.6.1** — frontend; status badge and AVG-01 display follow existing patterns; no new npm or WASM deps.
- **mockall 0.13** — new `WeekStatusService` trait gets `#[automock]`; lock-gate unit tests mock it.

**Integration patterns to copy verbatim:**

| Pattern needed | Copy from |
|---|---|
| Migration: soft-delete + partial unique on `(year, calendar_week)` | `migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql` |
| TEXT enum discriminant + manual `match` in `TryFrom` | `dao_impl_sqlite/src/extra_hours.rs`, `dao_impl_sqlite/src/special_day.rs` |
| `(year, calendar_week)` composite key + DAO CRUD | `dao_impl_sqlite/src/week_message.rs` |
| `SHIFTPLANNER_PRIVILEGE` gate in Basic-tier service | `service_impl/src/week_message.rs` |
| `is_shiftplanner` capture + hard-block in write path | `service_impl/src/shiftplan_edit.rs` |
| Vacation-week extraction with `to_iso_week_date()` | `service_impl/src/absence.rs` |
| Week-range worked-hours aggregation | `dao_impl_sqlite/src/shiftplan_report.rs` |

### Expected Features

**WST-01 — Calendar-Week Status (must have, build in full):**
- Four-state model: `None → InPlanning → Planned → Locked` (matches industry conventions; Planday/WhenIWork use two/three states; four states is a deliberate refinement for the weekly planning cycle).
- Visual status badge on week header (color-coded pill: grey/yellow/green/red).
- Planner-only state transitions (any direction, any state); non-planners read-only on status.
- Lock gate on all booking/slot write paths for non-shiftplanners; shiftplanner bypasses.
- i18n: de/en/cs for all four state labels.
- Explicit "In Planning" state mirrors the pre-publish draft phase already used in practice.

**WST-01 write paths requiring lock gate injection (exhaustive list):**
1. `ShiftplanEditService::book_slot_with_conflict_check` (booking create)
2. `ShiftplanEditService::modify_slot` (slot modify, multi-week)
3. `ShiftplanEditService::modify_slot_single_week` (slot modify, single-week override)
4. `ShiftplanEditService::remove_slot` (slot delete)
5. `ShiftplanEditService::copy_week_with_conflict_check` (copy bookings to destination week)
6. **NEW** `ShiftplanEditService::delete_booking` (booking delete — replaces direct `BookingService::delete` call in `DELETE /booking/{id}` REST handler)

**AVG-01 — Average Attendance (resolve discuss-phase decisions first):**
- Pure read-aggregate in `ReportingService`; no new tables; no snapshot version bump.
- "Flexible" = `EmployeeWorkDetails.is_dynamic == true` (confirmed field exists).
- Predecessor formula A-22-1 (`average_worked_hours_per_week`) is related but NOT identical — must not be reused directly without reviewing denominator exclusion rules.
- Display location: billing period report (new column/row per employee) or standalone endpoint; discuss-phase decides.
- Denominator decisions (D-AVG-01 through D-AVG-08) must be explicit before implementation.

**SDF-Desync — Special Days Button Bugfix (low risk, isolated):**
- After successful special-day create, Option 2: do not reset any form state (avoids controlled-select desync, avoids D-25-06 class bug).
- Isolated frontend-only fix; no backend changes.

**Defer to v2.2+:**
- Publish-notification when week moves to Planned.
- Bulk week-status operations (lock all past weeks at once).
- AVG-01 snapshot persistence / `BillingPeriodValueType` row.
- AVG-01 trend over multiple billing periods.
- Configurable absence-exclusion categories.

### Architecture Approach

WST-01 follows the established two-tier service pattern: `WeekStatusServiceImpl` is Basic tier (only DAO + PermissionService + TransactionDao), while the lock gate lives in `ShiftplanEditServiceImpl` (Business-Logic tier), which already aggregates all booking/slot write paths. `ShiftplanEditService` gains `WeekStatusService` as a new dependency wired in `shifty_bin/src/main.rs`. AVG-01 is a pure method extension on the existing `ReportingService` (Business-Logic tier) with no new deps. HTTP 423 (Locked) is the target status code for locked-week write attempts — confirm 423 vs. 409 in discuss-phase.

**New and modified components:**

| Component | Status | Tier |
|---|---|---|
| `dao::week_status::WeekStatusDao` + `dao_impl_sqlite::WeekStatusDaoImpl` | NEW | DAO |
| `service::week_status::WeekStatusService` + `ServiceImpl` | NEW | Basic |
| `service::ServiceError::WeekLocked { year, week }` | MODIFIED | — |
| `service_impl::shiftplan_edit::ShiftplanEditServiceImpl` | MODIFIED | Business-Logic (adds dep + gate + `delete_booking`) |
| `service::reporting::ReportingService` + `ReportingServiceImpl` | MODIFIED | Business-Logic (new AVG-01 method) |
| `rest::week_status` (GET/PUT/DELETE/GET-by-year) | NEW | REST |
| `rest::booking::delete_booking` handler | MODIFIED | REST (re-routes to `ShiftplanEditService`) |
| `rest::report` (new attendance-average endpoint) | MODIFIED | REST |
| `rest-types` | MODIFIED | Shared DTOs |
| `shifty_bin::main.rs` | MODIFIED | DI wiring |
| `shifty-dioxus` (api, state, page, i18n) | MODIFIED | Frontend |

**DI construction order in `main.rs`:**
1. (existing) permission, clock, uuid services
2. (existing) basic-tier entity managers
3. **(NEW)** `week_status_dao` → `week_status_service` [basic tier]
4. (existing) business-logic services
5. **(MODIFIED)** `shiftplan_edit_service` now receives `week_status_service` as an additional dep

### Critical Pitfalls

1. **ISO-week year vs. Gregorian year in the `(year, calendar_week)` lock key** — Dec 29–31 of a year may belong to ISO week 1 of the next year; storing Gregorian year silently miskeys the lock row and lets writes through on locked weeks. Fix: always derive `year` from `ShiftyDate::from_ymd(...).to_iso_week_date().0`, never from `date.year()`. Add unit tests for week-53 and year-boundary dates explicitly.

2. **Lock gate check outside the write transaction (TOCTOU race)** — checking lock status before opening the write transaction allows two concurrent requests to both pass the check. Fix: read lock status inside the same `BEGIN IMMEDIATE` transaction that performs the write; gate must be in the service layer, not the REST handler.

3. **Incomplete write-path audit — `modify_slot_single_week` and `DELETE /booking/{id}` bypass** — History (Phase 23/24) shows single-week slot paths are independently missed. Fix: implement a shared `assert_week_not_locked(year, week, context, tx)` helper called at the top of all six write methods; add a test matrix (6 paths × locked/unlocked).

4. **AVG-01 denominator trap: A-22-1 formula excludes all absences; AVG-01 specified for vacation-only** — directly reusing `average_worked_hours_per_week` will silently produce wrong results for employees with sick leave. Fix: discuss-phase decision D-AVG-04 must be explicit; implement a separate function if the rule differs from A-22-1; never modify A-22-1 itself (would break existing reporting).

5. **Missing `.sqlx/` offline cache after new queries breaks CI** — WST-01 adds new `query!`/`query_as!` macros; CI uses `SQLX_OFFLINE=true` and needs the corresponding `.sqlx/` entries committed. Fix: every WST-01 DAO phase must end with `cargo sqlx prepare --workspace` (in `nix develop`) + commit of updated `.sqlx/`.

6. **`cargo clippy --workspace -- -D warnings` failures from new enum patterns** — `nix build` enforces Clippy as a hard gate; `cargo test`/`cargo build` do not run it. Fix: run Clippy explicitly in every phase gate; name the `None` variant `Unset` or `Open` to avoid shadowing `Option::None`.

7. **Stale lock state / `SelectInput` desync (D-25-06 class) for WST-01 frontend** — Dioxus controlled `<select>` does not always re-apply `value=` when the signal value is unchanged. Fix: use read-only badge + action button pattern (not a controlled `SelectInput`); reload status from server after any status change or booking 423 response.

8. **Snapshot version drift if AVG-01 is inadvertently persisted** — `CURRENT_SNAPSHOT_SCHEMA_VERSION` is currently 12; adding a new `BillingPeriodValueType` without bumping it causes validators to run on mismatched schemas. Fix: discuss-phase must record explicit no-persist decision; executor must not add any new `BillingPeriodValueType` variant.

## Implications for Roadmap

Based on combined research, the natural phase structure is:

### Phase 1: WST-01 Discuss
**Rationale:** WST-01 has three unresolved permission questions (who sets status, who bypasses lock, allowed transitions + HTTP code) plus the UI approach decision (badge+button to prevent D-25-06 desync). These gate the service design.
**Delivers:** Explicit decision log: HTTP 423 vs 409, transition rules, permission model, UI approach.
**Addresses:** Pitfalls P2 (TOCTOU), P3 (write-path audit), P4 (permission ambiguity), P5 (frontend desync).
**Research flag:** Skip — codebase inspection is sufficient; no external research needed.

### Phase 2: WST-01 Backend (Migration + DAO + Service + REST)
**Rationale:** Schema must exist before service; service before REST; REST before frontend. All patterns are proven — this is a copy-and-adapt phase.
**Delivers:** `week_status` table; `WeekStatusService` (Basic tier); lock gate in all 6 write paths including new `delete_booking`; REST CRUD endpoints with OpenAPI.
**Avoids:** P1 (ISO-week year), P2 (TOCTOU — gate inside transaction), P3 (all 6 paths gated), P8 (`.sqlx/` committed), P9 (Clippy gate).
**Gates:** `cargo test`, `cargo clippy --workspace -- -D warnings`, `cargo sqlx prepare --workspace`, week-53 unit tests, 6-path × 2-state test matrix.
**Research flag:** Skip — all patterns verified from codebase.

### Phase 3: WST-01 Frontend
**Rationale:** Depends on backend REST endpoints existing and returning correct status codes.
**Delivers:** Status badge in week header, action button for shiftplanners, disabled booking controls when locked, inline 423 banner (non-blocking), i18n de/en/cs.
**Avoids:** P5 (badge+button, not controlled SelectInput; reload status from server after each change).
**Research flag:** Skip — Dioxus frontend patterns are established.

### Phase 4: AVG-01 Discuss
**Rationale:** AVG-01 has eight open definitional questions (D-AVG-01 through D-AVG-08) that must be decided before any implementation. Attempting to implement without these leads to wrong denominator (P6) or scope leak (P10).
**Delivers:** Explicit decisions on: reference period, numerator, denominator exclusion rule (vacation-only vs. all absences), employee scope definition (`is_dynamic == true`?), display location, minimum data threshold, no-persist confirmation.
**Addresses:** Pitfalls P6 (denominator), P7 (snapshot version), P10 (flexible employee definition).
**Key open question for owner:** Does "vacation weeks excluded from denominator" mean (B) any vacation day in the week, or (C) full-week vacation only? Sick-leave week treatment must be explicit.

### Phase 5: AVG-01 Backend + Frontend
**Rationale:** Pure additive phase after discuss decisions are recorded. No new DB tables (derive-on-read); no snapshot bump. Lower risk than WST-01 because it only reads data.
**Delivers:** New `ReportingService::get_attendance_average_for_flexible_employees` method; `GET /report/attendance-average/{year}` endpoint (HR-gated); frontend display; i18n de/en/cs.
**Avoids:** P6 (correct denominator, not A-22-1 directly), P7 (no new `BillingPeriodValueType`), P10 (server-side `is_dynamic` filter).
**Gates:** 4-scenario unit test (pure vacation week, sick-leave week, partial vacation + hours, empty week), mixed flexible/fixed employee exclusion test, Clippy gate.
**Research flag:** Skip — ReportingService extension is straightforward.

### Phase 6: SDF-Desync Frontend Bugfix
**Rationale:** Isolated, low-risk, no backend changes. Placed last to avoid blocking milestone if other phases are delayed.
**Delivers:** After successful special-day create, do not reset form state (Option 2); "Anlegen" button re-enables correctly.
**Avoids:** D-25-06 controlled-select desync class.
**Research flag:** Skip — one-line fix with established pattern.

### Phase Ordering Rationale

- WST-01 before AVG-01: WST-01 modifies existing write paths (higher regression risk) and should be verified stable before adding AVG-01's read-only extension.
- Discuss phases before implement phases: Both WST-01 and AVG-01 have open decisions that cause wrong implementation if skipped.
- Backend before frontend within each feature: Frontend depends on REST contract being finalized.
- SDF-Desync last: Isolated; never blocks other phases; trivially reversible.

### Research Flags

Phases needing `/gsd-plan-phase --research-phase`:
- **None** — all patterns are verified from the live codebase; no external research is needed for any phase.

Phases with standard/established patterns (skip research):
- **All 6 phases** — research is complete and HIGH confidence. Open items are product-owner decisions (D-AVG-01 through D-AVG-08, permission model questions), not technical unknowns.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All verified by direct source inspection; zero ambiguity on dependencies |
| Features | MEDIUM | WST-01 scope is clear; AVG-01 has 8 definitional decisions that are product-owner choices, not researchable unknowns |
| Architecture | HIGH | All integration points verified; DI construction order confirmed; tier classification matches CLAUDE.md rules |
| Pitfalls | HIGH | Derived from direct code inspection + documented project memory (SQLx prepare, Clippy gate, D-25-06, service tier rules) |

**Overall confidence:** HIGH for implementation approach; MEDIUM for AVG-01 feature definition (requires owner decisions before planning).

### Gaps to Address

- **AVG-01 denominator rule (D-AVG-03 / D-AVG-04):** Is "vacation week" defined as any vacation day in the week, or full-week vacation only? Are sick-leave weeks included or excluded from the denominator? Must be decided in discuss-phase before plan-phase.
- **AVG-01 employee scope (D-AVG-05):** Confirm "flexible" = `EmployeeWorkDetails.is_dynamic == true`. If no such field exists, decide whether to add it or use `expected_hours == 0.0` as the predicate. Server-side filter strongly preferred.
- **AVG-01 display location (D-AVG-06):** Billing period report (existing view, minimal new surface) vs. standalone attendance page. Owner decision.
- **WST-01 HTTP status code:** 423 (Locked, semantically correct) vs. 409 (Conflict, used elsewhere in codebase). Decide in discuss-phase and record as a decision.
- **WST-01 allowed state transitions:** Are all transitions valid in both directions (planner only), or are some transitions forbidden? Decide in discuss-phase.

## Sources

### Primary (HIGH confidence — direct codebase inspection)
- `dao_impl_sqlite/src/week_message.rs`, `special_day.rs`, `extra_hours.rs` — composite key + TEXT enum patterns
- `migrations/sqlite/20250123000000_add-week-message-table.sql`, `20260629000000_create-vacation-entitlement-offset.sql` — migration templates
- `service/src/permission.rs` — `SHIFTPLANNER_PRIVILEGE` constant
- `service_impl/src/week_message.rs`, `shiftplan_edit.rs` — permission gate patterns + write path enumeration
- `service_impl/src/absence.rs`, `dao_impl_sqlite/src/shiftplan_report.rs` — week-range data sources for AVG-01
- `service/src/reporting.rs` — A-22-1 formula (`average_worked_hours_per_week`) and `EmployeeWeeklyStatistics`
- `service_impl/src/billing_period_report.rs` — `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12`
- `shifty-dioxus/src/component/form/inputs.rs` — `SelectInput` D-25-06 controlled-mode behaviour
- `rest/src/booking.rs`, `rest/src/shiftplan_edit.rs` — current routing for `DELETE /booking/{id}` bypass
- `shifty-backend/CLAUDE.md` — snapshot version bump rules, Clippy hard gate, SQLx offline cache requirement

### Secondary (MEDIUM confidence — industry research)
- Planday, When I Work, Deputy, Shiftboard, Dayforce documentation — week lifecycle state conventions
- Oyster HR, Patriot Software, BASUSA, Hubstaff — average attendance metric definitions and denominator conventions

---
*Research completed: 2026-07-01*
*Ready for roadmap: yes*
