# Phase 45: shifty-dioxus Warnings-Aufräumen (FE) - Context

**Gathered:** 2026-07-02
**Status:** Ready for planning
**Mode:** Autonomous — reine Hygiene, keine Design-Fragen. Mechanische Lint-Behebung.

<domain>
## Phase Boundary

Der `shifty-dioxus`-Workspace kompiliert warnungsfrei mit `cargo build` UND `cargo clippy -- -D warnings`. Fortsetzung HYG-01/02 (v1.11) auf die verbliebenen ~198 pre-existing Lints. Backend bleibt unbeeinträchtigt.

Kein Snapshot-Bump, keine Migration, keine neuen Deps, keine Logik-Änderung — nur Lint-Behebung.

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion
- **Lint-Katalogisierung**: `cargo clippy -p shifty-dioxus --workspace --tests 2>&1 | grep -E "warning|error" | sort -u` → Kategorien identifizieren (`unused_imports`, `dead_code`, `clippy::needless_borrow`, `clippy::redundant_clone`, `clippy::let_and_return`, `clippy::unused_async`, `clippy::result_large_err`, `deprecated`, etc.).
- **Batch-Fixes**:
  - `cargo fix -p shifty-dioxus --allow-dirty` für auto-fixable Lints als erste Welle.
  - Manuelle Fixes für Rest (nach Kategorie gebündelt, kleine Commits).
- **Legitime `#[allow(…)]`-Kandidaten**: Lint-Regel, die durch Dioxus-RSX-Muster falsch getriggert wird → mit `#[allow(clippy::…)]` + Kommentar `// Dioxus-RSX-Trigger, false positive` (o.ä.). Kein blindes `#![allow(warnings)]`.
- **Bekannte Ausnahmen behalten**:
  - `#[allow(clippy::result_large_err)]` in einem Wrapper falls das Result-Größenproblem bleibt.
  - `dead_code`-Warnings auf Fixtures/Test-Helpers → `#[allow(dead_code)]` mit kurzem Kommentar.
- **Toolchain**: aus **Backend-Root-Shell** (`nix develop`), NICHT aus `shifty-dioxus/`-Shell (die Clippy-Toolchain dort ist kaputt — Memory: „Dioxus Clippy nicht gated + Toolchain-Split").
- **Gate-Reihenfolge**:
  1. `cargo build -p shifty-dioxus` — 0 Warnings.
  2. `cargo clippy -p shifty-dioxus --workspace --tests -- -D warnings` — grün.
  3. `cargo build --target wasm32-unknown-unknown` (im `shifty-dioxus/`-Dir) — grün, keine WASM-Regression.
  4. Backend `cargo clippy --workspace -- -D warnings` (aus Backend-Root) — bleibt grün.
  5. Backend `cargo test --workspace` — bleibt grün.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- HYG-01/02 (v1.11 Phase 38) — Prä-existierte Baseline (Backend-Clippy grün, FE-Build warnungsfrei). Phase 45 nimmt den nächsten Schritt (FE-Clippy).
- Prior Muster: kleine Commits pro Lint-Kategorie in Phase 38.

### Established Patterns
- `#[allow(clippy::…)]`-Kommentare pro Case begründet.
- `cargo fix --allow-dirty` als Startpunkt für mechanische Lints.

### Integration Points
- Reines Refactoring innerhalb `shifty-dioxus/src/**/*.rs`. Keine externen Files.

</code_context>

<specifics>
## Specific Ideas

- Wenn eine Lint-Klasse ausufernd viele Callsites betrifft (>50), das per `#[allow]` im Modul-Root deaktivieren mit Begründung, statt jedem Callsite ein `#[allow]` zu geben — pragmatisch.
- Wenn ein Fix ein Logik-Verhalten ändern würde (kein rein syntaktischer Fix), das per Kommentar dokumentieren und im Commit-Message vermerken.
- Nach jeder Kategorie ein separater Commit — leichter zu bisecten falls WASM-Build regressiert.

</specifics>

<deferred>
## Deferred Ideas

Nichts — Scope-treu innerhalb Phase 45.

</deferred>
