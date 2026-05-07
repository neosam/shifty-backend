---
phase: 6
slug: rest-types-unification-frontend-compile-through
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-07
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Quelle: `06-RESEARCH.md` §5 "Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo` (Standard Rust — kein pytest/jest) |
| **Config file** | none — Workspaces nutzen `Cargo.toml`/`Cargo.lock` |
| **Quick run command** | `cd shifty-dioxus && cargo check --target wasm32-unknown-unknown` |
| **Full suite command** | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` (Frontend) + `cd shifty-backend && cargo check --workspace && cargo test --workspace --no-run` (Backend) |
| **Estimated runtime** | ~30–60s Quick / ~3–5min Full |

> **Toolchain-Hinweis:** WASM-Builds laufen nur in `nix develop` (siehe Memory `reference_local_dev_commands.md`). Plans, die `cargo build --target wasm32-unknown-unknown` automatisieren, MÜSSEN `nix develop --command` verwenden oder `autonomous: false` setzen.

---

## Sampling Rate

- **After every task commit:** `cargo check --target wasm32-unknown-unknown` im jeweils geänderten Crate (Frontend) bzw. `cargo check --workspace` (Backend)
- **After every plan wave:** Voller WASM-Build im Frontend; `cargo check --workspace` im Backend
- **Before `/gsd-verify-work`:** Voller WASM-Build grün; `find . -type d -name rest-types | wc -l == 1`; Backend `cargo test --workspace --no-run` grün
- **Max feedback latency:** ~120s (Quick check)

---

## Per-Task Verification Map

> Wird vom Planner / nyquist-auditor gefüllt nachdem Plans existieren. Initial-Template:

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 6-XX-XX | XX | X | RT/FC-XX | — | N/A (Compile-Gate) | structure / compile | `<befehl>` | ✅ / ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

### Erwartete Verifikations-Befehle pro Requirement (aus RESEARCH §5)

| Req ID | Verhalten | Befehl |
|--------|----------|--------|
| RT-01 | `shifty-dioxus/Cargo.toml` hat `path = "../rest-types"` | `grep -E 'rest-types\s*=\s*\{\s*path\s*=\s*"\.\./rest-types"' shifty-dioxus/Cargo.toml` |
| RT-02 | Kein `shifty-dioxus/rest-types/`-Subtree | `test "$(find shifty-dioxus -type d -name rest-types \| wc -l)" -eq 0` UND `test "$(find . -type d -name rest-types -not -path './.git/*' \| wc -l)" -eq 1` |
| RT-03 | Alle 17 TOs + 4 Felder vom Frontend importierbar | Indirekt über FC-02 (Compile-Erfolg = Imports aufgelöst) |
| FC-01 | Match-Arme exhaustive | Compile-Erfolg unter `cargo build --target wasm32-unknown-unknown` (rustc enforces exhaustiveness) |
| FC-02 | WASM-Build grün | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` exit 0 |

---

## Wave 0 Requirements

> Wave 0 ist hier ein **Backend-Patch**, keine Test-Framework-Installation. Aus RESEARCH §5 + Landmine 2:

- [ ] Backend-Patch: `InvitationStatus`, `InvitationResponse`, `GenerateInvitationRequest` aus `rest/src/user_invitation.rs` nach `rest-types/src/lib.rs` migrieren (Pre-Req für Wave 1 Cargo-Swap)
- [ ] Backend bleibt grün: `cd shifty-backend && cargo check --workspace` exit 0 nach Migration
- [ ] Backend bleibt grün: `cd shifty-backend && cargo test --workspace --no-run` exit 0 nach Migration
- [ ] Optional Hygiene: `use shifty_utils::...`-Import in `rest-types/src/lib.rs` feature-gaten (`#[cfg(feature = "service-impl")]`)
- [ ] Repo-Inventur-Check: `find . -type d -name rest-types -not -path './.git/*'` listet vor Wave 1 noch BEIDE Verzeichnisse (Sanity)

*Existing infrastructure (cargo) covers all phase test execution. Wave 0 ist hier reine API-Vorbereitung.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Backend ↔ Frontend rest-types Drift = 0 (Field-by-Field) | RT-03 | Strukturelles Diff nicht trivial automatisierbar; Compile-Check verifiziert nur Imports | Nach Phase-Abschluss: `diff <(cd rest-types && cargo expand --lib 2>/dev/null) ...` ist optional. Plan-Reviewer prüft visuell, dass nach Phase 6 KEIN `shifty-dioxus/rest-types/`-Tree mehr existiert (RT-02 deckt das automatisch ab). |
| UI-SPEC zero-visual-delta-Check | UI-SPEC | Visuelles Verhalten nicht via Compile prüfbar | Nach Phase-Abschluss: `dx serve` startet (Phase 7); manuelle Sicht-Prüfung dass keine Tailwind-Klassen, RSX-Strukturen, Spacing oder Typografie geändert wurden. Phase 7 deckt dies ab — Phase 6 muss nur sicherstellen, dass keine *.rsx-Layouts oder Tailwind-Strings angefasst wurden. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (`InvitationStatus`-Familie)
- [ ] No watch-mode flags (cargo nicht im --watch)
- [ ] Feedback latency < 120s für Quick checks
- [ ] `nyquist_compliant: true` set in frontmatter — wird nach Plan-Generation und nyquist-Audit gesetzt

**Approval:** pending
