---
phase: 51
plan: 01
subsystem: service/slot
tags: [shortday, slot, clip, foundation, wave-1]
status: complete
requires: []
provides: ["Slot::clip_to canonical clip method (D-51-01)"]
affects: ["service/src/slot.rs"]
tech_stack:
  added: []
  patterns: ["Fachobjekt-Methode auf pub struct Slot (Value-Logik ohne Seiteneffekte)"]
key_files:
  created: []
  modified:
    - service/src/slot.rs
decisions:
  - "D-51-01: clip_to lebt auf service::slot::Slot (nicht im shifty-utils-Crate) — alle vier BE-Aggregat-Ketten (P03/P04/P05/P06) haben Slot bereits im Scope"
  - "D-04: Semantik-Tabelle 1:1 in drei Zweige übersetzt (Gleichheit ist explizit kein Sonderfall)"
metrics:
  duration_seconds: 396
  tasks_completed: 3
  files_touched: 1
  tests_added: 4
  completed_at: 2026-07-04T21:09:31Z
---

# Phase 51 Plan 01: Slot::clip_to (Foundation) Summary

Kanonische ShortDay-Clip-Methode `Slot::clip_to(cutoff) -> Option<Slot>` auf `service::slot::Slot` implementiert und mit vier D-04-Grenzfall-Tests abgesichert.

## Signature

```rust
impl Slot {
    /// Wendet den ShortDay-Cutoff auf diesen Slot an (D-51-01 / D-04).
    pub fn clip_to(&self, cutoff: time::Time) -> Option<Slot>
}
```

- **Datei:** `service/src/slot.rs`
- **Position:** Zeile 74–96 (`impl Slot`-Block direkt nach `impl Ord for Slot`, vor `#[automock]` von `SlotService`)
- **Sichtbarkeit:** `pub` — alle BE-Aggregat-Ketten (P03/P04/P05/P06) können direkt konsumieren
- **Panics:** keine. `unwrap()`: keine. `unsafe`: keins. `mut`-Args: keine.

## D-04 Mapping

| D-04 Zeile | Bedingung                          | Rückgabe                                | Code-Zweig (slot.rs)                 |
| ---------- | ---------------------------------- | --------------------------------------- | ------------------------------------ |
| 1          | `slot.to < cutoff`                 | `Some(unchanged)`                       | `self.to <= cutoff` → `Some(clone)`  |
| 2          | `slot.to == cutoff`                | `Some(unchanged)` (kein Sonderfall)     | selber Zweig wie Zeile 1             |
| 3          | `slot.from >= cutoff`              | `None`                                  | `self.from >= cutoff` → `None`       |
| 4          | `slot.from < cutoff < slot.to`     | `Some({..slot, to: cutoff})`            | Fallthrough → `Some(Slot{to,..})`    |

Zeilen 1 + 2 sind bewusst in einem Zweig (`<=`) zusammengelegt — D-04 legt fest, dass Gleichheit kein Sonderfall ist.

## Tests (`#[cfg(test)] mod tests`, Zeile 156–246)

| # | Test-Fn                                                       | Datei-Zeile | Deckt D-04 Zeile |
| - | ------------------------------------------------------------- | ----------- | ---------------- |
| 1 | `clip_to_leaves_slot_unchanged_when_slot_ends_before_cutoff`  | 177         | Zeile 1          |
| 2 | `clip_to_leaves_slot_unchanged_when_slot_ends_exactly_at_cutoff` | 191      | Zeile 2          |
| 3 | `clip_to_returns_none_when_slot_starts_at_or_after_cutoff`    | 204         | Zeile 3 (== und >) |
| 4 | `clip_to_shrinks_slot_when_slot_overlaps_cutoff`              | 222         | Zeile 4 (+ Feld-Erhalt) |

- Fixture-Helper `make_slot(from, to)` (Zeile 160) konstruiert einen validen Slot mit `Uuid::new_v4()`, `DayOfWeek::Tuesday`, `min_resources=1`, `valid_from=2026-01-01`, restliche Felder `None`.
- Test 4 verifiziert zusätzlich per `assert_eq!`, dass `id`, `day_of_week`, `min_resources`, `max_paid_employees`, `valid_from`, `valid_to`, `deleted`, `version`, `shiftplan_id` zwischen Input und Output identisch bleiben — "nur `to` mutieren" ist so per Test verankert.
- Test 3 deckt beide Grenzfälle in einer Fn: `slot.from == cutoff` und `slot.from > cutoff`, beide `None`.

## Verification Gates

| Gate                                                    | Ergebnis |
| ------------------------------------------------------- | -------- |
| `cargo build -p service`                                | ✅ grün  |
| `cargo test -p service --lib slot::tests`               | ✅ 4 passed, 0 failed |
| `cargo test --workspace`                                | ✅ 782 passed, 0 failed |
| `cargo clippy --workspace -- -D warnings`               | ✅ grün, kein `dead_code`-Lint |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION` (billing_period_report.rs:117) | 12 (unverändert) |

## Deviations from Plan

None — Plan wurde exakt wie geschrieben ausgeführt. Signatur 1:1 aus `51-RESEARCH.md § "Empfohlene Signatur"` übernommen; kein Refactoring an bestehenden `impl From<...>`/`impl PartialOrd`/`impl Ord`.

## Consumer Impact

- **Keine Konsumenten diese Wave** — Wave 1 ist Foundation-only.
- Wave 2 (P02–P06) fasst `service_impl/src/{block.rs, shiftplan.rs, booking_information.rs, shiftplan_report.rs}` an und routet über `slot.clip_to(cutoff)`.
- `pub`-Sichtbarkeit reicht dem Compiler — kein `#[allow(dead_code)]` nötig, da die Tests-Konsumenten die Fn benutzen.

## Commits

- `250cc40` feat(51-01): add Slot::clip_to for ShortDay cutoff
- `1ff9951` test(51-01): add D-04 unit tests for Slot::clip_to

## Self-Check: PASSED

- `service/src/slot.rs` FOUND, `impl Slot { pub fn clip_to }` FOUND (Zeile 74), `#[cfg(test)] mod tests` FOUND (Zeile 156)
- Commits `250cc40`, `1ff9951` FOUND in git log
- 782 workspace tests passed; clippy `-D warnings` grün; Snapshot-Version 12 unverändert
