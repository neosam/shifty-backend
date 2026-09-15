# Deferred Items — Phase 56

## ✅ RESOLVED (commit `d63c73aa`) — Pre-existing clippy warnings (out of scope, Plan 05)

**Resolution:** Fixed opportunistically at phase-close because these two errors
blocked the `nix build` clippy gate (`cargo clippy -p shifty-dioxus -- -D warnings`)
and Phase 56 closes the v2.6 milestone. Both test helpers converted to
struct-update syntax; `cargo clippy --all-targets -- -D warnings` on shifty-dioxus
is now clean (exit 0). Details of the original deferral below.

## Pre-existing clippy warnings (out of scope, Plan 05)

**File:** `shifty-dioxus/src/state/pdf_export_config.rs` (test module, lines ~184, ~267-268)

**Lint:** `clippy::field_reassign_with_default` — two test helpers build a
`PdfExportForm::default()` then reassign fields instead of using struct-update
syntax.

**Why deferred:** These are NOT caused by Plan 05's changes (the file was last
modified in commit `3ab61f1c`, the v2.2 milestone release — well before this
phase). A newer clippy revision now flags this old test code. Per the executor
SCOPE BOUNDARY rule, only issues directly caused by the current task are
auto-fixed; unrelated pre-existing lint failures are logged, not fixed.

**Note:** `cargo clippy -p shifty-dioxus --all-targets -- -D warnings` currently
fails on these two errors. My own changed files (settings.rs, state/rebooking.rs,
i18n/*, loader.rs, api.rs) are clippy-clean. A follow-up cleanup (or a
`/gsd-audit-fix`) should convert these two test helpers to struct-update syntax.
