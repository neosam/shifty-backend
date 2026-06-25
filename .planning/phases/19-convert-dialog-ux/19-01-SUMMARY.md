---
phase: 19-convert-dialog-ux
plan: "01"
subsystem: backend
tags: [absence, extra-hours, rest-types, convert-dialog, uv-01, uv-02]
dependency_graph:
  requires: []
  provides:
    - ExtraHoursMarkerTO.suggested_end
    - ExtraHoursMarkerTO.is_full_week
    - AbsenceService::suggest_convert_ranges_for_markers
  affects:
    - rest/src/absence.rs (both list endpoints)
tech_stack:
  added: []
  patterns:
    - workday/holiday forward-walk with per-week cap (reused from derive_hours_for_range)
    - index-aligned Vec<(Date, bool)> return convention (mirrors derive_days_for_hourly_markers)
key_files:
  created: []
  modified:
    - rest-types/src/lib.rs
    - service/src/absence.rs
    - service_impl/src/absence.rs
    - rest/src/absence.rs
    - service_impl/src/test/absence.rs
decisions:
  - "Backend owns calendar math (D-19-01): reused holiday/workday resolution from derive_hours_for_range, no WASM duplication"
  - "Half-day (derived_days <= 0.5) yields suggested_end == when (D-19-03)"
  - "Full-week detection uses relative f32 epsilon: (hours - expected_hours).abs() < EPSILON * hours.max(1.0)"
  - "60-calendar-day cap on forward walk guards against runaway on bad contract data (T-19-02)"
metrics:
  duration: ~25 min
  completed: "2026-06-26"
---

# Phase 19 Plan 01: Backend Pre-Compute of Convert-Range Suggestions Summary

Backend pre-computation of workday-aware `suggested_end` dates and `is_full_week` flags for hourly absence markers, enabling the convert-dialog UX (UV-01/UV-02) without duplicating calendar math in WASM.

## New REST-Types Fields

`ExtraHoursMarkerTO` in `rest-types/src/lib.rs` gains two fields:

```rust
/// UV-01: workday-based end date. [when, suggested_end] covers exactly derived_days
/// active workdays (weekends, holidays, per-week cap). For full-week markers this
/// is the Sunday of when's ISO week.
#[schema(value_type = String, format = "date")]
pub suggested_end: time::Date,   // no #[serde(default)] — always set by producer

/// UV-02: true iff amount == contract.expected_hours within f32 epsilon.
#[serde(default)]
pub is_full_week: bool,
```

`suggested_end` has no `#[serde(default)]` because it is always set by the producer
(mirrors `when`). `is_full_week` has `#[serde(default)]` (bool → false on missing).

## New Service Method Contract

### Trait signature (service/src/absence.rs)

```rust
async fn suggest_convert_ranges_for_markers(
    &self,
    sales_person_id: Uuid,
    markers: &[(Date, f32)],
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Vec<(Date, bool)>, ServiceError>;
```

Return: index-aligned `Vec<(suggested_end, is_full_week)>` per input marker.

### Implementation edge cases (service_impl/src/absence.rs)

| Input condition | suggested_end | is_full_week |
|----------------|--------------|--------------|
| Empty `markers` | — (empty Vec) | — |
| No active contract at `when` | `when` (fallback) | `false` |
| `hours == contract.expected_hours` (f32 epsilon) | Sunday of `when`'s ISO week | `true` |
| `derived_days <= 0.5` (half-day) | `when` | `false` |
| Normal workday walk (UV-01) | last counted workday after accumulating `ceil(derived_days)` counted days | `false` |

The forward walk:
- Counts only days where `contract.has_day_of_week(day.weekday()) && !holidays.contains(&day)`
- Enforces per-ISO-week cap (`workdays_per_week`), same as `derive_hours_for_range`
- Caps the walk at 60 calendar days from `when` (DoS guard, T-19-02)
- Holidays fetched per ISO week from `SpecialDayService::get_by_week` (deduplicated batch, same pattern as existing code)

## Handler Wiring Sites (rest/src/absence.rs)

Both list endpoints use the same `pairs` Vec already built for `derive_days_for_hourly_markers`:

### LoadAll path (line ~286)
```
// After derive_days_for_hourly_markers enrichment:
let suggest_ranges = svc.suggest_convert_ranges_for_markers(person.id, &pairs, ...).await?;
for (marker, (suggested_end, is_full_week)) in person_markers.iter_mut().zip(suggest_ranges.iter()) {
    marker.suggested_end = *suggested_end;
    marker.is_full_week = *is_full_week;
}
```

### LoadForSalesPerson path (line ~494)
Same pattern, gated by the existing `!hourly_markers.is_empty()` guard.

## Tests Added

`service_impl/src/test/absence.rs` — 6 new unit tests (mockall):

| Test | Scenario |
|------|---------|
| `suggest_workday_skip_crosses_weekend` | 3 workdays from Thursday → end = following Monday |
| `suggest_holiday_skip` | Holiday on Wednesday shifts 3-workday end to Thursday |
| `suggest_exact_week_full_week_flag` | 40h == expected_hours → is_full_week true, end = Sunday of week |
| `suggest_non_exact_no_full_week_flag` | 39.5h ≠ 40h → is_full_week false, UV-01 end logic |
| `suggest_half_day_returns_when` | 4h / 8h = 0.5 days → suggested_end == when |
| `suggest_no_contract_returns_when_false` | Empty work_details → (when, false) |

## OpenAPI / SQLx

No new SQLx queries were added (reuses existing `EmployeeWorkDetailsService::find_by_sales_person_id` and `SpecialDayService::get_by_week`). `sqlx prepare` regeneration not required.

OpenAPI: `ExtraHoursMarkerTO` schema now includes `suggested_end` and `is_full_week` via `ToSchema` derive — no manual annotation needed beyond `#[schema(value_type = String, format = "date")]` on `suggested_end`.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. Both fields are computed and wired on every list response.

## Threat Flags

None. The new surface is server-computed read-only hint data, gated by the existing HR∨self permission (T-19-01 mitigated). The 60-day walk cap addresses T-19-02.

## Self-Check: PASSED

- `rest-types/src/lib.rs` field `suggested_end` present at line 1756
- `service/src/absence.rs` trait method `suggest_convert_ranges_for_markers` present at line 295
- `service_impl/src/absence.rs` impl method present at line 652
- `rest/src/absence.rs` call sites at lines 286, 494 (count = 2)
- `cargo test --workspace` green: 461 service_impl unit tests + 61 integration tests, all passed; 0 failed
