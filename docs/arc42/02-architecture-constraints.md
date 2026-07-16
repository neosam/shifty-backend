# 2. Architecture Constraints

## 2.1 Technical Constraints

| Constraint | Background |
| --- | --- |
| **Rust across the whole stack** | Backend (Axum) and frontend (Dioxus → WASM) are Rust. Enables the shared DTO crate `rest-types` (compile-time API contract) but constrains hiring and library choice. |
| **SQLite as the only database** | Single-file DB, accessed via SQLx with compile-time-checked queries. Consequences: single-writer semantics (`BUSY` on parallel writes), no server-side DB, backup = file snapshot. See [05-transactions](../architecture/05-transactions.md). |
| **SQLx offline cache is mandatory** | CI and `nix build` run with `SQLX_OFFLINE=true`; after every new `query!`/`query_as!` a `cargo sqlx prepare --workspace` run and a committed `.sqlx/` are required. |
| **Nix is the production build system** | `flake.nix`/`default.nix` build the backend (features `mock_auth` or `oidc,json_logging`) and the WASM frontend bundle. `nix build` runs `cargo clippy -- --deny warnings` as a hard gate — code that passes `cargo test` can still fail the release build. |
| **Deployment target is NixOS** | The system ships as a NixOS module + systemd service via the sibling repo `shifty-nix`. `docker.nix` exists but is stale and not a supported path. See [07-deployment-view](07-deployment-view.md). |
| **Migrations are additive-only** | No down-migrations exist. Rollback of non-additive schema changes requires restoring a DB backup. |
| **Toolchain pins** | Dioxus CLI (`dx`) is pinned to 0.6.x (0.7 breaks the build); `wasm-bindgen` is pinned to the exact version matching the flake. `reqwest`/`reqwest_dav` must use `rustls-tls` (no `native-tls`) or the Nix build fails. |
| **TLS terminated externally** | The backend binds `127.0.0.1:3000` by default and assumes a reverse proxy for TLS and static frontend delivery. |

## 2.2 Organizational Constraints

| Constraint | Background |
| --- | --- |
| **Very small team / solo maintainer** | Optimizes for low coordination overhead: monolith, one repo per deployable, conventions enforced by compiler + clippy instead of review processes. |
| **Spec-driven change workflow (OpenSpec)** | Non-trivial changes go through `openspec/` proposals (proposal → design → tasks → specs). Archived changes double as decision records (see [chapter 9](09-architecture-decisions.md)). |
| **Version control via jj (Jujutsu)** | Commits and pushes are made with `jj`, not raw git ([onboarding](../onboarding/README.md)). |
| **Releases via `/release-version` skill** | SemVer since v2.0; version derived from the GSD milestone + git tags; `cli-update-version.sh` mechanizes the bump/tag/push. Deploy remains a manual `nixos-rebuild switch`. |
| **Frontend lives in-tree but builds separately** | `shifty-dioxus/` is excluded from the Cargo workspace (different target, different toolchain) yet versioned in the same repo to keep DTOs and API in lock-step. |

## 2.3 Conventions

| Convention | Rule |
| --- | --- |
| **Fat backend, thin client** | All domain logic (balance, conflicts, snapshots) lives in the backend. The frontend renders and does UX-only validation. Second clients must never re-implement a rule. |
| **Everything is a trait** | Services and DAOs are trait definitions (`service/`, `dao/`) with implementations in separate crates; `#[automock]` makes every boundary mockable. |
| **Soft-delete only** | No hard deletes; `deleted` timestamp column, readers filter `WHERE deleted IS NULL`. |
| **Three languages, always** | Every user-facing string exists in En, De, and Cs; full-sentence templates with placeholders, no fragment concatenation ([08-i18n](../architecture/08-i18n.md)). |
| **Documentation is bilingual** | Reference docs exist as `foo.md` (English) + `foo_de.md` (German). This arc42 documentation is intentionally English-only. |
| **ISO 8601 weeks** | All week arithmetic uses ISO calendar weeks (`year`, `calendar_week` 1..=53). |
| **`[To verify]` markers** | Unconfirmed statements in docs are tagged rather than silently asserted. |
