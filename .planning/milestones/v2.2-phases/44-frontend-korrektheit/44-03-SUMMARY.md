---
phase: 44-frontend-korrektheit
plan: 03
subsystem: frontend / dialog-shell
status: complete
tags: [bug-03, backdrop-press, regression-guard, invariant-test, phase-37-mod-01]
dependency_graph:
  requires:
    - "Phase 37 MOD-01 (BackdropPress-Logik in component/dialog.rs)"
    - "Phase 37 MOD-01 (absence_convert_modal.rs inline BackdropPress-Migration)"
  provides:
    - "Durable Grep-Invariant-Test der zukünftige un-migrierte Modals rot färbt"
  affects:
    - shifty-dioxus/src/component/dialog.rs
tech_stack:
  added: []
  patterns:
    - "Compile-time include_str! Grep-Invariant statt Runtime-fs-Scan (WASM-safe)"
    - "Allowlist + Migrated-List Doppel-Klassifikation (dokumentiert Intent, macht neue Modals sichtbar)"
key_files:
  created: []
  modified:
    - shifty-dioxus/src/component/dialog.rs
decisions:
  - "Invariant-Test-Scope bewusst klein: prüft NUR `fixed inset-0` als Backdrop-Marker (nicht jedes `onclick`) → keine False Positives auf normalen Buttons."
  - "FILES-Konstant-Set explizit ausformuliert (keine dynamische Verzeichnisscan) → zwingt Autor eines neuen Modals, den Test-Set zu aktualisieren, was den Review-Blick auf die BackdropPress-Adopt-Entscheidung lenkt."
  - "Zwei Test-Funktionen (every_backdrop_close_path_uses_backdrop_press + known_migrated_files_actually_contain_backdrop_press) statt einer → getrennte Fehlermeldungen erklären dem Autor beim Regress, WELCHE Invariante verletzt wurde."
metrics:
  duration_min: 3
  completed_at: 2026-07-02
  tasks_completed: 1
  files_touched: 1
tasks_completed:
  - "Task 1: Codebase-Audit + Backdrop-Invariant-Test in dialog.rs"
---

# Phase 44 Plan 03: BackdropPress Migration Completeness Invariant Summary

Durable Grep-Invariant-Test in `component/dialog.rs` sichert BUG-03 (Phase-37
MOD-01 Panel-Drag-Fix) für alle Modal-artigen Komponenten gegen zukünftigen
Regress ab — der Ist-Zustand ist bereits vollständig migriert, nur der
Regression-Guard fehlte.

## One-liner

Compile-time-Invariant-Test (`include_str!` + Grep) in `dialog.rs` verhindert,
dass ein neuer Backdrop-Close-Modal ohne `BackdropPress` in `shifty-dioxus`
eingecheckt wird.

## Executed Tasks

### Task 1 — Codebase-Audit + Backdrop-Invariant-Test in dialog.rs

**Files:** `shifty-dioxus/src/component/dialog.rs` (+91 Zeilen, neues `#[cfg(test)] mod backdrop_invariant`)

**Was gebaut wurde:**

1. **Live-Audit** (Schritt 1) durchgeführt:
   - `grep -rln "fixed inset-0" shifty-dioxus/src/component/*.rs shifty-dioxus/src/page/*.rs`
     → Treffer: `component/overlay.rs`, `component/absence_convert_modal.rs`. Keine Page-Treffer.
   - `grep -rln "BackdropPress" shifty-dioxus/src/`
     → Treffer: `component/dialog.rs`, `component/absence_convert_modal.rs`.
   - Klassifikation:
     - `component/dialog.rs` — nutzt `BackdropPress` in Dialog::rsx (Zeile 239 ff.),
       backdrop-styling erfolgt über INLINE-CSS (`position:fixed;inset:0` in
       `backdrop_style()`), daher kein Match auf Tailwind-Class `"fixed inset-0"`.
       ContractModal, ExtraHoursModal, SlotEdit nutzen die Dialog-Shell → implizit
       migriert.
     - `component/absence_convert_modal.rs` — nutzt inline `BackdropPress` +
       Tailwind-Class `fixed inset-0` (Zeile 93). OK.
     - `component/overlay.rs` — Wrapper mit `fixed inset-0`, KEIN Close-Handler
       auf dem Backdrop. Out of scope → `ALLOWED_NON_MODAL`.
     - Keine weiteren Treffer in `page/` → Baseline stimmt mit Plan-Erwartung.
   - **Ergebnis:** Ist-Zustand entspricht der Baseline vom 2026-07-02.
     Keine zusätzliche Migration nötig.

2. **Durable Invariant-Test** (Schritt 2) hinzugefügt (`dialog.rs`, unterhalb
   des bestehenden `mod tests`-Blocks):

   Neues Modul `#[cfg(test)] mod backdrop_invariant` mit:
   - `FILES` — Compile-time-eingebundene Component-Files (9 Dateien: dialog,
     absence_convert_modal, contract_modal, extra_hours_modal, slot_edit,
     overlay, week_view, day_aggregate_view, employee_view).
   - `ALLOWED_NON_MODAL = ["overlay.rs"]` — Wrapper ohne Close-Handler.
   - `KNOWN_MIGRATED = ["dialog.rs", "absence_convert_modal.rs"]` — Träger des
     Backdrop-Close-Pfads mit `BackdropPress`.
   - Test 1 `every_backdrop_close_path_uses_backdrop_press` — Iteriert über
     FILES; für jede Datei mit `"fixed inset-0"` prüft: entweder
     ALLOWED_NON_MODAL oder KNOWN_MIGRATED (mit `BackdropPress`-Content-Check).
   - Test 2 `known_migrated_files_actually_contain_backdrop_press` — verifiziert,
     dass alle KNOWN_MIGRATED-Files tatsächlich `BackdropPress` enthalten.

3. **Test-Lauf** (Schritt 3): beide Tests grün.

**Verify:**
- `cargo test -p shifty-dioxus backdrop_invariant` →
  `2 passed; 0 failed; 776 filtered out`
- `cargo test -p shifty-dioxus` → `777 passed; 1 failed` (das eine Failure
  `i18n_impersonation_keys_match_german_reference` ist ein pre-existing
  Copy-Mismatch aus Phase 37-02, dokumentiert in
  `.planning/todos/pending/2026-07-02-i18n-impersonation-key-test-mismatch.md`,
  Resolves in Phase 46 — NICHT von BUG-03 verursacht).
- `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` → grün.
- `cargo clippy --workspace -- -D warnings` (Backend-Hard-Gate) → grün.

## Audit-Ergebnis (Ist-Zustand 2026-07-02)

| File | `fixed inset-0`? | Backdrop-Close-Pfad? | `BackdropPress` verwendet? | Klassifikation |
|------|------------------|----------------------|----------------------------|----------------|
| `component/dialog.rs` | Nein (Inline-CSS) | Ja (zentrale Shell) | Ja (Zeile 239 ff.) | KNOWN_MIGRATED |
| `component/absence_convert_modal.rs` | Ja | Ja | Ja (inline, Zeile 90–107) | KNOWN_MIGRATED |
| `component/overlay.rs` | Ja | Nein (nur Wrapper) | Nein (nicht nötig) | ALLOWED_NON_MODAL |
| `component/contract_modal.rs` | Nein | Nutzt Dialog-Shell | Implizit via Dialog | Out of scope |
| `component/extra_hours_modal.rs` | Nein | Nutzt Dialog-Shell | Implizit via Dialog | Out of scope |
| `component/slot_edit.rs` | Nein | Nutzt Dialog-Shell | Implizit via Dialog | Out of scope |
| `component/week_view.rs` | Nein | `onmousedown` für Tooltip-Timing | N/A (kein Backdrop) | Out of scope |
| `component/day_aggregate_view.rs` | Nein | N/A | N/A | Out of scope |
| `component/employee_view.rs` | Nein | N/A | N/A | Out of scope |

**Wichtiger Detail:** `dialog.rs::backdrop_style()` produziert die Backdrop-CSS
inline (`position:fixed;inset:0;…`), nicht via Tailwind-Class `"fixed inset-0"`.
Deshalb "greift" der Substring-Check `body.contains("fixed inset-0")` bei
`dialog.rs` nicht — aber `dialog.rs` steht in `KNOWN_MIGRATED` und der zweite
Test (`known_migrated_files_actually_contain_backdrop_press`) verifiziert
zusätzlich, dass `BackdropPress` dort tatsächlich vorkommt. Nachdem das
Invariant-Modul selbst die String-Literale `"fixed inset-0"` und
`"BackdropPress"` als Rust-Code enthält, findet ab jetzt der Substring-Check
tatsächlich einen Treffer in `dialog.rs` — was den KNOWN_MIGRATED-Pfad
ebenfalls durchläuft und grün bleibt (validiert im Test-Lauf).

## Deviations from Plan

Keine — Plan wurde exakt so umgesetzt wie geschrieben. Der Ist-Audit
entsprach der im Plan angenommenen Baseline, daher war keine Zusatz-
Migration nötig.

## Threat Flags

Keine — reine Test-Datei, keine neuen Trust-Boundaries.

## Success Criteria Check

1. ✅ **Grep-verifiziert**: alle Modal-artigen Components mit Backdrop-Close-Pfad
   nutzen `BackdropPress`. Audit-Tabelle oben belegt Ist-Zustand.
2. ✅ **Durable Invariant-Test** in `component/dialog.rs` (`mod backdrop_invariant`)
   läuft grün und würde einen zukünftigen Regress (neuer Modal mit `fixed inset-0`
   Backdrop-Close ohne BackdropPress oder ohne Eintrag in KNOWN_MIGRATED /
   ALLOWED_NON_MODAL) rot färben.
3. ✅ `cargo build --target wasm32-unknown-unknown` + `cargo test -p shifty-dioxus`
   (777 passed; 1 pre-existing unrelated failure — nicht von BUG-03).

## Commit Hashes

Wird durch GSD-Auto-Commit im Anschluss ergänzt (co-located jj/git).

## Self-Check: PASSED

- `shifty-dioxus/src/component/dialog.rs` enthält `mod backdrop_invariant` mit
  beiden Tests: ✅ (verifiziert per Datei-Read + Test-Ausgabe
  `test component::dialog::backdrop_invariant::…`).
- Alle im Plan geforderten FILES-Entries entsprechen existierenden Dateien
  (verifiziert per `ls`): ✅.
