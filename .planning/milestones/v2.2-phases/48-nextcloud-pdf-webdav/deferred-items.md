# Deferred Items — Phase 48

Pre-existing issues found during Plan 48-01 execution, out of scope for this plan.

## Clippy `--all-targets` warning: `doc_lazy_continuation`

- **File:** `service_impl/src/test/shiftplan_edit_lock.rs:6`
- **Warning:** `//! 40-VALIDATION.md (T-40-01..17) + T-40-CR01 (CR-01-Regressions-Test).` — clippy wants indentation for doc list continuation.
- **Origin:** Phase 40 (T-40-CR01 test file), predates Phase 48.
- **Impact:** Only fires with `cargo clippy --workspace --all-targets -- -D warnings`. The plan-specified gate (`cargo clippy --workspace -- -D warnings`, without `--all-targets`) passes clean. `nix build` also runs the plan-specified gate (without `--all-targets`), so this does not break the release build.
- **Fix:** Indent the affected doc line by two spaces, or add `#[allow(clippy::doc_lazy_continuation)]` at module scope. Trivial — should be handled in a separate hygiene commit / Phase 46 (backend hygiene) if that phase covers doc-lint hygiene.
