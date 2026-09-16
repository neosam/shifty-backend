# Project Research Summary

**Project:** Shifty Backend
**Domain:** Rust/Axum/SQLx layered backend + Dioxus/WASM frontend — internal shift-planning/HR admin tool
**Researched:** 2026-09-16
**Confidence:** HIGH (all four researchers grounded findings in code/migrations/docs actually read in this repo; no external ecosystem research was needed because both features are "what does this repo already do")

## Executive Summary

Milestone v2.7 reads as a small two-line description ("merge Person, add a Planungs-Soll field") but all four researchers independently converge on the same conclusion: **it is larger than its description implies**, primarily because dissolving `sales_person_user` doesn't just move data — it removes a live authorization check (`find_sales_person_by_user_id`, used for self-vs-HR permission gating) and a database-enforced 1:1 invariant (`UNIQUE(sales_person_id)` + `UNIQUE(user_id)`) that must both be rebuilt by hand, not just migrated. No new dependencies, ORMs, or crates are needed anywhere — every technique (nullable `ALTER TABLE ADD COLUMN`, raw-SQL backfill, `randomblob(16)`, additive `#[serde(default)]` DTO fields) has a direct working precedent already in this codebase. That is the good news: technically this is low-risk, well-trodden ground.

The recommended approach is data-model-first, tier-respecting, in four stages: (1) migration + idempotent, soft-delete-inclusive backfill with `UNIQUE(person_id)` constraints rebuilt on both new FK columns; (2) a new Basic-tier `PersonService`/`PersonDao` that does NOT depend on `SalesPersonService` or `UserService` (to avoid a DI cycle through `PermissionService`, which everything depends on); (3) the DAO-internal rewrite of `sales_person_user`-backed queries — six methods, one of which (`find_sales_person_by_user_id`) is the authorization check and must be tested before/after, not deferred; (4) a Business-Logic-tier read aggregate (e.g. `PersonDirectoryService`) for the merged list/detail UI, following the v2.5 "Fat Backend, Thin Client" precedent (`SalesPersonAbsenceTO`/`WeeklySummaryTO`) with backend-computed display names and API-level `Option`-redaction for section-level role gating (the `VoluntaryStatsService` None-redaction pattern) rather than 403s. Planungs-Soll is architecturally independent of all of this — it only touches `reporting.rs`/`rest-types`/one frontend file — and can be sequenced early or in parallel as the smaller, better-understood change.

The single biggest risk, flagged explicitly by PITFALLS and echoed by ARCHITECTURE, is repeating v2.6's most expensive mistake (VOL-ACCT-03): writing a requirement shaped as "X stays neutral/unchanged" that is trivially easy to keep green while quietly blocking the actual feature. Two exact shapes of this trap exist in v2.7 — a person-merge requirement broad enough to freeze the self-permission-check query, and a Planungs-Soll requirement that could be misread as "must equal `expected_hours`" or "must equal `dynamic_hours`" (both wrong). This must be caught at requirements-drafting time, not at milestone audit.

## Key Findings

### Recommended Stack

No new packages needed for either feature (STACK.md, HIGH confidence). `sqlx 0.8.2`, `uuid 1.8`, `serde 1.0`, `utoipa 5` already cover everything. Nullable FK columns are added via plain `ALTER TABLE ADD COLUMN ... REFERENCES` (precedent: `slot.shiftplan_id`, `user_invitation.session_id`) — no table rebuild needed because `person_id` is permanently nullable (not a transitional state; two of the three "shapes" never get a link). The `person` backfill belongs in the "simple SQL-in-migration" camp, not the heavier Rust-cutover-service + quarantine-table camp used for the `extra_hours`→`absence_period` migration, because the old `sales_person_user`'s double `UNIQUE` constraint already proves every pair is unambiguous — no fuzzy matching, no manual-review case. Use `randomblob(16)` for new `person.id` values in the migration, not the dead/unused `V_UUID_V4` view. `planning_target_hours` follows the exact additive-field recipe already used for `has_pending_rebooking`/`pending_rebooking_id` on `ShortEmployeeReportTO` (`#[serde(default)]`, computed in the `From` impl). No `CURRENT_SNAPSHOT_SCHEMA_VERSION` bump applies to either feature (confirmed against `.planning/PROJECT.md`'s own decision log — stays at 13).

**Core technologies (all already present, zero installs):**
- `sqlx 0.8.2` + `sqlx::migrate!` — migration mechanism, unchanged; a new `PersonDao` follows existing `query!`/`query_as!` + manual `Uuid::from_slice`/`.as_bytes()` conversion style (this repo does NOT use sqlx's native `uuid` feature — stay consistent).
- `uuid 1.8` — `Uuid::new_v4()` for runtime `person` creation; `randomblob(16)` for migration-time backfill IDs.
- `serde` + `#[serde(default)]` — additive-field wire-compat convention, reused verbatim for `planning_target_hours`.
- `utoipa 5` — `ToSchema` already covers `f32`/`Option<Uuid>`, no new feature flags.

### Expected Features

FEATURES.md (MEDIUM confidence — no live web-search tooling available, so external precedents are vendor-docs/issue-tracker-sourced; current-app claims are HIGH, verified by reading code directly) frames this as a merged identity-admin screen for a handful of internal staff — small-scale, not enterprise identity management.

**Must have (table stakes):**
- Single flat list, one row per entry, with a shape badge/column ("Login only" / "Roster only" / "Login + Roster") replacing today's two-tab split (`UserMgmtTab::SalesPersons`/`Users`).
- Backend-computed display name per entry (Fat Backend, Thin Client — already flagged as an open DTO question in the seed).
- Filter by shape; search spanning whichever name(s) an entry has.
- A real picker/search for linking existing unlinked users/sales-persons, replacing today's raw-`TextInput`-typed-ID flow (`ConnectUserAccount`/`UpdateSalesPersonUser` in `sales_person_details.rs:189-222`) — explicitly the single biggest UX upgrade.
- Explicit, separately-labelled "link existing" vs "create new" actions (not one ambiguous field) — Keycloak's own admin-UI backlog names this exact ambiguity as a bug class to avoid.
- Confirmation copy before unlink/delete that states what disappears, and that differs by which half is being deleted (SalesPerson-side deletion risk >> User-side, given the 12-table fan-out onto `sales_person.id` including write-once billing snapshots).
- Section-level (not page-level) role gating on the merged detail view — locked as "must decide," not optional, per the seed's own open question.

**Should have / correctly deferred (not this milestone, explicitly named to avoid scope creep):**
- Folding `/employees/:id` (HR report view) into the same entry — seed defers pending real usage feedback.
- Bulk actions on the roster — only relevant if roster size grows.

**Defer indefinitely (v2+, likely never at this shop's scale):**
- Automatic fuzzy link-suggestion, full merge/audit-history UI, bulk-import wizard, multi-account linking (explicitly excluded — Variant A keeps the 1:1 shape), profile-completeness gamification, `person`-level profile fields (email/phone/DOB — seed keeps `person` identity-only by design).

### Architecture Approach

ARCHITECTURE.md (HIGH confidence) recommends a tier-respecting build order: migration+backfill → `PersonDao`/`PersonService` (Basic tier, thin CRUD only — must NOT depend on `SalesPersonService` or `UserService`) → DAO-internal rewrite of the six `sales_person_user`-backed methods (service/REST contracts frozen, so this is a two-layer change, not an N-layer one, PROVIDED the contract is genuinely kept stable) → a new Business-Logic-tier read aggregate (`PersonDirectoryService` or similar) for the merged list/detail, consuming `PersonService` + `SalesPersonService` + user/role reads, one tier above everything it unions — mirroring `ShiftplanViewService`. Planungs-Soll is a separate vertical slice entirely inside `reporting.rs`/`rest-types`/one frontend file, with no `person` involvement, and is architecturally independent of the Person work.

**Major components:**
1. **`PersonDao`/`PersonService` (Basic tier)** — new `person` CRUD + link/unlink primitives only; does not know about `SalesPersonService` or `UserService` to avoid a DI cycle through `PermissionService`.
2. **`sales_person_user`-dissolution inside existing DAO methods** — six methods rewritten to join through `person_id`; `assign_to_user`/`discard_assigned_user` are the one place with genuinely new business logic (resolve-or-create person, reject on ambiguous double-link).
3. **New Business-Logic read aggregate (e.g. `PersonDirectoryService`)** — merged list/detail DTOs (`PersonEntryTO`, small and purpose-built like `SalesPersonAbsenceTO`, NOT a reuse of `SalesPersonTO`/`UserTO`), one aggregate GET endpoint replacing today's two-load pattern, section-level `Option`-redaction for role gating.
4. **Planungs-Soll computation in `reporting.rs`** — extends `weight_for_week`'s return with a structurally decoupled 5th value, applied at both `assemble_weeks` and `get_reports_for_all_employees` (two independently-duplicated call sites — both must be fixed together), added to `ShortEmployeeReport`/`ShortEmployeeReportTO` with `#[serde(default)]`.

### Critical Pitfalls

PITFALLS.md (HIGH confidence, grounded in this repo's own migrations, code, and `RETROSPECTIVE.md`) surfaces ten pitfalls; the five most load-bearing for phase-cut decisions:

1. **Backfill migration must be idempotent** — this repo's own precedent for comparable backfills (`absence_period_migration_source`) used an idempotency-key pattern specifically because a raw SQL script that partially fails and gets hand-re-run will create duplicate `person` rows. Guard every INSERT/UPDATE with `WHERE NOT EXISTS`/`WHERE person_id IS NULL`, test apply-twice for stable row counts.
2. **Backfill must include soft-deleted `sales_person` rows** — `sales_person.deleted` exists, `user` has no soft-delete at all; the everyday DAO reflex `WHERE deleted IS NULL` is wrong here because soft-deleted sales persons can still own write-once `billing_period_sales_person` history that later needs to resolve back to a `Person`.
3. **The dissolved link table's `UNIQUE(sales_person_id)` + `UNIQUE(user_id)` invariant must be rebuilt as `UNIQUE(person_id)` on both new FK columns** — otherwise nothing stops two different `sales_person` rows (or an unrelated user) from silently sharing one identity; needs a DB-level rejection test, not just an application check (which races under concurrency).
4. **`find_sales_person_by_user_id` is an authorization check, not just CRUD** (`service_impl/src/sales_person.rs:356-401`, backing self-vs-HR permission gating on sales-person mutation) — it must be rewritten in the *same* phase as the schema swap, with an explicit before/after "self can edit own record, cannot edit another's" test matrix. A sloppy nullable-`Option` comparison here can fail open (security) or fail closed (lockout).
5. **Neutrality-shaped requirements repeat the exact v2.6 VOL-ACCT-03 failure** — a requirement worded "person merge changes nothing" or "planning_target_hours never affects the balance" is trivially testable and permanently green even when it silently blocks the milestone's actual goal (freezing the self-permission query that must be rewritten; or being satisfied by literally reusing `expected_hours`/`dynamic_hours`, both of which are wrong per the underlying bug analysis). Must be caught in REQUIREMENTS.md drafting, before Phase 60 is planned — this is explicitly a pre-phase gate, not a phase deliverable.

Also flagged, lower-severity but real: new `/person`-prefixed REST routes need a same-commit `Dioxus.toml` `[[web.proxy]]` entry (3rd occurrence of this exact bug class in this repo if missed — v1.6, v1.8 both missed it); this milestone touches an unusually large number of Docs-Freshness-Gate trigger files at once (migrations, service/dao traits, `permission.rs`, `reporting.rs`) so the doc-diff check should run after every phase, not just at close; the merged UI's per-Ausprägung/per-role section visibility is DOM-rendering logic that Dioxus-WASM signal propagation makes unreliable to test headlessly — push decision logic into pure, unit-tested helpers and plan an explicit human-UAT pass up front.

## Convergence and Divergence Across Researchers

**Strong convergence (raises confidence):**
- **STACK and ARCHITECTURE independently agree** the `sales_person_user` dissolution is safe to do as a DAO-internal rewrite with the service/REST contract frozen — STACK from the "no new tooling needed, additive migration" angle, ARCHITECTURE from the "~50+ call sites into stable `SalesPersonService` trait methods are unaffected" angle (grep-verified across `absence.rs`, `extra_hours.rs`, `booking.rs`, `shiftplan.rs`, etc.).
- **ARCHITECTURE and PITFALLS independently flag the exact same three sharp edges**: the `find_sales_person_by_user_id` authorization query (ARCHITECTURE calls it "the one place that is NOT a pure pass-through"; PITFALLS names it Pitfall 4 with a security-mistake writeup), the lost `UNIQUE` invariant on link/unlink, and the need to compute Planungs-Soll via the *same* holiday/absence reduction pipeline as `expected_hours` rather than reusing `dynamic_hours` (both independently trace this through `weight_for_week`/`assemble_weeks` and land on the identical conclusion).
- **FEATURES and ARCHITECTURE independently converge** on backend-computed display name + section-level `Option`-redaction (not page-level 403) as the correct DTO shape — FEATURES from a UX-precedent angle (GitLab/Salesforce), ARCHITECTURE from a direct code-precedent angle (`VoluntaryStatsService`'s existing None-redaction pattern).
- **STACK and PITFALLS agree** that a lightweight raw-SQL backfill (not the heavier Rust-cutover-service pattern used for `absence_period`) is the right proportional choice, both citing the same `UNIQUE`-constraint-guarantees-unambiguity argument — but PITFALLS adds the sharper caveat STACK doesn't: "structurally simple to migrate" is not "safe to run twice," so idempotency guards are still mandatory even without the heavier apparatus.

**Divergence / tension worth naming explicitly, not smoothing over:**
- **STACK's "Alternatives Considered" table treats the pure-SQL backfill as basically risk-free** ("zero risk" language for the `ALTER TABLE ADD COLUMN` step; the backfill section reads as low-ceremony). **PITFALLS treats the same backfill as the source of two of its ten pitfalls** (non-idempotency, soft-delete exclusion) and explicitly warns that "structurally simple" from STACK's framing does not imply "safe to run twice." Read STACK's stack-selection confidence (right tool, no new dependencies) as separate from PITFALLS' execution-discipline warning (still needs idempotency guards and soft-delete coverage) — these are not contradictory, but a roadmapper skimming only STACK could underestimate the backfill phase's care requirements that PITFALLS insists on.
- **ARCHITECTURE's suggested build order lists Planungs-Soll as sequenceable "in parallel or before/after" the Person work, calling it "the smaller, better-understood change."** PITFALLS' Pitfall 6 and Pitfall 7 push back on "better-understood" specifically — the todo's own history shows the "naheliegender Fix" (reuse `dynamic_hours`) is proven wrong, and the neutrality-requirement trap (Pitfall 7) is named as being just as live a risk for Planungs-Soll as for the Person merge. Treat "architecturally independent/parallelizable" (true) as distinct from "low-risk to get the requirement wording right" (not true) — these are different axes and both researchers are correct on their own axis.
- **FEATURES' MVP list scopes "Add-login/add-person actions + real picker" as P1**, explicitly requiring "Backend endpoint listing 'unlinked' candidates of the other kind." **This endpoint does not exist today and is not mentioned as in-scope by STACK or ARCHITECTURE's phase breakdowns** — see Unscoped Work below. This is a real gap between FEATURES' feature-completeness framing and the other three researchers' phase-structuring, not a disagreement about facts.

## Hard Constraints Carried Forward Verbatim

These must reach REQUIREMENTS.md and phase planning unmodified — they are not researcher opinions, they are code-verified facts with file:line citations.

- **`WorkingHoursPerSalesPerson.available_hours` must keep reading `expected_hours` (zeroed for dynamic contracts), never `planning_target_hours`.** `service_impl/src/booking_information.rs:641-651` and `:815-826` — this is the exact "Bezahlten-Vertrag" band the v2.5 ship notes locked as a regression guard (`.planning/PROJECT.md:275-276`: "unangetastet — Regression-Lock"). Single highest-risk regression site for the Planungs-Soll feature.
- **`find_sales_person_by_user_id` backs a live authorization check, not just CRUD.** `service_impl/src/sales_person.rs:356-401` resolves "is the current user allowed to touch this sales person because it's their own" via this DAO method (`dao_impl_sqlite/src/sales_person.rs:226-233`, joins through `sales_person_user`). Must be rewritten in the same phase as the schema swap, with an explicit before/after self-vs-not-self permission test matrix — not deferred as a "frontend follow-up."
- **`user` has no soft-delete column; `sales_person` does.** `sales_person.deleted TEXT` (`migrations/sqlite/20240506114107_add_sales_person.sql`) vs. `user` (`migrations/sqlite/20240426150045_user-roles.sql`) — hard-delete only. The backfill must include soft-deleted `sales_person` rows (they can still own write-once `billing_period_sales_person` history); `user` rows can't be soft-deleted so this asymmetry only cuts one way.
- **The 1:1 invariant currently enforced by `UNIQUE(sales_person_id)` + `UNIQUE(user_id)` on `sales_person_user`** (`migrations/sqlite/20240506114107_add_sales_person.sql:14-22`) **disappears when the table is dropped and must be rebuilt as `UNIQUE(person_id)` on both new FK columns**, plus a DB-level (not just application-level) rejection test for double-linking — application-level checks race under concurrent requests exactly the way the old table's `UNIQUE` would have prevented.
- **Two structurally different `expected_hours` fields exist in this codebase** — one on `EmployeeWorkDetails` (raw contract row, read by `voluntary_stats.rs`/`booking_information.rs:504,518,545,872` and unaffected either way) and one on `ShortEmployeeReport`/`GroupedReportHours` (the computed report aggregate, the one in scope for Planungs-Soll). Do not conflate these during implementation or review.
- **No `CURRENT_SNAPSHOT_SCHEMA_VERSION` bump applies to either v2.7 feature** — confirmed against `.planning/PROJECT.md`'s own decision log (stays at 13); Planungs-Soll is a display-only field computed from already-live logic and does not touch `billing_period_sales_person`/`BillingPeriodValueType`.

## Explicit Warning: Neutrality-Shaped Requirements

**This must reach REQUIREMENTS.md drafting as a named gate, not be buried in a pitfalls appendix.** PITFALLS.md identifies this as v2.6's single most expensive mistake (VOL-ACCT-03: a requirement that literally forced "balance-neutral" — the opposite of the milestone's goal — and survived three phases of clean, green verification because "stays neutral" is trivially easy to keep green forever, even when wrong). v2.7 has two requirement shapes that could repeat this exact failure:

1. A person-merge requirement phrased broadly as "linking/unlinking must not change any existing sales_person/user/auth behavior" — correct if scoped to unrelated behavior, but if it also freezes the self-permission-check query rewrite (a required change) or the display-name fallback logic, it blocks the very rewrite the migration requires.
2. A Planungs-Soll requirement phrased as "`planning_target_hours` must never affect the balance" — correct in intent, but if read/tested as "must equal `expected_hours`" or "must equal `dynamic_hours`," both readings are wrong (per the underlying bug's own root-cause analysis) yet both would pass a naive "value doesn't change existing balance tests" check.

**Action for requirements drafting:** any requirement worded "X stays unchanged"/"X is neutral with respect to Y" must carry an explicit one-line milestone-goal cross-check written into REQUIREMENTS.md itself before the first dependent phase is planned — e.g., "does this requirement, read literally, still allow a user to see a Planungs-Soll number at dynamic contracts?" Prefer requirements phrased as "computed via [specific named mechanism]" over bare "unchanged"/"neutral."

## Open Questions — Single Consolidated List

| # | Question | Status |
|---|---|---|
| 1 | Nullable `person_id` FK shape, "Variante A," names stay put (`sales_person.name`/`user.name`), no snapshot bump | **Settled by the seed** (`.planning/seeds/personenverwaltung-zusammenlegen.md`) — not reopened by any researcher |
| 2 | `person` is identity-only (no email/phone/DOB) for this milestone | **Settled by the seed** — explicit decision, revisit only when a concrete attribute is needed |
| 3 | `PersonService` is Basic tier, must not depend on `SalesPersonService`/`UserService`; merged-list aggregation lives in a new Business-Logic service one tier up | **Settled by ARCHITECTURE research**, consistent with `CLAUDE.md`'s own tier rule — recommend treating as decided unless discuss-phase surfaces a reason not to |
| 4 | Whether unlink-to-zero-children deletes the `person` row, garbage-collects it, or surfaces it in an admin view | **Needs a discuss-phase decision** — PITFALLS recommends "last unlink deletes the person row" as simplest/cheapest but flags this as a product decision belonging in that phase's CONTEXT, not left to the executor |
| 5 | Whether re-linking to an already-linked `sales_person`/`user` is rejected (409) vs. silently reassigned | **Needs a discuss-phase decision**, though PITFALLS strongly recommends reject-with-409 as the only safe default given the 12-table blast radius on `sales_person.id` |
| 6 | Whether `/employees/:id` (HR report view) folds into the merged entry this milestone or stays separate | **Settled as deferred** by the seed (open question explicitly deferred pending usage feedback) — do not scope into v2.7 |
| 7 | Section-level (not page-level) role gating mechanism/DTO shape for the merged detail view | **Needs a discuss-phase decision** — locked as "must exist," not optional, but the exact DTO shape (explicit `kind` enum vs. `Option`-populated fields as signal) is explicitly flagged uncertain by ARCHITECTURE |
| 8 | Whether the list DTO needs an explicit `kind: PersonEntryKindTO` enum or whether `Option`-populated fields are sufficient signal | **Needs a discuss-phase decision** — minor DTO-shape call, not a blocker |
| 9 | Exact formula for the "unconditional" Planungs-Soll raw value — does `get_reports_for_all_employees`'s reduction pipeline (`:1340-1420` region) apply the *same* reductions in the *same* order as `assemble_weeks` (`:1080-1101`)? | **Needs a line-by-line code check during planning** — the two loops are independently written and could have subtly diverged since the WOP-01 refactor split them |
| 10 | Whether `get_year`'s `ShortEmployeeReport` rows should also expose `planning_target_hours`, or whether that's out of scope since the annual view doesn't render this column today | **Needs a discuss-phase decision** — this research did not check `get_year`'s frontend consumer |
| 11 | Whether `hours_text_class`/`progress_bar_class` color thresholds and `total_target` in `working_hours_mini_overview.rs` get rebound from `expected_hours` to `planning_target_hours` | **Needs a discuss-phase decision per the todo's own framing** ("bewusste Entscheidung pro Stelle") — must NOT be a blanket find-and-replace |
| 12 | Whether strict RFC 4122 UUID conformance matters for `randomblob(16)`-generated `person.id` values in the migration (vs. the workspace's normal `Uuid::new_v4()`) | **Needs a discuss-phase decision**, though STACK assesses it as low-stakes — nothing in the DAO layer validates version/variant bits, only length |

## Unscoped Work Flagged by Research

These are real, load-bearing pieces of work that FEATURES/ARCHITECTURE/PITFALLS surface as necessary for the milestone's own stated goal but that do not appear accounted for in any researcher's phase-cut hypothesis as a named deliverable:

- **A backend endpoint listing "unlinked candidates of the other kind"** (e.g. "all users with no `person_id`" / "all sales_persons with no `person_id`") does not exist today. `SalesPersonDao::find_sales_person_by_user_id` finds an *existing* pairing but there is no query for *unlinked* rows. FEATURES.md names this explicitly as "new, small, backend work that unlocks replacing the raw-text-field linking flow" and calls out to the roadmapper that it is required-but-not-yet-scoped. Without it, the P1 "real picker" feature (replacing the raw `TextInput` flow, itself table-stakes per FEATURES) cannot be built.
- **Section-level role gating is a bigger permission-model change than it first looks.** Today `/user_details` is implicitly whole-page admin-territory and `/sales_person_details` is implicitly whole-page shiftplanner-territory; merging them means the *same* detail response must show/hide/disable individual sections per caller role. ARCHITECTURE recommends the API-level `Option`-redaction pattern (reusing `VoluntaryStatsService`'s precedent) but flags this should get "its own explicit design decision in discuss-phase rather than being assumed 'just render less.'" This is plumbing work beyond a simple DTO field addition and should be sized as such, not folded silently into the "merged detail view" phase's estimate.
- **A regression/compatibility test suite for the `sales_person_user` dissolution's authorization path** — an explicit before/after "self can edit own record, cannot edit another's" permission test matrix — is a named deliverable in PITFALLS but is not mentioned in STACK's or ARCHITECTURE's phase-sizing; without it, this is silent scope that will surface as unplanned work mid-phase.

## Implications for Roadmap

Based on combined research, a **four-phase structure**, data-model-first, tier-respecting, matches all four researchers' independent build-order recommendations (all converge on this shape; PITFALLS' provisional phase numbers Phase 60-63 are used below as a naming convention, not a commitment):

### Phase 60: Person data model + backfill
**Rationale:** Everything else depends on this; STACK/ARCHITECTURE/PITFALLS all place it first with no dissent. It is also the phase where the two highest-severity, easy-to-miss defects (non-idempotent backfill, soft-delete exclusion) must be caught, since they are invisible once the migration is marked applied.
**Delivers:** `person` table, nullable `person_id` FK columns with `UNIQUE(person_id)` constraints on both `sales_person` and `user`, an idempotent SQL-only backfill covering ALL `sales_person` rows regardless of `deleted` status and all `user` rows.
**Addresses:** Foundation for the entire Personenverwaltung feature set (FEATURES.md P1: "Person aggregate + backfill").
**Avoids:** Pitfall 1 (non-idempotent backfill), Pitfall 2 (soft-deleted rows excluded), Pitfall 3 (lost 1:1 invariant).
**Docs-Freshness-Gate trigger:** `docs/architecture/03-data-model.md` + `diagrams/db-schema-er.mmd`, same phase/PR.

### Phase 61: `PersonService` + `sales_person_user` dissolution + link/unlink service
**Rationale:** Builds the Basic-tier `PersonService` and rewrites the six DAO methods that currently join through `sales_person_user`, keeping the `SalesPersonService`/`UserService` public contracts frozen so the ~50+ downstream call sites are unaffected. This phase also owns the one place with real new business logic (resolve-or-create person, reject-on-ambiguous-link) and the security-critical authorization query rewrite.
**Delivers:** `PersonDao`/`PersonService` (Basic tier, DI-wired per the documented bottom-up order in `shifty_bin/src/main.rs`), rewritten `sales_person_user`-backed DAO queries joining through `person_id`, reject-with-409 on already-linked targets, the "unlinked candidates" endpoint(s) FEATURES flags as currently missing.
**Uses:** `sqlx` query rewrite pattern, `Arc<...>` DI convention already established for Basic services.
**Implements:** ARCHITECTURE's Basic-tier `PersonService` + DAO-internal-only schema swap.
**Avoids:** Pitfall 4 (missed self-permission-check rewrite — explicit before/after test matrix required), Pitfall 5 (link/unlink orphans/re-target ambiguity — reject-on-linked check), Pitfall 8 partially (if `/person` REST routes are exposed here, add the `Dioxus.toml` proxy line in the same commit).

### Phase 62: Personenverwaltung UI — merged list + detail
**Rationale:** Depends on Phase 61's aggregate-ready service layer; this is where the actual user-facing complaint ("extrem verwirrend") gets fixed. Section-visibility and role-gating decisions from the open-questions list must be resolved in this phase's discuss step, not discovered mid-implementation.
**Delivers:** One merged list at `/user_management/` (no tabs) with shape badge/filter and backend-computed display name; one merged detail component blending sections in/out by shape with section-level role gating; "add login"/"add person" actions backed by a real picker over the unlinked-candidates endpoint; shape-aware unlink/delete confirmation copy.
**Addresses:** FEATURES.md's full P1 list for the UI half of the milestone.
**Avoids:** Pitfall 5 (list-view derivation from `sales_person`/`user` outer-join, never from `person` directly — makes ghost rows invisible regardless of the zero-children decision), Pitfall 8 (Dioxus.toml proxy entry for any new `/person` route), Pitfall 10 (push section-visibility decision logic into pure, unit-tested helpers; plan an explicit human-UAT pass up front, not discovered at close).
**Note:** Section-level role gating here is flagged as unscoped-but-required work (see above) — size this phase accordingly, not as a thin UI-only phase.

### Phase 63: Planungs-Soll DTO field
**Rationale:** Architecturally independent of Phases 60-62 (no `person` involvement) — could in principle run earlier or in parallel, but sequencing it last keeps the milestone's riskiest requirements-wording trap (Pitfall 7) fresh in mind from Phase 60's own discuss-step precedent, and its smaller scope makes it a good buffer/closer.
**Delivers:** `planning_target_hours: f32` computed via the same holiday/absence reduction pipeline as `expected_hours` (never via `dynamic_hours`), added to `ShortEmployeeReport`/`ShortEmployeeReportTO` at both `assemble_weeks` and `get_reports_for_all_employees` call sites, with `#[serde(default)]` wire-compat and a byte-identity regression test (`planning_target_hours == expected_hours` for every non-dynamic fixture).
**Uses:** STACK's additive-DTO-field recipe (`has_pending_rebooking` precedent).
**Implements:** ARCHITECTURE's traced injection point in `weight_for_week`.
**Avoids:** Pitfall 6 (leak into balance/payroll, or silent consumer rebinding — each of `hours_text_class`/`progress_bar_class`/`total_target` treated as its own explicit per-site decision, not a blanket sweep), Pitfall 7 (neutrality-shaped requirement trap).

### Phase Ordering Rationale

- Data-model-first is not optional here — three of four researchers (STACK, ARCHITECTURE, PITFALLS) independently state the backfill's correctness proof (via the old `UNIQUE` constraints) is what makes the *rest* of the milestone safe; skipping ahead to UI work without the migration validated would mean building on an unverified foundation.
- Splitting "DAO dissolution" (Phase 61) from "UI" (Phase 62) follows the tier-respecting principle in `CLAUDE.md` and lets the security-critical authorization-query rewrite get its own tight, low-risk plan and test matrix, isolated from UI churn.
- Planungs-Soll last (Phase 63) is a sequencing choice, not a dependency — ARCHITECTURE explicitly notes it "can be sequenced in parallel or before/after the Person work without either blocking the other." Placing it last is a project-management choice (smaller/better-understood-but-not-risk-free change as a closing phase) rather than an architectural necessity; a roadmapper could equally sequence it first as a warm-up if that suits velocity.
- Requirements drafting must happen as an explicit pre-Phase-60 gate specifically because of Pitfall 7 — this is not a phase itself but should appear on the roadmap as a checkpoint before Phase 60 is planned.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 61 (link/unlink service):** The `assign_to_user`/`discard_assigned_user` rewrite is real new business logic (resolve-or-create person, ambiguous-merge rejection) with no 1:1 precedent elsewhere in the codebase — needs its own test-matrix design during planning, not just "port the query."
- **Phase 62 (UI):** Section-level role gating DTO shape and the zero-children product decision are both open per the consolidated questions list — needs a discuss-phase decision before planning can lock scope.
- **Phase 63 (Planungs-Soll):** The exact reduction-pipeline formula must be verified line-by-line between `assemble_weeks` and `get_reports_for_all_employees` before implementation — flagged uncertain by ARCHITECTURE itself, not just a pitfall.

Phases with standard patterns (skip research-phase):
- **Phase 60 (migration + backfill):** Every technique has a direct, cited precedent already in this repo (nullable `ALTER TABLE ADD COLUMN`, SQL-only backfill, `randomblob(16)`) — standard, well-documented pattern for this codebase specifically.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Every claim cites a specific file/line read directly in this repo; no ecosystem research needed since the question was "what does this repo already do." |
| Features | MEDIUM | Current-app claims are HIGH (verified by reading code); external UX precedents (Salesforce, Auth0, Keycloak, GitLab) are vendor-docs/issue-tracker sourced, self-assessed without the usual `gsd-tools` confidence-classification tooling (Node runtime unavailable this session). |
| Architecture | HIGH | All findings grounded in code read this session (`main.rs`, service/DAO traits, `reporting.rs` call chains fully traced); two items explicitly flagged MEDIUM/uncertain within the doc itself (DTO `kind`-enum decision, whether a new service is truly needed vs. an added method). |
| Pitfalls | HIGH | Grounded entirely in this repo's own migrations, code, and `RETROSPECTIVE.md` cross-milestone lessons — no external/generic pitfall sources used. |

**Overall confidence:** HIGH

### Gaps to Address

- **The "unlinked candidates" backend endpoint is unscoped** — flagged above under Unscoped Work; must be sized into Phase 61 explicitly, not discovered mid-implementation.
- **Section-level role gating's exact DTO/permission mechanism is undecided** — flagged as needing a discuss-phase decision, not resolvable from research alone.
- **The reduction-pipeline formula equivalence between `assemble_weeks` and `get_reports_for_all_employees` was not verified line-by-line** — ARCHITECTURE itself flags this as "should be re-read during planning to confirm... the two loops are independently written and could have subtly diverged."
- **FEATURES.md's confidence is MEDIUM rather than HIGH** due to unavailable tooling this session (no Node/`gsd-tools` runtime) — the external-precedent citations should be treated as directional (badge+filter pattern, modal-with-two-actions pattern) rather than as strong evidence; the current-app analysis within the same document is fully reliable.

## Sources

### Primary (HIGH confidence — code/docs read directly in this repo)
- `Cargo.toml`, `dao/Cargo.toml`, `dao_impl_sqlite/Cargo.toml`, `rest-types/Cargo.toml`, `service_impl/Cargo.toml`, `shifty_bin/Cargo.toml` — dependency versions.
- `migrations/sqlite/20240506114107_add_sales_person.sql`, `20240426150045_user-roles.sql`, `20260330000000_add-shiftplan-table.sql`, `20251017044013_add-session-tracking-to-user-invitation.sql`, `20260428101456_add-logical-id-to-extra-hours.sql`, `20260707000001_add-source-column-to-extra-hours.sql`, `20260503000000_create-absence-migration-quarantine.sql`, `20260503000001_create-absence-period-migration-source.sql` — schema precedents and backfill-pattern precedents.
- `dao_impl_sqlite/src/sales_person.rs`, `dao/src/sales_person.rs`, `service/src/sales_person.rs`, `service_impl/src/sales_person.rs`, `rest/src/sales_person.rs`, `dao_impl_sqlite/src/lib.rs` — full `sales_person_user` blast-radius enumeration.
- `service_impl/src/reporting.rs` (`weight_for_week`, `assemble_weeks`, `get_reports_for_all_employees`, `hours_per_week`, `get_week`), `service/src/reporting.rs`, `rest-types/src/lib.rs`, `rest/src/report.rs`, `service_impl/src/booking_information.rs`, `shifty-dioxus/src/loader.rs` — Planungs-Soll injection-point trace.
- `shifty_bin/src/main.rs`, `docs/architecture/02-service-tiers.md`, `CLAUDE.md` — DI ordering and service-tier convention.
- `shifty-dioxus/src/page/user_management.rs`, `user_details.rs`, `sales_person_details.rs`, `Dioxus.toml` — current-UI state and dev-proxy allowlist.
- `.planning/RETROSPECTIVE.md`, `.planning/PROJECT.md`, `.planning/seeds/personenverwaltung-zusammenlegen.md`, `.planning/todos/pending/2026-09-15-sollstunden-fehlen-bei-dynamischen-vertraegen-unter-schichtplan.md`, `.planning/milestones/v2.6-MILESTONE-AUDIT.md` — cross-milestone lessons, locked seed decisions, bug-report root cause.

### Secondary (MEDIUM confidence)
- Salesforce Help — Person Accounts / Account Fields — "one UI record, blended fields by shape" pattern.
- Auth0 Docs — User Account Linking — unlink-confirmation-copy recommendation.
- PingIdentity — Multi-Account Linking — used to confirm multi-linking is enterprise-scale, not needed here.
- Keycloak GitHub Issue #45704 — "create new vs link existing" ambiguity, basis for modal-with-two-actions recommendation.
- GitLab Docs/Work Item #262876, GitHub REST API org-members docs — badge+filter pattern, "pending/no-account" as first-class filterable state.

---
*Research completed: 2026-09-16*
*Ready for roadmap: yes*
