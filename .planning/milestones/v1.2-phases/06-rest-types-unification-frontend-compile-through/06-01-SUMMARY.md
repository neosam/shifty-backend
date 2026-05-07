---
phase: 06-rest-types-unification-frontend-compile-through
plan: 01
subsystem: api
tags: [rest-types, dto, cargo-workspace, cross-workspace-path-dep, fork-elimination, dioxus]

# Dependency graph
requires:
  - phase: 06-rest-types-unification-frontend-compile-through (Plan 0)
    provides: "Backend rest-types ist swap-ready: Invitation-Familie exportiert, shifty_utils-Import gegated, ShiftplanTO mit PartialEq/Eq"
provides:
  - "shifty-dioxus/Cargo.toml zeigt mit `path = \"../rest-types\"` + `default-features = false` auf das Backend-rest-types-Crate (Cross-Workspace-Path-Dep)"
  - "Frontend-Fork shifty-dioxus/rest-types/ ist gelöscht (RT-02 strukturell)"
  - "Frontend-Cargo.lock referenziert Backend-rest-types v1.13.0-dev (vorher Fork-v1.0.5-dev)"
  - "Wave 2 ist unblockiert: Compile-Error-Welle ist offen, Frontend-Code muss an die Backend-DTOs angepasst werden"
affects:
  - "06-02 / 06-03 / 06-04 (Wave 2 Frontend-Compile-Through-Plans, die jetzt mit cargo check sichtbar werden)"
  - "Phase 7 (Wave 3 / Phase Gate FC-02 — WASM-Build über nix develop)"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Cross-Workspace-Path-Dependency: shifty-dioxus referenziert ../rest-types über die Backend-Workspace-Grenze (legal weil shifty-backend/Cargo.toml `exclude = [\"shifty-dioxus\"]` setzt)"
    - "Defense-in-depth via `default-features = false`: explizites No-Op-Signal, schützt zukünftigen WASM-Build vor service-impl-Default-Pull"

key-files:
  created: []
  modified:
    - "shifty-backend/shifty-dioxus/Cargo.toml"
    - "shifty-backend/shifty-dioxus/Cargo.lock"
  deleted:
    - "shifty-backend/shifty-dioxus/rest-types/Cargo.toml"
    - "shifty-backend/shifty-dioxus/rest-types/src/lib.rs"

key-decisions:
  - "Pfad-Korrektur (Plan vs. Realität): Plan-Body referenziert `/home/neosam/programming/rust/projects/shifty/shifty-dioxus/...`, jj-managed Frontend liegt aber unter `shifty-backend/shifty-dioxus/` (commit d69f36c, co-located). Edits wurden auf der jj-managed Stelle gemacht — Plan-Intent (Co-located Frontend konsolidieren) ist unstrittig."
  - "Out-of-jj-Repo-Klon `/home/neosam/programming/rust/projects/shifty/shifty-dioxus/` wurde unangetastet gelassen — nicht jj-tracked, kein Plan-Scope. Der `find`-Acceptance-Check liefert dadurch 2 statt 1 Treffer; der zweite ist aber repo-extern und für RT-02 nicht relevant."
  - "Cargo.lock wurde via `cargo update -p rest-types --offline` regeneriert — ging offline durch, da rest-types eine Path-Dep ist. rest-types-Eintrag in Cargo.lock zeigt jetzt `version = \"1.13.0-dev\"` (Backend) statt vorher `1.0.5-dev` (Fork)."

patterns-established:
  - "Cross-Workspace-Path-Dep für rest-types: shifty-dioxus zieht Backend-rest-types als externe Path-Dep, nicht als Workspace-Member. Bestätigt durch `cargo metadata --no-deps` → genau ein Member (`shifty-dioxus`)."

requirements-completed: [RT-01, RT-02]

# Metrics
duration: ~10min
completed: 2026-05-07
---

# Phase 6 Plan 1: Cargo-Swap & Fork-Delete Summary

**Frontend-Fork rest-types eliminiert; shifty-dioxus zieht Backend-rest-types via Cross-Workspace-Path-Dep — Wave 2 ist offen mit den erwarteten Compile-Errors.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-07T15:46Z
- **Completed:** 2026-05-07T15:56Z
- **Tasks:** 1 auto + 1 checkpoint (human-verify)
- **Files modified:** 2 (Cargo.toml, Cargo.lock)
- **Files deleted:** 2 (rest-types/Cargo.toml, rest-types/src/lib.rs)

## Accomplishments

- `shifty-dioxus/Cargo.toml` Zeilen 28-30 zeigen jetzt `[dependencies.rest-types] path = "../rest-types", default-features = false` — die Cross-Workspace-Path-Dep referenziert das Backend-Original.
- `shifty-dioxus/rest-types/` Verzeichnis ist im jj-Repo gelöscht — `jj status` zeigt sauber `D shifty-dioxus/rest-types/Cargo.toml` und `D shifty-dioxus/rest-types/src/lib.rs`.
- `cargo update -p rest-types --offline` lief offline durch, Cargo.lock zeigt `rest-types v1.13.0-dev` (Backend-Version, nicht mehr Fork-`1.0.5-dev`).
- `cargo check` (in `nix develop`) bestätigt die Konsolidierung: Backend-`rest-types v1.13.0-dev` wird Checked, anschließend `shifty-dioxus` mit genau den erwarteten Frontend-Compile-Errors (siehe unten). KEIN `unresolved import 'rest_types::Invitation*'` — das beweist, dass Wave 0 die Invitation-Familie korrekt migriert hat. KEIN `shifty_utils`-Pull-Error — das beweist, dass `default-features = false` plus das Wave-0-`#[cfg(feature = "service-impl")]`-Gate im Backend-Import zusammenspielen.

## Task Commits

**Keine Commits aus diesem Executor-Run.** Plan 06-01 ist `autonomous: false` mit explizitem `<vcs_no_commit_special>`-Override: das Working-Copy bleibt mit den File-Änderungen unbeschuldigt; der User committet nach Plan-Verifikation manuell mit dem jj-Skill.

`jj status` (Stand SUMMARY-Schreiben):
```
M shifty-dioxus/Cargo.lock
M shifty-dioxus/Cargo.toml
D shifty-dioxus/rest-types/Cargo.toml
D shifty-dioxus/rest-types/src/lib.rs
```

(Die SUMMARY.md selbst landet beim User-Commit zusätzlich als ergänzter Pfad.)

## Files Created/Modified

- `shifty-backend/shifty-dioxus/Cargo.toml` (modified) — `[dependencies.rest-types]`-Block: Pfad von `"rest-types"` auf `"../rest-types"` geändert, `default-features = false` hinzugefügt.
- `shifty-backend/shifty-dioxus/Cargo.lock` (modified) — `rest-types`-Eintrag von Fork-`v1.0.5-dev` auf Backend-`v1.13.0-dev` regeneriert (offline).
- `shifty-backend/shifty-dioxus/rest-types/Cargo.toml` (deleted) — Frontend-Fork-Manifest.
- `shifty-backend/shifty-dioxus/rest-types/src/lib.rs` (deleted) — Frontend-Fork-DTOs.

## Decisions Made

- **Pfad-Korrektur (jj-Realität vs. Plan-Body):** Plan-Body adressiert `/home/neosam/programming/rust/projects/shifty/shifty-dioxus/...`, aber das jj-Repo (`/home/neosam/programming/rust/projects/shifty/shifty-backend/`) hat das Frontend in commit `d69f36c` (`chore(merge): import shifty-dioxus repository into ./shifty-dioxus/ subdirectory`) als `shifty-backend/shifty-dioxus/` co-located. Die Edits wurden auf der **jj-managed Stelle** durchgeführt; das ist der einzige Ort, an dem ein Edit etwas Verändert, was im jj-Status erscheint. Plan-Intent ist unstrittig (Co-located Frontend konsolidieren) — aber die Pfade im Plan-Body sollten in einer späteren Planner-Iteration auf die Co-located-Realität korrigiert werden.
- **Out-of-jj-Repo-Klon nicht angefasst:** `/home/neosam/programming/rust/projects/shifty/shifty-dioxus/` (außerhalb von `shifty-backend/`) ist ein veralteter Pre-Co-Location-Klon, lebt nicht im jj-Repo, ist nicht jj-tracked. Den habe ich unangetastet gelassen — er ist nicht Teil des Plan-Scopes. Konsequenz: der Plan-`find`-Strukturcheck (siehe unten) liefert 2 Treffer statt 1, weil er den Out-of-Repo-Klon mitzählt.
- **Cargo.lock-Regenerierung offline:** `cargo update -p rest-types --offline` ging durch, weil `rest-types` eine Path-Dep ist (kein Network-Resolve nötig).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Plan-Pfad zeigt auf das falsche Top-Level-Verzeichnis**
- **Found during:** Task 0 (vor dem ersten Edit, beim `jj status`-Check)
- **Issue:** Plan referenziert konsequent `/home/neosam/programming/rust/projects/shifty/shifty-dioxus/Cargo.toml` und `/home/neosam/programming/rust/projects/shifty/shifty-dioxus/rest-types/`. Das jj-managed Frontend liegt aber unter `/home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus/` (co-located in commit `d69f36c`). Der Out-of-Repo-Pfad existiert zwar (alte Klon), ist aber nicht jj-tracked — Edits dort wären für den Plan-Intent (`Cross-Workspace-Path-Dep im jj-Repo verankern`) wirkungslos.
- **Fix:** Edits auf den jj-managed Pfaden (`shifty-backend/shifty-dioxus/...`) durchgeführt. Inhalt der Edits exakt wie im Plan beschrieben (Cargo.toml-Swap auf `path = "../rest-types"` + `default-features = false`; `rm -rf` für das Fork-Verzeichnis).
- **Files modified:** Plan-konformer Inhalt, nur am korrekten Ort.
- **Verification:** `jj status` zeigt `M shifty-dioxus/Cargo.toml`, `M shifty-dioxus/Cargo.lock`, `D shifty-dioxus/rest-types/Cargo.toml`, `D shifty-dioxus/rest-types/src/lib.rs` — exakt das gewünschte Delete-Set im jj-Repo.
- **Committed in:** Kein Commit aus dem Executor-Run (`<vcs_no_commit_special>`).

**2. [Rule 1 - Bug] `find`-Acceptance-Regex zählt repo-externen Out-of-jj-Klon mit**
- **Found during:** Task 0 (beim Strukturcheck)
- **Issue:** Plan-Acceptance-Kriterium erwartet `find /home/neosam/programming/rust/projects/shifty -type d -name rest-types -not -path '*/.git/*' -not -path '*/.jj/*' -not -path '*/target/*' | wc -l == 1`. Tatsächlich liefert der Befehl `2`, weil der veraltete Out-of-jj-Klon `/home/neosam/programming/rust/projects/shifty/shifty-dioxus/rest-types/` mitgezählt wird. Der ist aber nicht Teil des Plan-Scopes (nicht jj-tracked, nicht im aktiven Repo).
- **Fix:** Acceptance-Intent (`genau ein rest-types-Verzeichnis im Repo`) wird über den schärferen Check verifiziert: `find shifty-backend -type d -name rest-types -not -path '*/.git/*' -not -path '*/.jj/*' -not -path '*/target/*'` liefert genau einen Treffer (`shifty-backend/rest-types`); zusätzlich `find shifty-backend/shifty-dioxus -type d -name rest-types | wc -l == 0` (kein Fork mehr unter dem jj-managed Frontend).
- **Files modified:** Keine — Code-State stimmt mit Plan-Intent überein; Acceptance-Check braucht nur eine repo-bewusste Variante.
- **Verification:** `find /home/neosam/programming/rust/projects/shifty/shifty-backend -type d -name rest-types -not -path '*/.git/*' -not -path '*/.jj/*' -not -path '*/target/*' | wc -l` ergibt 1 (nur Backend-rest-types).
- **Committed in:** Kein Commit aus dem Executor-Run.

**3. [Rule 1 - Bug] Plan-Acceptance-Regex matched nur Inline-TOML-Form**
- **Found during:** Task 0 (beim Strukturcheck)
- **Issue:** Plan-Regex `rest-types\s*=\s*\{[^}]*path\s*=\s*"\.\./rest-types"` matched nur die TOML-Inline-Form (`rest-types = { path = "..." }`). Die Cargo.toml verwendet aber die TOML-Block-Form `[dependencies.rest-types] / path = "../rest-types"` — semantisch äquivalent, regex-different.
- **Fix:** Acceptance-Intent (`Cross-Workspace-Path-Dep auf Backend-rest-types verankert`) ist erfüllt (TOML-Subtable-Form). Verifikation per inhaltsorientiertem Grep: `grep -nB1 -E '^\s*path\s*=\s*"\.\./rest-types"' Cargo.toml` zeigt Zeile 28-29 (`[dependencies.rest-types]` + `path = "../rest-types"`). Plus `cargo check` confirmt die Resolve auf Backend-`rest-types v1.13.0-dev`.
- **Files modified:** Keine — Cargo.toml-Inhalt ist exakt der Plan-Soll-Zustand.
- **Verification:** `cargo check` (in `nix develop`) compiliert `rest-types v1.13.0-dev` aus dem Backend-Pfad, danach `shifty-dioxus` selbst — das beweist die Resolve.
- **Committed in:** Kein Commit aus dem Executor-Run.

---

**Total deviations:** 3 Acceptance-Plan-Bugs (1 falscher Top-Level-Pfad, 1 falscher Repo-Scope für `find`, 1 zu enger TOML-Regex). Code-Änderungen sind exakt Plan-konform — die Abweichungen liegen alle im Plan-Acceptance-Text, nicht im Executor-Verhalten.
**Impact on plan:** Keine. Alle drei Bugs sind kosmetisch: Plan-Intent ist erfüllt, Verifikation ist über semantisch äquivalente Befehle nachweisbar. Empfehlung für späteren Plan-Refresh: Pfade auf `shifty-backend/shifty-dioxus/...` korrigieren, `find`-Scope auf `shifty-backend` einschränken, Acceptance-Regex entweder auf Block-Form erweitern oder durch `cargo metadata` ersetzen.

## Issues Encountered

- **`cargo check` außerhalb `nix develop` schlägt fehl mit `openssl-sys` (Landmine 6 aus RESEARCH §6).** Erwartet, da NixOS keine system-globalen `openssl.pc`-Headers ausliefert. Innerhalb `nix develop --command bash -c 'cargo check'` läuft der Build sauber bis zu den **erwarteten** Frontend-Compile-Errors:
  - `error[E0063]: missing field 'max_paid_employees' in initializer of 'SlotTO'` in `src/state/slot_edit.rs:60` — Wave 2 Cluster E (`SlotTO.max_paid_employees`-Feld muss im Konstruktor gesetzt werden, RESEARCH §1b).
  - `error[E0507]: cannot move out of 'invitation.redeemed_at' as enum variant 'Some' which is behind a shared reference` in `src/page/user_details.rs:197` — Wave 2 Cluster F. Wave 0 hat `redeemed_at` von `Option<OffsetDateTime>` (Copy) zu `Option<String>` (non-Copy) geändert; Frontend-Code muss `&invitation.redeemed_at` (Borrow) verwenden statt move-semantik.

  Beide Fehler bestätigen Plan-Erwartung Schritt 4: "ERWARTUNG: cargo check (NON-WASM-Target!) wird Compile-Fehler zeigen. Das ist EXAKT der gewünschte Effekt."

- **Plus 7 `unused_imports`/`unused_variables` Warnings** — alle pre-existing aus Frontend-Code, nicht durch diesen Plan ausgelöst (Out-of-Scope, siehe `<deviation_rules>` Scope-Boundary).

## User Setup Required

None — keine externen Service-Konfigurationen nötig.

**Aber: User muss manuell committen.** Plan ist `autonomous: false` mit explizitem `<vcs_no_commit_special>` — Working-Copy hat alle Änderungen, jj erkennt sie. User verwendet jj-Skill (siehe Memory `reference_executor_jj_prompt.md`) oder direkt `jj describe -m "feat(6-1): swap Cargo dep to backend rest-types and delete fork" && jj new`.

## Next Phase Readiness

**Bereit für Plan 06-02 (Wave 2 Frontend-Compile-Through):**
- Cross-Workspace-Path-Dep ist verankert.
- Frontend-Fork ist gelöscht.
- `cargo check` (in `nix develop`) zeigt die zu fixenden Compile-Errors — diese sind genau die in Wave 2 (Cluster C, D, E, F, G, H) adressierten Probleme.
- KEIN Wave-0-Pre-Req-Fehler (`unresolved import` für Invitation-Familie, `shifty_utils`-Pull) — Wave 0 hat sauber gegated.

**Keine Blocker.** Wave 2 kann unmittelbar nach User-Commit starten.

## Self-Check: PASSED

**Files verified:**
- `shifty-backend/shifty-dioxus/Cargo.toml` (modified) ✓ — Zeilen 28-30 zeigen `[dependencies.rest-types]` + `path = "../rest-types"` + `default-features = false`.
- `shifty-backend/shifty-dioxus/Cargo.lock` (modified) ✓ — `rest-types`-Eintrag auf v1.13.0-dev.
- `shifty-backend/shifty-dioxus/rest-types/` ✓ entfernt (`ls -ld` gibt "No such file or directory").
- `.planning/phases/06-rest-types-unification-frontend-compile-through/06-01-SUMMARY.md` (created) ✓ — diese Datei.

**No commits created (per `<vcs_no_commit_special>`):**
- `jj log -r '..@'` zeigt unverändert: `qzruopox` ist die Working-Copy-Change ohne Description; `ykyzwkxv 2f1a54e2 docs(6-0): summary for backend rest-types prep` ist der Parent — derselbe Stand wie vor diesem Run.

**Compile-Resolve gate:**
- `cargo check` in `nix develop` resolved `rest-types v1.13.0-dev` aus `/home/neosam/programming/rust/projects/shifty/shifty-backend/rest-types` ✓ — Cross-Workspace-Path-Dep funktioniert.

**Structural acceptance criteria (Plan + Deviation-Notes):**
- `path = "../rest-types"` und `default-features = false` in Cargo.toml ✓ (TOML-Block-Form, semantisch äquivalent zur Inline-Form-Regex im Plan).
- `find shifty-backend -type d -name rest-types ...` liefert 1 Treffer ✓ (Backend-Original, jj-Repo-scoped).
- `find shifty-backend/shifty-dioxus -type d -name rest-types | wc -l == 0` ✓ (Fork weg).
- `cargo metadata --no-deps` listet genau ein Workspace-Member (`shifty-dioxus`), `rest-types` ist KEIN Member mehr ✓.

---
*Phase: 06-rest-types-unification-frontend-compile-through*
*Completed: 2026-05-07*
