---
phase: 54
slug: data-model-voluntary-stats
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-07-06
last_updated: 2026-07-06
plan_count: 6
---

# Phase 54 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (workspace) + `cargo clippy -- -D warnings` + `cargo sqlx prepare --workspace` |
| **Config file** | `shifty-backend/Cargo.toml` (workspace root) |
| **Quick run command** | `cargo test --workspace --lib -- --quiet` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| **Estimated runtime** | ~60–120 s (quick) / ~180–240 s (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --workspace --lib -- --quiet` (fast pfad-scoped where possible)
- **After every plan wave:** Run full suite (`cargo test --workspace && cargo clippy --workspace -- -D warnings`)
- **Before `/gsd-verify-work`:** Full suite green + `cargo sqlx prepare --workspace` idempotent + `nix build` (Clippy-Gate) green
- **Max feedback latency:** 240 s

---

## Per-Task Verification Map

Diese Tabelle wird beim Planning-Wrap-Commit pro Task in PLAN.md aktualisiert. Die REQ-IDs stammen aus ROADMAP.md §Phase 54 (VOL-STAT-01/02, VOL-ACCT-01/02/03).

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 54-01-01 | 01 | 1 | — | — | Migration `rebooking_batch` schemakonform | integration | `cargo test -p dao_impl_sqlite migration_rebooking_batch` | ❌ W0 | ⬜ pending |
| 54-01-02 | 01 | 1 | — | — | Migration `rebooking_batch_entry` schemakonform + FK | integration | `cargo test -p dao_impl_sqlite migration_rebooking_batch_entry` | ❌ W0 | ⬜ pending |
| 54-01-03 | 01 | 1 | — | — | `ALTER extra_hours ADD source` — Default `'manual'`, alte Rows Backfill | integration | `cargo test -p dao_impl_sqlite migration_extra_hours_source` | ❌ W0 | ⬜ pending |
| 54-01-04 | 01 | 1 | — | — | Toggle-Seed `voluntary_rebooking_auto_active_from` idempotent | integration | `cargo test -p dao_impl_sqlite seed_voluntary_rebooking_toggle` | ❌ W0 | ⬜ pending |
| 54-02-01 | 02 | 2 | — | — | `RebookingBatchService` CRUD + Basic-Tier-Contract | unit | `cargo test -p service_impl rebooking_batch::` | ✅ | ✅ green |
| 54-02-02 | 02 | 2 | — | — | UNIQUE-Constraint respektiert (Service-Pre-Check → EntityAlreadyExists) | unit | `cargo test -p service_impl rebooking_batch::create_unique_conflict_maps_to_already_exists` | ✅ | ✅ green |
| 54-03-01 | 03 | 2 | VOL-STAT-01 | — | `voluntary_hours_per_contract_week` — Nenner enthält 0-h-Verträge | unit | `cargo test -p service_impl voluntary_stats::contract_weeks_zero_expected_counts_d_f1_01` | ✅ | ✅ green |
| 54-03-02 | 03 | 2 | VOL-ACCT-01 | — | `committed_voluntary_target_for_year` — pro-rata Mid-Week-Wechsel | unit | `cargo test -p service_impl voluntary_stats::f2_soll_prorata_midweek_change_d_f2_01` | ✅ | ✅ green |
| 54-03-03 | 03 | 2 | VOL-ACCT-03 | — | Property-Test „Rebooking-Neutralität": Marker-Row invariant für F1-Ist + F2-Soll | property | `cargo test -p service_impl voluntary_stats::f1_ist_rebooking_pair_invariant_vol_acct_03` | ✅ | ✅ green |
| 54-03-04 | 03 | 2 | VOL-STAT-02, VOL-ACCT-02 | — | HR-Only-DTO-Redaction: HR→`Some`, Non-HR→`None` | unit | `cargo test -p service_impl voluntary_stats::service_tests::service_non_hr_returns_all_none_vol_stat_02` | ✅ | ✅ green |
| 54-04-01 | 04 | 3 | VOL-STAT-01, VOL-ACCT-01 | — | REST-Endpoint liefert `VoluntaryStatsTO` (HR-Auth) / `None`-Redaction (Non-HR) | integration | `cargo test -p rest voluntary_stats` | ❌ W0 | ⬜ pending |
| 54-04-02 | 04 | 3 | — | — | OpenAPI-Schema aktualisiert; `#[utoipa::path]` präsent | unit | `cargo test -p rest openapi_voluntary_stats` | ❌ W0 | ⬜ pending |
| 54-05-01 | 05 | 3 | VOL-STAT-01/02, VOL-ACCT-01/02 | — | FE-Row „Freiwillige Stunden — Ist / Soll / Δ" HR-gated | manual | Browser-Test (get_page_text + find), Memory `reference_dioxus_browser_verify_reports` | ❌ W0 | ⬜ pending |
| 54-05-02 | 05 | 3 | — | — | `Dioxus.toml` `[[web.proxy]]` für neuen Endpoint (falls dediziert) | source | `grep '/voluntary-stats' shifty-dioxus/Dioxus.toml` | ❌ W0 | ⬜ pending |
| 54-05-03 | 05 | 3 | — | — | i18n de/en/cs Row-Labels vorhanden | source | `grep -l 'voluntary_stats_ist' shifty-dioxus/i18n/{de,en,cs}/*.ftl` | ❌ W0 | ⬜ pending |
| 54-06-01 | 06 | 4 | — | — | Docs-Freshness: `docs/features/F14-rebooking.md` + `_de.md` neu; `02-service-tiers.md` + `03-data-model.md` synchron | source | `test -f docs/features/F14-rebooking.md && test -f docs/features/F14-rebooking_de.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Migrations-Test-Harness in `dao_impl_sqlite/tests/` — vorhanden (Präzedenz v2.5). Neue Test-Files:
  - [ ] `dao_impl_sqlite/tests/migration_rebooking_batch.rs`
  - [ ] `dao_impl_sqlite/tests/migration_rebooking_batch_entry.rs`
  - [ ] `dao_impl_sqlite/tests/migration_extra_hours_source.rs`
  - [ ] `dao_impl_sqlite/tests/seed_voluntary_rebooking_toggle.rs`
- [ ] Property-Test-Framework: kein neuer Crate — Fixture-Test in `service_impl/src/test/voluntary_stats.rs` (Präzedenz VAA-04)
- [ ] `cargo sqlx prepare --workspace` nach jeder neuen `sqlx::query!`/`query_as!` (MEMORY `reference_sqlx_prepare_after_new_query`)
- [ ] `.sqlx/`-Diff mit-committen im gleichen Wrap-Commit
- [ ] Docs-Freshness-Files template (kann leer sein, wird in Wave 4 gefüllt):
  - [ ] `docs/features/F14-rebooking.md` + `F14-rebooking_de.md`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| FE-Row „Freiwillige Stunden — Ist / Soll / Δ" nur für HR-Auth sichtbar | VOL-STAT-02, VOL-ACCT-02 | WASM-Report-Screenshots time-outen (MEMORY `reference_dioxus_browser_verify_reports`). E2E via Dioxus-Browser-Automation. | 1. `nix develop`, backend + dx-serve starten. 2. Login als HR: URL `/employees/:id/:year` — 3 zusätzliche Werte sichtbar. 3. Login als Sales/Shiftplanner: Row NICHT sichtbar (Nullable-Guard). 4. `get_page_text` + `find` (nicht Screenshot). |
| i18n-Wording in cs korrekt (kontextuell) | VOL-STAT-01, VOL-ACCT-01 | Tschechische Übersetzung braucht native Verifikation | Bei Zweifel Domain-User (HR) fragen; sonst Übernahme nach EN/DE mit `[ASSUMED]`-Kommentar in RESEARCH.md §D-5 dokumentiert. |
| Docs-EN+DE-Konsistenz | — | Struktureller Gleichlauf (gleiche Sektionen, gleiche Diagramme) — nicht rein textuell prüfbar | Diff-Check zwischen `F14-rebooking.md` und `F14-rebooking_de.md`: gleiche H1/H2, gleiche Diagramm-Referenzen. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies (Manual-Row 05-01 ist als Ausnahme markiert — WASM-Screenshot-Constraint)
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify (Wave 3 mischt REST + FE — REST ist automatisiert, FE manuell → keine 3er-Kette)
- [ ] Wave 0 covers all MISSING references (siehe Wave 0 Requirements oben)
- [ ] No watch-mode flags in test commands
- [ ] Feedback latency < 240s
- [ ] `nyquist_compliant: true` set in frontmatter (Planer setzt beim Wrap-Commit)

**Approval:** approved 2026-07-06 — 6 Pläne erstellt (54-01..54-06), alle Task-IDs in der Per-Task-Map dienen weiterhin als Nyquist-Referenz. Die tatsächlichen Task-Namen im Executor folgen den Namen aus den PLAN-Dateien (Task 1..N pro Plan) und werden beim Wave-Merge gegen diese Tabelle abgeglichen. Wave 0 (Test-Dateien wie `service_impl/src/test/rebooking_batch.rs`, `service_impl/src/test/voluntary_stats.rs`, `rest/tests/voluntary_stats.rs`) wird von Plan 02 Task 5 + Plan 03 Task 1 + Plan 04 Task 3 erledigt.
