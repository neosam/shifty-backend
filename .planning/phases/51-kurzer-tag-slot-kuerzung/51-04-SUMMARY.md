---
phase: 51
plan: 04
subsystem: backend
tags: [chain-a-prime, block-service, ical, insufficient-booked, stichtag-gate, shortday]
status: complete
requires:
  - 51-01 (Slot::clip_to)
  - 51-02 (shortday_gate helpers)
provides:
  - block-service-clipped-slots
  - ical-clipped-slots-automatic
  - insufficient-booked-clipped-slots
  - my-blocks-clipped-slots-via-delegation
affects:
  - service_impl/src/block.rs
  - service_impl/src/test/block.rs
  - shifty_bin/src/main.rs (BlockServiceDeps + DI wire-up)
tech-stack:
  added: []
  patterns:
    - "Prefetch-Muster für ShortDay + Toggle pro Method-Call (identisch zu ShiftplanViewServiceImpl aus P03)"
    - "Freistehender Helper `clip_slot_for_week(slot, special_days, year, week, active_from) -> ClipOutcome` (Reuse durch beide Aggregat-Methoden)"
    - "`ClipOutcome::Keep(Slot)` / `ClipOutcome::Drop` — Owned-Slot-Rückgabe zwingt `day_map` von `Vec<&Slot>` auf `Vec<Slot>`"
key-files:
  created: []
  modified:
    - service_impl/src/block.rs
    - service_impl/src/test/block.rs
    - shifty_bin/src/main.rs
decisions:
  - "Helper `clip_slot_for_week` als freie Funktion im selben Modul (nicht als Method am BlockServiceImpl) — pure Funktion, keine DB, testbar aus jedem Konsumenten, kein Self-Borrow"
  - "`day_map` in `get_unsufficiently_booked_blocks` von `Vec<&Slot>` auf `Vec<Slot>` umgestellt (Owned) — `clip_to` liefert Owned; alternative `Cow<'_, Slot>` wäre komplexer ohne Benefit"
  - "Redundanten `slots.clone()`-Call im Merge-Loop entfernt (war für die `&Slot`-Kopie nötig, jetzt via `for (dow, mut slots)`-Pattern gelöst)"
  - "MyBlockService (Research §Risks 5) bestätigt als dead code — Trait existiert in `service/src/my_block.rs`, keine Impl, keine DI, keine Aufrufer. Nicht angefasst."
  - "Service-Tier-Check (CLAUDE.md): BlockService ist bereits Business-Logic-Tier (konsumiert BookingService, SlotService, ShiftplanViewService). Zwei neue Basic-Tier-Deps (SpecialDayService + ToggleService) hinzuzufügen bleibt zyklen-frei und tier-konform."
metrics:
  duration_minutes: ~18
  tasks_completed: 4
  files_touched: 3
  commits: 2
  tests_added: 6
  tests_total_module: 17
completed: 2026-07-04
---

# Phase 51 Plan 04: Chain A' — BlockService Slot-Clip vor Merge Summary

Chain A' (D-51-06): pro-Slot-Clip vor den bestehenden Merge-Loops in beiden BlockService-Aggregat-Methoden, damit iCal, insufficient-booked-Report und `get_blocks_for_current_user` (Employee-View) automatisch die geclippten Zeiten zeigen.

## Was gebaut wurde

**BlockService liefert jetzt geclippte Slot-Zeiten in beiden Aggregat-Pfaden.**

- `get_blocks_for_sales_person_week` (`service_impl/src/block.rs:87-96` → jetzt erweitert): Nach dem `slot_service.get_slot(...)`-Call und VOR dem `booking_slot_pairs.push(...)` läuft `clip_slot_for_week(...)` — Slots mit `from >= cutoff` werden übersprungen (D-04 Zeile 3), Slots mit `from < cutoff < to` werden auf `to = cutoff` verkürzt (D-04 Zeile 4). Die Merge-Detection (`slot.from == to`) im Loop danach greift jetzt auf die effektiven, geclippten Zeiten — kein False-Merge über den Cutoff hinaus.
- `get_unsufficiently_booked_blocks` (`:237-269` → jetzt erweitert): analoges Clip vor `day_map.entry(...).push(...)`. `day_map` von `BTreeMap<DayOfWeek, Vec<&Slot>>` auf `BTreeMap<DayOfWeek, Vec<Slot>>` umgestellt (owned), weil `Slot::clip_to` einen neuen owned Slot liefert.
- `get_blocks_for_next_weeks_as_ical` (`:182-225`) und `get_blocks_for_current_user` (`:349-395`) sind **unverändert** — sie delegieren an die zwei gefixten Methoden und ziehen sich das Clip-Verhalten automatisch mit (D-51-05).

Das erfüllt SHC-02 (iCal + insufficient-Report sehen geclippte Zeiten), SHC-05 (bestehende Bookings überleben — nur Aggregat-Sicht filtert, die Booking-DB-Row bleibt untouched; Test B belegt genau das) und SHC-06 (Stichtag-Gate ehrenamtlich am selben Ort wie Chain B/D).

## Deps-Diff (Tier-Analyse)

`BlockServiceImpl` bekommt zwei neue Basic-Tier-Deps:

```
+ SpecialDayService: SpecialDayService<Context = Self::Context>
+ ToggleService: ToggleService<Context = Self::Context, Transaction = Self::Transaction>
```

Beide sind Basic-Tier (nur DAO + Permission + Transaction als Deps). `BlockService` bleibt Business-Logic-Tier (konsumiert bereits BookingService, SlotService, SalesPersonService, ShiftplanViewService, IcalService, ConfigService, ClockService). Kein neuer Cross-Service-Zyklus entsteht.

**DI-Konstruktion in `shifty_bin/src/main.rs:1165-1174`:** `special_day_service.clone()` + `toggle_service.clone()` an die bestehende Handle-Sammlung angehängt. Beide Handles wurden bereits in P03 (für ShiftplanViewServiceImpl) konstruiert bzw. existieren aus früheren Phasen (Phase 25 für special_day, v1.7/HCFG-02 für toggle). Kein Umbau der Konstruktions-Reihenfolge nötig.

## Merge-Reihenfolge-Test-Belege

Kritischer Grenzfall aus Research §Risks 4: erst Clip pro Slot, dann `slot.from == to`-Consecutive-Detection.

**Test A** `test_get_blocks_clips_overlap_slot_at_shortday_cutoff`:
- Slot A Mo 10:00–14:00 + Slot B Mo 14:00–16:00, Cutoff 15:00, Gate ON.
- Erwartetes Ergebnis: **1 Block** Mo 10:00–15:00 mit 2 Slots ([10:00–14:00, 14:00–15:00]).
- Ohne Vor-Merge-Clip käme `block.to = 16:00` raus.
- Assertion `assert_eq!(block.to, 15:00)` beweist Clip-vor-Merge.

**Test B** `test_get_blocks_drops_post_cutoff_slot`:
- Slot 16:00–18:00 mit Cutoff 15:00 → `slot.from >= cutoff` → `ClipOutcome::Drop`.
- Booking-Mock liefert die Row weiterhin (SHC-5-Beleg: DB unangetastet, nur Aggregat-Sicht filtert).
- Assertion: `blocks.is_empty()`.

**Test C** `test_get_blocks_ungated_before_stichtag`:
- Gleiches Setup wie Test A, aber `active_from = Some(2099-01-01)` → Gate OFF.
- Assertion `block.to == 16:00` → beweist, dass das Stichtag-Gate vor dem Clip greift.

**Test D** `test_get_blocks_none_active_from_no_clip`:
- ShortDay existiert, aber `toggle_service` returned `Ok(None)` → `active_from = None` → Gate immer OFF (Rollout-Default).
- Beweist Legacy-Verhalten beim Toggle-Off-Zustand.

**Test E** `test_get_unsufficiently_booked_blocks_respects_cutoff`:
- Ungebuchter Slot 16:00–18:00 mit Cutoff 15:00 + Gate ON → nicht in `insufficient_blocks`.

**Test E-2** `test_get_unsufficiently_booked_blocks_without_shortday_lists_slot` (Sanity-Gegenprobe):
- Gleiche Konstellation, aber KEIN ShortDay → Slot ist drin. Beweist, dass Test E's Leer-Ergebnis wirklich am Clip liegt, nicht an einem anderen Filter.

Alle 17 Tests im `test::block`-Modul grün (11 alt + 6 neu).

## Test-Suite-Ergänzung

`BlockServiceDependencies` bekommt zwei neue Mocks:
- `special_day_service: MockSpecialDayService`
- `toggle_service: MockToggleService`

`build_dependencies()` setzt Legacy-Defaults:
- `special_day_service.expect_get_by_week().returning(|_, _, _| Ok(Arc::new([])))`
- `toggle_service.expect_get_toggle_value().returning(|_, _, _| Ok(None))`

Damit laufen alle 11 bestehenden Tests **ohne inhaltliche Änderung** weiter grün. Insbesondere `test_get_blocks_for_current_user_multiple_weeks` (das drei Wochen iteriert und `get_blocks_for_sales_person_week` 3× aufruft) funktioniert dank `.returning(...)` (unlimited) direkt.

## Deviations from Plan

Keine deviations gegenüber der Plan-Spec. Nennenswert:

1. **Helper-Extraktion umgesetzt (Task 2 „Alternative-Refactor, optional"):** Der Plan schlug vor, den Cutoff/Gate-Lookup als privaten Helper zu extrahieren, wenn es sauberer wird. Ich habe das gemacht — `clip_slot_for_week(...) -> ClipOutcome` lebt als freie Funktion im selben Modul, wird von beiden Aggregat-Methoden konsumiert. Vermeidet Prefetch-Duplizierung im Slot-Iterator (der Prefetch selbst — `special_days` + `active_from` — bleibt an beiden Method-Anfängen dupliziert, weil er pro Method genau einmal läuft; das aus dem Helper zu extrahieren wäre Overkill).
2. **`day_map`-Typ-Umbau in Task 2 wie erwartet:** Von `Vec<&Slot>` auf `Vec<Slot>` — 4 Zeilen (Type-Signatur + zwei `clone()`-Adjustments + der bereits vorhandene `slots.clone()`-Redundanz-Entfall). Deutlich unter der 20-Zeilen-Grenze aus dem Plan.
3. **Zusatz-Test E-2 (Sanity-Gegenprobe) nicht im Plan spezifiziert, aber sinnvoll:** Beweist, dass Test E's Leer-Ergebnis wirklich am Clip liegt. Kein Prüfer-Risiko, nur klarer Beweis-Pfad.

## Success Criteria Check

- [x] `cargo test --workspace` grün (660 Tests im service_impl-Kern-Modul, alle grün).
- [x] `cargo clippy --workspace -- -D warnings` grün.
- [x] Merge-Reihenfolge visuell verifizierbar über Test A (`block.to == 15:00` statt `16:00`).
- [x] Snapshot-Version bleibt 12 (kein `BillingPeriodValueType`-Change, kein Snapshot-Writer angefasst).
- [x] Kein neues `sqlx`-Prepare — keine neue Query, kein DB-Schema angefasst.

## Nachbar-Wave-Impact

- **P05 (Chain C — BookingInformation):** unabhängig, wartet auf Slot-Zeit-Arithmetik-Fix in `booking_information.rs`. Kein Konflikt.
- **P06 (Chain D — ShiftplanReport):** unabhängig, DAO-Layer-Refaktorierung. Kein Konflikt.
- **P07 (DTO/FE-Konsum):** unabhängig — `ShiftplanSlotTO.effective_to` wird aus Chain B gespeist, nicht Chain A'.
- **`MyBlockService`:** bestätigt dead code (Research §Risks 5). Trait existiert in `service/src/my_block.rs:17`, aber **keine Impl, keine DI, keine Aufrufer** in `service_impl/`, `shifty_bin/`. Kein Follow-up nötig; wenn `MyBlockService` je live geht, würde sein Impl vermutlich an `BlockService` delegieren und den Clip damit automatisch mitziehen.

## Commits

- `1354106` — feat(51-04): Chain A' — clip slots before Block merge in BlockService
- `0acb506` — test(51-04): Chain A' — cover clip-before-merge + Stichtag-Gate + insufficient

## Self-Check: PASSED

- Datei `service_impl/src/block.rs` — modified, enthält Helper + Prefetch + Clip in beiden Methoden.
- Datei `service_impl/src/test/block.rs` — modified, 6 neue Tests + Fixture-Erweiterung.
- Datei `shifty_bin/src/main.rs` — modified, `BlockServiceDeps` + DI-Wire-up erweitert.
- Commit `1354106` existiert (feat).
- Commit `0acb506` existiert (test).
- Test-Modul `test::block` — 17 Tests, alle grün (siehe Task-4-Verify-Output).
- Clippy workspace grün.
