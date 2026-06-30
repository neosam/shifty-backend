# Phase 34: Feiertags-Soll im Schichtplan — Pattern Map

**Mapped:** 2026-06-30
**Files analyzed:** 2 (1 modified source + 1 modified test)
**Analogs found:** 2 / 2

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `service_impl/src/reporting.rs` (get_week, lines 884–1112) | service / business-logic | request-response (per-employee fold) | Same file — injection points 1a (line 361) and 1b (line 754) | exact |
| `service_impl/src/test/reporting_holiday_auto_credit.rs` (lines 543–620 rebuild + new tests) | test | unit / mock-based | HOL-01 (line 226) and HOL-02 (line 271) in same file | exact |

---

## Pattern Assignments

---

### `service_impl/src/reporting.rs` — 4th injection point in `get_week`

**Role:** Business-Logic service method (async, per-employee loop)
**Analog:** Injection point 1b — `build_derived_holiday_map` call at lines 753–761 (inside `get_reports_for_all_employees`)

#### Analog call shape (lines 753–761)

```rust
// Phase 25: Precompute per-employee derived-holiday map for the range.
// Returns empty map when toggle has no value (automation off, D-25-05).
let derived_holiday = self
    .build_derived_holiday_map(
        from_date,
        to_date,
        &working_hours,
        &extra_hours,
        context.clone(),
    )
    .await?;
```

For the 4th injection point, `from_date`/`to_date` become the Monday/Sunday of the single target week:

```rust
ShiftyWeek::new(year, week).as_date(DayOfWeek::Monday),
ShiftyWeek::new(year, week).as_date(DayOfWeek::Sunday),
```

#### `build_derived_holiday_map` signature (lines 151–158)

```rust
async fn build_derived_holiday_map(
    &self,
    from_date: ShiftyDate,
    to_date: ShiftyDate,
    working_hours: &[EmployeeWorkDetails],
    extra_hours: &[ExtraHours],
    context: Authentication<Deps::Context>,
) -> Result<std::collections::HashMap<time::Date, f32>, ServiceError>
```

**Returns** `HashMap<time::Date, f32>` (keyed by holiday date, value = credited hours).
**Empty when** toggle has no value (automation off) or returns `Unauthorized`.
**Manual-wins built-in:** skips any date where `extra_hours` contains `ExtraHoursCategory::Holiday` for the same date (D-25-03).
**Stichtag gate built-in:** `holiday_date < cutoff → continue`.

#### `extra_hours` slice to pass (Pitfall 5 guard)

Inside `get_week`'s per-employee loop, `employee_extra_hours` is `Option<&Arc<[ExtraHours]>>`.
Correct conversion to `&[ExtraHours]` slice:

```rust
let employee_extra_hours_slice: &[ExtraHours] = employee_extra_hours
    .map(|v| v.as_ref())
    .unwrap_or(&[]);
```

Pass `employee_extra_hours_slice` as the `extra_hours` argument to `build_derived_holiday_map`.
**Never pass `&[]`** — that would bypass manual-wins (D-25-03).

#### Dynamic-week guard pattern (analog: lines 1066–1071)

The existing `absence_derived_balance_total` guard is the exact template to copy:

```rust
// Existing (lines 1066–1071):
let abense_hours_for_balance = if !has_contract_row || planned_hours <= 0.0 { 0.0f32 } else { abense_hours };
let absence_derived_balance_total = if !has_contract_row || planned_hours <= 0.0 {
    0.0f32
} else {
    absence_derived_vacation_hours + absence_derived_sick_leave_hours + absence_derived_unpaid_leave_hours
};
```

Apply the identical guard to the new derived-holiday term:

```rust
// NEW (4th injection — same guard pattern):
let holiday_derived_gated =
    if !has_contract_row || planned_hours <= 0.0 { 0.0f32 } else { derived_holiday_for_week };
```

#### `holiday_hours` assembly — current (lines 955–962)

```rust
let holiday_hours = employee_extra_hours
    .map(|eh| {
        eh.iter()
            .filter(|eh| eh.category == ExtraHoursCategory::Holiday)
            .map(|eh| eh.amount)
            .sum::<f32>()
    })
    .unwrap_or(0.0);
// Currently ONLY manual ExtraHours(Holiday). Derived not included. ← to be extended
```

After the injection point, shadow `holiday_hours` to add the derived contribution:

```rust
let holiday_hours = holiday_hours + holiday_derived_gated;
```

#### `expected_hours` formula — current (line 1072) and new

```rust
// Current (line 1072):
let expected_hours = planned_hours - abense_hours_for_balance - absence_derived_balance_total;

// After Phase 34 (new term appended — ONLY here, NOT in dynamic_hours):
let expected_hours = planned_hours
    - abense_hours_for_balance
    - absence_derived_balance_total
    - holiday_derived_gated;
```

#### `dynamic_hours` formula — UNCHANGED (line 1081, HSP-03 band guard)

```rust
// line 1081 — MUST NOT be changed:
let dynamic_hours = dynamic_hours - abense_hours_for_balance - absence_derived_balance_total;
// holiday_derived_gated is deliberately NOT subtracted here.
```

**Why a separate term (critical):** `abense_hours_for_balance` reduces BOTH `expected_hours` and
`dynamic_hours` (line 1072 and 1081). Folding derived holiday into `abense_hours_for_balance`
would reduce `dynamic_hours`, which feeds `paid_hours` in `booking_information` — violating
HSP-03. The separate `holiday_derived_gated` term, subtracted only from `expected_hours`,
preserves the band invariant.

#### `ShortEmployeeReport` push (lines 1095–1108) — `holiday_hours` field

```rust
result.push(ShortEmployeeReport {
    sales_person: Arc::new(sales_person),
    balance_hours,
    dynamic_hours,
    expected_hours,
    overall_hours,
    vacation_hours: vacation_hours + absence_derived_vacation_hours,
    sick_leave_hours: sick_leave_hours + absence_derived_sick_leave_hours,
    holiday_hours,          // ← now includes derived; shadowed above
    unavailable_hours,
    unpaid_leave_hours: unpaid_leave_hours + absence_derived_unpaid_leave_hours,
    volunteer_hours,
    custom_absence_hours,
});
```

No structural change needed here — once `holiday_hours` is shadowed earlier in the loop,
it flows through automatically.

---

### `service_impl/src/test/reporting_holiday_auto_credit.rs` — HOL-03 rebuild + HSP-04 tests

**Role:** Unit test (async, mockall-based)
**Analog:** HOL-01 (`test_holiday_auto_credit_basic`, line 226) and HOL-02 (`test_holiday_auto_credit_equivalence`, line 271)

#### `ReportingMocks` struct and `build()` (lines 43–116)

```rust
struct ReportingMocks {
    extra_hours_service: MockExtraHoursService,
    shiftplan_report_service: MockShiftplanReportService,
    employee_work_details_service: MockEmployeeWorkDetailsService,
    sales_person_service: MockSalesPersonService,
    // ...
    absence_service: MockAbsenceService,
    transaction_dao: dao::MockTransactionDao,
    special_day_service: MockSpecialDayService,
    toggle_service: MockToggleService,
}

impl ReportingMocks {
    fn new() -> Self {
        // toggle_service: default = Ok(None) (automation off)
        let mut toggle_service = MockToggleService::new();
        toggle_service
            .expect_get_toggle_value()
            .returning(|_, _, _| Ok(None));
        Self { /* ... */ special_day_service: MockSpecialDayService::new(), toggle_service }
    }

    fn build(self) -> ReportingServiceImpl<TestDeps> {
        ReportingServiceImpl {
            /* all deps wrapped in Arc::new(...) */
            special_day_service: Arc::new(self.special_day_service),
            toggle_service: Arc::new(self.toggle_service),
        }
    }
}
```

#### `make_holiday` helper (lines 122–134)

```rust
fn make_holiday(year: u32, calendar_week: u8, day_of_week: DayOfWeek) -> SpecialDay {
    SpecialDay {
        id: Uuid::new_v4(),
        year,
        calendar_week,
        day_of_week,
        day_type: SpecialDayType::Holiday,
        time_of_day: None,
        created: Some(datetime!(2024 - 01 - 01 00:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}
```

#### `make_holiday_extra_hours` helper (lines 137–149)

```rust
fn make_holiday_extra_hours(amount: f32, day: time::Date) -> ExtraHours {
    ExtraHours {
        id: Uuid::new_v4(),
        sales_person_id: fixture_sales_person_id(),
        amount,
        category: ExtraHoursCategory::Holiday,
        description: Arc::from("manual holiday"),
        date_time: time::PrimitiveDateTime::new(day, time::Time::from_hms(9, 0, 0).unwrap()),
        created: Some(datetime!(2024 - 01 - 01 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}
```

#### `get_week` mock setup pattern from HOL-03 (lines 544–572)

These 6 mock expectations are required for every `get_week` call in tests:

```rust
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
```

#### Toggle mock — active cutoff pattern (from HOL-01, line 231–235)

```rust
mocks.toggle_service = MockToggleService::new();
mocks
    .toggle_service
    .expect_get_toggle_value()
    .returning(|_, _, _| Ok(Some(Arc::from("2024-01-01"))));
```

#### SpecialDay mock — 1 holiday on KW23 Monday (from HOL-01, lines 239–247)

```rust
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
```

#### HOL-03 new assertions (D-34-03, HSP-01/02/03)

After the rebuild, HOL-03 must provide active toggle + SpecialDay mocks (above) and assert:

```rust
// HSP-03: band guard — dynamic_hours UNCHANGED
assert!(
    (report.dynamic_hours - 40.0).abs() < 0.01,
    "HSP-03: dynamic_hours must remain 40h (band invariant), got {}",
    report.dynamic_hours
);

// HSP-01: expected_hours reduced by 8h derived holiday
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
```

#### HSP-04a — Stichtag gate (cutoff AFTER holiday date → no effect)

Toggle returns cutoff = `"2024-12-31"` (holiday is 2024-06-03, i.e. before cutoff):

```rust
mocks.toggle_service = MockToggleService::new();
mocks
    .toggle_service
    .expect_get_toggle_value()
    .returning(|_, _, _| Ok(Some(Arc::from("2024-12-31"))));
```

Assert:

```rust
assert!((report.expected_hours - 40.0).abs() < 0.01, "HSP-04a: expected_hours must be 40h (before cutoff)");
assert!(report.holiday_hours.abs() < 0.01, "HSP-04a: holiday_hours must be 0 (before cutoff)");
assert!((report.dynamic_hours - 40.0).abs() < 0.01, "HSP-04a: dynamic_hours must be 40h");
```

#### HSP-04b — manual-wins (ExtraHours(Holiday) + SpecialDay → credited once)

Override `find_by_week` to return the manual holiday entry:

```rust
let manual_holiday = make_holiday_extra_hours(8.0, date!(2024 - 06 - 03));
let extras: Arc<[ExtraHours]> = Arc::from(vec![manual_holiday]);
mocks.extra_hours_service = MockExtraHoursService::new();
mocks
    .extra_hours_service
    .expect_find_by_week()
    .returning(move |_, _, _, _| Ok(extras.clone()));
```

Assert:

```rust
// Manual-wins: holiday_hours == 8.0 (manual), NOT 16.0 (double-count)
assert!(
    (report.holiday_hours - 8.0).abs() < 0.01,
    "HSP-04b: holiday_hours must be 8.0 (manual wins), not 16.0; got {}",
    report.holiday_hours
);
```

---

## Shared Patterns

### Dynamic-week guard
**Source:** `service_impl/src/reporting.rs:1066–1071`
**Apply to:** `holiday_derived_gated` term in get_week — identical `!has_contract_row || planned_hours <= 0.0` condition.

### `build_derived_holiday_map` reuse (no hand-rolling)
**Source:** `service_impl/src/reporting.rs:151–242`
**Apply to:** 4th injection point — call the existing helper, do NOT re-implement toggle lookup, Stichtag gate, manual-wins, or ISO-week arithmetic.

### `context.clone()` propagation
**Source:** Injection points at lines 361 and 754
**Apply to:** 4th injection point — `context.clone()` as the last argument to `build_derived_holiday_map`.

### Mock replacement pattern (override default in `new()`)
**Source:** `reporting_holiday_auto_credit.rs:231` (HOL-01), `reporting_holiday_auto_credit.rs:276` (HOL-02)
**Apply to:** HOL-03 rebuild and HSP-04 tests — replace the default `toggle_service` (Ok(None)) with a new `MockToggleService::new()` configured for the specific cutoff under test.

---

## No Analog Found

None. All patterns map directly to existing code in the same file.

---

## Metadata

**Analog search scope:** `service_impl/src/reporting.rs`, `service_impl/src/test/reporting_holiday_auto_credit.rs`
**Key line ranges read:** reporting.rs:151–242, 340–415, 740–784, 884–1112; test file:1–150, 150–350, 530–620
**Pattern extraction date:** 2026-06-30
