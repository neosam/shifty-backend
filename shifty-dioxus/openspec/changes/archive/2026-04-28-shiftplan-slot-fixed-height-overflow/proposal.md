## Why

Week-grid slot cells in the shiftplan currently let their inner content (PersonChips + min-resources pill + add/remove button) overflow visually when a slot has more bookings than its time-derived height can fit. Because each `WeekCellSlot` is absolutely positioned with a fixed `height = (to_hour - from_hour) * SCALING`, a 1-hour slot with six bookings paints chips on top of the next hour's slot. The shifty-design reference solves this with a fixed-height container, hidden outer overflow, and an internal scroll layer for the chips — the n/m pill and +/- button stay visible as absolutely positioned siblings, never participating in the scroll.

## What Changes

- Replace the slot cell's "outer flex-wrap container that grows" layout with a three-zone absolute layout inside the existing fixed-height slot box:
  - the n/m min-resources indicator pinned to `top: 6, left: 8` (absolute, `pointer-events: none`)
  - a chip area positioned via `inset: 6 32 6 38`, owning `flex-wrap: wrap` and its own `overflow-y: auto / overflow-x: hidden`
  - the existing +/- (or `…` dropdown) button at `top: 6, right: 6`
- Add `overflow: hidden` to the outer slot box so chips never paint outside their slot, even when too many to fit.
- Add `whitespace-nowrap` to the `PersonChip` shape classes so a long single name never wraps mid-word inside the inline-flex chip (chips remain atomic; the chip area wraps them).
- Tests: extend `week_view.rs` SSR-style assertions to cover the three-zone class fingerprint and the chip area's overflow tokens; add a `PersonChip` unit test asserting `whitespace-nowrap` is present in both color paths.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `shiftplan-page`: the slot cell's internal layout becomes fixed-height + three absolute zones with internal chip scroll; the PersonChip rendering requirement gains a no-wrap clause; the min-resources indicator requirement gains an absolute-positioning clause.
- `atom-components`: the `PersonChip` atom gains a `whitespace-nowrap` class invariant.

## Impact

- `src/component/week_view.rs` — `WeekCellSlot` (≈ lines 1029–1167): outer style + inner DOM structure.
- `src/component/atoms/person_chip.rs` — `SHAPE_CLASSES` adds `whitespace-nowrap`; new SSR test.
- No data-model, API, or i18n changes. Visible behavior change only inside the slot when bookings > visible capacity (now scrollable instead of overflowing).
- The change is purely CSS/markup; no migration, no settings.
