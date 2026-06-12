# Quick Task 260612-o7t — Scouting Findings (Orchestrator)

**Date:** 2026-06-12
**Author:** Orchestrator (pre-planner scout)
**Target file:** `shifty-dioxus/src/page/absences.rs` (2484 lines)

## The Gap (confirmed by code reading)

The HR person-filter dropdown already exists (`AbsenceFilterBar`, line ~1208, HR-only)
and the `person_filter: Signal<Option<Uuid>>` (line 1692) already filters the
`AbsenceList` (line 1899 receives `filtered_rc` / `filtered_markers_rc`).

BUT two aggregate widgets at the top of the page ignore `person_filter`:

1. **`StatsGrid`** — rendered at line 1880, receives raw `absences.clone()`
   (NOT `filtered_rc`). Computes sick_days / unpaid_days / active_count across
   ALL persons. Definition at line 1285–1343. Props: `absences: Rc<[AbsencePeriod]>`,
   `year`, `today`.

2. **`VacationEntitlementCard`** — rendered at line 1873, receives the full
   `vacation_team: Rc<[VacationBalance]>`. HR body (`VacationEntitlementHrBody`,
   line 361–418) sums remaining/entitled/used/planned/carryover across the whole
   team and shows a per-person list. Definition at line 231–274. The non-HR
   "self" body (`VacationEntitlementSelfBody`, line 285–351) is the rich
   hero-layout (big remaining/entitled number + 5 StatBoxes) that a sales person
   sees for themselves.

## User intent (clarified)

> Select a single Sales Person → see exactly the view the Sales Person sees.

This is a **filter** (not impersonation / no permission change). When
`person_filter` is `Some(uuid)`:
- `StatsGrid` should count only that person's absences.
- `VacationEntitlementCard` should show that single person's vacation
  entitlement — ideally the same rich **self-view layout** the sales person
  sees for themselves (`VacationEntitlementSelfBody`), using that person's
  `VacationBalance` from `vacation_team`.

## Locked decisions

- **D1 — StatsGrid filtering dimension:** Apply ONLY `person_filter` to
  `StatsGrid`, NOT category/status. Rationale: the stat boxes are independent
  metrics (sick days, unpaid days for the year, active count). Applying the
  category filter would make "sick days = 0" while filtering by "vacation",
  which is misleading. Person scoping is the explicit ask.
- **D2 — VacationEntitlementCard when a person is selected (HR):** Render the
  **self-view layout** (`VacationEntitlementSelfBody` hero + 5 StatBoxes) for the
  selected person, using their matching `VacationBalance` from `vacation_team`.
  This is the faithful "exactly the view the sales person sees" interpretation.
  When `person_filter` is `None`, keep the existing HR team aggregate + per-person list.
- **D3 — Non-HR (employee) path unchanged.** `is_hr=false` already loads only the
  current sales person's data and has no filter dropdown; no change needed there.
- **D4 — No backend / rest-types changes.** Pure frontend filtering over data
  already loaded into the stores.

## Implementation hints

- Pass `person_filter_val: Option<Uuid>` (already computed at line 1706) down to
  `StatsGrid` and `VacationEntitlementCard`, OR pre-filter the data at the call
  site (lines 1873–1884) before passing it in. Call-site pre-filtering keeps the
  child components dumb and is the smaller diff — recommended:
  - StatsGrid: pass `absences` filtered by `person_filter_val` (reuse a small
    pure filter, or `filtered_rc` minus the category/status dimensions — but
    `filtered_rc` already applies category+status, so build a person-only
    filtered Vec instead).
  - VacationEntitlementCard: when `person_filter_val` is Some, find the matching
    `VacationBalance` in `vacation_team` and pass it as `vacation_self` with a
    rendering path that uses the self-body. Add a prop like
    `forced_self: Option<VacationBalance>` OR `selected_person: Option<Uuid>` so
    the card knows to render the self-body for that person.

## Test guidance (project rule: tests required)

`absences.rs` already has a `#[cfg(test)] mod tests` (line 1957+) using the
`VirtualDom::new + rebuild_in_place + dioxus_ssr::render` snapshot pattern and a
`pin_de_locale()` helper. Prefer:
- A pure helper function for the person-scoped stats (e.g.
  `stats_for_person(absences, person_filter, year, today) -> (sick, unpaid, active)`)
  that is unit-testable WITHOUT rendering — mirrors the `marker_matches_filters`
  TDD approach from predecessor task 260612-nlv.
- Snapshot test(s) for VacationEntitlementCard with a selected person rendering
  the self-body (assert the selected person's number shows, team aggregate does not).

## Verification gates (from predecessor task)

- WASM build: `nix develop -c cargo build --target wasm32-unknown-unknown` (exit 0)
- Tests: `cargo test` in `shifty-dioxus/` (565 currently green — keep green + add new)
- jj-native commits ONLY (repo is jj-co-located, `use_worktrees=false`).
