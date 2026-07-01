---
name: release-version
description: >
  Release a new version of shifty. Derives the SemVer version from the GSD milestone
  (.planning/STATE.md) plus existing git tags, confirms it, generates release notes from
  changes since the last tag, runs cli-update-version.sh with the notes as the
  annotated-tag message, and updates + tags the deployment pin in ../shifty-nix (deploy stays
  manual). Use when the user says "release", "neue Version", "Version releasen",
  "Release bauen", or "/release-version".
---

# Release Version Skill

Release a new shifty version with SemVer versioning and release notes as the annotated-tag message.

## Versioning model

Shifty uses **SemVer** `MAJOR.MINOR.PATCH`:

- **MAJOR.MINOR** comes from the **GSD milestone** — milestones are named `vX.Y`
  (e.g. `v2.0`, `v2.1`) and are the source of truth for the major/minor version.
- **PATCH** is mechanical — the count of releases already cut on that `X.Y` line.
  First release of a milestone → `.0`; each hotfix on the same line → `.1`, `.2`, …

You do NOT hand-invent the number: derive a candidate (steps 1–2), confirm it with the
user (step 3), then pass it explicitly to `cli-update-version.sh` (step 5).

> Historical note: older tags used CalVer (`v2026.DAY.N`, May–Jul 2026) and even older
> ones plain SemVer up to `v1.12.5`. The CalVer tags are a frozen island — ignore them
> for version derivation; SemVer resumes at `v2.0.0`.

## Steps

### 1. Derive MAJOR.MINOR from the GSD milestone

Read the milestone version from `.planning/STATE.md` frontmatter:

```bash
MILESTONE=$(grep -m1 '^milestone:' .planning/STATE.md \
    | sed -E 's/^milestone:[[:space:]]*v?([0-9]+\.[0-9]+).*/\1/')
echo "GSD milestone (MAJOR.MINOR): $MILESTONE"
```

Sanity-check the result is a plain `X.Y` (two numbers). If it is empty, non-numeric,
or the status in STATE.md is "Awaiting next milestone" (meaning the milestone shown is the
*last completed* one, not what you are about to release), do NOT trust it blindly — surface
it to the user in step 3 and let them supply the intended `X.Y`.

### 2. Derive PATCH from existing tags

Count the highest patch already released on this `X.Y` line and add one:

```bash
LAST_PATCH=$(git tag -l "v${MILESTONE}.*" \
    | sed -E "s/^v${MILESTONE}\\.([0-9]+)$/\\1/" \
    | grep -E '^[0-9]+$' | sort -n | tail -1)
if [ -z "$LAST_PATCH" ]; then NEXT_PATCH=0; else NEXT_PATCH=$((LAST_PATCH + 1)); fi
RELEASE="${MILESTONE}.${NEXT_PATCH}"
echo "Derived release candidate: v$RELEASE"
```

The git tags are the source of truth for PATCH — they are the actual shipped record and
cannot drift. If the candidate tag already exists (`git rev-parse "v$RELEASE" 2>/dev/null`
succeeds), STOP and report it — something is out of sync; do not overwrite a released tag.

### 3. Confirm the version with the user

Show the derived candidate and ask the user to confirm or override before doing anything
irreversible. `/release-version` is always run deliberately by a human, so this one-second
check is the safety net that catches a stale/corrupted `milestone:` field, the "awaiting
next milestone" window, or a first jump onto a new `X.Y` line:

> Release **v2.0.0** ableiten (Milestone `v2.0` + Patch 0)? Enter zum Bestätigen, oder andere Nummer angeben.

Use whatever the user confirms as `$RELEASE`. They may override with a full explicit
`X.Y.Z` (e.g. a hotfix on an older line while a newer milestone is active).

### 4. Generate release notes from changes since the last tag

Find the most recently **created** tag (robust against the mixed CalVer/SemVer tag names —
do NOT use `sort -V`, which would pick `v2026.x` over `v2.0.0`), then list commit subjects
since then via jj:

```bash
LAST_TAG=$(git for-each-ref --sort=-creatordate --format='%(refname:short)' 'refs/tags/v*' | head -1)
echo "Last tag: $LAST_TAG"
jj log -r "tags(exact:\"$LAST_TAG\")..@" --no-graph -T 'description.first_line() ++ "\n"'
```

From the commit subjects, write structured release notes. Categorize into sections like
Features, Bug Fixes, Improvements, etc. Only include sections that have entries. Skip pure
planning/docs/chore churn (e.g. `docs(NN):`, `chore: archive ...`, STATE/ROADMAP
bookkeeping) unless it represents user-visible change. Use bullet points. Example:

```
Features:
- Inline HR vacation-offset editor

Bug Fixes:
- Cap vacation days/hours per week at workdays_per_week
- Read carryover from previous year, not current

Improvements:
- Register /vacation-entitlement-offset in dev proxy
```

### 5. Run the Release Script

Run the release script from the backend root (`shifty-backend/`) with the **confirmed
version** and the release notes as the tag message. NEXT (the next `-dev` base) is optional —
omit it and the script defaults to the release patch + 1 (`2.0.0` → `2.0.1-dev`); pass it
explicitly only to pre-bump the minor:

```bash
./cli-update-version.sh -m "<release notes>" "$RELEASE"
# or, to set the next dev base explicitly:
./cli-update-version.sh -m "<release notes>" "$RELEASE" 2.1.0
```

IMPORTANT:
- The release notes MUST always be provided via the `-m` flag. Without it, `git tag -a`
  opens an interactive editor and blocks the run.
- Pass `$RELEASE` explicitly — the script no longer derives any version itself. It only
  bumps, builds, commits, tags, and pushes what you give it.
- The script runs `update_versions.sh` (bumps every backend crate + the frontend
  `shifty-dioxus/` crate + all nix files), then a build gate over BOTH workspaces —
  `cargo build` for the backend plus `cargo build --target wasm32-unknown-unknown`
  for the frontend (shifty-dioxus is `exclude`d from the root workspace, so the root
  build does not cover it). It then commits via **jj**, moves the `main` branch,
  pushes, tags, and bumps to the next `-dev` version. It uses jj for commits (this is
  a jj-managed repo) — do not commit anything yourself.
- Run it from the **backend nix dev shell** (`nix develop`), which provides both the
  build toolchain and the `wasm32-unknown-unknown` target. If a build step fails
  because a tool/target is missing, you are in the wrong shell — do not improvise
  installs (see CLAUDE.local.md).
- Wait for the script to complete.

### 6. Update & tag the deployment pin (shifty-nix)

After the backend release is pushed and tagged, bump the deployment pin in the sibling
`../shifty-nix` repo so it points at the freshly released tag:

```bash
( cd ../shifty-nix && ./cli-release.sh "$RELEASE" )
```

`cli-release.sh` (mirrors `cli-update-version.sh`) takes the plain `X.Y.Z` and:
- runs `gen-backend.sh vX.Y.Z` → renders `shifty-backend.nix` + `shifty-frontend.nix` from
  the templates, builds both locally via nix to verify the pin resolves, writes
  `backend-version.txt`;
- commits via **jj** (message `X.Y.Z`), moves `main`, pushes **only** `main` (shifty-nix has
  stray bookmarks that must not be pushed), tags `vX.Y.Z`, pushes tags.

Notes:
- Run this AFTER step 5 — `gen-backend.sh` fetches
  `https://github.com/neosam/shifty-backend/archive/vX.Y.Z.zip`, so the backend tag must
  already be on GitHub. If the verify build 404s, the tag push may not have propagated yet;
  retry after a moment.
- The verify build (`nix-build`) is heavy and needs network — expect it to take a while.
- **Deployment stays manual** — do NOT run `deploy-binaries.sh` / `build-and-deploy.sh`.
  The user deploys themselves when they choose.

### 7. Report the Result

The backend script prints `New release version: X.Y.Z`. Report this to the user, and
confirm the shifty-nix pin was regenerated + tagged (`vX.Y.Z`) — and that deployment is
left to them.
