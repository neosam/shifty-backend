# Operations — Running, Deployment, Configuration

This section targets everyone who **operates** Shifty, deploys it,
migrates it, or configures it.

## Chapters

- **[deployment.md](./deployment.md)** — NixOS deployment via
  `shifty-nix`, release process (`/release-version`), version pinning.
- **[database.md](./database.md)** — SQLx migrations, `sqlx migrate run` vs
  `sqlx database reset`, `.sqlx/` cache for offline builds, backup strategy.
- **[configuration.md](./configuration.md)** — Environment variables,
  feature flags (`mock_auth`, `oidc`, logging modes), OIDC setup.

## Short Overview

### Two Deploy Forms

1. **Development.** Backend runs locally on port 3000, frontend via
   `dx serve` on port 8080. The frontend proxies REST requests according
   to `shifty-dioxus/Dioxus.toml`.
2. **Production.** The NixOS module from the neighboring `shifty-nix` repo
   deploys the backend as a systemd service. The frontend is shipped as a
   static WASM bundle.

### Release Flow

The `/release-version` skill:

1. Derives the new version from the GSD milestone (`.planning/STATE.md`) and
   the existing git tags following SemVer.
2. Asks for confirmation, generates release notes from commits since the last
   tag.
3. Calls `./update_versions.sh` with the notes as the annotated tag message.
4. Updates the deploy pin in `../shifty-nix` and tags it.

The deploy itself is **manual** — pushing to the server happens outside this
automation.

### Critical Ops Edge Cases

See [`../domain/edge-cases.md`](../domain/edge-cases.md), especially the
"Migrations & Schema" and "Snapshot Versioning" sections.

Short version of the most painful ones:

- **`sqlx database reset` is destructive.** Use only additive `sqlx migrate run`,
  except when you explicitly want to wipe the dev DB.
- **After a new `query!/query_as!` query**, `cargo sqlx prepare --workspace`
  must run and the `.sqlx/` cache must be committed — otherwise CI fails.
- **`nix build`** enforces `cargo clippy -- --deny warnings`. `cargo test`
  alone is not sufficient.
