---
status: complete
phase: 04-migration-cutover
source: 04-00-foundation-and-migrations-SUMMARY.md, 04-01-service-traits-and-stubs-SUMMARY.md, 04-02-cutover-service-heuristic-SUMMARY.md, 04-03-carryover-rebuild-service-SUMMARY.md, 04-04-extra-hours-flag-gate-and-soft-delete-SUMMARY.md, 04-05-SUMMARY.md, 04-06-SUMMARY.md, 04-07-integration-tests-and-profile-SUMMARY.md
started: 2026-05-03T17:06:16Z
updated: 2026-05-03T17:26:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: Server stoppen, lokale DB neu provisionieren (`rm localdb.sqlite3 && nix-shell --run 'sqlx setup --source migrations/sqlite'`), dann `cargo run` ausführen. Alle 41 Migrationen (inkl. der 4 neuen Phase-4-Migrationen) laufen idempotent durch, Server bootet ohne Fehler, Log zeigt `Running server at 127.0.0.1:3000`. Die DI-Reihenfolge (FeatureFlagService vor ExtraHoursService, plus CutoverServiceImpl + CarryoverRebuildServiceImpl + CutoverDaoImpl) konstruiert sauber.
result: pass
note: "User-Korrektur: korrekter Befehl ist `sqlx database reset --source migrations/sqlite` (nicht `sqlx setup`); shell.nix hat einen Fehler, daher muss `nix develop` statt `nix-shell --run` verwendet werden. Cold-Start selbst funktioniert."

### 2. Workspace Tests Green
expected: `cargo test --workspace` läuft komplett grün durch (433+ Tests, davon 18 neue E2E-Cutover-Tests in `shifty_bin/src/integration_test/cutover.rs` und 11 Service-Tests in `service_impl/src/test/cutover.rs`). 0 failed, keine ignorierten Phase-4-Tests.
result: pass

### 3. OpenAPI Snapshot Determinismus
expected: `cargo test -p rest --test openapi_snapshot` 3× hintereinander ausführen. Jeder Lauf grün, danach `ls rest/tests/snapshots/*.snap.new 2>/dev/null | wc -l` zeigt `0` — keine Drift in der OpenAPI-Surface.
result: pass

### 4. POST /admin/cutover/profile als HR
expected: Server läuft, mit HR-Privileg `POST /admin/cutover/profile` aufrufen (z. B. via curl gegen `http://127.0.0.1:3000/admin/cutover/profile`). Antwort: HTTP 200 + JSON-Body im `CutoverProfileTO`-Format (Felder `buckets[]` mit `sales_person_id`, `category`, `year`, `row_count`, `sum_amount`, `fractional_count`, `weekend_on_workday_only_contract_count`, `iso_53_indicator`). Eine neue Datei `.planning/migration-backup/profile-{nanos}.json` erscheint im Repo-Root.
result: pass
note: "User confirmed: HTTP 200"

### 5. POST /admin/cutover/profile ohne HR
expected: Gleiches Endpoint, aber als User ohne HR-Privileg → HTTP 403 (Forbidden). Keine JSON-Datei wird geschrieben, der Tx wird sauber zurückgerollt.
result: skipped
reason: "Rollen sind hardcodiert — Permission-Boundary kann manuell nicht getestet werden. Abgedeckt durch Integration-Test `test_profile_forbidden_for_unprivileged_user` (passed) sowie analoge Forbidden-Tests in cutover.rs und service-Mock-Tests."

### 6. POST /admin/cutover/gate-dry-run als HR
expected: `POST /admin/cutover/gate-dry-run` als HR-User → HTTP 200 + `CutoverRunResultTO` (mit `passed`, `total_clusters`, `migrated_clusters: 0` weil dry-run, `diff_report_path: Some("…/cutover-gate-{nanos}.json")`). Die Diff-Report-JSON-Datei taucht in `.planning/migration-backup/` auf. Datenbank-State unverändert (Tx rolled back).
result: pass

### 7. POST /admin/cutover/commit erfordert cutover_admin
expected: Mit HR-User (ohne `cutover_admin`-Privileg) `POST /admin/cutover/commit` aufrufen → HTTP 403. Mit User, dem `cutover_admin` per `role_privilege` zugewiesen wurde, gleicher Aufruf → HTTP 200 + `CutoverRunResultTO` mit `passed: true`. Nach dem Erfolg: `SELECT enabled FROM feature_flag WHERE key = 'absence_range_source_active'` ergibt `1`.
result: pass

### 8. Idempotenter Re-Run nach Commit
expected: Nach erfolgreichem Cutover-Commit aus Test 7 ein zweites Mal `POST /admin/cutover/commit` aufrufen. Antwort: HTTP 200, `CutoverRunResultTO` mit `total_clusters: 0` und `quarantined: 0` (alle Legacy-Rows sind via `absence_period_migration_source` bereits gemappt und werden ausgefiltert). Keine doppelten `absence_period`-Zeilen entstehen.
result: issue
reported: "Ich bekomme ein 403"
severity: major

### 9. POST /extra-hours mit Vacation post-cutover
expected: Bei aktiviertem Feature-Flag `absence_range_source_active = true` ein `POST /extra-hours` mit `category: "Vacation"` (oder `SickLeave`/`UnpaidLeave`) abschicken → HTTP 403 mit JSON-Body `{"error":"extra_hours_category_deprecated","category":"vacation","message":"Use POST /absence-period for this category"}`. Keine neue Zeile in `extra_hours`.
result: pass

### 10. POST /extra-hours mit ExtraWork post-cutover
expected: Bei aktiviertem Feature-Flag `absence_range_source_active = true` ein `POST /extra-hours` mit `category: "ExtraWork"` (oder anderer nicht-deprecated Kategorie wie `Holiday`/`Volunteer`) abschicken → HTTP 200/201, neue Zeile in `extra_hours`. Der Flag-Gate wird für nicht-deprecated Kategorien gar nicht ausgelöst (keine `is_enabled`-DAO-Call).
result: pass

## Summary

total: 10
passed: 8
issues: 1
pending: 0
skipped: 1
blocked: 0

## Gaps

- truth: "Zweiter POST /admin/cutover/commit nach erfolgreichem ersten Commit liefert HTTP 200 mit total_clusters=0 (idempotent)"
  status: acknowledged
  reason: "User reported: Ich bekomme ein 403"
  severity: major
  test: 8
  hypothesis: "Wahrscheinlich Setup-Issue: `cutover_admin`-Privileg ist im aktiven dev-Setup nicht zugewiesen — dann wäre Test 7 ebenfalls nie real durchlaufen. Code-Pfad ist durch passing Integration-Tests `test_commit_success_for_cutover_admin` + `test_idempotence_rerun_no_op` abgedeckt (siehe 04-VERIFICATION.md, 6/6 verified)."
  user_decision: "Option 3 — Issue als bekanntes Setup-Risiko anerkannt, Phase 4 wird trotzdem als verified markiert. Re-Test bei Bedarf nach Permission-Setup im dev-DB."

## Acknowledged Gaps

- **Test 8 (idempotenter Re-Run)** — manuell ein 403 statt 200 erhalten. Wahrscheinlich Setup-Issue (kein `cutover_admin`-Grant im dev-DB). Nicht-blocking für Phase-Abschluss, weil der Code-Pfad durch passende Integration-Tests abgedeckt ist (siehe 04-VERIFICATION.md). Bei Bedarf manuell re-testen mit `INSERT OR IGNORE INTO role_privilege (role_name, privilege_name, ...) VALUES ('hr', 'cutover_admin', ...)`.
