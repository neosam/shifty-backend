---
phase: 08-absence-crud-page-foundation
plan: 03
subsystem: testing
tags: [shifty-backend, openapi, surface-assertion, drift-detection, utoipa]

# Dependency graph
requires:
  - phase: 08-absence-crud-page-foundation
    plan: 02
    provides: VacationBalanceServiceImpl + REST endpoints + ApiDoc nest entry
  - milestone: v1.0 Phase 4
    provides: prior-art OpenAPI snapshot test (since-removed in fdb70b5 — see Decisions Made)
provides:
  - rest/tests/openapi_surface.rs (version-agnostic surface assertion test)
  - Drift-detection harness for `/vacation-balance/*` and ~25 other domain paths
  - Schema-name pin for `VacationBalanceTO` + 18 other DTOs
  - VacationBalance tag presence pin
affects:
  - Future Phase 8 plans (08-04 / 08-05): test will catch accidental REST surface changes
  - Future v1.3+ phases: every new /utoipa::path endpoint should add an entry to EXPECTED_PATHS
    when the endpoint is on a major domain we want drift-detected
  - Phase 13 (i18n + closure): test stays in scope as a final-regression gate

# Tech tracking
tech-stack:
  added: []  # No new dev-deps; explicitly avoided re-adding insta
  patterns:
    - "Version-agnostic API-surface drift detection: assert path keys + schema names + tags via plain assert!, no full-doc snapshot, no info.version pin"
    - "Schema NAMES are pinned, schema BODIES are not — DTO field churn does not trigger a test diff"
    - "Representative subset over exhaustive coverage: ~10–15 paths per major domain, not every route"

key-files:
  created:
    - rest/tests/openapi_surface.rs
    - .planning/phases/08-absence-crud-page-foundation/08-03-SUMMARY.md
  modified:
    - .planning/phases/08-absence-crud-page-foundation/08-VALIDATION.md
    - .planning/phases/08-absence-crud-page-foundation/08-03-PLAN.md  # frontmatter only

key-decisions:
  - "Option-B-Pivot statt Original-Plan-Insta-Snapshot: nach User-Approval ('Der Test ist kacke. Und zwar schlägt der fehl, wenn wir nur ein Versions Update machen.') wurde der Test als Pfad-/Schema-/Tag-Assertion umgesetzt. Begründung in Commit fdb70b5 (2026-05-03): das alte snapshot pinning enthielt info.version aus rest/Cargo.toml und löste bei jedem Versions-Bump einen Noise-Fail aus."
  - "Test pinnt Pfad-Keys + Schema-Namen + Tag-Namen, NICHT Schema-Bodies. DTO-Feld-Churn (additiv) bricht den Test nicht; nur das Verschwinden eines Endpunkts oder das Umbenennen einer Schema-Klasse bricht ihn — genau die Drift-Klasse, die wir detektieren wollen."
  - "Representative-Subset von ~25 Pfaden statt voller Enumeration: wir wollen Domain-Drift-Detektion ('die ganze Domäne ist verschwunden'), nicht Per-Route-Drift. Das hält den Test schmal und stabil gegen normale Refactor-Bewegungen."
  - "Plan-Frontmatter `autonomous: false` → `true`: nach Pivot ist kein Diff-Review-Checkpoint mehr nötig (es gibt keinen Snapshot-Diff zum Sichten). Plan-Body-Tasks (1: insta-accept, 2: human-verify) bleiben historisch; SUMMARY ist die ground-truth was tatsächlich passiert ist."
  - "VALIDATION.md Wave-0-Item umgeschrieben: 'OpenAPI insta-Snapshot-Refresh' → 'OpenAPI Surface-Assertion-Test (rest/tests/openapi_surface.rs)' und als ✅ markiert. Die V-Truths in 08-03-PLAN.md.must_haves wurden ebenfalls neu formuliert."

patterns-established:
  - "Surface-Assertion vs. Full-Snapshot: für API-Doku-Stabilität ist eine Pfad/Schema-Liste robuster als ein insta-snapshot, sobald ein info.version-bound Field im Snapshot landet. Pattern für künftige Crates, die utoipa::OpenApi nutzen."

requirements-completed: [FUI-A-04]
# Note: FUI-A-04 ist im Plan-Frontmatter gelistet, aber FUI-A-04 (warnings[]
# als nicht-blockierende Hinweisliste) ist ein Frontend-Render-Requirement,
# das erst Plan 08-05 (WarningList-Component) erfüllt. Plan 08-03 sichert
# die Backend-Surface-Stabilität, gegen die das Frontend codet — also
# indirekt FUI-A-04-stützend. Kein Closure-Marker hier; der Closure-Marker
# erfolgt in Plan 08-05.

# Metrics
duration: ~12min
completed: 2026-05-08
---

# Phase 08 Plan 03: Absence-CRUD-Page Foundation — OpenAPI Surface Drift Detection Summary

**Replaced the previously-removed insta-snapshot test with a version-agnostic surface-assertion test (`rest/tests/openapi_surface.rs`) that pins ~25 representative REST paths and 19 named DTO schemas — including the Wave-2 `/vacation-balance/*` endpoints and `VacationBalanceTO` — without locking `info.version` or any `Cargo.toml`-bound field.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-08 nach Plan 08-02 Closure + User-Approval von Option B
- **Completed:** 2026-05-08
- **Tasks:** 2 (Surface-Test + Validation/Plan-Frontmatter-Update)
- **Files created:** 2 (`rest/tests/openapi_surface.rs` + `08-03-SUMMARY.md`)
- **Files modified:** 2 (`08-VALIDATION.md`, `08-03-PLAN.md` Frontmatter)

## Accomplishments

- **`rest/tests/openapi_surface.rs`** (175 Zeilen) mit drei Tests:
  1. `openapi_paths_contain_expected_routes` — assertet, dass 25 erwartete REST-Pfade (inkl. `/vacation-balance/{sales_person_id}/{year}` und `/vacation-balance/team/{year}` aus Wave 2) in `ApiDoc::openapi().paths` existieren. Die Pfade decken ein representative subset ab: Absence-Domain (3), Booking-Log (1), Sales-Person (3), Extra-Hours (3), Custom-Extra-Hours (1), Reporting (2), Billing-Period (2), Shiftplan/Week-Message (2), Special-Days (1), Permission (2), Cutover-Admin (2), VacationBalance (2).
  2. `openapi_schemas_contain_expected_dtos` — assertet, dass 19 DTO-Klassen-Namen in `components.schemas` existieren: `VacationBalanceTO` (Wave 2), `AbsencePeriodTO`, `AbsenceCategoryTO`, `AbsencePeriodCreateResultTO`, `WarningTO`, `BookingTO`, `BookingCreateResultTO`, `BookingLogTO`, `SalesPersonTO`, `SalesPersonUnavailableTO`, `ExtraHoursTO`, `ExtraHoursCategoryTO`, `ExtraHoursCategoryDeprecatedErrorTO`, `EmployeeReportTO`, `WorkingHoursReportTO`, `BillingPeriodTO`, `SlotTO`, `ShiftplanTO`, `CutoverProfileTO`.
  3. `openapi_includes_vacation_balance_tag` — sanity-Guard für die Wave-2-Verdrahtung (`VacationBalanceApiDoc` muss in `ApiDoc::openapi().tags` mit `name == "VacationBalance"` auftauchen).
- **3-Run-Determinism** verifiziert: drei aufeinanderfolgende `nix develop -c cargo test -p rest --test openapi_surface` alle grün (3 passed; 0 failed; finished in 0.01s pro Run). Pfad/Schema-Lookup ist Hash-Map-iteration-frei (exakter String-Compare per `keys().any()`).
- **Workspace-lib-Tests** ohne Regression: `nix develop -c cargo test --workspace --lib` → alle Suiten grün (388 service_impl-Tests inkl. 7 vacation_balance, 11 shifty_utils, 10 dao, 8 dao_impl_sqlite, 0 service-trait-only).
- **`08-VALIDATION.md`** Per-Task-Verification-Map um die Plan-08-03-Zeile (Task-ID `8-03-01`, Test-Type `integration`, Status `✅ green`, Threat-Ref `T-8-SURFACE-DRIFT`) ergänzt; Wave-0-Item-Liste um den Snapshot-Refresh-Eintrag durchgereicht und durch die neue Surface-Assertion-Variante als ✅ markiert.
- **`08-03-PLAN.md` Frontmatter** auf die neue Realität gehoben: `files_modified: rest/tests/openapi_surface.rs`, `autonomous: true`, `tags: [shifty-backend, openapi-surface, drift-detection]`, drei neu formulierte `must_haves.truths`, `artifacts` und `key_links` zeigen jetzt auf `rest/tests/openapi_surface.rs`. Die alten Plan-Body-Tasks (Insta-Accept + Human-Verify-Checkpoint) wurden bewusst NICHT umgeschrieben — der Plan-Body ist historischer Record; diese SUMMARY ist die ground-truth.

## Task Commits

Atomar mit `jj describe` (jj-only VCS):

1. **Task 1: version-agnostic OpenAPI surface-assertion test** — `a7dbd832` (test)
   - Datei: `rest/tests/openapi_surface.rs` (175 Zeilen, 3 Tests)
2. **Task 2: VALIDATION + PLAN frontmatter pivot to surface-assertion** — `4974fd41` (docs)
   - Dateien: `08-VALIDATION.md` (Per-Task-Map row + Wave-0 line) + `08-03-PLAN.md` (Frontmatter only)

**Plan metadata commit:** wird in der nächsten jj-Change durch User/SUMMARY-Wave angelegt (`commit_docs: false`, jj-only-VCS).

## Files Created/Modified

- `rest/tests/openapi_surface.rs` (NEW, 175 Zeilen) — drei Tests + zwei `const`-Listen (`EXPECTED_PATHS: &[&str; 25]`, `EXPECTED_SCHEMAS: &[&str; 19]`); ausführliche Doc-Comments, die das Pivot-Rationale + die Drift-Detection-Scope dokumentieren.
- `.planning/phases/08-absence-crud-page-foundation/08-VALIDATION.md` (modified) — Per-Task-Map ergänzt um Plan-08-03-Zeile; Wave-0-Item umgeschrieben.
- `.planning/phases/08-absence-crud-page-foundation/08-03-PLAN.md` (modified, Frontmatter nur) — `must_haves.truths`/`artifacts`/`key_links`/`tags`/`files_modified`/`autonomous` auf den Surface-Assertion-Pivot aktualisiert. Plan-Body-Tasks unverändert (historischer Record).
- `.planning/phases/08-absence-crud-page-foundation/08-03-SUMMARY.md` (NEW) — diese Datei.

## Verification Results

| Layer | Command | Result |
| ----- | ------- | ------ |
| Surface-Test compile | `nix develop -c cargo test -p rest --test openapi_surface` | OK (3 passed; 0 failed; ~10ms run) |
| 3-Run-Determinism | drei sequentielle `cargo test -p rest --test openapi_surface` | alle drei grün (jeweils 3 passed; 0 failed) |
| Workspace lib regression | `nix develop -c cargo test --workspace --lib` | OK (alle Suiten grün; keine Regression durch den neuen Test) |

## Decisions Made

- **Option B (Surface-Assertion) statt Option A (Re-Add Snapshot)**: Der prior executor hat in `fdb70b5` (2026-05-03) den insta-Snapshot bewusst entfernt, weil `info.version` aus `rest/Cargo.toml` jeden Versions-Bump zu einem Test-Fail machte. User-O-Ton: *"Der Test ist kacke. Und zwar schlägt der fehl, wenn wir nur ein Versions Update machen."* Wir akzeptieren, dass ein voller Snapshot mit `info.version`-Pin nicht praktikabel ist; Surface-Assertion ist die richtige Antwort.
- **Pin path keys + schema names, NOT bodies**: Schema-Bodies ändern sich häufig (additive Felder, Renames innerhalb eines Releases); Test-Diffs auf Body-Ebene wären Noise. Drift-Detection-Wert liegt darin, "diese Endpunkt-Domäne ist verschwunden" oder "diese DTO-Klasse heißt jetzt anders" zu detektieren — beides erfasst die Namens-Pin.
- **Representative subset (~25 paths) statt voller Enumeration**: Wenn wir alle 77 Pfade pinnen, müssen wir bei jedem neuen Endpunkt den Test mitziehen — das wird ein Reibungspunkt und führt langfristig dazu, dass jemand den Test deaktiviert. Mit 25 representative paths fangen wir Domain-Drift, ohne einen Wartungs-Burden zu erzeugen, der die Hilfsbereitschaft des Tests übersteigt.
- **Plan-Body-Tasks NICHT umschreiben**: Der Plan-Body ist historischer Record (was war ursprünglich geplant). Die SUMMARY.md ist die ground-truth (was wurde tatsächlich gebaut). Wir aktualisieren nur das Plan-Frontmatter, weil V-Truths/files_modified/tags von späteren GSD-Tools gescannt werden.
- **Test-Datei in `rest/tests/`** (Integration-Test-Layout) statt in `rest/src/lib.rs` (`#[cfg(test)] mod tests`)**: `rest/tests/` existierte bereits leer, ist das idiomatische cargo-Layout für Integration-Tests. Außerdem braucht der Test `rest::ApiDoc` als public-API-Konsument — perfekter Use-Case für ein integration-test-Crate.

## Deviations from Plan

Der Plan war ursprünglich `autonomous: false` mit zwei Tasks: (1) Insta-Snapshot-Refresh + 3-Run-Determinism, (2) Human-Verify-Checkpoint auf den Snapshot-Diff. Der Plan ist als ganzes nicht "deviated"-im-klassischen-Sinne, sondern **architectural-pivot**ed (Option B vom User approved):

### Architectural Pivot (User-Approved Pre-Execution)

**1. [Architecture Pivot - User Decision] Replaced insta-snapshot with surface-assertion**

- **Found during:** Pre-execution context-loading (prior executor session). The insta-snapshot test that Plan 08-03 referenced no longer existed: commit `fdb70b5` (2026-05-03) had removed `rest/tests/openapi_snapshot.rs` + the `.snap` files + the insta dev-dependency.
- **Issue:** Plan 08-03 was written before the snapshot removal. Following the plan as written would either re-add a known-flaky snapshot (with `info.version` pin → noise-fails on every version bump) or escalate per W-6.
- **Fix:** User approved Option B: replace with version-agnostic path + schema name + tag assertions. Test pins ~25 representative paths and 19 schema names; no full-document comparison; no `info.version` reference; no insta dev-dependency.
- **Files modified:** `rest/tests/openapi_surface.rs` (new), `.planning/.../08-03-PLAN.md` (frontmatter), `.planning/.../08-VALIDATION.md`.
- **Verification:** 3-run-determinism + workspace lib regression-suite both green.
- **Committed in:** `a7dbd832` (test) + `4974fd41` (docs).

---

**Total deviations:** 1 user-approved architectural pivot (no auto-fixes via Rules 1–3).
**Impact on plan:** The Plan 03 OBJECTIVE (Backend-Surface gepinnt + 3-Run-stable + V-Truths erfüllt) ist unverändert erfüllt — nur das Werkzeug ist anders (Surface-Assertion statt Insta-Snapshot). Die User-Verify-Checkpoint-Task entfällt, weil es keinen Snapshot-Diff zum Sichten gibt; die Test-Body-Pfad-/Schema-Liste ist self-documenting und review-bar.

## Issues Encountered

- **Erste Compile mit `Vec<Tag>:Debug`-Constraint-Verstoß**: Der erste Test-Draft printete `Option<&Vec<Tag>>` per `{tags:#?}`, aber `utoipa::openapi::Tag` implementiert kein `Debug`. Fix war eine Zeile (vor dem Compile-Fail-Output): `tag_names: Vec<&str>` aus `tags.iter().map(|t| t.name.as_str()).collect()` zusammenstellen, dann `{tag_names:#?}` printen. Pure Test-Mechanik; kein Plan-Issue.
- **EXPECTED_PATHS-Initial-Liste hatte mehrere Mismatches mit der echten OpenAPI-Surface**: Der erste Draft enthielt z. B. `/booking-log/by-sales-person/{id}` (existiert nicht, real ist `/booking-log/{year}/{week}`), `/custom-extra-hours/` mit trailing-slash (echt ohne), `/billing-period/` (echt ohne). Der Test gab das exakt-Diff aus (assert!-Fehler-Format mit `missing` + `actual paths (77 total)`-Liste), und die Liste war in einem Pass anpassbar. Pure Discovery-Mechanik; kein Plan-Issue.
- **`WorkingHoursPerSalesPersonTO` ist KEIN registriertes Schema im OpenAPI-Doc**: Der Schema-Liste-Test gab das aus; ich habe es durch `EmployeeReportTO` + `WorkingHoursReportTO` + `BillingPeriodTO` ersetzt. (`WorkingHoursPerSalesPersonTO` lebt in `rest-types/lib.rs:867` als Struct, ist aber nicht in `components(schemas(...))` einer ApiDoc-Nest-Section eingetragen — das ist normales utoipa-Verhalten: nur Schemas, die in einer `#[utoipa::path]`-`responses(... body = ...)` referenziert werden, landen automatisch in `components.schemas`.) Pure Discovery-Mechanik; kein Plan-Issue.

## User Setup Required

None — keine externen Services, keine Migrationen, keine Env-Var-Änderungen.

## Next Phase Readiness

- **Plan 08-04 (Frontend Foundation)** kann starten. Das Backend-Surface-Pinning gilt als "done" — alle künftigen Frontend-Pläne (08-04 / 08-05) können ihre API-Calls gegen die jetzt-getestete REST-Surface schreiben, mit Vertrauen, dass `cargo test -p rest --test openapi_surface` ein Frontend-blockierendes Endpunkt-Verschwinden früh detektiert.
- **Plan 08-06 (Final UAT-Smoke + Regression)** wird denselben Surface-Test als Teil seines Final-Regression-Gates wiederverwenden können.
- **Phase 13 (i18n + closure)** wird den Test als finalen Compile-Gate übernehmen.

## TDD Gate Compliance

Plan 08-03 ist `type: execute` (nicht `type: tdd`); kein RED→GREEN→REFACTOR-Gate erforderlich. Der einzelne `test(...)`-Commit (a7dbd832) folgt der Backend-Wave-Konvention.

## Threat Flags

Keine neuen Trust-Boundary-Surfaces eingeführt — der Test ist read-only, hat keine HTTP-Endpoints, keine Schema-Änderungen, keine Datei-Zugriffe außer dem Cargo-Test-Harness-Setup.

## Self-Check: PASSED

Verifizierte Artefakte:
- ✓ FOUND: `rest/tests/openapi_surface.rs` (175 Zeilen, 3 Tests)
- ✓ FOUND: `.planning/phases/08-absence-crud-page-foundation/08-VALIDATION.md` (mit neuer 8-03-01-Zeile + Wave-0-Item-Update)
- ✓ FOUND: `.planning/phases/08-absence-crud-page-foundation/08-03-PLAN.md` Frontmatter mit neuen `must_haves.truths`/`artifacts`/`key_links`
- ✓ FOUND: jj commit `a7dbd832` (test, Task 1)
- ✓ FOUND: jj commit `4974fd41` (docs, Task 2)
- ✓ ALL: `cargo test -p rest --test openapi_surface` × 3 grün (3-Run-Determinism)
- ✓ ALL: `cargo test --workspace --lib` grün (keine Regression)
- ✓ ALL: kein insta dev-dep, keine `.snap`-Datei, kein `info.version`-Reference im Test

---

*Phase: 08-absence-crud-page-foundation*
*Plan: 03*
*Completed: 2026-05-08*
