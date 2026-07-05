---
phase: 51
plan: 03
subsystem: shiftplan-read-aggregate
status: complete
tags: [chain-b, shortday, clip, stichtag-gate, bugfix, d-51-06, d-51-07, d-51-09]
dependency_graph:
  requires: [51-01, 51-02]
  provides: ["ShiftplanSlot.effective_to for view-layer", "clipped-week aggregate for WeekView + PDF"]
  affects: ["service::shiftplan::ShiftplanSlot (public struct field)", "service_impl::shiftplan (all 4 aggregate methods)"]
tech_stack:
  added: []
  patterns:
    - "verbatim toggle-lookup pattern from reporting.rs:164-180 (holiday_auto_credit)"
    - "field-init ordering so `effective_to: slot.to` reads Copy value before `slot` is moved"
key_files:
  created: []
  modified:
    - service/src/shiftplan.rs
    - service_impl/src/shiftplan.rs
    - service_impl/src/pdf_render.rs
    - service_impl/src/test/shiftplan.rs
    - shifty_bin/src/main.rs
decisions:
  - "Service-Tier remains Business-Logic — no relocation needed. `ShiftplanViewServiceImpl` already consumes `AbsenceService` + `SalesPersonUnavailableService` (Phase 3), so adding `ToggleService` (Basic-tier) as an additional dep respects the CLAUDE.md tier rule (Basic services do not consume domain services; Business-Logic may consume both)."
  - "Toggle-Lookup happens once per endpoint (not per day of week), so `build_shiftplan_day` takes a pre-resolved `active_from: Option<time::Date>` instead of being async and taking a `ToggleService`. Same pattern as reporting.rs."
  - "`ShortDay` cutoff resolution stayed inside `build_shiftplan_day`; only the gate check (`should_clip(day_date, active_from)`) was added right after it. This keeps the helper's SpecialDay-lookup ergonomics intact."
  - "The struct-field ordering trick `effective_to: slot.to,` before `slot,` in pdf_render.rs test fixtures avoids introducing a local `slot_end` binding at 13 sites — `time::Time` is `Copy`, so no borrow-checker issue."
metrics:
  duration: ~50 min
  completed: 2026-07-04
  tasks_planned: 3
  tasks_completed: 3
---

# Phase 51 Plan 03: Chain B — ShiftplanSlot Read-Aggregat Summary

One-liner: `ShiftplanSlot` gains `effective_to` and `build_shiftplan_day` replaces its ShortDay filter-anti-pattern (`slot.to > cutoff → continue`) with `Slot::clip_to` guarded by the D-51-07 Stichtag-Gate; the pre-existing D-04-Zeile-4 bug (overlap-Slots dropped instead of clipped) is fixed inline.

## What changed

### `service/src/shiftplan.rs`

New public field on `ShiftplanSlot`:

```rust
pub effective_to: time::Time,
```

Doc-Kommentar erklärt: equals `slot.to` unless a ShortDay + active gate clip; `slot.to` bleibt roh (bidirektionale-DTO-Regel, siehe P07).

### `service_impl/src/shiftplan.rs`

- Added `use service::toggle::ToggleService;` und `use crate::shortday_gate;`.
- **`build_shiftplan_day`** — Signatur um `year: u32, week: u8, active_from: Option<time::Date>` erweitert. Body:
  - Nach dem existierenden `short_day_cutoff`-Block wird `effective_cutoff` berechnet: `short_day_cutoff.filter(|_| ...should_clip(day_date, active_from))`.
  - Im Slot-Loop (vorher Zeilen 62–66 mit `if slot.to > cutoff { continue; }`) läuft jetzt:
    ```rust
    let effective_to = if let Some(cutoff) = effective_cutoff {
        match slot.clip_to(cutoff) {
            None => continue,               // D-04 Zeile 3
            Some(clipped) => clipped.to,    // D-04 Zeilen 1, 2, 4
        }
    } else {
        slot.to                              // Legacy
    };
    ```
  - Push-Zeile trägt zusätzlich `effective_to`.
- **`build_shiftplan_day_for_sales_person`** reicht die drei neuen Args verbatim durch.
- **`gen_service_impl!`-Block** um `ToggleService: ... = toggle_service` erweitert.
- Alle vier Konsumenten (`get_shiftplan_week`, `get_shiftplan_day`, `get_shiftplan_week_for_sales_person`, `get_shiftplan_day_for_sales_person`) rufen jetzt vor dem Day-Loop den Toggle ab (verbatim reporting.rs-Muster mit `Unauthorized → None`-Fallback) und übergeben `year, week, active_from` an den Helper.

### `service_impl/src/pdf_render.rs`

13 Test-Fixture-Init-Blöcke für `ShiftplanSlot { ... }` (Zeilen 873, 990, 995, 1037, 1081, 1139, 1229, 1234, 1239, 1247, 1322, 1355, 1376) erhalten `effective_to: <slot>.to,` (Feld VOR `slot,` platziert, damit `.to` als Copy vor dem Move gelesen wird). Semantik-Neutral: alle PDF-Fixtures nutzen ungeclippte Slots, `effective_to == slot.to`.

### `service_impl/src/test/shiftplan.rs`

- `ShiftplanViewServiceDependencies` bekommt `pub toggle_service: MockToggleService` und die assoziierte `type ToggleService = MockToggleService;`.
- `build_dependencies()` liefert Default-Mock `.expect_get_toggle_value().returning(|_, _, _| Ok(None))` (Legacy off).
- **`test_get_shiftplan_week_with_special_days`** überarbeitet: expliziter Tuesday-Fixture (überlappender Slot 12:00–15:00 + vollständig hinter-Cutoff-Slot 15:00–17:00), Toggle wird lokal auf `Some("2020-01-01")` überschrieben. Assertions verifizieren jetzt:
  - Monday.slots.len() == 0 (Holiday, unverändert).
  - Tuesday.slots.len() == 1 (der 15:00–17:00-Slot fehlt via D-04 Zeile 3).
  - `slot.slot.to == 15:00` (roh, D-51-09).
  - `slot.effective_to == 14:00` (Clip, D-04 Zeile 4).
- **`test_build_shiftplan_day_filters_short_day`** umgestellt: statt Legacy-Off (der jetzt nichts clippen würde) fährt der Test mit aktivem Stichtag-Gate. Late-Slot 14:00–18:00 wird jetzt korrekt weggelassen (D-04 Zeile 3), Early-Slot 09:00–12:00 bleibt raw (D-04 Zeile 1).
- Zehn `build_shiftplan_day`-Direktaufrufe in bestehenden Unit-Tests bekommen die drei neuen Args `(2024, 3, None)` (Legacy off — bestehendes Verhalten).
- Fünf **neue Tests** angehängt (siehe Test-Namen unten).

### `shifty_bin/src/main.rs`

- `ShiftplanViewServiceDependencies`-`impl` erhält `type ToggleService = ToggleService;`.
- `ShiftplanViewServiceImpl { ... }`-Konstruktion erhält `toggle_service: toggle_service.clone(),` (der `toggle_service` wird bereits weiter oben in ~Zeile 1064 gebaut).

## Neue Tests (Namen + Zeilennummern nach Commit)

| Test-Name | Lokation | Verifiziert |
| --- | --- | --- |
| `test_build_shiftplan_day_effective_to_unclipped_before_stichtag` | `test/shiftplan.rs:1287` | D-51-07 Vortag: `effective_to == slot.to` |
| `test_build_shiftplan_day_effective_to_clipped_at_stichtag` | `test/shiftplan.rs:1336` | D-51-07 inklusiv am Stichtag + D-04 Zeile 4 |
| `test_build_shiftplan_day_none_active_from_no_clip` | `test/shiftplan.rs:1388` | Legacy `None` → nie clippen |
| `test_build_shiftplan_day_slot_field_stays_raw_when_clipped` | `test/shiftplan.rs:1428` | D-51-09 (`slot.to` bleibt roh) |
| `test_build_shiftplan_day_preserves_bookings_on_clipped_slot` | `test/shiftplan.rs:1476` | SHC-05 |

## Gates

- `cargo test --workspace` → **green** (654 shifty-utils + service_impl + rest + shifty_bin + integration).
- `cargo clippy --workspace --tests --all-targets -- -D warnings` → **green**.
- Fix von 5 pre-existing `cloned_ref_to_slice_refs`-Warnings in den neu geschriebenen Tests (via `std::slice::from_ref`).
- Snapshot-Schema-Version unverändert (kein neuer `value_type`).
- Keine neuen Cargo-Deps. Keine SQL-Änderungen. Kein `sqlx prepare` nötig.

## Deviations from Plan

**None.** Der Plan verlief exakt wie geschrieben; das erwartete Struct-Init-Ripple in `pdf_render.rs` traf ein (13 Sites), alle anderen Fixtures und Tests wurden gemäß dem `<action>`-Block angepasst. Ein Detail, das im Plan nicht namentlich erwähnt war: `test_build_shiftplan_day_filters_short_day` (Zeile ~479, war _kein_ Zielobjekt der `<action>`) musste ebenfalls angefasst werden, weil sein alter Contract (mit `active_from=None` clippen) nicht mehr existiert — der Test läuft jetzt mit aktivem Gate und verifiziert dieselbe D-04-Zeile-3-Semantik.

## Threat Flags

Keine neue Angriffsfläche. Der Toggle-Read läuft im existierenden Auth-Kontext (`context.clone()`), mit demselben `Unauthorized → None`-Fallback wie `reporting.rs` — das Toggle-Datum steuert lediglich Anzeige-Kürzung, keine Berechtigungen.

## Known Stubs

Keine. Alle Änderungen sind vollständig implementiert und getestet.

## Follow-ups für spätere Plans

- **P07 (bidirectional-`SlotTO` audit)** wird verifizieren, dass `slot.to` in TOs weiterhin roh serialisiert wird (D-51-09-Invariante).
- **PDF-Renderer (SHC-04)**: Der PDF-Pfad (`service_impl/src/pdf_shiftplan.rs` → `render_shiftplan_week_pdf`) konsumiert dasselbe `ShiftplanWeek`-Aggregat, ist also **automatisch mit-korrekt**, sobald der Renderer `effective_to` statt `slot.to` für die Anzeige verwendet. Dieser Renderer-Switch ist in einem späteren Plan (P04/P05) zu verifizieren; hier passt sich noch nichts an, da alle Fixtures ungeclippte Slots verwenden.

## Commits

- `8d12645` — `feat(51-03): Chain B — ShiftplanSlot.effective_to + Stichtag-gated clip` (single atomic commit; struct-field addition + gate + tests are inseparable due to the ripple across `pdf_render.rs` fixtures).

## Self-Check: PASSED

- `service/src/shiftplan.rs` — modified, has new `pub effective_to: time::Time` field.
- `service_impl/src/shiftplan.rs` — modified, `use crate::shortday_gate;` present, no `if slot.to > cutoff` line remaining.
- `service_impl/src/pdf_render.rs` — modified, all 13 `ShiftplanSlot { ... }` init blocks include `effective_to`.
- `service_impl/src/test/shiftplan.rs` — modified, five new Phase-51 tests + updated existing test.
- `shifty_bin/src/main.rs` — modified, `toggle_service` wired into `ShiftplanViewServiceImpl`.
- Commit `8d12645` present in git log.

## Gap-Closure (P06-Follow-up, 2026-07-05)

Chain B's inline `Err(Unauthorized) → None`-Match wurde durch den zentralen
Helper `shortday_gate::read_active_from` ersetzt. Verhaltensinvariant — der
Refactor zieht nur die Wiederholung des HCFG-02-Patterns aus vier Konsumenten
in einen Ort. Regression-Guard: `test_get_shiftplan_week_tolerates_toggle_unauthorized`.
- Refactor-Commit: `6088cd0`.
- Test-Commit: `5aee47e`.
