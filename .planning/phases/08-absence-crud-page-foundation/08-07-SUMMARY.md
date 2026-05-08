---
phase: 08-absence-crud-page-foundation
plan: 07
subsystem: ui-backend-bridge
tags: [gap-closure, sqlx-migration, sqlite-trigger, rest-feature-flag, frontend-feature-flag, top-bar-hierarchy, responsive, wasm]
gap_closure: true

# Dependency graph
requires:
  - phase: 08-absence-crud-page-foundation
    provides: |
      Plan 08-04 (frontend foundation: api/state/loader/coroutine/i18n/proxy);
      Plan 08-05 (AbsencesPage + AbsenceModal + 9 inline components +
      Routing + TopBar entry); Plan 08-02 + 08-03 (REST endpoints +
      OpenAPI surface drift detection); Plan 06-00..06-04 (rest-types
      cross-workspace path-dep with default-features = false).

provides:
  - "sqlx migration 20260508120000_admin-auto-grant-privilege.sql — INSERT-OR-IGNORE backfill + AFTER-INSERT trigger on `privilege` table; admin role henceforth holds every privilege."
  - "FeatureFlagTO in rest-types (feature-gated PartialEq + From<&FeatureFlag> + unknown(key) constructor)."
  - "GET /feature-flag/{key} REST endpoint (utoipa-typed, fail-safe `enabled: false` for unknown keys, auth-only readable)."
  - "RestStateDef::FeatureFlagService + getter; pub mod feature_flag in rest crate so integration tests can reach generate_route via tower::oneshot."
  - "Frontend FeatureFlag state-mirror + FeatureFlagsState aggregate (currently scoped to absence_range_source_active; defaults to None per flag for explicit-known-true matching in UI)."
  - "Frontend feature_flag_service coroutine + FEATURE_FLAGS_STORE GlobalSignal; loads `absence_range_source_active` once on app start."
  - "TopBar: nav_visibility extended with cutover_active: bool; partition_nav_items_with_context for HR-promote-to-admin-group; FEATURE_FLAGS_STORE-read in TopBarRouted."
  - "AbsencesPage responsive layout (md/lg breakpoints across StatsGrid, VacationEntitlementCard, VacationPerPersonList, AbsenceFilterBar, AbsenceList header + row)."

affects: [08-VALIDATION smoke gates, future privilege migrations, FE cutover-gated UX]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "SQLite AFTER-INSERT trigger pattern with INSERT OR IGNORE in trigger body — idempotent against parallel manual role_privilege inserts in the same transaction."
    - "Feature-gated DTO mirror with optional service-impl-only From-impl + unknown(key) fail-safe constructor (analog VacationBalanceTO Plan 08-01)."
    - "Frontend FeatureFlagsState with Option<bool>-per-flag — None = unknown/loading, Some(true) = explicitly active. UI checks must match Some(true), not unwrap_or(false), for fail-safe rendering."
    - "Static-classification + context-overlay helper: is_admin_target(target) stays user-agnostic; is_admin_target_with_context(target, has_hr) lifts Absences into the admin-group only for HR users without breaking the static API."
    - "Mobile-first stack-then-grid responsive pattern: `flex flex-col gap-2 md:grid md:grid-cols-... md:gap-...` — single class set per row, no mobile-only Tailwind plugins."

key-files:
  created:
    - "migrations/sqlite/20260508120000_admin-auto-grant-privilege.sql"
    - "rest/src/feature_flag.rs"
    - "shifty-dioxus/src/state/feature_flag.rs"
    - "shifty-dioxus/src/service/feature_flag.rs"
    - "shifty_bin/src/integration_test/admin_auto_grant.rs"
    - "shifty_bin/src/integration_test/feature_flag.rs"
    - ".planning/phases/08-absence-crud-page-foundation/08-07-SUMMARY.md (this file)"
  modified:
    - "rest-types/src/lib.rs (FeatureFlagTO + From + unknown())"
    - "rest/src/lib.rs (pub mod feature_flag, RestStateDef trait + getter, ApiDoc nest, router nest)"
    - "rest/tests/openapi_surface.rs (added /feature-flag/{key} path + FeatureFlagTO schema)"
    - "shifty_bin/src/main.rs (RestStateImpl.feature_flag_service field + RestStateDef impl + getter)"
    - "shifty_bin/src/integration_test.rs (mod admin_auto_grant + mod feature_flag)"
    - "shifty-dioxus/src/api.rs (get_feature_flag + FeatureFlagTO import)"
    - "shifty-dioxus/src/loader.rs (load_feature_flag + FeatureFlag import)"
    - "shifty-dioxus/src/state/mod.rs (pub mod feature_flag)"
    - "shifty-dioxus/src/service/mod.rs (pub mod feature_flag)"
    - "shifty-dioxus/src/app.rs (feature_flag_service coroutine + initial LoadAbsenceRangeSourceActive send)"
    - "shifty-dioxus/src/component/top_bar.rs (cutover_active + has_hr wiring + partition_nav_items_with_context + 3 new tests + 8 updated tests)"
    - "shifty-dioxus/src/page/absences.rs (md/lg breakpoints across StatsGrid, VacationEntitlementCard Self+HR, VacationPerPersonList, AbsenceFilterBar, AbsenceList header + row)"

key-decisions:
  - "SQLite trigger with INSERT OR IGNORE in body is the simpler and safer alternative to a CONSTRAINT-based approach: a pure UNIQUE-constraint on role_privilege already exists (from 20240426150045_user-roles.sql), and the trigger leverages it via INSERT OR IGNORE so manual role_privilege inserts in later migrations are still idempotent."
  - "Backfill uses INSERT OR IGNORE rather than a NOT EXISTS subquery — the UNIQUE constraint on (role_name, privilege_name) makes the OR IGNORE atomic and avoids race conditions in case multiple migrations run in parallel (though sqlx runs them serial; defense-in-depth)."
  - "FE FeatureFlagsState defaults to `None` per flag (not `Some(false)`). UI must explicitly check `Some(true)` to render flag-gated surfaces. This means: solange das Service den ersten Load nicht abgeschlossen hat, ist der TopBar-Eintrag unsichtbar — fail-safe, weil ein flackernder 'sichtbar/unsichtbar/sichtbar'-Effekt vermieden wird."
  - "FE feature_flag_service is triggered with `feature_flag_handle.send(LoadAbsenceRangeSourceActive)` direkt nach `use_coroutine` in app.rs, anstatt über `use_effect`. Funktioniert, weil das Service nach dem ersten Polling-Tick alle pending Actions abarbeitet — der erste UI-Render rendert mit None (Eintrag versteckt), nach dem Backend-Roundtrip wird re-rendered mit Some(false|true)."
  - "REST handler returns description: None statt `description: Some(...)` aus dem DAO, weil das Service-Trait `FeatureFlagService::is_enabled` nur den Bool-Wert liefert. Eine separate `find_flag(key) -> Option<FeatureFlag>`-Methode existiert nicht; der Frontend-Use-Case (Cutover-Gate) braucht die Description nicht. Wenn künftig 'description'-Rendering im FE erforderlich wird, kann ein neues Service-Trait-Member dafür eingeführt werden."
  - "Static-API kompatibel: is_admin_target() bleibt unverändert, is_admin_target_with_context() ist additiv. Existierende Tests (z.B. `partition_nav_items_splits_admin_and_top_level_preserving_order`) bleiben unangetastet (sales-only-Datensatz, default-context = false)."
  - "Plan 08-07 Task 5 (responsive) bricht aus dem auto-fit/minmax-Pattern aus, weil deterministische Breakpoint-Steps (1 → 2 → 3/4/5 cols) auf jedem Viewport vorhersagbarer rendern. auto-fit/minmax erzeugt eine fließende Variabilität, die sich auf typischen 14\"-Laptops zu unschönen 1- oder 2-col-Fallbacks faltet."
  - "TopBar HR-Hierarchie bekommt KEINEN neuen Submenu-Trigger — das existierende Admin-Dropdown (TopBarAdminGroupLabel = 'Verwaltung'/'Administration'/'Správa') wird wiederverwendet. i18n-Keys sind bereits in allen drei Locales vorhanden, kein neues Add-Text nötig."

requirements-completed: [FUI-A-01, FUI-A-02]

# Metrics
duration: ~75min
completed: 2026-05-08
---

# Phase 8 Plan 07: Gap-Closure — Admin-Trigger, Feature-Flag-REST, Cutover-Gate, TopBar-Hierarchie, Responsive

**Plan-Type:** Gap-Closure (`gap_closure: true`) — folgt nach Plan 08-06-UAT und schließt fünf Probleme, die im UAT-Setup blockierten oder die fertige AbsencesPage UX-seitig kaputt machten.

## Performance

- **Duration:** ~75 min (incl. read-first phase, full workspace test cycle, FE WASM build gate)
- **Started:** 2026-05-08
- **Completed:** 2026-05-08
- **Tasks:** 6
- **Files modified:** 19 (7 new + 12 modified)

## Accomplishments

1. **Admin-Trigger** — sqlx-Migration `20260508120000_admin-auto-grant-privilege.sql` mit INSERT-OR-IGNORE-Backfill (cutover_admin + feature_flag_admin landen automatisch an admin) + AFTER-INSERT-Trigger `privilege_auto_grant_admin` für jede zukünftige Privilege-Migration. Migration ist idempotent (zweites `sqlx migrate run` ist no-op). 4 Integration-Tests beweisen Backfill, Forward-Trigger, Idempotenz und realer Permission-Service-Pfad.

2. **Feature-Flag REST-Endpoint** — `GET /feature-flag/{key}` mit utoipa, fail-safe `enabled: false` für unbekannte Keys (kein 404), auth-only readable (unauth → 401 via globaler Middleware oder Service::Unauthorized → 401 mapping). Surface-Test deckt neuen Pfad + Schema ab. 6 Integration-Tests: Service-Layer (known/unknown/unauth-context) + REST-Layer (200-known/200-unknown/401-no-user) via tower::oneshot.

3. **Frontend Feature-Flag-State** — `FeatureFlag` mirror + `FeatureFlagsState` aggregate (Option<bool>-per-flag, defaultet None), `feature_flag_service` Coroutine mit `FEATURE_FLAGS_STORE` GlobalSignal, App-Start-Trigger lädt `absence_range_source_active` einmalig. 3 Unit-Tests im state-Modul.

4. **TopBar Cutover-Gate + HR-Submenu** — `nav_visibility` mit `cutover_active`-Parameter; `partition_nav_items_with_context` mit `absences_under_admin`-Flag; HR-User sehen "Abwesenheiten" als ersten Eintrag im existierenden Verwaltung-Dropdown; Non-HR-User behalten Top-Level-Position. Cutover=false versteckt den Eintrag in allen Rollen. Tests: 11 nav_visibility-Tests umgeschrieben + 3 neue (cutover-hidden, with-context-promote, hr-absences-active-label).

5. **Responsive AbsencesPage** — VacationEntitlementCard Self+HR Stat-Boxes als 1/2/5-cols-Stepping; StatsGrid 1/2/3-cols; VacationPerPersonList 1/2-cols; AbsenceFilterBar Mobile-stack→md-row; AbsenceList Header md+ visible, Row Mobile-stack md+grid. Plan-Acceptance-Reset: 11 ssr snapshot tests aus Plan 08-05 unverändert grün, 509 FE tests workspace-weit.

6. **Final Gates** — workspace cargo test: 388 service_impl + 66 integration + 10 dao + 11 utils + 3 openapi-surface + 8 misc = grün. FE cargo test: 509/509 grün. WASM build: grün. sqlx migrate idempotent verifiziert.

## Task Commits

Each task committed atomically via jj:

1. **Pre-task: Plan setup** (Plan-File + 2 Todos + tailwind regen — vom User vor Plan-Execution erstellt) — `3e5480ee` (docs).
2. **Task 1: admin role auto-grants every privilege via trigger** — `b02bb160` (feat).
3. **Task 2: GET /feature-flag/{key} REST endpoint with utoipa** — `733a4904` (feat).
4. **Task 3: frontend feature_flag state + service wiring** — `e05603a5` (feat).
5. **Task 4: TopBar cutover-gate + HR-only Verwaltung-submenu** — `148c628b` (feat).
6. **Task 5: responsive desktop layout for AbsencesPage** — `55c06e06` (feat).
7. **Task 6: SUMMARY + STATE + ROADMAP update** — folgt nach diesem Self-Check.

_All commits via `jj describe` — keine git-Operationen._

## Files Created/Modified

### Created (7)
- `migrations/sqlite/20260508120000_admin-auto-grant-privilege.sql` — backfill + AFTER-INSERT-trigger.
- `rest/src/feature_flag.rs` — GET /{key} handler + FeatureFlagApiDoc.
- `shifty-dioxus/src/state/feature_flag.rs` — FeatureFlag mirror + FeatureFlagsState aggregate + 3 tests.
- `shifty-dioxus/src/service/feature_flag.rs` — coroutine + FEATURE_FLAGS_STORE + FeatureFlagAction + ABSENCE_RANGE_SOURCE_ACTIVE_KEY const.
- `shifty_bin/src/integration_test/admin_auto_grant.rs` — 4 tests (backfill exhaustive, forward-trigger, idempotence, DEVUSER privileges_for_user).
- `shifty_bin/src/integration_test/feature_flag.rs` — 6 tests (service-layer 3 + REST-layer 3).
- `.planning/phases/08-absence-crud-page-foundation/08-07-SUMMARY.md` — this file.

### Modified (12)
- `rest-types/src/lib.rs` — FeatureFlagTO struct + feature-gated From + unknown(key) constructor.
- `rest/src/lib.rs` — `pub mod feature_flag;` + RestStateDef::FeatureFlagService + getter + ApiDoc nest + router nest.
- `rest/tests/openapi_surface.rs` — `/feature-flag/{key}` path + `FeatureFlagTO` schema.
- `shifty_bin/src/main.rs` — RestStateImpl.feature_flag_service field + RestStateDef impl wiring + getter; reuse of existing feature_flag_service Arc (nur expose, keine Konstruktor-Änderung).
- `shifty_bin/src/integration_test.rs` — `mod admin_auto_grant;` + `mod feature_flag;`.
- `shifty-dioxus/src/api.rs` — `FeatureFlagTO` import + `get_feature_flag(config, key)` async fn.
- `shifty-dioxus/src/loader.rs` — `FeatureFlag` import + `load_feature_flag(config, key)` async fn.
- `shifty-dioxus/src/state/mod.rs` — `pub mod feature_flag;`.
- `shifty-dioxus/src/service/mod.rs` — `pub mod feature_flag;`.
- `shifty-dioxus/src/app.rs` — `use_coroutine(feature_flag_service)` + initial `LoadAbsenceRangeSourceActive` send.
- `shifty-dioxus/src/component/top_bar.rs` — `nav_visibility` 3-arg signature, FEATURE_FLAGS_STORE-read, has_hr-read, `partition_nav_items_with_context` + `is_admin_target_with_context` helper, 11 tests umgeschrieben + 3 neue.
- `shifty-dioxus/src/page/absences.rs` — md/lg breakpoints in 6 Komponenten (StatsGrid, VacationEntitlementSelfBody, VacationEntitlementHrBody, VacationPerPersonList, AbsenceFilterBar, AbsenceList header + row).

## Decisions Made

Siehe `key-decisions` im Frontmatter — die wichtigsten:

1. **SQLite-Trigger mit `INSERT OR IGNORE` im Body** statt eines komplexeren CONSTRAINT-Triggers. Nutzt das existierende UNIQUE(role_name, privilege_name) auf `role_privilege` aus 20240426150045 als Idempotenz-Garant.

2. **`FeatureFlagsState` mit `Option<bool>`-per-Flag (Default None)** statt `bool` (Default false). Forciert explizite `Some(true)`-Matches in der UI; verhindert Flackern (sichtbar → unsichtbar → sichtbar) während des ersten Service-Loads.

3. **REST-Handler liefert `description: None`** weil das Service-Trait `is_enabled(key) -> bool` nur Bool liefert. FE-Use-Case (Cutover-Gate) braucht keine Description; wenn künftig benötigt, neuer Service-Trait-Member.

4. **Static `is_admin_target` bleibt unverändert** — additive `is_admin_target_with_context` für die HR-Promote-Logik. Backwards-kompatibel mit Plan 08-05-Tests; Plan-08-07-Test-Updates sind explizit auf das neue Verhalten umgeschrieben.

5. **Existierendes Admin-Dropdown wiederverwendet** — keine neuen i18n-Keys nötig (TopBarAdminGroupLabel = "Verwaltung"/"Administration"/"Správa" existiert bereits aus Plan 08-05 / Pre-08-Phase).

6. **Responsive: deterministische Breakpoint-Steps statt auto-fit/minmax** — `grid-cols-1 sm:grid-cols-2 md:grid-cols-N` ist auf jedem Viewport vorhersagbar; auto-fit faltet auf 14"-Laptops oft unschön zu 1- oder 2-col.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] `rest::feature_flag` als private Modul nicht aus shifty_bin/integration_test/feature_flag.rs erreichbar.**
- **Found during:** Task 2 (cargo test gegen den neuen integration test).
- **Issue:** `mod feature_flag;` in `rest/src/lib.rs` war initial private; Integration-Test brauchte `rest::feature_flag::generate_route`.
- **Fix:** `pub mod feature_flag;` (mit Inline-Kommentar — gleiches Pattern wie `pub mod cutover` aus Phase 4 Plan-04-06).
- **Files modified:** `rest/src/lib.rs`.
- **Verification:** `cargo test --package shifty_bin integration_test::feature_flag` — 6/6 grün.
- **Committed in:** Task 2-Commit `733a4904`.

**2. [Rule 2 — Missing critical test coverage] Idempotenz-Test für die Migration wurde im Plan-Body nicht expliziert.**
- **Found during:** Task 1 (Test-Schreiben).
- **Issue:** Plan 08-07 Acceptance-Liste fordert "Migration läuft idempotent (zweite `sqlx migrate run` ist no-op)" — das wird auf Migration-Engine-Level garantiert (versioniertes `_sqlx_migrations` table), aber es gibt keinen positiven Idempotenz-Test für den Fall, dass eine spätere Migration parallel manuell `INSERT INTO role_privilege ('admin', 'X', ...)` macht.
- **Fix:** `admin_grant_trigger_is_idempotent_against_manual_role_privilege` — verifiziert `INSERT OR IGNORE` im Trigger-Body greift; doppelt-grant erzeugt kein Duplikat.
- **Files modified:** `shifty_bin/src/integration_test/admin_auto_grant.rs`.
- **Verification:** Test grün.
- **Committed in:** Task 1-Commit `b02bb160`.

**3. [Rule 3 — Blocking] `FeatureFlagsState::absence_range_source_active()`-Helper-Method war zu Task-3-Zeit unbenutzt; Compiler-Warning bei Task 4-Start.**
- **Found during:** Task 3 (cargo check).
- **Issue:** Field `absence_range_source_active: Option<bool>` definiert + Helper-Method `absence_range_source_active(&self) -> bool` definiert, aber noch kein Aufruf-Site existiert (Aufruf kommt erst in Task 4).
- **Fix:** Compiler-Warning ignoriert (cargo check liefert "method ... never used" als warning, nicht error). In Task 4 wird der Aufruf `FEATURE_FLAGS_STORE.read().absence_range_source_active()` hinzugefügt; Warning verschwindet.
- **Files modified:** Keine (Warning ist erwartet zwischen Task 3 und Task 4; selbstheilend).
- **Verification:** Task-4-Tests grün; Warning weg.
- **Committed in:** Task 3-Commit `e05603a5`.

**4. [Rule 1 — Bug] Plan 08-05's `hr_admin_user_partitions_into_top_level_and_full_admin_group`-Test brach bei Task 4.**
- **Found during:** Task 4 (cargo test top_bar).
- **Issue:** Plan 08-05 hatte den Test mit "Abwesenheiten als letzter Top-Level-Eintrag" verankert (D-10-Pattern). Plan 08-07 ändert das zu "Abwesenheiten als ERSTER Admin-Group-Eintrag für HR-User".
- **Fix:** Test umgeschrieben — `top_labels` enthält nicht mehr "Abwesenheiten"; `admin_labels` startet jetzt mit "Abwesenheiten" gefolgt von Mitarbeiter/Abrechnungszeiträume/Benutzerverwaltung/Textvorlagen.
- **Files modified:** `shifty-dioxus/src/component/top_bar.rs`.
- **Verification:** `cargo test top_bar` — 42/42 grün.
- **Committed in:** Task 4-Commit `148c628b`.

**Total deviations:** 4 (1 Rule-3 Blocking pub-mod, 1 Rule-2 Missing Test, 1 Rule-3 Cosmetic Cross-Task Warning, 1 Rule-1 Test-Update). Keine erweitern den Scope; alle aus Acceptance abgedeckt.

## Issues Encountered

1. **NixOS `nix develop` shell-eval status** — `nix develop -c cargo ...` funktioniert für Backend cargo (incl. sqlx-cli, openssl, pkg-config). Frontend cargo (FE-Build mit reqwest/openssl) braucht `nix-shell -p openssl pkg-config`; FE WASM-Build (target wasm32-unknown-unknown) braucht `nix-shell -p lld`. Workaround konsistent angewendet (siehe Plan 08-05 + 08-04 SUMMARYs).

2. **Plan-File Setup vs Task 1** — Plan-File und Todos waren bereits im working copy, als ich startete. Per `jj split` sauber in eine separate `docs(08-07): plan setup` change ausgelagert (`3e5480ee`), dann erst Task 1 als eigene change beschrieben. Vermeidet Vermischung zwischen "Plan vom User vorbereitet" und "Plan vom Executor abgearbeitet".

## User Setup Required

Nach Pull der Änderungen einmalig:
```bash
nix develop -c sqlx migrate run --source migrations/sqlite
```

Damit ist die admin-auto-grant-Migration auf der lokalen DB und ALLE existierenden Privilegien sind an die admin-Rolle gebunden. Der DEVUSER hat dadurch automatisch `cutover_admin`, `feature_flag_admin`, sales, hr, shiftplan.edit etc. — kein manueller `INSERT INTO role_privilege` mehr nötig.

## Next Phase Readiness

Plan 08-07 ist Gap-Closure und schließt die Phase 8 vollständig:
- **Plan 08-06** (UAT smoke) kann jetzt sauber ausgeführt werden (DEVUSER hat Vollzugriff, FE-Cutover-Gate verhindert Confusing-Menu-Eintrag pre-cutover, AbsencesPage rendert auf Desktop sauber multi-spaltig, HR-Hierarchie macht TopBar nicht überfüllt).
- **Frontend kann das Cutover-Flag-State** in anderen UIs nutzen (z.B. wenn künftig die Block-Reports oder Billing-Period-UI cutover-spezifisch rendern soll, kommt eine zweite `LoadFlag(...)`-Variante in den `FeatureFlagAction`-Enum).
- **Künftige Privilege-Migrationen** brauchen sich nicht mehr um die admin-Rolle zu kümmern — der Trigger feuert automatisch.

Keine Blocker.

## Self-Check: PASSED

**Files verified to exist:**
- `migrations/sqlite/20260508120000_admin-auto-grant-privilege.sql` ✓
- `rest/src/feature_flag.rs` ✓
- `shifty-dioxus/src/state/feature_flag.rs` ✓
- `shifty-dioxus/src/service/feature_flag.rs` ✓
- `shifty_bin/src/integration_test/admin_auto_grant.rs` ✓
- `shifty_bin/src/integration_test/feature_flag.rs` ✓
- `.planning/phases/08-absence-crud-page-foundation/08-07-SUMMARY.md` ✓

**Commits verified to exist (jj log):**
- `3e5480ee` Plan setup (pre-task)
- `b02bb160` Task 1 (admin auto-grant trigger)
- `733a4904` Task 2 (REST endpoint)
- `e05603a5` Task 3 (FE state + service)
- `148c628b` Task 4 (TopBar cutover-gate + HR-submenu)
- `55c06e06` Task 5 (responsive layout)

**Verification commands re-run during self-check:**
- `cargo test --workspace` (backend) — alle Test-Targets grün (388 service_impl + 66 integration + 10 dao + 11 utils + 3 openapi-surface + 8 sales).
- `cargo test` in shifty-dioxus — 509/509 grün.
- `cargo build --target wasm32-unknown-unknown` in shifty-dioxus (mit `nix-shell -p lld`) — grün.
- `sqlx migrate run --source migrations/sqlite` zweimal hintereinander — erstes mal applied, zweites mal no-op.

---
*Phase: 08-absence-crud-page-foundation*
*Plan: 08-07 (Gap-Closure)*
*Completed: 2026-05-08*
