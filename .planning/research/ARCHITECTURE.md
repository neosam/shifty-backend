# Architecture Research — v2.7 Personenverwaltung & Planungs-Soll

**Domain:** Rust layered backend (Axum → Service (2-tier) → DAO → SQLite), shared `rest-types` with a Dioxus/WASM frontend.
**Researched:** 2026-09-16
**Confidence:** HIGH (all findings grounded in code read this session; two items flagged MEDIUM/uncertain below)

## Standard Architecture (this repo, as it stands today)

```
┌───────────────────────────────────────────────────────────────────┐
│ REST (Axum) — rest/src/*.rs                                       │
│   sales_person.rs  user.rs  report.rs  booking_information.rs ... │
├───────────────────────────────────────────────────────────────────┤
│ Service — Business-Logic tier (service_impl/src/*.rs)             │
│   AbsenceService, ShiftplanViewService, ReportingService,         │
│   BookingInformationService, WorkingHoursService, ...             │
│   (may consume Basic + other Business-Logic services)             │
├───────────────────────────────────────────────────────────────────┤
│ Service — Basic tier (entity managers)                             │
│   SalesPersonService, SlotService, BookingService, ...            │
│   (DAO + PermissionService + TransactionDao ONLY)                 │
├───────────────────────────────────────────────────────────────────┤
│ DAO (dao trait + dao_impl_sqlite impl)                             │
│   SalesPersonDao, EmployeeWorkDetailsDao, ...                     │
├───────────────────────────────────────────────────────────────────┤
│ SQLite (migrations/sqlite/*.sql)                                   │
└───────────────────────────────────────────────────────────────────┘
```

DI is hand-wired in `shifty_bin/src/main.rs`: every `*ServiceDependencies` struct
declares associated types (`service_impl/src/main.rs:73-767` for the trait-impl
declarations), then `RestStateImpl::new()` (`shifty_bin/src/main.rs:997` onward)
constructs concrete `Arc<...>` instances **strictly bottom-up**: Basic services
first, Business-Logic services after, in dependency order, with no
`OnceLock`/forward-declaration tricks (`docs/architecture/02-service-tiers.md:92-93`).

---

## (a) Where does `PersonService` belong, and what is the DI story?

### Classification: Basic (entity manager)

Per the tier rule of thumb in `CLAUDE.md:152-154` and
`docs/architecture/02-service-tiers.md:94-96` ("count dependencies — only DAOs +
Permission + Transaction → Basic; as soon as another domain service appears as a
dep → Business-Logic"):

A `PersonService` that does CRUD on the new `person` table plus the
link-maintenance operations (create person, link an existing `sales_person`/`user`
to a person, look up a person by `sales_person_id` or by `user.name`) needs
**only**:
- a new `PersonDao` (DAO trait + `PersonDaoImpl`)
- `PermissionService`
- `ClockService` / `UuidService` (person needs a UUID id + audit columns, same
  pattern as `SalesPersonServiceDependencies` in `shifty_bin/src/main.rs:115-124`)
- `TransactionDao`

No other domain service is required for the identity-bundling operation itself —
"does this person's row exist, attach this sales_person_id to it, attach this
username to it" is pure CRUD over `person`, structurally identical to
`SalesPersonService` (`service/src/sales_person.rs:51-130`) or
`RebookingBatchService` (a recent Basic-tier precedent, see
`docs/architecture/02-service-tiers.md:18-23`, itself introduced specifically to
avoid a cycle with a same-milestone Business-Logic sibling).

**Verdict: `PersonService` is Basic tier.**

### Why NOT Business-Logic

The temptation is to make `PersonService` "the" aggregate that also assembles the
merged list/detail view (question c) — but that would immediately force it to
depend on `SalesPersonService` **and** `UserService`/`PermissionService`'s user
operations to enrich the DTO (roles, invitations, shiftplan assignment, balance).
That is exactly the shape of `ShiftplanViewService` or `BookingInformationService`
— a **Business-Logic read aggregate that CONSUMES `PersonService`**, not
`PersonService` itself. Keeping `PersonService` a thin Basic identity-manager and
building the aggregate one tier higher (see part c) avoids forcing a Business-Logic
shape onto what is structurally a CRUD table.

### Cyclic-coupling risk with `SalesPersonService` / `UserService` / `PermissionService`

**Real risk, and it needs an explicit design decision, not a default.**

- `SalesPersonService` currently owns the assignment operations `set_user`,
  `get_assigned_user`, `get_all_user_assignments`, `get_sales_person_for_user`,
  `get_sales_person_current_user`, `verify_user_is_sales_person`
  (`service/src/sales_person.rs:95-130`, implemented
  `service_impl/src/sales_person.rs:281-407`) — i.e. `SalesPersonService` **already
  knows about users** today (it takes `user_id: Arc<str>` params and queries
  `sales_person_user` via its DAO). If `PersonService` also needs to know about
  sales persons and users to do backfill/linking, and `SalesPersonService` keeps
  calling into `PersonService` (e.g. to resolve a person for `set_user`), you get
  exactly the two-Basic-services-need-each-other situation `CLAUDE.md:147-149`
  warns about.
- `PermissionService` depends on `UserService` today
  (`shifty_bin/src/main.rs:74-80`: `PermissionServiceDeps::UserService = UserService`).
  `PermissionService` is a dependency of **every** other service. `PersonService`
  must NOT become a dependency of `PermissionService` or of `UserService` under
  any circumstance — that would make it un-constructible (every Basic service
  needs `PermissionService`, so `PermissionService` cannot need something that
  needs it back).

**Recommended resolution (matches the "third service one tier higher" escape
hatch in `CLAUDE.md:148-149` / `docs/architecture/02-service-tiers.md:88-91`):**

1. `PersonService` (Basic) owns ONLY `person` CRUD: `create`, `get_by_id`,
   `get_by_sales_person_id`, `get_by_user_name`, `link_sales_person`,
   `link_user`. It does **not** depend on `SalesPersonService` or `UserService`.
2. `SalesPersonService` and the user-facing service keep their existing public
   methods (`set_user` et al.) **unchanged in signature** — see part (b): the
   dissolution of `sales_person_user` is a DAO-internal implementation swap
   (join through `person_id` instead of the old link table), not a service-API
   change. This means `SalesPersonService` does NOT need to start consuming
   `PersonService` to keep working.
3. If a merged read (list/detail, question c) needs to combine `person` +
   `sales_person` + `user` + roles + shiftplan assignment, that lives in a NEW
   Business-Logic service (e.g. `PersonDirectoryService` or similar) that
   consumes `PersonService` + `SalesPersonService` + `UserInvitationService` +
   `PermissionService`'s user/role read paths — one tier above all of them,
   exactly like `ShiftplanViewService` sits above `SlotService` + `BookingService`
   + `SalesPersonService` (`shifty_bin/src/main.rs:514-530`).

This gives: `PersonService` (Basic) ⟶ consumed by the new aggregate
(Business-Logic) ⟶ no cycle, because neither `SalesPersonService` nor
`UserService`/`PermissionService` need to know `PersonService` exists.

### DI construction order in `shifty_bin/src/main.rs`

Following the existing bottom-up pattern (Basic tier is built in one contiguous
block starting `shifty_bin/src/main.rs:1000`, e.g. `sales_person_dao` at
`:1008`, `sales_person_service` at `:1046-1052`):

1. `person_dao = Arc::new(PersonDao::new(pool.clone()))` — alongside the other
   `*_dao` constructions near `shifty_bin/src/main.rs:1000-1020`.
2. `person_service = Arc::new(PersonServiceImpl { person_dao, permission_service,
   clock_service, uuid_service, transaction_dao })` — placed in the Basic-tier
   block, near `sales_person_service` (`:1046`) since they are conceptually
   siblings, but with **no** `sales_person_service:` field.
3. If/when a `PersonDirectoryService` (Business-Logic) is added for the merged
   list/detail (part c), it is constructed **after** `person_service`,
   `sales_person_service`, and whichever user/role read-service it needs —
   mirroring where `ShiftplanViewService` is built (`:1385`, after
   `slot_service`, `booking_service`, `sales_person_service`,
   `special_day_service`, `shiftplan_service`).
4. `RestStateImpl` struct (`:770-832`) gets a new `person_service: Arc<PersonService>`
   field (and, if built, `person_directory_service`), plus the matching
   `RestStateDef` trait methods (pattern at `:833-996`).

No existing construction order needs to move — `PersonService` has no deps on
anything built later in `main()`, so it can be inserted anywhere in the Basic
block without perturbing the carefully-commented ordering already documented for
Phase 54/55/56 services.

### Docs-Freshness-Gate trigger

Adding `PersonService` touches `shifty_bin/src/main.rs` (DI wiring) → per
`CLAUDE.md:234` this **requires** an update to
`docs/architecture/02-service-tiers.md` and
`docs/architecture/diagrams/service-graph-runtime.mmd` in the same PR (both EN
and DE variants, `CLAUDE.md:242-243`). Flag this for the phase plan explicitly —
it is a hard gate, not a suggestion.

---

## (b) Blast radius of `sales_person_user` dissolution

Full enumeration, `grep -rn "sales_person_user"` across the backend (frontend
excluded from this table, listed separately below):

| Layer | File:Line | What it does today | Change needed |
|---|---|---|---|
| Migration (historical, do not edit) | `migrations/sqlite/20240506114107_add_sales_person.sql:14-22` | Defines `sales_person_user` with `UNIQUE(sales_person_id)` + `UNIQUE(user_id)` — the 1:1 link | New migration `DROP TABLE sales_person_user` after backfill; do not touch this file |
| DAO trait | `dao/src/sales_person.rs:33-37, 50-76` | `find_by_user`, `get_assigned_user`, `get_all_user_assignments`, `assign_to_user`, `discard_assigned_user`, `find_sales_person_by_user_id` — six trait methods whose contract is "resolve via the link table" | **Signatures can stay identical.** Only the SQL inside the impl changes (see next row) |
| DAO impl | `dao_impl_sqlite/src/sales_person.rs:102-118` (`find_by_user`), `:159-172` (`assign_to_user`), `:174-188` (`discard_assigned_user`), `:190-204` (`get_assigned_user`), `:206-224` (`get_all_user_assignments`), `:226-243` (`find_sales_person_by_user_id`) | Raw SQL joins/writes against `sales_person_user` | Rewrite each query to join/write through `sales_person.person_id` ⟷ `user.person_id` instead of the link table. `assign_to_user`/`discard_assigned_user` become "set/clear `sales_person.person_id`, creating a `person` row on first link if needed" — this is the one place with real logic change (join-table row ↔ two FK columns is not a 1:1 SQL rewrite, see risk note below) |
| DAO impl (bulk cleanup) | `dao_impl_sqlite/src/lib.rs:286` | `BasicDaoImpl::clear_all()` test-helper: `DELETE FROM sales_person_user;` | Remove this line (table gone); if `person` needs the same test-reset treatment, add `DELETE FROM person;` in its place |
| Service impl | `service_impl/src/sales_person.rs:281-407` | Implements the six `SalesPersonService` trait methods by delegating straight to the DAO methods above | **No change needed** if DAO signatures stay stable — this is the layer that insulates all 190+ call sites below from the schema change |
| Service trait | `service/src/sales_person.rs:95-129` | Public contract: `get_assigned_user`, `get_all_user_assignments`, `set_user`, `get_sales_person_for_user`, `get_sales_person_current_user`, `verify_user_is_sales_person` | No change needed (same reasoning) |
| REST routes | `rest/src/sales_person.rs:27-29` (route registration), `:247-266` (`GET /{id}/user`), `:287-303` (`POST /{id}/user`), `:320-335` (`DELETE /{id}/user`), `:350-365` (`GET /current`), `:592-598` (OpenAPI registration) | Thin handlers calling the service methods above | No change needed if the service contract is stable |

### The one place that is NOT a pure pass-through: `assign_to_user` / `discard_assigned_user`

`dao_impl_sqlite/src/sales_person.rs:159-188` currently INSERT/DELETE a row in
the join table. Once `sales_person_user` is gone, "assign a user to a sales
person" must instead:
1. Resolve (or create) the `person` row for the target `sales_person_id`
   (its `person_id` FK may already be set, may be null).
2. Resolve (or create) the `person` row for the target `user_id`.
3. If both already point at **different, non-null** persons, this is now an
   ambiguous merge case that the old `UNIQUE(sales_person_id)`/`UNIQUE(user_id)`
   constraints made structurally impossible on the link table — the new model
   must decide (reject with a validation error, most likely, since "this
   sales-person already has a person and so does this user, and they differ" is
   a genuine conflict, not something to silently merge).
4. Otherwise, point both `sales_person.person_id` and `user.person_id` at the
   same `person.id` (creating the `person` row if neither existed yet).

This is real business logic, not a mechanical SQL rewrite — **flag this for the
phase plan as the one sub-task within DAO-layer work that needs its own test
matrix** (link two unlinked, re-link an already-linked sales person to a
different user, discard, etc.).

### Widely-fanned-out but UNAFFECTED consumers

`grep -rn "verify_user_is_sales_person\|get_assigned_user\|get_sales_person_current_user\|get_all_user_assignments"` across `service_impl/` returns **~50+ call
sites** (`service_impl/src/absence.rs`, `extra_hours.rs`, `booking.rs`,
`shiftplan.rs`, `shiftplan_edit.rs`, `vacation_balance.rs`,
`sales_person_unavailable.rs`, `custom_extra_hours.rs`, `block.rs`,
`employee_work_details.rs`, `reporting.rs:1710`, plus ~40 test files). **None of
these need to change** — they all call the stable `SalesPersonService` trait
methods, which is precisely the point of keeping the service-layer contract
frozen while the DAO implementation swaps its storage. This is the single most
important architectural finding for scoping the phase: **the dissolution of
`sales_person_user` is a two-layer change (migration + DAO), not an N-layer
change**, provided the service contract is preserved.

### Frontend (context only — backend-authoritative, listed for completeness)

`shifty-dioxus/src/page/user_management.rs` and
`shifty-dioxus/src/service/user_management.rs` reference `sales_person_user`
only conceptually (comment `page/user_management.rs:8`) — they call the stable
REST endpoints, not the table. No change forced by the migration itself; any
frontend change here is driven by the merged-view UI work (part c), not by the
schema dissolution.

### What must NOT be touched (per milestone constraint, re-verified in code)

- `sales_person.id` (`migrations/sqlite/20240506114107_add_sales_person.sql:3-4`
  `id blob(16) NOT NULL PRIMARY KEY`) — confirmed referenced by
  `billing_period_sales_person` and eleven other tables per the seed
  (`.planning/seeds/personenverwaltung-zusammenlegen.md:89-95`); this research
  did not re-enumerate all twelve but did not find any code path that would
  require renumbering it. Not touched by this design.
- `user.name` as PK — confirmed as the FK target in the now-dissolving
  `sales_person_user.user_id → user(name)` constraint
  (`migrations/sqlite/20240506114107_add_sales_person.sql:19`), and used
  throughout `dao/src/permission.rs` (`UserEntity`, `find_user`, `roles_for_user`
  etc., `dao/src/permission.rs:25-55`) and `service_impl/src/lib.rs:59-62`
  (`UserServiceImpl`). Not touched by this design — `user.person_id` is an
  ADDITIVE nullable column, `user.name` stays PK.

---

## (c) Aggregate DTO for the merged entry list / detail — follow the v2.5 precedent

### The precedent, read in code

`WeeklySummaryTO.sales_person_absences: Arc<[SalesPersonAbsenceTO]>`
(`rest-types/src/lib.rs:1097-1103`) is the v2.5 "Fat Backend, Thin Client"
aggregate precedent named in the question. Structurally:
- `SalesPersonAbsenceTO` (`rest-types/src/lib.rs:1063-1068`) is a **small, flat,
  purpose-built DTO** — three fields (`sales_person_id`, `name`, `hours`), not a
  reuse of `SalesPersonTO` or `AbsencePeriodTO`.
- It is populated by a Business-Logic service
  (`BookingInformationService::get_weekly_summary` /
  `get_summery_for_week`, per `.planning/PROJECT.md:264-274`), which is exactly
  one tier above the Basic services it unions data from.
- The union/merge logic (bezahlte Mitarbeiter + Freiwillige mit aktiver
  Absence) lives entirely in the backend; the frontend does "reinen
  Union-Merge" (`.planning/PROJECT.md:276-281`) — no filtering/business rule in
  the client.
- Wire-compat is preserved with `#[serde(default)]` on the new field
  (`rest-types/src/lib.rs:1088-1089, 1101`) so old frontends don't break on the
  additive field — the same technique should be used for any new list/detail DTO
  field here.

`ShiftplanViewService` (`docs/architecture/02-service-tiers.md:44-45`, wired
`shifty_bin/src/main.rs:514-530`) is the second cited precedent: a
Business-Logic **read aggregate** over `SlotService` + `BookingService` +
`SalesPersonService` + `SpecialDayService` + `AbsenceService` +
`SalesPersonUnavailableService`, i.e. it reads from many Basic (and some
Business-Logic) sources but exposes one coherent view.

### Recommendation: one aggregate endpoint, backend-computed display name

Given both precedents, the merged list/detail should be:

1. **One new Business-Logic service** (working title `PersonDirectoryService`,
   see part a) that consumes `PersonService` (Basic) + `SalesPersonService`
   (Basic) + whatever exposes user/role/invitation reads today (likely via
   `PermissionService`'s user-facing reads and `UserInvitationService`) +
   `SalesPersonShiftplanService` if the shiftplan-assignment badge is needed in
   the list.
2. **One new flat list DTO**, e.g. `PersonEntryTO { person_id, display_name,
   kind: PersonEntryKindTO (UserOnly | SalesPersonOnly | Both), sales_person_id:
   Option<Uuid>, username: Option<Arc<str>>, inactive: bool, ... }` — small and
   purpose-built like `SalesPersonAbsenceTO`, NOT a reuse of `SalesPersonTO` or a
   `UserTO`. `display_name` is computed backend-side per the seed's own framing
   (`.planning/seeds/personenverwaltung-zusammenlegen.md:140-145`): "Nach Fat
   Backend, Thin Client sollte das Backend den Anzeigenamen fertig im DTO
   liefern" — this is not this researcher's inference, it's already decided in
   the seed and should be treated as settled for the phase plan.
3. **One aggregate GET endpoint** for the list (`GET /person` or similar)
   replacing the current two-load pattern
   (`page/user_management.rs:55-56`: `LoadAllUsers` + `LoadAllSalesPersons` run
   as two separate coroutine actions today) — one HTTP round trip, one backend
   union, exactly mirroring the WOP-01 motivation (v2.5 eliminated N sequential
   service calls in favor of one aggregate; the same economic argument applies
   here at smaller scale).
4. **The detail view is a separate, richer DTO/endpoint** (`GET /person/{id}`),
   not the same shape as the list row — the three existing detail pages
   (`page/sales_person_details.rs`, `page/user_details.rs`,
   `page/employee_details.rs`, per
   `.planning/seeds/personenverwaltung-zusammenlegen.md:29-33`) each expose
   different section data (Stammdaten/Shiftplan, Rollen/Einladungen,
   Report/Balance). Per the seed's own open question #1
   (`.planning/seeds/personenverwaltung-zusammenlegen.md:170-173`), whether
   `/employees/:id`'s HR-gated report data folds into this same aggregate or
   stays a separate linked view is **explicitly still open** — this research
   does not resolve it, but notes the section-based DTO shape (fields present
   or `None` depending on `kind`) is what "Fat Backend" implies once that
   decision is made: the backend, not the frontend, decides which sections
   apply to a given entry's `kind`.
5. **Per-section auth-gating stays a backend concern.** The seed's open question
   #2 (`.planning/seeds/personenverwaltung-zusammenlegen.md:174-177`) —
   Rollen/Einladungen are Admin-gated, Stammdaten/Shiftplan are
   shiftplanner-gated — means the detail DTO's role/invitation fields should be
   `Option<T>`-redacted server-side for non-admins, the same "API-level
   None-redaction" pattern already used for `VoluntaryStatsService`
   (`docs/architecture/02-service-tiers.md:55-57`: "Non-HR receives all-`None`
   fields, not 403") rather than a 403 on the whole aggregate endpoint. This
   precedent is directly reusable and should be named explicitly in the phase
   plan.

### Uncertain / needs a decision before planning (flagged per quality gate)

- Whether the list DTO needs a `kind` enum or whether `sales_person_id` /
  `username` being `Option<...>`-populated is sufficient signal — a minor
  DTO-shape decision, not an architecture blocker.
- Whether `PersonDirectoryService` is truly a *new* service or whether the
  merged list is cheap enough to live as an added method on `SalesPersonService`
  itself. This research recommends the new service because `SalesPersonService`
  is Basic-tier and must not gain a `UserService`/roles dependency (see part a);
  but the discuss-phase should confirm this explicitly rather than let it be
  assumed.

---

## (d) Planning-target-hours: exact injection point and untouched consumers

### The chain, traced end-to-end

1. **`weight_for_week`** (`service_impl/src/reporting.rs:2137-2193`) is the
   single shared pure function computing per-week contract weight. It returns a
   4-tuple `(expected_hours_maybe_zeroed, dynamic_hours_raw, days_per_week,
   workdays_per_week)`. The `is_dynamic` zero-out happens at
   `service_impl/src/reporting.rs:2186-2188`.
2. **Three independent call sites** consume it, each reproducing similar
   downstream reduction logic (absence/holiday subtraction) — this duplication
   is pre-existing, not something this milestone should try to unify unless
   explicitly scoped to do so:
   - `assemble_weeks` (`service_impl/src/reporting.rs:769-1160`, call at
     `:996-1000`) — feeds `ShortEmployeeReport` (constructed
     `:1126-1150`), consumed by both `get_week` (`:1900-1979`, thin wrapper,
     confirmed by reading `:1920-1978`) and `get_year`. `get_week` is the exact
     function backing `GET /report/week/{year}/{calendar_week}`
     (`rest/src/report.rs:145-175`, `get_short_week_report`), which is what
     `shifty-dioxus/src/loader.rs:376-398`
     (`load_working_hours_minified_for_week` → `api::get_working_hours_for_week`)
     calls. **This is the confirmed, verified path feeding
     `WorkingHoursMiniOverview`** — the todo's suspicion is correct.
   - `get_reports_for_all_employees` (`service_impl/src/reporting.rs:1164-1665`,
     `weight_for_week` call at `:1318-1332`) — a **separate, independently
     duplicated** loop (does NOT call `assemble_weeks`) also producing
     `ShortEmployeeReport` (constructed `:1650-1664`), backing
     `GET /report?year&until_week` (`rest/src/report.rs:63-99`,
     `get_short_report_for_all`). Same DTO type, same `is_dynamic`-zero
     problem, computed by entirely separate code. **Both call sites must be
     fixed together** — this is structurally the same trap as the Phase 57
     rebooking-filter bug (`.planning/PROJECT.md:197-210`: one requirement,
     two write-sites, only one got fixed first). Flag this explicitly in the
     phase plan's test matrix.
   - `hours_per_week` (`service_impl/src/reporting.rs:2195-2467`,
     `weight_for_week` call at `:2231-2243`) — feeds `GroupedReportHours`
     (`service/src/reporting.rs:78-103`), which backs `EmployeeReport.by_week`/
     `by_month` (`service/src/reporting.rs:174-203`) and ultimately
     `EmployeeReportTO`, NOT `ShortEmployeeReportTO`. **Out of scope** per the
     todo's own framing (`.planning/todos/pending/2026-09-15-...md:96-99`:
     "das ist aber eine bewusste Entscheidung pro Stelle") — do not touch this
     site unless a follow-up explicitly asks for the employee-detail report to
     show the same planning target.

### Where the new value should be computed

Per "Fat Backend, Thin Client" (`.planning/PROJECT.md:56-78`) and the todo's own
conclusion (`.planning/todos/pending/2026-09-15-...md:66-83`), the new value —
working title `planning_target_hours` — belongs on `ShortEmployeeReport`
(`service/src/reporting.rs:147-171`) / `ShortEmployeeReportTO`
(`rest-types/src/lib.rs:377-406`), computed at **both** of the two call sites
that produce `ShortEmployeeReport` (`assemble_weeks` and
`get_reports_for_all_employees`), NOT by reusing `dynamic_hours` (already
proven wrong by the todo's own analysis: `dynamic_hours` is deliberately kept
free of the derived-holiday reduction per the HSP-03 band guard, so switching
the frontend to `dynamic_hours` would regress the correct holiday-in-Soll
behavior for every non-dynamic contract).

Concretely, the minimal-diff shape:
1. Extend `weight_for_week`'s return with one more `f32`: the **unconditional**
   `employee_work_details.expected_hours * relation` (i.e., the same value
   currently only exposed as `dynamic_hours`, but computed independently so a
   future change to `dynamic_hours`'s semantics can't silently move this too —
   or, more simply, an explicit 5th tuple field so the two are structurally
   decoupled from day one).
2. At each of the two `ShortEmployeeReport`-producing call sites, apply the
   **same absence/holiday reduction pipeline currently applied to compute the
   real (possibly-zeroed) `expected_hours`** to this new raw value instead of
   applying the `is_dynamic → 0.0` branch. Concretely in `assemble_weeks`, that
   means mirroring `expected_hours_for_cap` (`:1087-1088`) and the final
   `holiday_derived_gated` subtraction (`:1091`) against the unconditional raw
   value rather than the `is_dynamic`-zeroed one.
3. Add `planning_target_hours: f32` to `ShortEmployeeReport`
   (`service/src/reporting.rs:147-171`) and `ShortEmployeeReportTO`
   (`rest-types/src/lib.rs:377-393`, plus the `From` impl at `:396-406`), with
   `#[serde(default)]` on the TO field for wire-compat (same pattern as
   `has_pending_rebooking`/`pending_rebooking_id` at `rest-types/src/lib.rs:386-390`).
4. Frontend: extend `WorkingHoursMini` and `build_working_hours_mini`
   (`shifty-dioxus/src/loader.rs:400-426`) to carry the new field, and switch
   the display target in `working_hours_mini_overview.rs:113,216` (per the
   todo's own file list) from `expected_hours` to `planning_target_hours`. This
   is the one place per the todo's own "Zu klären" section where the change to
   downstream consumers (progress bar, color thresholds, `total_target`) is a
   **deliberate per-site decision**, not automatic — flag for discuss-phase.

### Consumers of `expected_hours` (on `ShortEmployeeReport`) that must NOT be touched

Verified by reading actual usage, not just grepping the name (there are TWO
distinct fields named `expected_hours` in this codebase — one on
`EmployeeWorkDetails`, i.e. the raw contract row, and one on `ShortEmployeeReport`/
`GroupedReportHours`, the computed report aggregate; only the latter is in scope
here):

- **`BookingInformationServiceImpl::get_weekly_summary` /
  `get_summery_for_week`** (`service_impl/src/booking_information.rs:641-651`
  and `:815-826`): `WorkingHoursPerSalesPerson.available_hours =
  report.expected_hours` where `report` is the `ShortEmployeeReport` returned
  by `reporting_service.get_week(...)`. This is the exact "Bezahlten-Vertrag"
  band the v2.5 ship notes explicitly locked as a regression guard
  (`.planning/PROJECT.md:275-276`: "`WorkingHoursPerSalesPerson`
  (Bezahlten-Vertrag) unangetastet — Regression-Lock"). **Must keep reading
  `expected_hours` (still zeroed for dynamic contracts) — do not swap this to
  `planning_target_hours`.** This is the single highest-risk regression site in
  this feature; call it out explicitly in the phase plan's must-not-break list.
- **`billing_period_report.rs:234-237`** (`report_delta.expected_hours`,
  `report_start.expected_hours`, `report_end.expected_hours`,
  `report_end_of_year.expected_hours`) — these operate on `EmployeeReport`
  (from `hours_per_week`/`GroupedReportHours`, a different struct than
  `ShortEmployeeReport`), so they are structurally unreachable by this change
  as scoped, but confirms the billing/snapshot path's `expected_hours` is a
  wholly separate code path from the one being touched — reinforcing why no
  `CURRENT_SNAPSHOT_SCHEMA_VERSION` bump is needed here (matches the milestone
  decision in `.planning/PROJECT.md:548-550`).
- **`voluntary_stats.rs` / `reporting.rs:223`** (`expected_hours == 0` counting
  rule, D-F1-01) and **`booking_information.rs:504,518,545,872`**
  (`wh.expected_hours == 0.0` checks) all read `EmployeeWorkDetails.expected_hours`
  — the **contract's own field**, set directly on the `working_hours` row, not
  the computed report field this milestone touches. Confirmed by reading the
  surrounding code (`wh.` prefix = an `EmployeeWorkDetails` iteration variable).
  Unaffected either way, but worth naming in the plan so nobody conflates the
  two same-named fields during implementation or review.

### Uncertain / to verify at implementation time

- Whether `get_year` (the other consumer of `assemble_weeks`, feeding annual
  views) should also expose `planning_target_hours` on its
  `ShortEmployeeReport` rows, or whether that's out of scope because the annual
  view doesn't render this Soll column today. This research did not check
  `get_year`'s frontend consumer; flag for discuss-phase.
- The exact holiday/absence reduction formula to mirror for the "unconditional"
  raw value was read in `assemble_weeks` (`:1080-1101`) but the equivalent
  block in `get_reports_for_all_employees` (`:1340-1420` region, not fully
  transcribed above) should be re-read line-by-line during planning to confirm
  it applies the *same* reductions in the *same* order — the two loops are
  independently written and could have subtly diverged since the WOP-01
  refactor split them.

---

## Suggested build order (data-model-first, tier-respecting)

1. **Migration + backfill** (`person` table, nullable FKs, `sales_person_user`
   backfill into shared `person` rows) — per milestone constraint, before
   anything else. Docs-Freshness-Gate: `docs/architecture/03-data-model.md` +
   `diagrams/db-schema-er.mmd` in the same phase.
2. **`PersonDao` + `PersonService` (Basic tier)** — new DAO trait/impl, new
   Basic service, wired into `main.rs` per part (a). No REST surface required
   yet if nothing consumes it.
3. **`sales_person_user` dissolution in the DAO layer** — rewrite the six DAO
   methods (part b) to use `person_id` joins; drop the table in a follow-up
   migration once the backfill is verified. Service/REST layers untouched by
   construction (contract frozen) — this should be its own tight, low-risk
   plan precisely because the blast radius is small if the contract is kept.
4. **Person aggregate (Business-Logic tier) + list/detail DTOs + REST +
   frontend merged list/detail** (part c) — depends on 2 and 3.
5. **Planning-target-hours** (part d) is architecturally independent of 1-4 (it
   touches `reporting.rs`/`rest-types`/frontend `working_hours_mini_overview.rs`,
   no `person` involvement) and can be sequenced in parallel or before/after
   the Person work without either blocking the other. Given it is the smaller,
   better-understood change (bug-report-quality investigation already exists),
   it is a reasonable candidate for an early, low-risk phase while the Person
   migration is being reviewed.

## Sources

- `CLAUDE.md` (service tier rules, DI order, snapshot contract) — read in full.
- `.planning/PROJECT.md` (Fat Backend/Thin Client principle, v2.5/v2.6 ship
  notes, v2.7 milestone framing) — read in full.
- `.planning/seeds/personenverwaltung-zusammenlegen.md` — read in full.
- `.planning/todos/pending/2026-09-15-sollstunden-fehlen-bei-dynamischen-vertraegen-unter-schichtplan.md` — read in full.
- `shifty_bin/src/main.rs` — read in full (both halves).
- `docs/architecture/02-service-tiers.md` — read in full.
- `service/src/sales_person.rs`, `dao/src/sales_person.rs`,
  `service_impl/src/sales_person.rs`, `dao_impl_sqlite/src/sales_person.rs`,
  `rest/src/sales_person.rs` — read relevant sections.
- `service_impl/src/reporting.rs` (`weight_for_week`, `assemble_weeks`,
  `get_reports_for_all_employees`, `hours_per_week`, `get_week`) — read in
  detail across all three call sites.
- `service/src/reporting.rs` (`GroupedReportHours`, `ShortEmployeeReport`,
  `EmployeeReport`) — read in full.
- `rest-types/src/lib.rs` (`ShortEmployeeReportTO`, `WeeklySummaryTO`,
  `SalesPersonAbsenceTO`) — read relevant sections.
- `rest/src/report.rs`, `rest/src/booking_information.rs` — read relevant
  sections.
- `service_impl/src/booking_information.rs` — read relevant sections
  (`get_weekly_summary`/`get_summery_for_week` `WorkingHoursPerSalesPerson`
  construction).
- `shifty-dioxus/src/loader.rs` — read `build_working_hours_mini` and
  `load_working_hours_minified_for_week` in full.
- `migrations/sqlite/20240506114107_add_sales_person.sql` — read in full.
- `dao_impl_sqlite/src/lib.rs` (`BasicDaoImpl::clear_all`) — read relevant
  section.

---
*Architecture research for: Shifty Backend v2.7 (Personenverwaltung & Planungs-Soll)*
*Researched: 2026-09-16*
