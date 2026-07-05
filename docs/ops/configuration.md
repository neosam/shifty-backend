# Configuration

Shifty is configured at runtime via environment variables and at build
time via feature flags.

## Environment variables

Base template: `env.example`. For dev: `cp env.example .env`.

**[To verify]** — complete env-var list from `service_impl/src/config.rs`
or the startup code. Typical entries:

- `DATABASE_URL` — SQLite path.
- `PORT` — backend port (default 3000).
- OIDC config under the `oidc` feature — `OIDC_ISSUER`, `OIDC_CLIENT_ID`,
  `OIDC_CLIENT_SECRET`, `OIDC_REDIRECT_URI`.
- Logging level (`RUST_LOG`).

## Feature flags

Feature flags are activated via `cargo build --features "..."`.

### Auth modes (mutually exclusive)

- **`mock_auth`** — development. Mocked admin user on every request.
- **`oidc`** — production. External OpenID Connect provider.

### Logging modes

- **`local_logging`** — human-readable text.
- **`json_logging`** — structured JSON for log aggregation.

## `config.rs`

The central config struct lives in `service_impl/src/config.rs`. It
reads env vars on startup and exposes them to services.

## OIDC setup

For a production deployment:

1. Register with the IdP (create a client, set the redirect URI).
2. Enter the `OIDC_*` env vars in `.env` or the systemd environment.
3. Build with `--features oidc`.
4. On first login the user is created inside Shifty (**[To verify]**
   whether an explicit invitation is required — see
   [F10](../features/F10-templates-communication.md)).

## Nextcloud / WebDAV

If PDF export to Nextcloud is active:

- WebDAV URL, username, token in the configuration.
- Details: [F11 Export](../features/F11-export.md).

## Scheduler configuration

The scheduler currently runs with the hard-wired cron expression
`"0 * * * * *"` (see `service_impl/src/scheduler.rs:45`). It is only
configurable via code change today.

## Related edge cases

- Toggle and feature-flag behaviour →
  [`../domain/edge-cases.md#9-feature-toggles--stichtag-rollouts`](../domain/edge-cases.md#9-feature-toggles--stichtag-rollouts).
