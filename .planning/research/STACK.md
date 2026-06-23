# Stack Research — v1.4 Committed Voluntary Capacity

**Domain:** Brownfield internal feature — add one time-versioned `f32` field (`committed_voluntary`) to `EmployeeWorkDetails`, threading it through DAO → Service → rest-types → reporting → Dioxus frontend.
**Researched:** 2026-06-22
**Confidence:** HIGH (every mechanism verified against actual repo files at the commit on `HEAD`)

> This is a **reuse map**, not a tech selection. The stack is fixed. There is **almost nothing new to add** — the whole job is "copy how `cap_planned_hours_to_expected` was added in v1.3 and how `is_dynamic` was added before it." Two near-identical precedents exist for adding a column to this exact entity; follow them line-for-line.

---

## TL;DR — Mechanisms This Feature MUST Reuse

| Concern | Existing mechanism to copy | Reference precedent |
|---------|---------------------------|---------------------|
| Time-versioning | **from/to ISO-week date ranges** (`from_year/from_calendar_week/from_day_of_week` .. `to_*`), NOT logical_id rotation | already on `EmployeeWorkDetails` |
| Column add | `ALTER TABLE … ADD COLUMN … INTEGER/REAL NOT NULL DEFAULT 0` migration | `20260426120000_add-cap-flag-to-employee-work-details.sql` |
| sqlx compile-check | `cargo sqlx prepare` regenerates `.sqlx/` after editing `query!`/`query_as!` | 155 files in `.sqlx/` |
| DI wiring | `gen_service_impl!` macro (no change needed — no new dep) | `service_impl/src/employee_work_details.rs:21` |
| Row→Entity conversion | hand-written `TryFrom<&…Db>` with `as f32` / `!= 0` coercions | `dao_impl_sqlite/src/employee_work_details.rs:43` |
| DTO derives | `Serialize, Deserialize` + `#[serde(default)]` for backward-compat | `rest-types/src/lib.rs:596` |
| Snapshot versioning | bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` | `service_impl/src/billing_period_report.rs:74` (currently `7`) |
| Frontend model | mirror DTO into `state::employee_work_details::EmployeeWorkDetails` + both `TryFrom` directions | `shifty-dioxus/src/state/employee_work_details.rs` |

---

## 1. Time-Versioning Pattern (the answer to "how does this field get versioned?")

**Verdict: date-range versioning via ISO-week tuples, NOT logical_id rotation.**

`EmployeeWorkDetails` is **not** versioned by `logical_id` rotation (that pattern exists elsewhere — `extra_hours` got a `logical_id` in `20260428101456_add-logical-id-to-extra-hours.sql` — but `EmployeeWorkDetails` does **not** use it). Instead each row carries a validity window:

```
from_day_of_week / from_calendar_week / from_year   ← inclusive start (ISO week date)
to_day_of_week   / to_calendar_week   / to_year     ← inclusive end   (ISO week date)
```

(`service/src/employee_work_details.rs:13-41`, mirrored in `dao/src/employee_work_details.rs:9-37`.)

**How "versioning" actually happens** (`service_impl/src/employee_work_details.rs`):
- `create` (l.182) inserts a brand-new row with a fresh `id` + `version` UUID and an open-ended `from`/`to` window.
- `update` (l.217) only mutates the **trailing boundary + payload fields** (`to_*`, `expected_hours`, `vacation_days`, `workdays_per_week`, `is_dynamic`, `cap_planned_hours_to_expected`) and rotates the optimistic-lock `version` UUID. To "change a value from week X onward" the caller closes the old row's `to_*` and `create`s a successor — the frontend does this (`save_new_employee_work_details` + `update_employee_work_details` in `loader.rs:666/674`).
- Lookup-by-week is a pure range query: `find_for_week` (DAO `dao_impl_sqlite/src/employee_work_details.rs:254`) does `(from_year*100+from_calendar_week) <= (?*100+?) AND (to_year*100+to_calendar_week) >= (?*100+?)`; service `find_for_week` (l.90) does the same comparison in Rust.

**Consequence for `committed_voluntary`:** it is a plain payload field on the *same* row — it inherits the from/to window automatically. **D-01/Variante B is satisfied for free:** because the voluntary pledge lives in its own column on the same versioned row as `expected_hours`, it is independently editable but shares the time window. There is **no new versioning machinery to build.** The one decision the planner must make: **add `committed_voluntary` to the mutable set in the service `update` method** (`service_impl/src/employee_work_details.rs:241-248`) and to the DAO `UPDATE` statement (`dao_impl_sqlite/src/employee_work_details.rs:421-448`) — exactly parallel to how `cap_planned_hours_to_expected` appears in both lists.

`has_day_of_week` / the weekday booleans are an **orthogonal** concern (which weekdays the person works) and are NOT part of versioning — do not touch them.

---

## 2. SQLite Migration + `.sqlx` Regeneration (NixOS workflow)

### The migration
Add **one** additive migration `migrations/sqlite/<UTC-stamp>_add-committed-voluntary-to-employee-work-details.sql`. Copy the cap-flag precedent verbatim, changing type to `REAL` (f32 → SQLite `REAL`):

```sql
ALTER TABLE employee_work_details
ADD COLUMN committed_voluntary REAL NOT NULL DEFAULT 0;
```

(Precedent: `20260426120000_add-cap-flag-to-employee-work-details.sql` used `INTEGER NOT NULL DEFAULT 0` for a bool; `is_dynamic` did the same. For an `f32` use `REAL`. `NOT NULL DEFAULT 0` keeps all existing rows valid → no backfill, no down-migration needed; this project's migrations are forward-only.)

Filename stamp: sortable `YYYYMMDDHHMMSS`; latest in tree is `20260611000002`. Pick a stamp after it.

### Local workflow on NixOS (verified)
`flake.nix:197-198` puts **`sqlx-cli`** and **`sqlite`** in the dev shell. Per project memory (`reference_local_dev_commands`, `feedback_destructive_db_ops`):

```bash
nix develop                       # enter dev shell (NOT nix-shell — shell.nix is broken)

# additive, NON-destructive — apply the new migration to localdb.sqlite3:
sqlx migrate run --source migrations/sqlite

# regenerate the offline query cache after editing any query!/query_as!:
cargo sqlx prepare --workspace
#   (writes/updates .sqlx/query-*.json — 155 files today; commit the new/changed ones)
```

- **`DATABASE_URL=sqlite:./localdb.sqlite3`** (`env.example`) must be set (`.env`).
- `sqlx migrate run` is **additive** — safe, only applies pending migrations.
- **`sqlx database reset` is DESTRUCTIVE** (drops + recreates) — the dev DB is **not recoverable**; require explicit user confirmation before ever running it. For this feature `migrate run` is sufficient; `reset` is not needed.
- `query_as!`/`query!` are **compile-time checked** against the DB schema. After editing the EWD `SELECT`/`INSERT`/`UPDATE` statements you MUST run the migration first (so the live schema has the column) then `cargo sqlx prepare`, otherwise `cargo build` fails with "column not found" or a stale-cache error.

---

## 3. DI Macro + DAO Trait/Impl + `TryFrom` Threading

No DI change — the field is data, not a dependency.

- **`gen_service_impl!`** (`service_impl/src/employee_work_details.rs:21`): the dependency set (`EmployeeWorkDetailsDao`, `SalesPersonService`, `PermissionService`, `ClockService`, `UuidService`, `TransactionDao`) is **unchanged**. Do not add deps. `EmployeeWorkDetailsService` is a **Basic Service** per the service-tier convention — keep it basic (no new domain-service deps).
- **DAO trait** (`dao/src/employee_work_details.rs`): add field to `EmployeeWorkDetailsEntity` struct (l.9-37). Trait method signatures are unchanged.
- **DAO impl** (`dao_impl_sqlite/src/employee_work_details.rs`), four touch points — all parallel to `cap_planned_hours_to_expected`:
  1. `EmployeeWorkDetailsDb` row struct: add `pub committed_voluntary: f64` (SQLite REAL → Rust `f64`; precedent: `expected_hours: f64` at l.17).
  2. `TryFrom<&EmployeeWorkDetailsDb>` (l.43): `committed_voluntary: working_hours.committed_voluntary as f32` (mirror `expected_hours as f32` at l.50).
  3. `SELECT` lists (4 queries: `all`, `find_by_id`, `find_by_sales_person_id`, `find_for_week`) — add the column name.
  4. `INSERT` in `create` (l.339) — add column + `?` placeholder + bound local; and `UPDATE` in `update` (l.421) — add `committed_voluntary = ?` to the SET list (this is what makes the value editable on the active version).
- **Service↔Entity `From`/`TryFrom`** (`service/src/employee_work_details.rs`): add the field to `From<&…Entity> for EmployeeWorkDetails` (l.42) and `TryFrom<&EmployeeWorkDetails> for …Entity` (l.192). Add to the service `update` mutable set (l.241-248).

---

## 4. rest-types DTO Derive Conventions

`EmployeeWorkDetailsTO` (`rest-types/src/lib.rs:596`) derives **`Debug, Serialize, Deserialize`** (note: it does **NOT** derive `ToSchema` — this DTO predates strict OpenAPI coverage; do not add `ToSchema` just for this field, match the surrounding struct).

**Backward-compat convention — confirmed in-struct:** newer/optional fields carry **`#[serde(default)]`** (`id` l.598, `cap_planned_hours_to_expected` l.610, `created`/`deleted` l.627/629, `version` with `#[serde(rename="$version")]` l.631). For `committed_voluntary`, add:

```rust
#[serde(default)]
pub committed_voluntary: f32,
```

`#[serde(default)]` → `0.0` for any persisted/old payload missing the field, so old clients and old stored JSON deserialize cleanly. Then thread the field through **both** conversion impls in this file: `From<&service::…EmployeeWorkDetails> for EmployeeWorkDetailsTO` (l.636) and `From<&EmployeeWorkDetailsTO> for service::…EmployeeWorkDetails` (l.674).

**Reporting DTOs (only if the year-view surfaces a separate number):** `WeeklySummaryTO` (`rest-types/src/lib.rs:900`) derives `Clone, Debug, Serialize, Deserialize`. D-01 requirement #4 ("committed-Kapazität SEPARAT ausweisen") implies adding a new field here (e.g. `committed_voluntary_hours: f32`) plus its source field on `service::booking_information::WeeklySummary` and the `From` impl at l.918. Use `#[serde(default)]` here too for forward/back compat. The computation lives in `service_impl/src/booking_information.rs:95` (`get_weekly_summary`) where `overall_available_hours = volunteer_hours + paid_hours` (l.197) — this is the formula the requirement modifies.

---

## 5. Reporting Formula + Snapshot Version (the load-bearing logic change)

The "no double-counting" rule lands in two places — verified:
- **Per-employee yearly report** `service_impl/src/reporting.rs`: `apply_weekly_cap(cap_active, raw_shiftplan_hours, expected_hours)` (defined ~l.96, called l.266 and l.852) returns `(capped_shiftplan_hours, auto_volunteer_hours)`. `volunteer_hours = manual_volunteer_hours + auto_volunteer_hours` (l.367, l.854). The D-01 formula `Überschuss = max(0, actual_volunteer − committed_voluntary)` and `verfügbar = expected + committed_voluntary` gets woven in around these sites. **Gate: only for `cap_planned_hours_to_expected = true`** (`.any(|wh| wh.cap_planned_hours_to_expected)` at l.265/l.814 is the existing flag check to reuse).
- **Weekly year-overview** `service_impl/src/booking_information.rs:get_weekly_summary` (l.95): `volunteer_hours` (l.141) and `overall_available_hours` (l.197) are where the committed capacity must be added as a separately-reported value.

**Snapshot bump is MANDATORY.** Per CLAUDE.md "Billing Period Snapshot Schema Versioning": because the input set and computation of the volunteer/capacity value changes, bump **`CURRENT_SNAPSHOT_SCHEMA_VERSION`** in `service_impl/src/billing_period_report.rs:74` from `7` → `8`. (It is read at write time on l.338.)

---

## 6. Frontend (Dioxus) Touchpoints — no new deps

`shifty-dioxus/` consumes only the shared `rest-types` crate + reqwest; **no new frontend dependency.** Component-Service-State pattern (per frontend CLAUDE.md). Touch points for `committed_voluntary` on the work-details record:

| File | Change |
|------|--------|
| `shifty-dioxus/src/state/employee_work_details.rs` | add `committed_voluntary: f32` to `EmployeeWorkDetails` (l.43), to `blank_standard` (l.72), and to **both** `TryFrom<&EmployeeWorkDetailsTO>` (l.145) and `TryFrom<&EmployeeWorkDetails> for EmployeeWorkDetailsTO` (l.185) |
| `shifty-dioxus/src/component/employee_work_details_form.rs` | add an input field (HR edit mask) — note datepicker caveat does NOT apply (this is a numeric input, not `<input type=date>`) |
| `shifty-dioxus/src/service/employee_work_details.rs` | passes the model through; usually no logic change |

For the **year-overview (`weekly_overview`)** separate display (D-01 #4):

| File | Change |
|------|--------|
| `shifty-dioxus/src/state/weekly_overview.rs` (`WeeklySummary`, l.11) | add a `committed_voluntary_hours` field + map it in `From<&WeeklySummaryTO>` (l.29) |
| `shifty-dioxus/src/page/weekly_overview.rs` | render it as a SEPARATE column/figure (not folded into `paid`/`volunteer`) |
| `shifty-dioxus/src/api.rs` (`get_weekly_overview` l.915) / `loader.rs` (l.615/628) | no change beyond the auto-updated DTO |

**i18n:** any new label MUST be added to all three locales (En, De, Cs) in `shifty-dioxus/src/i18n/`.

**WASM build gate:** after changes, `cargo build --target wasm32-unknown-unknown` from `shifty-dioxus/` must stay green (the single-`rest-types` consolidation means a backend DTO change breaks the frontend compile if not mirrored — that's the intended safety net).

---

## What NOT to Add / Do (explicit do-not list)

| Avoid | Why | Instead |
|-------|-----|---------|
| Any new Cargo dependency (backend or frontend) | `f32` field needs nothing new; sqlx/serde/utoipa/reqwest already present | Reuse existing crates |
| A `logical_id` column on `employee_work_details` | This entity versions by from/to date range, not logical-id rotation | Put `committed_voluntary` on the existing versioned row |
| A new table / join table for the pledge | Decoupled-but-co-versioned per D-01 means same row | Single `ADD COLUMN` |
| An invariant `committed >= expected` or a derived/subtractive read | Explicitly the rejected Variante A | Variante B: independent additive column |
| `sqlx database reset` | Destructive, dev DB unrecoverable; not needed for an additive migration | `sqlx migrate run` (additive) |
| Adding the field to the weekday booleans / `has_day_of_week` logic | Those model which days are worked, unrelated to capacity | Leave weekday logic untouched |
| Adding `ToSchema` to `EmployeeWorkDetailsTO` | Surrounding struct deliberately lacks it; not in scope | Match existing derive set (`Serialize, Deserialize`) |
| New DI deps on `EmployeeWorkDetailsService` | It's a Basic Service; adding domain-service deps breaks the tier convention | Keep dependency set unchanged |
| Forgetting the snapshot bump | Drift would be indistinguishable from a data bug | Bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` 7→8 |
| A down/rollback migration | Project migrations are forward-only; `NOT NULL DEFAULT 0` makes old rows valid | Forward-only `ADD COLUMN … DEFAULT 0` |

---

## Existing Stack (unchanged — for context)

| Layer | Technology | Notes |
|-------|-----------|-------|
| Backend web | Axum + utoipa | EWD routes in `rest/src/employee_work_details.rs` (POST/PUT/DELETE/GET) — unchanged shape |
| Persistence | SQLite via `sqlx` (compile-time checked, offline `.sqlx/` cache) | `query!`/`query_as!` macros |
| DI | `gen_service_impl!` macro | per-service dependency declaration |
| Shared DTOs | single `rest-types` crate (consolidated v1.2) | consumed by backend + frontend |
| Frontend | Dioxus 0.6.3 (WASM), Tailwind, reqwest | dx-CLI pinned to 0.6.x in `flake.nix` — do NOT use 0.7.x |
| Dev shell | `nix develop` → provides `sqlx-cli`, `sqlite` | `shell.nix` is broken; use the flake |
| VCS | jj (co-located git) | commit via jj, never `git commit` |

---

## Sources

- Repo files at `HEAD` (commit `5ade710`), read directly — HIGH confidence:
  - `service/src/employee_work_details.rs`, `dao/src/employee_work_details.rs`, `dao_impl_sqlite/src/employee_work_details.rs`, `service_impl/src/employee_work_details.rs`
  - `rest-types/src/lib.rs` (l.596-705, l.900-944)
  - `service_impl/src/reporting.rs`, `service_impl/src/booking_information.rs`, `service_impl/src/billing_period_report.rs:74`
  - `migrations/sqlite/20260426120000_add-cap-flag-to-employee-work-details.sql`, `…20251029192107_add-column-is-dynamic-…sql`
  - `shifty-dioxus/src/state/{employee_work_details,weekly_overview}.rs`, `shifty-dioxus/src/{api,loader}.rs`
  - `flake.nix:197-198` (sqlx-cli, sqlite), `env.example` (DATABASE_URL)
- Project memory: `reference_local_dev_commands` (nix develop / migrate run vs reset), `feedback_destructive_db_ops`, `project_frontend_dx_version_pin`, `feedback_service_tier_convention` — HIGH confidence (user-confirmed conventions)
- `.planning/PROJECT.md` v1.4 section + `todos/pending/2026-06-22-committed-voluntary-capacity-jahresansicht.md` (D-01 Variante B) — requirements source

---
*Stack research for: v1.4 Committed Voluntary Capacity (brownfield field-add)*
*Researched: 2026-06-22*
