# 9. Architecture Decisions

Since ~2026-03, significant decisions are made through the **OpenSpec
workflow**: `openspec/changes/` proposals (proposal → design → tasks → specs)
are the primary decision records, archived under
`openspec/changes/archive/` with the resulting capability specs in
`openspec/specs/`. New decisions should be recorded there; this chapter
lists the foundational decisions that predate OpenSpec (or span the whole
system) in lightweight ADR form.

## ADR-1: Rust monolith with trait-separated layers

**Decision:** One deployable Rust binary; REST/Service/DAO as separate crates
where interfaces (`service`, `dao`) and implementations (`service_impl`,
`dao_impl_sqlite`) are distinct crates wired only in `shifty_bin`.
**Rationale:** A small team gets compiler-enforced boundaries without
microservice ops cost; every boundary is mockable; implementations are
swappable (DB engine, HTTP framework) without touching consumers.
**Consequences:** DI wiring in `main.rs` is large (~1500 lines) and manual;
adding a service means touching several crates (accepted trade-off).

## ADR-2: SQLite instead of a database server

**Decision:** SQLite as the only persistence, via SQLx with compile-time
checked queries.
**Rationale:** Deployment scale is a single small company instance;
zero-administration, single-file backup, in-memory variant for integration
tests, compile-time SQL validation.
**Consequences:** Single-writer semantics (`BUSY` under write contention),
no horizontal scaling, backup/restore is file-based, `.sqlx/` cache must be
maintained. Accepted for the target scale; revisit if multi-instance
operation is ever needed.

## ADR-3: Fat backend, thin client — with a shared DTO crate

**Decision:** All domain logic server-side; the frontend consumes `rest-types`
directly as a Rust dependency.
**Rationale:** Balance/conflict/snapshot rules must exist exactly once
(quality goal 1); second clients get the same guarantee via REST + OpenAPI.
Sharing the DTO crate turns API drift into a compile error instead of a
runtime bug.
**Consequences:** Frontend is locked into Rust/WASM; backend DTO changes force
a frontend rebuild (wanted); wire format changes are always deliberate.

## ADR-4: Dioxus/WASM frontend

**Decision:** SPA in Dioxus 0.6 (Rust → WASM), Tailwind for styling, signal
state, custom i18n.
**Rationale:** Same language across the stack, `rest-types` sharing (ADR-3),
single-developer efficiency.
**Consequences:** Pinned `dx` 0.6.x / `wasm-bindgen` versions; smaller
ecosystem than JS frameworks; dev-proxy entries in `Dioxus.toml` per endpoint
prefix (known footgun); frontend excluded from the backend workspace and its
clippy gate.

## ADR-5: Compile-time auth modes (`mock_auth` vs `oidc`)

**Decision:** Authentication mode is a Cargo feature, not runtime config; the
service-facing context type is unified (`Option<Arc<str>>`).
**Rationale:** Dev machines need zero IdP setup; production code paths carry
no mock branches; misconfiguration cannot enable mock auth at runtime.
**Consequences:** Two build artifacts; RBAC deny paths are invisible in dev
(mock user is admin) → mandatory explicit deny tests (see 8.8).

## ADR-6: `Authentication::Full` internal bypass

**Decision:** Business-logic services read sub-aggregates with a privileged
`Full` context after the outer call authenticated the user.
**Rationale:** Avoids threading user permissions through every internal read
and avoids N redundant permission checks per report.
**Consequences:** `Full` in a REST handler would be a critical vulnerability;
usage is restricted by convention + review + edge-case doc (§6). The
trade-off is documented rather than type-enforced (candidate for future
hardening).

## ADR-7: Composable transactions via `Option<Transaction>`

**Decision:** Every service method takes `Option<Transaction>`; ref-counted
commit at the outermost owner.
**Rationale:** Multi-service operations (snapshot, carryover-for-all,
re-points) need atomicity without a global transaction manager or async-local
magic.
**Consequences:** Boilerplate parameter on every method; misuse (forgotten
`tx.clone()` pass-through) compiles but breaks atomicity — covered by
convention and tests.

## ADR-8: Write-once billing snapshots with schema versioning

**Decision:** Billing periods persist frozen per-employee metrics; every row
carries `snapshot_schema_version` (bumped on any formula/value-type/input
change); only the latest period is deletable.
**Rationale:** Payout stability (quality goal 2): later corrections must not
change past payouts, and a validator must distinguish "computed under old
rules" from "data bug".
**Consequences:** Formula evolution requires disciplined version bumps
(forgotten bump = false bug reports); snapshots store aggregates only, row
detail stays live. (OpenSpec: `billing-period-snapshot-versioning`.)

## ADR-9: Persisted year-end carryover

**Decision:** Year-end balances are persisted per employee
(`employee_yearly_carryover`) by a scheduled job and used as the next year's
starting point.
**Rationale:** Reports would otherwise recompute all history; carryover makes
report cost proportional to one year.
**Consequences:** Retroactive edits in closed years can drift from persisted
carryover (documented edge case; convention: don't edit closed years —
re-runs of the current-year job absorb recent changes).

## ADR-10: Range-based absences replacing single-day extra hours (cutover)

**Decision:** Absences moved from single-day `extra_hours` rows to
`absence_period` ranges whose hours are derived at read time from the
contract active on each day; legacy rows stay authoritative for pre-cutover
history and non-absence categories.
**Rationale:** Contract changes no longer require re-editing absence rows;
no double entry; half-days and category-priority overlaps become first-class.
**Consequences:** Readers must aggregate **both** sources forever (or until an
explicit full conversion); conversion is opt-in and audited. (Domain doc:
[absence-system](../domain/absence-system.md).)

## ADR-11: Nix as build and deployment foundation

**Decision:** Flake-based builds for backend and frontend; clippy
`--deny warnings` inside the build; deployment via NixOS module in
`shifty-nix` pinning an exact version; manual `nixos-rebuild switch`.
**Rationale:** Reproducibility (quality goal 4), enforced lint hygiene,
trivially auditable rollbacks via pin history.
**Consequences:** Nix knowledge required for ops; rustls-only TLS in
dependencies; the deploy step stays consciously manual (small-scale,
human-supervised).

## ADR-12: Two-tier service layer

**Decision:** Services are classified as Basic (one aggregate, DAOs only) or
Business-Logic (composes services); dependency edges only point from BL to
Basic/BL, acyclically.
**Rationale:** Prevents cyclic DI, keeps `main.rs` construction order
deterministic, gives every aggregate exactly one write owner.
**Consequences:** Cross-aggregate features need a (sometimes new) BL service
rather than a shortcut dependency; classification discipline required
([02-service-tiers](../architecture/02-service-tiers.md)).

## ADR-13: No email subsystem

**Decision:** Shifty renders text (templates) but never sends email;
invitation links and reports are delivered out-of-band.
**Rationale:** Avoids SMTP configuration, deliverability, and secret-handling
complexity for a feature the target scale can do manually.
**Consequences:** Invitation/report distribution is a manual step; revisit if
the user base grows.
