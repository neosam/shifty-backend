# Project Research Summary

**Project:** Shifty — v1.4 "Committed Voluntary Capacity"
**Domain:** Brownfield field-add — time-versioned `committed_voluntary: f32` on `EmployeeWorkDetails`, threaded through DAO → Service → rest-types → reporting/year-view → Dioxus, with no-double-count reporting and a billing-snapshot bump (D-01 / Variante B)
**Researched:** 2026-06-22
**Confidence:** HIGH (every integration point, formula site, and pitfall was verified against actual repo files at `HEAD`)

## Executive Summary

v1.4 adds a single forward-looking number — `committed_voluntary` — to the time-versioned `EmployeeWorkDetails` contract record, so the Jahresansicht (year view) can show *pledged* voluntary capacity alongside paid and actual-volunteer hours. The data-model half is mechanically trivial: there are **two near-identical precedents** (`cap_planned_hours_to_expected` from v1.3 and `is_dynamic` before it) for adding exactly this kind of field to exactly this entity. Copy them line-for-line — one additive `ALTER TABLE … ADD COLUMN committed_voluntary REAL NOT NULL DEFAULT 0` migration, the field on the DAO row/entity/service struct/DTO, both conversion directions at each boundary, `#[serde(default)]` for wire backward-compat, and `cargo sqlx prepare` to regenerate the offline query cache. No new dependency, no new DI wiring (`EmployeeWorkDetailsService` stays a Basic Service), no OpenAPI/utoipa work (this endpoint family deliberately has none today).

The *hard* half is reporting integration, and the single most important finding across all four research files is that **the year view is NOT fed by `reporting.rs`.** The Jahresansicht (`weekly_overview`) is fed by `booking_information.rs::get_weekly_summary` → `WeeklySummaryTO`. There are **two independent, non-interchangeable "volunteer_hours" axes**: Axis A is the reactive cap-overflow computed in `reporting.rs` (consumed by the per-employee report and the billing snapshot), and Axis B is the sum of booked hours of `is_paid=false` persons computed in `booking_information.rs` (this IS the year view's volunteer number). The todo's formula wording (`available = expected + committed_voluntary`) reads as if it targets `reporting.rs`, but the year-view integration must land in `booking_information.rs::get_weekly_summary`. Conflating the two axes is the highest-risk mistake in the milestone. The no-double-count rule reduces to a clean closed form — `counted_volunteer = max(committed_voluntary, actual_volunteer)` **per ISO-week, summed over the year** (never `max(Σ, Σ)`) — and the strong recommendation is to add a **new, separate** `committed_voluntary_hours` term to `WeeklySummary` rather than folding it into the paid or volunteer terms, which both satisfies the "separate display" requirement (D-01 #4) and structurally prevents double-counting.

Three risks are load-bearing and should drive phase design. (1) **Snapshot drift:** the change alters the input/computation of the persisted `BillingPeriodValueType::Volunteer`, so `CURRENT_SNAPSHOT_SCHEMA_VERSION` MUST bump 7→8 **in the same commit** as the reporting change. (2) **Unpaid-volunteer record side effects:** giving a pure volunteer an `EmployeeWorkDetails` row (so they can hold a pledge) breaks the historical "has work-details ⇒ paid employee" assumption — `is_paid` lives on `SalesPerson`, not `EmployeeWorkDetails`, and several reporting/billing sites iterate work-details-holders without a paid filter. (3) **Scope-gate mismatch:** the intended gate is `cap_planned_hours_to_expected=true`, but the year-view volunteer axis actually keys on `is_paid=false` — these do not coincide for the motivating "5h paid + 5h pledged" person, so where exactly the pledge lands is a real open decision.

## Key Findings

### Recommended Stack

The stack is **fixed** — this is a reuse map, not a tech selection. There is essentially nothing new to add; the job is to copy how `cap_planned_hours_to_expected` was threaded through every layer in v1.3. See `STACK.md` for the line-by-line reuse table.

**Core technologies (all existing, unchanged):**
- SQLite via `sqlx` (compile-time-checked, committed `.sqlx/` offline cache) — additive `ADD COLUMN … REAL NOT NULL DEFAULT 0` migration; `cargo sqlx prepare` after editing any `query!`/`query_as!`. On NixOS use `nix develop` + `sqlx migrate run` (additive); **never** `sqlx database reset` (destructive, requires user confirmation).
- `gen_service_impl!` DI macro — **no change**; the field is data, not a dependency; `EmployeeWorkDetailsService` stays a Basic Service.
- Single shared `rest-types` crate — `EmployeeWorkDetailsTO` gets `#[serde(default)] committed_voluntary: f32` (no `ToSchema`, matching the surrounding struct); the consolidation means a missing mirror breaks the WASM compile (intended safety net).
- Dioxus 0.6.3 (WASM, dx-CLI pinned to 0.6.x in `flake.nix`) — mirror the field into frontend state + both `TryFrom` directions; numeric input (the date-input test caveat does NOT apply).

### Expected Features

See `FEATURES.md`. The closed-form rule is `counted_volunteer = max(committed_voluntary, actual_volunteer)` per ISO-week (`committed=0` ⇒ identical to today, guaranteeing backward-compat).

**Must have (table stakes):**
- `committed_voluntary: f32` on `EmployeeWorkDetails`, time-versioned (shares the row's from/to window) — the foundation.
- Pledge folded into year-view available capacity **per week** via `max(committed, actual)`, integrated in `booking_information.rs::get_weekly_summary` (Axis B), **not** `reporting.rs`.
- Committed capacity shown **separately** from paid & volunteer (a third token, e.g. `zugesagt`), with i18n in all three locales (En/De/Cs).
- Snapshot version bump 7→8 (the `Volunteer` value_type computation changes).
- "alle"-filter + unpaid-volunteer record path (`is_paid=false`, `expected_hours=0`) with verified `get_week` side-effects.

**Should have (competitive / defer to v1.5):**
- Surplus display (`5 + 2` when actual exceeds pledge) — `diff_color_and_sign` token reuse.
- Pledge-unmet **inline banner** (not a blocking dialog, per project preference) when committed > actual.
- Committed band in `WeeklyOverviewChart`.

**Defer (v2+):**
- Pledge approval workflow (out per PROJECT.md SC-01), min-paid-capacity / skill matching (SC-02), average-attendance evaluation.

**Explicit anti-features:** do NOT reuse the `reporting.rs` cap-overflow path for the year-view number (double-counts against Axis B); do NOT give unpaid volunteers a paid-style record (`expected_hours>0` flips them into paid loops); do NOT add a `committed >= expected` invariant (Variante A was rejected); do NOT pro-rate the pledge by partial-week weight; do NOT couple the pledge to absence/vacation.

### Architecture Approach

See `ARCHITECTURE.md`. The existing layered architecture is fixed; this is an integration/build-order exercise. The field rides the *same* time-versioned `EmployeeWorkDetails` row as `expected_hours` and `cap_planned_hours_to_expected`, so it inherits the from/to ISO-week window for free — only the *value* is decoupled, not the *time window* (a key subtlety, see Pitfall 4).

**Major components / touch boundaries:**
1. **Data model** (`migrations/sqlite`, `dao`, `dao_impl_sqlite`, `service/src/employee_work_details.rs`, `rest-types`) — field + conversions at every boundary; `.sqlx` regen is the first compile gate. REST handlers and OpenAPI: **no change** (serde-transparent, no `#[utoipa::path]`/`ToSchema` here today — do not add a phantom OpenAPI task).
2. **Reporting + snapshot** (`service_impl/src/reporting.rs` 3 parallel volunteer sites; `service_impl/src/billing_period_report.rs` version bump) — Business-Logic tier, no new DI dep; bump fused into the same commit.
3. **Year-view** (`service_impl/src/booking_information.rs::get_weekly_summary` → `service::booking_information::WeeklySummary` → `WeeklySummaryTO` → frontend `state/weekly_overview.rs` + `page/weekly_overview.rs`) — the real display integration, via a **new separate** `committed_voluntary_hours` term.
4. **Contract editor + "alle"-filter** (`shifty-dioxus/src/component/contract_modal.rs`, `state/employee_work_details.rs`, plus relaxing paid-only filters in `loader.rs`/`reporting.rs`/`booking_information.rs`) — the most design-open part.

### Critical Pitfalls

Top items from `PITFALLS.md` (read P1, P2, P5 first — they are load-bearing):

1. **Double-counting committed vs already-counted volunteer hours** — implement replacement-not-addition: `counted = committed + max(0, actual − committed) = max(committed, actual)` per week. Pick **one** reconciliation site (`booking_information::get_weekly_summary`), enter the pledge as a separate max-clamped term, never `committed + actual`.
2. **Forgetting the snapshot bump in the same commit** — the `Volunteer` value_type's input/computation changes, so bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` 7→8 with a `// - v8:` history note, atomically with the reporting change (or write an explicit no-bump justification proving no persisted value_type changed).
3. **Unpaid-volunteer record leaks into paid-only queries** — `is_paid` is on `SalesPerson`, not `EmployeeWorkDetails`; enumerate and gate every work-details-iterating site (see at-risk list below). Gate on `sales_person.is_paid`, not on record presence; keep `expected_hours=0` AND explicitly exclude `!is_paid` from `paid_hours`/billing.
4. **Time-version skew** — value is decoupled but the time window is *shared* with the contract row; changing the pledge mid-window requires rotating the row (same as the cap flag). Read the pledge via the **same** `find_working_hours_for_calendar_week` selection; define behaviour for two overlapping active rows in one week (sum vs max — open decision).
5. **Scope-gate leakage** — gate every read of `committed_voluntary` behind the same `cap_active` predicate; when cap is off, the pledge contributes `0.0` to everything.

**At-risk work-details-iterating sites for the unpaid-volunteer record (enumerated):**
- `reporting::get_week` (`reporting.rs:719`) — iterates `all_for_week` (NOT paid-filtered) → unpaid volunteer now gets a `ShortEmployeeReport` row. **The main surprise** (feeds the year view via `:133`).
- `booking_information` `paid_hours` accumulation (`:176-178`, `:288-290`) — sums `dynamic_hours` over every `get_week` row → unpaid volunteer's `expected_hours` could leak into `paid_hours`.
- `reporting::get_reports_for_all_employees` (`:139-142`) — `is_paid`-filtered → unpaid volunteer *excluded* here → person-set inconsistency vs the year summary.
- `booking_information` day-level loop (`:341`) — `is_paid==true` only → per-day vs weekly-total disagreement.
- `billing_period_report::build_new_billing_period` (`:315-332`) — iterates `get_all` (no paid filter) → could write a snapshot balance row for an unpaid person.
- `vacation_balance` (`:146`) — uses `get_all_paid` → excluded (verify nothing reads work-details directly).

## Implications for Roadmap

Phases are ordered strictly by **compile dependency** (each compiles/tests green before the next; backend foundation precedes frontend that consumes it). This matches the build order in `ARCHITECTURE.md`.

### Phase A: Data-model foundation (backend)
**Rationale:** The field must exist on `EmployeeWorkDetails` (service) before reporting can read it or the DTO can transport it. `.sqlx` regeneration is the first hard compile gate.
**Delivers:** migration + `.sqlx` regen → DAO entity/row + 4 SELECT + INSERT + UPDATE + `TryFrom` → service struct + 2 conversions → `EmployeeWorkDetailsTO` + 2 `From` impls + `#[serde(default)]` → extend `employee_work_details_update` integration test (fractional roundtrip).
**Addresses:** table-stakes field; backward-compat (`DEFAULT 0`).
**Avoids:** P3 (forward-default migration), the silent-`0.0`/compile-error of a missed conversion site.
**Note:** field is inert (read nowhere yet); no REST/OpenAPI change.

### Phase B: Reporting no-double-count + snapshot bump (SAME commit)
**Rationale:** Needs Phase A's field; the bump and the formula switch are atomic by the snapshot-versioning contract.
**Delivers:** `reporting.rs` — all 3 parallel volunteer sites changed identically, gated on `cap_planned_hours_to_expected`, exposing a separate `committed_voluntary_hours` on the report structs; `billing_period_report.rs` — `CURRENT_SNAPSHOT_SCHEMA_VERSION` 7→8 + `// - v8:` history entry; fixture + snapshot-validator tests.
**Implements:** Business-Logic reporting tier (no new DI dep).
**Avoids:** P1 (double-count), P2 (snapshot drift), P6 (scope-gate leakage), Anti-Pattern 2 (changing only one of the three sites).

### Phase C: Jahresansicht display (year view)
**Rationale:** Needs Phase B's `committed_voluntary_hours`; the read/display path is lower-risk and self-contained, so ship it before the design-open editing path.
**Delivers:** `booking_information.rs::get_weekly_summary` adds the **separate** `committed_voluntary_hours` term into `overall_available_hours` (per-week `max`); `service::booking_information::WeeklySummary` + `WeeklySummaryTO` + `From`; frontend `state/weekly_overview.rs` + `page/weekly_overview.rs` renders a third token; i18n x3.
**Addresses:** "pledge counts into available capacity" + "shown separately" (D-01 #3/#4).
**Avoids:** P1 (integration in Axis B, not Axis A), the `From<&WeeklySummaryTO>` mapping-omission gotcha.

### Phase D: Contract editor input + "alle"-filter / unpaid-volunteer path
**Rationale:** Input editing needs the Phase A DTO field; the "alle" path carries the open paid-only-assumption design question — split out so its risk doesn't block C. **Most design-open phase.**
**Delivers:** `contract_modal.rs` numeric input (next to the cap toggle) + `state/employee_work_details.rs` both `TryFrom`; "alle"-toggle relaxing paid-only filters in list/loader/reporting so unpaid volunteers get an EWD record and become selectable; i18n x3; side-effect tests.
**Addresses:** todo req 5 (unpaid-volunteer visibility + record).
**Avoids:** P5 (unpaid-volunteer leakage) — the enumerated at-risk sites must each be gated on `is_paid`.

### Phase Ordering Rationale
- **A before everything:** field must exist before it can be read/transported; `.sqlx` is the gating step.
- **B fuses snapshot bump with the formula switch:** non-negotiable (same commit, or the stamp lies about the rules).
- **C before D:** display (read path) is lower-risk and self-contained; editing + "alle"-filter carries the open design question. C ships the year view showing committed capacity (zero until D enables entry) without being blocked by D.
- **Frontend trails its backend DTO in each phase:** the single `rest-types` crate breaks the WASM compile if a field is unmirrored — forcing sync within the phase.
- **VCS:** jj-only commits; GSD auto-commit disabled; user commits each phase manually.

### Research Flags

Phases likely needing deeper research / a dedicated design note during planning:
- **Phase B (reporting/year-view math):** highest-risk phase — the two-axes reconciliation and the per-week-`max` no-double-count rule warrant a design note before planning. Resolve **D-FORMULA-PATH** and **D-SCOPE-GATE** first.
- **Phase D (unpaid-volunteer record + "alle"-filter):** the at-risk-site enumeration (P5) and the paid-only-assumption relaxation are the deliverable; this is the most design-open area. Resolve **D-UNPAID-RECORD**.

Phases with standard patterns (skip research-phase):
- **Phase A (data-model):** two exact precedents (`cap_planned_hours_to_expected`, `is_dynamic`); copy line-for-line.

## Open Decisions for Requirements/Roadmap

These MUST be answered before/within requirements. Each is a real fork verified against the code, not a stylistic choice.

| ID | Decision | Recommendation |
|----|----------|----------------|
| **D-FORMULA-PATH** | Does the year-view integration land in `booking_information.rs::get_weekly_summary` (Axis B) or `reporting.rs` (Axis A)? The todo's wording implies `reporting.rs` but that path never reaches the year view. | **Axis B / `booking_information.rs`.** Treat Axis A and Axis B as independent; do NOT merge. |
| **D-SCOPE-GATE** | For a capped **paid** person (`is_paid=true`, `cap=true`, the "5h paid + 5h pledged" example), where does the pledge land — `paid_hours`, `volunteer_hours`, or a new term? The intended gate (`cap=true`) and the year-view volunteer gate (`is_paid=false`) do **not** coincide for this person. | **A new, separate `committed_voluntary_hours` term** on `WeeklySummary`, gated on `cap_planned_hours_to_expected=true`. Genuinely "separate" (req 4) and never double-counts against Axis B. Confirm display for non-capped persons: blank/dash, not `0`. |
| **D-PARTIAL-WEEK** | Is the pledge flat-per-active-week or pro-rated by `weight_for_week` like `expected_hours`? | **Flat per active week** — pro-rating would silently shrink a "5h pledge" on first/last partial weeks. |
| **D-UNPAID-RECORD** | What field values for a pure unpaid-volunteer's EWD record, and how are the at-risk paid-only sites gated? | **`is_paid=false` + `expected_hours=0`** (likely `cap=true` so the gate lets the pledge through); explicitly gate every enumerated site on `sales_person.is_paid` (not record presence); add the `get_week` side-effect integration test (#4/#5). |
| **D-SNAPSHOT** | Bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` 7→8? | **Yes, bump 7→8, same commit as Phase B** — the persisted `BillingPeriodValueType::Volunteer` computation/input changes. (One file notes a conditional "verify it's persisted first" framing; the architecture + pitfalls reads confirm `Volunteer` IS persisted and IS affected → bump.) |
| **D-ABSENCE-DISPLAY** | Does the pledge interact with absence/holiday/vacation? | **No** — flat capacity number, no cross-entity coupling. Any "hide on full-holiday week" is a display-only filter, kept out of the core formula. |
| **D-OVERLAP-AGG** *(new, from P4)* | When two active `EmployeeWorkDetails` rows overlap one ISO-week and both carry `committed_voluntary`, which wins — sum, max, or first? The existing `.any(...)` cap pattern doesn't define this for a numeric value. | **Pin it explicitly with a test** (the cap flag only needs `.any()` because it's boolean; a numeric pledge needs a defined aggregation). Add to a CONTEXT decision. |

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Every mechanism verified against repo files at `HEAD`; two exact precedents for the field-add. |
| Features | HIGH | Formula + side-effects verified directly against `reporting.rs` and `booking_information.rs`, line areas cited. |
| Architecture | HIGH | All 14 integration files located and line-verified; build order derived from compile dependencies. |
| Pitfalls | HIGH | Grounded in direct reads plus the v1.3 Phase-8.4 additive-merge and snapshot-versioning lessons in STATE.md/CLAUDE.md. |

**Overall confidence:** HIGH

### Gaps to Address

- **D-SCOPE-GATE landing (the "5h paid + 5h pledged" person):** the precise term and per-week math for capped-paid persons is the one genuinely-open formula decision — resolve in Phase B planning with a worked-example test fixture. *Highest-leverage gap.*
- **D-OVERLAP-AGG:** overlapping active work-details rows in one week is undefined for a numeric pledge; decide sum/max/first and pin with a test (data-model + reporting phases).
- **D-UNPAID-RECORD side-effects (#4/#5):** the benign-`get_week`-row behaviour for a zero-expected unpaid record is the highest-risk integration test; verify person-set consistency across year-summary / all-employees-report / billing.
- **Pledge carried forward on contract-version rotation:** confirm the editor/update path doesn't reset `committed_voluntary` to default when rotating a row (struct-update spread should carry it — verify).
- **D-SNAPSHOT framing reconciliation:** FEATURES.md states the bump conditionally ("verify `volunteer_hours` is persisted first"); ARCHITECTURE.md + PITFALLS.md confirm `BillingPeriodValueType::Volunteer` IS persisted and affected → bump 7→8. Resolved as "bump," but trace it explicitly in Phase B success criteria.

## Sources

### Primary (HIGH confidence — direct repo reads at `HEAD`, commit `5ade710`)
- `service_impl/src/reporting.rs` — `apply_weekly_cap` (:94-107), 3 volunteer sites (:362-367/:781-788/:852-854), `get_week` record-keyed iteration (:719), `is_paid` filter (:139-142).
- `service_impl/src/booking_information.rs` — `get_weekly_summary` Axis B (:95-218), volunteer sum (:141-153), `paid_hours` (:176-178), `overall_available_hours` (:197/:309), day-level paid filter (:341).
- `service_impl/src/billing_period_report.rs:74` — `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7`; `Volunteer` value_type (:240-250); `build_new_billing_period` (:315-332).
- `service/`, `dao/`, `dao_impl_sqlite/src/employee_work_details.rs` — struct, conversions, DAO columns; `service/src/sales_person.rs:17` (`is_paid` on SalesPerson).
- `rest-types/src/lib.rs` — `EmployeeWorkDetailsTO` (:596-705), `WeeklySummaryTO` (:900-944).
- `shifty-dioxus/src/{state,page}/weekly_overview.rs`, `state/employee_work_details.rs`, `component/contract_modal.rs`, `loader.rs`, `api.rs`.
- `migrations/sqlite/20260426120000_add-cap-flag-to-employee-work-details.sql` (migration template); `flake.nix:197-198` (sqlx-cli/sqlite).
- `shifty-backend/CLAUDE.md` Layered Architecture / Service-Tier-Konventionen / Billing Period Snapshot Schema Versioning.
- `.planning/STATE.md` (Phase-8.4 additive-merge no-double-count + snapshot-bump-same-commit + i18n guard lessons).
- `.planning/PROJECT.md` v1.4 + `.planning/todos/pending/2026-06-22-committed-voluntary-capacity-jahresansicht.md` (D-01 / Variante B, reqs 1-5, worked examples).

### Secondary (HIGH confidence — user-confirmed conventions)
- Project memory: `reference_local_dev_commands` (nix develop / migrate run vs destructive reset), `feedback_destructive_db_ops`, `project_frontend_dx_version_pin`, `feedback_service_tier_convention`, `feedback_warnings_inline_not_dialog`, `reference_dioxus_browser_test_date_inputs`.

### Detailed research documents
- `.planning/research/STACK.md`, `FEATURES.md`, `ARCHITECTURE.md`, `PITFALLS.md` (all HIGH confidence, line-verified).

---
*Research completed: 2026-06-22*
*Ready for roadmap: yes*
