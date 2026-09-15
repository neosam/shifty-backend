---
phase: 54-data-model-voluntary-stats
plan: 04
subsystem: [backend, rest, rest-types, openapi]
tags: [rest, dto, hr-gate, openapi, utoipa, voluntary-stats, VOL-STAT-01, VOL-STAT-02, VOL-ACCT-01, VOL-ACCT-02, D-F1-01]
status: complete
requirements:
  - VOL-STAT-01
  - VOL-STAT-02
  - VOL-ACCT-01
  - VOL-ACCT-02
dependency_graph:
  requires:
    - 54-03 (VoluntaryStatsService trait + impl + DI-Wiring)
  provides:
    - REST-Endpoint GET /report/{id}/voluntary-stats?year=YYYY
    - VoluntaryStatsTO DTO (rest-types) mit OpenAPI-Schema
  affects:
    - Plan 54-05 (Frontend konsumiert diesen Endpoint direkt — Prefix /report ist im
      bestehenden Dioxus.toml Proxy bereits abgedeckt)
tech-stack:
  added: []
  patterns:
    - Additiv-neues DTO in rest-types (kein Bloat auf EmployeeReportTO — RESEARCH C.1)
    - HR-Only-Redaktion an Service-Grenze (Praezedenz VAC-OFFSET-01 v1.8, kein 403)
    - error_handler-Wrapper + tracing::instrument analog get_weekly_statistics
    - #[serde(default)] auf jedem Option-Feld fuer Wire-Backward-Compat
    - Integration-Test via tower::ServiceExt::oneshot gegen echten RestStateImpl
      (Praezedenz: convert_to_absence.rs, feature_flag.rs)
key-files:
  created:
    - shifty_bin/src/integration_test/voluntary_stats.rs
  modified:
    - rest-types/src/lib.rs (VoluntaryStatsTO struct + From-Impl)
    - rest/src/report.rs (Route + Handler + VoluntaryStatsRequest + ReportApiDoc)
    - rest/src/lib.rs (pub mod report; fuer Test-Zugriff)
    - shifty_bin/src/integration_test.rs (mod voluntary_stats-Registrierung)
decisions:
  - "Handler leitet Context 1:1 an VoluntaryStatsService::get_voluntary_stats — die HR-Only-Redaktion passiert bereits im Service (Plan 03). Kein zusaetzlicher permission_service.has_privilege-Check im REST-Layer, weil das die etablierte VAC-OFFSET-01-Praezedenz konsistent haelt (Non-HR → 200 mit lauter None-Feldern statt 403 Forbidden)."
  - "Integration-Test lebt in shifty_bin/src/integration_test/voluntary_stats.rs (nicht in rest/tests/voluntary_stats.rs wie der Plan-Text als Primaerpfad andeutet). Grund: RestStateDef hat inzwischen ~35 Services; eine handrolled Fixture in rest/tests/ waere > 50 Zeilen unimplemented!()-Placeholder — was der Plan-Text als Fallback-Kriterium fuer den shifty_bin-Pfad explizit nennt."
  - "report-Modul in rest/src/lib.rs auf pub geaendert (analog extra_hours, feature_flag) — Dokumentations-Kommentar erlaeutert den Test-Zugriff via tower::oneshot."
  - "VoluntaryStatsRequest hat nur `year: u32`, kein `until_week` — die Aggregation laeuft ueber das gesamte ISO-Jahr (F1/F2-Semantik aus Plan 03). Damit unterscheidet sich der Query-Vertrag bewusst von ReportRequest."
metrics:
  duration: ~7 min
  completed: 2026-07-07
  tasks: 4
  files_touched: 5
  tests_added: 2
  commits: 3
---

# Phase 54 Plan 04: REST-Endpoint `/report/{id}/voluntary-stats` + DTO Summary

**One-liner:** HR-Only REST-Endpoint fuer das Freiwillig-Stunden-Konto — neues `VoluntaryStatsTO`-DTO, `GET /report/{id}/voluntary-stats?year=YYYY`, OpenAPI-Registrierung, API-Level-None-Redaktion fuer Non-HR (kein 403), verifiziert durch 2 HTTP-Roundtrip-Integrationstests.

## Route-Deklaration Diff-Snippet

```rust
// rest/src/report.rs — generate_route
.route(
    "/{id}/voluntary-stats",
    get(get_voluntary_stats::<RestState>),
)
```

## utoipa::path Doc-Snippet

```rust
#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/{id}/voluntary-stats",
    tags = ["Report"],
    params(
        ("id" = Uuid, Path, description = "Sales person ID"),
        ("year" = u32, Query, description = "ISO-year for voluntary-stats aggregation")
    ),
    responses(
        (status = 200, description = "HR-only voluntary hours statistics; Non-HR receives all fields as null (API-level redaction)", body = VoluntaryStatsTO, content_type = "application/json"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_voluntary_stats<RestState: RestStateDef>(
    rest_state: State<RestState>,
    query: Query<VoluntaryStatsRequest>,
    Path(sales_person_id): Path<Uuid>,
    Extension(context): Extension<Context>,
) -> Response { /* delegates to voluntary_stats_service().get_voluntary_stats(...) */ }
```

## OpenAPI-Schema-Registrierung

```rust
// rest/src/report.rs — ReportApiDoc
#[derive(OpenApi)]
#[openapi(
    tags((name = "Report", ...)),
    paths(
        get_short_report_for_all,
        get_report,
        get_short_week_report,
        get_weekly_statistics,
        get_attendance_statistics,
        get_voluntary_stats            // <-- neu
    ),
    components(schemas(
        ShortEmployeeReportTO,
        EmployeeReportTO,
        ReportRequest,
        EmployeeWeeklyStatisticsTO,
        EmployeeAttendanceStatisticsTO,
        WeekdayAttendanceTO,
        VoluntaryStatsTO,              // <-- neu
        VoluntaryStatsRequest          // <-- neu
    ))
)]
pub struct ReportApiDoc;
```

## VoluntaryStatsTO Struktur

```rust
// rest-types/src/lib.rs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct VoluntaryStatsTO {
    #[serde(default)] pub ist_per_contract_week: Option<f32>,
    #[serde(default)] pub ist_total:             Option<f32>,
    #[serde(default)] pub soll_total:            Option<f32>,
    #[serde(default)] pub delta:                 Option<f32>,
    #[serde(default)] pub contract_weeks:        Option<u32>,
}
```

## Test-Ergebnis (HR + Non-HR)

`shifty_bin/src/integration_test/voluntary_stats.rs` — 2 HTTP-Roundtrip-Tests via `tower::ServiceExt::oneshot`:

| Test | Auth-Context | Seed | Erwartung | Status |
|------|-------------|------|-----------|--------|
| `rest_voluntary_stats_hr_returns_populated_fields` | `Some("DEVUSER")` (via `create_admin_user` → HR-Privileg) | 4 KW 2026 (KW10..=13), `committed_voluntary=2.0`, 8h Manual `VolunteerWork` in KW10 | 200; `ist_total=Some(8.0)`, `soll_total=Some(8.0)`, `delta=Some(0.0)`, `contract_weeks=Some(4)`, `ist_per_contract_week=Some(2.0)` | ok |
| `rest_voluntary_stats_non_hr_returns_all_null` | `Some("some-non-hr-user")` (kein Role-Binding) | identisch | 200; alle 5 Felder `None`; JSON-Body enthaelt explizit `"ist_per_contract_week":null` + `"contract_weeks":null` (keine `serde(skip)`-Falle) | ok |

```
running 2 tests
test integration_test::voluntary_stats::rest_voluntary_stats_hr_returns_populated_fields ... ok
test integration_test::voluntary_stats::rest_voluntary_stats_non_hr_returns_all_null ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 64 filtered out; finished in 0.13s
```

## Decision-Coverage-Diff

| Truth (Plan `must_haves.truths`) | Verifikation |
|---|---|
| HR-Auth → `Some`-Felder; Non-HR → `null`-Felder (API-Level-Redaktion) | `rest_voluntary_stats_hr_returns_populated_fields` (Some-Assertion) + `rest_voluntary_stats_non_hr_returns_all_null` (None-Assertion + Wire-JSON-`null`-Check) |
| Endpoint in ReportApiDoc via `#[utoipa::path]` registriert, `application/json` | Grep `openapi_surface.rs` bleibt gruen; `#[utoipa::path]`-Annotation im Handler; `content_type = "application/json"` explizit gesetzt |
| **[D-F1-01]** HR-Antwort konsistent Ist/Soll/Delta gemaess contract-weeks-Semantik aus Plan 03 (kein DTO-seitiges Nachrechnen) | HR-Test verifiziert `ist_total=8.0`, `soll_total=8.0`, `delta=ist_total - soll_total = 0.0`, `ist_per_contract_week = ist_total / contract_weeks = 8.0 / 4 = 2.0` — DTO uebernimmt Service-Werte unveraendert via `From<&VoluntaryStats>` |

## Deviations from Plan

**1. [Rule 3 - Blocker-Fix] Test-Datei in shifty_bin statt rest/tests/**

- **Gefunden bei:** Task 3 (Test-Setup)
- **Issue:** Plan-Text nennt `rest/tests/voluntary_stats.rs` als Primaerpfad. Der `RestStateDef`-Trait deklariert inzwischen ~35 Services + Getter; eine handrolled Fixture in `rest/tests/` waere > 50 Zeilen `unimplemented!()`-Placeholders — der Plan-Text nennt als Kriterium fuer Fallback explizit: *"Falls die adhoc Fixture zu breit wird (mehr als ~50 Zeilen): reduziere den Test auf `service_impl` Ebene ... Alternativ: falls Vollimpl zu invasiv, HTTP-Test im shifty_bin/tests/-Modul, das bereits das volle RestState hat."*
- **Fix:** `shifty_bin/src/integration_test/voluntary_stats.rs` mit dem echten `RestStateImpl` — dieselbe Praezedenz wie `convert_to_absence.rs` (Phase 8.5), `feature_flag.rs` (Phase 8.7).
- **Files:** `shifty_bin/src/integration_test/voluntary_stats.rs`, `shifty_bin/src/integration_test.rs` (mod-Registrierung), `rest/src/lib.rs` (`pub mod report;`).
- **Commit:** `74e9cf0`

**2. [Rule 3 - Blocker-Fix] `pub mod report;` in `rest/src/lib.rs`**

- **Gefunden bei:** Task 3 (Integration-Test-Compile)
- **Issue:** Das `report`-Modul war `mod report;` (privat). Der Integrations-Test in `shifty_bin` braucht `rest::report::generate_route`.
- **Fix:** Auf `pub mod report;` erhoben mit Praezedenz-Kommentar auf `pub mod feature_flag;` und `pub mod extra_hours;` (siehe Phase 8.5/8.7-Muster).
- **Files:** `rest/src/lib.rs`
- **Commit:** `74e9cf0`

Keine weiteren Abweichungen — Plan-Text wurde 1:1 abgearbeitet. Insbesondere KEIN neuer Proxy-Eintrag in `Dioxus.toml` (Prefix `/report` ist bereits abgedeckt — Verifikation faellt in Plan 05).

## Auth-Gates

Keine — DEVUSER-Setup ist in TestSetup vorinstalliert.

## Commits

- `a06133e` **feat(54-04): add VoluntaryStatsTO DTO in rest-types** — Neues Struct mit 5 Option-Feldern, `ToSchema` + `#[serde(default)]` je Feld, `From<&VoluntaryStats>`-Impl hinter `service-impl`-Feature.
- `671a107` **feat(54-04): add GET /report/{id}/voluntary-stats endpoint** — Route + `VoluntaryStatsRequest` Query-Struct + Handler + `#[utoipa::path]`-Annotation + `ReportApiDoc`-Erweiterung.
- `74e9cf0` **test(54-04): add HTTP integration tests for voluntary-stats endpoint** — 2 tower::oneshot-Tests (HR + Non-HR) + `pub mod report;` in `rest/src/lib.rs` + mod-Registrierung.

## Verification-Log

| Gate | Command | Result |
|------|---------|--------|
| DTO-Compile | `cargo build -p rest-types --features service-impl` | Finished (1m 01s, keine Warnings) |
| REST-Compile | `cargo build -p rest` | Finished (29.62s) |
| Integration-Test | `DATABASE_URL="sqlite::memory:" cargo test -p shifty_bin voluntary_stats` | 2 passed; 0 failed |
| Full-Suite Build | `cargo build --workspace` | Finished (29.74s) |
| Full-Suite Tests | `SQLX_OFFLINE=true DATABASE_URL="sqlite::memory:" cargo test --workspace --lib --tests` | ok (66 shifty_bin integration + alle unit-tests im workspace) |
| Full-Suite Tests inkl. Doctests | `SQLX_OFFLINE=true DATABASE_URL="sqlite::memory:" cargo test --workspace` | Doctests grün (0 failed, 2 ignored in `service_impl::shortday_gate`) |
| Clippy Gate | `SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings` | Finished (11.25s), keine Warnings |

## Self-Check: PASSED

- `rest-types/src/lib.rs` enthaelt `pub struct VoluntaryStatsTO` (grep) ✅
- `rest-types/src/lib.rs` enthaelt `impl From<&service::voluntary_stats::VoluntaryStats> for VoluntaryStatsTO` ✅
- `rest/src/report.rs` enthaelt Route `/{id}/voluntary-stats` + `pub async fn get_voluntary_stats` ✅
- `rest/src/report.rs` enthaelt `#[utoipa::path]`-Annotation + `VoluntaryStatsTO` + `VoluntaryStatsRequest` in `ReportApiDoc` ✅
- `rest/src/lib.rs` fuehrt `pub mod report;` ✅
- `shifty_bin/src/integration_test/voluntary_stats.rs` existiert mit 2 Tests ✅
- Commits `a06133e`, `671a107`, `74e9cf0` in git log ✅
- `cargo clippy --workspace -- -D warnings` grün ✅
