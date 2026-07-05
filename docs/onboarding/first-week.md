# First Week as a Dev on Shifty

A pragmatic guide from a blank laptop to your first merge.

## Day 1 — Setup

### Toolchain

- **Rust:** Stable, via `rustup`. The backend workspace pins the version
  in `rust-toolchain.toml` (if present).
- **Nix:** For reproducible builds. `nix develop` gives you a shell with
  all tools (`sqlx`, `cargo-watch`, etc.).
- **`sqlx-cli`:** Available in the `nix develop` shell. If you work outside
  of it: `cargo install sqlx-cli`.
- **`dx` (Dioxus CLI):** Version **0.6.x** — pinned in `flake.nix`.
  Do not update to 0.7.x, otherwise the frontend will not start.

### Clone the Repo

```bash
git clone <repo> && cd shifty-backend
```

This is the active monorepo. Do not clone the external `shifty-dioxus/` —
that is legacy.

### Start the Backend

```bash
cd shifty-backend
cp env.example .env         # adjust if needed
nix develop                 # shell with tools
sqlx database reset --source migrations/sqlite  # ⚠ destructive, first-run only
cargo run                   # backend on port 3000
```

**Careful:** `sqlx database reset` wipes the DB. For incremental migrations
later: `sqlx migrate run --source migrations/sqlite`.

### Start the Frontend

In a second terminal:

```bash
cd shifty-backend/shifty-dioxus
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch  # terminal A
# in terminal B:
dx serve --hot-reload       # frontend on port 8080
```

Open `http://localhost:8080`. In dev mode (`mock_auth`) you are
automatically logged in as admin.

### Sanity Check

- Open the shiftplan.
- Open Employees.
- If both pages show something, the stack is running.

## Day 2 — Understand the Architecture

Read in this order:

1. [Layered Architecture](../architecture/01-layered.md) — Why REST →
   Service → DAO → SQLite.
2. [Service Tiers](../architecture/02-service-tiers.md) — Basic vs
   Business-Logic. Very important so you do not create cyclic deps.
3. [Transactions](../architecture/05-transactions.md) — The
   `Option<Transaction>` pattern.
4. [Auth](../architecture/04-auth.md) — especially the `Full` bypass.
5. [Testing](../architecture/07-testing.md) — Mockall, in-mem SQLite,
   clippy gate.

## Day 3 — Understand the Domain

Read:

1. [Glossary](../domain/glossary.md) — the terminology.
2. [Time Accounting](../domain/time-accounting.md) — how balance is
   computed. Core knowledge.
3. [Billing Period](../domain/billing-period.md) — the snapshot contract.
4. [Absence System](../domain/absence-system.md) — v1.0+ range-based.

Then click through the frontend pages and understand which domain sits
behind which UI.

## Day 4 — First Small Change

Pick a small task — e.g. change a text label or add a small field. Use the
flow:

1. `cargo build --workspace` — make sure everything compiles.
2. Make the change.
3. `cargo build && cargo test` — green?
4. `cargo clippy --workspace -- -D warnings` — clippy green? (**Mandatory**,
   otherwise `nix build` fails.)
5. If you changed SQL: `cargo sqlx prepare --workspace` and commit the
   `.sqlx/` directory along.
6. `jj commit -m "..."` — this repo runs on jj (see
   `CLAUDE.local.md`).
7. Push.

## Day 5 — Conventions and Edge Cases

- [Edge Cases](../domain/edge-cases.md) — **read through once completely**.
  Do not memorize, but know where to find them.
- Root `CLAUDE.md` — the short form of the conventions.
- `shifty-backend/CLAUDE.md` — backend specifics (service tier rules,
  snapshot versioning contract, clippy gate).

## Common Mistakes in Week 1

1. **`cargo test` passes, `nix build` fails.** Almost always clippy
   warnings. Run `cargo clippy --workspace -- -D warnings` before pushing.
2. **New endpoint returns 404 in dev.** You created the backend endpoint
   but `shifty-dioxus/Dioxus.toml` has no `[[web.proxy]]` entry for it.
3. **New query, build fails in CI.** You used `query!`/`query_as!` but
   forgot to run `cargo sqlx prepare --workspace`.
4. **UI change looks good but the backend has nothing.** The roundtrip
   was not tested — the edit path is not the create path. Test it.
5. **Auth error after switching mock_auth.** OIDC mode has different
   roles. If you want to test deny paths: write an explicit unit test
   with an auth context that is not admin.
6. **`Authentication::Full` in a REST handler.** Catastrophic auth
   bypass. Never do this.

## When Something Hangs

- **Backend does not start:** `.env` missing? DB URL wrong? Migrations
  not run?
- **Frontend shows a blank page:** Tailwind watcher not running? dx CLI
  on 0.7.x? Style path in `Dioxus.toml` wrong?
- **Test hangs:** In-memory DB not leaked; probably a transaction that
  never commits.

## Slash Commands

The repo uses GSD (`.planning/`). When you work in Claude Code, skills like
`/gsd-progress`, `/gsd-plan-phase`, `/release-version` are available.
`/gsd-progress` gives you a situational report.
