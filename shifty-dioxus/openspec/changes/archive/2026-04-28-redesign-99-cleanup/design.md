## Context

The redesign series ships in two layers:

1. **Foundation changes (`01`–`04`)** introduce new components — `Btn`, `Dialog`, `Field`, `FormTextInput`, `FormSelectInput`, `FormTextareaInput` — *next to* the existing legacy components (`Button`, `Modal`, `TextInput`, `Select`, …). Each foundation change ships purely additive code so it stays small and reviewable, with explicit doc-comments marking the legacy components as deprecated.
2. **Page changes (`05`–`09`)** migrate consumers page-by-page. Each page that gets redesigned drops its `Modal { … }` / `Button { … }` mounts and uses the new atoms instead.

After `05`–`09` were merged (commits `f0cd479` … `0f47d8e`), most consumers of the legacy components are gone. This change is the housekeeping pass that:

- removes the now-unused legacy components,
- frees up the old slot names so `FormTextInput` / `FormSelectInput` / `FormTextareaInput` can drop the transitional `Form*` prefix,
- removes the legacy custom Tailwind colors (`missingColor`, `blockedColor`) that have been replaced by token-based `bg-warn-soft` / `bg-bad-soft`.

### Current state — verification grep

A `grep` of `src/` reveals call sites for the legacy names that survived the page migrations:

| Legacy name | Survivors | Files | Reason |
|---|---|---|---|
| `Modal` | 1 mount | `src/page/my_employee_details.rs` (line 60) | Mirror of `employee_details.rs` flow that was migrated to `ContractModal` / `ExtraHoursModal` in `07`. The `MyEmployeeDetails` page wraps `EmployeeWorkDetailsForm` directly in legacy `Modal`. |
| `Button` | ~11 mounts | `src/page/user_details.rs`, `src/page/sales_person_details.rs`, `src/component/employee_work_details_form.rs`, `src/component/add_extra_days_form.rs` | These pages/forms were not in the redesign scope of `05`–`09` (per the `08` proposal, edit-detail routes were called out as "via existing route"). |
| `TextInput` | 3 mounts | `src/page/sales_person_details.rs` | Same reason — `sales_person_details.rs` was not in the `08` scope. |
| `bg-missingColor` | 1 occurrence | `src/component/week_view.rs` (line 108) | Surrounding `week_view.rs` mostly migrated to `bg-warn-soft` / `bg-bad-soft`; one residual reference remains for the warning indicator on a slot title. |

The proposal anticipated this case ("abort and document if call sites remain"). We choose to migrate these residual call sites as part of this cleanup rather than abort, because:

- they are small (one `Modal`, one `bg-missingColor`, ~14 form-element mounts in 4 files),
- they all map cleanly to existing replacements (`Modal` → `Dialog`, `Button` → `Btn`, `TextInput` → `FormTextInput`, `bg-missingColor` → `bg-warn-soft`),
- leaving them in place would block the renames (the slot `TextInput` cannot be reused while the legacy component still exists with that name).

## Goals / Non-Goals

**Goals:**

- Migrate the residual `Modal` / `Button` / `TextInput` / `bg-missingColor` call sites to the new tokens-based equivalents.
- Delete the legacy `Modal` (`src/component/modal.rs`), `Button`, and `TextInput` (both in `src/component/base_components.rs`) once their call sites are zero.
- Rename `FormTextInput` → `TextInput`, `FormSelectInput` → `SelectInput`, `FormTextareaInput` → `TextareaInput` (and module re-exports). Update all importers.
- Remove `missingColor` and `blockedColor` entries from `tailwind.config.js`, including the matching `safelist` entries.
- Keep the full test suite green and verify by `cargo check` / `cargo clippy` / `cargo test` after each removal step.

**Non-Goals:**

- Visual or behavior changes. The migrated `MyEmployeeDetails` modal, `UserDetails` buttons, etc. should look and behave the same after the migration as the redesigned analogues already do — no re-styling beyond what the new atoms already enforce.
- Removing other components from `base_components.rs` (`Header`, `Label`, `Form`, `FormPair`, `FormItem`, `FormGroup`, `Checkbox`, `DateInput`, `IntegerInput`, `TimeInput`, `FloatInput`, `Select`, `SimpleSelect`, `Option`). These remain — they are still used and serve as project-specific helpers. They may be reviewed in a future change.
- Backend changes.
- Renaming `FormCheckbox` (no slot conflict — the legacy `Checkbox` stays) — kept stable.
- Adding new tests beyond what is needed to confirm the migration; existing tests must continue to pass and any tests referencing the removed names must be updated to point at the new ones.

## Decisions

### 1. Migrate residual call sites in this change instead of aborting

The proposal's verification step says: "grep over `src/` for the removed names; abort and document if call sites remain." The intent is to avoid silently breaking the build. We extend that rule to: *abort* if a call site cannot be cleanly migrated; *migrate* if it can.

Each surviving call site has a clear 1:1 replacement:

| Old | New |
|---|---|
| `Modal { … }` | `Dialog { open: …, on_close: …, title: …, children: … }` |
| `Button { primary: true, on_click: …, "Save" }` | `Btn { variant: BtnVariant::Primary, on_click: …, "Save" }` |
| `Button { on_click: …, "Cancel" }` (no `primary`) | `Btn { variant: BtnVariant::Secondary, on_click: …, "Cancel" }` |
| `TextInput { value, on_change }` | `FormTextInput { value, on_change }` (which becomes `TextInput` after step §3) |

There is no expected behavior delta — the new atoms render visually different (token-based) but semantically identical input. The pages affected (`user_details.rs`, `sales_person_details.rs`, `my_employee_details.rs`, `add_extra_days_form.rs`, `employee_work_details_form.rs`) are administrative detail/edit forms; touching them visually is acceptable and aligned with the redesign vision.

**Alternative considered: defer the legacy removal until those pages get their own redesign change.** Rejected because (a) no such change is planned, (b) leaving the legacy in `base_components.rs` blocks the slot renames in §3, which is the second-largest payoff of this cleanup, and (c) keeping two parallel input families indefinitely is the exact kind of drift we wanted to avoid.

### 2. Rename order — migrate-then-delete-then-rename

Renames must run *after* the legacy components are deleted, otherwise we get name collisions (a Dioxus crate cannot have two `TextInput` types in scope). The internal order is:

1. Migrate residual call sites away from `Modal` / `Button` / `TextInput`.
2. Verify zero `Modal` / `Button` / `TextInput` references in `src/` (grep).
3. Delete `src/component/modal.rs`; delete the `Button` and `TextInput` items from `src/component/base_components.rs`. Update `src/component/mod.rs` re-exports.
4. Rename the module-level types `FormTextInput` → `TextInput`, `FormSelectInput` → `SelectInput`, `FormTextareaInput` → `TextareaInput` inside `src/component/form/inputs.rs`. Update `src/component/form/mod.rs` re-exports and the top-level `src/component/mod.rs` re-export.
5. Mass-rename the `Form*` import sites and JSX-style mount sites (`grep`-replace).
6. Run `cargo check && cargo clippy && cargo test` and address any fall-out.

This order is enforced as the task list ordering — see `tasks.md`.

### 3. Renames are mechanical; tests and docs follow

Test names and doc-comments that mention the old names get updated as part of the same edit:

- The `inputs.rs` module-doc currently says "All three atoms (`FormTextInput`, `FormSelectInput`, `FormTextareaInput`) share the `form-input` class…" → the names are updated in the doc-comment.
- The `form/mod.rs` doc-comment currently says "After full migration, a cleanup change drops the `Form*` prefix" → this becomes a past-tense note or is removed.
- Tests in `inputs.rs` (functions `text_input_renders_input_with_form_input_class`, etc.) keep their names (the function-name still describes the test); only the in-test references to the type name change.

The `form-input` *CSS class* stays — it is a stable global selector, not a Rust type, and unrelated to the rename.

### 4. Custom Tailwind colors and safelist

`missingColor` and `blockedColor` are defined in `tailwind.config.js` (`extend.colors`) and listed in `safelist` (`bg-missingColor`, `bg-blockedColor`). Both have been replaced semantically by `bg-warn-soft` and `bg-bad-soft`. The single residual reference (`week_view.rs:108`, `if warning.is_some() { "bg-missingColor" }`) becomes `"bg-warn-soft"`, matching the surrounding migrated branches in the same file (line 88: `cursor-not-allowed bg-warn-soft print:bg-surface`).

After the call site is updated, both the `colors` entries and the two `safelist` entries are removed in a single edit. Tailwind's `tailwind-cli` does not emit utility classes that don't appear in source, so the removed-but-still-defined custom colors would technically not generate output anyway — but pruning the config keeps it honest and future devs from re-introducing the names.

### 5. The `MyEmployeeDetails → Modal` migration mirrors `EmployeeDetails`

In `redesign-07-page-employees`, the `EmployeeDetails` page replaced its `Modal { ContractForm }` mount with the dedicated `ContractModal` component. The mirror page `MyEmployeeDetails` was not in the `07` scope — but the same pattern fits: wrap `EmployeeWorkDetailsForm` in a `Dialog` with `title: "Vertragsdetails"` (or the equivalent i18n key already used by `ContractModal`'s readonly variant) and `on_close` calling the existing `MyEmployeeDetailsAction::CloseEmployeeWorkDetailsDialog`.

We **do not** introduce a `MyContractModal` wrapper component — there is only one call site, the `Dialog` is used inline. If the readonly contract view ends up needing the same field arrangement as `ContractModal`, the future refactor can extract a shared component; for now keeping the inline form preserves the existing `EmployeeWorkDetailsFormType::ReadOnly` flow.

### 6. `Button` migration — variant mapping

Legacy `Button` has only `primary: bool`. The new `Btn` has four variants (`Primary`, `Secondary`, `Ghost`, `Danger`). The mapping is straightforward but worth pinning so the migration is mechanical:

| Legacy | New |
|---|---|
| `Button { primary: true, … }` | `Btn { variant: BtnVariant::Primary, … }` |
| `Button { … }` (default; `primary: false`) | `Btn { variant: BtnVariant::Secondary, … }` |
| `Button { … "Abort" }` / "Cancel" | `Btn { variant: BtnVariant::Secondary, … "Abort" }` |
| `Button { … "Delete" }` / destructive | `Btn { variant: BtnVariant::Danger, … "Delete" }` |

Per-call-site judgment is required only for "delete"-style buttons. We treat any button whose label is `"Delete"`, `"Löschen"`, `"Reset"`, or whose surrounding context is destructive as `Danger`. Everywhere else the rule above stands.

The `disabled` prop maps 1:1; the `on_click` prop maps 1:1.

### 7. Removing `Button`/`TextInput` from `base_components.rs` without breaking unrelated re-exports

`base_components.rs` is a single ~600-line file with many components. Deleting two items requires editing both the type/component definitions and the public re-export in `src/component/mod.rs` (which currently does `pub use base_components::{Button, TextInput, …}`). The other items in that re-export list are unaffected.

We do not split `base_components.rs` into per-component files in this change — that would be an unrelated refactor and is explicitly out of scope (only the *items being removed* are touched).

## Risks / Trade-offs

**[Migrating untested admin pages]** → risk that `user_details.rs` / `sales_person_details.rs` rendering changes break for admin users in a way the test suite doesn't catch (these pages have light test coverage). Mitigation: keep changes purely mechanical (replace component, keep prop shape); verify with `cargo check` / `cargo clippy`; run the dev server locally and click through the admin flows (open SalesPerson detail → save; open User detail → toggle role; open ExtraHours form). Document the smoke test checklist in `tasks.md`.

**[Visual delta in admin pages]** → the new atoms look different (tokens, accent focus ring, 34 px height vs. legacy `border-2 p-2`). After this change, admin pages look like the rest of the redesigned UI. Trade-off accepted; the alternative (mixed UI styles) is worse.

**[Renames break external editor tooling cache]** → IDEs may need a re-index. Not a real risk for a one-shot rename, but worth noting in the PR description.

**[Tailwind PurgeCSS missed something]** → we believe `bg-missingColor` and `bg-blockedColor` only appear in the safelist + the one Rust call site, but a forgotten reference in a `format!()` macro elsewhere would still match the safelist and silently break after the colors are removed. Mitigation: the verification grep step (`grep -rn "missingColor\|blockedColor" src/`) is a hard gate, not advisory.

**[Test files referencing old names]** → tests that use `Modal { … }` or `Button { … }` for SSR rendering will fail to compile after deletion. Mitigation: the migration step covers test files too. The `cargo test` run at the end is the gate.

## Migration Plan

This change is itself a migration cleanup; there is no separate rollout step. Order of execution:

1. **Migrate** residual call sites (Modal, Button, TextInput, `bg-missingColor`).
2. **Verify** zero references via `grep` (any remaining match → abort and document).
3. **Delete** legacy components and their `mod.rs` re-exports.
4. **Rename** `Form*` atoms (types + module exports + every call site).
5. **Prune** `tailwind.config.js` (`colors` entries + `safelist` entries).
6. **Run** `cargo check`, `cargo clippy`, `cargo test`. Fix any fall-out.
7. **Smoke test** the dev server against the migrated admin pages (manual).

Rollback is trivial — this is a code-hygiene change with no schema or wire-format impact. If the `dx serve` smoke test reveals a regression, revert the commit; the redesigned UI continues to function on the previous SHA.

## Open Questions

- **Should `FormCheckbox` be renamed to `Checkbox` for consistency?** No — `Checkbox` already exists in `base_components.rs` and is *not* being removed (it is in the explicit "stays as project-specific helper" list per the proposal). Keeping `FormCheckbox` distinct avoids a slot collision and is consistent with what the proposal documents. Closed.
- **Are there hidden call sites in test fixtures or example HTML files outside `src/`?** Verified: `tests/`, `assets/`, `dist/` contain no Rust call sites for the removed types; only `src/` is migrated. The `bg-missingColor` / `bg-blockedColor` strings are absent in CSS files and assets after the safelist entries are removed.
