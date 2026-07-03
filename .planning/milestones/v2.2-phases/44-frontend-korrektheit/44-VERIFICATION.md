---
phase: 44-frontend-korrektheit
verified_at: 2026-07-02T00:00:00Z
verified: 2026-07-02T00:00:00Z
status: passed
score: 3/3 must-haves verified
behavior_unverified: 0
overrides_applied: 0
---

# Phase 44: Frontend-Korrektheit — Verification Report

**Phase Goal (ROADMAP):** Drei pre-existing Frontend-Bugs abarbeiten: `save_slot_edit` hält keinen `SLOT_EDIT_STORE`-Write-Borrow mehr über `.await`, `list_user_invitations` meldet Parse-Fehler sichtbar, und noch nicht migrierte Modals nutzen die drag-safe `BackdropPress`-Logik.
**Verified:** 2026-07-02
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### ROADMAP Success Criteria — Observable Truths

| # | Truth (ROADMAP SC) | Status | Evidence |
|---|--------------------|--------|----------|
| 1 | `save_slot_edit` liest alle nötigen Store-Felder VOR jedem `.await` in lokale Werte — Regressionstest, der eine borrow-across-await-Panic simuliert, ist grün. | VERIFIED | `src/service/slot_edit.rs:128-173` — `save_slot_edit` nutzt `snapshot_for_save` in Block-Scope (Read-Guard dropped auf Zeile 133), `.await` läuft rein auf `snapshot` (owned), Write-Guard erst nach `.await` in eigenem Block (Zeile 162-165). 6 neue Regressionstests in `service::slot_edit::tests` grün (`cargo test slot_edit` → 6/6 passing). |
| 2 | `list_user_invitations`-Loader unterscheidet „leere Liste" von „Parse-Fehler" — Fehler-Zweig rendert sichtbare Fehler-Meldung, kein silent-empty. | VERIFIED | `src/error.rs`: neue Variante `InvitationParse(String)`. `src/api.rs`: silent-empty `Ok(Rc::new([]))`-Fallback entfernt (`grep -c` = 0). `src/service/user_management.rs:64,376,387`: Store-Feld `user_invitations_load_error: Option<ImStr>` gesetzt bei Err, geresettet bei Ok. `src/page/user_details.rs:166`: `if let Some(err) = user_management.user_invitations_load_error` rendert Inline-Banner. i18n-Key `UserInvitationsLoadError` in de/en/cs/mod.rs. 5 neue `parse_invitations_tests` grün. |
| 3 | Alle Dialoge/Modals unterhalb `component/`, die den Backdrop-Close-Pfad nutzen, gehen über `BackdropPress` (verifiziert per Grep); Panel-Drag mit mouseup-außerhalb schließt kein Modal mehr. | VERIFIED | Audit-Grep: `fixed inset-0` in `component/*.rs` → nur `overlay.rs` (ALLOWED_NON_MODAL, kein Close-Handler), `absence_convert_modal.rs` (KNOWN_MIGRATED, nutzt inline `BackdropPress`), `dialog.rs` (via Invariant-Test-Literal). Dialog-Shell mit zentraler `BackdropPress`-Logik (`src/component/dialog.rs:130-239`). Neues Modul `#[cfg(test)] mod backdrop_invariant` in `dialog.rs:805+` mit 2 durable Invariant-Tests — beide grün (`cargo test backdrop_invariant` → 2/2). |

**Score:** 3/3 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `shifty-dioxus/src/service/slot_edit.rs` | Refactor + 6 tests, no long-lived write-guard across .await | VERIFIED | `snapshot_for_save`, `apply_save_outcome`, `SaveMode`, `SaveOutcome`, `SaveSlotEditSnapshot` alle vorhanden (Zeile 59-125). 6 Tests unter `mod tests` alle grün. Grep bestätigt: kein `mut store = SLOT_EDIT_STORE.write()` zwischen den `.await`-Punkten in `save_slot_edit`. |
| `shifty-dioxus/src/error.rs` | Neue Variante `InvitationParse(String)` | VERIFIED | grep `InvitationParse` in `error.rs` = 1 Vorkommen (Variante deklariert). |
| `shifty-dioxus/src/api.rs` | Parse-Fehler propagieren, silent-empty entfernt | VERIFIED | grep `Ok(Rc::new([]))` = 0. grep `InvitationParse` = 11 Vorkommen (parse_invitations_response + Tests). |
| `shifty-dioxus/src/service/user_management.rs` | Store-Feld + Err-Handling | VERIFIED | `user_invitations_load_error: Option<ImStr>` Zeile 64; reset in Ok-Zweig Zeile 376; set in Err-Zweig Zeile 387. |
| `shifty-dioxus/src/page/user_details.rs` | Inline-Banner | VERIFIED | `if let Some(err) = user_management.user_invitations_load_error.as_ref()` Zeile 166 — rendert roten Banner. |
| `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs` | Key `UserInvitationsLoadError` in allen 4 | VERIFIED | grep bestätigt Präsenz in allen 4 Files. |
| `shifty-dioxus/src/component/dialog.rs` | `mod backdrop_invariant` mit Grep-Scan | VERIFIED | Modul auf Zeile 805 mit `FILES` (9 Component-Files via `include_str!`), `ALLOWED_NON_MODAL=["overlay.rs"]`, `KNOWN_MIGRATED=["dialog.rs","absence_convert_modal.rs"]`, 2 Tests. |
| `.planning/todos/…save-slot-edit-borrow-across-await.md` | Resolved | VERIFIED | Datei liegt in `.planning/todos/completed/`, nicht mehr in `pending/`. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `SlotEditAction::SaveSlot` | `save_slot_edit().await` | Coroutine-driver | WIRED | Signatur unverändert; Refactor rein intern. |
| `api::list_user_invitations` | `loader → service → user_details.rs Render` | ShiftyError-Propagation | WIRED | Error-Typ konsistent `ShiftyError` durchpropagiert, Store-Feld steuert Banner, ERROR_STORE feuert Overlay. |
| Dialog-Shell | ContractModal/ExtraHoursModal/SlotEdit | `use Dialog { ... }` | WIRED | Diese Modals nutzen die zentrale Shell → implizit BackdropPress. |
| `absence_convert_modal.rs` | Inline BackdropPress | onmousedown/onclick | WIRED | Inline-Migration Zeile 90-107 (Phase 37 MOD-01). |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| BUG-01 Regressionstests grün | `cargo test -- service::slot_edit` | 6 passed | PASS |
| BUG-02 Parse-Tests grün | `cargo test -- parse_invitations` | 5 passed | PASS |
| BUG-03 Invariant-Tests grün | `cargo test -- backdrop_invariant` | 2 passed | PASS |
| Silent-empty entfernt | `grep -c 'Ok(Rc::new(\[\]))' src/api.rs` | 0 | PASS |
| Todo-Datei umgezogen | `ls .planning/todos/completed/…borrow-across-await.md` | vorhanden | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| BUG-01 | 44-01-PLAN.md | save_slot_edit ohne write-borrow across .await | SATISFIED | Snapshot-Pattern + 6 Tests + Grep-Gate |
| BUG-02 | 44-02-PLAN.md | list_user_invitations Parse-Fehler sichtbar | SATISFIED | ShiftyError::InvitationParse + Banner + i18n + 5 Tests |
| BUG-03 | 44-03-PLAN.md | BackdropPress-Audit + durable Invariant-Test | SATISFIED | 2 Invariant-Tests + Audit-Baseline dokumentiert |

### Anti-Patterns Found

Keine BUG-blocking Anti-Patterns. Der pre-existing i18n-Impersonation-Test-Failure (`i18n_impersonation_keys_match_german_reference`) ist explizit **out of scope für Phase 44** (Phase 46 zuständig, Todo unter `.planning/todos/pending/2026-07-02-i18n-impersonation-key-test-mismatch.md`). Frontend-Clippy-Baseline ist ~198 pre-existing Lints und NICHT gegated in CI — Phase 45 zuständig.

### Human Verification Required

Keine — alle Truths sind über Regressionstests + Grep-Gates VERIFIED.

Der Plan-Autor markierte manuelle Live-Sanity-Checks (Browser-e2e Panel-Drag; Backend-JSON-Patching für Banner-Trigger) als optional/informativ (D-25-06-Klasse: WASM-Browser-e2e nicht deterministisch). Struktur-Verifikation über die neuen Regressions- und Invariant-Tests ist die Nyquist-Ebene und deckt alle Codepfade.

### Gaps Summary

Keine Gaps. Alle drei ROADMAP-Success-Criteria sind durch Refactor-Code + neue Regressionstests + durable Grep-Invariant erfüllt. Die drei Bugs sind behoben, die Regressionsguards eingezogen, der WR-02-v1.10-Todo aufgelöst.

---

_Verified: 2026-07-02_
_Verifier: Claude (gsd-verifier)_
