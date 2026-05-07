# Redesign 99 — Cleanup

> **Status**: skeleton. Master plan: `../REDESIGN_PLAN.md`. Runs **after** `05`–`09` are all merged and archived.

## Why

The foundation changes (`02`, `04`) deliberately ship new components alongside legacy ones rather than replacing them, so each foundation change stays small and additive. Page changes (`05`–`09`) migrate call sites incrementally. After all page migrations land, the legacy components have zero consumers and should be removed; new components with `Form*` / `Dialog` / `Btn` names can shed transitional naming where it improves clarity.

This change is the housekeeping pass.

## What Changes

### Removals (no remaining consumers expected; verify with grep before deleting)

- Delete legacy `Modal` from `src/component/modal.rs` (replaced by `Dialog` in `04`)
- Delete legacy `Button` from `src/component/base_components.rs` (replaced by `Btn` in `02`)
- Delete legacy `TextInput` from `src/component/base_components.rs` (replaced by `FormTextInput` in `04`)
- Remove `missingColor` and `blockedColor` custom Tailwind colors from `tailwind.config.js` (replaced by `bg-warn-soft` / `bg-bad-soft` after `09`)
- Remove related entries from `safelist` in `tailwind.config.js` if present

### Renames (drop transitional `Form*` prefix)

- Rename `FormTextInput` → `TextInput` (slot freed by removal above)
- Rename `FormSelectInput` → `SelectInput` (note: legacy `Select` in `base_components.rs` is different — used as a styled wrapper component; keep or evaluate during this change)
- Rename `FormTextareaInput` → `TextareaInput`

### Verification

- `grep` over `src/` for the removed names; abort and document if call sites remain
- Compile clean, full test suite green
- Manual smoke test: every page renders, every modal opens/closes, every form input accepts focus

## Out of scope

- Removing `Header`, `Label`, `Form`, `FormPair`, `FormItem`, `FormGroup`, `Checkbox`, `DateInput`, `IntegerInput`, `TimeInput`, `FloatInput`, `Select`, `SimpleSelect` from `base_components.rs` — these may stay as project-specific helpers; evaluate per-component during cleanup if any are now also unused
- Visual changes — pure code hygiene
- Backend changes

## Capabilities

### Modified
- `atom-components`: cleanup of legacy parallel Button/Modal
- `dialog`: rename Form* atoms (no behavior change)

## Impact

- Files: `src/component/modal.rs` (delete), `src/component/base_components.rs` (delete `Button`, `TextInput`), `src/component/form/inputs.rs` (rename), `tailwind.config.js`, callers (mass rename via grep)
- Risk: low if the verification step is honest; high if a call site was missed in `05`–`09`. Mitigation: run a grep-and-fail check before any delete.
- Tests: existing tests must keep passing; any test referencing the old names is updated
