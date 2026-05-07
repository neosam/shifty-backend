# Phase 5: Slot Paid Capacity Warning - Pattern Map

**Mapped:** 2026-05-03
**Files analyzed:** 11 files to be created or modified
**Analogs found:** 11 / 11 (all have strong in-repo patterns to copy)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `migrations/sqlite/YYYYMMDDHHMMSS_add-max-paid-employees-to-slot.sql` | migration | DDL | `migrations/sqlite/20240813080347_add-column-min-resources.sql` | exact (same table, same Slot-capacity concept; differs only NOT NULL → nullable) |
| `dao/src/slot.rs` (modify) | model (DAO trait) | CRUD | self (existing `SlotEntity`) — `min_resources: u8` analog | exact |
| `dao_impl_sqlite/src/slot.rs` (modify) | model (DAO impl) | CRUD | self (existing 4 read sites + create/update) | exact |
| `service/src/slot.rs` (modify) | model (service trait + DTO) | CRUD | self (existing `Slot.min_resources` + `From` impls) | exact |
| `service_impl/src/slot.rs` (modify) | service | CRUD | self (existing `update_slot` validation block at lines 285-350) | exact |
| `service/src/warning.rs` (modify, add variant) | model (enum) | event-driven | self (existing 4 variants at lines 22-54) | exact |
| `service_impl/src/shiftplan_edit.rs` (modify, emit new warning) | service | request-response | self (existing `book_slot_with_conflict_check` at lines 398-495) | exact |
| `service_impl/src/shiftplan.rs` (modify, add `current_paid_count` to read DTO) | service (Business-Logic-Tier) | read-aggregation | self (existing `build_shiftplan_day` at lines 27-114) | exact |
| `service/src/shiftplan.rs` (modify `ShiftplanSlot`) | model | — | self (struct at line 47-50) | exact |
| `rest-types/src/lib.rs` (modify `SlotTO`, `WarningTO`, `ShiftplanSlotTO`) | model (DTO) | — | self (struct + `From`-impl pairs at lines 305-358, 1655-1735) | exact |
| `rest/src/slot.rs` (no behavioural change; utoipa-annotation optional per phase scope) | controller | request-response | `rest/src/shiftplan_edit.rs` lines 122-155 (utoipa pattern) | role-match |
| Tests in `service_impl/src/test/slot.rs` + `service_impl/src/test/booking.rs` (or `shiftplan_edit.rs`) | test | mock | existing `test_create_slot` / `test_create` / `book_slot_with_conflict_check` tests | exact |

> **Note on the missing `rest/tests/openapi_snapshot.rs`:** CONTEXT.md mentions an OpenAPI snapshot file to update via `cargo insta review`. Commit `fdb70b5` (most recent) **deleted** that test. The `rest/tests/` directory is empty. The planner should drop the snapshot-update task from scope, OR re-introduce the test as a separate decision (not within this phase's plan). See "No Analog Found" section.

---

## Pattern Assignments

### `migrations/sqlite/YYYYMMDDHHMMSS_add-max-paid-employees-to-slot.sql` (migration, DDL)

**Analog:** `migrations/sqlite/20240813080347_add-column-min-resources.sql`

**Direct copy** (the entire file is 2 lines):

```sql
ALTER TABLE slot
ADD COLUMN min_resources INTEGER DEFAULT 2 NOT NULL
```

**Adaptation for Phase 5** (nullable, no default):

```sql
ALTER TABLE slot
ADD COLUMN max_paid_employees INTEGER
```

**Pitfall:** Do NOT add `NOT NULL` and do NOT add `DEFAULT`. Existing rows must remain `NULL` (D-15: "kein Backfill"). The `is_paid` migration (`20240618125847_paid-sales-persons.sql` line 4) shows how to add `BOOLEAN DEFAULT 0 NOT NULL` — that is the **wrong** pattern here; copy the structure of `min_resources` but strip both `DEFAULT` and `NOT NULL`.

**Naming:** `YYYYMMDDHHMMSS` — use `sqlx migrate add add-max-paid-employees-to-slot --source migrations/sqlite` from inside `nix develop` (per `CLAUDE.local.md`); do NOT run `sqlx setup` or `sqlx database reset` (destructive — confirm with user first per MEMORY.md).

---

### `dao/src/slot.rs` (model, CRUD trait)

**Analog:** self — `SlotEntity` at `dao/src/slot.rs` lines 10-22, especially line 16 (`pub min_resources: u8`).

**Field-add pattern** (insert into the existing struct, after `min_resources`):

```rust
#[derive(Debug, PartialEq, Eq)]
pub struct SlotEntity {
    pub id: Uuid,
    pub day_of_week: DayOfWeek,
    pub from: time::Time,
    pub to: time::Time,
    pub min_resources: u8,
    pub max_paid_employees: Option<u8>,   // NEW — Phase 5
    pub valid_from: time::Date,
    pub valid_to: Option<time::Date>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
    pub shiftplan_id: Option<Uuid>,
}
```

**Trait surface:** `SlotDao` (lines 24-61) requires NO new methods — `create_slot`, `update_slot`, and the four `get_*` methods all take/return `SlotEntity`, so the new field flows through automatically. The auto-generated `MockSlotDao` (via `#[automock]` at line 24) updates with the struct.

---

### `dao_impl_sqlite/src/slot.rs` (model, CRUD impl)

**Analog:** self — there are **four** read sites that build `SlotEntity` (lines 36-56, 73-94, 122-144, 171-194) plus `create_slot` (lines 197-231) and `update_slot` (lines 233-254).

**Read pattern — current `min_resources: row.min_resources as u8`** at e.g. line 42:

```rust
SlotEntity {
    id: Uuid::from_slice(row.id.as_ref())?,
    day_of_week: DayOfWeek::from_number(row.day_of_week as u8)
        .ok_or(DaoError::InvalidDayOfWeek(row.day_of_week as u8))?,
    from: Time::parse(&row.time_from, &Iso8601::TIME)?,
    to: Time::parse(&row.time_to, &Iso8601::TIME)?,
    min_resources: row.min_resources as u8,
    // ...
}
```

**Read pattern for nullable `max_paid_employees`** (apply at all 4 read sites):

```rust
max_paid_employees: row.max_paid_employees.map(|n| n as u8),
```

**Pitfall — nullable INTEGER inference:** SQLx infers nullable INTEGER columns as `Option<i64>` (not `Option<u8>` — Rust has no compile-time way to tell SQLx "this i64 fits a u8"). The `.map(|n| n as u8)` cast is the standard pattern (compare `min_resources: row.min_resources as u8` at line 42 — non-null INTEGER → `i64`, then cast to u8). Truncation is safe in practice since `max_paid_employees` ∈ [0, 255]; if you want belt-and-suspenders, use `.map(|n| u8::try_from(n).map_err(|_| DaoError::…))` and add an `InvalidMaxPaidEmployees(i64)` variant to `DaoError` — this is NOT done elsewhere for `min_resources`, so keeping the bare `as u8` is consistent.

**SELECT-list update — current at line 29 / 67 / 110-111 / 158-159 (4 places):**

```rust
// BEFORE
"SELECT id, day_of_week, time_from, time_to, min_resources, valid_from, valid_to, deleted, update_version, shiftplan_id FROM slot ..."

// AFTER (add max_paid_employees in the same column position as the entity field)
"SELECT id, day_of_week, time_from, time_to, min_resources, max_paid_employees, valid_from, valid_to, deleted, update_version, shiftplan_id FROM slot ..."
```

**`create_slot` INSERT pattern — current at lines 212-226:**

```rust
let min_resources = slot.min_resources;
let shiftplan_id_vec = slot.shiftplan_id.map(|id| id.as_bytes().to_vec());
query!("INSERT INTO slot (id, day_of_week, time_from, time_to, valid_from, valid_to, deleted, update_version, update_process, min_resources, shiftplan_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    id_vec, day_of_week, from, to, valid_from, valid_to, deleted,
    version_vec, process, min_resources, shiftplan_id_vec,
)
```

**Adapted INSERT** (add `max_paid_employees` binding — directly bind `Option<u8>`, sqlx supports it natively):

```rust
let min_resources = slot.min_resources;
let max_paid_employees = slot.max_paid_employees;   // NEW — Option<u8> binds directly
let shiftplan_id_vec = slot.shiftplan_id.map(|id| id.as_bytes().to_vec());
query!("INSERT INTO slot (id, day_of_week, time_from, time_to, valid_from, valid_to, deleted, update_version, update_process, min_resources, max_paid_employees, shiftplan_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    id_vec, day_of_week, from, to, valid_from, valid_to, deleted,
    version_vec, process, min_resources, max_paid_employees, shiftplan_id_vec,
)
```

**`update_slot` UPDATE pattern — current at lines 243-249:** the existing UPDATE only writes `valid_to, deleted, update_version, update_process`. **It does NOT currently write `min_resources`** — the modify_slot service in `service_impl/src/shiftplan_edit.rs` lines 101-115 handles `min_resources` changes by deleting + recreating the slot row. **Decision for the planner (D-11 implication):** either (a) extend `update_slot` to also write `max_paid_employees` (cheapest, lets the existing slot-service `update_slot` mutate the limit in-place), or (b) follow `min_resources` precedent and require a `modify_slot` (delete + recreate) path. Recommendation: option (a) — `max_paid_employees` is a soft, non-temporal config (no historical replay implication) so updating in-place on the existing row is safe. Concrete patch:

```rust
// AFTER
let max_paid_employees = slot.max_paid_employees;
query!("UPDATE slot SET valid_to = ?, deleted = ?, max_paid_employees = ?, update_version = ?, update_process = ? WHERE id = ?",
    valid_to, deleted, max_paid_employees, version_vec, process, id_vec,
)
```

**Pitfall — sqlx-prepare cache:** SQLx `query!`/`query_as!` macros require a fresh local DB. After running the migration locally, `cargo build` will fail until you re-run the migration:
```
cd shifty-backend && nix develop --command sqlx migrate run --source migrations/sqlite
```
(Per MEMORY.md `feedback_destructive_db_ops.md` — use `migrate run` not `database reset`.)

---

### `service/src/slot.rs` (model, service trait + DTO)

**Analog:** self — `Slot` struct at lines 12-24, plus `From<&SlotEntity>` (lines 25-40) and `From<&Slot>` (lines 41-56).

**Add field to struct** (line 18 area):

```rust
pub struct Slot {
    pub id: Uuid,
    pub day_of_week: DayOfWeek,
    pub from: time::Time,
    pub to: time::Time,
    pub min_resources: u8,
    pub max_paid_employees: Option<u8>,   // NEW — Phase 5
    // ...
}
```

**Update both `From` impls** — line 33 and line 49 — to map `max_paid_employees` straight through (analog to `min_resources`).

**Trait surface:** `SlotService` (lines 71-127) requires NO new method. The new field flows through `create_slot` / `update_slot` / `get_*` automatically because they take/return `Slot`.

---

### `service_impl/src/slot.rs` (service, CRUD)

**Analog:** self — `update_slot` at lines 285-350 is the slot-mutating path that already enforces field-immutability rules.

**Validation pattern** — currently locks `day_of_week`, `from`, `to`, `valid_from` as immutable (lines 313-329):

```rust
let mut validation = Vec::new();
if persisted_slot.day_of_week != slot.day_of_week {
    validation.push(ValidationFailureItem::ModificationNotAllowed(
        "day_of_week".into(),
    ));
}
if persisted_slot.from != slot.from {
    validation.push(ValidationFailureItem::ModificationNotAllowed("from".into()));
}
// ... etc
```

**For Phase 5:** `max_paid_employees` SHOULD be mutable (D-11 explicit: "Update von `slot.max_paid_employees` braucht Rolle `shiftplanner`"). So **do NOT add it to the immutability check** — let it flow through the UPDATE. The existing `SHIFTPLANNER_PRIVILEGE` check at line 292-294 already covers the permission requirement.

**`min_resources` is similarly mutable already** — note it is NOT in the validation list, even though the SQLite-DAO `update_slot` does not currently persist it (this is the inconsistency flagged above; it's been handled in `shiftplan_edit::modify_slot` via delete-and-recreate). For `max_paid_employees`, recommend persisting it via plain `update_slot` per the DAO patch above.

**`create_slot` validation at lines 203-258:** no new validation needed. `max_paid_employees: Option<u8>` is always valid (any `u8` ≥ 0 is accepted; `None` means "no limit"). Just let it flow through into the new `Slot` constructed at lines 248-252.

---

### `service/src/warning.rs` (model, enum)

**Analog:** self — existing 4 variants of `Warning` at lines 22-54.

**Add variant pattern** — copy the structural style of `BookingOnUnavailableDay` (lines 32-39) which carries pure-data fields, no enums of other domain types:

```rust
pub enum Warning {
    BookingOnAbsenceDay { /* ... */ },
    BookingOnUnavailableDay { /* ... */ },
    AbsenceOverlapsBooking { /* ... */ },
    AbsenceOverlapsManualUnavailable { /* ... */ },

    /// NEW Phase 5 — emittiert beim Anlegen eines Bookings, wenn der Slot
    /// ein konfiguriertes `max_paid_employees`-Limit hat und der aktuelle
    /// Live-Count der bezahlten Mitarbeiter:innen in derselben (year, week,
    /// slot)-Kombination das Limit strikt übersteigt (`current > max`,
    /// D-06). Buchung wird NICHT zurückgerollt (D-07).
    PaidEmployeeLimitExceeded {
        slot_id: Uuid,
        booking_id: Uuid,
        year: u32,
        week: u8,
        current_paid_count: u8,
        max_paid_employees: u8,
    },
}
```

**Documentation header style:** Each variant gets a 2-3 line `///`-comment describing trigger + reference to the decision-ID. See lines 24-25, 32-33, 40-41, 47-49 for the established style.

**Stable ordering note:** The Phase-3 doc-comment (line 21-22) calls out "Stable per D-Phase3-14" — Phase 5 adds a 5th variant; this is additive and serde-tag-disambiguated, so it does NOT break wire compatibility for existing variants.

---

### `service_impl/src/shiftplan_edit.rs` (service, request-response)

**Analog:** self — `book_slot_with_conflict_check` at lines 398-495.

**Imports** (line 17) — already imports `Warning`. No change needed.

**Booking-create + warning emission pattern** at lines 459-494:

```rust
// Persist via Basic-Service — BookingService::create UNVERÄNDERT
// (D-Phase3-18 Regression-Lock).
let persisted_booking = self
    .booking_service
    .create(booking, Authentication::Full, tx.clone().into())
    .await?;

// Warnings mit echter persistierter Booking-ID.
let mut warnings: Vec<Warning> = Vec::new();
for ap in absence_periods.iter() {
    warnings.push(Warning::BookingOnAbsenceDay {
        booking_id: persisted_booking.id,
        // ...
    });
}
// ... ManualUnavailable loop ...

self.transaction_dao.commit(tx).await?;
Ok(BookingCreateResult {
    booking: persisted_booking,
    warnings: Arc::from(warnings),
})
```

**Phase-5 emission insertion point** — between the existing warning emission and the commit (so the new check happens after persist + after existing warnings are computed):

```rust
// NEW Phase 5 — Paid-Capacity-Check (D-04, D-06, D-08).
// Slot-Lookup wurde oben bereits gemacht (line 419-422); `slot.max_paid_employees`
// ist `Option<u8>`. Nur wenn gesetzt, zählen wir.
if let Some(max) = slot.max_paid_employees {
    let current_paid_count = self
        .count_paid_bookings_in_slot_week(
            booking.slot_id,
            booking.year,
            booking.calendar_week as u8,
            tx.clone(),
        )
        .await?;
    if current_paid_count > max {
        warnings.push(Warning::PaidEmployeeLimitExceeded {
            slot_id: booking.slot_id,
            booking_id: persisted_booking.id,
            year: booking.year,
            week: booking.calendar_week as u8,
            current_paid_count,
            max_paid_employees: max,
        });
    }
}
```

**Helper signature** (per CONTEXT.md "Claude's Discretion") — recommendation: implement as private method on `ShiftplanEditServiceImpl` (NOT on `BookingService` — see "Service-Tier" pitfall below):

```rust
impl<Deps: ShiftplanEditServiceDeps> ShiftplanEditServiceImpl<Deps> {
    async fn count_paid_bookings_in_slot_week(
        &self,
        slot_id: Uuid,
        year: u32,
        week: u8,
        tx: Deps::Transaction,
    ) -> Result<u8, ServiceError> {
        // Strategy: existing get_for_week + filter by slot_id, then
        // for each booking lookup sales_person.is_paid via SalesPersonService.
        // Alternative (more efficient): add SalesPersonService::all_paid()
        // (already exists in DAO trait line 27 of dao/src/sales_person.rs)
        // and intersect.
        let bookings = self
            .booking_service
            .get_for_week(week, year, Authentication::Full, Some(tx.clone()))
            .await?;
        let paid_persons = self
            .sales_person_service
            .all_paid(Authentication::Full, Some(tx.clone()))
            .await?;
        let paid_ids: std::collections::HashSet<Uuid> =
            paid_persons.iter().map(|sp| sp.id).collect();
        let count = bookings
            .iter()
            .filter(|b| b.slot_id == slot_id && b.deleted.is_none())
            .filter(|b| paid_ids.contains(&b.sales_person_id))
            .count();
        Ok(count.min(u8::MAX as usize) as u8)
    }
}
```

**Pitfall — `SalesPersonService::all_paid`:** check whether `service::sales_person::SalesPersonService` exposes an `all_paid` wrapper around `SalesPersonDao::all_paid` (`dao/src/sales_person.rs` line 27). If not, the planner needs to either (a) add a one-line wrapper to the SP-service, or (b) use `get_all` + filter on `is_paid`. Both are acceptable; option (a) matches the Basic-Tier convention.

**Pitfall — Service-Tier (CLAUDE.md "Service-Tier-Konventionen"):** CONTEXT.md D-12 says "Limit-Check + Read-Aggregation leben im **`BookingService`**". This is **incompatible** with the Basic-Tier rule: `BookingService` is Basic-Tier and may NOT consume `SalesPersonService` (it currently does — see line 26 of `service_impl/src/booking.rs`, which violates the rule today; this is a pre-existing inconsistency). The cleanest path forward is to put the new check in the **Business-Logic-Tier** `ShiftplanEditService` instead — it already consumes both `BookingService` and `SalesPersonService` (lines 29 and 32 of `service_impl/src/shiftplan_edit.rs`) and already runs the conflict-aware booking flow. **Recommend:** the planner override D-12 here and emit the warning from `ShiftplanEditService::book_slot_with_conflict_check`, NOT from `BookingService::create`. This keeps `BookingService::create` as the dumb persist (D-Phase3-18 "Regression-Lock") and isolates Phase-5 logic in the same service that already handles cross-source warnings.

**Note on POST endpoints:** The existing `POST /booking` (`rest/src/booking.rs` lines 16-27) calls `BookingService::create` directly (no warnings). The Phase-3 conflict-aware path is `POST /shiftplan-edit/booking` (`rest/src/shiftplan_edit.rs` lines 32-35 + handler at lines 134-155). Phase 5 warnings will only fire on the Phase-3 endpoint — confirm with the planner whether this is the intended behaviour or whether the legacy `POST /booking` also needs to emit warnings (probably yes; either route the legacy endpoint through `book_slot_with_conflict_check`, or add a slimmer per-slot variant of the helper).

---

### `service_impl/src/shiftplan.rs` (service, read-aggregation)

**Analog:** self — `build_shiftplan_day` at lines 27-114.

**Slot-bookings aggregation pattern** at lines 67-95:

```rust
// Find bookings for this slot
let slot_bookings = bookings
    .iter()
    .filter(|b| b.slot_id == slot.id)
    .map(|booking| {
        let sales_person = sales_persons
            .iter()
            .find(|sp| sp.id == booking.sales_person_id)
            .ok_or_else(|| ServiceError::EntityNotFound(booking.sales_person_id))?
            .clone();
        // ...
        Ok(ShiftplanBooking {
            booking: booking.clone(),
            sales_person,
            self_added,
        })
    })
    .collect::<Result<Vec<_>, ServiceError>>()?;

day_slots.push(ShiftplanSlot {
    slot: slot.clone(),
    bookings: slot_bookings,
});
```

**Phase-5 augmentation** — compute `current_paid_count` as part of the same loop (the `sales_person` is already resolved per-booking, so `is_paid` is in-hand):

```rust
let slot_bookings = bookings
    .iter()
    .filter(|b| b.slot_id == slot.id)
    .map(|booking| {
        let sales_person = sales_persons
            .iter()
            .find(|sp| sp.id == booking.sales_person_id)
            .ok_or_else(|| ServiceError::EntityNotFound(booking.sales_person_id))?
            .clone();
        // ...
        Ok(ShiftplanBooking { /* ... */ })
    })
    .collect::<Result<Vec<_>, ServiceError>>()?;

// NEW Phase 5 — derive current_paid_count from the resolved bookings
// (D-04: count active bookings whose sales_person.is_paid = true;
// soft-deleted bookings already filtered upstream by booking_dao;
// soft-deleted sales_persons already filtered by SalesPersonService).
let current_paid_count = slot_bookings
    .iter()
    .filter(|sb| sb.sales_person.is_paid)
    .count()
    .min(u8::MAX as usize) as u8;

day_slots.push(ShiftplanSlot {
    slot: slot.clone(),
    bookings: slot_bookings,
    current_paid_count,   // NEW field
});
```

**`ShiftplanSlot` struct extension** (in `service/src/shiftplan.rs` lines 47-50):

```rust
#[derive(Debug, Clone)]
pub struct ShiftplanSlot {
    pub slot: crate::slot::Slot,
    pub bookings: Vec<ShiftplanBooking>,
    pub current_paid_count: u8,   // NEW Phase 5
}
```

**Discretion (CONTEXT.md "Claude's Discretion"):** the planner picks whether `current_paid_count` is `Option<u8>` (only computed when `slot.max_paid_employees.is_some()`) or always `u8` (always computed). Recommendation: always `u8`. Cost is one extra `.filter().count()` per slot — Shiftplan-Week has ~50 slots × 7 days = 350 ops, all in-memory on already-loaded data. Premature optimization to skip. The DTO contract is also simpler (no `Option` round-trip).

**Apply same change to `build_shiftplan_day_for_sales_person`** (lines 130-178+) — it calls `build_shiftplan_day` first (lines 142-149), so the `current_paid_count` is already populated transitively. No additional change needed there.

---

### `rest-types/src/lib.rs` (model, DTOs)

**Three sub-changes:**

#### a) Extend `SlotTO` (lines 305-358)

**Pattern** — current `min_resources: u8` field (line 314) maps directly:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SlotTO {
    #[serde(default)]
    pub id: Uuid,
    pub day_of_week: DayOfWeekTO,
    #[schema(value_type = String, format = "time")]
    pub from: time::Time,
    #[schema(value_type = String, format = "time")]
    pub to: time::Time,
    pub min_resources: u8,
    #[serde(default)]                       // NEW — accept absent in payloads
    pub max_paid_employees: Option<u8>,     // NEW — Phase 5
    pub valid_from: time::Date,
    // ...
}
```

**Both `From` impls** at lines 326-358 — map `max_paid_employees` through (analog to `min_resources` at lines 333 and 350).

**Pitfall:** the `#[serde(default)]` is required so existing API consumers that omit the field don't break — Phase 5 must remain backward-compatible.

#### b) Extend `WarningTO` (lines 1655-1735)

**Pattern** — copy `BookingOnUnavailableDay` variant (lines 1668-1673) as it's the closest structural analog (pure-data fields):

```rust
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum WarningTO {
    BookingOnAbsenceDay { /* ... */ },
    BookingOnUnavailableDay {
        booking_id: Uuid,
        year: u32,
        week: u8,
        day_of_week: DayOfWeekTO,
    },
    // ... other existing variants ...

    /// NEW Phase 5 — Paid-Mitarbeiter-Limit überschritten (D-08).
    PaidEmployeeLimitExceeded {
        slot_id: Uuid,
        booking_id: Uuid,
        year: u32,
        week: u8,
        current_paid_count: u8,
        max_paid_employees: u8,
    },
}
```

**Update `From<&service::warning::Warning> for WarningTO`** at lines 1692-1735 — add a 5th match-arm:

```rust
service::warning::Warning::PaidEmployeeLimitExceeded {
    slot_id, booking_id, year, week, current_paid_count, max_paid_employees,
} => Self::PaidEmployeeLimitExceeded {
    slot_id: *slot_id,
    booking_id: *booking_id,
    year: *year,
    week: *week,
    current_paid_count: *current_paid_count,
    max_paid_employees: *max_paid_employees,
},
```

**JSON wire-form:** `{ "kind": "paid_employee_limit_exceeded", "data": { "slot_id": "...", "booking_id": "...", "year": 2026, "week": 18, "current_paid_count": 3, "max_paid_employees": 2 } }`. Tag is auto-derived via `#[serde(rename_all = "snake_case")]` at line 1654.

#### c) Extend `ShiftplanSlotTO` (lines 967-971)

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ShiftplanSlotTO {
    pub slot: SlotTO,
    pub bookings: Vec<ShiftplanBookingTO>,
    pub current_paid_count: u8,   // NEW Phase 5
}
```

**`From` impl** at lines 1003-1010 — pass through:

```rust
impl From<&service::shiftplan::ShiftplanSlot> for ShiftplanSlotTO {
    fn from(slot: &service::shiftplan::ShiftplanSlot) -> Self {
        Self {
            slot: (&slot.slot).into(),
            bookings: slot.bookings.iter().map(Into::into).collect(),
            current_paid_count: slot.current_paid_count,
        }
    }
}
```

**Note on `BlockTO`** (lines 1438-1448) — also contains `slots: Vec<SlotTO>`, so the `max_paid_employees` field on `SlotTO` cascades there automatically. No structural change to `BlockTO`.

---

### `rest/src/slot.rs` (controller — minimal change)

**Analog:** `rest/src/shiftplan_edit.rs` lines 122-155 (utoipa-annotated handler — current best-practice pattern).

**Current state of `rest/src/slot.rs`** — the existing handlers `get_all_slots`, `get_slot`, `get_slots_for_week`, `create_slot`, `update_slot` (lines 26-142) have **no `#[utoipa::path]` annotations**. They're not registered in any `ApiDoc`. This is a known gap (similar gap noted in `rest/src/shiftplan_edit.rs` lines 202-208 for `edit_slot`/`delete_slot`/`add_vacation`).

**Phase-5 scope decision:** Phase 5 changes the JSON shape of `SlotTO` (adds `max_paid_employees`) and `ShiftplanSlotTO` (adds `current_paid_count`). It does NOT add new endpoints. The existing PUT `/slot/{id}` accepts a `SlotTO` body — once `SlotTO` carries `max_paid_employees`, the field is automatically writable through the existing handler with no rewrite. Permission check (D-11: `shiftplanner`) is enforced at the service layer (`service_impl/src/slot.rs` line 292-294) — already covers it.

**Recommendation:** No code change required in `rest/src/slot.rs` for the data-shape change. The planner SHOULD add `#[utoipa::path]` annotations to `update_slot` if the broader OpenAPI-coverage is in scope; otherwise leave it as-is. Pattern to copy if annotating:

```rust
#[utoipa::path(
    put,
    path = "/{id}",
    tags = ["Slot"],
    request_body = SlotTO,
    responses(
        (status = 200, description = "Slot updated", body = SlotTO),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn update_slot<RestState: RestStateDef>(/* ... */) -> Response { /* ... */ }
```

And register a new `SlotApiDoc` in `rest/src/lib.rs` lines 485-507 (compare `(path = "/sales-person", api = SalesPersonApiDoc)` style at line 490).

---

### Tests in `service_impl/src/test/slot.rs` and `service_impl/src/test/booking.rs` (or `shiftplan_edit.rs`)

**Analog 1 (Slot DAO-roundtrip-style):** `service_impl/src/test/slot.rs` lines 33-46 (`generate_default_slot`), 47-60 (`generate_default_slot_entity`), 244+ (`test_create_slot`).

**Pattern — fixture setup**:

```rust
pub fn generate_default_slot() -> Slot {
    Slot {
        id: default_id(),
        day_of_week: DayOfWeek::Monday,
        from: time::Time::from_hms(10, 0, 0).unwrap(),
        to: time::Time::from_hms(11, 0, 0).unwrap(),
        min_resources: 2,
        max_paid_employees: None,   // NEW — default to no-limit
        valid_from: time::Date::from_calendar_date(2022, 1.try_into().unwrap(), 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: default_version(),
        shiftplan_id: Some(default_shiftplan_id()),
    }
}
```

Add Phase-5 specific fixture variant:

```rust
pub fn generate_slot_with_paid_limit(max: u8) -> Slot {
    Slot { max_paid_employees: Some(max), ..generate_default_slot() }
}
```

**Analog 2 (Booking-create with mocks):** `service_impl/src/test/booking.rs` lines 113-192 (`build_dependencies`) and 297-341 (`test_create`).

**Pattern — adding a paid-fixture sales_person:**

```rust
deps.sales_person_service.expect_get_all().returning(|_, _| {
    Ok(Arc::from([
        SalesPerson { id: paid_sp_id(), is_paid: true, /* ... */ },
        SalesPerson { id: unpaid_sp_id(), is_paid: false, /* ... */ },
    ]))
});
```

**Required test cases per CONTEXT.md (Pflicht-Tests):**

1. **DAO-Roundtrip** — covered by SQLx integration tests in `dao_impl_sqlite/` (if such exist) or via the `service_impl::test::slot` mock-based tests. The mock tests trigger the new field through `expect_create_slot`/`expect_update_slot`/`expect_get_slot` mocks.

2. **`test_book_paid_into_full_slot_emits_warning`** — fixture: slot with `max_paid_employees: Some(2)`, two existing bookings with paid sales_persons in same (year, week). Booking a third paid SP → `BookingCreateResult.warnings` contains `Warning::PaidEmployeeLimitExceeded { current_paid_count: 3, max_paid_employees: 2, .. }`. Place in `service_impl/src/test/shiftplan_edit.rs` (since the warning is emitted there per the Service-Tier pitfall above).

3. **`test_book_unpaid_into_full_slot_no_warning`** — same fixture, but new SP has `is_paid: false`. Assert `warnings.is_empty()`.

4. **`test_book_with_no_limit_no_warning`** — slot has `max_paid_employees: None`. Assert no compute, no warning.

5. **`test_book_paid_in_absence_still_counts`** — sales_person in vacation/absence is still gebucht; should still count toward limit (D-05).

6. **`test_shiftplan_week_emits_current_paid_count`** — fixture: slot with mixed paid/unpaid bookings, assert `ShiftplanSlot.current_paid_count` reflects the paid-only count. Place in `service_impl/src/test/shiftplan.rs` (look for existing `test_get_shiftplan_week*` patterns there as the analog).

7. **REST-Test for permission** — slot-update with `max_paid_employees` as `shiftplanner` (200) and as non-shiftplanner (403). The service-layer permission check at `service_impl/src/slot.rs` line 292-294 makes this a service-test, not a REST-integration-test (the service's mock-based test framework already covers role checks — see `test_create_slot_no_permission`-style tests).

---

## Shared Patterns

### Service-Tier: where Phase-5 logic lives

**Source:** `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen".
**Apply to:** `service_impl/src/shiftplan_edit.rs`.

Phase-5 warning emission lives in `ShiftplanEditService::book_slot_with_conflict_check` (Business-Logic-Tier), NOT in `BookingService::create` (Basic-Tier). Reasoning: Basic-Tier services may only depend on DAOs + `PermissionService` + `TransactionDao`. Cross-entity computations (counting bookings × sales_person.is_paid) are inherently Business-Logic-Tier.

### Transaction pattern

**Source:** `shifty-backend/CLAUDE.md` § "Transaction Management"; concrete excerpt at every service method (e.g. `service_impl/src/booking.rs` lines 187-300).
**Apply to:** the new `count_paid_bookings_in_slot_week` helper.

```rust
async fn do_something(&self, tx: Option<Self::Transaction>) -> Result<T, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // ... business logic and DAO calls ...
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

For the new private helper, accept `tx: Deps::Transaction` directly (already-active tx, no `Option<>`), since it's only ever called from within `book_slot_with_conflict_check` which has already `use_transaction`'d. See e.g. `BookingServiceImpl::check_booking_permission` (`service_impl/src/booking.rs` lines 34-68) for the in-tx-helper precedent — though that one uses `Option<Deps::Transaction>` which works either way.

### Soft-Delete filter

**Source:** `shifty-backend/CLAUDE.md` § "DAO Implementation" — "All queries include soft delete checks (`WHERE deleted IS NULL`)".
**Apply to:** none structurally — `BookingService::get_for_week` already filters soft-deleted bookings at the DAO level (see `dao_impl_sqlite/src/booking.rs`'s queries which all carry `WHERE deleted IS NULL`). Defensive in-memory `b.deleted.is_none()` filter in the helper is belt-and-suspenders, harmless. Same for `SalesPersonDao::all_paid` which already includes `WHERE deleted IS NULL AND is_paid = 1` (see `dao_impl_sqlite/src/sales_person.rs` line 72).

### `gen_service_impl!` macro

**Source:** `service_impl/src/shiftplan_edit.rs` lines 25-41 (existing 11-dep example) and `service_impl/src/booking.rs` lines 20-31 (8-dep example).
**Apply to:** none — Phase 5 does NOT add new dependencies. `ShiftplanEditServiceImpl` already has `BookingService`, `SalesPersonService`, `SlotService`, `TransactionDao`, `PermissionService`. All Phase-5 needs are in-hand.

### `ToSchema` derive on every DTO

**Source:** `shifty-backend/CLAUDE.md` § "OpenAPI Documentation".
**Apply to:** `SlotTO` (line 305 — already has it), `WarningTO` (line 1653 — already has it), `ShiftplanSlotTO` (line 967 — already has it). No new structs introduced, so no new derive needed.

### `#[serde(default)]` on optional new fields

**Source:** `rest-types/src/lib.rs` lines 307-308, 317-318, 322-323 — every optional `SlotTO` field has `#[serde(default)]`.
**Apply to:** the new `max_paid_employees: Option<u8>` field on `SlotTO`.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `rest/tests/openapi_snapshot.rs` | test | snapshot | **DELETED** in commit `fdb70b5` ("test(rest): remove flaky openapi snapshot test"). The `rest/tests/` directory is empty. CONTEXT.md's mention of "OpenAPI-Snapshot wird aktualisiert (cargo insta review)" is **stale** relative to the current repo state. Planner action: drop from scope OR add a separate decision to re-introduce the snapshot test (recommend dropping — the test was removed for being flaky). |

---

## Pitfalls Summary

1. **Migration must be nullable.** Copy the `min_resources` migration's structure but strip both `DEFAULT` and `NOT NULL`. Existing rows MUST stay `NULL`.
2. **SQLx infers nullable INTEGER as `Option<i64>`, not `Option<u8>`.** Use `.map(|n| n as u8)` at the read site (truncation is safe for u8 range).
3. **`update_slot` DAO method currently does NOT persist `min_resources`** — it only updates `valid_to, deleted, version, process`. If `max_paid_employees` is to be mutable in-place (D-11), extend the UPDATE statement to include it. (Alternative: route through `shiftplan_edit::modify_slot`'s delete-and-recreate pattern; not recommended — adds complexity for a non-temporal field.)
4. **Service-Tier conflict (D-12 vs CLAUDE.md):** D-12 says "BookingService" carries the check, but `BookingService` is Basic-Tier and shouldn't consume cross-entity logic. Recommendation: emit the warning from `ShiftplanEditService::book_slot_with_conflict_check` instead. Document the deviation explicitly in PLAN.md.
5. **Two booking-create endpoints exist:** `POST /booking` (legacy, Basic) and `POST /shiftplan-edit/booking` (Phase-3, conflict-aware). Phase-5 warnings only fire on the latter under the recommended Service-Tier choice. Decide whether the legacy endpoint also needs to emit warnings — if yes, route it through the conflict-aware path.
6. **OpenAPI-snapshot test is gone.** Don't try to update a file that doesn't exist; either drop from scope or re-introduce as a separate decision.
7. **`SalesPersonService` may not expose `all_paid()`.** The DAO has it (`dao/src/sales_person.rs` line 27). Either add a one-line wrapper to the service or use `get_all` + `.filter(|sp| sp.is_paid)` in the helper.
8. **`#[serde(default)]` on the new `max_paid_employees` SlotTO field is mandatory** for backward-compatibility — existing API consumers omit the field.
9. **`CURRENT_SNAPSHOT_SCHEMA_VERSION` does NOT need a bump** (D-14 confirmed; CLAUDE.md "Billing Period Snapshot Schema Versioning" — Phase 5 changes no `value_type`s).
10. **Local-DB workflow** (per CLAUDE.local.md + MEMORY.md): use `nix develop --command sqlx migrate run --source migrations/sqlite` to apply migrations additively. Never `sqlx setup` or `sqlx database reset` without user confirmation (destructive).
11. **VCS:** repo is jj-managed. Do not run `git commit` or `git add` directly. Auto-commit is disabled — user controls all commits manually.

---

## Metadata

**Analog search scope:**
- `migrations/sqlite/` (2 historical analogs: `min_resources`, `is_paid`)
- `dao/src/{slot,booking,sales_person}.rs`
- `dao_impl_sqlite/src/{slot,sales_person}.rs`
- `service/src/{slot,booking,warning,shiftplan,shiftplan_edit}.rs`
- `service_impl/src/{slot,booking,shiftplan,shiftplan_edit}.rs`
- `service_impl/src/test/{slot,booking}.rs`
- `rest/src/{slot,booking,shiftplan_edit,lib}.rs`
- `rest-types/src/lib.rs`
- v1.0 Phase 3 plan/verification docs (`.planning/phases/03-*/`)

**Files scanned:** 18 source files + 2 migrations + 3 phase-3 planning docs.

**Pattern extraction date:** 2026-05-03.
