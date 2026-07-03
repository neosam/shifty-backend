---
phase: 44-frontend-korrektheit
plan: 01
subsystem: shifty-dioxus / service / slot_edit
tags: [frontend, bugfix, refactor, dioxus, signal, borrow-checker]
requirements:
  - BUG-01
status: complete
completed: 2026-07-02
key-files:
  modified:
    - shifty-dioxus/src/service/slot_edit.rs
  moved:
    - .planning/todos/pending/2026-06-30-fe-save-slot-edit-borrow-across-await.md → .planning/todos/completed/…
decisions:
  - "Snapshot-Pattern: alle Store-Felder VOR dem ersten `.await` in eine owned `SaveSlotEditSnapshot` extrahieren; Write-Guard nach `.await` frisch aufmachen und pure `apply_save_outcome` anwenden."
  - "Read-Guard statt Write-Guard vor dem `.await`, weil in dieser Phase nur gelesen wird — der Write-Guard entsteht erst nach dem `.await`."
  - "SaveMode als drei-Zustand-Enum (`AllFromWeek | SingleWeek | Create`) statt boolean-Tupel — macht die Loader-Dispatch-Logik am Snapshot testbar ohne Signale."
  - "`trigger_shiftplan_refresh()` bleibt an die `SaveOutcome`-Variante gekoppelt (`EditSaved | CreateSucceeded` → refresh, `CreateFailed` → kein refresh) — Semantik des pre-refactor `return Ok(());`-Zweigs unverändert."
metrics:
  new_tests: 6
  files_changed: 1
  todo_resolved: 2026-06-30-fe-save-slot-edit-borrow-across-await.md
status_note: BUG-01 abgeschlossen; WR-02 v1.10 aufgelöst.
---

# Phase 44 Plan 01: BUG-01 save_slot_edit Borrow-across-await Summary

**One-liner:** `save_slot_edit` refaktoriert auf Snapshot-vor-`.await` + Pure-fn-Outcome-Application; kein `SLOT_EDIT_STORE`-Guard lebt mehr über einen `.await`-Punkt, 6 neue Regressionstests.

## Objective (erfüllt)

BUG-01 (v2.2, WR-02 v1.10) beheben: der `save_slot_edit`-Handler in
`shifty-dioxus/src/service/slot_edit.rs` hielt bereits im Edit-Zweig eine
`SLOT_EDIT_STORE.write()`-Referenz `store` über mehrere `.await`-Punkte hinweg
(`loader::save_slot`, `loader::save_slot_single_week`, `loader::create_slot`).
Der Guard wurde nicht droppt, sodass parallele Signal-Zugriffe aus anderen
Koroutinen (`new_slot_edit`, `update_slot_edit`, Component-Reads) einen
`already borrowed`-Panic auslösen konnten.

## Änderungen

**`shifty-dioxus/src/service/slot_edit.rs`**

Neue Typen (`pub(crate)`):

- `enum SaveMode { AllFromWeek, SingleWeek, Create }` — reines Dispatch-Enum.
- `struct SaveSlotEditSnapshot { slot_clone, year, week, mode }` — owned Snapshot
  vor `.await`.
- `enum SaveOutcome { EditSaved, CreateSucceeded, CreateFailed }` — Effekt-Marker
  nach `.await`.

Neue Pure Functions (`pub(crate)`):

- `snapshot_for_save(&SlotEdit) -> SaveSlotEditSnapshot` — mapped
  `slot_edit_type × single_week` auf `SaveMode` und klont `slot`/`year`/`week`.
- `apply_save_outcome(&mut SlotEdit, SaveOutcome)` — mutiert `visible`/`has_errors`
  gemäß Outcome:
  - `EditSaved` → `visible=false`, `has_errors` **unangetastet**.
  - `CreateSucceeded` → `visible=false`, `has_errors=false`.
  - `CreateFailed` → `has_errors=true`, `visible` bleibt (Modal bleibt offen).

Refactor `save_slot_edit`:

1. Block-scope `let snapshot = { let store = SLOT_EDIT_STORE.read(); snapshot_for_save(&store) };` — Read-Guard droppt sofort.
2. `.await` läuft rein auf `snapshot` — kein Signal-Zugriff während des Netzwerks.
3. Nach `.await` block-scope `let mut store = SLOT_EDIT_STORE.write(); apply_save_outcome(&mut store, outcome);` — Write-Guard droppt vor dem `trigger_shiftplan_refresh()`.
4. `trigger_shiftplan_refresh()` nur bei `EditSaved | CreateSucceeded` — `CreateFailed` überspringt den Refresh (Semantik des alten `return Ok(());`-Zweigs).

Kein Signatur-Wechsel — `slot_edit_service`-Dispatcher unverändert.

## Tests (vorher / nachher)

| | vorher | nachher |
|-|--------|---------|
| Tests in `service::slot_edit` | 0 | **6** (alle grün) |

Neue Tests (`#[cfg(test)] mod tests` am Datei-Ende):

1. `snapshot_for_save_edit_multi_week_maps_to_all_from_week`
2. `snapshot_for_save_edit_single_week_maps_to_single_week`
3. `snapshot_for_save_new_maps_to_create`
4. `apply_save_outcome_edit_saved_closes_modal_and_leaves_has_errors_alone`
5. `apply_save_outcome_create_failed_sets_error_and_keeps_modal_open`
6. `apply_save_outcome_create_succeeded_closes_modal_and_clears_errors`

## Verify

- `cargo test -p shifty-dioxus service::slot_edit -- --nocapture`
  → `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 767 filtered out`.
- `cargo build --target wasm32-unknown-unknown` (aus `shifty-dioxus/`) → `Finished dev [unoptimized + debuginfo] target(s) in 1m 03s`, keine Fehler.
- Grep-Gate: im refaktorierten `save_slot_edit`-Body (Zeilen 128–170) existiert
  **kein** `let mut store = SLOT_EDIT_STORE.write()`-Binding zwischen den `.await`-Punkten.
  Die einzige `.write()`-Bindung (Zeile 163) liegt **nach** dem letzten `.await`
  (Zeile 153) und wird durch einen Block-Scope vor dem Funktions-Ende geschlossen.
- Backend Clippy Hard-Gate: `cargo clippy --workspace -- -D warnings` grün
  (kein Backend-Code angefasst).

## Semantik-Erhalt (Nachweis)

| Zweig | vorher | nachher |
|-------|--------|---------|
| Edit + `!single_week` | `save_slot`.await; `visible=false`; refresh | `save_slot`.await; `EditSaved` → `visible=false`; refresh |
| Edit + `single_week` | `save_slot_single_week`.await; `visible=false`; refresh | `save_slot_single_week`.await; `EditSaved` → `visible=false`; refresh |
| New + create_ok | `create_slot`.await; `visible=false`; refresh | `create_slot`.await; `CreateSucceeded` → `visible=false`, `has_errors=false`; refresh |
| New + create_failed | `create_slot`.await; `has_errors=true`; **return** (kein Refresh, `visible` bleibt) | `create_slot`.await; `CreateFailed` → `has_errors=true`, `visible` bleibt; **kein** Refresh |

Verhalten identisch, Panic-Risiko eliminiert.

## Aufgelöstes Deferred-Item

`.planning/todos/pending/2026-06-30-fe-save-slot-edit-borrow-across-await.md`
(WR-02 v1.10, Phase 35 Deferred) → verschoben nach
`.planning/todos/completed/…` und mit `resolved`-Metadaten + Erklärungs-Notiz versehen.

## Restrisiken

- Keine — reiner FE-Refactor, kein Backend/REST/Persistenz-Touch, kein DTO-Wechsel.
- Manuelle Browser-e2e-Sanity (D-25-06-Klasse) nicht deterministisch triggerbar
  (Race brauchte parallele Component-Reads exakt während des Netzwerks); die
  Pure-fn-Tests + der Grep-Gate sind die Nyquist-Ebene, ein Rückfall würde
  in Code-Review auffallen (Kommentar-Marker + Struktur).

## Self-Check: PASSED

- FOUND: shifty-dioxus/src/service/slot_edit.rs (modified, tests present, refactor applied).
- FOUND: .planning/todos/completed/2026-06-30-fe-save-slot-edit-borrow-across-await.md (moved from pending).
- FOUND: 6 tests passing in `cargo test -p shifty-dioxus service::slot_edit`.
- Grep gate satisfied: no `SLOT_EDIT_STORE.write()` binding between `.await` points in `save_slot_edit`.
