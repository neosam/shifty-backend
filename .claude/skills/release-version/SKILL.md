---
name: release-version
description: >
  Release a new version of shifty. Generates release notes from changes since the last tag,
  runs cli-update-version.sh with the release notes as the annotated-tag message, and reports
  the new version number. Use when the user says "release", "neue Version", "Version releasen",
  "Release bauen", or "/release-version".
---

# Release Version Skill

Release a new shifty version with release notes as the annotated-tag message.

The version itself is auto-derived by `cli-update-version.sh` (date-based `YEAR.DAY.COUNTER`
read from the current `-dev` version in Cargo.toml) — you do NOT pick the version number.
Your job is to produce good release notes and invoke the script non-interactively.

## Steps

### 1. Find the Last Tag and Get Changes

Find the latest release tag (version-sorted, so `v2026.x` wins over the older `v1.x` tags),
then list the commit subjects since then via jj:

```bash
LAST_TAG=$(git tag -l 'v*' | sort -V | tail -1)
echo "Last tag: $LAST_TAG"
jj log -r "tags(exact:\"$LAST_TAG\")..@" --no-graph -T 'description.first_line() ++ "\n"'
```

### 2. Generate Release Notes

From the commit subjects, write structured release notes. Categorize changes into sections
like Features, Bug Fixes, Improvements, etc. Only include sections that have entries.
Skip pure planning/docs/chore churn (e.g. `docs(NN):`, `chore: archive ...`, STATE/ROADMAP
bookkeeping) unless it represents user-visible change. Use bullet points. Example format:

```
Features:
- Inline HR vacation-offset editor

Bug Fixes:
- Cap vacation days/hours per week at workdays_per_week
- Read carryover from previous year, not current

Improvements:
- Register /vacation-entitlement-offset in dev proxy
```

### 3. Run the Release Script

Run the release script from the backend root (`shifty-backend/`) with the release notes
as the tag message:

```bash
./cli-update-version.sh -m "<release notes>"
```

IMPORTANT:
- The release notes MUST always be provided via the `-m` flag. Without it, `git tag -a`
  opens an interactive editor and blocks the run.
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

### 4. Report the Result

The script prints `New release version: X.Y.Z` at the end. Report this version number
to the user.
