# Redesign 08 — Page: Benutzerverwaltung

> **Status**: skeleton. Master plan: `../REDESIGN_PLAN.md`. Reference: `design_handoff_shifty/README.md` (Screens § 6).

## Why

Reference design splits Benutzerverwaltung into two tabs (`SalesPerson` and `Benutzer`) so each entity is managed independently. Current page already keeps Sollstunden out of SalesPerson admin (verified in plan), so this is mostly visual + the tab split.

## What Changes

- Rewrite `src/page/user_management.rs`:
  - Two top-level tabs: `SalesPerson` and `Benutzer`
  - Tab styling: flat underline tabs with accent active state (consistent with shiftplan tab bar in `09`)
- **SalesPerson tab** — table with columns:
  - SalesPerson (color dot + name)
  - Typ (Bezahlt/Freiwillig pill)
  - Verknüpfter Benutzer (mono login or `—`)
  - Edit button → existing `sales_person_details.rs` route
- **Benutzer tab** — table with columns:
  - Benutzer (mono login)
  - Verknüpfter SalesPerson
  - Rollen (accent-soft pill chips: `admin`, `shiftplanner`, `sales`, `hr`)
  - Status (good/muted dot + label)
  - Edit button → existing `user_details.rs` route
- Reuse existing `UserManagementAction` service — no logic change

## Out of scope

- Adding/changing roles or permissions
- Backend changes

## Capabilities

### Modified
- `user-management-page`: tab split, table redesign

## Impact

- Files: `src/page/user_management.rs`
- Tests: render with mixed users, tab switching preserves filter state if any
