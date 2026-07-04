---
phase: 51
plan: 05
subsystem: booking_information
status: complete
tags: [shortday, clip, chain-c, booking-information, view-layer]
requires:
  - 51-01
  - 51-02
provides:
  - chain-c-clip
  - shortday_gate::clip_slot_for_week (pub(crate) helper)
affects:
  - service_impl/src/booking_information.rs
  - service_impl/src/shortday_gate.rs (helper promoted)
  - service_impl/src/block.rs (consumes promoted helper)
  - service_impl/src/test/booking_information_vfa.rs
  - service_impl/src/test/booking_information_chain_c.rs (new)
  - service_impl/src/test/booking.rs
  - shifty_bin/src/main.rs
tech-stack:
  added: []
  patterns:
    - "Promoted `clip_slot_for_week` + `ClipOutcome` from block.rs to shortday_gate.rs as pub(crate) — shared by Chain A' and Chain C. Kein Duplikat, keine forced abstraction."
    - "ToggleService als Basic-Tier-Dep zu BookingInformationServiceImpl — Toggle-Lookup einmal pro Method-Call, nicht pro Woche (Perf-Note im year-view loop)."
key-files:
  created:
    - service_impl/src/test/booking_information_chain_c.rs
  modified:
    - service_impl/src/booking_information.rs
    - service_impl/src/shortday_gate.rs
    - service_impl/src/block.rs
    - service_impl/src/test/booking_information_vfa.rs
    - service_impl/src/test/booking.rs
    - service_impl/src/test/mod.rs
    - shifty_bin/src/main.rs
decisions:
  - "D-51-06 Chain C: `get_weekly_summary` (bis 388-409) + `get_summery_for_week` (bis 506-525) klippen Slots via `Slot::clip_to`, statt sie zu verwerfen (pre-existing Bug fix, D-04)."
  - "Reuse-Wahl (Plan reuse_note Option 2): `clip_slot_for_week` + `ClipOutcome` von `block.rs` in `shortday_gate.rs` hochgezogen; block.rs re-importiert. Rechtfertigung: identisches Shape zu P04, zwei Konsumenten, kein weiterer Splitpunkt in Sicht."
  - "D-51-03 verify: Test F liegt in `test/booking.rs::phase51_create_post_cutoff_slot_not_rejected`, weil `BookingService::create` dort getestet wird (nicht in booking_information); Doku-Assertion, dass Chain C's Clip AUSSCHLIESSLICH im View-Layer wirkt."
  - "Perf-Note: `get_weekly_summary` iteriert über weeks_in_year+3 Wochen — Toggle-Wert wird EINMAL vor dem Loop geholt und via `active_from: Option<Date>` in die Loop weitergereicht. `get_summery_for_week` ist single-week → Toggle wird beim Aufruf lokal geholt."
metrics:
  duration: "~35 min"
  completed: "2026-07-05"
---

# Phase 51 Plan 05: Chain C — booking_information.rs WeeklySummary + Booking-Conflicts Summary

Chain C aus D-51-06 landet: die beiden ShortDay-Filter-Anti-Patterns in
`booking_information.rs` sind durch pro-Slot-Clip via
`shortday_gate::clip_slot_for_week` ersetzt. Der Fix schließt einen
pre-existing Bug (D-04-Verletzung: Slot komplett verwerfen statt clippen)
und implementiert gleichzeitig SHC-02 (Weekly-Summary sieht geclippte
Dauer), SHC-05 (existierende Bookings bleiben unangetastet) und SHC-06
(Stichtag-Gate). D-51-03 (Booking-Create nicht abgelehnt) ist durch
Test F belegt und dokumentiert, dass die Clip-Semantik View-Layer-only
bleibt.

## Sites-Diff

### Site 1: `get_weekly_summary` (booking_information.rs 388-409)

Vorher (Bug):
```rust
.filter(|slot| {
    !special_days.iter().any(|day| {
        day.day_of_week == slot.day_of_week
            && (day.day_type == SpecialDayType::Holiday
                || day.day_type == SpecialDayType::ShortDay
                    && day.time_of_day.is_some()
                    && slot.to > day.time_of_day.unwrap())
    })
})
.cloned()
.collect();
```

Nachher (Fix):
```rust
.filter(|slot| {
    // Holiday-Filter bleibt hart: kompletter Slot raus.
    !special_days.iter().any(|day| {
        day.day_of_week == slot.day_of_week
            && day.day_type == SpecialDayType::Holiday
    })
})
.filter_map(|slot| {
    // Chain C: ShortDay-Clip pro Wochentag + Stichtag-Gate.
    match shortday_gate::clip_slot_for_week(slot, &special_days, year, week, active_from) {
        ClipOutcome::Keep(s) => Some(s),
        ClipOutcome::Drop => None,
    }
})
.collect();
```

`slot_hours = (slot.to - slot.from) * min_resources` bleibt unverändert
— sieht jetzt automatisch die geclippten Zeiten (D-04 Zeile 4).

### Site 2: `get_summery_for_week` (booking_information.rs 506-525)

Analoges Ersetzen. `required_hours_by_day` (680-697) fold über
`slots.iter()` bleibt ebenfalls unverändert und foldet automatisch die
geclippten Slots pro Wochentag. **Kein zweiter Fix nötig** — die
Plan-INDEX-Warning ist bestätigt (siehe D-51-06 Regel und key_links).

### Helper-Promotion (Reuse-Choice)

`clip_slot_for_week` + `ClipOutcome` aus `block.rs` (P04) sind nach
`shortday_gate.rs` als `pub(crate)` gewandert. `block.rs` importiert
`use crate::shortday_gate::{clip_slot_for_week, ClipOutcome};` — kein
Duplikat, keine forced abstraction.

## DI-Änderung

`BookingInformationServiceImpl` konsumiert jetzt `ToggleService` (Basic-
Tier). Keine Zyklen — ToggleService kennt keine Domain-Services.

- `service_impl/src/booking_information.rs` — `gen_service_impl!`-Block
  erweitert.
- `shifty_bin/src/main.rs` — `type ToggleService = ToggleService;` im
  `BookingInformationServiceDependencies`-Impl; `toggle_service.clone()`
  im Konstruktor. Handle war bereits aus P03/P04 im Scope.
- `service_impl/src/test/booking_information_vfa.rs` — `MockToggleService`
  mit Default-`.returning(|_, _, _| Ok(None))` als Stub ergänzt; VFA-02-
  Test bleibt grün.

## Neue Tests

Sechs neue Tests, alle grün:

**`service_impl/src/test/booking_information_chain_c.rs`** (neu, 5 Tests):

- `test_get_weekly_summary_clips_slot_hours_at_shortday` (A) — 0,5h statt 1h.
- `test_get_weekly_summary_ungated_no_clip` (B) — `active_from = None` → 1,0h.
- `test_get_weekly_summary_drops_post_cutoff_slot` (C) — Post-Cutoff-Slot fehlt.
- `test_get_summery_for_week_required_hours_by_day_respects_clip` (D) —
  Mo 0,5h + Di 2,0h, `required_hours_by_day` foldet korrekt.
- `test_get_summery_for_week_stichtag_boundary` (E) — Grenzfall
  `booking_date < active_from` (ungeclippt) vs. `== active_from`
  (inklusiv geclippt).

**`service_impl/src/test/booking.rs`** (1 Test):

- `phase51_create_post_cutoff_slot_not_rejected` (F, D-51-03) —
  `BookingService::create` liefert Ok für einen Slot 15:00–16:00, obwohl
  hypothetischer ShortDay-Cutoff 14:30. Kein 409, kein neuer Error-Variant.

Fixture-Setup: 2026-W31 (Mo = 2026-07-27), `active_from = 2026-07-01`
für Gate-aktiv-Cases; `active_from = 2026-07-28` bzw. `2026-07-27` für
den Boundary-Test.

## Deviations from Plan

**None** — Reuse-Note-Wahl war Option (2) (Helper promoten), wie im Plan
als Präferenz genannt. Test F-Ort ist wie im Plan als Backup genannt
(`test/booking.rs`) — nicht in `test/booking_information.rs`, weil der
zu testende Aufruf `BookingService::create` ist, nicht
`BookingInformationService`.

## Verifikation

- `cargo test --workspace` grün (666 Tests in service_impl, 64 Integration,
  alle anderen Crates grün).
- `cargo clippy --workspace -- -D warnings` grün.
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12 (kein persistierter
  value_type angefasst).
- Kein neuer Cargo-Dep, keine Migration, kein Datei-Delete.

## Self-Check: PASSED

- service_impl/src/booking_information.rs — modified (2 clip-sites)
- service_impl/src/shortday_gate.rs — modified (helper promoted)
- service_impl/src/block.rs — modified (helper re-imported)
- service_impl/src/test/booking_information_chain_c.rs — FOUND (new)
- service_impl/src/test/booking.rs — modified (Test F appended)
- service_impl/src/test/booking_information_vfa.rs — modified (MockToggleService)
- service_impl/src/test/mod.rs — modified (mod declaration)
- shifty_bin/src/main.rs — modified (DI wire-up)
- Commits `62a2f35` (Task 1+2) und `0a8483b` (Task 3+F) FOUND.
