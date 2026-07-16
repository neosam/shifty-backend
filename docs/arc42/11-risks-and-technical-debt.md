# 11. Risks and Technical Debt

Ordered roughly by (impact × likelihood). Items marked *(edge-cases §n)* are
elaborated in [`docs/domain/edge-cases.md`](../domain/edge-cases.md).

## 11.1 Risks

| # | Risk | Impact | Mitigation / status |
| --- | --- | --- | --- |
| R1 | **Carryover drift**: retroactive edits in a closed year don't invalidate persisted carryover; live report and carryover diverge silently. *(§1)* | Wrong balances propagate into following years. | Convention "never edit closed years"; current-year job re-runs absorb recent changes. Open idea: drift detector comparing carryover vs recomputation. |
| R2 | **Dual absence sources forever**: every consumer must aggregate legacy `extra_hours` *and* `absence_period` or lose/double-count history. *(§2)* | Silent misreporting if a new consumer forgets one source. | Documented invariant; explicit audited conversion exists (`AbsenceConversionService`). Long-term: complete the conversion and retire the read path. |
| R3 | **`Authentication::Full` misuse**: one `Full` in a REST handler bypasses all RBAC. *(§6)* | Full data exposure. | Convention + review + docs; not type-enforced. Hardening candidate: separate internal trait/type so handlers can't construct `Full`. |
| R4 | **Forgotten snapshot version bump** on formula change. *(§3)* | Old snapshots misdiagnosed as data bugs; audit value degraded. | The bump rule is tracked in GSD state/phase planning (potential bumps are pinned per phase); old-snapshot-read tests. No automated guard yet. |
| R5 | **`f32` money-adjacent arithmetic**: hours are `f32`; summation order and rounding are non-associative. *(§5)* | Cent/minute-level report discrepancies, user distrust. | Convention: display backend totals only. Debt: migrate to a fixed-point/decimal representation. |
| R6 | **SQLite single-writer**: write bursts (snapshot creation, carryover-for-all in one TX) block other writers (`BUSY`). *(§7)* | Sporadic 5xx/retries under concurrent editing. | Acceptable at current scale; keep long TX short, monitor. |
| R7 | **No down-migrations, backup story `[To verify]`**: rollback beyond additive changes needs a file snapshot that only a checklist step creates. | Data loss on failed deploy without snapshot. | Deploy checklist; debt: automate snapshot in the systemd pre-start or `shifty-nix`. |
| R8 | **WebDAV app token stored cleartext** in `pdf_export_config` (masked in API only). | Credential leak via DB file access. | Low exposure (single-host, file perms); debt: encrypt at rest or move to env/secret store. |
| R9 | **Timezone/DST semantics partially unverified** (`TIMEZONE` default UTC; DST transition weeks `[To verify]`). *(§4)* | Off-by-one-hour balances in DST weeks. | Flagged in docs; needs a decisive test + documented convention. |
| R10 | **In-process schedulers**: legacy `tokio-cron` fires the carryover job every minute (`"0 * * * * *"`); two scheduler libraries coexist; missed ticks vanish with the process. | Wasted work; confusion; silent job loss on crash. | Consolidate on `tokio-cron-scheduler`, sane cadence, persist last-run telemetry (PDF export already does). |

## 11.2 Technical Debt

| # | Debt | Note |
| --- | --- | --- |
| D1 | **Frontend lint debt**: ~198 pre-existing clippy warnings; frontend excluded from the CI clippy gate. | Burn down, then add the frontend to the gate. |
| D2 | **Dev-proxy footgun**: every new endpoint prefix needs a manual `[[web.proxy]]` entry in `Dioxus.toml` (forgotten twice already). | Candidate: generate proxy entries or use a catch-all prefix. |
| D3 | **Stale artifacts**: `docker.nix` references files that don't exist; `AUTHENTICATION.md` still documents the removed `MockContext`; `docs/ops/configuration.md` lists env names (`OIDC_ISSUER`, `PORT`) that differ from code (`ISSUER`, `SERVER_ADDRESS`); `docs/architecture/README.md` references `service-graph-traits.mmd` which is absent; the retired OpenSpec workflow still ships its `openspec/` tree and `.claude/skills/openspec-*` skills alongside the active GSD workflow. | Delete or fix; cheap wins. Mark `openspec/` clearly as archive (or remove the skills) so new contributors don't pick the wrong workflow. |
| D4 | **No pagination / rate limiting** on list endpoints. | Fine at current data volume; revisit before larger datasets or public exposure. |
| D5 | **In-memory OIDC session store** (`MemoryStore`): OIDC-layer sessions don't survive restarts (app sessions do, in SQLite). | Users re-authenticate after deploys; acceptable, but document or persist. |
| D6 | **`main.rs` DI wiring ~1500 lines**, fully manual. | Accepted (ADR-1); macro/codegen could reduce boilerplate if it grows further. |
| D7 | **Duplicate legacy paths**: raw `POST /booking/` bypasses the conflict-warning layer of `/shiftplan-edit/booking`; two extra-hours-era categories (`Unavailable`, absence types) overlap newer aggregates. | Deprecate legacy endpoints/categories once clients are migrated. |
| D8 | **Docs `[To verify]` markers** across auth/transactions/i18n/ops docs. | Periodically resolve markers against code (several were resolved while writing this arc42 doc — see D3). |
| D9 | **F14 voluntary rebooking partially shipped** (phases 54–56). | Finish or feature-flag off; avoid half-active data paths. |

## 11.3 Accepted Trade-offs (not debt)

Documented consciously, no action planned: SQLite over a DB server (ADR-2),
manual deploys (ADR-11), no email (ADR-13), monolith over services (ADR-1),
Rust/WASM frontend lock-in (ADR-4).
