//! Phase 8 (Plan 08-03) — OpenAPI surface-assertion test.
//!
//! Background
//! ----------
//! The previous incarnation of this test (`openapi_snapshot.rs`, removed in
//! commit `fdb70b5` on 2026-05-03) used `insta::assert_json_snapshot!` to pin
//! the entire `ApiDoc::openapi()` document — including `info.version`, which
//! is sourced from `rest/Cargo.toml`. Every version bump tripped the snapshot
//! and forced a manual `cargo insta accept` for noise rather than real API
//! drift.
//!
//! The user-approved replacement (08-03 Option B) is this lighter,
//! version-agnostic surface-assertion test:
//!
//! * It builds the live `ApiDoc::openapi()` document.
//! * It asserts that a curated set of expected REST paths exist in `paths`.
//! * It asserts that the named schemas under `components.schemas` contain
//!   the key DTOs that frontend / contract consumers depend on.
//! * It does NOT compare `info.version` or any field sourced from
//!   `Cargo.toml`. Version bumps no longer trip the test.
//! * It uses plain `assert!` / `assert_eq!`. No `insta` dependency.
//!
//! Drift detection scope
//! ---------------------
//! This test pins ~10–15 representative paths covering each major domain.
//! The goal is to detect "this entire endpoint silently disappeared", not to
//! exhaustively enumerate every route. Schema bodies are intentionally NOT
//! pinned — we only check the schema NAMES exist, so adding/removing fields
//! on a DTO does not require a test update.
//!
//! Determinism
//! -----------
//! The test is deterministic: it iterates the `paths` map by exact-string
//! lookup and the `components.schemas` map likewise. No HashMap-iteration
//! ordering is observed. Three consecutive runs MUST pass green.

use rest::ApiDoc;
use utoipa::OpenApi;

/// Representative subset of REST paths that must exist in the OpenAPI
/// surface. Each entry covers one major domain — disappearance signals
/// either an accidental removal during refactor or an unintended nest-prefix
/// change. Paths are stored as effective/full paths (i.e. with the nest
/// prefix applied), exactly as utoipa renders them in `paths`.
///
/// Phase 8 additions: `/vacation-balance/{sales_person_id}/{year}` and
/// `/vacation-balance/team/{year}` (Wave 2).
const EXPECTED_PATHS: &[&str] = &[
    // Phase 8 — VacationBalance (new in Wave 2).
    "/vacation-balance/{sales_person_id}/{year}",
    "/vacation-balance/team/{year}",
    // Phase 8 Plan 08-07 Gap-Closure — Feature-Flag read endpoint.
    "/feature-flag/{key}",
    // v1.0 — Absence domain (range-based vacation / sick / unpaid leave).
    "/absence-period",
    "/absence-period/{id}",
    "/absence-period/by-sales-person/{sales_person_id}",
    // Booking-log (read-only audit trail for bookings).
    "/booking-log/{year}/{week}",
    // Sales-person catalog.
    "/sales-person",
    "/sales-person/{id}",
    "/sales-person/current",
    // Extra-hours (legacy single-day overtime / sick / vacation flow).
    "/extra-hours",
    "/extra-hours/{id}",
    "/extra-hours/by-sales-person/{id}",
    // Custom extra hours (admin catalog).
    "/custom-extra-hours",
    // Reporting (employee balance / week-report).
    "/report",
    "/report/week/{year}/{calendar_week}",
    // Billing periods (invoice / payroll snapshots).
    "/billing-period",
    "/billing-period/{id}",
    // Shiftplan info / week message (read aggregates).
    "/shiftplan-info/{shiftplan_id}/{year}/{week}",
    "/week-message",
    // Special days (holidays / company events).
    "/special-days",
    // Permission catalog (RBAC management).
    "/permission/user",
    "/permission/role",
    // Cutover admin (legacy → range-based migration).
    "/admin/cutover/profile",
    "/admin/cutover/commit",
    // Cutover convert (Phase 8.1).
    "/admin/cutover/convert-quarantine-entry",
    "/admin/cutover/bulk-convert-quarantine-rows",
];

/// Representative subset of named schemas that must exist in
/// `components.schemas`. We pin names only — bodies evolve as DTOs gain or
/// shed fields, and we don't want to gate every field tweak on a test diff.
///
/// `VacationBalanceTO` is the Wave-2 addition that is the primary reason
/// this test exists in Phase 8.
const EXPECTED_SCHEMAS: &[&str] = &[
    // Phase 8 — VacationBalance aggregate.
    "VacationBalanceTO",
    // Phase 8 Plan 08-07 Gap-Closure — Feature-Flag DTO.
    "FeatureFlagTO",
    // v1.0 — Absence domain.
    "AbsencePeriodTO",
    "AbsenceCategoryTO",
    "AbsencePeriodCreateResultTO",
    "WarningTO",
    // Booking + booking-log.
    "BookingTO",
    "BookingCreateResultTO",
    "BookingLogTO",
    // Sales-person.
    "SalesPersonTO",
    "SalesPersonUnavailableTO",
    // Extra-hours.
    "ExtraHoursTO",
    "ExtraHoursCategoryTO",
    "ExtraHoursCategoryDeprecatedErrorTO",
    // Reporting / billing.
    "EmployeeReportTO",
    "WorkingHoursReportTO",
    "BillingPeriodTO",
    // Shiftplan / slot.
    "SlotTO",
    "ShiftplanTO",
    // Cutover.
    "CutoverProfileTO",
    // Cutover (Plan 08-08) — per-entry inline drift diagnostics.
    "CutoverQuarantineEntryTO",
    // Cutover convert DTOs (Phase 8.1).
    "CutoverConvertQuarantineEntryRequest",
    "CutoverConvertQuarantineEntryResponse",
    "CutoverBulkConvertQuarantineRowsRequest",
    "CutoverBulkConvertQuarantineRowsResponse",
    "CutoverConvertErrorTO",
];

#[test]
fn openapi_paths_contain_expected_routes() {
    let openapi = ApiDoc::openapi();
    let actual: Vec<&str> = openapi.paths.paths.keys().map(String::as_str).collect();

    let mut missing: Vec<&str> = Vec::new();
    for expected in EXPECTED_PATHS {
        if !actual.iter().any(|p| p == expected) {
            missing.push(expected);
        }
    }

    assert!(
        missing.is_empty(),
        "OpenAPI surface drift: expected paths missing from ApiDoc::openapi().paths.\n\
         missing: {missing:#?}\n\
         actual paths ({n} total): {actual:#?}",
        n = actual.len(),
    );
}

#[test]
fn openapi_schemas_contain_expected_dtos() {
    let openapi = ApiDoc::openapi();
    let components = openapi
        .components
        .as_ref()
        .expect("ApiDoc::openapi() must populate components");
    let actual: Vec<&str> = components.schemas.keys().map(String::as_str).collect();

    let mut missing: Vec<&str> = Vec::new();
    for expected in EXPECTED_SCHEMAS {
        if !actual.iter().any(|s| s == expected) {
            missing.push(expected);
        }
    }

    assert!(
        missing.is_empty(),
        "OpenAPI surface drift: expected schemas missing from components.schemas.\n\
         missing: {missing:#?}\n\
         actual schemas ({n} total): {actual:#?}",
        n = actual.len(),
    );
}

/// Sanity guard: Phase 8 Plan 02 added the VacationBalance endpoints. If the
/// VacationBalance tag is missing from the OpenAPI doc, either the
/// `vacation_balance::VacationBalanceApiDoc` was unwired from the parent
/// `ApiDoc`'s `nest(...)` block, or the tag definition was renamed.
#[test]
fn openapi_includes_vacation_balance_tag() {
    let openapi = ApiDoc::openapi();
    let tag_names: Vec<&str> = openapi
        .tags
        .as_ref()
        .map(|ts| ts.iter().map(|t| t.name.as_str()).collect())
        .unwrap_or_default();

    let has_tag = tag_names.iter().any(|n| *n == "VacationBalance");

    assert!(
        has_tag,
        "OpenAPI surface drift: 'VacationBalance' tag missing.\n\
         tag names = {tag_names:#?}",
    );
}
