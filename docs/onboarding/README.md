# Onboarding — New Developers

This section takes you from a blank laptop to your first merge.

## Order

1. **[first-week.md](./first-week.md)** — What you need in the first few days:
   clone the repo, toolchain, editor setup, start backend + frontend, first
   fake bug fix.
2. **Understand the architecture** — Before you write code, read:
   - [`../architecture/01-layered.md`](../architecture/01-layered.md) — Why REST → Service → DAO → SQLite.
   - [`../architecture/02-service-tiers.md`](../architecture/02-service-tiers.md) — Basic vs Business-Logic; which service goes where?
   - [`../architecture/05-transactions.md`](../architecture/05-transactions.md) — The `Option<Transaction>` pattern.
3. **Learn the domain** — Without domain knowledge, code is roulette:
   - [`../domain/glossary.md`](../domain/glossary.md) — Sales Person, Slot, Booking, Absence, Balance, Billing Period.
   - [`../domain/time-accounting.md`](../domain/time-accounting.md) — How the time account is calculated.
   - [`../domain/edge-cases.md`](../domain/edge-cases.md) — The sharp edges. Please read before touching reporting.
4. **Respect conventions**:
   - [`../architecture/07-testing.md`](../architecture/07-testing.md) — Mockall, in-mem SQLite, `cargo sqlx prepare`, clippy gate.
   - [`../architecture/08-i18n.md`](../architecture/08-i18n.md) — New strings need En/De/Cs.

## Important Baseline Attitude

- **Do not duplicate domain knowledge in the frontend.** If you are computing
  hours in the UI, the flow is wrong. Compute in the backend, send the result.
- **No hard-delete.** All deletions are soft-delete. Readers filter
  `deleted IS NULL`.
- **No direct `git commit`.** This repo runs on **jj** (co-located with
  git). The GSD executor commits automatically via git; manual commits
  go exclusively through `jj`.
- **No backend endpoint without a `Dioxus.toml` proxy entry** if the frontend
  is supposed to call it — otherwise you get 404 in `dx serve` dev mode.

## Help

- `.planning/` — GSD planning artifacts for current and past phases. This
  is context for why a feature looks the way it does.
- `CLAUDE.md` (repo root) — short form of the most important conventions.
- This documentation is the reference long-form of that.
