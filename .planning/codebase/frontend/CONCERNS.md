# Codebase Concerns

**Analysis Date:** 2026-05-07

---

## 1. CRITICAL — `rest-types` is a Drifted Fork (Top-Priority Concern)

The frontend ships its own private copy of the `rest-types` crate that has
diverged significantly from the backend's source-of-truth crate. There is
**no compile-time link** between the two — plan discipline (developers
remembering to copy changes across both crates) is the only mechanism
keeping them in sync.

### Evidence

| Location | Version | Lines |
|----------|---------|-------|
| `rest-types/src/lib.rs` (frontend, drifted fork) | `1.0.5-dev` | 1468 |
| Backend `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest-types/src/lib.rs` (truth) | `1.13.0-dev` | 2041 |

The frontend's outer crate (`shifty-dioxus`) is at `1.13.0-dev`, but its
internal `rest-types/Cargo.toml` is still pinned at `1.0.5-dev`. The
backend's `rest-types/Cargo.toml` is at `1.13.0-dev`. The lag covers
**twelve minor releases** of TOs that the frontend cannot see.

### Why This Is a Real Risk, Not a Cosmetic Concern

- **Wire-shape ignorance:** The frontend deserialises responses into its
  own `XxxTO` structs. Any new field on the backend that lacks
  `#[serde(default)]` will fail to deserialise the moment a real backend
  (running the new schema) talks to a stale frontend. Conversely, fields
  present on the frontend but not on the backend silently round-trip to
  default values.
- **Variant blindness:** Newly added enum variants (e.g. backend
  `WarningTO::PaidEmployeeLimitExceeded`) cannot exist in the frontend's
  enum — serde will reject the JSON tag.
- **Plan discipline is fragile:** Every backend PR that touches a TO
  needs a matching, manually-curated change in the frontend fork. Nothing
  in CI fails when this is forgotten.

### Concrete Drift Inventory

The following items exist in
`/home/neosam/programming/rust/projects/shifty/shifty-backend/rest-types/src/lib.rs`
but are **entirely missing** from the frontend's
`rest-types/src/lib.rs`. Each line reference points to the backend file.

#### A. Missing TO Structs / Enums (whole types absent)

| Backend type | Backend lines | Used by backend feature |
|--------------|---------------|-------------------------|
| `ShiftplanAssignmentTO` | 1487-1496 | sales-person ↔ shiftplan multi-assignment (Phase frontend serialises a parallel `ShiftplanAssignment` struct in `src/state/user_management.rs:8-12` instead of re-using the TO) |
| `ToggleTO` | 1498-1526 | feature-flag service surface |
| `ToggleGroupTO` | 1528-1553 | feature-flag service surface |
| `ImpersonateTO` | 1555-1560 | admin impersonation flow |
| `AbsenceCategoryTO` | 1566-1592 | Phase 1 range-based absence domain |
| `AbsencePeriodTO` | 1594-1646 | Phase 1 range-based absence domain |
| `WarningTO` (5-variant tag-enum) | 1670-1781 | Phase 3 cross-source warning surface (BOOK-01/BOOK-02) and Phase 5 paid-employee-limit warning |
| `UnavailabilityMarkerTO` (3-variant tag-enum) | 1785-1823 | Phase 3 per-sales-person shiftplan day marker |
| `BookingCreateResultTO` | 1825-1840 | wrapper for `POST /shiftplan-edit/booking` |
| `CopyWeekResultTO` | 1842-1857 | wrapper for `POST /shiftplan-edit/copy-week` |
| `AbsencePeriodCreateResultTO` | 1859-1875 | wrapper for `POST /absence-period`, `PATCH /absence-period/{id}` |
| `CutoverGateDriftRowTO` | 1885-1896 | Phase 4 cutover gate report |
| `CutoverGateDriftReportTO` | 1898-1908 | Phase 4 cutover gate report |
| `CutoverRunResultTO` | 1910-1922 | Phase 4 cutover run result |
| `CutoverProfileBucketTO` | 1937-1949 | Phase 4 production-data profile |
| `CutoverProfileTO` | 1954-1963 | Phase 4 production-data profile |
| `ExtraHoursCategoryDeprecatedErrorTO` | 1926-1934 | HTTP-403 body shape after cutover flag is on |

#### B. Missing Fields on Existing TOs

| Backend type | Missing field | Backend lines | Purpose |
|--------------|---------------|---------------|---------|
| `SlotTO` | `max_paid_employees: Option<u8>` | 316-321 | Phase 5 D-10: optional cap on paid employees per slot/week (`None` = no limit). |
| `ShiftplanSlotTO` | `current_paid_count: u8` | 980-986 | Phase 5 D-09: live count of paid bookings in the slot for the view-week. Drives D-08 limit warning. |
| `ShiftplanDayTO` | `unavailable: Option<UnavailabilityMarkerTO>` | 993-997 | Phase 3 per-sales-person view marker (only set by `get_shiftplan_*_for_sales_person` endpoints). |
| `BillingPeriodTO` | `snapshot_schema_version: u32` | 1311 | Snapshot-schema versioning (see `service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION` and `shifty-backend/CLAUDE.md` invariants). Validators downstream use it to distinguish drift caused by schema bumps from real bugs. |

#### C. Missing Variants on Existing Enums

The frontend's `ExtraHoursReportCategoryTO` and `ExtraHoursCategoryTO` do
have `UnpaidLeave` and `VolunteerWork` (see `rest-types/src/lib.rs:386,
387, 671, 672`), so the v1.1 categories are present on the wire. However,
**the `From` impls in the frontend fork are stale and do not exhaustively
match the backend variants**:

- Frontend `From<&service::reporting::ExtraHoursReportCategory>` at
  `rest-types/src/lib.rs:391-403` does **not** map `UnpaidLeave` /
  `VolunteerWork` — these arms are missing entirely. This is dead code in
  practice (the `service-impl` feature is gated off by the
  frontend's `Cargo.toml`, which keeps `default = []`), but it is
  misleading and will silently break the day someone enables
  `service-impl` to bring helpers across.

#### D. Stale `From<&service::...>` Impls (Dead Wire-Cruft)

`rest-types/src/lib.rs` (frontend) carries `From<&service::...>` impls
that are unreachable in the Wasm build because the frontend's
`Cargo.toml` (`rest-types/Cargo.toml:9`) sets `default = []` and the
frontend never enables `service-impl`. The `service` crate isn't even on
the frontend's dependency graph. These impls are a maintenance liability
because:

- They silently lie about what fields are mapped (e.g. frontend
  `ShortEmployeeReportTO::from()` at lines 348-358 omits `volunteer_hours`).
- They keep the file ~600 lines longer than the actually-needed
  serde-only surface.
- They will not compile if anyone ever flips the feature on, because
  several `service::` paths (e.g. `service::shiftplan_catalog::Shiftplan`
  used at backend line 27) don't exist in the frontend's vendor tree.

#### E. Stale TODO Comments Inside the Fork

- `rest-types/src/lib.rs:1309` — `template_engine: TemplateEngineTO::Tera, // TODO: map from service type`
- `rest-types/src/lib.rs:1332` — `// template_engine: TODO map from TemplateEngineTO`

  Both belong to the dead `service-impl`-gated impls and are never
  exercised, but they encode the wrong behaviour (silently losing the
  user's `MiniJinja` choice if someone ever flips the feature).

### Drift-Detection Gap

There is no automated check for any of the above. A `diff -u
backend/rest-types/src/lib.rs frontend/rest-types/src/lib.rs` returns
hundreds of lines of difference that grow with every backend release.
There is no script in `shifty-stuff/` or CI that compares the two crates,
nor is there a workspace constraint forcing them to share source.

### Fix Approach

Two ordered options, simplest first:

1. **Short-term (recommended next step):** Add a CI script that diffs the
   two `lib.rs` files (or, better, compiles a "compatibility" probe crate
   that asserts each frontend TO can serde-roundtrip-deserialise the JSON
   shape produced by the backend's TO with the same name) and fails the
   pipeline on drift.
2. **Long-term:** Make the frontend depend directly on
   `shifty-backend/rest-types` via a path or git dependency, with
   `default-features = false` so the `service-impl`-gated code does not
   pull in the service crate. This eliminates the fork entirely. The
   risk is the wasm build's reluctance to compile certain time/uuid
   feature combinations — but the dependency surfaces match (both are
   `serde 1`, `time 0.3.41`, `uuid 1.x`, `utoipa 5`), so this is
   tractable.

Until one of these lands, the `rest-types/` fork is the single highest-
priority technical-debt item in the frontend.

---

## 2. Pervasive `unwrap()` / `expect()` in Hot UI Paths

**Issue:** ~194 hits of `unwrap()` or `expect()` across `src/`. Most are
on browser-API plumbing (`web_sys::window()`, time-format parsing) that
realistically cannot fail — but several sit on user-driven branches and
will panic the WASM module on bad input.

**High-risk examples:**
- `src/page/shiftplan.rs:650` —
  `let slot_id: Uuid = slot_id.unwrap().parse().unwrap();` — chained unwrap
  on a parsed event-data Uuid. Two failure modes (missing data,
  un-parseable Uuid) both panic.
- `src/page/shiftplan.rs:662, 803` — same pattern on event-data parsing.
- `src/page/sales_person_details.rs:35` —
  `Uuid::parse_str(&props.sales_person_id).unwrap()` — panics if a stale
  router URL ever carries a non-Uuid id.
- `src/page/shiftplan.rs:117-118, 450, 481` — `format!`-style time
  formatting unwraps. The format strings are static, so realistically
  safe, but the panic-on-failure behaviour is brittle if the format
  string is ever edited.
- `src/component/tooltip.rs:11, 18, 20` — `window().unwrap().inner_width()
  .unwrap().as_f64().unwrap()` triple-chain. Theoretically safe in a
  browser, but not in any non-browser test harness.

**Files:** see grep output for the full list — concentrated in
`src/page/shiftplan.rs`, `src/page/my_shifts.rs`,
`src/component/base_components.rs`, `src/component/tooltip.rs`,
`src/api.rs` (lines 37-45 — config-load expects).

**Impact:** A panic in WASM unwinds the entire SPA — the user sees a
blank screen and must reload. There is no error-boundary equivalent.

**Fix approach:** Audit the user-data-driven unwraps first
(`src/page/shiftplan.rs:650, 662, 803`,
`src/page/sales_person_details.rs:35`,
`src/page/my_shifts.rs:329-330`), convert to `Result` propagation or
graceful skip-with-log. The browser-plumbing unwraps are lower priority.

---

## 3. Monolithic `src/api.rs` (1269 lines, ~50 endpoints)

**Issue:** `src/api.rs` is a flat module with one async function per REST
endpoint. There is no domain grouping (booking/sales-person/reporting/
billing-period live side by side), no shared error-handling, no shared
URL builder.

**Files:** `src/api.rs` (1269 lines).

**Symptoms:**
- Each function open-codes `format!("{}/path/...", config.backend, ...)`
  — refactoring the URL prefix means touching every function.
- Each function repeats the
  `let response = reqwest::get(url).await?; response.error_for_status_ref()?;`
  pattern. The retry/error-mapping policy is duplicated ~50 times.
- The `list_user_invitations` function at lines 1127-1171 hand-rolls
  serde-json parsing inside the function body to work around a deser
  issue (the comment at line 1167 says
  `// TODO: Find a better way to convert serde error to reqwest error`)
  and silently swallows the error by returning `Ok(Rc::new([]))` on
  failure (line 1168) — this hides server-side problems from the user.
- Logging is `info!`-only (155 of 155 log lines in `api.rs`); failures
  log nothing because `?` discards the error before any log call.

**Impact:** Adding a new endpoint means copy-paste-edit, and forgetting
to add `error_for_status_ref()?` means 500 responses parse as garbage.
The silent-failure case in `list_user_invitations` is a bug magnet.

**Fix approach:** Split `api.rs` per domain (one file per backend
sub-router), introduce a tiny helper trait
`async fn fetch_json<T: DeserializeOwned>(client, url) -> Result<T, ApiError>`
that centralises `error_for_status` + JSON-parse + error-context logging,
and replace the silent `Ok(Rc::new([]))` branch with a propagated error.

---

## 4. Large Component Files (Cohesion / Maintainability)

**Issue:** Several component/page files are over 1000 lines and mix
state, view, and event handlers in one module.

| File | Lines |
|------|-------|
| `src/component/week_view.rs` | 1631 |
| `src/page/shiftplan.rs` | 1377 |
| `src/component/top_bar.rs` | 1166 |
| `src/component/employee_view.rs` | 997 |
| `src/component/working_hours_mini_overview.rs` | 894 |
| `src/loader.rs` | 865 |
| `src/page/user_management.rs` | 840 |
| `src/i18n/de.rs` | 687 |
| `src/component/dialog.rs` | 687 |

**Impact:** PRs that touch these files conflict often, code-review fatigue,
slow type-check feedback in `dx serve`.

**Fix approach:** The atom/molecule split started in `src/component/atoms`
and `src/component/form` is the right pattern. Apply it to `week_view.rs`
(slot row → atom, day column → molecule, week grid → organism) and
`shiftplan.rs` (the per-day editor body is a strong split candidate).

---

## 5. Logging is `info!`-Only and Verbose; No `error!` on Failures

**Issue:** Both `src/api.rs` and `src/loader.rs` import only `tracing::info`
(checked by grep). Every successful path logs at INFO; every error path
logs nothing because `?` returns before any log call. There are no
`warn!` / `error!` calls in the API layer.

**Files:**
- `src/api.rs:12` — `use tracing::info;` (only `info!` used)
- `src/loader.rs:7` — `use tracing::info;`

**Impact:** Production console.log noise is high (every fetch logs ~3
INFO lines), while real failures (network errors, 500 responses) leave
no trace. Diagnosing a user-reported issue from browser devtools is
harder than necessary.

**Fix approach:** Demote routine fetch INFO logs to `debug!`, add
`error!` arms in any callers of API functions that handle the `Err`
branch, and surface a user-visible error toast (the dialog atom in
`src/component/dialog.rs` already supports an error variant).

---

## 6. Accessibility Gaps

**Issue:** Aria/role usage is sparse (~36 hits across all components and
pages combined). Two `<img>` tags are missing `alt` attributes.

**Files:**
- `src/page/home.rs:33` — `img { src: asset!("/assets/shifty.webp") }`
  (no `alt`)
- `src/page/not_authenticated.rs:19` — same image, no `alt`

**Impact:** Screen-reader users get the file URL read aloud. Decorative
images should have `alt=""`; informational ones should describe the
brand mark.

**Fix approach:** Add `alt: ""` (decorative) on both. Audit the rest of
the component tree for buttons rendered as `div onclick:` (there are
many — every `onclick:` hit in non-interactive elements is a tab-order
and screen-reader bug).

---

## 7. WASM-Specific Performance Concerns

**Issue:** Several patterns can hurt WASM bundle size or runtime cost.

- **`Arc<str>` and `Rc<[T]>` everywhere on the wire** — these decode
  fine, but every `clone()` on them in component-render paths is a
  reference-count touch. Combined with Dioxus's virtual-DOM diffing,
  hot lists (week-view, shiftplan grid) re-clone these per render.
  See `src/component/week_view.rs` and
  `src/component/employee_view.rs` for the worst offenders.
- **No code-splitting:** Every page is in the same WASM bundle.
  `cargo build --release` produces a single `.wasm` blob that ships on
  first load. Pages like `text_template_management.rs` and
  `billing_period_details.rs` (admin-only) contribute to the
  initial-paint cost of the public landing page.
- **`reqwest` over fetch** — `reqwest = "0.12.15"` with `json` feature
  pulls a non-trivial subset of the std-net adapter. A direct
  `web_sys::fetch` wrapper (or `gloo-net`) would reduce bundle size,
  though at the cost of API ergonomics.

**Files:** all of `src/component/`, `Cargo.toml:14`.

**Impact:** First-paint TTI is higher than necessary; admin pages are
loaded eagerly even for non-admin users.

**Fix approach:** Profile with `wasm-opt -Os` first; if bundle size is
the dominant concern, evaluate `gloo-net` swap; for code-splitting the
Dioxus 0.6.x surface needs a manual lazy-route pattern (no
out-of-the-box support yet).

---

## 8. Test Coverage Gaps

**Issue:** Tests live in `src/tests/` and cover serde round-trips,
state transitions, error mapping, and a handful of SSR HTML smokes.
What is absent:

- **No end-to-end browser tests.** All HTML assertions are SSR via
  `dioxus-ssr`, which renders without browser semantics — `onclick`
  handlers, focus/blur, IntersectionObserver, etc. are not exercised.
- **No tests for `src/api.rs`.** `mockito` is in `[dev-dependencies]`
  but no test file uses it (grep `mockito` returns the Cargo.toml line
  only).
- **No tests for the `loader.rs` slot-filtering logic** beyond the
  existing handful — the `holiday vs short-day vs slot.to >
  special_day.time_of_day` branch at lines 122-129 is the kind of
  off-by-one-prone code that benefits most from a test matrix.

**Files:** `src/tests/`, `Cargo.toml` (dev-dependencies).

**Impact:** Regressions in slot filtering or in the booking
event-handler chain are caught only by manual smoke testing.

**Fix approach:** Add `mockito`-driven tests for the most-stable API
functions (e.g. `get_short_reports`, `get_shiftplan_week`) before
restructuring `api.rs`. Add a test matrix for the `loader.rs` slot
filter covering holiday/short-day/regular crosses.

---

## 9. Stale `panic!` Branches in Identifier Conversion

**Issue:** Several enum-from-string conversions panic on unknown input
rather than returning `Result`.

- `src/state/employee.rs:89` — `panic!("Unknown working hours category: {}", identifier)`
- `src/state/employee.rs:151` — `panic!(...)` on unknown ExtraHours
  category
- `src/state/shiftplan.rs:59` — `panic!("Invalid weekday number: {}", num)`

**Impact:** Any backend that ever ships a new category (and an outdated
frontend talks to it) panics the SPA on first deserialisation. This is
how the `rest-types` drift surfaces for the user — not as a fail-soft
fallback but as a blank screen.

**Fix approach:** Convert the `panic!` arms to a fall-back variant
(e.g. `WorkingHoursCategory::Unknown(identifier)`) that the UI can
gracefully render as "(unsupported category)" while keeping the rest of
the response usable.

---

## 10. Cargo / Dependency Hygiene

**Issue:** Some minor concerns from `Cargo.toml`:

- `Cargo.toml:24` — `wasm-bindgen = "0.2.97"` is pinned older than
  what `web-sys 0.3.77` recommends; mismatch warnings can appear during
  build.
- The frontend's `rest-types/Cargo.toml` keeps `default = []` and gates
  every `service` impl on `service-impl` — yet the file references
  `service` in 50+ places. This is fine while the feature stays off,
  but it inflates the file and confuses code-search (every
  `From<&service::...>` impl is technically dead code in this crate).

**Files:** `Cargo.toml`, `rest-types/Cargo.toml`,
`rest-types/src/lib.rs`.

**Fix approach:** Upgrade `wasm-bindgen` to match `web-sys 0.3.77`.
Decision on the `service-impl` impls is tied to the rest-types-fork
fix in concern §1 — once the frontend depends on the backend's crate
directly, the feature gate becomes meaningful again.

---

*Concerns audit: 2026-05-07*
