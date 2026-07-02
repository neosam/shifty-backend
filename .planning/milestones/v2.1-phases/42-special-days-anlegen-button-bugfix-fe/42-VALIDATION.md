---
phase: 42
slug: special-days-anlegen-button-bugfix-fe
status: complete
nyquist_compliant: true
wave_0_complete: true
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

- [x] Extract the inline validity predicate (`settings.rs` ~387) into a pure fn (analog existing helpers `sd_type_to_select_value`, `is_duplicate_special_day`) with `#[cfg(test)]` unit tests → `is_special_day_form_valid` + 5 predicate tests
- [x] Pure fn modeling post-create retention (which signals reset vs retained), unit-tested → `SpecialDayForm` + `special_day_form_after_create` + 2 retention tests (incl. "valid stays true after create")
- [x] (best-effort) SSR/VirtualDom mount test — **justified skip** (D-42-06, Fall B), see below

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live: create day A → change date to B → create again without re-toggling the type dropdown; button stays active throughout | SDF-01 | D-25-06 class — WASM signal↔DOM; the pure-fn + (optional) SSR test are the hard gate | Optional browser smoke |

*The predicate + retention logic have automated pure-fn verification (the hard gate).*

---

## D-42-06 SSR/VirtualDom Mount Test — Justified Skip (Fall B)

**Decision:** No SSR/VirtualDom render test was added; the pure-fn tests from Task 1
(D-42-05) are the sole, sufficient hard gate.

**Why the component is not sensibly mountable without a live harness:**

1. **Global provider dependency.** `SettingsPage` reads three global signals at the
   top of its body — `I18N.read()`, `CONFIG.read()`, `AUTH.read()`. A bare
   `VirtualDom::new(SettingsPage)` has none of these initialized to a realistic
   state.
2. **Admin guard short-circuits before the button renders.** The component's first
   act is `is_admin = AUTH.read().auth_info.as_ref().map(|a| a.has_privilege("admin"))`.
   Without an initialized `auth_info` this is `false`, so the component returns the
   "Not authorized." branch and the Card-3 create form (and the Anlegen button) is
   never rendered — the assertion target would not exist.
3. **Network resources on mount.** The body starts `use_resource(get_toggle_enabled)`
   and `use_resource(api::get_special_days_for_year)`, both of which trigger HTTP
   calls against the backend proxy. Mounting without a live backend/config would
   exercise error paths unrelated to the button-disabled invariant.
4. **No existing VirtualDom harness.** The crate's `settings.rs` tests are all pure
   `#[cfg(test)]` unit tests over pure helpers (confirmed in 42-CONTEXT.md); adding a
   one-off mount harness for this isolated bugfix would be brittle and low-value.

**Coverage instead:** `special_day_form_valid_stays_true_after_create` proves the
button-enabled invariant at the predicate level (the exact `!sd_form_valid` input to
`Btn { disabled }`), and `special_day_form_retained_after_create` proves all three
fields are kept. The live WASM signal↔DOM behavior (D-25-06 class) remains the
optional browser smoke listed under Manual-Only Verifications.

---

## Validation Sign-Off

- [x] Validity predicate extracted to a tested pure fn; equals old inline logic
- [x] "stays true after create (fields retained)" case covered
- [x] Retention-policy pure fn tested (3 resets removed; year/restart kept)
- [x] Controlled-select D-06/D-08 not broken (field stays filled, not emptied)
- [x] SSR best-effort attempted or justified-skip documented (Fall B, above)
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved (2026-07-02, executor — pure-fn hard gate green, WASM build warning-free)
