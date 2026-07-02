---
phase: 42
slug: special-days-anlegen-button-bugfix-fe
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-02
---

# Phase 42 — Validation Strategy

> Isolated FE-only state bugfix. Test strategy fully specified by D-42-05/06.
> Research skipped (well-understood single-file bugfix, per plan-phase guidance).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test -p shifty-dioxus` (pure-fn unit; optional SSR) |
| **Config file** | none |
| **Quick run command** | `cargo test -p shifty-dioxus special_day` |
| **Full suite command** | `cargo test -p shifty-dioxus` |
| **Estimated runtime** | ~30–60 seconds |

---

## Sampling Rate

- **After every task commit:** `cargo test -p shifty-dioxus special_day` + `cargo build --target wasm32-unknown-unknown`
- **Before `/gsd-verify-work`:** Full FE suite green (except the documented pre-existing `i18n_impersonation_keys_match_german_reference`) + backend `cargo clippy --workspace -- -D warnings` green
- **Max feedback latency:** ~60 seconds

---

## Requirements → Test Map

| Req | Behavior | Test type | Command | Wave 0? |
|-----|----------|-----------|---------|---------|
| SDF-01 | Validity predicate extracted to a PURE fn; equals the old inline `date non-empty && type is Some && (type≠ShortDay ‖ time non-empty)` | pure-fn unit (HARD gate, D-42-05) | `cargo test -p shifty-dioxus special_day` (validity predicate cases) | new |
| SDF-01 | Predicate stays `true` for a filled form AFTER create (fields retained) — button stays enabled | pure-fn unit (HARD gate) | same | new |
| SDF-01 | Post-create retention policy: the 3 field resets are NOT applied; `sd_year`/`sd_resource.restart` ARE (modeled as a pure fn) | pure-fn unit | same | new |
| SDF-01 | Button not disabled after mount (rendered form) | SSR render (BEST-EFFORT, D-42-06) | `cargo test -p shifty-dioxus` | new IF mountable |

---

## Wave 0 Requirements

- [ ] Extract the inline validity predicate (`settings.rs` ~387) into a pure fn (analog existing helpers `sd_type_to_select_value`, `is_duplicate_special_day`) with `#[cfg(test)]` unit tests
- [ ] Pure fn modeling post-create retention (which signals reset vs retained), unit-tested
- [ ] (best-effort) SSR/VirtualDom mount test asserting the "Anlegen" button is not `disabled` after mount — only if the component mounts without live backend/config; otherwise document a justified skip (D-42-06)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live: create day A → change date to B → create again without re-toggling the type dropdown; button stays active throughout | SDF-01 | D-25-06 class — WASM signal↔DOM; the pure-fn + (optional) SSR test are the hard gate | Optional browser smoke |

*The predicate + retention logic have automated pure-fn verification (the hard gate).*

---

## Validation Sign-Off

- [ ] Validity predicate extracted to a tested pure fn; equals old inline logic
- [ ] "stays true after create (fields retained)" case covered
- [ ] Retention-policy pure fn tested (3 resets removed; year/restart kept)
- [ ] Controlled-select D-06/D-08 not broken (field stays filled, not emptied)
- [ ] SSR best-effort attempted or justified-skip documented
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
