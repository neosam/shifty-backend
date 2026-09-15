---
phase: 56-wochen-cron-rollout-backfill-f4
plan: 04
subsystem: rebooking-backfill-rest
tags: [rebooking, backfill, rest, axum, utoipa, iso-week, hr-admin, pagination, f4]

# Dependency graph
requires:
  - phase: 56-wochen-cron-rollout-backfill-f4
    plan: 02
    provides: "RebookingReconciliationService::backfill_range + BackfillPreviewItem / SkipReason domain types (HR-Admin gated, dry-run/apply, upper-bound clamp)"
  - phase: 55-f3-manuelle-umbuchung-hr-alert
    provides: "rest/src/rebooking.rs handler+error_handler pattern, rest-types/src/rebooking.rs TO+from_domain pattern, /admin/impersonate nest analog"
provides:
  - "POST /admin/rebooking/backfill?from=YYYY-Www&dry_run={true|false}[&page&per_page] (F4 one-time historical rollout, HR-Admin gated)"
  - "BackfillRequestTO / BackfillPreviewTO / BackfillPreviewItemTO wire DTOs (ToSchema + OpenAPI)"
  - "BackfillPreviewTO::from_domain + BackfillPreviewItemTO::from_domain (skip_reason -> stable lowercase String)"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "REST handler is a thin delegate to backfill_range — no business logic in REST (auth gate + upper-bound clamp + write-path all live in the service, Plan 02)"
    - "from=YYYY-Www parsed+validated to a 400 BEFORE any service call (T-56-11 DoS-bound: garbage never triggers iteration)"
    - "HR-Admin gate NOT duplicated in REST — relies on the service gate (first await) + Forbidden->403 error mapping (T-56-10)"
    - "Pagination slices items AFTER from_domain mapping; total_candidates stays the un-sliced count (D-05)"
    - "skip_reason enum -> stable lowercase String (no_excess|already_processed|week_locked|toggle_off) so the wire contract is decoupled from the Rust enum"
    - "REST-layer coverage split: parsing/validation via in-crate unit tests, contract via OpenAPI-surface integration test (established codebase pattern, no live DB+auth oneshot harness)"

key-files:
  created:
    - rest/tests/rebooking_backfill_test.rs
  modified:
    - rest-types/src/rebooking.rs
    - rest/src/rebooking.rs
    - rest/src/lib.rs

key-decisions:
  - "REST integration coverage follows the codebase's OpenAPI-surface pattern (openapi_surface.rs / content_type_surface.rs), not a live-app DB+auth oneshot. Rationale (documented in content_type_surface.rs): a live roundtrip needs per-handler DB seeding + mocking the full RestStateDef (~45 associated services); the dry-run/apply/403 behavior is already exhaustively covered at the SERVICE layer (Plan 02, 7 tests). The REST layer's own logic — from-week parsing/validation (400) and pagination — is unit-tested in-crate; the OpenAPI contract (path present, 200/400/403 declared, DTO schemas registered) is asserted in the integration test."
  - "dry_run defaults to false (Apply only on explicit opt-in is the safe default for an absent param); any dry_run value other than 'true'/'false' -> 400 rather than silently coercing."
  - "from-week validation clamps week to 1..=weeks_in_year(year) using time::util::weeks_in_year. 2026 is a 53-week ISO year (W53 valid); the malformed test uses 2025-W53 (52-week year) for the out-of-range case."

requirements-completed: [REB-AUTO-04]

coverage:
  - id: T1
    description: "TOs: BackfillRequestTO serde round-trip; BackfillPreviewTO/ItemTO serde round-trip; from_domain maps a mixed vector (one booked, one already_processed) and counts total_candidates + already_processed correctly with skip_reason->lowercase string"
    requirement: "REB-AUTO-04"
    verification:
      - kind: unit
        ref: "rest-types/src/rebooking.rs (backfill_request_serde_roundtrip, backfill_preview_from_domain_maps_and_counts)"
        status: pass
    human_judgment: false
  - id: T2
    description: "from-week parse/validation matrix (valid YYYY-Www incl. W01 + 53-week year; malformed shapes -> BadRequest/400) + OpenAPI contract (path /admin/rebooking/backfill present, 200/400/403 declared, request/response DTOs registered as schemas)"
    requirement: "REB-AUTO-04"
    verification:
      - kind: unit
        ref: "rest/src/rebooking.rs (parse_iso_week_accepts_valid_yyyy_www, parse_iso_week_rejects_malformed_input)"
        status: pass
      - kind: integration
        ref: "rest/tests/rebooking_backfill_test.rs (3 OpenAPI-surface tests)"
        status: pass
    human_judgment: false

# Metrics
duration: 11min
completed: 2026-07-26
status: complete
---

# Phase 56 Plan 04: F4 Backfill REST Endpoint (POST /admin/rebooking/backfill) Summary

**Exposes the Plan-02 backfill engine over HTTP as a thin, HR-Admin-gated delegate: `POST /admin/rebooking/backfill?from=YYYY-Www&dry_run={true|false}` parses+validates the from-week to a 400 before touching the service, delegates to `backfill_range` (which owns the auth gate, upper-bound clamp, and dry-run/apply write-path), maps `Vec<BackfillPreviewItem>` to `BackfillPreviewTO` with pagination, and carries the whole request/response contract into OpenAPI.**

## Performance

- **Duration:** ~11 min
- **Started:** 2026-07-26T06:33:12Z
- **Completed:** 2026-07-26T06:44:24Z
- **Tasks:** 2 (both TDD)
- **Files:** 4 (1 created, 3 modified)
- **New tests:** 5 (2 rest-types unit + 2 rest in-crate unit + 3 rest integration = 7 assertions across 5 test fns; full workspace green)

## Accomplishments

- **Wire DTOs (`rest-types/src/rebooking.rs`):** `BackfillRequestTO { from, dry_run }`, `BackfillPreviewItemTO` (mirrors every `BackfillPreviewItem` field, `skip_reason: Option<String>`), `BackfillPreviewTO { dry_run, items, total_candidates, already_processed }` — all with the crate's standard `#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]`.
- **`from_domain` mappings** under `#[cfg(feature = "service-impl")]`: `BackfillPreviewItemTO::from_domain` maps one domain item; `BackfillPreviewTO::from_domain(&[..], dry_run)` computes `total_candidates = items.len()` and `already_processed = count(skip_reason == AlreadyProcessed)`. A shared `skip_reason_to_string` helper projects the `SkipReason` enum to a stable lowercase string (`no_excess | already_processed | week_locked | toggle_off`) so both the item mapping and the count use the same names.
- **`trigger_backfill<RestState>` handler (`rest/src/rebooking.rs`):** extracts `from` / `dry_run` / optional `page` / `per_page` from query params; parses+validates `from` via `parse_iso_week` (→ 400 on malformed, before any service call); delegates to `rebooking_reconciliation_service().backfill_range(from_week, dry_run, context.into(), None)` inside `error_handler`; maps via `BackfillPreviewTO::from_domain`, slices `items` by page/per_page (keeping `total_candidates` un-sliced), returns 200 JSON.
- **`parse_iso_week` helper:** validates `YYYY-Www` shape (4-digit year, `-W` separator, numeric week) and range (`1..=weeks_in_year(year)`); every malformed shape → `RestError::BadRequest` → 400. This is the REST-layer arm of the DoS mitigation (T-56-11) — garbage input never reaches `backfill_range`.
- **Route + OpenAPI registration (`rest/src/lib.rs` + `rest/src/rebooking.rs`):** `generate_admin_route()` (`POST /backfill`) nested at `/admin/rebooking` next to `/admin/impersonate`; `(path = "/admin/rebooking", api = rebooking::RebookingApiDoc)` added to the ApiDoc nest block; `trigger_backfill` added to `RebookingApiDoc.paths` and the three new TOs to its `components(schemas(..))`.

## Task Commits

1. **Task 1: BackfillRequestTO/PreviewTO DTOs + from_domain** — `55ed21d3` (feat)
2. **Task 2: trigger_backfill handler + route + ApiDoc + validation + tests** — `1bd4368a` (feat)

## Files Created/Modified

- `rest-types/src/rebooking.rs` (modified) — 3 new TOs + 2 `from_domain` impls + `skip_reason_to_string` helper; 2 new unit tests (round-trip + from_domain mapping/count).
- `rest/src/rebooking.rs` (modified) — `trigger_backfill` handler, `generate_admin_route`, `parse_iso_week`, `#[utoipa::path]` with from/dry_run/page/per_page params + 200/400/403 responses, RebookingApiDoc path+schema entries, 2 in-crate parse unit tests.
- `rest/src/lib.rs` (modified) — `/admin/rebooking` ApiDoc nest entry + runtime nest registration next to `/admin/impersonate`.
- `rest/tests/rebooking_backfill_test.rs` (created) — 3 OpenAPI-surface contract tests (path registered, DTOs registered, 200/400/403 declared).

## Decisions Made

- **REST integration coverage via the OpenAPI-surface pattern, not a live oneshot.** The codebase deliberately avoids full-app DB+auth roundtrip tests (see `content_type_surface.rs` rationale). The dry-run-writes-nothing / apply-persists / HR-gate-403 behavior is already covered exhaustively at the service layer (Plan 02's `rebooking_backfill.rs`, 7 tests). What the REST layer uniquely owns — from-week parsing/validation and pagination — is unit-tested in-crate; the endpoint contract (path, status codes, DTO schemas) is asserted in the integration test. This keeps the test aligned with the plan's threat model (T-56-10 403 / T-56-11 400) without re-mocking ~45 `RestStateDef` services.
- **`dry_run` defaults to `false` when the param is absent, but any non-`true`/`false` value is a 400.** Apply requires an explicit `dry_run=false` or an absent param; the endpoint never silently coerces a typo (e.g. `dry_run=yes`).
- **from-week range uses `time::util::weeks_in_year`.** 2026 is a 53-week ISO year, so `2026-W53` is valid; the out-of-range malformed test case uses `2025-W53` (a 52-week year).

## Deviations from Plan

### Clarified / adjusted

**1. [Rule 1 - Test correctness] Malformed from-week test case swapped 2026-W53 -> 2025-W53**
- **Found during:** Task 2 (running the parse unit tests)
- **Issue:** The initial malformed matrix treated `2026-W53` as out-of-range, but 2026 is a 53-week ISO year (`weeks_in_year(2026) == 53`), so `2026-W53` is valid and the assertion failed.
- **Fix:** Used `2025-W53` (a 52-week year) for the "week beyond weeks_in_year" case; kept `2026-W99` for the clearly-out-of-range case.
- **Files:** rest/tests... (in-crate unit test in rest/src/rebooking.rs)
- **Commit:** `1bd4368a`

**2. [Rule 1 - Clippy gate] iter().any() -> contains() in the integration test**
- **Found during:** Task 2 (`cargo clippy --workspace -- -D warnings`)
- **Issue:** `clippy::manual_contains` flagged two `actual.iter().any(|x| *x == expected)` lookups in the OpenAPI-surface test.
- **Fix:** Replaced with `actual.contains(&expected)`.
- **Files:** rest/tests/rebooking_backfill_test.rs
- **Commit:** `1bd4368a`

**Total deviations:** 2 test-correctness/lint adjustments. No scope change; no architectural change.

## Threat Model Compliance

| Threat ID | Category | Mitigation in this plan | Verified |
|-----------|----------|-------------------------|----------|
| T-56-10 | Elevation of Privilege (non-HR-Admin backfill) | HR gate stays in `backfill_range` (first await, Plan 02); REST does NOT duplicate it and lets `Forbidden -> 403` via `error_handler`. The 403 is declared in the OpenAPI contract. | contract test asserts 403 declared; service-layer HR-gate test (Plan 02) asserts the enforcement |
| T-56-11 | Denial of Service (malformed/oversized from-week) | `parse_iso_week` rejects malformed/out-of-range from-week -> 400 BEFORE any service call; service clamps upper bound to last completed week (Plan 02); pagination bounds the response. | `parse_iso_week_rejects_malformed_input` (11 shapes) + contract test (400 declared) |
| T-56-12 | Tampering (dry_run=true writing) | `dry_run` threads unchanged into `backfill_range`, which opens no write tx when true (Plan 02); REST adds no write path of its own. | service-layer dry-run no-write test (Plan 02) |
| T-56-SC | Tampering (npm/pip/cargo installs) | No new dependencies — no Cargo.toml change. | verified (git diff shows no Cargo.toml touched) |

No new externally-reachable surface beyond the one planned endpoint; no new schema/trust-boundary changes.

## Verification

- `cargo test -p rest-types rebooking` — 2 passed (default features); `cargo test -p rest-types --features service-impl rebooking` — 3 passed (incl. from_domain)
- `cargo test -p rest --lib parse_iso_week` — 2 passed
- `cargo test -p rest --test rebooking_backfill_test` — 3 passed
- `cargo test --workspace` — green (no non-zero failures across all crates + doctests)
- `cargo clippy --workspace --all-targets -- -D warnings` — clean (hard gate)
- `cargo build` — workspace + shifty_bin build green (route wiring + ApiDoc compile)

## Known Stubs

None. The handler is fully wired to the real `backfill_range` service call; DTOs map real domain data. No hardcoded/placeholder data paths.

## Deferred Items

- **Docs-Freshness-Gate (CLAUDE.md):** this plan added a REST endpoint but did NOT touch a docs trigger file (no `service/**`, `dao/**`, `migrations/`, `permission.rs`, or `main.rs` change — the endpoint is a thin delegate). Per the trigger table, no `docs/` update is mandated by this diff. The F4 feature doc (`docs/features/F*.md`) update for the whole auto-cron/backfill feature is already deferred by Plan 02/03 to phase close / `/gsd-docs-update`; noted here for the milestone-close gate.

## Next Phase Readiness

- The F4 backfill is now reachable end-to-end: HTTP -> validation -> `backfill_range` -> DTO. Plans 05/06 (rollout wiring / frontend) can consume the `POST /admin/rebooking/backfill` contract and the `BackfillPreviewTO` schema directly from OpenAPI.

---
*Phase: 56-wochen-cron-rollout-backfill-f4*
*Completed: 2026-07-26*

## Self-Check: PASSED

- Created file present on disk: `rest/tests/rebooking_backfill_test.rs`.
- Both task commits present in git history (`55ed21d3`, `1bd4368a`).
- Full workspace `cargo test` green; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo build` green.
