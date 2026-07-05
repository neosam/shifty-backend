---
phase: 51
plan: 07
subsystem: rest-types + frontend + pdf-renderer
tags: [dto, frontend, pdf, view-layer, shortday, clip]
requires:
  - "51-03: ShiftplanSlot.effective_to (service layer)"
provides:
  - "ShiftplanSlotTO.effective_to (wire DTO)"
  - "Frontend WeekView renders clipped end time (via state::Slot.to = effective_to)"
  - "PDF renderer produces clipped time labels and clipped slot heights"
affects:
  - rest-types/src/lib.rs
  - shifty-dioxus/src/loader.rs
  - service_impl/src/pdf_render.rs
tech-stack:
  added: []
  patterns:
    - "Wrapper-Field-Pattern für view-layer-Derived-Werte (DTO carries both raw + view-effective; bidirectional SlotTO stays untouched)"
    - "Fat Backend, thin client (D-51-02): FE consumes finished values, zero clip logic"
key-files:
  created: []
  modified:
    - rest-types/src/lib.rs
    - shifty-dioxus/src/loader.rs
    - service_impl/src/pdf_render.rs
decisions:
  - "D-51-09 realized in DTO: effective_to lives on ShiftplanSlotTO (wrapper), NOT on SlotTO (bidirectional invariant preserved)"
  - "compute_slot_duration_hours + format_slot_time_label take &ShiftplanSlot (wrapper) instead of &Slot — cleaner than cloning a synthetic slot at each callsite"
metrics:
  duration_seconds: 696
  completed_date: 2026-07-05
status: complete
---

# Phase 51 Plan 07: DTO + FE-Loader + PDF-Verify Summary

One-liner: `ShiftplanSlotTO` gains `effective_to` on the wrapper, FE loader consumes it, PDF renderer signature-changes to also consume it — Chain B's clipping (P03) is now visible in the WeekView and in printed shift plans without any FE-side clip logic and without touching `SlotTO`.

## What Was Built

### 1. `rest-types/src/lib.rs` — DTO field + mapper

`ShiftplanSlotTO` gained `pub effective_to: time::Time` immediately after `current_paid_count`. The `From<&service::shiftplan::ShiftplanSlot>` mapper propagates `slot.effective_to` from the service-layer wrapper. `SlotTO` is untouched — its bidirectional POST/PUT `/slot` roundtrip stays byte-clean.

Two new test cases in the crate (`test_shiftplan_slot_to_effective_to`):
- `mapper_populates_effective_to_from_wrapper_and_leaves_slot_to_raw` — TO.effective_to = 14:30 while TO.slot.to stays 15:00.
- `slot_to_roundtrip_never_touches_effective_to` — proves the D-51-09 invariant: `SlotTO → Slot → SlotTO` never touches `effective_to` (there is no such field on `SlotTO`; a compile-time reference at the end of the test enforces this by construction).

### 2. `service_impl/src/pdf_render.rs` — renderer consumes `effective_to`

**Signature change** (Task 2):
- `fn compute_slot_duration_hours(shiftplan_slot: &ShiftplanSlot) -> f32` — was `&service::slot::Slot`. Reads `shiftplan_slot.effective_to.hour()/.minute()` for the end minute, `shiftplan_slot.slot.from` for the start.
- `fn format_slot_time_label(shiftplan_slot: &ShiftplanSlot) -> String` — same shape. Renders `"14:00 - 14:30"` for a clipped slot instead of `"14:00 - 15:00"`.

**Caller ripple**: three sites migrated from `&s.slot` / `&slot.slot` to the wrapper:
- Line 233 (main render pipeline `day_slot_renders`),
- Line 628 (bold time label inside `render_slot_box`),
- Line 1296 (test `row_alignment_across_days_pushes_all_columns_down_together`).

### 3. `service_impl/src/pdf_render.rs` — SHC-04 verification tests

Added at the end of the existing `#[cfg(test)] mod test` block:
- `pdf_slot_duration_uses_effective_to_when_clipped` — Slot 14:00-15:00 with `effective_to=14:30` → `compute_slot_duration_hours == 0.5h`.
- `pdf_slot_duration_matches_raw_when_effective_to_equals_to` — default path (`effective_to == slot.to`) → 1h. Proves non-ShortDays are unaffected.
- `pdf_bytes_embed_clipped_time_label_not_raw` — end-to-end: renders the whole `ShiftplanWeek`, then byte-greps the PDF for the hex-encoded `"14:00 - 14:30"` (must be present) AND for `"14:00 - 15:00"` (must be absent). This is the SHC-04 evidence — the PDF renderer picks up the clipped value all the way through `render_shiftplan_week_pdf`.

### 4. `shifty-dioxus/src/loader.rs` — FE loader reads the wrapper

Two sites (`load_shift_plan` line ~101, `load_day_aggregate` line ~154) switched from `to: slot.slot.to` to `to: slot.effective_to`. No `state::Slot`-schema change, no WeekView-component change, no i18n change. D-51-02 (Fat Backend) satisfied: FE receives the finished clipped value.

## Deviations from Plan

### PDF renderer required a code change (Task 2)

The prompt summary indicated "no `pdf_render.rs` code change" for SHC-04. The PLAN.md itself was explicit that a signature change WAS necessary (Task 2, lines 124–143) — the pdf renderer's `compute_slot_duration_hours` and `format_slot_time_label` used to read `&service::slot::Slot` directly, so they had no way to see `effective_to` which lives on the wrapper. Without the signature change the PDF would render the raw duration/label even though the wrapper is clipped.

Followed the PLAN.md (authoritative). Documenting here for the orchestrator: the "verify SHC-04 without touching pdf_render.rs" phrasing in the prompt was inaccurate; the plan correctly identified this as unavoidable and prescribed the wrapper-signature approach.

No other deviations. No Rule 1/2/3 auto-fixes needed. No architectural questions.

## Gates

Backend:
- `cargo build --workspace` — green.
- `cargo test --workspace` — 795 backend tests green (incl. the 5 new ones: 2 rest-types + 3 pdf_render).
- `cargo clippy --workspace -- -D warnings` — green.

Frontend (`shifty-dioxus/`):
- `cargo build --target wasm32-unknown-unknown` — green.
- `cargo test` — 795 FE tests green.
- `cargo clippy -- -D warnings` — green (no pre-existing lints tripped, contrary to the memory note — the touched loader.rs sites were clippy-clean).

## Invariants Verified

- `git diff HEAD~5 -- rest-types/src/lib.rs | grep -A2 'pub struct SlotTO'` returns empty → SlotTO untouched (D-51-09).
- `git diff HEAD~5 -- shifty-dioxus/src/component/ shifty-dioxus/src/state/` returns empty → no visual marker added, no state-schema change (D-51-04, D-51-02).
- Snapshot version unchanged (no persistence writer touched).
- No new Cargo dependency added.
- No Dioxus.toml proxy addition needed — `/shiftplan` was already allowlisted (line 86).

## Commits

- `b0d0156` test(51-07): add failing test for ShiftplanSlotTO.effective_to mapper
- `61e42be` feat(51-07): add ShiftplanSlotTO.effective_to wrapper field (D-51-09)
- `67502ec` test(51-07): add failing PDF-renderer tests for SHC-04 (effective_to clipping)
- `9d60643` feat(51-07): PDF renderer consumes ShiftplanSlot.effective_to (SHC-04)
- `8b66f18` feat(51-07): FE loader consumes ShiftplanSlotTO.effective_to (D-51-02)

## Requirements Coverage

- SHC-03 (WeekView renders 14:00-15:00 with cutoff 14:30 as 14:00-14:30 cell) — loader.rs feeds `state::Slot.to = effective_to`; existing WeekView renders `slot.to_hour()` so the shortened cell falls out automatically.
- SHC-04 (PDF consumes same ShiftplanWeek aggregate as WeekView) — `pdf_bytes_embed_clipped_time_label_not_raw` proves the whole render pipeline consumes `effective_to`.

## Threat Flags

None. No new network endpoints, no auth surface changes, no schema at a trust boundary. This plan only threads a value from the service layer through the DTO to the FE + PDF; the value is derived server-side.

## Self-Check: PASSED

- `[ -f rest-types/src/lib.rs ]` — FOUND
- `[ -f shifty-dioxus/src/loader.rs ]` — FOUND
- `[ -f service_impl/src/pdf_render.rs ]` — FOUND
- `git log --oneline --all | grep -q b0d0156` — FOUND
- `git log --oneline --all | grep -q 61e42be` — FOUND
- `git log --oneline --all | grep -q 67502ec` — FOUND
- `git log --oneline --all | grep -q 9d60643` — FOUND
- `git log --oneline --all | grep -q 8b66f18` — FOUND
