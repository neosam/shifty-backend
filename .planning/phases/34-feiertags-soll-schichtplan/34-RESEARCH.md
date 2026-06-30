# Phase 34: Feiertags-Soll im Schichtplan — Research

**Researched:** 2026-06-30
**Domain:** Rust backend — ReportingService / get_week injection point
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-34-01:** Feiertag reduziert ausschließlich `expected_hours` in `get_week`
  (`reporting.rs:884`) als 4. Injektionspunkt. Per-Tag-Aggregat-Zeile in
  `booking_information.rs:517` wird NICHT angefasst.
- **D-34-02:** Backend-only. HSP-02 gilt als erfüllt sobald
  `WorkingHoursPerSalesPerson.holiday_hours` korrekt gefüllt ist — keine neue
  FE-Spalte, kein i18n.
- **D-34-03:** `test_holiday_auto_credit_no_year_view_impact`
  (`service_impl/src/test/reporting_holiday_auto_credit.rs:545`) wird in place
  umgebaut (bisherige Panic-Guards → positive Assertions). Neuer separater
  HSP-04-Test für Stichtag-Gating und manual-wins.
- **D-34-04:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12 — kein Bump.

### Claude's Discretion

- Exakte Stelle/Form des 4. Injektionspunkts in `get_week` (eigener
  `holiday_derived`-Term vs. Einbau in den bestehenden absence/expected-
  Reduktionspfad) — solange: (a) `expected_hours` korrekt sinkt, (b)
  `holiday_hours` den derived-Beitrag enthält, (c) `dynamic_hours`/Bänder
  unberührt bleiben, (d) dynamic-Week-Guard (`planned_hours <= 0.0 → 0`)
  konsistent angewandt wird.
- Genaue Fixture-/Test-Struktur im Rahmen von D-34-03.

### Deferred Ideas (OUT OF SCOPE)

- Sichtbarer Feiertags-Indikator/Spalte/Tooltip im Schichtplan-Frontend.
- Feiertags-Berücksichtigung der per-Tag-Aggregat-Zeile (Mo–So) unter dem
  Schichtplan.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HSP-01 | `get_week` erhält derived-Holiday-Beitrag → `expected_hours` sinkt pro Mitarbeiter konsistent zum Stundenkonto | 4th injection point pattern confirmed; exact formula and guard documented below |
| HSP-02 | `WorkingHoursPerSalesPerson.holiday_hours` korrekt gefüllt (derived + manual additiv) | Propagation path confirmed: get_week → ShortEmployeeReport.holiday_hours → booking_information:331 |
| HSP-03 | Kapazitätsbänder (`paid_hours`/`dynamic_hours`/`committed_voluntary`/`volunteer`) unverändert | Band-guard design: separate term only for expected_hours, NOT dynamic_hours; test assertion documented |
| HSP-04 | Stichtag-Gate + manual-wins (keine Doppelzählung) in get_week identisch zum Stundenkonto via Wiederverwendung von `build_derived_holiday_map` | build_derived_holiday_map already implements both gates; HOL-03 test rebuild + new HSP-04 test documented |
</phase_requirements>

---

## Summary

Phase 34 wires the Phase-25 holiday automation (`build_derived_holiday_map`) into
the `get_week` report path as a **fourth injection point**. Currently `get_week`
only credits `holiday_hours` from manual `ExtraHours(Holiday)` entries; derived
holidays are absent, causing the Schichtplan-Wochentabelle to show unreduced
`expected_hours`/`available_hours` even when a SpecialDay(Holiday) is active.

The fix is confined to `service_impl/src/reporting.rs` in the `get_week` method
body (lines 884–1112). No new service dependencies, no new DAO calls, no schema
migrations, no frontend changes. All prerequisite infrastructure — the
`build_derived_holiday_map` helper, the toggle/SpecialDay service wiring, and the
propagation via `booking_information` — already exists from Phase 25.

The critical subtlety is that derived holiday must reduce `expected_hours` **only**
and must NOT reduce `dynamic_hours`. This asymmetry is required by HSP-03: the
capacity bands (`paid_hours = Σ report.dynamic_hours`) must not be affected by
holiday credits. This distinguishes the 4th injection from the existing
`absence_derived_balance_total` pattern, which reduces both. A dedicated
`holiday_derived_gated` term (separate from `abense_hours_for_balance` and
`absence_derived_balance_total`) is the implementation vehicle.

The HOL-03 regression test must be rebuilt: the old form guards against any
SpecialDay/toggle call in `get_week` (panic-on-call semantics); the new form
must provide these mocks and assert the correct outcome.

**Primary recommendation:** Add a `holiday_derived_gated` term in `get_week` by
calling `build_derived_holiday_map` per-employee in the existing loop,
applying the `planned_hours <= 0.0 → 0` guard, and subtracting only from
`expected_hours` (not `dynamic_hours`). Rebuild HOL-03 in place; add HSP-04
test in the same file.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Derived holiday computation | Service / Business-Logic (`ReportingServiceImpl`) | — | `build_derived_holiday_map` is a method on `ReportingServiceImpl`; toggle and SpecialDay are Basic-tier deps already wired |
| expected_hours reduction | `get_week` (same service method) | — | Only write-point; propagation to booking_information is automatic read-through |
| `available_hours` / `holiday_hours` propagation | `booking_information.rs:317–334` | — | Read-only: `available_hours = report.expected_hours`, `holiday_hours = report.holiday_hours`; no changes needed |
| Band invariance guard | `get_week` formula logic | — | Separate `holiday_derived_gated` term not applied to `dynamic_hours` |
| Snapshot isolation | `billing_period_report.rs` | — | Reads `get_report_for_employee_range`, not `get_week`; no impact |

---

## Standard Stack

No new dependencies. Phase 34 reuses the existing Rust workspace.

| Crate | Version | Role |
|-------|---------|------|
| `service_impl` | workspace | Contains `ReportingServiceImpl` and `get_week` |
| `service` (traits) | workspace | `ShortEmployeeReport`, `SpecialDayService`, `ToggleService` |
| `tokio` | 1.44 | Async runtime for tests (`#[tokio::test]`) |
| `mockall` | workspace | Mock generation for unit tests |

**Installation:** none — all crates already in workspace.

---

## Architecture Patterns

### System Architecture Diagram

```
SpecialDayService ──────────┐
ToggleService ──────────────┤
                            ▼
                   build_derived_holiday_map
                   (reporting.rs:151)
                            │
                            │ HashMap<Date, f32>
                            ▼
                        get_week                    ← WRITE POINT
                   (reporting.rs:884)
                            │
                  ShortEmployeeReport
                  ├─ expected_hours = planned_hours
                  │     - abense_hours_for_balance
                  │     - absence_derived_balance_total
                  │     - holiday_derived_gated  ← NEW TERM
                  ├─ holiday_hours = manual_holiday + derived_holiday_gated ← UPDATED
                  └─ dynamic_hours = planned_hours
                        - abense_hours_for_balance
                        - absence_derived_balance_total  ← UNCHANGED (HSP-03)
                            │
                            ▼
               booking_information.rs:317–334   (READ-ONLY)
               ├─ available_hours = report.expected_hours
               ├─ holiday_hours   = report.holiday_hours
               └─ paid_hours     += report.dynamic_hours  ← BAND, UNCHANGED
```

### Recommended Project Structure

No structural changes — all edits are in existing files:

```
service_impl/src/
├── reporting.rs               # get_week: add 4th injection point
└── test/
    └── reporting_holiday_auto_credit.rs   # HOL-03 rebuild + HSP-04 new test
```

---

## Code Anatomy — Exact Read Points

### 1. `build_derived_holiday_map` (reporting.rs:151–242) [VERIFIED: codebase grep]

**Signature:**
```rust
async fn build_derived_holiday_map(
    &self,
    from_date: ShiftyDate,
    to_date: ShiftyDate,
    working_hours: &[EmployeeWorkDetails],
    extra_hours: &[ExtraHours],
    context: Authentication<Deps::Context>,
) -> Result<HashMap<time::Date, f32>, ServiceError>
```

**Returns:** `HashMap<time::Date, f32>` — keyed by concrete holiday date, value = credited hours.

**Empty when:**
- Toggle `holiday_auto_credit` has no `value` (automation off, D-25-05).
- Returns `Unauthorized` → silently returns `Ok(HashMap::new())` (safe for test contexts).

**Stichtag gate (HCFG-01):** `holiday_date < cutoff → continue` (inclusive: `>=` qualifies).

**Manual-wins (D-25-03):** Skips any date where `extra_hours` contains `ExtraHoursCategory::Holiday` with `eh.date_time.date() == holiday_date`.

**Per-employee filter:** Credits only if `EmployeeWorkDetails::has_day_of_week(holiday.day_of_week) == true` for the contract valid that week.

**Amount:** `EmployeeWorkDetails::holiday_hours()` = `expected_hours / potential_days_per_week()` (D-25-02).

**ISO-week safety:** Uses `time::Date::from_iso_week_date` — never manual week math.

### 2. Existing injection points (template for 4th) [VERIFIED: codebase grep]

| Point | Location | What it does |
|-------|----------|-------------|
| 1a | `hours_per_week` (called by `get_report_for_employee_range`) — pre-built map passed as param | Sums `derived_holiday` per week → `holiday_hours = manual + derived`, adds to `absense_hours` |
| 1b | `get_reports_for_all_employees` — calls `build_derived_holiday_map` at line 361 per-employee | Same pattern: pre-built map iterated in per-week fold |
| 1c | `EmployeeReport.holiday_hours = by_week.iter().map(|w| w.holiday_hours).sum()` (line 872) | Aggregates by_week into report-level field (billing_period snapshot reads this) |
| **4th (new)** | `get_week` body, per-employee loop | Builds map per-employee for the single week; new `holiday_derived_gated` term |

### 3. `get_week` current logic for `expected_hours` (reporting.rs:1055–1083) [VERIFIED: codebase grep]

```rust
// Current formula (lines 1066–1083, simplified):
let abense_hours_for_balance =
    if !has_contract_row || planned_hours <= 0.0 { 0.0 } else { abense_hours };
let absence_derived_balance_total =
    if !has_contract_row || planned_hours <= 0.0 { 0.0 } else {
        absence_derived_vacation_hours
            + absence_derived_sick_leave_hours
            + absence_derived_unpaid_leave_hours
    };
let expected_hours = planned_hours
    - abense_hours_for_balance
    - absence_derived_balance_total;       // ← NEW TERM follows here
let dynamic_hours = dynamic_hours
    - abense_hours_for_balance
    - absence_derived_balance_total;       // ← dynamic_hours NOT changed by holiday
```

### 4. `holiday_hours` current assembly in `get_week` (reporting.rs:955–962) [VERIFIED: codebase grep]

```rust
let holiday_hours = employee_extra_hours
    .map(|eh| {
        eh.iter()
            .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
            .map(|eh| eh.amount)
            .sum::<f32>()
    })
    .unwrap_or(0.0);
// ← Currently ONLY manual ExtraHours(Holiday). Derived not included.
```

### 5. Propagation (booking_information.rs:317–334) — READ-ONLY [VERIFIED: codebase grep]

```rust
for report in week_report.iter() {
    paid_hours += report.dynamic_hours;           // Band — must stay 40h
    if is_shiftplanner {
        working_hours_per_sales_person.push(WorkingHoursPerSalesPerson {
            available_hours: report.expected_hours,   // ← reduced by holiday after fix
            holiday_hours:   report.holiday_hours,    // ← filled by fix
            // ...
        });
    }
}
```

**No changes needed in `booking_information.rs`** — propagation is automatic.

---

## Implementation Pattern — 4th Injection Point [ASSUMED: exact form is Claude's Discretion]

The 4th injection point adds a `holiday_derived_gated` term that only reduces
`expected_hours`, not `dynamic_hours`. This is the key design difference from
`absence_derived_balance_total` (which reduces both).

```rust
// Within get_week's per-employee loop, after computing absence_derived:

// ── 4th injection point (HSP-01/02, D-34-01) ──────────────────────────────
// Build derived-holiday map for this employee + this single week.
// Reuses build_derived_holiday_map (same logic as injection points 1a/1b).
let employee_extra_hours_slice: &[ExtraHours] = employee_extra_hours
    .map(|v| v.as_ref())
    .unwrap_or(&[]);
let derived_holiday_map = self
    .build_derived_holiday_map(
        ShiftyWeek::new(year, week).as_date(DayOfWeek::Monday),
        ShiftyWeek::new(year, week).as_date(DayOfWeek::Sunday),
        &working_hours,
        employee_extra_hours_slice,
        context.clone(),
    )
    .await?;
let derived_holiday_for_week: f32 = derived_holiday_map.values().sum();

// Dynamic-week guard (same pattern as absence_derived_balance_total):
// If planned_hours <= 0.0, the week is dynamic → holiday credit must not
// reduce expected further (no negative expected, no inflated balance).
let holiday_derived_gated =
    if !has_contract_row || planned_hours <= 0.0 { 0.0 } else { derived_holiday_for_week };

// Update holiday_hours to include derived contribution (additive to manual).
let holiday_hours = holiday_hours + holiday_derived_gated;

// ── Modified expected_hours formula ───────────────────────────────────────
let expected_hours = planned_hours
    - abense_hours_for_balance
    - absence_derived_balance_total
    - holiday_derived_gated;             // ← only here

// dynamic_hours formula UNCHANGED (HSP-03 band guard):
let dynamic_hours = dynamic_hours
    - abense_hours_for_balance
    - absence_derived_balance_total;     // ← holiday_derived_gated NOT subtracted
```

**Why a separate term (not folded into `abense_hours_for_balance`):**
`abense_hours_for_balance` reduces both `expected_hours` and `dynamic_hours`
(line 1081). Adding derived holiday there would reduce `paid_hours` in
`booking_information` — violating HSP-03.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Derived holiday computation | New holiday-calculation loop | `build_derived_holiday_map` (reporting.rs:151) | Already implements Stichtag gate, manual-wins, ISO-week arithmetic, empty-on-no-toggle |
| Stichtag parsing | Manual ISO date parsing | Built into `build_derived_holiday_map` — `time::Date::parse(..., Iso8601::DEFAULT)` | Year-boundary safety already handled |
| SpecialDay lookup per week | Direct DAO call | Already in `build_derived_holiday_map` via `self.special_day_service.get_by_week` | Avoids duplicating fetch + filter logic |
| Manual-wins conflict check | Extra `extra_hours` scan | `build_derived_holiday_map` already skips auto-credit if manual `ExtraHours(Holiday)` exists for same date | D-25-03 compliance guaranteed |

**Key insight:** The entire holiday computation is encapsulated in `build_derived_holiday_map`. The 4th injection is purely a call-site addition — no new logic.

---

## Common Pitfalls

### Pitfall 1: `holiday_derived_gated` also applied to `dynamic_hours`
**What goes wrong:** Band (`paid_hours` in booking_information) drops when a holiday is auto-credited.
**Why it happens:** `dynamic_hours` in get_week is reduced by the same terms as `expected_hours` (lines 1081). Folding derived holiday into `abense_hours_for_balance` reduces both.
**How to avoid:** Separate `holiday_derived_gated` term, subtracted ONLY from `expected_hours`.
**Warning signs:** HOL-03 rebuilt test assertion `dynamic_hours == 40.0` fails.

### Pitfall 2: Missing dynamic-week guard on `holiday_derived_gated`
**What goes wrong:** For a dynamic contract (`planned_hours = 0.0`), subtracting holiday hours makes `expected_hours` go negative, inflating the balance.
**Why it happens:** No guard applied to derived holiday hours before subtraction.
**How to avoid:** `if !has_contract_row || planned_hours <= 0.0 { 0.0 } else { derived_holiday_for_week }` — identical guard pattern to `absence_derived_balance_total`.
**Warning signs:** Dynamic-contract employee gets negative `expected_hours` in `get_week`.

### Pitfall 3: Double-count — derived holiday + `abense_hours_for_balance`
**What goes wrong:** A manual `ExtraHours(Holiday)` is already in `abense_hours` (Unavailable filter, line 931–938). If derived holiday is ALSO added without the manual-wins check, holiday credited twice.
**Why it happens:** Forgetting that `build_derived_holiday_map` already skips auto-credit when a manual entry exists (D-25-03).
**How to avoid:** Reuse `build_derived_holiday_map` — manual-wins is built in. Do NOT bypass the function.
**Warning signs:** HCFG-03 test shows `holiday_hours == 16.0` instead of 8.0.

### Pitfall 4: Stale HOL-03 test panics after fix
**What goes wrong:** The rebuilt HOL-03 test must provide `special_day_service` and `toggle_service` mocks. If not, mockall panics on unexpected calls.
**Why it happens:** Old test had no expectations (panic-guard semantics). New code calls these services.
**How to avoid:** In the rebuilt test, set `expect_get_by_week()` on `special_day_service` and `expect_get_toggle_value()` on `toggle_service` (active cutoff). See D-34-03.
**Warning signs:** Test panics with "unexpected call to MockSpecialDayService::get_by_week".

### Pitfall 5: Wrong slice passed to `build_derived_holiday_map` for `extra_hours`
**What goes wrong:** Passing an empty slice always triggers auto-credit even when a manual entry exists for the same day (manual-wins bypassed).
**Why it happens:** `employee_extra_hours` in `get_week` is `Option<&Arc<[ExtraHours]>>`. Using `unwrap_or` incorrectly or forgetting to pass the employee's extra hours.
**How to avoid:** `employee_extra_hours.map(|v| v.as_ref()).unwrap_or(&[])` — always pass actual per-employee slice.
**Warning signs:** HCFG-03 / manual-wins test fails (holiday_hours == 16.0).

### Pitfall 6: `abense_hours` not updated for the display field
**What goes wrong:** The `ShortEmployeeReport.balance_hours` is correct (via `expected_hours`), but the intermediate `abense_hours` display could drift.
**Why it happens:** `abense_hours` in `get_week` (line 931) sums `Availability::Unavailable` extra_hours (manual only). Derived holiday is NOT Unavailable extra_hours.
**How to avoid:** For the display fields (vacation_hours, sick_leave_hours, etc.) there is no `abense_hours` display field in `ShortEmployeeReport` — it is only used internally to compute `expected_hours`. The balance is correct via the `holiday_derived_gated` subtraction from `expected_hours`. No separate `abense_hours` update needed.
**Warning signs:** (Non-issue, but confirm: `ShortEmployeeReport` has no raw `abense_hours` field.)

---

## Snapshot Verification (D-34-04) [VERIFIED: codebase grep]

**Grep evidence confirming no Bump needed:**

```
# billing_period_report.rs — no references to get_week or booking_information:
grep "get_week\|booking_information" service_impl/src/billing_period_report.rs
→ Only match: comment at line 368 (refers to snapshot consistency, not a call site)

# billing_period_report.rs calls to ReportingService:
grep "get_report_for_employee_range" service_impl/src/billing_period_report.rs
→ Lines 146, 157, 168, 179 — ALL calls are get_report_for_employee_range

# Confirmed: build_derived_holiday_map NOT called from billing_period_report.rs:
grep "build_derived_holiday_map\|derived_holiday" service_impl/src/billing_period_report.rs
→ No matches
```

**Conclusion:** `billing_period_report.rs` reads `EmployeeReport.holiday_hours` exclusively
via `get_report_for_employee_range` → `hours_per_week` → injection points 1a/1c.
Phase 34 only modifies `get_week`. No persisted `BillingPeriodValueType` changes
computation. **`CURRENT_SNAPSHOT_SCHEMA_VERSION` stays 12.**

Current value confirmed at `service_impl/src/billing_period_report.rs:117`:
`pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;`

---

## HOL-03 Test Rebuild Plan (D-34-03) [VERIFIED: codebase grep]

### Current HOL-03 (to be replaced in place, line 543–620)

Current semantics: `special_day_service` and `toggle_service` have NO expectations set → any call panics (regression guard that `get_week` has no derive-on-read).
Assertions: `dynamic_hours == 40.0`, `holiday_hours == 0.0`.

After Phase 34, `get_week` DOES call these services → the old test panics on success.

### New HOL-03 skeleton

```rust
#[tokio::test]
async fn test_holiday_auto_credit_no_year_view_impact() {
    let mut mocks = ReportingMocks::new();

    // get_week mock setup (same as before):
    mocks
        .employee_work_details_service
        .expect_all_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![fixture_work_details_8h_mon_fri()])));
    mocks
        .shiftplan_report_service
        .expect_extract_shiftplan_report_for_week()
        .returning(|_, _, _, _| Ok(Arc::from(vec![])));
    mocks
        .extra_hours_service
        .expect_find_by_week()
        .returning(|_, _, _, _| Ok(Arc::from(Vec::<ExtraHours>::new())));
    mocks
        .sales_person_service
        .expect_get()
        .returning(|_, _, _| Ok(fixture_sales_person()));
    mocks
        .transaction_dao
        .expect_use_transaction()
        .returning(|_| Ok(dao::MockTransaction));
    mocks
        .absence_service
        .expect_derive_hours_for_range()
        .returning(|_, _, _, _, _| Ok(BTreeMap::new()));

    // NEW: provide SpecialDay mock — 1 holiday on KW23 Monday (2024-06-03)
    mocks.special_day_service = MockSpecialDayService::new();
    mocks
        .special_day_service
        .expect_get_by_week()
        .returning(|_, wk, _| {
            if wk == 23 {
                Ok(Arc::from(vec![make_holiday(2024, 23, DayOfWeek::Monday)]))
            } else {
                Ok(Arc::from(vec![]))
            }
        });

    // NEW: provide toggle mock — active cutoff before holiday date
    mocks.toggle_service = MockToggleService::new();
    mocks
        .toggle_service
        .expect_get_toggle_value()
        .returning(|_, _, _| Ok(Some(Arc::from("2024-01-01"))));

    let service = mocks.build();
    let reports = service
        .get_week(2024, 23, Authentication::Full, None)
        .await
        .expect("get_week must succeed");

    assert_eq!(reports.len(), 1, "HOL-03: 1 report expected");
    let report = &reports[0];

    // HSP-03 band guard: dynamic_hours MUST NOT change (feeds paid_hours in booking_information)
    assert!(
        (report.dynamic_hours - 40.0).abs() < 0.01,
        "HSP-03: dynamic_hours must remain 40h (band invariant), got {}",
        report.dynamic_hours
    );

    // HSP-01: expected_hours reduced by 8h holiday
    assert!(
        (report.expected_hours - 32.0).abs() < 0.01,
        "HSP-01: expected_hours must be 32h (40 - 8 holiday), got {}",
        report.expected_hours
    );

    // HSP-02: holiday_hours filled with derived contribution
    assert!(
        (report.holiday_hours - 8.0).abs() < 0.01,
        "HSP-02: holiday_hours must be 8h (derived auto-credit), got {}",
        report.holiday_hours
    );
}
```

### New HSP-04 test (separate, same file)

Two sub-cases:
1. **Stichtag-Gate:** Holiday date before cutoff → `expected_hours == 40.0`, `holiday_hours == 0.0`.
   - Reuse same mock setup as rebuilt HOL-03 but pass cutoff = `"2024-12-31"` (after holiday 2024-06-03).
2. **Manual-wins:** Same SpecialDay + manual `ExtraHours(Holiday, 8.0)` on same date → `holiday_hours == 8.0` (once).
   - Set `find_by_week` to return `[make_holiday_extra_hours(8.0, date!(2024-06-03))]`.
   - Assert `holiday_hours == 8.0` (not 16.0).

---

## Runtime State Inventory

> Not applicable — greenfield injection into existing function; no rename/refactor/migration.
> No stored data changes. No OS-registered state. No secrets. No build artifacts.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | All compilation | ✓ | workspace (edition 2021) | — |
| tokio (test) | `#[tokio::test]` | ✓ | 1.44 | — |
| mockall | Mock generation | ✓ | workspace | — |
| SQLx + SQLite | Integration (not used by unit tests) | ✓ | workspace | — |

**Missing dependencies with no fallback:** none.

---

## Project Constraints (from CLAUDE.md)

1. **Clippy hard gate:** `cargo clippy --workspace -- -D warnings` MUST pass before commit. `cargo test` alone does not run clippy. Any new variables/bindings must be used or prefixed `_`.
2. **Snapshot Versioning rule:** Bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` when a persisted `BillingPeriodValueType` computation changes. Phase 34 does NOT change `get_report_for_employee_range` or `hours_per_week`, so no bump (D-34-04).
3. **Service-Tier convention:** `ReportingServiceImpl` is Business-Logic tier. `SpecialDayService` and `ToggleService` are Basic-tier (already wired). No cycle risk.
4. **Transaction pattern:** `build_derived_holiday_map` is `&self` async; it manages its own service calls. `get_week` passes `context.clone()`. No `tx` param needed (SpecialDay and Toggle services are transaction-free).
5. **`jj` VCS:** Never `git commit`. Use `jj-commit` skill.
6. **i18n:** No new user-visible text (D-34-02 = BE-only).
7. **Tests:** `cargo test --workspace` must pass after all changes.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in + tokio-test (async) |
| Config file | `Cargo.toml` in workspace root |
| Quick run | `cargo test -p service_impl holiday` |
| Full suite | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HSP-01 | `expected_hours` reduced by 8h for 40h contract + 1 holiday in `get_week` | unit | `cargo test -p service_impl test_holiday_auto_credit_no_year_view_impact` | ✅ (HOL-03 rebuild) |
| HSP-02 | `holiday_hours == 8.0` in `get_week` output | unit | same as HSP-01 | ✅ (HOL-03 rebuild) |
| HSP-03 | `dynamic_hours == 40.0` unchanged (band guard) | unit | same as HSP-01 | ✅ (HOL-03 rebuild) |
| HSP-04a | Holiday BEFORE cutoff → no effect (`expected_hours == 40`, `holiday_hours == 0`) | unit | `cargo test -p service_impl test_hsp04_before_cutoff` | ❌ Wave 0 |
| HSP-04b | Manual ExtraHours(Holiday) + SpecialDay → `holiday_hours == 8.0` (once, not twice) | unit | `cargo test -p service_impl test_hsp04_manual_wins` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p service_impl holiday && cargo clippy --workspace -- -D warnings`
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `service_impl/src/test/reporting_holiday_auto_credit.rs` — HOL-03 rebuild (in place, lines 543–620)
- [ ] HSP-04 test functions `test_hsp04_before_cutoff` + `test_hsp04_manual_wins` (new, same file)

---

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | Auth gate already in `get_week` via `employee_work_details_service.all_for_week` |
| V5 Input Validation | no | No new inputs; derived holiday uses pre-validated SpecialDay entities |
| V6 Cryptography | no | — |

No new threat patterns introduced. `build_derived_holiday_map` calls `toggle_service` with
the caller's `context`, consistent with existing auth propagation.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Derived holiday must use a separate `holiday_derived_gated` term (NOT folded into `abense_hours_for_balance`) to preserve `dynamic_hours` | Implementation Pattern | dynamic_hours would drop → HSP-03 violated → paid_hours band in booking_information reduced |
| A2 | `employee_extra_hours.map(|v| v.as_ref()).unwrap_or(&[])` correctly provides per-employee ExtraHours slice to `build_derived_holiday_map` | Code Examples | If wrong: manual-wins conflict check sees empty slice → double-count |

---

## Open Questions (RESOLVED)

1. **`abense_hours` display field vs. balance field**
   - What we know: `abense_hours` in `get_week` is used only for `abense_hours_for_balance`, not surfaced in `ShortEmployeeReport`
   - What's unclear: Whether any downstream consumer expects `abense_hours` (internal to `get_week`) to include derived holiday for some other purpose
   - Recommendation: Confirm `ShortEmployeeReport` has no raw `abense_hours` field (it does not — verified). No update needed.

2. **`balance_hours` in `get_week` after fix**
   - What we know: `balance_hours = overall_hours - expected_hours`. After fix, `expected_hours` decreases by `holiday_derived_gated`, so `balance_hours` increases.
   - What's unclear: Whether this balance change (holiday reduces expected → improves balance) is the intended business behavior.
   - Recommendation: Yes — this is the correct behavior (holiday = like working extra, or equivalently like a manual ExtraHours(Holiday) which has the same effect). D-34-01 confirms this is intentional.

---

## Sources

### Primary (HIGH confidence — verified in codebase)

- `service_impl/src/reporting.rs:151–242` — `build_derived_holiday_map` complete implementation
- `service_impl/src/reporting.rs:884–1112` — `get_week` complete implementation
- `service_impl/src/reporting.rs:532–543` — injection point 1b (get_reports_for_all_employees)
- `service_impl/src/reporting.rs:754–761` — injection point 1b call
- `service_impl/src/reporting.rs:866–872` — injection point 1c
- `service_impl/src/test/reporting_holiday_auto_credit.rs:543–620` — HOL-03 test (to be rebuilt)
- `service_impl/src/billing_period_report.rs:117` — `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12`
- `service_impl/src/billing_period_report.rs:146–179` — reads `get_report_for_employee_range` (NOT `get_week`)
- `service_impl/src/booking_information.rs:317–334` — propagation path from `get_week` output

### Secondary (MEDIUM confidence)

- `34-CONTEXT.md` — D-34-01/02/03/04 decisions (user-confirmed)
- `25-CONTEXT.md` — D-25-03/05/08 predecessor decisions
- `REQUIREMENTS.md` — HSP-01..04 requirements

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies
- Architecture: HIGH — full code read of all relevant functions
- Pitfalls: HIGH — derived from actual code analysis + test structure
- Implementation pattern: MEDIUM (A1/A2 in Assumptions Log) — exact form is Claude's Discretion

**Research date:** 2026-06-30
**Valid until:** 2026-07-30 (code-stable; no external dependencies)
