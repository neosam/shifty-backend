---
phase: 56-wochen-cron-rollout-backfill-f4
plan: 05
subsystem: rebooking-backfill-frontend
tags: [rebooking, backfill, dioxus, wasm, settings, admin, i18n, toggle, f4, fat-backend-thin-client]

# Dependency graph
requires:
  - phase: 56-wochen-cron-rollout-backfill-f4
    plan: 04
    provides: "POST /admin/rebooking/backfill?from=YYYY-Www&dry_run={bool} + BackfillPreviewTO / BackfillPreviewItemTO wire DTOs (rest-types, shared with FE)"
  - phase: 51
    provides: "Card 2b shortday activation-date editor pattern (signal init + use_resource/use_effect load + save/clear spawn handlers via toggle-value)"
  - phase: 48
    provides: "pdf_export_config admin card (trigger-button spawn + result-banner pattern)"
provides:
  - "Settings Card 5 'Automatische Freiwilligen-Umbuchung' (admin-gated): Stichtag editor (voluntary_rebooking_auto_active_from) + one-time backfill (dry-run/apply + preview table)"
  - "/admin/rebooking Dioxus proxy (dx serve resolves the backfill POST)"
  - "api::post_rebooking_backfill + loader::{get,set}_voluntary_rebooking_active_from + loader::trigger_rebooking_backfill"
  - "state::rebooking::is_valid_iso_week_input (pure YYYY-Www gate) + BackfillPreviewItem render mapper"
  - "21 i18n Key variants (de/en/cs) for the F4 admin card"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Stichtag editor mirrors Card 2b 1:1 (toggle-value get/set/clear; toggle name string lives in exactly one place in loader.rs referencing voluntary_rebooking_gate::TOGGLE_NAME)"
    - "Backfill dry-run + apply both call the same loader::trigger_rebooking_backfill, differing only in the dry_run bool; the result is stored verbatim in a single preview signal"
    - "FE renders BackfillPreviewTO verbatim via a pure BackfillPreviewItem field-copy mapper — no arithmetic (Fat Backend, Thin Client, T-56-15)"
    - "The only FE logic is is_valid_iso_week_input (button-enable gate); the backend owns the final range/53-week validation (T-56-11)"
    - "Card admin-gated by the outer is_admin return in SettingsPage (no inner gate); the backend endpoint independently enforces HR-Admin (defense-in-depth, T-56-14)"
    - "Proxy added FIRST (Pitfall 5) so the backfill POST resolves under dx serve"

key-files:
  created: []
  modified:
    - shifty-dioxus/Dioxus.toml
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/state/rebooking.rs
    - shifty-dioxus/src/page/settings.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs

key-decisions:
  - "Toggle name string kept in exactly one place (loader.rs const VOLUNTARY_REBOOKING_ACTIVE_FROM_TOGGLE) with a comment pinning it to service_impl::voluntary_rebooking_gate::TOGGLE_NAME. The FE cannot import the backend const (rest-types is the only shared crate and does not re-export it), so a byte-exact mirror + reference comment is the boundary contract — same approach the shortday loader uses."
  - "is_valid_iso_week_input is a syntactic YYYY-Www gate only (permissive week upper bound 01..=53). The authoritative range/53-week-year check stays in the backend (parse_iso_week, Plan 04) — the FE gate only enables/disables the buttons, mirroring how Card 2b's is_valid_shortday_date_input gates Save without duplicating backend semantics."
  - "Dry-run and Apply share one preview signal and one loader fn; the dry_run bool is the only difference. The preview table renders identically for both, so the operator sees the same candidate breakdown whether previewing or applying."
  - "Stichtag editor reuses the existing is_valid_shortday_date_input ISO-date validator rather than adding a sibling — it is already a generic YYYY-MM-DD validator, so no new helper was needed for Section 1."

requirements-completed: [REB-AUTO-02, REB-AUTO-04]

coverage:
  - id: T1
    description: "is_valid_iso_week_input accepts well-formed YYYY-Www (incl. W01, permissive W53) and rejects malformed shapes (missing W, week 00/>53, 2-digit year, German date notation, natural language)"
    requirement: "REB-AUTO-04"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/state/rebooking.rs (is_valid_iso_week_input_accepts_wellformed, is_valid_iso_week_input_rejects_malformed)"
        status: pass
    human_judgment: false
  - id: T2
    description: "BackfillPreviewItem render mapper is a pure field copy (every field 1:1, no arithmetic) and preserves skip_reason"
    requirement: "REB-AUTO-04"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/state/rebooking.rs (backfill_item_mapper_is_pure_field_copy, backfill_item_mapper_preserves_skip_reason)"
        status: pass
    human_judgment: false
  - id: T3
    description: "All 21 F4 admin-card i18n keys present (non-empty, non-'??') in de/en/cs; German reference copy pinned"
    requirement: "REB-AUTO-02"
    verification:
      - kind: unit
        ref: "shifty-dioxus/src/i18n/mod.rs (i18n_voluntary_rebooking_keys_present_in_all_locales, i18n_voluntary_rebooking_keys_match_german_reference)"
        status: pass
    human_judgment: false
  - id: T4
    description: "Full shifty-dioxus WASM build clean + full test suite green after card integration"
    requirement: "REB-AUTO-04"
    verification:
      - kind: build
        ref: "cargo build --target wasm32-unknown-unknown (nix env)"
        status: pass
      - kind: unit
        ref: "cargo test -p shifty-dioxus (837 passed)"
        status: pass
    human_judgment: false

# Metrics
duration: 8min
completed: 2026-07-26
status: complete
---

# Phase 56 Plan 05: F4 Frontend Admin Surface (Settings Card + Backfill) Summary

**Adds the operator UI for F4 as a single admin-gated Settings card that (1) edits the `voluntary_rebooking_auto_active_from` Stichtag via the toggle-value endpoint (mirroring Card 2b 1:1) and (2) drives the Plan-04 `POST /admin/rebooking/backfill` endpoint with a dry-run/apply button pair, rendering the backend-computed `BackfillPreviewTO` verbatim in a per-candidate preview table — Fat Backend, Thin Client, no FE arithmetic. The `/admin/rebooking` Dioxus proxy is added first (Pitfall 5) so the POST resolves under `dx serve`, and all 21 user-visible strings ship in de/en/cs.**

## Performance

- **Duration:** ~8 min
- **Tasks:** 3
- **Files:** 9 modified (0 created in code; 1 planning `deferred-items.md` created)
- **New tests:** 6 (4 state/rebooking unit + 2 i18n); full dioxus suite 837 passed / 0 failed

## Accomplishments

- **`/admin/rebooking` proxy (`Dioxus.toml`):** added the `[[web.proxy]]` block next to `/rebooking` and `/rebooking-suggestions` — done first so the backfill POST is reachable in dev (Pitfall 5).
- **`api::post_rebooking_backfill` (`api.rs`):** `post_rebooking_backfill(config, from, dry_run) -> BackfillPreviewTO` builds `{backend}/admin/rebooking/backfill?from=..&dry_run=..` and POSTs with an empty body (all params in the query string), mirroring the existing reqwest `post().send()` pattern.
- **Loader wrappers (`loader.rs`):** `get_voluntary_rebooking_active_from` / `set_voluntary_rebooking_active_from` (get/set/clear via `api::*_toggle_value`, toggle name in one place referencing `voluntary_rebooking_gate::TOGGLE_NAME`) + `trigger_rebooking_backfill` (thin pass-through to the api client; no arithmetic).
- **`state/rebooking.rs`:** `is_valid_iso_week_input` — a pure, unit-tested `YYYY-Www` format gate (empty → false; week 01..=53) — plus a `BackfillPreviewItem` render mapper that is a pure field copy of `BackfillPreviewItemTO` (including `skip_reason`), so the card iterates a stable render type without recomputing anything.
- **Settings Card 5 (`settings.rs`):** admin-gated (outer `is_admin` return). Section 1 is the Stichtag editor (signals + `use_resource`/`use_effect` load + save/clear spawn handlers, reusing `is_valid_shortday_date_input`). Section 2 is the backfill: a `from`-week text input, a dry-run + apply button pair (both call `trigger_rebooking_backfill`, differing only in the `dry_run` bool), a running/error banner, a summary line (`total_candidates` / `already_processed`), and a preview table rendering every `BackfillPreviewItem` field (person, week, ist, soll, committed, excess, would_book, skip_reason) verbatim.
- **i18n (mod.rs + de/en/cs):** 21 `Key` variants + faithful copy in all three locales (German reference; cs `[ASSUMED]`, native-check deferred to manual verify) + 2 tests (all-locale presence + German-reference pin).

## Task Commits

1. **Task 1: proxy + api backfill POST + toggle/backfill loaders** — `3d732162` (feat)
2. **Task 3: i18n keys in de/en/cs** — `b886faa0` (feat)
3. **Task 2: Settings admin card + state/rebooking helpers** — `76598250` (feat)

(Task 3 committed before Task 2 because the card RSX references the i18n keys.)

## Files Modified

- `shifty-dioxus/Dioxus.toml` — `/admin/rebooking` proxy block.
- `shifty-dioxus/src/api.rs` — `post_rebooking_backfill` client.
- `shifty-dioxus/src/loader.rs` — toggle getter/setter for the Stichtag + `trigger_rebooking_backfill` wrapper + the single toggle-name const.
- `shifty-dioxus/src/state/rebooking.rs` — `is_valid_iso_week_input` + `BackfillPreviewItem` mapper + 4 tests.
- `shifty-dioxus/src/page/settings.rs` — Card 5 signals, handlers, and RSX.
- `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` — 21 keys + copy + 2 tests.

## Decisions Made

- **Toggle name mirrored, not imported.** `rest-types` (the only shared crate) does not re-export `voluntary_rebooking_gate::TOGGLE_NAME`, so the FE keeps a byte-exact copy in one `const` with a reference comment — identical to how the shortday loader mirrors its toggle name. A drift would surface as a runtime 404/empty-value, and the comment makes the coupling explicit for review.
- **FE validity gate is syntactic only.** `is_valid_iso_week_input` gates the buttons on a `YYYY-Www` shape (permissive week 01..=53); the authoritative range and 53-week-year check stays server-side (`parse_iso_week`, Plan 04). This mirrors Card 2b's Save-gate philosophy and keeps the FE from re-implementing backend semantics.
- **One preview signal for both dry-run and apply.** Both actions call the same loader fn and store into the same signal, so the operator sees the identical candidate breakdown table for a preview and an apply — no divergent rendering path.

## Deviations from Plan

None — the plan executed as written. The plan left the `state/rebooking.rs` mapper optional ("if convenient"); I added it because iterating a stable render type keeps the RSX clean and gave a natural home for the tested validity predicate, both of which the plan explicitly requested.

## Threat Model Compliance

| Threat ID | Category | Mitigation in this plan | Verified |
|-----------|----------|-------------------------|----------|
| T-56-14 | Elevation of Privilege (non-admin using the backfill card) | Card rendered only inside the `is_admin` return of `SettingsPage`; the backend endpoint independently enforces HR-Admin (Plan 04, 403) so the gate is defense-in-depth. | code: card lives after the `if !is_admin { return }` guard; backend 403 asserted in Plan 04's contract test |
| T-56-15 | Tampering (FE recomputing preview values → drift) | `BackfillPreviewItem::from` is a pure field copy; the card renders `voluntary_ist/soll/committed/excess/would_book/skip_reason` verbatim; the only FE logic is input validity. | `backfill_item_mapper_is_pure_field_copy` + `backfill_item_mapper_preserves_skip_reason` assert 1:1 copy |
| T-56-SC | Tampering (npm/cargo installs) | No new dependencies — no Cargo.toml/package.json change; the dioxus 0.6.x pin untouched. | git diff shows no manifest change |

No new externally-reachable server surface (the proxy only forwards to the existing Plan-04 endpoint); no schema/trust-boundary change.

## Verification

- `cargo build --target wasm32-unknown-unknown` (shifty-dioxus, nix env) — clean, no warnings.
- `cargo test -p shifty-dioxus rebooking` — 28 passed (incl. 6 new).
- `cargo test -p shifty-dioxus i18n` — 59 passed (incl. 2 new).
- `cargo test -p shifty-dioxus` — 837 passed / 0 failed (full suite green after integration).
- `cargo clippy -p shifty-dioxus --all-targets -- -D warnings` — my changed files are clean (settings.rs, state/rebooking.rs, i18n/*, loader.rs, api.rs). Two pre-existing errors in `pdf_export_config.rs` test code (unrelated to this plan) are logged as a deferred item, not fixed (SCOPE BOUNDARY).

**FE build note:** The WASM build was fully verified in the nix env (`nix-shell -p openssl pkg-config lld --run "cargo build --target wasm32-unknown-unknown"`), so the environment constraint in the plan gates was satisfied — no `cargo check`-only fallback was needed.

## Known Stubs

None. The card is wired to the real toggle-value endpoint (Stichtag) and the real `POST /admin/rebooking/backfill` endpoint (backfill); the preview table renders live `BackfillPreviewTO` data. No hardcoded/placeholder data paths.

## Deferred Items

- **Pre-existing clippy warnings in `shifty-dioxus/src/state/pdf_export_config.rs`** (test module, `clippy::field_reassign_with_default`, ~lines 184/267-268). NOT caused by this plan (file last changed in the v2.2 milestone commit `3ab61f1c`); a newer clippy revision now flags the old test helpers. Logged to `.planning/phases/56-wochen-cron-rollout-backfill-f4/deferred-items.md`. Note: this currently makes `cargo clippy -p shifty-dioxus --all-targets -- -D warnings` fail on those two errors — a follow-up (or `/gsd-audit-fix`) should convert them to struct-update syntax. My own files are clippy-clean.
- **cs.rs copy `[ASSUMED]`** — Czech translations follow the en/de convention but want a native-speaker check (manual verify), matching the existing cs.rs convention comment for prior phases.
- **Docs-Freshness-Gate:** this plan is FE-only; it touched no `docs/` trigger file (no `service/**`, `dao/**`, `migrations/`, `permission.rs`, or `main.rs`). No `docs/` update mandated by this diff; the F4 feature-doc update remains deferred to phase close / `/gsd-docs-update`.

## Next Phase Readiness

- F4 is now reachable end-to-end from the browser: an admin opens Settings → Card 5 → sets the Stichtag and/or runs a dry-run/apply backfill against the live endpoint. Plan 06 (rollout wiring / any remaining glue) can build on the shipped operator surface.
- **Manual verify (per 56-VALIDATION.md):** open Settings as an admin, expand Card 5, enter a from-week (e.g. `2026-W20`), click Preview, and confirm the preview table renders backend rows; then confirm the Stichtag Save/Clear round-trips.

---
*Phase: 56-wochen-cron-rollout-backfill-f4*
*Completed: 2026-07-26*

## Self-Check: PASSED

- All three task commits present in git history (`3d732162`, `b886faa0`, `76598250`).
- Modified files present on disk (Dioxus.toml, state/rebooking.rs, settings.rs, SUMMARY.md).
- Toggle name `voluntary_rebooking_auto_active_from` present in loader.rs; `/admin/rebooking` proxy present in Dioxus.toml.
- WASM build clean; `cargo test -p shifty-dioxus` 837 passed / 0 failed.
