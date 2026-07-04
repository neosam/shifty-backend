---
phase: 51
plan: 02
subsystem: backend
tags: [toggle, migration, shortday, clipping, gate]
status: complete
requires: [51-01]
provides:
  - "shortday_gate::TOGGLE_NAME"
  - "shortday_gate::parse_active_from"
  - "shortday_gate::should_clip"
  - "shortday_gate::resolve_active_from_for_week"
  - "toggle row `shortday_slot_clipping_active_from`"
affects: []
tech-stack:
  added: []
  patterns:
    - "additive INSERT OR IGNORE toggle seed (HCFG-02 precedent)"
    - "pure Rust gate helper (no async, no DAO deps)"
key-files:
  created:
    - "migrations/sqlite/20260704000001_seed-shortday-slot-clipping-toggle.sql"
    - "service_impl/src/shortday_gate.rs"
  modified:
    - "service_impl/src/lib.rs"
decisions:
  - "D-51-07 implemented via toggle `shortday_slot_clipping_active_from` (value=NULL → off, ISO date → clip if booking_date >= date)"
  - "Migration timestamp bumped from 20260704000000 to 20260704000001 to avoid collision with the pre-existing pdf-export-cron hotfix migration"
metrics:
  duration: "8m03s"
  completed: "2026-07-04"
  tasks_completed: 3
  files_created: 2
  files_modified: 1
---

# Phase 51 Plan 02: Toggle-Seed-Migration + `shortday_gate`-Helper Summary

Wave-1-Backbone für SHC-06: Additive Toggle-Seed-Migration
`shortday_slot_clipping_active_from` (analog HCFG-02 aus v1.7) plus
wiederverwendbarer `service_impl::shortday_gate`-Helper mit purer
Stichtag-Gate-Semantik. Kein Schema-Change, Snapshot-Version bleibt 12.

## Deliverables

### Migration

**Datei:** `migrations/sqlite/20260704000001_seed-shortday-slot-clipping-toggle.sql`

```sql
INSERT OR IGNORE INTO toggle (name, enabled, description, update_process)
VALUES (
    'shortday_slot_clipping_active_from',
    0,
    'When a cutoff date is set in `value` (ISO YYYY-MM-DD), slots at short-day dates >= that date are clipped at the ShortDay cutoff time in rendering and hours calculation. Leave value NULL to disable (legacy behavior).',
    'phase-51-migration'
);
```

- Additive (`INSERT OR IGNORE`) — idempotent, kein Schema-Change.
- `enabled = 0`, `value = NULL` → Rollout-Default „Kürzung aus".
- `update_process = 'phase-51-migration'` als Herkunfts-Marker.
- **Timestamp geändert von `20260704000000` → `20260704000001`** wegen
  Kollision mit bereits existierender `20260704000000_fix-pdf-export-cron-6-field.sql`
  (v2.3-Hotfix, gleicher Datums-Prefix). sqlx nutzt die numerische
  Timestamp-Zahl als Unique-Key in `_sqlx_migrations`.

### Helper-Modul `service_impl::shortday_gate`

**Datei:** `service_impl/src/shortday_gate.rs`
**Registrierung:** `service_impl/src/lib.rs` — neue Zeile `pub mod shortday_gate;`
alphabetisch zwischen `shiftplan_report` und `slot`.

**API:**

| Item | Signatur | Semantik |
|---|---|---|
| `TOGGLE_NAME` | `pub const &str` | `"shortday_slot_clipping_active_from"` — Konsumenten übergeben diesen an `ToggleService::get_toggle_value` statt Magic-String |
| `parse_active_from` | `fn(Option<&str>) -> Option<time::Date>` | `None`/`""`/malformed → `None`; sonst ISO-8601-Parse via `Iso8601::DEFAULT` (analog `reporting.rs:173-179`) |
| `should_clip` | `fn(Date, Option<Date>) -> bool` | `None` → `false`; sonst `booking_date >= active_from` (inklusiv am Stichtag) |
| `resolve_active_from_for_week` | `fn(u32, u8, DayOfWeek, Option<Date>) -> bool` | Baut `Date` aus ISO-Woche + Weekday; defensiver Skip (`false`) bei ungültiger Woche; delegiert an `should_clip` |

Pure Fns — keine `Result`, kein `async`, keine DAO/Service-Deps. Volle
Testabdeckung ohne Fixtures.

**Tests** (`#[cfg(test)] mod tests`) — 9 Cases:

- `parse_none_returns_none`
- `parse_empty_returns_none`
- `parse_malformed_returns_none`
- `parse_iso_valid`
- `should_clip_none_active_from_returns_false`
- `should_clip_before_stichtag_returns_false` (Vortag 2026-07-31 vs 2026-08-01)
- `should_clip_at_or_after_stichtag_returns_true` (inklusiv am Stichtag)
- `resolve_active_from_for_week_delegates_to_should_clip`
- `resolve_active_from_for_week_returns_false_on_invalid_week` (2025 hat keine Wo 53)

## Verification

- ✅ `cargo test -p service_impl --lib shortday_gate::tests` — 9 passed
- ✅ `cargo test --workspace` — 645+ passed, 0 failed (nach Migration-Rename)
- ✅ `cargo clippy --workspace -- -D warnings` — clean
- ✅ `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt `12` (grep verifiziert)
- ✅ Migration-Datei existiert, enthält Toggle-Namen und `phase-51-migration`-Marker

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] Migration-Timestamp-Kollision**
- **Found during:** Task 3 (workspace gates)
- **Issue:** Plan spezifizierte Filename
  `20260704000001_seed-shortday-slot-clipping-toggle.sql`… nein Moment:
  Plan spezifizierte `20260704000000_seed-shortday-slot-clipping-toggle.sql`.
  Dieser Timestamp war bereits vergeben durch
  `20260704000000_fix-pdf-export-cron-6-field.sql` (v2.3-Hotfix, gleicher
  Tag). sqlx nutzt die numerische Version als Unique-Key → 4 Integration-
  Tests failten mit `UNIQUE constraint failed: _sqlx_migrations.version`.
- **Fix:** `git mv` → `20260704000001_…`. Migration-Inhalt unverändert.
  Migration bleibt in-order nach dem pdf-export-Fix.
- **Commit:** `69748a6`

### TDD Gate Compliance

- RED-Commit `40ef5a5` (`test(51-02): add failing tests …`) — 9 Tests
  gegen `unimplemented!()`-Stubs, `cargo test` schlägt fehl (verifiziert).
- GREEN-Commit `9eef0ad` (`feat(51-02): implement shortday_gate helper`) —
  alle 9 Tests grün.
- REFACTOR: keine Umbauten nötig, Impl direkt sauber.

## Wave-2-Konsum-Ausblick

Alle vier Aggregat-Ketten (P03/P04/P05/P06) konsumieren dieses Modul so:

```rust
let raw = toggle_service
    .get_toggle_value(shortday_gate::TOGGLE_NAME, ctx, None)
    .await
    .ok()
    .flatten(); // Option<String>
let active_from = shortday_gate::parse_active_from(raw.as_deref());

for slot in raw_slots {
    let booking_date = /* … pro Kette unterschiedlich … */;
    let clipped = if shortday_gate::should_clip(booking_date, active_from) {
        slot.clip_to(cutoff)  // aus P01
    } else {
        Some(slot.clone())  // Legacy / vor Stichtag
    };
    // …
}
```

## Self-Check: PASSED

- Migration-Datei: `migrations/sqlite/20260704000001_seed-shortday-slot-clipping-toggle.sql` → FOUND
- Helper-Modul: `service_impl/src/shortday_gate.rs` → FOUND
- `pub mod shortday_gate;` in `service_impl/src/lib.rs` → FOUND
- Commits `87ca244`, `40ef5a5`, `9eef0ad`, `69748a6` → all present in `git log`
- `cargo test --workspace` green, `cargo clippy --workspace -- -D warnings` green
- Snapshot-Version 12 unverändert
