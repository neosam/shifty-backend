# Deferred Items — Phase 23 Frontend Slot Paid-Capacity UI

## Pre-existing crate-wide clippy lints in `shifty-dioxus` (out of scope)

**Discovered during:** Plan 23-01, Task 3 (clippy gate).

**Status:** NOT fixed — out of scope (executor scope boundary: only auto-fix issues
directly caused by the current task's changes).

`cargo clippy --workspace -- -D warnings` in the `shifty-dioxus` crate reports **199
pre-existing clippy errors** spread across ~30 files (api.rs, loader.rs, js.rs,
component/atoms/*, component/*, page/*, state/*). These exist independently of the
Phase 23 change set and predate plan 23-01.

The 2 lints the plan's code introduced were fixed in-plan:
- `clippy::if_same_then_else` in `cell_background_class` (combined `discourage ||
  paid_overage` into one arm).
- `clippy::unnecessary_map_or` at the WeekCellSlot call site (`map_or(false, ...)` →
  `is_some_and(...)`).

Sample of the pre-existing lint categories (full crate-wide):
- clippy::redundant_pattern_matching
- clippy::wrong_self_convention (e.g. `from_hour(&self)`, `from_as_calendar_week(&self)`)
- clippy::unnecessary_cast (e.g. `calendar_week as u8` where already `u8`)
- clippy::manual_contains, collapsible_if, useless_conversion, unused_unit, type_complexity, ...

**Environment note:** the `shifty-dioxus/` dev shell ships `clippy 0.1.93`
(clippy-driver rustc 1.93.0) but `rustc 1.95.0`, so clippy cannot run there
(E0514 toolchain mismatch). Clippy was run from the backend-root `nix develop`
shell (matching rustc 1.93.0 + clippy 0.1.93) with openssl provided via
`OPENSSL_LIB_DIR`/`OPENSSL_INCLUDE_DIR` env overrides. This toolchain-pairing /
openssl quirk should be reconciled in the dioxus flake before clippy can be a
reliable local gate for this crate.

**Recommendation:** Schedule a dedicated "clippy cleanup" plan for the dioxus crate
(crate-wide, not phase-23-specific), and align the dioxus flake's clippy/rustc
versions + openssl env so the gate runs out-of-the-box.
