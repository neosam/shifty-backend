# Deployment

## Prod deploy via `shifty-nix`

Shifty is deployed as a NixOS module. The sibling repo `shifty-nix`
contains:

- The systemd service definition for the backend.
- Static delivery of the Dioxus WASM bundle.
- The Nix pin on the backend commit version to deploy.

**The deploy itself is manual.** `/release-version` prepares the release
(tag, Nix pin); the actual `nixos-rebuild switch` on the server is done
by the user.

## Release flow (`/release-version`)

The skill:

1. Derives the SemVer version from the GSD milestone
   (`.planning/STATE.md`) and existing git tags.
2. Asks for confirmation.
3. Generates release notes from the commits since the last tag.
4. Invokes `./update_versions.sh` with the notes as the annotated-tag
   message.
5. Updates and tags the deploy pin in `../shifty-nix`.

**What does NOT happen:** a push to the server. That's on you.

## Versioning — SemVer from v2.0 onwards

History:

- **v1.x — v2.x:** SemVer. MAJOR.MINOR from the GSD milestone, PATCH
  from git tags.
- **CalVer `v2026.x`:** Frozen island — do not rename, since
  `shifty-nix` pins these tag names.
- **GSD auto-tag:** disabled (`git.create_tag = false`). Releases go
  exclusively through `/release-version`.

## Feature flags at build time

- **`mock_auth`:** development. Hard-wired admin user.
- **`oidc`:** production. OpenID Connect against an external IdP.

Exactly one of these two auth flags is active in a build.

- **`local_logging`:** text logging, for dev/local.
- **`json_logging`:** structured JSON logging, for production.

## Production startup checklist

When a new deployment rolls out:

1. **Migrations applied?** — `sqlx migrate run` against the prod DB.
2. **`.sqlx/` cache in sync?** — if CI was green, yes.
3. **OIDC config current?** — client ID / secret / redirect URI.
4. **Feature flags correct?** — `oidc`, `json_logging`.
5. **Backup taken beforehand?** — snapshot the SQLite DB file.
6. **After deploy:** exercise the login flow, open a report.

## Rollback

If a deploy is broken:

1. Roll the `../shifty-nix` pin back to the last working commit.
2. `nixos-rebuild switch` on the server.
3. Database rollback: **only if the migration is backwards-compatible.**
   Otherwise restore the DB from backup.

**Important:** SQLx migrations in Shifty have no down script. Going
backwards only works for purely additive changes. For schema changes
that old software can no longer read: restore from backup.

## Monitoring

**[To verify]** — what is set up in production? Log aggregation,
health check endpoint, alerting?

## Related edge cases

See [`../domain/edge-cases.md#10-migrations--sqlx-offline-cache`](../domain/edge-cases.md#10-migrations--sqlx-offline-cache)
for deploy-relevant edges.
