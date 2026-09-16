# Pitfalls Research

**Domain:** v2.7 Personenverwaltung (`Person` aggregate over `user`/`sales_person`) + Planungs-Soll DTO field
**Researched:** 2026-09-16
**Confidence:** HIGH (all findings grounded in this repo's code, migrations, and RETROSPECTIVE.md — no external/generic sources used)

Provisional phase numbers below use the roadmap's stated starting point (Phase 60) and
the seed's own phase-cut hypothesis (data-model → link/unlink service → UI → Planungs-Soll).
Treat the numbers as placeholders for "which phase owns this," not as a committed cut.

## Critical Pitfalls

### Pitfall 1: Backfill migration is not idempotent / cannot be re-run

**What goes wrong:**
A migration that does `INSERT INTO person SELECT ...` followed by
`UPDATE sales_person SET person_id = ...` / `UPDATE user SET person_id = ...` runs exactly
once by sqlx's migration ledger (`_sqlx_migrations`) — but if it partially fails (crash,
lock timeout, disk full on a live SQLite file), sqlx will *not* silently retry it: it will
mark the migration dirty and the app refuses to start until a human intervenes. If the
person is tempted to hand-fix the DB and re-run the same migration file, a non-idempotent
`INSERT INTO person SELECT * FROM sales_person WHERE person_id IS NULL` will create a
**second** `person` row for every already-migrated `sales_person`/`user`, because the
`WHERE person_id IS NULL` guard was already satisfied by the first partial run — but only
for the rows the first run reached before it failed.

**Why it happens:**
This repo's only prior backfill of comparable shape (`absence_period_migration_source`,
Phase 4 / v1.0) was deliberately built as a **re-runnable Rust service with an idempotency
key** (`extra_hours_id` as PK on the migration-source table, skip-if-already-mapped), not a
single blind SQL script — see the comment in
`migrations/sqlite/20260503000001_create-absence-period-migration-source.sql:2`:
*"Idempotency key: extra_hours_id PK. Re-run skips already-mapped rows."* A raw SQL
migration for `person` is tempting because the Person merge is *structurally* simpler (no
fuzzy matching — the `UNIQUE(sales_person_id)` + `UNIQUE(user_id)` on `sales_person_user`
already guarantee unambiguous pairs), but "structurally simple" does not imply "safe to
run twice."

**How to avoid:**
- Every `INSERT INTO person ...` and the two `UPDATE ... SET person_id = ...` statements
  must be idempotent on their own: guard inserts with `WHERE NOT EXISTS (...)` keyed on the
  legacy row's PK (`sales_person.id` / `user.name`), and guard the FK backfill with
  `WHERE person_id IS NULL`.
- Wrap the whole backfill in one SQLite transaction (sqlx does this per migration file by
  default — do not split the backfill across multiple migration files that could each
  partially apply).
- Add a throwaway test that applies the migration file's SQL twice against a fixture DB and
  asserts row counts are unchanged the second time — cheap insurance, no new prod schema
  needed (unlike the v1.0 quarantine-table approach, this doesn't need a permanent audit
  table because there is no ambiguity to log — see Pitfall 2 for why a table is still
  useful).

**Warning signs:**
- Migration file has no `WHERE NOT EXISTS` / `WHERE person_id IS NULL` guard on its
  `INSERT`/`UPDATE` statements.
- No test exercises "apply migration, apply again" for row-count stability.

**Phase to address:**
Phase 60 (Person data model + backfill) — this is the first thing the plan for that phase
must specify, before any DAO/service code is written on top.

---

### Pitfall 2: Backfill silently excludes soft-deleted / historical rows

**What goes wrong:**
`sales_person` has a `deleted TEXT` soft-delete column (`migrations/sqlite/20240506114107_add_sales_person.sql`);
`user` has **no** such column at all (`migrations/sqlite/20240426150045_user-roles.sql`) — it is a hard-delete
table. A backfill written with the everyday habit "filter deleted rows out" (`WHERE deleted IS NULL`) will
skip creating `person` rows for soft-deleted sales persons. But `sales_person.id` is referenced by twelve
tables including the **write-once** `billing_period_sales_person` snapshots (per
`.planning/seeds/personenverwaltung-zusammenlegen.md`) — a soft-deleted sales person can still own historical
snapshot rows that a future report or audit tool may want to resolve back to a `Person`. Skipping deleted rows
in the backfill produces `sales_person.person_id IS NULL` for exactly the accounts most likely to be touched
later by a payroll audit, and any code that assumes "every sales_person has a person_id" (a reasonable
assumption once the migration is "done") will panic or silently show blank identity for historical staff.

**Why it happens:**
Soft-delete filtering (`WHERE deleted IS NULL`) is the default reflex in this codebase — CLAUDE.md states
"All queries include soft delete checks" as a DAO convention — but that convention is written for *live*
queries, not for a one-time backfill whose job is to give **every historical row** an identity, active or not.

**How to avoid:**
- Backfill `person` rows for **all** `sales_person` rows regardless of `deleted`, and for **all** `user` rows
  (they can't be soft-deleted anyway).
  Explicitly write this as a one-line comment/test in the migration so a future "cleanup" pass doesn't retrofit
  the usual `deleted IS NULL` filter.
- Add a post-backfill assertion (integration test against a fixture with a soft-deleted sales person that has
  a `billing_period_sales_person` row) that `person_id IS NOT NULL` afterward.

**Warning signs:**
- Backfill SQL/service contains `WHERE deleted IS NULL` or `WHERE inactive = false`.
- No fixture in the migration test covers a soft-deleted sales person with historical billing-period rows.

**Phase to address:**
Phase 60 (Person data model + backfill).

---

### Pitfall 3: Dissolving `sales_person_user` removes the 1:1 invariant, not just a table

**What goes wrong:**
`sales_person_user` carries `UNIQUE(sales_person_id)` **and** `UNIQUE(user_id)`
(`migrations/sqlite/20240506114107_add_sales_person.sql:14-22`) — the database itself
guarantees "at most one link per side" today. Once the table is dropped in favor of
`sales_person.person_id` / `user.person_id`, that guarantee is gone unless it is rebuilt
explicitly: nothing stops two different `sales_person` rows from pointing at the same
`person_id` (two sales persons "sharing an identity"), nor a `user` row and an unrelated
`sales_person` row being linked to the same `person` by an errant UPDATE. This is exactly
the "invariant silently disappears" risk named in the question — the old schema enforced it
for free; the new schema requires someone to have remembered to re-enforce it.

**Why it happens:**
A nullable FK column looks like a strict improvement over a join table (fewer joins, one
row per entity) and it's easy to migrate the *data* correctly while forgetting the
*constraint* was doing real work. SQLite's `UNIQUE` on a nullable column still allows
multiple `NULL`s (which is correct here — many sales persons/users legitimately have no
person link... except after this migration every one of them will, per Pitfall 2) but does
**not** by itself prevent two *different* `sales_person` rows sharing the same non-null
`person_id` unless the column itself is declared `UNIQUE`.

**How to avoid:**
- Declare `UNIQUE(person_id)` on **both** `sales_person.person_id` and `user.person_id` in
  the same migration that adds the columns — this reproduces the old table's two `UNIQUE`
  constraints at the column level and costs nothing.
- If "at most one active SalesPerson + at most one User per Person" needs to survive
  soft-delete semantics differently (e.g. a soft-deleted sales person freeing up its
  `person_id` for reuse), that is a *product decision*, not a default — call it out
  explicitly in the phase's CONTEXT rather than let the schema silently allow it via a
  loose constraint.
- Write a regression test that attempts to link a second `sales_person` to a `person_id`
  already in use and asserts a DB-level rejection, not just an application-level check —
  application-level checks race under concurrent requests exactly the way the old
  `sales_person_user` insert would have without its `UNIQUE`.

**Warning signs:**
- Migration adds `person_id BLOB(16) NULL` without a `UNIQUE` constraint or matching
  index+application check.
- No test attempts "double-link" and expects failure.

**Phase to address:**
Phase 60 (schema) with a dedicated regression test; Phase 61 (link/unlink service) must
not be the *only* place the invariant is enforced.

---

### Pitfall 4: The self-permission check that joins through the dissolved table is missed

**What goes wrong:**
`service_impl/src/sales_person.rs:356-401` resolves "is the current user allowed to touch
this sales person because it's *their own*" via
`find_sales_person_by_user_id(&user_id, ...)`, which in
`dao_impl_sqlite/src/sales_person.rs:226-233` is implemented as
`SELECT ... FROM sales_person sp JOIN sales_person_user spu ON sp.id = spu.sales_person_id
WHERE spu.user_id = ?`. This is a real, currently-exercised authorization path (self vs.
HR/Shiftplanner gating on sales-person mutation), not incidental plumbing. When
`sales_person_user` is dropped, this query must be rewritten as a join through `person_id`
(`sales_person.person_id = (SELECT person_id FROM user WHERE name = ?)`) or the "self" check
silently starts failing closed (locking legitimate users out of their own record) or —
worse — failing open if the rewrite is sloppy (e.g. comparing on a nullable column without
excluding `NULL = NULL` matches, which SQL never actually matches, but a hand-rolled Rust
comparison of two `Option<Uuid>` easily could).

**Why it happens:**
This DAO method's only visible callers are inside `sales_person.rs`; a search for
"sales_person_user usage" only surfaces `dao/src/sales_person.rs`,
`dao_impl_sqlite/src/lib.rs`, `dao_impl_sqlite/src/sales_person.rs`,
`rest/src/sales_person.rs`, and `service_impl/src/sales_person.rs` — five files, easy to
believe "I got them all" without specifically grepping for the **authorization** use
(`find_sales_person_by_user_id`) as distinct from the CRUD use
(`assign_user_to_sales_person`/`remove_...`).

**How to avoid:**
- Before removing `sales_person_user`, grep for every call site of
  `find_sales_person_by_user_id` specifically (not just "sales_person_user" as a string)
  and classify each as CRUD vs. authorization.
- Add/keep a permission-layer test that asserts a non-privileged user can modify their own
  sales-person record and cannot modify another's, run it *before and after* the schema
  swap (byte-identical pass/fail matrix), analogous to the Byte-Identity-Fixture pattern
  used for the v2.5 performance refactor.
- Do the rewrite of this query in the *same* phase/plan as the schema swap — do not defer
  it as "frontend follow-up," it's backend authorization logic.

**Warning signs:**
- `cargo build` succeeds after dropping `sales_person_user` (a stale raw SQL string
  referencing a dropped table is a compile-time-invisible failure with `sqlx::query!` only
  if the local dev DB still has the old table, or a runtime error if it doesn't — either way
  it will not show up as a Rust compile error until the query macro re-checks against a
  migrated DB).
- No permission test explicitly exercises "self" access to a sales person after the schema
  change.

**Phase to address:**
Phase 61 (Person aggregate service / link management) should own the query rewrite; Phase
60 should not be declared "done" while this call site is unaddressed even though it's a
different file than the migration.

---

### Pitfall 5: Link/unlink through the UI creates orphaned or dangling `Person` rows

**What goes wrong:**
The seed doc's own model makes "Login hinzufügen" / "Person hinzufügen" *actions on an
existing entry* that flip it between the three shapes (User+SalesPerson,
User-only, SalesPerson-only). Once this is a live UI action (not just a one-time backfill),
two new failure modes appear that the backfill never had to handle:
1. **Unlink to zero:** a user detaches the SalesPerson side and later the User side too
   (e.g. deactivating a login on an account that's also being deleted) — the `person` row
   is left with no children at all. Nothing prevents this, and nothing cleans it up; a
   "Personen" list view that lists `person` rows (rather than deriving the list from
   `sales_person`/`user`) will show ghost entries forever.
2. **Re-link ambiguity:** "Person hinzufügen" to a bare `user` row must pick *which*
   `sales_person` to link — if the UI/service allows linking to a `sales_person` that
   already has a non-null `person_id` (i.e., already linked to a *different* person), that
   silently reassigns history (all twelve tables' rows for that `sales_person.id`, including
   billing snapshots) to a new identity without any of that historical data changing rows.
   This is invisible in the data — `sales_person.id` doesn't move — but very visible to a
   user who now sees a stranger's shift history "belong" to their merged person view.

**Why it happens:**
The backfill migration only ever has to handle the *existing, already-validated* 1:1 pairs
(guaranteed unambiguous by the old `UNIQUE` constraints). The live link/unlink UI has to
enforce the same invariants **without** the safety net of "this data was already validated
once," against concurrent requests, against already-linked targets, and against the
zero-children case — none of which the migration's correctness proof covers.

**How to avoid:**
- Link action must reject (409, not silent overwrite) if the target `sales_person`/`user`
  already has a non-null `person_id` pointing elsewhere — surface this as an explicit
  `ServiceError` variant, not a generic validation error, so the FE can show "already linked
  to X."
- Unlink action must decide up front (product decision, put in the phase's CONTEXT, not
  discovered mid-implementation) whether a zero-children `person` row is: (a) allowed and
  garbage-collected by a background job, (b) allowed and surfaced in an admin "orphaned
  Personen" view, or (c) disallowed — the last unlink of a Person's two sides deletes the
  `person` row too. Given `person` currently carries only identity + audit columns (no
  independent data), (c) is the simplest and cheapest to build a test around.
- Never derive the "list of Personen" purely from `SELECT * FROM person` — derive it from
  `sales_person`/`user` outer-joined through `person_id`, the way the v1.0 Person seed
  itself frames the three shapes ("Ausprägung"). This makes ghost `person` rows invisible
  even if (a) is chosen, without needing a cleanup job to be correct.

**Warning signs:**
- No `ServiceError` variant for "target already linked."
- The Personen list endpoint queries `person` as its base table rather than
  `sales_person`/`user`.
- No test exercises unlink-both-sides and re-checks list-view behavior afterward.

**Phase to address:**
Phase 61 (link/unlink service) for the reject-on-already-linked check; Phase 62
(Personenverwaltung UI) for the list-view derivation and the zero-children product
decision, but the decision itself belongs in that phase's CONTEXT/discuss step, not left to
the executor.

---

### Pitfall 6: `planning_target_hours` leaks into balance/payroll, or an existing consumer is silently rebound to it

**What goes wrong:**
Two independent code paths already read very similar-looking numbers off the same
`EmployeeWorkDetails`/report structs — `expected_hours` (bilanzrelevant, deliberately `0.0`
for dynamic contracts, see `service_impl/src/reporting.rs:2184-2188`) and `dynamic_hours`
(the raw contract value, *not* holiday-adjusted per the `HSP-03` band guard). At least three
separate `rest-types` TO structs carry both fields side by side
(`rest-types/src/lib.rs:377-402`, `:479-512`, `:544-578`) — meaning any new
`planning_target_hours` field has to be threaded through the same number of structs and
`From` impls, and it is easy to add it to one and forget the others (per-boundary threading
is exactly the failure mode the v1.4 retrospective calls out: *"End-to-end-Feld-Threading
braucht Per-Boundary-Tests"*). Two concrete leak directions:
1. **Leak into balance:** a future refactor of `apply_weekly_cap`/`balance_hours` (which
   today deliberately consumes `expected_hours`, not `dynamic_hours`) could be "simplified"
   by someone who doesn't know *why* `expected_hours` is zeroed for dynamic contracts and
   swaps in the new, always-populated `planning_target_hours` because it's the more
   convenient non-zero number — this would make dynamic-contract balances swing on a number
   that was deliberately excluded from bilanzrelevanz.
2. **Existing consumer silently rebound:** the todo itself lists the FE consumers that key
   off `expected_hours` today — `hours_text_class`/`progress_bar_class` color thresholds and
   the `total_target` summary row in `working_hours_mini_overview.rs` — and explicitly flags
   "die sollen vermutlich alle auf das Planungs-Soll umgestellt werden, das ist aber eine
   bewusste Entscheidung pro Stelle" (`.planning/todos/pending/2026-09-15-...md`). If an
   executor rebinds all of them in one sweep without per-site sign-off, a **non-dynamic**
   contract's rendering must be proven byte-identical (since `planning_target_hours ==
   expected_hours` there by construction) — if the two computations drift even slightly
   (e.g. one applies the holiday-derived reduction and the other doesn't), *every* employee's
   card — not just dynamic ones — silently changes.

**Why it happens:**
The whole point of the new field is "same shape as an existing bilanzrelevant field, but
computed slightly differently for one case" — that similarity is exactly what makes an
accidental substitution easy and easy to miss in review, and this codebase already has a
name for the underlying class of bug: **Achse-A-vs-Achse-B-Doppelzählung**, the #1 top
lesson across all milestones (`RETROSPECTIVE.md` Cross-Milestone Trends): *"jede
Kapazitäts-/Stunden-Änderung muss explizit benennen, welche Achse sie berührt."*

**How to avoid:**
- Write `planning_target_hours` as `expected_hours`-equivalent-plus-dynamic-override,
  explicitly derived from the *same* holiday-/absence-reduction pipeline as `expected_hours`
  today, not as an independent recomputation from `dynamic_hours` — the todo already
  concludes this is the only correct shape ("das gleich berechnete Soll ... statt 0").
- Add a regression test asserting `planning_target_hours == expected_hours` for every
  **non-dynamic** contract fixture already in the reporting test suite (reuse existing
  fixtures, don't write new ones) — this is the byte-identity discipline from v2.5 applied
  to a field instead of a whole endpoint.
- Grep-verify (and lock with a comment, the way `HSP-03`'s comment already does at
  `working_hours_mini_overview.rs`) that `planning_target_hours` never appears on the
  left-hand side of `balance_hours`/`apply_weekly_cap`/anything feeding
  `billing_period_report.rs`.
- Treat each of the "further consumers" (`hours_text_class`, `progress_bar_class`,
  `total_target`) as its own explicit decision point in the phase's plan, not a blanket
  find-and-replace — per the todo's own "Zu klären" list.
- Document the two-formulas-one-domain split explicitly (analogous to v2.6's D-57-01
  "gedeckelt schreiben, ungedeckelt vorschlagen" pattern — "two deliberately different
  formulas on the same domain, both documented").

**Warning signs:**
- `planning_target_hours` computed by a fresh formula instead of reusing the
  `weight_for_week`-style pipeline.
- Any of the three `rest-types` TO structs gets `dynamic_hours`/`expected_hours` but not
  `planning_target_hours`, or vice versa.
- A single PR/plan changes `hours_text_class`, `progress_bar_class`, and `total_target` all
  at once "while we're at it."

**Phase to address:**
Phase 63 (Planungs-Soll DTO) — must include the non-dynamic byte-identity regression test
as an explicit success criterion, not an afterthought.

---

### Pitfall 7: A neutrality-shaped requirement gets pinned before the milestone goal is checked against it

**What goes wrong:**
v2.6's single most expensive mistake (per `RETROSPECTIVE.md`, "Key Lessons" #1 and Top
Lesson #5) was a requirement (VOL-ACCT-03) that *literally* forced "the rebooking pair is
balance-neutral" — the opposite of what the milestone existed to build — and it survived
three phases of clean, green verification because "balance stays neutral" is trivially easy
to test and keep green forever, even when it's wrong. This milestone has **two** places
where a superficially similar "keep it exactly unchanged" requirement is easy to write and
just as easy to get backwards:
1. **Person merge:** a requirement like "linking/unlinking a Person must not change any
   existing sales_person/user/auth behavior" is *correct* as long as it's scoped to
   behavior that has nothing to do with identity — but if phrased broadly enough to also
   freeze the self-permission-check query (Pitfall 4) or the display-name derivation
   (`sales_person.name`/`user.name` fallback logic, per the seed's own open question #3),
   it would pin the *current, join-table-based* implementation and block the very rewrite
   the migration requires.
2. **Planungs-Soll:** a requirement like "`planning_target_hours` must never affect the
   balance" is correct in intent (it's literally the point) but must not be worded/tested in
   a way that could also be misread as "must equal `expected_hours`" or "must equal
   `dynamic_hours`" — either literal reading is wrong (see Pitfall 6), yet both would also
   pass a naive "value doesn't change existing balance tests" verification.

**Why it happens:**
Per `RETROSPECTIVE.md` Top Lesson #5: *"Ein Requirement, das eine Invariante festschreibt
..., lässt sich mühelos testen und dauerhaft grün halten — auch dann, wenn es genau das
Feature verhindert."* Phase-level verification checks a phase against its requirements, not
against the milestone goal — the milestone-goal cross-check only happens at the (later,
more expensive) milestone audit unless done explicitly earlier (Top Lesson #6).

**How to avoid:**
- Apply the pattern v2.6 Phase 57 established under duress, proactively this time: any
  requirement in this milestone's REQUIREMENTS.md that reads as "X stays unchanged" or "X
  is neutral with respect to Y" gets an explicit one-line milestone-goal cross-check before
  the first phase that depends on it is planned — not deferred to the audit.
  Concretely for v2.7: write out "does this requirement, read literally, still allow a user
  to see a Planungs-Soll number at dynamic contracts?" and "does this requirement, read
  literally, still allow the self-permission query to be rewritten?" as acceptance
  questions in REQUIREMENTS.md itself.
- Prefer requirements phrased as "computed via [specific named mechanism]" over bare
  "unchanged"/"neutral" — e.g. "planning_target_hours reuses the expected_hours reduction
  pipeline, differing only in the dynamic-contract zero-clamp" rather than "does not affect
  balance."

**Warning signs:**
- A requirement whose acceptance test is "assert old_value == new_value" for a
  behavior/value that the milestone's own goal requires to change for at least one case.
- A requirement that can be fully satisfied by *doing nothing* to the code path it names.

**Phase to address:**
Requirements drafting, before Phase 60 is planned — this is a discuss/requirements-review
gate, not a phase deliverable, but the roadmap should flag REQUIREMENTS.md review as an
explicit step before Phase 60 kicks off given the direct v2.6 precedent.

---

### Pitfall 8: New REST path family ships without a `Dioxus.toml` dev-proxy entry

**What goes wrong:**
`shifty-dioxus/Dioxus.toml`'s `[[web.proxy]]` list is a manual, per-prefix allowlist — there
is no catch-all and no `/user` or `/person` entry exists today (verified: the current list
has ~24 explicit entries, e.g. `/sales-person`, `/user-invitation`, `/vacation-balance`, but
no bare `/user` and obviously no `/person` yet). Any new `/person`-prefixed REST namespace
(and, separately, if the Planungs-Soll work adds any new endpoint rather than just a DTO
field) that ships without a matching proxy line will compile clean, pass every backend and
FE unit test, and then fail with **HTTP 404/405 only in the live dev server** — invisible to
`cargo test`, invisible to `cargo build --target wasm32-unknown-unknown`.

**Why it happens:**
This is a **three-times-repeated** failure in this exact repo, per `RETROSPECTIVE.md`: v1.6
missed `/toggle`, v1.8 missed `/vacation-entitlement-offset`, and the cross-milestone Top
Lesson #3 explicitly names it as the pattern: *"Neue REST-Pfad-Familien brauchen am selben
Tag den `Dioxus.toml`-Dev-Proxy-Eintrag."* Every prior occurrence was caught only in a live
human/browser smoke test, i.e. late.

**How to avoid:**
- The moment a plan introduces a new top-level REST path segment (`/person`, or whatever the
  Person CRUD/link endpoints are named), add the matching `[[web.proxy]]` entry in the
  *same* plan/commit that adds the backend route — not "when we get to the frontend phase."
- Add this as an explicit checklist line in the phase's plan template for any phase that
  adds a REST path family (the retrospective itself proposes this: *"Kandidat für eine
  Plan-Checkliste 'neuer REST-Pfad → Proxy-Eintrag'"* — v2.7 is a good place to finally add
  that checklist item rather than rediscover the bug a third time).
- If Planungs-Soll ships as a pure DTO field on an existing endpoint (no new path), this
  pitfall doesn't apply to that part — but double-check it doesn't introduce a companion
  endpoint (e.g., a "recompute" or "history" endpoint) that would.

**Warning signs:**
- A new `rest/src/person.rs` (or similar) route module with no corresponding line added to
  `shifty-dioxus/Dioxus.toml`.
- FE-side loader code for the new feature is written and unit-tested (mocked HTTP) but never
  exercised against a live `dx serve` + `cargo run` pair before UAT.

**Phase to address:**
Phase 61/62 (whichever phase first exposes a `/person`-prefixed REST route) — add the proxy
line as a required deliverable in that plan's file list, verified by grep in the plan's own
checker step.

---

### Pitfall 9: Docs-Freshness-Gate drift accumulates silently across this milestone's many trigger files

**What goes wrong:**
This milestone touches an unusually large number of the Docs-Freshness-Gate's trigger files
in one pass: `migrations/sqlite/*.sql` (person table + backfill), `service/**/*.rs` and
`dao/**/*.rs` (Person aggregate traits), `service_impl/src/permission.rs` (self-check
rewrite, Pitfall 4), and `service_impl/src/reporting.rs` (Planungs-Soll). Per
`CLAUDE.md`/`PROJECT.md`, each trigger requires **both** `docs/*.md` and `docs/*_de.md`
updated in the same commit/PR, checked at milestone close — but `RETROSPECTIVE.md` records
this exact category of drift recurring across nearly every milestone: v1.4 doc-drift on a
mid-milestone snapshot-version reversal, v1.7's Milestone-Close delay leaving
REQUIREMENTS.md frozen against a moving target, v2.5's "Doku-Follow-ups nicht sofort" (new
pure helpers not documented until close), and v2.6's Nyquist-VALIDATION checklists left on
`draft` for three phases despite the underlying tests existing. With five trigger categories
hit in one milestone instead of the usual one or two, the probability of at least one being
missed until the (expensive, late) milestone audit is high.

**Why it happens:**
Per `RETROSPECTIVE.md` Key Lessons (v1.7): *"Milestone-Close zeitnah nach 'complete &
verified' fahren — sonst friert die aktive REQUIREMENTS.md den alten Stand ein."* Docs
updates get treated as a close-time chore rather than a per-phase deliverable, especially
when (as here) the *permission.rs* change is a side-effect of the schema change rather than
its headline, making it easy to forget it also triggers `docs/architecture/04-auth.md` +
`docs/features/F12-auth-session.md`.

**How to avoid:**
- Run the Docs-Freshness trigger-diff check (per `PROJECT.md`'s documented `git diff
  --name-only <last-milestone-tag>..HEAD -- ...` command) **after every phase**, not just at
  close — cheap to run, catches drift while the change is still fresh in context.
  Given this milestone's phase count is likely 3-4 (per the seed's phase-cut question),
  running it 3-4 times is far cheaper than one large reconciliation at close.
- Explicitly assign the `permission.rs` → `04-auth.md`/`F12-auth-session.md` doc update to
  whichever phase does the Pitfall-4 query rewrite, even though that phase's headline is
  "Person link/unlink service," not "auth."
- Nyquist-VALIDATION checklists: tick them off as part of each phase's own close, not
  deferred — three consecutive milestones (v2.5 partially, v2.6 fully) shipped with stale
  `draft` checklists that the audit had to explain away as "drift, not a gap."

**Warning signs:**
- A phase's SUMMARY.md lists changes to `permission.rs`, `migrations/sqlite/`, or
  `dao/src/`/`service/src/` traits without a corresponding `docs/` diff in the same commit.
- VALIDATION.md status stays `draft` after a phase's VERIFICATION.md is `passed`.

**Phase to address:**
Every phase in this milestone (60-63) — this is a per-phase discipline item, not a single
phase's deliverable; the roadmap should note it once and each phase's plan should inherit
it as a standing checklist line.

---

### Pitfall 10: The merged Personenverwaltung list/detail UI produces a new `human_needed` DOM-rendering gap

**What goes wrong:**
`RETROSPECTIVE.md`'s Key Lessons for v2.6 note explicitly: *"`human_needed` häuft sich dort,
wo automatisierte Tests strukturell nicht hinkommen (DOM-Rendering, Wall-Clock-Cron)."* Two
`human_needed` VERIFICATIONs shipped in v2.6 alone for exactly this reason (HR-row
visibility, cron firing). The Personenverwaltung UI work (merging three detail pages into
one entry component with per-Ausprägung section visibility, badges, and per-section
role-gating) is squarely in "DOM-rendering, Dioxus-WASM-signal-driven" territory — the same
class of thing v1.5's `modify_slot` bug and multiple prior milestones' Dev-Proxy bugs were
only caught by live browser UAT, never by `cargo test`.

**Why it happens:**
Dioxus-WASM signal propagation is, per this repo's own established pattern
(`RETROSPECTIVE.md` v2.5 Patterns Established: *"INT-Roundtrip statt Browser-Automation für
Rendering-Sightcheck ... Dioxus-WASM-Signal-Propagation ist headless nicht zuverlässig
automatisierbar"*), not reliably testable outside a real browser. A UI that conditionally
shows/hides three different sections based on Ausprägung + role is exactly the shape of
logic most likely to have a live-only bug (e.g. a section that renders for the wrong role,
or a badge that doesn't update after a link/unlink action without a page reload).

**How to avoid:**
- Push every piece of *decision* logic ("which sections are visible for this Ausprägung and
  this role") into a pure Rust helper function with exhaustive unit tests — per the
  established "Backend-rechnet-vor" / "pure helper" pattern — and keep the RSX itself as a
  thin, mostly-untested passthrough, the same split used for v1.8's grouped selector helper.
- Explicitly scope one live human-UAT pass for the merged entry component before this
  milestone closes, rather than discovering the need for it during close (as happened
  repeatedly across v1.6/v1.8/v2.6).
- If Phase 59 (Playwright E2E infra, prepared but not yet promoted) gets pulled into scope,
  this is the natural first target — otherwise plan for a manual UAT checklist item
  up front, not as a surprise late in Phase 62.

**Warning signs:**
- Phase 62's plan has no explicit human-UAT or Playwright step for the merged entry view.
- Section-visibility logic lives inline in `rsx!{}` blocks rather than in a named, tested
  helper function.

**Phase to address:**
Phase 62 (Personenverwaltung UI) — plan the human-UAT step at plan-writing time, not at
verification time.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|-----------------|------------------|
| Backfill as raw SQL migration instead of a re-runnable Rust service (unlike the v1.0 absence-migration precedent) | Less code, faster to ship | No idempotency-key safety net if it fails partway on a live DB; harder to add a dry-run | Acceptable *only* because the Person merge has no ambiguity to resolve (unlike v1.0's fuzzy category mapping) — still needs the idempotency guards from Pitfall 1, just doesn't need a full quarantine-table apparatus |
| Keep `person` table identity-only (no email/phone) for this milestone | Smaller schema change, no new PII handling | Every future "real" person attribute (email, phone) needs its own migration later; teams sometimes then duplicate the attribute onto `sales_person`/`user` instead of waiting | Acceptable — explicitly the seed's own decision; revisit only when a concrete attribute is needed |
| Deriving the Personen list by outer-joining `sales_person`/`user` instead of querying `person` directly | Avoids needing to garbage-collect orphaned `person` rows | Slightly more complex query; must be kept in sync if a real Person-owned attribute is added later | Never trade this away for "simpler" until `person` earns real owned columns |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|------------------|-------------------|
| Frontend dev proxy (`Dioxus.toml`) | New `/person`-prefixed REST route shipped without a matching `[[web.proxy]]` entry — 3rd occurrence of this exact bug class in this repo | Add the proxy line in the same plan/commit as the backend route; grep-verify in the plan's checker step |
| Self-permission check (`find_sales_person_by_user_id`) | Rewriting the schema without rewriting this specific query, or rewriting it with a nullable-`Option` equality bug | Rewrite in the same phase as the schema swap; test both "self succeeds" and "not-self fails" before/after |
| `rest-types` dual-consumption (backend + `shifty-dioxus`) | Adding `planning_target_hours` to the backend-side struct without threading it through every `From` impl feeding the three TO structs that already carry `expected_hours`/`dynamic_hours` | Grep all three struct sites (`rest-types/src/lib.rs:377,479,544`) before considering the field "added" |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|-----------------|
| Personen-list view doing N+1 lookups (one query per entry to resolve linked sales_person/user) instead of a bulk join | Slow `/user_management/` load once entry count grows | Single outer-join query for the list, mirroring the year-batch bulk-load pattern established in v2.5's `get_weekly_summary` refactor | Noticeable once shop staff count grows past a few dozen — not urgent at today's scale, but the pattern is already known-bad in this repo (v2.5's whole milestone existed to fix an N+1-shaped bug) |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|-------------|
| Self-permission check regressing to fail-open after the `sales_person_user` → `person_id` rewrite (Pitfall 4) | A user could gain edit access to another sales person's record if a nullable-FK comparison bug matches unrelated `NULL`s or unlinked rows | DB-level join comparison (never two separately-fetched `Option<Uuid>` compared in Rust); explicit before/after permission test matrix |
| Link action allowing re-target of an already-linked `sales_person`/`user` without an explicit conflict error (Pitfall 5) | Silent identity reassignment moves a stranger's shift/payroll history under a different Person view | Reject-with-409 on already-linked target, never silently overwrite |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|--------------|-------------------|
| Personenverwaltung list still implies "SalesPerson tab" vs "User tab" mental model even after merging | Users continue to think of two separate registries, defeating the milestone's entire stated purpose ("extrem verwirrend") | One list, one entry, Ausprägung as a visible badge/column — per the seed's own explicit decision, not a compromise |
| Planungs-Soll shown without any visual distinction from bilanzrelevant Soll | HR/Shiftplanner may mistake the always-populated planning number for the real balance-affecting Soll on dynamic contracts | Explicit UI cue (label/tooltip) that this number is for planning only and doesn't affect the balance, since the two now visually coexist for the first time |

## "Looks Done But Isn't" Checklist

- [ ] **Backfill migration:** Often missing an idempotency guard — verify it can be applied twice against the same fixture DB with no row-count change.
- [ ] **Backfill migration:** Often missing soft-deleted `sales_person` rows — verify a soft-deleted sales person with `billing_period_sales_person` history gets a `person_id`.
- [ ] **`sales_person_user` dissolution:** Often missing the `UNIQUE(person_id)` constraint on the new FK columns — verify a double-link attempt is rejected at the DB layer, not just in application code.
- [ ] **`sales_person_user` dissolution:** Often missing the rewrite of `find_sales_person_by_user_id`'s self-permission join — verify with an explicit "self can edit own record" test after the schema swap.
- [ ] **Link/unlink UI:** Often missing the "target already linked" conflict check — verify linking to an already-linked entity is rejected, not silently reassigned.
- [ ] **`planning_target_hours`:** Often missing the non-dynamic byte-identity guarantee — verify `planning_target_hours == expected_hours` for every non-dynamic fixture already in the reporting test suite.
- [ ] **`planning_target_hours`:** Often missing propagation to all three `rest-types` TO structs that already carry `expected_hours`+`dynamic_hours` — grep all three sites.
- [ ] **New `/person` REST routes:** Often missing the `Dioxus.toml` proxy entry — verify with a live `dx serve` + `cargo run` smoke, not just unit tests.
- [ ] **Docs-Freshness-Gate:** Often missing `permission.rs`'s doc trigger (`04-auth.md`/`F12-auth-session.md`) because that change looks like a side-effect of the schema work rather than its headline.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|-----------------|------------------|
| Non-idempotent backfill partially applied on prod (Pitfall 1) | MEDIUM | Write a targeted reconciliation script (not a re-run of the original migration) that only inserts/updates rows where `person_id IS NULL`, guarded the same way the original should have been; verify row counts against `sales_person`/`user` totals before/after |
| Duplicate `person` rows discovered after the fact (missing `UNIQUE(person_id)`, Pitfall 3/5) | MEDIUM-HIGH | Identify duplicates via `GROUP BY person_id HAVING COUNT(*) > 1` on each FK side; requires a manual merge decision per duplicate (which `person_id` wins) — this is exactly the kind of ambiguity the old `UNIQUE` constraints existed to prevent, so recovery is manual, not scriptable |
| `planning_target_hours` found leaking into a balance computation post-ship (Pitfall 6) | LOW-MEDIUM | Since no snapshot bump was taken for this milestone, any persisted billing-period value is *not* supposed to have moved — a leak would show up as a snapshot/live-recompute mismatch; the validator pattern already built in v2.6 (snapshot vs. re-run diff) is directly reusable to detect and then bound the blast radius |
| Missing `Dioxus.toml` proxy entry discovered late (Pitfall 8) | LOW | One-line fix, redeploy dev environment — but note the time already lost to live-UAT discovery instead of catching it at plan-review time, three times running in this repo |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|-------------------|----------------|
| 1. Non-idempotent backfill | Phase 60 | Apply migration twice against fixture DB, assert stable row counts |
| 2. Soft-deleted rows excluded from backfill | Phase 60 | Fixture with soft-deleted sales_person + billing_period_sales_person row, assert `person_id IS NOT NULL` after |
| 3. Dissolved link-table invariant not rebuilt | Phase 60 (schema), Phase 61 (service) | DB-level double-link rejection test |
| 4. Self-permission query rewrite missed | Phase 61 | Before/after permission test matrix on "self can edit own sales person" |
| 5. Link/unlink orphans / re-target ambiguity | Phase 61 (reject-on-linked), Phase 62 (list derivation + zero-children decision) | Conflict-error test on re-target; list-view test after unlink-both-sides |
| 6. `planning_target_hours` leak / consumer swap | Phase 63 | Non-dynamic byte-identity regression test; grep-lock on balance computation call sites |
| 7. Neutrality-shaped requirement pinned prematurely | Requirements drafting (pre-Phase 60) | Explicit milestone-goal cross-check question written into REQUIREMENTS.md for any "unchanged"/"neutral" requirement |
| 8. Missing `Dioxus.toml` proxy entry | Phase 61/62 (whichever adds `/person` routes) | Grep check in plan's checker step; live `dx serve` smoke before UAT |
| 9. Docs-Freshness-Gate drift | Every phase (60-63) | Run trigger-diff check after each phase close, not only at milestone close |
| 10. Merged UI `human_needed` DOM gap | Phase 62 | Pure-helper unit tests for section-visibility logic + explicit human-UAT step planned up front |

## Sources

- `.planning/RETROSPECTIVE.md` — cross-milestone lessons (v1.4 through v2.6), especially the
  v2.6 "Neutralitäts-Requirements" lesson (Top Lesson #5/#6), the Dev-Proxy lesson (Top
  Lesson #3, v1.6/v1.8), and the Achse-A-vs-Achse-B lesson (Top Lesson #1, v1.4).
- `CLAUDE.md` (repo root) — snapshot-version-bump contract, Docs-Freshness-Gate trigger
  table, service-tier conventions.
- `.planning/PROJECT.md` — architecture principles (Fat Backend/Thin Client), Docs-Freshness-
  Gate mechanics, v2.7 milestone framing and "unantastbar" constraints.
- `.planning/seeds/personenverwaltung-zusammenlegen.md` — locked Person modeling decisions,
  blast-radius list (twelve tables referencing `sales_person.id`), Variante A schema sketch.
- `.planning/todos/pending/2026-09-15-sollstunden-fehlen-bei-dynamischen-vertraegen-unter-schichtplan.md`
  — root cause of the Planungs-Soll gap, the "naheliegender Fix wäre falsch" trap, and the
  list of further consumers to check individually.
- `.planning/milestones/v2.6-MILESTONE-AUDIT.md` — precedent for cross-phase integration
  checks, snapshot-version-bump verification method, and the `human_needed` override pattern.
- `migrations/sqlite/20240506114107_add_sales_person.sql`,
  `20240426150045_user-roles.sql` — the exact schema being dissolved/extended (link table's
  double `UNIQUE`, `user`'s hard-delete-only PK).
- `migrations/sqlite/20260503000000_create-absence-migration-quarantine.sql`,
  `20260503000001_create-absence-period-migration-source.sql`,
  `20260503000003_add-cutover-admin-privilege.sql` — this repo's own prior backfill
  precedent (idempotency-key pattern, quarantine table, dedicated privilege-gated cutover
  endpoint).
- `service_impl/src/sales_person.rs:356-401`, `dao/src/sales_person.rs:71`,
  `dao_impl_sqlite/src/sales_person.rs:109,167,181,197,211,226-233` — the concrete
  `sales_person_user` call sites, including the self-permission-check join that must be
  rewritten.
- `rest-types/src/lib.rs:377-402,479-512,544-578` — the three TO structs already threading
  `expected_hours`/`dynamic_hours` together, showing the propagation surface for
  `planning_target_hours`.
- `service_impl/src/reporting.rs:2184-2189` — `weight_for_week`, the exact site zeroing
  `expected_hours` for dynamic contracts.
- `shifty-dioxus/Dioxus.toml` — current dev-proxy allowlist (confirms no `/user` or
  `/person` entry exists today).

---
*Pitfalls research for: v2.7 Personenverwaltung & Planungs-Soll (Shifty backend, existing production system)*
*Researched: 2026-09-16*
