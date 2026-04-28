# Redesign 05 — Page: Meine Schichten

> **Status**: skeleton. Master plan: `../REDESIGN_PLAN.md`. Reference: `design_handoff_shifty/README.md` (Screens § 3).

## Why

Easiest page to redesign — pure read-only week cards. Good first page to validate that tokens, atoms, top bar, and modal work together before touching the more complex pages.

## What Changes

- Rewrite `src/page/my_shifts.rs` to use new tokens and atoms:
  - Per-week card layout, max-width 760 px, centered
  - Card header: `KW <n> · <date range>` left, total hours mono right
  - One row per day: day label (mono, 110 px) · shift items · hours (mono, 60 px right-aligned)
  - Each shift item: `09:00–13:00` (mono) + colored area badge (`Laden` etc., accent-soft pill)
  - Days with `note` starting `⚠` get `bg-warn-soft` background and warning text
- Mobile: compact day rows `80px 1fr 50px`
- Reuse data loading from existing service — no API changes

## Out of scope

- Adding shift items inline edit/remove on this page — read-only by design
- Backend changes

## Capabilities

### Modified
- `my-shifts-page` (or whichever capability name applies to this page; create if absent): visual rewrite, layout change

## Impact

- Files: `src/page/my_shifts.rs`
- Tests: render test for sample week including warning note path
