---
phase: 56
slug: wochen-cron-rollout-backfill-f4
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-26
---

# Phase 56 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust workspace) + `mockall` unit tests |
| **Config file** | none — workspace `Cargo.toml` + in-memory SQLite for integration tests |
| **Quick run command** | `cargo test -p service_impl <module>` (e.g. `voluntary_rebooking_scheduler`) |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~60–120 seconds (workspace) |

**Hard gate (project CLAUDE.md):** `cargo clippy --workspace -- -D warnings` MUST pass before any commit — `cargo test` does not run clippy.

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p service_impl <module>` for the touched module
- **After every plan wave:** Run `cargo test --workspace` + `cargo clippy --workspace -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite green + clippy clean
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

> Filled by the planner/executor per task. Anchored to the Validation Architecture in 56-RESEARCH.md.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 56-XX-XX | XX | 1 | REB-AUTO-XX | — | {expected behavior} | unit/integration | `cargo test -p service_impl <name>` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Critical validation assets that must exist before or alongside the feature code (from 56-RESEARCH.md `## Validation Architecture`):

- [ ] **Straddling-Golden-Snapshot-Fixture** (D-01 gate) — one billing period computed strictly PRE, strictly POST, and TRAVERSING `active_from`; `f32::to_bits()` byte-identity assertion. Precedent: v2.5 Phase 52 WOP-03 (8 fixtures). If any assertion fails → snapshot version bumps 12→13.
- [ ] **Three neutrality property-tests** (D-06) at `active_from=None`: `balance_before==balance_after`, `F1_Ist_before==after`, `F2_Soll_before==after`.
- [ ] **Cron idempotency test** — second run over an already-processed ISO week is a no-op (`ON CONFLICT DO NOTHING` via UNIQUE partial index from Phase 54).
- [ ] **Claim-on-Suggest race test** (D-02) — pending HR-suggestion blocks the cron slot for the same (SalesPerson, ISO-week).
- [ ] **Backfill dry_run vs apply test** (D-05) — `dry_run=true` writes nothing; `dry_run=false` persists `rebooking_batch(kind=auto_cron_backfill)`.

*If existing infrastructure covers a requirement, note it in the per-task map instead.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| FE Admin-Card render + Backfill/Dry-Run buttons | REB-AUTO-04 | Dioxus WASM UI interaction not covered by cargo test | Start FE, open Settings → „Automatische Freiwilligen-Umbuchung" card, trigger dry-run, verify preview table renders |

*Backend behaviors all have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
