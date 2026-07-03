---
phase: 46-backend-hygiene-i18n
plan: 01
subsystem: shifty-dioxus/i18n
tags: [i18n, test, imp-05, carryover-fix, phase-46]
status: complete
requires: []
provides:
  - IMP-05
affects:
  - shifty-dioxus/src/i18n/mod.rs (test-body only)
tech-stack:
  added: []
  patterns:
    - i18n-copy-pinning-test (Phase 32 / Phase 37-02 precedent — de.rs canonical, test aligned)
key-files:
  created: []
  modified:
    - shifty-dioxus/src/i18n/mod.rs
decisions:
  - "IMP-05 D-CONTEXT-Q1: de.rs is canonical, test adapts (not the other way round)"
metrics:
  duration: ~5 min
  completed: 2026-07-02
requirements: [IMP-05]
---

# Phase 46 Plan 01: IMP-05 — i18n Impersonation Test-Referenz an shipped 🥸-De-Copy angleichen

One-liner: Fix a red carry-over test (`i18n_impersonation_keys_match_german_reference`) by aligning its `assert_eq!` expectations to the shipped canonical De-copy (`"🥸 Agieren"` etc.) instead of the idealised prose (`"Als diese Person agieren"`) — per CONTEXT.md IMP-05 Q1 decision that de.rs is canonical.

## Result

Plan executed exactly as written. Single-file test-body edit in `shifty-dioxus/src/i18n/mod.rs`. `de.rs` untouched (impersonation lines 1154-1156 byte-identical).

### Verified RED → GREEN

**RED (before edit) — verified via `cargo test -p shifty-dioxus i18n_impersonation_keys_match_german_reference`:**

```
thread 'i18n::tests::i18n_impersonation_keys_match_german_reference' panicked at src/i18n/mod.rs:1581:9:
assertion `left == right` failed
  left: "🥸 Agieren"
 right: "Als diese Person agieren"
```

Exactly the failure the plan predicted.

**GREEN (after edit):** all 3 impersonation tests pass:

```
test i18n::tests::i18n_impersonation_keys_match_german_reference ... ok
test i18n::tests::i18n_impersonation_keys_present_in_all_locales ... ok
test i18n::tests::i18n_impersonation_banner_carries_user_placeholder ... ok

test result: ok. 3 passed; 0 failed
```

## Files Modified

- `shifty-dioxus/src/i18n/mod.rs` — updated the three `assert_eq!` expected values in `i18n_impersonation_keys_match_german_reference` (lines ~1578-1593) to match the shipped De copy from `de.rs:1154-1156`:
  - `Key::ImpersonateActAs` → `"🥸 Agieren"` (was: `"Als diese Person agieren"`)
  - `Key::ImpersonateBanner` → `"Du agierst als {user}."` (unchanged, already matched)
  - `Key::ImpersonateStop` → `"Impersonation beenden"` (unchanged, already matched)
- Added a `// Phase 46 (IMP-05): pins the shipped 🥸-De-Copy; test aligned to de.rs, not vice versa.` clarifier comment above the existing Pitfall-2 guard comment.
- Presence test (line 1538) and placeholder test (line 1562) and Contract-Help test (line 1600) unchanged.

## Gates

| Gate | Command | Result |
|------|---------|--------|
| IMP-05 primary test | `cargo test -p shifty-dioxus i18n_impersonation_keys_match_german_reference` | green |
| Neighbour impersonation tests | `cargo test -p shifty-dioxus i18n_impersonation` | 3/3 green |
| FE WASM build | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` | green |
| FE clippy `-D warnings` | `cargo clippy -p shifty-dioxus -- -D warnings` | green |
| FE clippy `--tests -D warnings` | `cargo clippy -p shifty-dioxus --tests -- -D warnings` | green |
| Full FE test suite | `cargo test -p shifty-dioxus` | 778 passed, 0 failed |
| Backend clippy sanity | `cargo clippy --workspace -- -D warnings` (nix develop) | green |
| de.rs constraint | `git diff shifty-dioxus/src/i18n/de.rs` filtered on impersonation | zero impersonation-line diff |

The `de.rs` file has a pre-existing (non-plan) working-tree modification in unrelated areas (`UserInvitationsLoadError`, `SettingsSpecialDaysDuplicateHint`). Grep-filtered inspection confirms **no impersonation lines** were touched by this plan.

## Deviations from Plan

None — plan executed exactly as written.

## Todos Closed

- `2026-07-02-i18n-impersonation-key-test-mismatch.md` moved from `.planning/todos/pending/` to `.planning/todos/completed/`.

## Self-Check: PASSED

- FOUND: `shifty-dioxus/src/i18n/mod.rs` (modified — verified via `cargo test`)
- FOUND: `.planning/phases/46-backend-hygiene-i18n/46-01-SUMMARY.md` (this file)
- FOUND: `.planning/todos/completed/2026-07-02-i18n-impersonation-key-test-mismatch.md`
- Constraint verified: de.rs impersonation lines 1154-1156 byte-identical (grep -filter empty)
