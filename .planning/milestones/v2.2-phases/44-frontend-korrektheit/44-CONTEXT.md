# Phase 44: Frontend-Korrektheit (FE) - Context

**Gathered:** 2026-07-02
**Status:** Ready for planning
**Mode:** Autonomous — 3 pre-existing FE-Bugs, klare Success Criteria, keine offenen Design-Fragen.

<domain>
## Phase Boundary

Drei pre-existing Frontend-Bugs im `shifty-dioxus`-Workspace:

- **BUG-01**: `save_slot_edit` hält den `SLOT_EDIT_STORE`-Write-Borrow nicht mehr über `.await` (Panic-Risiko WR-02 v1.10). Alle Store-Reads vor `.await` in lokale Werte kopieren.
- **BUG-02**: `list_user_invitations`-Loader unterscheidet „leere Liste" von „Parse-Fehler"; Fehler-Zweig rendert sichtbare Fehlermeldung, kein silent-empty.
- **BUG-03**: Verbleibende Modals unter `component/` mit Backdrop-Close-Pfad → `BackdropPress` migriert (drag-safe, kein mouseup-außerhalb-Close). Grep-Verifikation.

Kein Snapshot-Bump, keine Migration, keine neuen Deps.

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion
Success Criteria sind hart und präzise (siehe ROADMAP.md Phase 44). Alle Umsetzungsdetails an Claude:

- **BUG-01**: `shifty-dioxus/src/page/shiftplan.rs` (o.ä.) `save_slot_edit`: alle nötigen Store-Felder VOR jedem `.await` in lokale `let`-Bindings, `SLOT_EDIT_STORE.write()` scope-schließen vor `.await`. Regressionstest: pure-fn oder Component-Test der die Sequenz (borrow → await → borrow) durchspielt und keine Panic wirft.
- **BUG-02**: `list_user_invitations`-Loader — `Result::Err(parse_err) => …` explizit rendern statt fallback auf `Ok(vec![])`. Component zeigt sichtbare Fehlermeldung.
- **BUG-03**: `grep -rn "BackdropPress\|onmousedown.*close" shifty-dioxus/src/component/` → identifiziere Modals, die noch nicht `BackdropPress` nutzen. Migration analog Phase 37 MOD-01.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `BackdropPress` (Phase 37 MOD-01) — zentrale drag-safe Backdrop-Close-Logik.
- Absence-/Convert-Modal → Phase-37-Vorlage für Migration.
- Store-Muster im `shifty-dioxus` — Signal-basierte globale Stores.

### Established Patterns
- Pure-fn Tests in `#[cfg(test)] mod tests` in der jeweiligen Datei.
- Component-Tests mit `dioxus_ssr` oder Signal-Direktzugriff.

### Integration Points
- **BUG-01**: `shifty-dioxus/src/page/shiftplan.rs` (`save_slot_edit`) + `SLOT_EDIT_STORE`.
- **BUG-02**: `shifty-dioxus/src/page/user_management.rs` oder `user_details.rs` (`list_user_invitations`-Loader).
- **BUG-03**: `shifty-dioxus/src/component/*modal*.rs`.

</code_context>

<specifics>
## Specific Ideas

- BUG-01 Regressionstest: minimal — testet, dass nach `save_slot_edit` die Signale konsistent sind, nicht die WASM-Runtime-Panic direkt (wäre nicht deterministisch triggerbar).
- BUG-03: Grep als Success-Kriterium; jedes verbleibende `onmousedown`-Close-Pattern das nicht `BackdropPress` ist wird migriert oder begründet (Kommentar).

</specifics>

<deferred>
## Deferred Ideas

Nichts — Scope-treu innerhalb Phase 44.

</deferred>
