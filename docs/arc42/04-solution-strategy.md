# 4. Solution Strategy

The fundamental decisions, each mapped to the quality goal it serves
(numbers from [chapter 1.2](01-introduction-and-goals.md#12-quality-goals)).
Details and rationale in [chapter 9](09-architecture-decisions.md).

| Decision | Approach | Serves |
| --- | --- | --- |
| **Layered monolith with trait boundaries** | REST (Axum) → Service → DAO → SQLite as separate crates; interfaces (`service`, `dao`) are pure trait crates, implementations (`service_impl`, `dao_impl_sqlite`) are swappable and only wired together in the composition root `shifty_bin/main.rs`. | 1, 3 |
| **Two-tier service layer** | *Basic services* manage exactly one aggregate and depend only on DAOs; *business-logic services* compose basic services. Prevents cyclic dependency injection and keeps construction order deterministic ([02-service-tiers](../architecture/02-service-tiers.md)). | 3 |
| **Fat backend, thin client** | Every domain rule is computed server-side exactly once; the Dioxus/WASM frontend renders results and shares the `rest-types` DTO crate with the backend, so API drift is a compile error. | 1, 5 |
| **Single source of truth for the balance** | One `ReportingService` computes `balance = worked − expected + carryover` for every consumer (HR report, employee view, snapshots, carryover). Nothing re-derives it. | 1 |
| **Write-once, versioned snapshots** | Billing periods freeze results with a `snapshot_schema_version`; formula changes bump the version instead of mutating history. Year-end balances are persisted as carryover so history never needs recomputation. | 2, also performance |
| **SQLite + SQLx compile-time queries** | Zero-ops single-file database matching the deployment scale; SQL checked at compile time against the schema (offline cache in CI). | 3, 4 |
| **Compile-time auth modes** | Feature flags `mock_auth` (dev: auto-admin, no IdP needed) vs `oidc` (prod: external IdP via `axum-oidc`). Services see a unified `Authentication<Option<Arc<str>>>` context either way; RBAC (users → roles → privileges) is checked in the service layer, never in handlers only. | 3, plus security |
| **Reproducible everything** | Nix flake builds both artifacts; clippy `--deny warnings` is part of the build; deployment pins an exact commit in `shifty-nix`; rollback = pin rollback. | 4 |
| **Composable transactions** | Every service method takes `Option<Transaction>`: open one if absent, join if present. Multi-service operations (booking migration, snapshot creation, carryover for all employees) stay atomic without a global transaction manager ([05-transactions](../architecture/05-transactions.md)). | 1, 2 |
| **Evolution via toggles, flags, and cutovers** | Risky semantic changes ship behind feature flags / date-based toggles (e.g. the extra-hours → absence-period cutover), with old semantics reconstructable while both sources coexist. | 1, 3 |
