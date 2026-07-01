# Phase 38: Frontend-Build-Hygiene - Context

**Gathered:** 2026-07-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Make `shifty-dioxus` build warning-free and keep the backend clippy gate green.
Pure cleanup — **no new capabilities**, no behavior changes, no new deps, no migration,
no snapshot-schema bump (stays 12).

**Scope anchor (from ROADMAP HYG-01/HYG-02):**
- **HYG-01:** `shifty-dioxus` rustc-warning-free. Current baseline: **50 `cargo build` warnings**
  (14 auto-fixable via `cargo fix`, ~34 dead-code, 2 deprecated `time::parse`).
- **HYG-02:** Backend stays `cargo clippy --workspace -- -D warnings` green (regression gate).
  Deliberately-kept lints are documented.

**Success (from ROADMAP):**
1. `cargo build` (dioxus) with zero warnings
2. `cargo clippy --workspace -- -D warnings` (backend) green
3. `cargo test -p shifty-dioxus` green
4. `cargo build --target wasm32-unknown-unknown` (WASM build) green

</domain>

<decisions>
## Implementation Decisions

### Dead-Code Policy (~34 warnings — the core)
- **D-01:** **Default = delete.** Dead code is recoverable from git history; keeping it rots.
  Unused private functions / methods / enums / variants / constants / fields → remove.
- **D-02:** Unused imports / unused variables / unnecessary `mut` (the 14 `cargo fix` cases) →
  always remove.
- **D-03:** `#[allow(dead_code)]` **only as a justified exception**, and each such site MUST
  carry a `// reason: <why>` comment. Valid reasons: (a) trait/signature symmetry,
  (b) obviously-planned API tied to a concrete open requirement, or (c) deleting would trigger
  a larger restructure that blows the "hygiene" scope (then keep + document rather than
  expand scope). No blanket keep-list requested by the user — candidates like
  `register_user_to_slot`, `load_slots`, `get_bookings_for_week`, `has_sunday_slots` are
  **deletion candidates by default** unless a reason above applies during implementation.
- **D-04:** Preference is to actually **fix** the frontend (remove dead code), not to blanket-
  suppress warnings where removal is the cleaner option.

### Deprecated `time::parse` (2 sites)
- **D-05:** **Migrate to `parse_borrowed`** (same `time` crate, pure API rename, no behavior
  change) rather than `#[allow(deprecated)]`. Only fall back to allow if the migration
  unexpectedly requires the format-version argument and becomes awkward.

### Documentation of deliberately-kept lints
- **D-06:** Document **inline at the symbol**: `#[allow(dead_code)] // reason: <why>`. Travels
  with the code, no separate drifting document.
- **D-07:** Additionally collect the list of kept exceptions in the DISCUSSION-LOG/CONTEXT as
  an overview for the gate.

### Scope of the clippy work
- **D-08:** **dioxus stays OUT of the CI clippy gate** (out of scope). dioxus is a separate
  WASM workspace with ~198 pre-existing clippy lints; clippy only runs from the **backend**
  nix-shell (E0514 in the dioxus shell). Building a dioxus `clippy -D warnings` gate would be
  its own large phase.
- **D-09:** **Strict scope: only the 50 rustc `cargo build` warnings.** dioxus clippy stays
  entirely untouched — do NOT start cleaning the 198 clippy lints. That can become its own
  phase later if wanted.

### Verification
- **D-10:** Gate = the 4 success checks above (compiler + tests + WASM build). Additionally,
  run `dx serve` once at the end for a **manual smoke-check** (app loads, main pages render)
  as a safeguard against the unlikely case that "dead" code was reachable at runtime —
  **not a hard gate**. WASM *build* only proves it compiles, not that it runs.

### Claude's Discretion
- Per-symbol delete-vs-keep judgment during implementation, applying the D-03 rule.
- Ordering: run `cargo fix` first for the 14 auto-fixable, then manual dead-code + deprecated
  migration.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` (Phase 38 block, HYG-01/HYG-02) — the fixed scope and success criteria.

### Build/lint conventions
- `shifty-backend/CLAUDE.md` — clippy is a hard gate for the backend; dioxus is a separate
  workspace excluded from CI clippy; every gate must run `cargo clippy --workspace -D warnings`.
- `shifty-backend/shifty-dioxus/CLAUDE.md` — frontend build conventions and WASM build-gate
  (`cargo build --target wasm32-unknown-unknown`).

No new external specs/ADRs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Warning inventory (from `cargo build`, 2026-07-01 — 50 total)
- **14 auto-fixable** (`cargo fix --bin "shifty-dioxus" -p shifty-dioxus`): unused imports,
  unused variables (`on_close`, `error_store`, `date_iso_format_clone1/2`), `mut` not needed.
- **2 deprecated:** `time::format_description::parse` → `parse_borrowed`.
- **~34 dead-code:** never-constructed enum variants (`SystemThemeChanged`, `SaveSalesPerson`,
  `LoadAllSalesPersonUserLinks`, `LoadAllUserSalesPersonLinks`, `LoadAllUserRoles`,
  `LoadTemplate`, `ClearSelection`, `ClearFilter`, `Sheet`, `Refresh`, `LoadWeekMessage`,
  `LoadBillingPeriod`, `Delete`); unused functions/methods (`slot_to_column_view_item_with_tooltips`,
  `register_user_to_slot`, `partition_nav_items`, `parse_time_input`, `load_slots`,
  `load_bookings`, `is_escape_key`, `has_sunday_slots`, `get_slots`, `get_bookings_for_week`,
  `get_absence_period`, `generate_custom_report`, `day_total_label`, `ColumnViewSlot`,
  `add_booking`, `id`, `as_str`, `from_str`); unused enums/structs/constants
  (`ModalMode`, `AddExtraHoursFormAction`, `WorkingSchedule`, `STORAGE_KEY`, `DARK_MEDIA_QUERY`);
  never-read fields (`field 0`).
- Named landmark from ROADMAP: `has_sunday_slots` at `state/shiftplan.rs:315`.

### Established Patterns / Constraints
- dioxus clippy is broken in the dioxus nix-shell (E0514); run it (if at all) from the
  **backend** nix-shell. Backend clippy is a hard `-D warnings` gate.
- Dev commands run via `nix develop ../` from `shifty-dioxus/` (not `nix-shell` — shell.nix broken).

### Integration Points
- None new — this phase only removes code / renames a deprecated call.

</code_context>

<specifics>
## Specific Ideas

- User emphasis: the important thing is `cargo build` is warning-free AND the frontend is
  actually **fixed** (dead code removed where sensible), not just suppressed.
- `dx serve` is understood as separate from the WASM *build*; a manual smoke-check with it is
  a safeguard, not a gate (D-10).

</specifics>

<deferred>
## Deferred Ideas

- **dioxus clippy cleanup** (~198 pre-existing lints) and adding dioxus to the CI clippy gate —
  explicitly out of scope for Phase 38; candidate for its own future phase.

</deferred>

---

*Phase: 38-frontend-build-hygiene*
*Context gathered: 2026-07-01*
