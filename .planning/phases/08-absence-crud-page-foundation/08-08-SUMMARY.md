---
phase: 08-absence-crud-page-foundation
plan: 08-08
subsystem: api
tags: [cutover, drift-report, openapi, rest-types, dto, ux]

# Dependency graph
requires:
  - phase: 04-migration-cutover
    provides: CutoverService.run + DriftRow + diff-report file format + QuarantineReason enum
  - phase: 08-absence-crud-page-foundation/08-07
    provides: Feature-Flag REST + admin-auto-grant trigger (DEVUSER kann cutover ausführen)
provides:
  - CutoverRunResultTO.gate_drift_report — inline failed-gate diagnostics over HTTP
  - CutoverGateDriftRowTO.quarantined_entries — per-extra-hours-row breakdown
  - CutoverQuarantineEntryTO — neue DTO mit reason_code + reason_text + suggested_action + weekday-Code
  - QuarantineReason::human_text() / suggested_action() — englische Beschreibungen pro Variante
affects: [09-booking-flow, cutover-ui-backlog, future-i18n-of-cutover-response]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Inline-Drift-Report-Pattern — File-Pfad als Audit-Artefakt, JSON über HTTP für UI-Konsumenten."
    - "Reason-Mapping zentral am Service-Enum — DTOs stringifizieren nur, kein ad-hoc Text in From-Impls."
    - "Backwards-compatible additive DTO-Felder mit `#[serde(default)]` für Old-Client-Toleranz."
    - "Plan-08-08 Quarantine-Bucket-Map als Cross-Phase-Datentransport (Migration → Gate)."

key-files:
  created:
    - .planning/phases/08-absence-crud-page-foundation/08-08-SUMMARY.md
  modified:
    - service/src/cutover.rs
    - service/src/absence.rs
    - service_impl/src/cutover.rs
    - service_impl/src/test/cutover.rs
    - rest-types/src/lib.rs
    - rest/src/cutover.rs
    - rest/tests/openapi_surface.rs
    - shifty_bin/src/integration_test/cutover.rs

key-decisions:
  - "Reason-Text + suggested_action als Methoden auf service::cutover::QuarantineReason — nicht im DTO-Layer. Single source of truth, reusable für künftige CLI-Tools."
  - "Backend-Default ist Englisch; i18n übernimmt das spätere Cutover-UI auf Frontend-Seite (eigenes Backlog-Item)."
  - "Quarantine-Bucket-Map (HashMap<(Uuid, AbsenceCategory, u32), Vec<CutoverQuarantineEntry>>) als zusätzlicher Rückgabewert von migrate_legacy_extra_hours_to_clusters → vom Gate konsumiert. Hält die per-Entry-Daten in-memory ohne zusätzliche DAO-Roundtrips."
  - "AbsenceCategory bekommt Hash-Derive — purely additive, ermöglicht HashMap-Key-Use ohne Service-Tier-Bruch."
  - "On-disk diff-report-Format unverändert gelassen (Audit-Artefakt). Inline gate_drift_report ist eine Kopie in typisierter Form über HTTP — der File-Pfad-Detour wird optional, nicht ersetzt."
  - "Weekday-Code als freistehende Funktion `pub fn weekday_code(time::Date) -> &'static str` in rest-types — orphan-rule-freundlich, von DTO + Future-Konsumenten reusable."
  - "CutoverQuarantineEntryTO als eigene Schema im OpenAPI-Surface-Test verankert — Drift-Detection für Frontend-Konsumenten."

patterns-established:
  - "Service-Enum-Reason-Mapping: Wenn ein typisiertes Service-Enum für End-User sichtbar wird, leben die human-readable und remediation-Strings als Methoden direkt am Enum (nicht im Wire-DTO). DTOs serialisieren via `enum.method().to_string()`. Reusable für REST + CLI + zukünftige UI-Konsumenten."
  - "Inline-Drift-Report-Pattern: REST-Antworten, die heute auf Filesystem-Pfade verweisen (z.B. Audit-Artefakte), bekommen zusätzlich einen typisierten Inline-Body. File bleibt für Audit-Trail, der inline-Body ist die UX-Datenquelle."
  - "Cross-Phase-Daten-Bucketing: Wenn Phase A und Phase B über Service-State hinweg per-Entity-Daten teilen müssen, ist eine HashMap-Map mit (Uuid, EnumKategorie, Year)-Composite-Key die einfachste Form ohne Tx-Roundtrip."

requirements-completed: []

# Metrics
duration: 50min
completed: 2026-05-08
---

# Phase 08 Plan 08-08: Cutover-Response Drift-Details Summary

**Inline `gate_drift_report` plus per-entry quarantine details (extra_hours_id + date + weekday + amount + reason_text + suggested_action) im Response von POST /admin/cutover/{commit,gate-dry-run} — der User sieht jetzt sofort *was* den Gate-Fail verursacht hat, ohne den diff_report-File zu öffnen.**

## Performance

- **Duration:** ~50 min
- **Started:** 2026-05-08T13:54Z
- **Completed:** 2026-05-08T14:43Z
- **Tasks:** 5 (4 atomic feat/test + 1 closure)
- **Files modified:** 8 (Service-Trait + Service-Impl + Service-Test + Service-Absence-Cat + REST-DTO + REST-Cutover-Doc + OpenAPI-Surface-Test + Integration-Test)

## Accomplishments

- `QuarantineReason::human_text()` + `QuarantineReason::suggested_action()` für alle 5 Varianten — englische Texte pro Variante; defense-in-depth-Test (`quarantine_reason_text_and_action_non_empty_per_variant`) verhindert "vergessene Variante"-Regression.
- Service-Layer: `CutoverQuarantineEntry` + `CutoverGateDriftReport`-Strukturen + `DriftRow.quarantined_entries`-Feld + `CutoverRunResult.gate_drift_report`-Feld; per-Entry-Daten kommen direkt aus `migrate_legacy_extra_hours_to_clusters` ohne extra DAO-Roundtrip.
- DTO-Layer: `CutoverQuarantineEntryTO` (extra_hours_id + ISO-Date + 3-Letter-Weekday + amount + reason_code + reason_text + suggested_action) + `CutoverGateDriftRowTO.quarantined_entries` + `CutoverRunResultTO.gate_drift_report` mit `#[serde(default)]` für Backwards-Compat.
- `weekday_code(time::Date) -> &'static str` als reusable Helper in rest-types.
- Integration-Test `test_failed_gate_returns_inline_drift_report_with_per_entry_details` deckt das User-Live-Beispiel ab (3-Tage-Woche-Vertrag + Vacation-Eintrag am Freitag → quarantine_reason `contract_hours_zero_for_day`, drift = 20.0h) und assertet sowohl Service-Layer als auch Wire-Tier-DTO-Roundtrip.

## Task Commits (jj-native)

Each task committed atomically with `jj describe -m` followed by `jj new`:

1. **Plan setup** — `c575b7bf` (`docs(08-08): plan setup — cutover-response polish (drift inline diagnostics)`)
2. **Task 1: human-readable text + suggested-action per QuarantineReason** — `a0b71fc1` (`feat`)
3. **Task 2: per-entry quarantine details in CutoverRunResult** — `b141a067` (`feat`)
4. **Task 3: inline gate_drift_report in CutoverRunResultTO with per-entry details** — `c4b0e91a` (`feat`)
5. **Task 4: integration test + openapi surface for inline drift_report** — `4b07738e` (`test`)
6. **Task 5: SUMMARY + STATE + ROADMAP** — pending (this commit, after final-gates)

_All commits via `jj describe -m "<msg>"; jj new` — keine git-Befehle._

## Files Created/Modified

- `service/src/cutover.rs` — `QuarantineReason::{human_text, suggested_action}` + neue Strukturen `CutoverQuarantineEntry` + `CutoverGateDriftReport` + `DriftRow.quarantined_entries` + `CutoverRunResult.gate_drift_report`.
- `service/src/absence.rs` — `AbsenceCategory` bekommt `Hash`-Derive (additiv, für `HashMap`-Key in der Quarantine-Bucket-Map).
- `service_impl/src/cutover.rs` — `migrate_legacy_extra_hours_to_clusters` returniert jetzt 3-Tuple mit `QuarantineBucketMap`; `compute_gate` konsumiert die Map und befüllt `DriftRow.quarantined_entries`; `run` baut bei Gate-Fail das inline `CutoverGateDriftReport` über den freistehenden Helper `build_gate_drift_report`.
- `service_impl/src/test/cutover.rs` — neuer Defense-in-Depth-Test für Reason-Mapping; 3 destructure-Sites auf das neue 3-Tuple migriert.
- `rest-types/src/lib.rs` — neue DTO `CutoverQuarantineEntryTO` + `CutoverGateDriftRowTO.quarantined_entries` + `CutoverRunResultTO.gate_drift_report`; `From`-Impls + `pub fn weekday_code(time::Date) -> &'static str`.
- `rest/src/cutover.rs` — `CutoverQuarantineEntryTO` zur `components(schemas(...))`-Liste der `CutoverApiDoc` ergänzt.
- `rest/tests/openapi_surface.rs` — `"CutoverQuarantineEntryTO"` zur `EXPECTED_SCHEMAS`-Liste.
- `shifty_bin/src/integration_test/cutover.rs` — neuer End-to-End-Test `test_failed_gate_returns_inline_drift_report_with_per_entry_details`.

## Decisions Made

- **Reason-Mapping am Service-Enum, nicht am DTO** — `QuarantineReason::human_text()` und `QuarantineReason::suggested_action()` sind die single source of truth. DTOs (`CutoverQuarantineEntryTO`) rufen die Methoden auf und stringifizieren. Reuse für künftige CLI-Tools / Admin-Reports ohne Wire-Tier-Abhängigkeit.
- **Englisch als Backend-Default** — i18n bleibt eine Frontend-Verantwortlichkeit (separates Backlog-Item für das spätere Cutover-UI). Backend liefert stabilen `reason_code` + englische `reason_text` / `suggested_action`; das Frontend kann den Code als i18n-Key behandeln, falls Lokalisierung gewünscht ist.
- **Backwards-compatible-additiv** — `quarantine_reasons: Vec<String>` (Aggregat-Liste) bleibt im DriftRow erhalten; `quarantined_entries: Vec<CutoverQuarantineEntryTO>` ist purely additive mit `#[serde(default)]`. Old-Clients sehen die DTO-Erweiterung transparent.
- **On-disk diff-report unverändert** — Das persistierte JSON-File unter `.planning/migration-backup/cutover-gate-{ts}.json` wurde nicht geändert. Es bleibt das Audit-Artefakt mit dem etablierten Format (Plan 04-05 / `test_diff_report_json_schema` weiterhin grün). Inline `gate_drift_report` ist eine in-memory typisierte Kopie für REST-Konsumenten — File-Pfad-Detour wird optional, nicht ersetzt.
- **Quarantine-Bucket-Map als Cross-Phase-Daten-Transport** — `HashMap<(Uuid, AbsenceCategory, u32), Vec<CutoverQuarantineEntry>>` wird vom Migrationsschritt aufgebaut und an den Gate-Schritt weitergereicht. Verhindert zusätzliche DAO-Roundtrips, hält den Code lokal an einem Ort.
- **`AbsenceCategory: Hash`** — Purely additive Derive-Erweiterung, kein Service-Tier-Bruch (Basic-Service `AbsenceService` exposiert das Enum unverändert). Ermöglicht den Composite-HashMap-Key.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] OpenAPI-Komponenten-Liste in rest/src/cutover.rs erweitert**
- **Found during:** Task 3 (DTO-Erweiterung)
- **Issue:** `CutoverQuarantineEntryTO` ist eine neue named schema. utoipa registriert nicht-transitiv genutzte ToSchema-Derives nicht automatisch in der ApiDoc — der OpenAPI-Surface-Test (Plan 08-03) hätte das Schema sonst nicht gesehen.
- **Fix:** `CutoverQuarantineEntryTO` zur `components(schemas(...))`-Liste in `rest/src/cutover.rs` ergänzt; analog zu `CutoverGateDriftReportTO`.
- **Files modified:** `rest/src/cutover.rs`
- **Verification:** `cargo test -p rest --test openapi_surface` → 3/3 grün, neue Schema-Eintrag im `EXPECTED_SCHEMAS` greift.
- **Committed in:** `c4b0e91a` (zusammen mit Task 3 — die Komponenten-Liste-Pflege gehört wire-tier zur DTO-Erweiterung).

**2. [Rule 3 - Blocking] AbsenceCategory braucht Hash-Derive**
- **Found during:** Task 2 (Quarantine-Bucket-Map)
- **Issue:** `HashMap<(Uuid, service::absence::AbsenceCategory, u32), Vec<CutoverQuarantineEntry>>` braucht `Hash` auf `AbsenceCategory`; das Enum hatte bisher nur `Clone, Copy, Debug, PartialEq, Eq`.
- **Fix:** `Hash` zur Derive-Liste hinzugefügt — purely additiv, kein bestehender Konsument betroffen.
- **Files modified:** `service/src/absence.rs`
- **Verification:** `cargo test --workspace` grün; bestehende AbsenceCategory-Tests unverändert grün.
- **Committed in:** `b141a067` (zusammen mit Task 2 — minimaler Scope-Bruch, gehört zum Migrate-→-Gate-Daten-Transport).

---

**Total deviations:** 2 auto-fixed (1 missing-critical OpenAPI-Eintrag, 1 blocking Hash-Derive). Beide notwendig für die Plan-Acceptance-Kriterien. Kein Scope-Creep.

## Issues Encountered

- Keine. Alle 4 Tasks mechanisch durchlaufen, Tests beim ersten Run grün (12/12 service_impl-cutover, 19/19 shifty_bin-cutover, 3/3 openapi_surface).

## Verification Snapshot

```text
$ nix develop -c cargo test --workspace
test result: ok. 389 passed (service_impl/lib)
test result: ok. 67 passed (shifty_bin/integration)
test result: ok. 11 passed (rest/lib)
test result: ok. 10 passed (service/lib)
test result: ok. 8 passed (dao_impl_sqlite)
test result: ok. 3 passed (rest/openapi_surface)
... alle anderen Crates ok mit 0 failed
```

```text
$ cd shifty-dioxus && nix-shell -p lld --run "cargo build --target wasm32-unknown-unknown"
warning: `shifty-dioxus` (bin "shifty-dioxus") generated 38 warnings (pre-existing)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.23s
```

WASM-Build grün, keine FE-Änderungen.

## Live Drift Example — Vorher / Nachher

User-Live-Szenario (Max Schmidt, 3-Tage-Woche-Vertrag, Vacation-Eintrag 2026-05-08 (Friday), 20.0h):

**Vor diesem Plan:**
```json
{
  "gate_passed": false,
  "gate_drift_rows": 1,
  "diff_report_path": ".planning/migration-backup/cutover-gate-1746...json"
}
```
→ User muss den File öffnen, manuell parsen.

**Nach diesem Plan:**
```json
{
  "gate_passed": false,
  "gate_drift_rows": 1,
  "diff_report_path": ".planning/migration-backup/cutover-gate-1746...json",
  "gate_drift_report": {
    "gate_run_id": "...",
    "drift_threshold": 0.01,
    "total_drift_rows": 1,
    "drift": [
      {
        "sales_person_id": "08d0c879-9d7c-4d0b-a6f3-9acd3c5b206b",
        "sales_person_name": "Max Schmidt",
        "category": "Vacation",
        "year": 2026,
        "legacy_sum": 20.0,
        "derived_sum": 0.0,
        "drift": 20.0,
        "quarantined_extra_hours_count": 1,
        "quarantine_reasons": ["contract_hours_zero_for_day"],
        "quarantined_entries": [
          {
            "extra_hours_id": "...",
            "date": "2026-05-08",
            "weekday": "Fri",
            "amount": 20.0,
            "reason_code": "contract_hours_zero_for_day",
            "reason_text": "Employee has zero contract hours on this weekday (e.g. a 4-day-week contract with the booking falling on a non-workday)",
            "suggested_action": "Delete the entry, or move it to a workday on which the employee's contract specifies > 0 hours"
          }
        ]
      }
    ],
    "passed": false
  }
}
```
→ User kann ohne externe Doku verstehen, was zu tun ist.

## User Setup Required

None - keine externe Service-Konfiguration nötig.

## Next Phase Readiness

- Plan 08-08 closed. Phase 8 plan_count: 8 (08-01..08-05 + 08-07 + 08-08 done; 08-06 UAT-smoke noch offen).
- Backend-Surface für künftiges Cutover-UI (Backlog-Item) ist jetzt vollständig — der Browser kann den Failed-Gate ohne Filesystem-Access rendern.
- Optional Follow-Up im FE-Backlog: i18n-Locale-Mapping `reason_code → de.rs/en.rs/cs.rs` für lokalisierte Reason-Text-/Action-Anzeige.

## Self-Check: PASSED

- [x] `service/src/cutover.rs` updated (QuarantineReason methods + 2 neue Structs + DriftRow/CutoverRunResult Felder)
- [x] `service/src/absence.rs` updated (Hash-Derive auf AbsenceCategory)
- [x] `service_impl/src/cutover.rs` updated (3-Tuple Return + Bucket-Map-Build + compute_gate-Signatur + build_gate_drift_report-Helper)
- [x] `service_impl/src/test/cutover.rs` updated (Reason-Mapping-Test + 3 destructure-Sites)
- [x] `rest-types/src/lib.rs` updated (CutoverQuarantineEntryTO + erweiterte DTOs + From-Impls + weekday_code-Helper)
- [x] `rest/src/cutover.rs` updated (Component-Liste in CutoverApiDoc)
- [x] `rest/tests/openapi_surface.rs` updated (CutoverQuarantineEntryTO in EXPECTED_SCHEMAS)
- [x] `shifty_bin/src/integration_test/cutover.rs` updated (neuer Test test_failed_gate_returns_inline_drift_report_with_per_entry_details)
- [x] Commits in jj log: c575b7bf (plan setup), a0b71fc1 (Task 1), b141a067 (Task 2), c4b0e91a (Task 3), 4b07738e (Task 4)
- [x] cargo test --workspace grün (389 + 67 + 11 + 10 + 8 + 3 = ≥488 Tests, 0 failed)
- [x] WASM-Build grün
- [x] OpenAPI surface test grün

---

*Phase: 08-absence-crud-page-foundation*
*Plan: 08-08*
*Completed: 2026-05-08*
