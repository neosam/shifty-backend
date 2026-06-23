# Phase 8: Absence-CRUD-Page Foundation - Pattern Map

**Mapped:** 2026-05-08
**Files analyzed:** 24 (Backend: 8, Frontend: 16)
**Analogs found:** 24 / 24

> Misch-Phase Backend + Frontend. Frontend-Code sitzt unter
> `shifty-backend/shifty-dioxus/`. Alle Pfade in dieser Datei sind relativ zum
> Monorepo-Root (`shifty-backend/`) — der Wave-0-Gap-Block in 08-RESEARCH.md ist
> 1:1 die Quelle der Files-Tabelle.

---

## File Classification

### Backend (Wave 1 — Resturlaubs-Endpoint)

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `service/src/vacation_balance.rs` (NEW) | service-trait | request-response (Cross-Entity-Read) | `service/src/absence.rs` | exact (gleiche Trait-Form, BL-Tier) |
| `service/src/lib.rs` (modify) | mod-decl | n/a | `service/src/lib.rs` (existing block lines 7–45) | exact |
| `service_impl/src/vacation_balance.rs` (NEW) | service-impl | request-response (BL-Tier-Aggregation) | `service_impl/src/absence.rs` (`gen_service_impl!` Block lines 45–63) | exact |
| `service_impl/src/lib.rs` (modify) | mod-decl | n/a | analog `pub mod absence;` Eintrag | exact |
| `service_impl/src/test/vacation_balance.rs` (NEW) | backend-test | unit (mockall) | `service_impl/src/test/absence.rs` (1052 LOC, 30+ Tests) | exact |
| `service_impl/src/test/mod.rs` (modify) | mod-decl | n/a | `service_impl/src/test/mod.rs` Z. 2 (`pub mod absence;`) | exact |
| `rest/src/vacation_balance.rs` (NEW) | rest-handler | request-response | `rest/src/absence.rs` (260 LOC, 6 Endpoints) | exact |
| `rest/src/lib.rs` (modify) | rest-wiring | n/a | existing `mod absence;` (Z. 3), nest (Z. 567), ApiDoc nest-entry (Z. 485) | exact |
| `rest-types/src/lib.rs` (modify) | dto | data | `rest-types/src/lib.rs` `AbsencePeriodTO` Block (Z. 1565–1647) | exact |
| `shifty_bin/src/main.rs` (modify) | di-wiring | n/a | `AbsenceServiceDependencies` impl (Z. 228–255) + Konstruktor (Z. 798–815) | exact |

### Frontend (Wave 2 — Page + Modal + Service-Coroutine + State + i18n)

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `shifty-dioxus/src/state/absence_period.rs` (NEW) | state | data | `shifty-dioxus/src/state/employee.rs` (`Employee`, `From<&EmployeeReportTO>`) | role-match (struct + From-Impl) |
| `shifty-dioxus/src/state/vacation_balance.rs` (NEW) | state | data | analog dto-mirror; vgl. `state/employee.rs:170+` Struct-with-From | role-match |
| `shifty-dioxus/src/state/mod.rs` (modify) | mod-decl | n/a | existing block | exact |
| `shifty-dioxus/src/service/absence.rs` (NEW) | service-coroutine | event-driven (Action-Enum + GlobalSignal-Store) | `shifty-dioxus/src/service/employee.rs` (361 LOC) | exact |
| `shifty-dioxus/src/service/vacation_balance.rs` (NEW) | service-coroutine | event-driven | `shifty-dioxus/src/service/employee.rs` (Pattern-Reuse, einfacher) | role-match |
| `shifty-dioxus/src/service/mod.rs` (modify) | mod-decl | n/a | existing block (lines 1–19) | exact |
| `shifty-dioxus/src/api.rs` (modify) | api-client | request-response | `api.rs:392–468` (extra_hours-CRUD) | exact (CRUD + 409 + 422) |
| `shifty-dioxus/src/loader.rs` (modify) | loader | transform (TO→state, side-joins) | `loader.rs:76–102` (`load_bookings` mit SalesPerson-Cross-Resolve) | exact |
| `shifty-dioxus/src/page/absences.rs` (NEW) | page | request-response (UI orchestration) | `shifty-dioxus/src/page/employee_details.rs` (213 LOC) + `extra_hours_modal.rs` (597 LOC) | exact (Auth-Gate + Coroutine + Inner Modal) |
| `shifty-dioxus/src/page/mod.rs` (modify) | mod-decl | n/a | existing block (lines 1–32) | exact |
| `shifty-dioxus/src/component/absence_modal.rs` (NEW; falls Plan-Phase ihn herauszieht) | component | event-driven | `shifty-dioxus/src/component/extra_hours_modal.rs` (597 LOC) | exact |
| `shifty-dioxus/src/router.rs` (modify) | route | n/a | existing `Route`-Variant-Block (lines 19–51) | exact |
| `shifty-dioxus/src/component/top_bar.rs` (modify) | menu-wiring | n/a | `NavVisibility` Z. 21–30, `nav_visibility` Z. 32–45, `NavTarget` Z. 47–57, `is_active_for` Z. 59–82, `nav_items` Z. 312–371 | exact |
| `shifty-dioxus/src/i18n/mod.rs` (modify) | i18n-key-decl | n/a | existing `Key`-Enum-Block (line 55+, Comment-Sektionen `// Top bar`, `// Shiftplan` etc.) | exact |
| `shifty-dioxus/src/i18n/{en,de,cs}.rs` (modify) | i18n-translation | n/a | `i18n/de.rs:1–25` (`add_text(Locale::De, Key::*, "...")`) | exact |
| `shifty-dioxus/src/error.rs` (modify) | error-variant | n/a | existing `ShiftyError`-enum (Z. 4–16); add `Validation(String)` analog `Conflict(String)` | role-match |
| `shifty-dioxus/src/app.rs` (modify) | coroutine-registration | n/a | `app.rs:13–26` (`use_coroutine(...)` Block) | exact |
| `shifty-dioxus/Dioxus.toml` (modify) | proxy-config | n/a | existing `[[web.proxy]]` Einträge (Z. 45–92) | exact |

---

## Pattern Assignments

### `service/src/vacation_balance.rs` (service-trait, request-response)

**Analog:** `service/src/absence.rs` (Trait + Domain-Modell + automock)

**Imports + automock-Trait pattern** (lines 14–24, 136–216):
```rust
use std::sync::Arc;
use async_trait::async_trait;
use mockall::automock;
use time::Date;
use uuid::Uuid;
use crate::{permission::Authentication, ServiceError};

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait AbsenceService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// HR only — full visibility (D-10 Option A).
    async fn find_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError>;

    /// HR ∨ verify_user_is_sales_person(sales_person_id)
    async fn find_by_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError>;
    // ...
}
```

**Apply to VacationBalanceService:** gleiche Trait-Form, gleiche `type Context` + `type Transaction` + `Authentication<Self::Context>`-Signatur. Methoden: `get(sales_person_id, year, ctx, tx) -> VacationBalance` (HR ∨ self), `get_team(year, ctx, tx) -> Arc<[VacationBalance]>` (HR only). Rückgabe-Struct `VacationBalance` analog `AbsencePeriod`-Domain-Struct (lines 53–86 dort): `pub struct VacationBalance { sales_person_id, year, entitled_days, carryover_days, used_days, planned_days, remaining_days }`.

---

### `service_impl/src/vacation_balance.rs` (service-impl, BL-Tier)

**Analog:** `service_impl/src/absence.rs` (`gen_service_impl!`-Block + Methoden-Pattern)

**`gen_service_impl!`-Macro pattern** (lines 45–63):
```rust
gen_service_impl! {
    struct AbsenceServiceImpl: AbsenceService = AbsenceServiceDeps {
        AbsenceDao: AbsenceDao<Transaction = Self::Transaction> = absence_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        SpecialDayService: SpecialDayService<Context = Self::Context> = special_day_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Context = Self::Context, Transaction = Self::Transaction> = employee_work_details_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
        BookingService: BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        SalesPersonUnavailableService: SalesPersonUnavailableService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_unavailable_service,
        SlotService: SlotService<Context = Self::Context, Transaction = Self::Transaction> = slot_service,
    }
}
```

**Permission-Pattern HR ∨ self** (lines 110–119):
```rust
let (hr, sp) = join!(
    self.permission_service
        .check_permission(HR_PRIVILEGE, context.clone()),
    self.sales_person_service.verify_user_is_sales_person(
        sales_person_id,
        context,
        tx.clone().into()
    ),
);
hr.or(sp)?;
```

**Transaction-Pattern** (lines 88–101):
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
self.permission_service.check_permission(HR_PRIVILEGE, context).await?;
let entities = self.absence_dao.find_all(tx.clone()).await?;
let result: Arc<[AbsencePeriod]> = entities.iter().map(AbsencePeriod::from).collect();
self.transaction_dao.commit(tx).await?;
Ok(result)
```

**Apply to VacationBalanceServiceImpl:** Dep-Set (BL-Tier per D-04, RESEARCH.md Pattern 5):

```rust
gen_service_impl! {
    struct VacationBalanceServiceImpl: VacationBalanceService = VacationBalanceServiceDeps {
        AbsenceService: AbsenceService<Context=Self::Context, Transaction=Self::Transaction> = absence_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Context=Self::Context, Transaction=Self::Transaction> = working_hours_service,
        CarryoverService: CarryoverService<Context=Self::Context, Transaction=Self::Transaction> = carryover_service,
        SalesPersonService: SalesPersonService<Context=Self::Context, Transaction=Self::Transaction> = sales_person_service,
        PermissionService: PermissionService<Context=Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction=Self::Transaction> = transaction_dao,
    }
}
```

Each method: `use_transaction → join!(HR, verify_user_is_sales_person).or() → service-calls (absence_service.find_by_sales_person + working_hours.* + carryover.*) → arithmetic → commit → Ok(...)`. Plan-Phase OQ1 (`AbsenceService` direkt vs. `ReportingService.vacation_days`) entscheidet die innere Aggregation.

---

### `service_impl/src/test/vacation_balance.rs` (backend-test, mockall)

**Analog:** `service_impl/src/test/absence.rs` (1052 LOC, 30+ Tests)

**Imports + Mock-Setup** (lines 13–38):
```rust
use std::sync::Arc;
use dao::absence::{AbsenceCategoryEntity, AbsencePeriodEntity, MockAbsenceDao};
use dao::MockTransaction;
use dao::MockTransactionDao;
use mockall::predicate::{always, eq};
use service::absence::{AbsenceCategory, AbsencePeriod, AbsenceService};
use service::booking::MockBookingService;
use service::clock::MockClockService;
use service::employee_work_details::MockEmployeeWorkDetailsService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::sales_person::MockSalesPersonService;
use service::sales_person_unavailable::MockSalesPersonUnavailableService;
use service::slot::{MockSlotService, Slot};
use service::special_days::MockSpecialDayService;
use service::uuid_service::MockUuidService;
use service::{MockPermissionService, ServiceError, ValidationFailureItem};
use time::macros::{date, datetime};
use uuid::{uuid, Uuid};
use crate::absence::{AbsenceServiceDeps, AbsenceServiceImpl};
use crate::test::error_test::{
    test_conflicts, test_date_order_wrong, test_forbidden, test_not_found, test_validation_error,
};
```

**Test-Helper-Konstanten** (lines 40–66 — gleiches Pattern für `default_sales_person_id`, `default_version`, `*_active_entity`).

**Apply to VacationBalanceServiceImpl-Tests:** ~8 Tests (siehe RESEARCH.md Code Example 7):
- `get_returns_entitlement_minus_used_minus_planned` (Happy-Path, mit fixed `Carryover{vacation: 5}`, `WorkingHours{vacation_days_per_year: 25}`, 2 Vacation-Periods)
- `get_other_sales_person_without_hr_is_forbidden`
- `get_with_hr_succeeds`
- `get_team_without_hr_is_forbidden`
- `get_team_aggregates_per_paid_sales_person`
- `get_year_without_carryover_returns_zero` (Carryover-Path)
- `get_with_no_active_contract_returns_zero_entitlement`
- `get_with_only_planned_periods_includes_them_in_planned_days`

Jeder Test nutzt `MockTransaction`, `MockTransactionDao`, mockall-Stubs der Cross-Services. Erwartung: `expect_use_transaction().returning(|_| ...)`, `expect_check_permission().with(eq(HR_PRIVILEGE), always()).returning(|_,_| Err(ServiceError::Forbidden))` (für Self-Path). Reuse `test::error_test::test_forbidden(...)`-Helper. **Pflicht:** `service_impl/src/test/mod.rs` mit `pub mod vacation_balance;` ergänzen.

---

### `rest/src/vacation_balance.rs` (rest-handler, request-response)

**Analog:** `rest/src/absence.rs` (vollständige Vorlage, 260 LOC)

**Router + utoipa-path Pattern** (lines 30–77):
```rust
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", post(create_absence_period::<RestState>))
        .route("/", get(get_all_absence_periods::<RestState>))
        .route("/{id}", get(get_absence_period::<RestState>))
        // ...
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    post,
    path = "",
    tags = ["Absence"],
    request_body = AbsencePeriodTO,
    responses(
        (status = 201, description = "Absence period created (with warnings if any)", body = AbsencePeriodCreateResultTO),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "Validation error"),
    ),
)]
pub async fn create_absence_period<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Json(body): Json<AbsencePeriodTO>,
) -> Response {
    error_handler(
        (async {
            let svc = rest_state.absence_service();
            let result = svc.create(&(&body).into(), context.into(), None).await?;
            let to = AbsencePeriodCreateResultTO::from(&result);
            Ok(Response::builder()
                .status(201)
                .header("Content-Type", "application/json")
                .body(Body::new(serde_json::to_string(&to).unwrap()))
                .unwrap())
        })
        .await,
    )
}
```

**OpenApi-Doc-Block** (lines 240–261):
```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        create_absence_period,
        get_all_absence_periods,
        // ...
    ),
    components(schemas(
        AbsencePeriodTO,
        AbsenceCategoryTO,
        AbsencePeriodCreateResultTO,
        WarningTO,
    )),
    tags(
        (name = "Absence", description = "Absence period management (range-based)"),
    ),
)]
pub struct AbsenceApiDoc;
```

**Apply to rest/src/vacation_balance.rs:** Routes `GET /{sales_person_id}/{year}` (Self ∨ HR) und `GET /team/{year}` (HR-only). Service-Aufruf via `rest_state.vacation_balance_service()` (neuer Getter im `RestStateDef`-Trait, lines 270–390 in `rest/src/lib.rs`). 200/403/404-Status; KEIN 422 nötig (read-only). Tag: `"VacationBalance"`. `VacationBalanceApiDoc` analog `AbsenceApiDoc`.

---

### `rest/src/lib.rs` (rest-wiring)

**Analog:** existing absence-Wiring an drei Stellen.

**Mod-decl** (line 3): `mod absence;` → ergänzen `mod vacation_balance;`.

**RestStateDef-Trait-Erweiterung** (lines 270–390): füge analog `absence_service()` (line 375):
```rust
type VacationBalanceService: service::vacation_balance::VacationBalanceService<Context = Context>
    + Send + Sync + 'static;
fn vacation_balance_service(&self) -> Arc<Self::VacationBalanceService>;
```

**ApiDoc nest-entry** (line 485): `(path = "/vacation-balance", api = vacation_balance::VacationBalanceApiDoc),`.

**Router nest** (line 567 ff): `.nest("/vacation-balance", vacation_balance::generate_route())`.

---

### `rest-types/src/lib.rs` (dto)

**Analog:** `AbsencePeriodTO` Block (lines 1565–1647)

**DTO-Struct + ToSchema + (de)serialize + service-impl-feature-gated From-Impls** (lines 1595–1647):
```rust
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct AbsencePeriodTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub category: AbsenceCategoryTO,
    #[schema(value_type = String, format = "date")]
    pub from_date: time::Date,
    #[schema(value_type = String, format = "date")]
    pub to_date: time::Date,
    #[serde(default)]
    pub description: Arc<str>,
    #[serde(default)]
    pub created: Option<time::PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}

#[cfg(feature = "service-impl")]
impl From<&service::absence::AbsencePeriod> for AbsencePeriodTO {
    fn from(a: &service::absence::AbsencePeriod) -> Self {
        Self { id: a.id, sales_person_id: a.sales_person_id, category: (&a.category).into(), /* ... */ }
    }
}
#[cfg(feature = "service-impl")]
impl From<&AbsencePeriodTO> for service::absence::AbsencePeriod { /* mirror */ }
```

**Apply to VacationBalanceTO:** struct `VacationBalanceTO { sales_person_id: Uuid, year: u32, entitled_days: f32, carryover_days: i32, used_days: f32, planned_days: f32, remaining_days: f32 }` mit `#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, PartialEq)]`. KEIN `$version`-Field (read-only, kein Optimistic-Lock). `From<&service::vacation_balance::VacationBalance>`-Impl gegated mit `#[cfg(feature = "service-impl")]`. Falls Plan-Phase HR-Aggregat als Liste hinzunimmt (per `get_team`): `pub type VacationTeamRowTO = VacationBalanceTO;` reicht (gleiche Felder), oder dünnerer `VacationTeamRowTO`-Wrapper falls SalesPerson-Name beigelegt werden soll.

⚠ **Pitfall 1 (rest-types Cross-Crate):** Frontend zieht `rest-types` mit `default-features = false` (Cargo.toml line 28–30) — `service-impl`-Feature DARF NICHT aktiviert werden. Alle From-Impls auf `#[cfg(feature = "service-impl")]` setzen, sonst bricht WASM-Build.

---

### `shifty_bin/src/main.rs` (di-wiring)

**Analog:** `AbsenceServiceDependencies` impl (lines 228–255) + Konstruktor-Position (lines 798–815)

**Deps-Struct + impl** (lines 228–252):
```rust
pub struct AbsenceServiceDependencies;
impl service_impl::absence::AbsenceServiceDeps for AbsenceServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type AbsenceDao = AbsenceDao;
    type PermissionService = PermissionService;
    type SalesPersonService = SalesPersonService;
    // ... weitere Deps
    type BookingService = BookingService;
    type SalesPersonUnavailableService = SalesPersonUnavailableService;
    type SlotService = SlotService;
}
type AbsenceService = service_impl::absence::AbsenceServiceImpl<AbsenceServiceDependencies>;
```

**Konstruktor (BL-Tier nach Basic-Services)** (lines 788–815):
```rust
let working_hours_service = Arc::new(EmployeeWorkDetailsServiceImpl { /* ... */ });
let absence_service = Arc::new(AbsenceServiceImpl {
    absence_dao: absence_dao.clone(),
    permission_service: permission_service.clone(),
    sales_person_service: sales_person_service.clone(),
    // ...
    booking_service: booking_service.clone(),
    sales_person_unavailable_service: sales_person_unavailable_service.clone(),
    slot_service: slot_service.clone(),
});
```

**Apply to VacationBalanceService-Wiring:** drei Edits in `shifty_bin/src/main.rs`:

1. **Deps-Struct + impl** nach Z. ~290 (analog `AbsenceServiceDependencies`):
   ```rust
   pub struct VacationBalanceServiceDependencies;
   impl service_impl::vacation_balance::VacationBalanceServiceDeps for VacationBalanceServiceDependencies {
       type Context = Context;
       type Transaction = Transaction;
       type AbsenceService = AbsenceService;
       type EmployeeWorkDetailsService = WorkingHoursService;
       type CarryoverService = CarryoverService;
       type SalesPersonService = SalesPersonService;
       type PermissionService = PermissionService;
       type ClockService = ClockService;
       type TransactionDao = TransactionDao;
   }
   type VacationBalanceService = service_impl::vacation_balance::VacationBalanceServiceImpl<VacationBalanceServiceDependencies>;
   ```

2. **Konstruktor** NACH `absence_service` (Z. 798), `working_hours_service` (Z. 788), `carryover_service` (Z. 843) — d. h. **nach Z. 843**, sonst sind die Deps nicht im Scope. **Pitfall 3:** falsche Reihenfolge → "value cannot be constructed" Compiler-Error.

3. **`RestStateImpl`-Field + Getter:** struct-Field `vacation_balance_service: Arc<VacationBalanceService>` und `fn vacation_balance_service(&self) -> Arc<Self::VacationBalanceService> { self.vacation_balance_service.clone() }` (Pattern: `absence_service` field + getter sind im weiteren `RestStateImpl`-Block oberhalb Z. 680).

⚠ **Pitfall:** OpenAPI-Snapshot (per insta) — neuer REST-Endpoint ändert OpenAPI-Surface; Plan-Phase muss Snapshot-Refresh dokumentieren (siehe RESEARCH.md Open Question 5).

---

### `shifty-dioxus/src/state/absence_period.rs` (state)

**Analog:** `state/employee.rs` (struct-with-From-Impl-Pattern)

**Struct-mit-From-TO Pattern** (vgl. `state/employee.rs:170+`, hier paraphrased aus `From<&EmployeeReportTO>`):

```rust
use rest_types::AbsencePeriodTO;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbsencePeriod {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub category: AbsenceCategory,
    pub from_date: time::Date,
    pub to_date: time::Date,
    pub description: Arc<str>,
    pub version: Uuid,
    // Side-join Felder (loader.rs füllt sie):
    pub person_name: Arc<str>,
    pub background_color: Arc<str>,
}

impl From<&AbsencePeriodTO> for AbsencePeriod {
    fn from(t: &AbsencePeriodTO) -> Self {
        Self {
            id: t.id,
            sales_person_id: t.sales_person_id,
            category: (&t.category).into(),
            from_date: t.from_date,
            to_date: t.to_date,
            description: t.description.clone(),
            version: t.version,
            person_name: "".into(),         // filled by loader
            background_color: "".into(),    // filled by loader
        }
    }
}
```

**AbsenceCategory enum** mit `From<&AbsenceCategoryTO>` (siehe `rest-types/src/lib.rs:1567` und `service/src/absence.rs:34–51` für Match-Arme).

**Apply to state/vacation_balance.rs:** `pub struct VacationBalance { sales_person_id, year, entitled_days, carryover_days, used_days, planned_days, remaining_days }` plus `From<&VacationBalanceTO>`. KEINE Side-Join-Felder nötig — Frontend resolved den Person-Namen separat über `SalesPerson`-Liste in der HR-Variante.

---

### `shifty-dioxus/src/service/absence.rs` (service-coroutine, event-driven)

**Analog:** `shifty-dioxus/src/service/employee.rs` (361 LOC — vollständiges Vorbild)

**GlobalSignal-Store + Refresh-Token-Pattern** (lines 28–72):
```rust
#[derive(Clone, PartialEq)]
pub struct EmployeeStore { /* fields */ }

pub static EMPLOYEES_LIST_REFRESH: GlobalSignal<u64> = Signal::global(|| 0);
pub static EMPLOYEE_STORE: GlobalSignal<EmployeeStore> = Signal::global(|| EmployeeStore { /* default */ });

pub(crate) fn bump_employees_list_refresh() {
    *EMPLOYEES_LIST_REFRESH.write() += 1;
}
```

**Action-Enum** (lines 74–86):
```rust
#[derive(Debug)]
pub enum EmployeeAction {
    LoadEmployeeDataUntilNow { sales_person_id: Uuid },
    LoadCurrentEmployeeDataUntilNow,
    Refresh,
    DeleteExtraHours(Uuid),
    UpdateExtraHours(ExtraHoursTO),
    // ...
}
```

**Coroutine-Service mit 409-Branch-Handling** (lines 191–276):
```rust
pub async fn employee_service(mut rx: UnboundedReceiver<EmployeeAction>) {
    while let Some(action) = rx.next().await {
        info!("EmployeeAction: {:?}", &action);
        match match action {
            EmployeeAction::UpdateExtraHours(extra_hours) => {
                match update_extra_hours(extra_hours).await {
                    Ok(()) => refresh_employee_data().await,
                    Err(ShiftyError::Conflict(_)) => {
                        let message = I18N.read().t(Key::ExtraHoursConflictNotice).as_ref().to_string();
                        let refresh_result = refresh_employee_data().await;
                        *ERROR_STORE.write() = ErrorStore {
                            error: Some(ShiftyError::Conflict(message)),
                        };
                        refresh_result
                    }
                    Err(other) => Err(other),
                }
            }
            // ...
        } {
            Ok(_) => {}
            Err(err) => {
                *ERROR_STORE.write() = ErrorStore { error: Some(err.into()) };
            }
        }
    }
}
```

**Apply to service/absence.rs:** `ABSENCE_STORE: GlobalSignal<Rc<[AbsencePeriod]>>`, `ABSENCE_REFRESH: GlobalSignal<u64>`, `bump_absence_refresh()`. `AbsenceAction { LoadAll, LoadForSalesPerson(Uuid), Create(AbsencePeriodTO), Update(AbsencePeriodTO), Delete(Uuid), Refresh }`.

⚠ **Abweichung von Pattern (D-08):** 409 wird NICHT global in `ERROR_STORE` geschrieben (anders als `update_extra_hours`-Branch oben). Stattdessen:
- 409 bleibt als `ShiftyError::Conflict` bis zur Page durchgereicht.
- Page hält Modal-lokalen Banner-State; bei `Conflict`-Receipt setzt sie das Banner-Signal.
- `ERROR_STORE` bleibt für andere Service-Actions reserviert.

Realistisch: Action-Enum trägt `EventHandler<Result<(), ShiftyError>>` als Side-Channel (analog `extra_hours_modal.rs:189–192` `svc.send(...) ; on_saved.call(())`-Pattern), oder zweiter `GlobalSignal<Option<AbsenceModalConflict>>` (Plan-Phase entscheidet).

⚠ **Pitfall 4 (AUTH-State während Loading):** Page-Mount-Effect MUSS `auth.loading_done` checken bevor `LoadAll` vs. `LoadForSalesPerson` dispatched wird; sonst passiert ein zusätzlicher API-Call beim Auth-Resolve.

---

### `shifty-dioxus/src/service/vacation_balance.rs` (service-coroutine, simpler)

**Analog:** `service/employee.rs` Pattern, aber nur read-only (`Load` + `Refresh`).

```rust
pub static VACATION_BALANCE_STORE: GlobalSignal<Option<VacationBalance>> = Signal::global(|| None);
pub static VACATION_TEAM_STORE: GlobalSignal<Rc<[VacationBalance]>> = Signal::global(|| Rc::new([]));

#[derive(Debug)]
pub enum VacationBalanceAction {
    LoadSelf(Uuid, u32),       // sales_person_id, year
    LoadTeam(u32),             // year
    Refresh,
}
```

Coroutine ruft `loader::load_vacation_balance` bzw. `loader::load_team_vacation`. Refresh hört nicht auf eigenen Signal — wird stattdessen aus `ABSENCE_REFRESH`-Bump im Page-Effect getriggert (UI-SPEC § "Refresh-Flow"). Plan-Phase OQ2 entscheidet TTL vs. ABSENCE_REFRESH-piggyback.

---

### `shifty-dioxus/src/api.rs` (api-client, request-response + 409 + 422)

**Analog:** `api.rs:392–468` (extra_hours-CRUD-Flow)

**POST mit 422-Inline-Anzeige (NEU)** (extrapoliert aus 409-Pattern lines 452–468):
```rust
pub async fn create_absence_period(
    config: Config, body: AbsencePeriodTO,
) -> Result<AbsencePeriodCreateResultTO, ShiftyError> {
    let url = format!("{}/absence-period", config.backend);
    let client = reqwest::Client::new();
    let response = client.post(url).json(&body).send().await?;
    if response.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
        let text = response.text().await.unwrap_or_default();
        return Err(ShiftyError::Validation(text));
    }
    response.error_for_status_ref()?;
    let result: AbsencePeriodCreateResultTO = response.json().await?;
    Ok(result)
}
```

**PUT mit 409 + 422** (analog `update_extra_hour` lines 452–468):
```rust
pub async fn update_absence_period(
    config: Config, id: Uuid, body: AbsencePeriodTO,
) -> Result<AbsencePeriodCreateResultTO, ShiftyError> {
    let url = format!("{}/absence-period/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.put(url).json(&body).send().await?;
    if response.status() == reqwest::StatusCode::CONFLICT {
        return Err(ShiftyError::Conflict(String::new()));
    }
    if response.status() == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
        let text = response.text().await.unwrap_or_default();
        return Err(ShiftyError::Validation(text));
    }
    response.error_for_status_ref()?;
    Ok(response.json().await?)
}
```

**GET-list (analog `get_extra_hours_for_year` lines 424–440):**
```rust
pub async fn list_absence_periods(config: Config) -> Result<Rc<[AbsencePeriodTO]>, reqwest::Error> {
    let url = format!("{}/absence-period", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    Ok(response.json().await?)
}
```

**DELETE (analog `delete_extra_hour` lines 442–450):**
```rust
pub async fn delete_absence_period(config: Config, id: Uuid) -> Result<(), reqwest::Error> {
    let url = format!("{}/absence-period/{}", config.backend, id);
    let client = reqwest::Client::new();
    let response = client.delete(url).send().await?;
    response.error_for_status_ref()?;
    Ok(())
}
```

**ID + Version-Setting bei POST (analog `add_extra_hour` lines 392–422):**
```rust
let body = AbsencePeriodTO {
    id: Uuid::nil(),       // ⚠ Pitfall: !is_nil() → 422 IdSetOnCreate
    version: Uuid::nil(),  // ⚠ Pitfall: !is_nil() → 422 VersionSetOnCreate
    // ...
};
```

⚠ **Pitfall 9 (Anti-Patterns to Avoid):** `AbsencePeriodTO.id` MUSS `Uuid::nil()` bei POST sein, sonst Backend liefert `422 IdSetOnCreate`. Frontend NIE den Server-vergebenen UUID auf der Create-Seite setzen.

**Funktionen-Liste:** `list_absence_periods`, `list_absence_periods_by_sales_person(sales_person_id)`, `get_absence_period(id)`, `create_absence_period(body)`, `update_absence_period(id, body)`, `delete_absence_period(id)`, `get_vacation_balance(sales_person_id, year)`, `get_team_vacation_balance(year)`.

---

### `shifty-dioxus/src/loader.rs` (loader, transform mit side-joins)

**Analog:** `loader.rs:76–102` (`load_bookings` mit SalesPerson-Cross-Resolve)

**Cross-Source-Loader Pattern**:
```rust
pub async fn load_bookings(
    config: Config,
    sales_persons: Rc<[SalesPerson]>,
    week: u8, year: u32,
) -> Result<Rc<[Booking]>, ShiftyError> {
    let booking_tos = api::get_bookings_for_week(config, week, year).await?;
    let bookings: Rc<[Booking]> = booking_tos
        .iter()
        .map(|booking_to| booking_to.into())
        .map(|booking: Booking| {
            let sales_person = sales_persons
                .iter()
                .find(|sales_person| sales_person.id == booking.sales_person_id);
            if let Some(sales_person) = sales_person {
                Booking {
                    label: sales_person.name.clone(),
                    background_color: sales_person.background_color.clone(),
                    ..booking
                }
            } else {
                booking
            }
        })
        .collect();
    Ok(bookings)
}
```

**Apply to `load_absence_periods`:** identische Form — `api::list_absence_periods()` → map TO→state → join mit `sales_persons: Rc<[SalesPerson]>` für `person_name` + `background_color`. Für Self-Variante reicht `api::list_absence_periods_by_sales_person(sales_person_id)` ohne Cross-Resolve. `load_vacation_balance` und `load_team_vacation` rufen die neuen API-Funktionen direkt + state-mapping (kein Join nötig wenn `VacationBalanceTO.sales_person_id` direkt mit der `SalesPerson`-Liste auf der Page-Seite kombiniert wird).

---

### `shifty-dioxus/src/page/absences.rs` (page, request-response)

**Analog:** `page/employee_details.rs` (213 LOC — Auth + Coroutine + Inner Modal-State) und `component/extra_hours_modal.rs` (597 LOC — Form-Atom-Composition + Cross-Field-Validation)

**Auth-Gate + Coroutine-Konsum** (employee_details.rs:39–105):
```rust
#[component]
pub fn EmployeeDetails(props: EmployeeDetailsProps) -> Element {
    let employee_id = match Uuid::parse_str(&props.employee_id) { /* ... */ };
    let mut show_extra_hours_dialog = use_signal(|| false);
    let mut editing_extra_hours = use_signal(|| None::<ExtraHours>);

    let employee_service = use_coroutine_handle::<EmployeeAction>();
    let i18n = I18N.read().clone();

    // Local Action-Enum + use_coroutine for page-internal flows:
    let cr = use_coroutine(move |mut rx: UnboundedReceiver<EmployeeDetailsAction>| async move {
        while let Some(action) = rx.next().await {
            match action {
                EmployeeDetailsAction::OpenExtraHours => {
                    editing_extra_hours.set(None);
                    show_extra_hours_dialog.set(true);
                }
                EmployeeDetailsAction::ExtraHoursSaved => {
                    show_extra_hours_dialog.set(false);
                    employee_service.send(EmployeeAction::Refresh);
                }
                // ...
            }
        }
    });
    // ...
    rsx! { TopBar {} ErrorView {} /* Modal */ /* Page-Content */ }
}
```

**Modal-Form-Atoms-Composition** (extra_hours_modal.rs:78–200):
```rust
let mut category = use_signal(|| init_category.clone());
let mut amount = use_signal(|| init_amount);
let mut description = use_signal(|| init_description.clone());
let mut from = use_signal(|| /* date */);
let mut to = use_signal(|| /* date */);

// Re-seed-Pattern beim Edit-Mode-Switch:
let editing_key = props.editing.as_ref().map(|e| e.id);
let mut last_editing_key = use_signal(|| editing_key);
if *last_editing_key.peek() != editing_key {
    last_editing_key.set(editing_key);
    category.set(init_category.clone());
    /* ... */
}

// SSR-test-fähigkeit per try_consume_context:
let employee_service = try_consume_context::<Coroutine<EmployeeAction>>();
```

**Form-Atoms-Composition (Field + TextInput input_type="date" + SelectInput + TextareaInput + Btn-Variants)**:
- `Field { label, span: Some(2), error: Option<ImStr>, children: ... }` (form/field.rs:13–56)
- `TextInput { value, on_change, input_type: ImStr::from("date"), disabled }` (form/inputs.rs:14–53)
- `SelectInput { children: rsx!{ option { ... } }, on_change }` (form/inputs.rs:55–94)
- `TextareaInput { value, on_change, rows: 3 }` (form/inputs.rs:96–138)
- `Btn { variant: BtnVariant::Primary, on_click, children: rsx!{ "{submit_str}" } }` (atoms/btn.rs)

**Status-Berechnung Pure-Function (Pitfall 8)**:
```rust
#[cfg(target_arch = "wasm32")]
fn current_date_for_init() -> time::Date { js::current_date() }

#[cfg(not(target_arch = "wasm32"))]
fn current_date_for_init() -> time::Date {
    use time::macros::date;
    date!(2026-05-08)
}

pub fn compute_status(from: time::Date, to: time::Date, today: time::Date) -> AbsenceStatus {
    if to < today { AbsenceStatus::Finished }
    else if from > today { AbsenceStatus::Planned }
    else { AbsenceStatus::Active }
}
```
(Pattern aus `extra_hours_modal.rs:50–58` `current_datetime_for_init` mit `#[cfg]`-Branch — testbar mit fixed dates.)

**Tailwind-Kategorie-Klassen Pitfall 5 (statische match-Arme)**:
```rust
let (text, bg) = match category {
    AbsenceCategory::Vacation    => ("text-good",      "bg-good-soft"),
    AbsenceCategory::SickLeave   => ("text-warn",      "bg-warn-soft"),
    AbsenceCategory::UnpaidLeave => ("text-ink-muted", "bg-surface-2"),
};
// NIE: format!("text-{}", category)  ← Tailwind purge-Killer
```

---

### `shifty-dioxus/src/router.rs` (route)

**Analog:** existing `Route`-Variant-Block (lines 19–51)

**Edit-Locations:**
1. **`pub use`-Block** (lines 1–17): `pub use crate::page::AbsencesPage;`
2. **`Route`-enum-Variant** (innerhalb lines 19–51):
```rust
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    // ...
    #[route("/absences/")]
    Absences {},      // NEU — Trailing-Slash-Konvention der anderen Routes folgen
    // ...
}
```

---

### `shifty-dioxus/src/component/top_bar.rs` (menu-wiring)

**Analog:** Fünf separate Edits in `top_bar.rs` — alle Stellen explizit benannt.

**1) NavVisibility-Field** (lines 21–30):
```rust
pub(crate) struct NavVisibility {
    pub shiftplan: bool,
    pub my_shifts: bool,
    pub my_time: bool,
    pub year_overview: bool,
    pub absences: bool,         // NEU (D-10: für alle Eingeloggten)
    pub employees: bool,
    pub billing_periods: bool,
    pub user_management: bool,
    pub templates: bool,
}
```

**2) `nav_visibility` Builder** (lines 32–45):
```rust
pub(crate) fn nav_visibility(auth_info: Option<&AuthInfo>, is_paid: bool) -> NavVisibility {
    let has = |p: &str| auth_info.map(|a| a.has_privilege(p)).unwrap_or(false);
    let show_reports = has("hr");
    let logged_in = auth_info.is_some();
    NavVisibility {
        shiftplan: has("sales") || has("shiftplanner"),
        // ...
        absences: logged_in,    // D-10: für alle eingeloggten User sichtbar
        // ...
    }
}
```

**3) `NavTarget`-Variant** (lines 47–57): füge `Absences,` ein.

**4) `is_active_for` match-Arm** (lines 59–82): füge `NavTarget::Absences => matches!(route, Route::Absences {}),` ein.

**5) `nav_items` push** (lines 312–371):
```rust
if visibility.absences {
    items.push((
        NavTarget::Absences,
        Route::Absences {},
        i18n.t(Key::AbsenceMenuLabel).to_string(),
    ));
}
```
(Position: nach `my_time`/`year_overview`-Block, vor `employees` — die Reihenfolge folgt der UI-SPEC Reihenfolge im Mockup.)

⚠ **Hinweis:** `partition_nav_items` (Z. 386) entscheidet, welche Items in der Top-Level-Bar vs. im Admin-Dropdown landen. `Absences` soll Top-Level sein → KEINE Eintragung in `is_admin_target` (Z. 106+).

---

### `shifty-dioxus/src/i18n/mod.rs` (i18n-key-decl)

**Analog:** existing `Key`-Enum-Block (line 55+, Comment-Sektionen)

**Comment-Block-Pattern** (lines 68–96, beispielhaft):
```rust
pub enum Key {
    // ...
    // Top bar
    Shiftplan,
    Employees,
    MyTime,
    // ...
    // Shiftplan
    ShiftplanCalendarWeek,
    // ...
}
```

**Apply:** Comment-Block `// Absence management` mit ~50 neuen Variants per UI-SPEC Copywriting-Tabelle. Position: vor dem letzten Block (z. B. vor `// Working-hours mini overview`, line ~398).

**i18n-Test-Pattern** (mod.rs:425–457):
```rust
#[test]
fn i18n_employees_keys_present_in_all_locales() {
    for locale in [Locale::En, Locale::De, Locale::Cs] {
        let i18n = generate(locale);
        for key in [Key::SearchPlaceholder, Key::OtherHours, /* ... */] {
            let value = i18n.t(key);
            assert!(
                !value.is_empty() && value.as_ref() != "??",
                "missing translation for {:?} in {:?}: got `{}`",
                key, locale, value
            );
        }
    }
}
```
**Apply:** neuer Test `i18n_absence_keys_present_in_all_locales` mit allen ~50 neuen Keys; zusätzlicher Test `i18n_absence_keys_match_german_reference` mit ~5 Stichproben (z. B. `assert_eq!(de.t(Key::AbsencePageTitle).as_ref(), "Abwesenheiten");`).

---

### `shifty-dioxus/src/i18n/{en,de,cs}.rs` (i18n-translations)

**Analog:** `i18n/de.rs:1–25` (`add_text(Locale::De, Key::*, "...")` Pattern)

**Pattern**:
```rust
pub fn add_i18n_de(i18n: &mut I18n<Key, Locale>) {
    i18n.add_locale(Locale::De);
    i18n.add_text(Locale::De, Key::Home, "Start");
    // ...
    // Top bar
    i18n.add_text(Locale::De, Key::Shiftplan, "Schichtplan");
    // ...
}
```

**Apply:** in jedem der drei Files (en.rs, de.rs, cs.rs) Comment-Block `// Absence management (Phase 8)` mit `add_text(Locale::*, Key::Foo, "...")` für alle ~50 Keys. Texte 1:1 aus UI-SPEC Copywriting-Tabelle (sortiert nach Use-Site: Page-Level → Primary CTA → Empty State → Form Labels → Categories → Status → Liste → VacationCard → StatsCards → Errors/Warnings → Destructive → Filter).

⚠ **Pitfall 2 (Locale::En-statt-Locale::De):** `de.rs` MUSS `Locale::De` als ersten Parameter haben, NICHT `Locale::En`. Editor-Suche `Locale::De` als Pre-Commit-Check; gleichermaßen `Locale::Cs` in cs.rs. Historischer Bug (CLAUDE.md `shifty-dioxus/CLAUDE.md`).

---

### `shifty-dioxus/src/error.rs` (error-variant)

**Analog:** existing `ShiftyError` enum (lines 4–16)

```rust
#[derive(Error, Debug)]
pub enum ShiftyError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Time ComponentRange error: {0}")]
    TimeComponentRange(#[from] time::error::ComponentRange),

    /// HTTP 409 Conflict — typically optimistic-lock failure on a versioned PUT.
    #[error("{0}")]
    Conflict(String),

    /// HTTP 422 Validation error — e.g. self-overlap of absence periods.    // NEU
    #[error("{0}")]
    Validation(String),
}
```

**Plus** in `error_handler` (lines 18–33) ein `ShiftyError::Validation(msg) => eprintln!("Validation: {}", msg),`-Arm.

OQ3 (RESEARCH.md): Plan-Phase entscheidet, ob neuer Variant oder Multiplex über `Conflict`. Empfehlung Research: neuer Variant.

---

### `shifty-dioxus/src/app.rs` (coroutine-registration)

**Analog:** `app.rs:13–26` (existing `use_coroutine(...)` Block)

```rust
pub fn App() -> Element {
    use_coroutine(service::config::config_service);
    use_coroutine(service::theme::theme_service);
    // ...
    use_coroutine(service::employee::employee_service);
    use_coroutine(service::slot_edit::slot_edit_service);
    use_coroutine(service::billing_period::billing_period_service);
    // NEU:
    use_coroutine(service::absence::absence_service);
    use_coroutine(service::vacation_balance::vacation_balance_service);
    // ...
}
```

---

### `shifty-dioxus/Dioxus.toml` (proxy-config)

**Analog:** existing `[[web.proxy]]`-Einträge (lines 45–92)

```toml
[[web.proxy]]
backend = "http://localhost:3000/extra-hours"
[[web.proxy]]
backend = "http://localhost:3000/working-hours"
```

**Apply:** zwei neue Einträge:
```toml
[[web.proxy]]
backend = "http://localhost:3000/absence-period"
[[web.proxy]]
backend = "http://localhost:3000/vacation-balance"
```

⚠ **Pitfall 10:** Ohne diese beiden Einträge liefert dx-Dev-Server (Port 8080) 404 für die neuen API-Calls.

---

## Shared Patterns

### Permission HR ∨ self (Backend Cross-File)

**Source:** `service_impl/src/absence.rs:110–119`
**Apply to:** `service_impl/src/vacation_balance.rs::get` (Self-Path), `find_by_sales_person`-Methoden allgemein.
```rust
let (hr, sp) = join!(
    self.permission_service
        .check_permission(HR_PRIVILEGE, context.clone()),
    self.sales_person_service.verify_user_is_sales_person(
        sales_person_id,
        context,
        tx.clone().into()
    ),
);
hr.or(sp)?;
```

### Transaction-Rahmen (Backend Cross-Service)

**Source:** `service_impl/src/absence.rs:88–101`
**Apply to:** Jede VacationBalanceService-Methode.
```rust
let tx = self.transaction_dao.use_transaction(tx).await?;
// ... permission checks + service-calls ...
self.transaction_dao.commit(tx).await?;
Ok(result)
```

### error_handler-Wrapper (Backend REST)

**Source:** `rest/src/lib.rs:122–267` und `rest/src/absence.rs:60–76`
**Apply to:** Alle vacation_balance-Handler.
```rust
error_handler(
    (async {
        let svc = rest_state.vacation_balance_service();
        let result = svc.get(sales_person_id, year, context.into(), None).await?;
        let to = VacationBalanceTO::from(&result);
        Ok(Response::builder().status(200).header(...).body(...).unwrap())
    }).await,
)
```
ServiceError-Mapping (Forbidden→403, EntityNotFound→404, EntityConflicts→409, ValidationError→422) ist bereits im `error_handler` zentralisiert — keine Per-Handler-Logik nötig.

### `gen_service_impl!` + Tier-Konvention (Backend)

**Source:** `service_impl/src/absence.rs:45–63`
**Apply to:** `VacationBalanceServiceImpl`. Service-Tier-Klassifizierung BL (D-04) — KEIN Basic-Tier (würde Cycle mit AbsenceService produzieren, weil AbsenceService selbst BL ist und VacationBalance es konsumiert).

### rest-types `service-impl` Feature-Gating (Cross-Crate)

**Source:** `rest-types/src/lib.rs:1574, 1616, 1632` (alle `#[cfg(feature = "service-impl")]`-Blocks)
**Apply to:** Alle From-Impls auf `VacationBalanceTO`. Frontend zieht `default-features = false` (Cargo.toml lines 28–30); ohne Feature-Gate würde `service`-Crate in den WASM-Build gezogen → Toolchain-Bruch.

### Coroutine-Service-Pattern (Frontend)

**Source:** `shifty-dioxus/src/service/employee.rs:191–276`
**Apply to:** `service/absence.rs::absence_service` und `service/vacation_balance.rs::vacation_balance_service`.

Komponenten:
1. `pub static *_STORE: GlobalSignal<...>` für Lese-Cache.
2. `pub static *_REFRESH: GlobalSignal<u64>` für Cross-Page-Bump.
3. `pub enum *Action { ... }` für Action-Typen.
4. `pub async fn *_service(mut rx: UnboundedReceiver<*Action>)` mit `while let Some(action) = rx.next().await { match action { ... } }`.
5. `pub(crate) fn bump_*_refresh()`-Helper (intern).
6. Coroutine-Registration in `app.rs::App`.

### Form-Atoms-Composition (Frontend Modal)

**Source:** `shifty-dioxus/src/component/extra_hours_modal.rs:78–250`
**Apply to:** AbsenceModal in `page/absences.rs` (oder eigenes File).

Reuse-Reihenfolge:
- `Dialog { variant: DialogVariant::Center, width: 520, open, on_close, title, subtitle, ... }` (`dialog.rs:113–`)
- `Field { label, span: Some(2), hint, error, children }` (`form/field.rs:13–56`)
- `TextInput { value, on_change, input_type: ImStr::from("date") }` (`form/inputs.rs:14–53`)
- `SelectInput { children, on_change }` (`form/inputs.rs:55–94`)
- `TextareaInput { value, on_change, rows: 3 }` (`form/inputs.rs:96–138`)
- `Btn { variant: BtnVariant::{Primary,Ghost,Danger}, on_click, disabled, children }` (`atoms/btn.rs`)

Re-Seed-Pattern bei Edit-Mode-Switch (`extra_hours_modal.rs:140–148` — `last_editing_key`-Signal vergleicht peeked-id und reseedet alle Form-Signals).

`try_consume_context::<Coroutine<...>>()` (statt `use_coroutine_handle`) für SSR-Test-Tauglichkeit (`extra_hours_modal.rs:154`).

### i18n-Drei-Locales-Pattern (Frontend Cross-File)

**Source:** `i18n/mod.rs:411–415` (`generate(locale)` switch) und `i18n/de.rs:5–6` (`add_text(Locale::De, ...)`)
**Apply to:** alle ~50 neuen Keys, drei Files synchron befüllen, Plus i18n-Test in `i18n/mod.rs::tests`.

⚠ **Pitfall 2 lock-down:** Editor-Suche prüft `Locale::De` in de.rs vor Commit; gleichermaßen `Locale::Cs` in cs.rs. UI-SPEC § "Sign-Off" macht das zur Pflicht.

---

## No Analog Found

Keine Files in dieser Phase ohne Analog. Alle 24 betroffenen Pfade haben einen direkten Vorbild-Pfad in der bestehenden Codebase. Die Reife der Codebase (siehe RESEARCH.md "Don't Hand-Roll" Tabelle) ist hoch — Phase 8 ist Komposition, kein Net-New-Pattern-Build.

---

## Metadata

**Analog search scope:**
- Backend: `service/src/`, `service_impl/src/`, `service_impl/src/test/`, `rest/src/`, `rest-types/src/`, `shifty_bin/src/main.rs`
- Frontend: `shifty-dioxus/src/{api.rs, loader.rs, app.rs, error.rs, router.rs}`, `shifty-dioxus/src/{page,component,service,state,i18n}/`, `shifty-dioxus/Dioxus.toml`, `shifty-dioxus/Cargo.toml`

**Files scanned:** 19 (drei verifiziert vollständig: `service/absence.rs`, `service_impl/absence.rs` Header, `rest/absence.rs`; sechszehn weitere mit gezielten Reads inkl. Line-Range-Verifikation der angegebenen Excerpts)

**Pattern extraction date:** 2026-05-08

**Coverage by tier:**
- Backend exact-analog: 8/8
- Frontend exact-analog: 13/16 (drei role-match: `state/absence_period.rs`, `state/vacation_balance.rs`, `service/vacation_balance.rs` — gleicher Pattern-Typ, aber ohne 1:1-Vorbild gleichen Domain-Inhalts)
- Frontend role-match: 3/16

**Critical conventions to enforce in Plan-Phase:**
1. **rest-types `default-features = false`** im Frontend (Pitfall 1) — niemals entfernen.
2. **Locale::De/Cs richtig in de.rs/cs.rs** (Pitfall 2) — Pre-Commit-Editor-Suche.
3. **VacationBalanceService = BL-Tier** (Pitfall 3) — DI-Reihenfolge in `main.rs` NACH AbsenceService, WorkingHoursService, CarryoverService.
4. **AUTH `loading_done`-Check** (Pitfall 4) — Page-Render-Branch früh, sonst Flackern + Doppel-Fetch.
5. **Tailwind-Kategorie-Klassen statisch** (Pitfall 5) — `match`-Arme, NIE `format!`-Strings.
6. **`Uuid::nil()` bei POST-Body** (Anti-Pattern) — id + version, sonst 422.
7. **Dx-Proxy für `/absence-period` UND `/vacation-balance`** (Pitfall 10) — beide Einträge hinzufügen.
8. **`ShiftyError::Validation(String)` Variant** für 422-Inline (RESEARCH.md OQ3) — semantisch sauber gegen `Conflict`.
9. **OpenAPI-Snapshot** (insta) refresh nach Backend-Wave — Plan-Phase final-step `cargo insta accept`.
10. **jj statt git** (CLAUDE.local.md) — Executor MUSS jj-nativ committen, GSD `commit_docs: false`.
