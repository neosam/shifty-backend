---
phase: 63
slug: personen-detailansicht-auspr-gungs-aktionen
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-09-20
---

# Phase 63 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `63-RESEARCH.md` § Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Backend: `cargo test` (workspace), `mockall` `#[automock]` unit tests + in-memory-SQLite integration tests in `service_impl/src/test/`. Frontend: `cargo test` in `shifty-dioxus`, SSR rendertests via `dioxus_ssr::render` over a prop-injected `VirtualDom` (pattern: `shifty-dioxus/src/page/user_management.rs:474+`) |
| **Config file** | none — workspace `Cargo.toml` at repo root; no separate test-framework config |
| **Quick run command** | `nix develop --command bash -c "cargo test -p service_impl person_detail"` (backend) / `nix develop --command bash -c "cargo test --manifest-path shifty-dioxus/Cargo.toml person_detail"` (frontend) |
| **Full suite command** | `nix develop --command bash -c "cargo test --workspace"` **then** `nix develop --command bash -c "cargo clippy --workspace -- -D warnings"` |
| **Estimated runtime** | ~90-180 s full suite + clippy; ~10-20 s targeted module run |

**Toolchain note:** `cargo` and `node` are NOT on `PATH`. Every command must be wrapped in
`nix develop --command bash -c "<cmd>"`.

**Clippy is a separate hard gate.** `cargo test` does NOT run clippy; `nix build` and CI both run
`cargo clippy -- --deny warnings`. Treat clippy as part of every test gate.

---

## Sampling Rate

- **After every task commit:** targeted `cargo test -p <crate> <module>` for the touched module **plus** `cargo clippy --workspace -- -D warnings`
- **After every plan wave:** `cargo test --workspace` (full suite) + clippy
- **Before `/gsd-verify-work`:** full suite green + clippy clean + `dx serve` smoke on `/person/{id}` for a linked entry AND a freshly created user-only entry
- **Max feedback latency:** ~180 s (full suite + clippy)

---

## Per-Task Verification Map

> Task IDs are filled in by the planner. Rows below are the requirement-level contract each plan
> must map its tasks onto.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | PADM-02 | T-63-V4 | Combined create gates on `admin` privilege first; server-side enforces "at least one toggle on" (D-63-07), not frontend-only | integration | `cargo test -p service_impl person_detail::create_combined` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-03 | T-63-V4 | Add-missing-facet gates on `admin`; creates + links only the missing object, existing object untouched | integration | `cargo test -p service_impl person_detail::add_facet` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-04 | T-63-V4 | Candidate-list read endpoints gate on `admin`; already-linked targets absent from the list | integration | `cargo test -p service_impl person_detail::candidates` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-04 | T-63-RACE | Already-linked target requested directly is still rejected with 409 (`DaoError::EntityAlreadyExists`, D-61-04) — regression guard | integration (existing) | `cargo test -p service_impl sales_person::set_user` | ✅ (Phase 61) | ⬜ pending |
| TBD | TBD | TBD | PADM-05 | T-63-V4 | Detail aggregate gates on `admin`; returns exactly the sections implied by `PersonKindTO`, no N+1 | integration | `cargo test -p service_impl person_detail::get_detail` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-05 | — | Section visibility derived from `PersonKindTO` only, never from role (PADM-F-02 stays out of scope) | unit (pure helper) | `cargo test --manifest-path shifty-dioxus/Cargo.toml person_detail_visibility` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-05 | — | Per-`PersonKindTO`-variant SSR rendertest: no empty placeholder sections | unit (SSR) | `cargo test --manifest-path shifty-dioxus/Cargo.toml person_detail` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-07 | T-63-ORPHAN | Remove-facet deletes the right side (hard vs soft) and cleans up the orphaned `person` row per D-63-01 | integration | `cargo test -p service_impl person_detail::remove_facet` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-07 | — | Delete-confirmation copy matches D-63-08's fixed per-side wording (Verkäufer-Seite vs. Benutzer-Seite) | unit (pure helper) | `cargo test --manifest-path shifty-dioxus/Cargo.toml delete_confirmation_copy` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/person_detail.rs` (new module, registered in `service_impl/src/test/mod.rs`) — covers PADM-02, PADM-03, PADM-04, PADM-05, PADM-07 at service level
- [ ] REST-level tests for the new `/person/*` routes (detail GET, combined-create POST, add-facet, candidate lists, remove-facet DELETE)
- [ ] `shifty-dioxus/src/page/person_detail.rs` `#[cfg(test)] mod tests` — pure section-visibility helper, pure delete-confirmation-copy helper, and per-`PersonKindTO` SSR rendertests (mirror `user_management.rs:506+`)
- [ ] No new test framework install — `cargo test` + `mockall` + `dioxus_ssr` are all already present

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `Dioxus.toml` proxy entry for `/person` actually routes | PADM-05 | Proxy config is dev-server runtime wiring, not reachable from `cargo test`. This exact missing-proxy bug class has shipped three times in this repo (RESEARCH Pitfall). | `nix develop --command bash -c "dx serve"` in `shifty-dioxus`, open `/person/{id}`, confirm the detail request returns 200 and not a dev-server 404 |
| Ausprägungs-abhängige Sektions-Sichtbarkeit im echten Browser | PADM-05 | Dioxus/WASM DOM rendering is not headless-verifiable (ROADMAP Verifikations-Hinweis, PITFALLS Nr. 10 — same class as the two `human_needed` overrides from v2.6). SSR rendertests cover the pure logic but not the live render. | Human-UAT: open a linked entry, a user-only entry, and a sales-person-only entry; confirm exactly the expected sections appear and no empty placeholders |
| Rückbau der zwei alten Anlege-Buttons (D-63-03) | PADM-02 | Visual/UX regression from the Phase 62 tab-merge; correctness is "no longer present and replaced by one dialog", which is a layout judgement | Human-UAT: `/user_management/` shows exactly one Anlege-Einstieg; „Neuen Verkäufer erstellen" and „Neuen Benutzer hinzufügen" are gone |
| Delete-confirmation copy reads correctly to a human | PADM-07 | Copy quality (does it actually tell the admin what disappears?) is not assertable | Human-UAT: trigger remove-facet and full-delete on both sides, read the confirmation text |

**Human-UAT is planned from the start**, not discovered at phase close — the ROADMAP explicitly
requires this (Verifikations-Hinweis). The planner MUST place a `checkpoint:uat` in the final wave.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] Clippy gate (`cargo clippy --workspace -- -D warnings`) present in every wave gate
- [ ] Human-UAT checkpoint planned in the final wave (not deferred to phase close)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
