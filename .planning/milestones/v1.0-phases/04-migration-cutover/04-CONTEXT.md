# Phase 4: Migration & Cutover - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Bestehende `extra_hours`-Einträge der Kategorien `Vacation` / `SickLeave` / `UnpaidLeave` werden heuristisch zu `absence_period`-Zeiträumen rekonstruiert. Vor dem Feature-Flag-Flip stellt ein **Validierungs-Gate pro `(sales_person_id, kategorie, jahr)`** sicher, dass `sum(derive_hours_for_range) == sum(extra_hours_legacy)` mit Toleranz < 0.01h. Erst dann wird `absence_range_source_active = true` in **derselben SQLite-Tx** wie MIG-01 (Migration), MIG-04 (Carryover-Refresh + Pre-Backup) und MIG-05 (Soft-Delete der alten Rows) atomar geflippt. REST-Endpunkte für die alten Vacation/Sick/UnpaidLeave-Pfade werden **flag-gated** stillgelegt — vor Cutover unverändert, nach Cutover liefert POST `/extra-hours` mit diesen 3 Kategorien `403 ExtraHoursCategoryDeprecated`. Phase 4 läuft im **laufenden Server**, kein Bin-Restart, kein Downtime.

**In Scope (Phase 4):**

- **Wave 0 — Hygiene** (D-Phase4-15):
  - `dao/Cargo.toml` und `dao_impl_sqlite/Cargo.toml`: `uuid = { ..., features = ["v4"] }` ergänzen, damit `cargo test -p dao` + `cargo test -p dao_impl_sqlite` standalone grün laufen (pre-existing Phase-1-Drift, dokumentiert in `.planning/phases/03-.../deferred-items.md`).
  - Doku-Hinweis zum lokalen `localdb.sqlite3`-Drift in `.planning/phases/04-migration-cutover/deferred-items.md` (jeder Dev kann seine lokale DB neu provisionieren, kein Code-Fix nötig).

- **MIG-01 Migrations-Heuristik:**
  - Neue Tabelle `absence_migration_quarantine` (D-Phase4-03) mit (`extra_hours_id` BLOB(16) PK, `reason` TEXT NOT NULL, `sales_person_id` BLOB(16) NOT NULL, `category` TEXT NOT NULL, `date_time` TEXT NOT NULL, `amount` FLOAT NOT NULL, `migrated_at` TEXT NOT NULL, `cutover_run_id` BLOB(16) NOT NULL).
  - Neue Tabelle `absence_period_migration_source` (D-Phase4-04) mit (`extra_hours_id` BLOB(16) PK, `absence_period_id` BLOB(16) NOT NULL, `cutover_run_id` BLOB(16) NOT NULL, `migrated_at` TEXT NOT NULL, FK auf `absence_period(id)`).
  - **Strict-Cluster-Heuristik** (D-Phase4-02): konsekutive Werktage gleicher Sales-Person + gleicher Kategorie, deren `extra_hours.amount` exakt den Vertrags-Tagesstunden entsprechen (per `EmployeeWorkDetails` am jeweiligen Tag), werden zu einer `absence_period`-Range gemerged. Werktage = per Vertrag aus `EmployeeWorkDetails.workdays` (D-Phase4-01). Bruchstunden, Wochenend-Einträge bei Mo-Fr-Vertrag, Vertragswechsel-mid-cluster mit unpassenden Stunden, ISO-53-Edge-Cases → Quarantäne.
  - Re-Run-idempotent über `absence_period_migration_source.extra_hours_id` PK.

- **MIG-02 Cutover-Gate:**
  - Berechnung pro `(sales_person_id, kategorie, jahr)` (D-Phase4-05): vergleicht `sum(extra_hours.amount WHERE category IN (Vacation, SickLeave, UnpaidLeave) AND year(date_time) = Y)` gegen `sum(derive_hours_for_range(year_start..year_end, sp).hours)` mit Toleranz < 0.01h.
  - Diff-Report (D-Phase4-06): JSON-Datei in `.planning/migration-backup/cutover-gate-{ISO_TIMESTAMP}.json` mit Schema `{ gate_run_id, run_at, drift_threshold, total_drift_rows, drift: [{ sales_person_id, sales_person_name, category, year, legacy_sum, derived_sum, drift, quarantined_extra_hours_count }], passed: bool }`. Plus `tracing::error!` pro Drift-Zeile.
  - Verzeichnis `.planning/migration-backup/` muss vor dem ersten Run existieren (Plan-Phase legt es per Wave-0-Task an).

- **MIG-03 Cutover-Surface:**
  - Zwei REST-Endpunkte (D-Phase4-07): `POST /admin/cutover/gate-dry-run` (Permission: HR) und `POST /admin/cutover/commit` (Permission: neues Privileg `cutover_admin`, analog `feature_flag_admin` aus Phase 2).
  - Neuer Service `CutoverService` (Business-Logic-Tier per `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen"): konsumiert `AbsenceService` (Business-Logic, für `derive_hours_for_range` und ggf. `create`), `ExtraHoursService` (Basic, für Read-All und Soft-Delete), `CarryoverService` (Basic), `FeatureFlagService` (Basic), `EmployeeWorkDetailsService` (Basic, für Per-Tag-Vertragsstunden in der Heuristik), `PermissionService`, `TransactionDao`. Eine einzige Methode `run(dry_run: bool, ctx, tx) -> Result<CutoverRunResult, ServiceError>` + Helper-Methoden für Heuristik / Gate / Refresh.
  - `CutoverRunResult { run_id: Uuid, ran_at: PrimitiveDateTime, dry_run: bool, gate_passed: bool, total_clusters: u32, migrated_clusters: u32, quarantined_rows: u32, gate_drift_rows: u32, diff_report_path: Option<Arc<str>> }`.
  - **Drift-Schutz** (D-Phase4-08): Commit fährt Migration + Gate erneut auf dem aktuellen extra_hours-State zum Zeitpunkt der Tx — kein Lock, kein Token-Pessimistic-Check, identischer Code-Pfad wie Dry-Run. Pre-Flight ist nur Komfort.
  - Tx-Grenzen (D-Phase4-14): **eine** SQLite-Tx über alles via `TransactionDao::use_transaction`. BEGIN → INSERT absence_period (für alle gemergten Cluster) + INSERT absence_period_migration_source + INSERT absence_migration_quarantine → Gate-Berechnung → IF dry_run OR gate_fail: ROLLBACK + Diff-Report-Response → ELSE: UPDATE extra_hours SET deleted = NOW(), update_process = 'phase-4-cutover-migration' WHERE id IN migrated_set + INSERT employee_yearly_carryover_pre_cutover_backup + UPDATE employee_yearly_carryover (pro (sp, year) im Gate-Scope) + UPDATE feature_flag SET enabled = 1 WHERE key = 'absence_range_source_active' → COMMIT. Alle inneren Service-Calls bekommen die Tx als `Some(tx)` und committen NICHT.
  - utoipa-Annotations + Wrapper-DTOs `CutoverRunResultTO`, `CutoverGateDriftRowTO` inline in `rest-types/src/lib.rs` (Repo-Konvention).

- **MIG-04 Carryover-Refresh:**
  - Refresh-Scope (D-Phase4-12): genau die `(sales_person_id, year)`-Tupel, die im Gate eine non-zero Vacation/Sick/UnpaidLeave-Stunden-Summe gefunden haben. Alle anderen `employee_yearly_carryover`-Rows bleiben unberührt.
  - Pre-Migration-Backup (D-Phase4-13): neue Tabelle `employee_yearly_carryover_pre_cutover_backup` (Schema identisch zu `employee_yearly_carryover` + `cutover_run_id` BLOB(16) NOT NULL, `backed_up_at` TEXT NOT NULL). Innerhalb der atomaren Tx VOR dem UPDATE: `INSERT INTO ... SELECT * FROM employee_yearly_carryover WHERE (sales_person_id, year) IN (gate_scope_set)`.
  - Carryover-Refresh berechnet pro `(sp, year)` neu: lädt Reporting-Inputs auf der nun-aktiven AbsencePeriod-Quelle (FeatureFlag ist innerhalb der Tx schon `true`) und schreibt `carryover_hours` + `vacation` neu. **Plan-Phase entscheidet**, ob ein neuer `CarryoverService::rebuild_for_year(sp, year, ctx, tx)`-Helper eingeführt wird (sauberer, wiederverwendbar) oder ob die Logik inline im `CutoverService` lebt (engerer Scope, weniger Surface). Vorgabe: neuer Helper auf `CarryoverService` — er ist Cross-Entity (konsumiert `ReportingService` o.ä.) und gehört damit ins Business-Logic-Tier.

- **MIG-05 REST-Deprecation:**
  - REST-Handler in `rest/src/extra_hours.rs` für POST prüfen: WENN `feature_flag_service.is_enabled("absence_range_source_active") == true` UND Body-Category ∈ {Vacation, SickLeave, UnpaidLeave} → return `403 ExtraHoursCategoryDeprecated { category }` (D-Phase4-09). Vor Cutover: identisches Verhalten wie heute. Atomarer Übergang via Flag.
  - Soft-Delete der alten Rows (D-Phase4-10): `UPDATE extra_hours SET deleted = NOW(), update_process = 'phase-4-cutover-migration' WHERE id IN (SELECT extra_hours_id FROM absence_period_migration_source WHERE cutover_run_id = ?)`. Quarantänierte Rows bleiben aktiv. DELETE + GET-Endpunkte bleiben für legacy-Cleanup/Historie unverändert.
  - Neue ServiceError-Variante `ExtraHoursCategoryDeprecated { category: ExtraHoursCategory }` in `service/src/lib.rs`, mapped in `rest/src/lib.rs::error_handler` auf HTTP 403 mit Body `{ "error": "extra_hours_category_deprecated", "category": "vacation", "message": "Use POST /absence-period for this category" }`.

- **OpenAPI-Snapshot-Test (SC-6, D-Phase4-11):**
  - Neue Test-Datei `rest/tests/openapi_snapshot.rs` ruft `ApiDoc::openapi()` und schreibt mit `insta::assert_json_snapshot!(...)` ein Pin-File `rest/tests/snapshots/openapi_snapshot__openapi.snap`.
  - Neue Crate-Dep `insta = { version = "1", features = ["json"] }` in `rest/Cargo.toml` (dev-dependencies).
  - Bei jedem Diff (Field-Rename, Endpoint-removed, Status-Code-changed) wird der Test rot. Updates per `cargo insta review` (Mensch bestätigt, dann jj-Commit).

- **SC-1 Production-Data-Profile:**
  - Plan-Phase entscheidet, ob das Profile als read-only Service-Methode (`CutoverService::profile()` → JSON in `.planning/migration-backup/profile-{ts}.json`) oder als ein-und-derselbe `gate-dry-run`-Endpoint mit zusätzlichem `include_profile: true`-Query-Parameter implementiert wird. Vorgabe: separate Methode `profile()`, weil sie LANGSAM (full-table-scan über extra_hours) sein darf und unabhängig vom Gate-Run ist.
  - Profile-Inhalte (Plan-Phase finalisiert): Histogramm pro (sp, kategorie, jahr) mit Counts + Sum + Bruchstunden-Quote + Wochenend-Einträge-Count + ISO-53-Indicator. JSON-Format konsistent mit Diff-Report-Format.

- **Pflicht-Tests:**
  - `_forbidden`-Tests pro neue public service method auf `CutoverService` (HR ∨ `cutover_admin`-Pattern aus Phase 2 D-Phase2-07).
  - Dry-Run-Test: Migration + Gate auf Test-Fixture läuft, Diff-Report wird generiert, DB-State unverändert (Gate fail OR pass — beides testbar).
  - Commit-Test: Migration + Gate-Pass auf Fixture läuft, Flag wird true, alte Rows soft-deleted, Carryover refreshed, Backup-Tabelle befüllt.
  - Rollback-Test: Gate-Fail (Fixture mit absichtlicher Quarantäne) → Tx rollback'd, Flag bleibt false, alle DB-State unverändert.
  - Idempotenz-Test: Cutover läuft 2x; zweiter Run sieht alle Rows in `absence_period_migration_source` und skippt sie.
  - Per-Mitarbeiter-Per-Jahr-Per-Kategorie-Invariant-Test (SC-5) in `shifty_bin/src/integration_test/cutover.rs`: Pre-Migration-Stunden-Summe == Post-Migration-derived-Stunden-Summe für jede Kombination.
  - OpenAPI-Snapshot-Test (D-Phase4-11) als Smoke-Test bei `cargo test --workspace`.
  - Integration-Test für REST-Pfade (`gate-dry-run` + `commit` + `extra_hours-Vacation-POST` vor und nach Cutover).

**Strikt nicht in Scope (Phase 4):**

- Frontend (Dioxus) — separater Workstream. Frontend-Migration vom alten `/extra-hours`-POST auf `/absence-period` UND vom alten /booking-Endpunkt auf den /shiftplan-edit-Endpunkt (Phase 3) ist Frontend-Workstream-Verantwortung.
- Bin-Tools / CLI-Subcommands für Cutover (User-Korrektur 2026-05-03: alles im laufenden Server).
- REST-Endpunkte zum Auflisten der Quarantäne-Rows (deferred — bei Bedarf in Folgephase als HR-Admin-Surface).
- Auto-Cleanup der Quarantäne-Rows (HR muss manuell per `/absence-period`-POST aufräumen oder die Quarantäne-Row akzeptieren als unveränderten extra_hours-Bestand).
- Frontend-Notification-Banner / E-Mail-Notifications zur Cutover-Vollendung — Operations-Verantwortung.
- Erweiterung von `CarryoverService` über `rebuild_for_year` hinaus (z.B. Bulk-Rebuild-Endpoint für Operations) — bei Bedarf in Folgephase.
- Reporting-Pfad-Änderungen (Phase 2 hat den Switch bereits hinter dem Flag verdrahtet — Phase 4 flippt nur den Flag).
- `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump (Phase 2 hat `2 → 3` bereits erledigt; Phase 4 ändert die Berechnungs-Logik nicht, sondern nur welcher Flag-State live ist — Trigger-4 wurde dort vorgegriffen).

</domain>

<decisions>
## Implementation Decisions

### Migrations-Heuristik

- **D-Phase4-01:** **Werktage = per-Vertrag aus `EmployeeWorkDetails.workdays`.** Für jeden Mitarbeiter und jeden Tag im extra_hours-Cluster werden die zum Datum gültigen `EmployeeWorkDetails.workdays`-Bool-Maske (Mo bis So) gelesen. Konsistent mit der `derive_hours_for_range`-Logik aus Phase 2 (D-Phase2-02) — das Cutover-Gate vergleicht so Äpfel mit Äpfeln. Mitarbeiter mit anderen Vertragsmodellen (z.B. 6-Tage-Wochen, Schicht-Modelle) werden korrekt behandelt.
- **D-Phase4-02:** **Strict-Cluster-Heuristik: `amount == contract_hours_at(day)` (sonst Quarantäne).** Konsekutive Werktage gleicher Sales-Person + gleicher Kategorie, deren `extra_hours.amount` exakt den per `EmployeeWorkDetails` am Tag gültigen Vertrags-Tagesstunden entsprechen, werden zu einer `absence_period`-Range gemerged. Bruchstunden (z.B. 4h bei 8h-Vertrag), Wochenend-Einträge bei Mo-Fr-Vertrag (amount > 0 auf Tag mit contract_hours = 0), ISO-Woche-53-Lücken, Vertragswechsel-mid-cluster mit unpassenden Stunden → **Quarantäne**. Garantiert Cutover-Gate-Identität (kein Spread-Drift); viel manuelle HR-Nacharbeit; **kein Daten-Verlust** (Quarantäne-Rows bleiben als `extra_hours` aktiv und sichtbar). Vertragswechsel-mid-week wird automatisch durch den Per-Tag-Stunden-Check abgedeckt: an der Vertragsgrenze ändert sich `contract_hours_at(day)`, der Cluster bricht natürlich auf.
- **D-Phase4-03:** **Quarantäne in eigener Tabelle `absence_migration_quarantine`** mit (`extra_hours_id` BLOB(16) PK, `reason` TEXT NOT NULL, `sales_person_id` BLOB(16) NOT NULL, `category` TEXT NOT NULL, `date_time` TEXT NOT NULL, `amount` FLOAT NOT NULL, `migrated_at` TEXT NOT NULL, `cutover_run_id` BLOB(16) NOT NULL). `extra_hours` bleibt während MIG-01 völlig unangetastet. Re-Run der Migration ist trivial idempotent (UPSERT auf `extra_hours_id`). `reason`-Beispiele: `"weekend_entry_with_workday_only_contract"`, `"amount_below_contract_hours"`, `"amount_above_contract_hours"`, `"contract_hours_zero_for_day"`, `"iso_53_week_gap"`. Spätere REST-Surface zum Listen ist möglich (deferred).
- **D-Phase4-04:** **Idempotenz via Mapping-Tabelle `absence_period_migration_source`** mit (`extra_hours_id` BLOB(16) PK, `absence_period_id` BLOB(16) NOT NULL, `cutover_run_id` BLOB(16) NOT NULL, `migrated_at` TEXT NOT NULL, FK auf `absence_period(id)`). Pro `extra_hours_id` genau eine Row. Re-Run skippt jede `extra_hours_id`, die bereits gemappt ist. Klarer Audit-Trail welche extra_hours zu welcher absence_period gehören (Cluster aus N extra_hours → N Mapping-Rows mit gleichem absence_period_id). Ermöglicht präzises späteres Rollback. **Hinweis:** Die `extra_hours.id` ist der Idempotenz-Key, nicht `logical_id` — das `extra_hours`-Schema hat KEIN logical_id-Feld (siehe `migrations/sqlite/20240618125847_paid-sales-persons.sql`).

### Cutover-Gate-Mechanik

- **D-Phase4-05:** **Gate-Granularität = pro `(sales_person_id, kategorie, jahr)`.** Für jedes Jahr, in dem ein Mitarbeiter Vacation/Sick/UnpaidLeave-Einträge hat, wird die Stunden-Summe über das gesamte Jahr verglichen. Granular genug um Drift-Quellen zu lokalisieren ("welches Jahr ist falsch?"), grob genug um Rounding-Noise auf Tagesebene zu absorbieren. Konsistent mit Carryover-Refresh-Logik (auch jahresweise) und Reporting-Snapshots (Billing-Period = Quartal/Jahr). Diff-Report listet eine Zeile pro `(sp, kat, jahr)` mit `|drift| > 0.01h`. Toleranz-Wert: `0.01` Stunden absolut (nicht relativ — relative Toleranz bei kleinen Stunden wäre zu lax).
- **D-Phase4-06:** **Diff-Report = JSON-Datei + tracing::error-Logs.** Pfad: `.planning/migration-backup/cutover-gate-{ISO_TIMESTAMP_UTC}.json`. Schema:
  ```json
  {
    "gate_run_id": "uuid",
    "run_at": "2026-05-03T14:23:00Z",
    "dry_run": true,
    "drift_threshold": 0.01,
    "total_drift_rows": 3,
    "drift": [
      { "sales_person_id": "uuid", "sales_person_name": "...", "category": "Vacation", "year": 2024, "legacy_sum": 120.0, "derived_sum": 112.0, "drift": 8.0, "quarantined_extra_hours_count": 2 }
    ],
    "passed": false
  }
  ```
  Für jede Drift-Zeile zusätzlich `tracing::error!("[cutover-gate] drift {sp_name}/{cat}/{year}: legacy={} derived={} drift={}", ...)`. CI-friendly, diff-friendly, mit jj committable.
- **D-Phase4-07:** **Zwei separate REST-Endpunkte: `POST /admin/cutover/gate-dry-run` + `POST /admin/cutover/commit`.** Dry-Run: HR-Permission (read-only-ish — er rollt back). Commit: neues Privileg `cutover_admin` (analog `feature_flag_admin` aus Phase 2 D-Phase2-07; eigene Migration seedet das Privileg). Beide nutzen denselben internen `CutoverService::run(dry_run: bool)`-Helper. Frontend kann distinct "Pre-Check"-Button und "Commit"-Button rendern. utoipa-Annotation auf beiden Pflicht. Server bleibt durchgehend online.
- **D-Phase4-08:** **Drift-Schutz = Commit fährt Migration + Gate erneut.** Pre-Flight ist HR-Komfort. Der Commit-Endpunkt fährt Migration + Gate-Berechnung NEU auf dem aktuellen extra_hours-State zum Zeitpunkt der Tx. Wenn das Gate jetzt fail't, Tx rollback + HR sieht den aktuellen Diff im Response-Body. Kein Pessimistic-Lock, kein Token-Mechanismus, kein DB-Lock-Flag — die Re-Run-Validation IST die Konsistenz-Garantie. Folge: Identischer Code-Pfad `CutoverService::run(dry_run: bool)` für beide Endpunkte, Branch nur auf COMMIT vs ROLLBACK.

### REST-Strategie + Schicksal alter ExtraHours

- **D-Phase4-09:** **`/extra-hours` POST-Block ist FLAG-GATED.** Solange `absence_range_source_active = false` (vor Cutover) bleibt das gesamte `/extra-hours`-API zu 100 % unverändert — POST mit Vacation/SickLeave/UnpaidLeave funktioniert wie heute. Erst NACH erfolgreichem Cutover (Flag ist `true`, Update passiert in derselben atomaren Tx wie die Migration) liefert POST `/extra-hours` mit diesen 3 Kategorien `403 ExtraHoursCategoryDeprecated { category }`. Die REST-Handler (oder Service-Layer in `service_impl/src/extra_hours.rs::create`) prüfen dafür `feature_flag_service.is_enabled("absence_range_source_active", ctx, tx)`-Pattern analog zum Reporting-Switch in `service_impl/src/reporting.rs:478`. **Atomarer Übergang während des laufenden Server-Betriebs** — kein Restart, kein Zwischen-Zustand, kein doppeltes API erlaubt. DELETE und GET bleiben unverändert für legacy-Cleanup/Historie. ExtraWork/Holiday/Custom/Volunteer-POSTs sind nicht betroffen (alle Flag-States akzeptieren sie).
- **D-Phase4-10:** **Soft-Delete der migrierten extra_hours-Rows passiert INNERHALB der atomaren Cutover-Tx.** Vor dem Cutover sind die Rows live + sichtbar; nach dem Cutover (selbe Tx wie Flag-Flip + Migration + Carryover) sind sie soft-deleted: `UPDATE extra_hours SET deleted = NOW(), update_process = 'phase-4-cutover-migration' WHERE id IN (SELECT extra_hours_id FROM absence_period_migration_source WHERE cutover_run_id = ?)`. **Quarantänierte Rows bleiben aktiv** (HR muss sie noch sehen + manuell auflösen). Reverse-Migration trivial möglich: `UPDATE extra_hours SET deleted = NULL, update_process = 'phase-4-cutover-rollback' WHERE update_process = 'phase-4-cutover-migration'`. GET-Endpunkte fragen `WHERE deleted IS NULL` und sehen migrierte Rows nicht mehr; historische Audits über update_process-Suche bleiben möglich.
- **D-Phase4-11:** **OpenAPI-Snapshot-Test via Insta** (SC-6). Neue Test-Datei `rest/tests/openapi_snapshot.rs` ruft `<ApiDoc as utoipa::OpenApi>::openapi()` und schreibt mit `insta::assert_json_snapshot!(openapi)` ein Pin-File. Neue dev-dependency `insta = { version = "1", features = ["json"] }` in `rest/Cargo.toml`. Bei jedem Diff (Field-Rename, Endpoint-entfernt, Status-Code-geändert, Schema-Modify) wird der Test rot. Updates per `cargo insta review` (Mensch bestätigt explizit, dann jj-Commit). **Wichtig:** Das OpenAPI-Schema selbst ist flag-unabhängig — die Endpunkte und Schemas existieren immer; was sich ändert ist nur das Runtime-Verhalten (200 vs 403). Der Snapshot lockt also den Vertrag, nicht den Flag-State. Pin-File wird im selben Phase-4-Commit angelegt wie die utoipa-Erweiterungen für die neuen `/admin/cutover/*`-Endpunkte und die neue `ExtraHoursCategoryDeprecated`-Error-Surface.

### Carryover-Refresh + Atomic-Tx-Boundaries

- **D-Phase4-12:** **Carryover-Refresh-Scope = alle `(sp, year)`-Tupel, die im Gate verglichen wurden.** Genau die Tupel, für die das Gate eine non-zero Vacation/Sick/UnpaidLeave-Stunden-Summe gefunden hat. Der Refresh berechnet `employee_yearly_carryover` pro Tupel neu, basierend auf der nun-aktiven AbsencePeriod-Quelle (FeatureFlag ist innerhalb der Tx schon `true`, also liest der `ReportingService` automatisch über `derive_hours_for_range`). Präzise (kein überschüssiger Refresh für Mitarbeiter ohne range-Absences), performant, deckt jeden betroffenen `(sp, year)`-Snapshot ab. Carryover-Werte für Jahre ohne Vacation/Sick/UnpaidLeave bleiben unberührt. **Implizit:** Quarantäne-Rows würden Gate-Fail erzeugen — wenn der Cutover-Gate grün ist, sind alle Jahre im Scope safe-to-refresh.
- **D-Phase4-13:** **Pre-Cutover-Backup via separate Tabelle `employee_yearly_carryover_pre_cutover_backup`.** Schema identisch zu `employee_yearly_carryover` (sales_person_id, year, carryover_hours, vacation, created, deleted, update_process, update_version) plus `cutover_run_id` BLOB(16) NOT NULL und `backed_up_at` TEXT NOT NULL. PK auf `(cutover_run_id, sales_person_id, year)`. Innerhalb derselben atomaren Tx VOR dem UPDATE: `INSERT INTO employee_yearly_carryover_pre_cutover_backup (sales_person_id, year, carryover_hours, vacation, ..., cutover_run_id, backed_up_at) SELECT *, ?, ? FROM employee_yearly_carryover WHERE (sales_person_id, year) IN (gate_scope_set)`. Restore möglich via `UPDATE employee_yearly_carryover SET (carryover_hours, vacation) = (b.carryover_hours, b.vacation) FROM employee_yearly_carryover_pre_cutover_backup b WHERE ...`. Innerhalb der Tx sicher, kein File-IO-Risiko, schema-konsistent.
- **D-Phase4-14:** **Eine einzige SQLite-Tx über alles via `TransactionDao::use_transaction`.** Standard-Pattern aus `shifty-backend/CLAUDE.md` § "Transaction Management". Atomar by definition. Alle inneren Service-Calls (`AbsenceService::create` für migrierte Cluster, `CarryoverService::rebuild_for_year`, `FeatureFlagService::set`, `ExtraHoursService::soft_delete_bulk` falls neu) bekommen die Tx als `Some(tx)` und committen NICHT — sie nutzen sie nur. Der `CutoverService::run` hält die Tx bis zum finalen `transaction_dao.commit(tx)` oder ruft `transaction_dao.rollback(tx)` (oder lässt sie via Drop rollback'n) bei Gate-Fail / Dry-Run. SQLite-Tx-Länge ist kein Issue für ein einmaliges Operations-Event; HR weiß, dass Cutover ein bewusster Schritt ist und kann anstehende anderen Schreib-Operationen kurz pausieren.

### Hygiene (Phase-Carry-Over)

- **D-Phase4-15:** **Wave-0 Hygiene-Tasks aus Phase-3-deferred-items mitnehmen.** Plan-Phase legt einen Mini-Wave-0-Plan (z.B. `04-00-PLAN.md`) an: `dao/Cargo.toml` und `dao_impl_sqlite/Cargo.toml` `features = ["v4"]` ergänzen (pre-existing Phase-1-Drift, dokumentiert in `.planning/phases/03-.../deferred-items.md`). Damit `cargo test -p dao` + `cargo test -p dao_impl_sqlite` standalone grün laufen. Plus: Doku-Hinweis zum lokalen `localdb.sqlite3`-Drift (jeder Dev provisioniert seine lokale DB selbst) in einem neuen `.planning/phases/04-migration-cutover/deferred-items.md`. Klein, niedrig-Risiko, nutzt die Cleanup-Energie der Phase 4.

### Claude's Discretion (Plan-Phase entscheidet)

- **C-Phase4-01:** **Migrations-Datei-Anzahl + Reihenfolge.** Plan-Phase wählt zwischen einer einzigen großen Migration (`<TS>_phase-4-cutover.sql` mit allen 3 Tabellen + Privileg-Insert) und mehreren feinen Migrations-Files (eine pro Tabelle + eine für Privileg). Vorgabe: drei separate Files für Sauberkeit (`<TS>_create-absence-migration-quarantine.sql`, `<TS+1>_create-absence-period-migration-source.sql`, `<TS+2>_create-employee-yearly-carryover-pre-cutover-backup.sql`, `<TS+3>_add-cutover-admin-privilege.sql`). Konsistent mit Phase-1- und Phase-2-Pattern.
- **C-Phase4-02:** **`CarryoverService::rebuild_for_year` Surface.** Plan-Phase entscheidet zwischen "neuer Helper auf `CarryoverService`" und "inline im `CutoverService`". Vorgabe: neuer Helper, weil Cross-Entity (konsumiert `ReportingService` o.ä.) und damit reusable. Aufgrund der Service-Tier-Konvention müsste `CarryoverService` dann zum Business-Logic-Tier gehören (heute basic — `service/src/carryover.rs` hat nur get/set). **Wichtig:** das ist ein Service-Tier-Wechsel und sollte explizit in Plan-Phase entschieden werden. Alternative: neuer Service `CarryoverRebuildService` (Business-Logic) bleibt strikt separat von `CarryoverService` (Basic) — sauberer, aber mehr Surface.
- **C-Phase4-03:** **Heuristik-Cluster-Algorithmus-Reihenfolge.** Plan-Phase wählt zwischen "iterativ Tag für Tag" (read alle extra_hours für sp+category sortiert nach date_time, build Cluster greedy bei konsekutiven Werktagen) und "SQL-Window-Function" (LAG/LEAD über sortierte Rows, group_id über conditional Cumulative Sum). Vorgabe: iterativ in Rust — einfacher zu testen, einfacher zu reasoning, SQLx-portabel. SQL-Window wäre eleganter, aber SQLite-Window-Support ist eingeschränkt und Test-Coverage in Rust ist klarer.
- **C-Phase4-04:** **Soft-Delete-Modus für migrierte extra_hours.** Plan-Phase wählt zwischen "neuer `ExtraHoursService::soft_delete_bulk(ids, tx)`-Methode" (saubere API, eine SQL-Statement) und "in-line UPDATE im `CutoverService`" (kein neuer Service-Surface, schmaler). Vorgabe: neue Bulk-Methode auf `ExtraHoursService` — saubere Service-Layer-Surface, easier zu testen.
- **C-Phase4-05:** **Production-Data-Profile-Format-Detail (SC-1).** Plan-Phase finalisiert die Histogramm-Spalten und Schwellen. Vorgabe: pro `(sp, category, year)` Counts + Sum + Bruchstunden-Quote (% extra_hours mit `amount != contract_hours_at(day)`) + Wochenend-Einträge-Count (`amount > 0` an einem Tag mit `contract_hours_at(day) == 0`) + ISO-53-Indicator. JSON-Schema konsistent mit Diff-Report-Format. Dateiname: `.planning/migration-backup/profile-{ISO_TIMESTAMP_UTC}.json`.
- **C-Phase4-06:** **Migrations-Heuristik-Vertragslookup-Performance.** Pro extra_hours_id wird `EmployeeWorkDetailsService::find_active_for(sp, date)` gerufen. Bei N=10000 Bestand-Rows = 10000 Service-Calls. Plan-Phase darf eine Pre-fetch-Optimierung einbauen (alle Verträge pro sp einmal laden, Map-Lookup pro Tag) wenn Performance-Tests Druck zeigen. Vorgabe: erst messen, dann optimieren.
- **C-Phase4-07:** **REST-Routen-Schnitt für `/admin/cutover/*`.** Plan-Phase wählt zwischen "neue Route-Gruppe `/admin/cutover/`" und "unter bestehender `/admin/`-Gruppe (falls vorhanden)" oder "unter `/feature-flag/`-Gruppe (analog Phase 2)". Vorgabe: neue Route-Gruppe `/admin/cutover/`, weil Cutover ein eigenständiges Operations-Event ist und sich semantisch von feature-flag-Toggles unterscheidet.
- **C-Phase4-08:** **Privileg-Surface.** Plan-Phase wählt zwischen "neues `cutover_admin`-Privileg" (eigenes Privileg für die destruktive Commit-Operation, separat von HR allgemein) und "Reuse `feature_flag_admin`" (analog zur Idee, dass der Flag selbst geflippt wird). Vorgabe: **neues** `cutover_admin`-Privileg — semantisch stärker, plus dass Operations-Trennung zwischen "Toggle ändern" (HR-Read-Operation in der Praxis) und "produktive Daten migrieren" (sehr destruktiv) klar ist. Migration `<TS>_add-cutover-admin-privilege.sql` mit `INSERT INTO privilege (name, update_process) VALUES ('cutover_admin', 'phase-4-migration')`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-Level Spezifikationen

- `.planning/ROADMAP.md` § Phase 4 — Goal (atomare Phase MIG-01..05), Depends-on Phase 1+2 (Phase 3 nicht hart), Success-Criteria 1-6 (Production-Data-Profile, Heuristik+Quarantäne, Cutover-Gate, Atomic-Tx, Per-MA-Per-Jahr-Per-Kategorie-Invariant-Test, OpenAPI-Snapshot-Test).
- `.planning/STATE.md` — Aktuelle Position; Architektur-Decisions: Hybrid materialize-on-snapshot / derive-on-read; Service-Tier-Konvention; Phase-3-deferred-items als Phase-4-Carry-Over.
- `shifty-backend/CLAUDE.md` § "Transaction Management" — `Option<Self::Transaction>` + `TransactionDao::use_transaction`-Pattern. **Cutover-Service folgt diesem Pattern strikt** — eine einzige Tx über alle internen Service-Calls.
- `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen: Basic vs. Business-Logic Services" — **MUST READ**. Cutover-Service ist Business-Logic-Tier; konsumiert AbsenceService + ExtraHoursService + CarryoverService + FeatureFlagService + EmployeeWorkDetailsService einseitig. Kein Cycle.
- `shifty-backend/CLAUDE.md` § "Billing Period Snapshot Schema Versioning" — Phase 2 hat `2 → 3` bereits gebumpt. Phase 4 ändert die Berechnungs-Logik nicht (nur welcher Flag-State live ist) — KEIN weiterer Bump nötig.
- `shifty-backend/CLAUDE.md` § "OpenAPI Documentation" — `#[utoipa::path(...)]` Pflicht für die zwei neuen `/admin/cutover/*`-Endpunkte; `ToSchema` für `CutoverRunResultTO`, `CutoverGateDriftRowTO`.
- `shifty-backend/CLAUDE.local.md` — VCS via `jj` (alle Commits manuell durch User; GSD-Auto-Commit deaktiviert via `commit_docs: false`); NixOS-Hinweise (`nix-shell` für `sqlx-cli`).
- `~/.claude/CLAUDE.md` — Tests sind Pflicht für jede Änderung.

### Vorphase-Outputs (Pflicht-Lektüre)

- `.planning/phases/01-absence-domain-foundation/01-CONTEXT.md` — D-02 (Kategorien-Liste: Vacation/SickLeave/UnpaidLeave), D-04/05 (absence_period-Schema + 3 partial indexes), D-09 (Permission HR ∨ self), D-12 (find_overlapping kategorie-scoped), D-15 (exclude_logical_id beim Update), D-16/17 (DateRange-Utility verfügbar).
- `.planning/phases/01-absence-domain-foundation/01-VERIFICATION.md` — Bestätigt dass `AbsenceService::create` end-to-end existiert.
- `.planning/phases/02-reporting-integration-snapshot-versioning/02-CONTEXT.md` — D-Phase2-02 (Cross-Category-Resolver lebt in `derive_hours_for_range`), D-Phase2-03 (Priorität `SickLeave > Vacation > UnpaidLeave`), D-Phase2-06 (`feature_flag` Schema), D-Phase2-07 (FeatureFlagService API + `feature_flag_admin`-Privileg-Pattern → analog für `cutover_admin`), D-Phase2-08-A (Reporting-Switch hinter Flag), D-Phase2-10 (Snapshot-Bump 2→3 bereits erledigt). **Wichtig:** `derive_hours_for_range` ist die single source of truth — Cutover-Gate fährt diese Logik 1:1.
- `.planning/phases/02-reporting-integration-snapshot-versioning/deferred-items.md` — `localdb.sqlite3`-Drift (lokal, nicht checked-in; jeder Dev muss seine DB neu provisionieren); 8 absence_period-Tests waren Phase-1-Migrations-Lücke (in Phase 3 Plan 06 recovered).
- `.planning/phases/03-booking-shift-plan-konflikt-integration/03-CONTEXT.md` — D-Phase3-01 (Wrapper-Result-Pattern im Business-Logic-Tier — relevant für `CutoverRunResult`), D-Phase3-18 (`BookingService` strikt basic — Cutover-Service touched ihn nicht), Service-Tier-Korollar.
- `.planning/phases/03-booking-shift-plan-konflikt-integration/deferred-items.md` — Hygiene-Items für D-Phase4-15 (uuid `v4`-Feature in `dao/Cargo.toml` + `dao_impl_sqlite/Cargo.toml`).

### Code-Templates für Phase 4

#### `extra_hours` (Bestand-Quelle)

- `migrations/sqlite/20240618125847_paid-sales-persons.sql` — `CREATE TABLE extra_hours` Schema. **Wichtig:** KEIN `logical_id`-Feld; Idempotenz-Key ist `extra_hours.id`.
- `migrations/sqlite/20250413073750_add-custom-extra-hours-table.sql` + `20250418200122_insert-custom-column-to-extra-hours.sql` — Custom-Extra-Hours-Erweiterung; Cutover ignoriert sie (out of scope für Vacation/Sick/UnpaidLeave).
- `service/src/extra_hours.rs:41` — `ExtraHoursCategory`-Enum (Vacation/SickLeave/UnpaidLeave + andere); `service/src/extra_hours.rs:51` — `as_report_type()` mapping.
- `service_impl/src/extra_hours.rs` — Service-Implementation. Plan-Phase ergänzt evtl. `soft_delete_bulk(ids, tx)`-Methode (C-Phase4-04).
- `dao/src/extra_hours.rs` + `dao_impl_sqlite/src/extra_hours.rs` — DAO-Layer.
- `rest/src/extra_hours.rs` — REST-Handler. Phase 4 erweitert POST-Handler um Flag-Check (D-Phase4-09).

#### `absence_period` (Migrations-Ziel)

- `migrations/sqlite/20260502170000_create-absence-period.sql` — Schema (Phase 1, recovered in Phase 3.06).
- `service/src/absence.rs` — `AbsenceService::create` returnt `AbsencePeriodCreateResult` (seit Phase 3 D-Phase3-04). Cutover ruft `create` für jeden gemergten Cluster — Warnings werden im Cutover-Kontext **ignoriert** (Migration produziert AbsencePeriods, Booking-Konflikte sind Operations-Concern, nicht Migrations-Concern). Plan-Phase entscheidet ob Cutover statt `create` einen direkten DAO-Insert macht (umgeht Forward-Warning-Loop, performanter, weniger Service-Surface) oder `create` nutzt (konsistente Permission-Checks). Vorgabe: direkter DAO-Insert mit `BookingService`-Lookup-Skip — Migration ist privileged Operation.
- `service/src/absence.rs:208` — `derive_hours_for_range` (Phase 2). **Cutover-Gate ruft genau diese Methode** zur Berechnung von `derived_sum`.
- `service_impl/src/absence.rs` — Implementation; Migrations-Logik darf hier KEINE neuen Methoden anhängen (Cutover-Service ist sein eigener Service).

#### `feature_flag` (Cutover-Trigger)

- `migrations/sqlite/20260501000000_add-feature-flag-table.sql` — Schema + Seed `('absence_range_source_active', 0, ...)` (Phase 2). **Wichtig:** Phase 4 SETZT diesen Flag in der atomaren Tx; nicht ein Re-Seed, sondern UPDATE.
- `service/src/feature_flag.rs:36` — `FeatureFlagService::is_enabled` und `set` API.
- `service_impl/src/reporting.rs:478` — Reporting-Switch via `feature_flag_service.is_enabled("absence_range_source_active", ...)` (Phase 2 D-Phase2-08-A). **Pattern für REST-Layer-Flag-Check** in `/extra-hours`-POST.

#### `employee_yearly_carryover` (Refresh-Ziel)

- `migrations/sqlite/20241215063132_add_employee-yearly-carryover.sql` + `20241231065409_add_employee-yearly-vacation-carryover.sql` — Schema: PK `(sales_person_id, year)`, Spalten `carryover_hours REAL`, `vacation INTEGER`. Cutover schreibt beide neu für Tupel im Refresh-Scope.
- `service/src/carryover.rs` — `CarryoverService` mit `get_carryover` + `set_carryover`. Plan-Phase fügt `rebuild_for_year(sp, year, ctx, tx)`-Methode hinzu (C-Phase4-02). **Service-Tier-Wechsel zu Business-Logic** möglicherweise nötig — explizit in Plan-Phase entscheiden.
- `service_impl/src/carryover.rs` — Implementation.
- `service_impl/src/test/carryover.rs` — Test-Patterns.

#### Permission/RBAC

- `service_impl/src/permission.rs` — `HR_PRIVILEGE` und Privileg-Konstanten-Konvention.
- `migrations/sqlite/20260105000000_app-toggles.sql:30` — Privileg-Insert-Pattern (`INSERT INTO privilege (name, update_process) VALUES ('toggle_admin', 'initial')`); analog für `cutover_admin` in der neuen Migration (D-Phase4-07 + C-Phase4-08).

#### REST + DTOs

- `rest/src/lib.rs` — `error_handler`-Wrapper; muss um Mapping `ServiceError::ExtraHoursCategoryDeprecated` → HTTP 403 erweitert werden.
- `rest/src/lib.rs` — `ApiDoc` (utoipa) erweitern um neue `/admin/cutover/*`-Routes + neue Response-Types.
- `rest-types/src/lib.rs` — `CutoverRunResultTO`, `CutoverGateDriftRowTO`, `ExtraHoursCategoryDeprecatedErrorTO` inline (Repo-Konvention seit Phase 1).
- `rest/src/feature_flag.rs` (falls existiert) oder neue Datei `rest/src/cutover.rs` — neue REST-Handler.

#### Service-Konstruktion

- `shifty_bin/src/main.rs` — DI-Verdrahtung. Neuer `CutoverServiceDependencies`-Block analog Phase-2/3-Pattern. Konstruktionsreihenfolge: Basic Services (`ExtraHoursService`, `CarryoverService`, `FeatureFlagService`, `EmployeeWorkDetailsService`) → Business-Logic (`AbsenceService` aus Phase 1+2+3, `CutoverService` neu).

#### Tests

- `shifty_bin/src/integration_test/` — Pattern für End-to-End-Tests (siehe `absence_period.rs`, `booking_absence_conflict.rs`). Phase 4 fügt `cutover.rs` hinzu (C-Phase4-04 + Pflicht-Tests aus Domain).
- `service_impl/src/test/` — Service-Layer-Mock-Tests (siehe `carryover.rs`, `absence.rs`).

#### Insta (OpenAPI-Snapshot)

- `rest/Cargo.toml` — neue dev-dependency `insta = { version = "1", features = ["json"] }` (D-Phase4-11).
- `rest/tests/openapi_snapshot.rs` — neuer Test (D-Phase4-11). Pin-File in `rest/tests/snapshots/openapi_snapshot__openapi.snap` (committed mit jj).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`AbsenceService::derive_hours_for_range`** (Phase 2) — single source of truth für Per-Tag-Stunden mit Cross-Category-Resolution. Cutover-Gate ruft diese Methode 1:1; keine Re-Implementierung der Logik. Garantiert dass das Gate dieselbe Berechnung fährt wie das Live-Reporting nach dem Flip.
- **`shifty_utils::DateRange::iter_days()` / `day_count()`** (Phase 1 D-16) — für Per-Tag-Iteration im Cluster-Algorithmus + im Carryover-Refresh.
- **`gen_service_impl!`-Macro** — DI-Pattern direkt für `CutoverServiceImpl` übertragbar.
- **`FeatureFlagService::set`** (Phase 2) — `UPDATE feature_flag SET enabled = ? WHERE key = ?` einzelne Methode; Cutover ruft sie für `'absence_range_source_active'` mit `true` innerhalb der atomaren Tx.
- **`ExtraHoursService::find_*`** — bestehende DAO-Read-Methoden zum Laden des Vacation/Sick/UnpaidLeave-Bestands pro Sales-Person (Plan-Phase prüft, ob ein Range-Read existiert oder ergänzt werden muss).
- **`EmployeeWorkDetailsService::find_*` (Plan-Phase verifiziert exakte Methoden)** — für Per-Tag-Vertragsstunden-Lookup im Cluster-Algorithmus + im Carryover-Refresh. Phase 2 hat den Lookup für `derive_hours_for_range` bereits etabliert.
- **`AbsenceService::create`** (Phase 3 returnt Wrapper) — Plan-Phase entscheidet, ob Cutover diesen Pfad oder einen direkten DAO-Insert nutzt (siehe Code-Templates oben). Vorgabe: direkter DAO-Insert für Performance + Privileg-Bypass; Service-Pfad nur falls Plan-Phase Hindernisse sieht.
- **`TransactionDao::use_transaction` / `commit` / `rollback`** — Standard-Pattern; Cutover hält die Tx über alle Phasen.

### Established Patterns

- **Layered Architecture**: REST → Service-Trait → DAO-Trait → SQLx. Cutover folgt strikt; neue REST-Handler in `rest/src/cutover.rs` (oder Erweiterung in bestehender Datei).
- **Soft-Delete-Konvention** (`WHERE deleted IS NULL`): gilt für `extra_hours` (D-Phase4-10 Soft-Delete der migrierten Rows). `absence_period_migration_source`, `absence_migration_quarantine`, `employee_yearly_carryover_pre_cutover_backup` brauchen KEIN soft-delete (Migrations-Audit-Tabellen, write-once).
- **Service-Tier-Konvention** (CLAUDE.md): `CutoverService` ist Business-Logic-Tier — konsumiert mehrere Domain-Services. Strikte Direction: Business-Logic → Basic, kein Cycle.
- **Migration-Seed-Pattern**: `INSERT INTO ... VALUES (...)` direkt in der `up.sql` (siehe `20260105000000_app-toggles.sql:30` für `toggle_admin`-Privileg); analog für `cutover_admin` (C-Phase4-08).
- **Snapshot-Versioning-Disziplin** (CLAUDE.md): Phase 4 ändert die Berechnungs-Logik NICHT (nur Flag-State). KEIN weiterer Bump nötig.
- **Wrapper-Result-Pattern** (Phase 3 D-Phase3-01): `CutoverRunResult` lebt im `CutoverService` (Business-Logic) — nicht im `ExtraHoursService` o.ä. (Basic).
- **Atomic-Tx-Pattern**: Phase 2 D-Phase2-10 hat den Snapshot-Bump in einem einzigen jj-Commit etabliert. Phase 4 erweitert das Pattern: alle MIG-01..05-Operationen in einer SQLite-Tx.

### Integration Points

- **`service_impl/src/reporting.rs:478`** — Reporting-Switch via `is_enabled("absence_range_source_active", ...)`. Phase 4 stellt sicher, dass dieser Flag korrekt gesetzt wird; **keine Änderung am Reporting-Code** — der Switch ist Phase-2-bereits-da.
- **`service_impl/src/extra_hours.rs::create`** (oder REST-Handler in `rest/src/extra_hours.rs`) — neuer Flag-Check vor dem DAO-Insert: `if feature_flag_service.is_enabled("absence_range_source_active", ctx, tx)? && matches!(category, Vacation | SickLeave | UnpaidLeave) { return Err(ServiceError::ExtraHoursCategoryDeprecated { category }) }`. Plan-Phase wählt zwischen Service-Layer-Check und REST-Handler-Check; Vorgabe: Service-Layer (konsistenter, Tests einfacher).
- **`service/src/lib.rs`** — `ServiceError`-Enum erweitern um `ExtraHoursCategoryDeprecated { category: extra_hours::ExtraHoursCategory }`. Mapping in `rest/src/lib.rs::error_handler` auf 403.
- **`shifty_bin/src/main.rs`** — `CutoverServiceDependencies`-Block; DI-Konstruktionsreihenfolge: alle Basic Services → AbsenceService (BL) → CutoverService (BL).
- **Neue Migrations-Files** (C-Phase4-01):
  - `<TS>_create-absence-migration-quarantine.sql` (D-Phase4-03)
  - `<TS+1>_create-absence-period-migration-source.sql` (D-Phase4-04)
  - `<TS+2>_create-employee-yearly-carryover-pre-cutover-backup.sql` (D-Phase4-13)
  - `<TS+3>_add-cutover-admin-privilege.sql` (D-Phase4-07 + C-Phase4-08)
- **`rest/Cargo.toml`** — dev-dependency `insta = { version = "1", features = ["json"] }` (D-Phase4-11).
- **`dao/Cargo.toml` + `dao_impl_sqlite/Cargo.toml`** — `uuid = { version = "...", features = ["v4"] }` (Wave-0-Hygiene D-Phase4-15).

### Risiken / Pitfalls für Phase 4

- **Pitfall (Atomarität-Bruch):** Wenn irgendein Schritt fail't, MUSS die ganze Tx rollback'n. Plan-Phase muss sicherstellen, dass kein Service-Call implizit eine eigene Tx öffnet (Standard-Pattern via `use_transaction(Some(tx))` reicht). Test: Mock einen Carryover-Refresh-Fehler und prüfe, dass `feature_flag.enabled` immer noch `false` ist.
- **Pitfall (Bit-Identitäts-Drift):** Wenn der Cutover-Gate sagt OK, aber das Live-Reporting nach Flag-Flip andere Werte zeigt, ist das ein Bug in der Gate-Berechnung — sie muss `sum(derive_hours_for_range)` exakt nachfahren, nicht eine Reimplementation. SC-5-Test (Per-MA-Per-Jahr-Per-Kategorie-Invariant in `shifty_bin/src/integration_test/cutover.rs`) deckt das ab.
- **Pitfall (Quarantäne-Übersehen):** Wenn die Heuristik (D-Phase4-02) zu permissiv ist (z.B. alle 7-Tage-Bestände konvertiert ohne Werktage-Check), wird der Gate in Production massiv fail'en und der Cutover ist nicht aufrufbar. Conservative bleibt richtig — lieber zu viel Quarantäne (HR fixt manuell) als zu wenig (HR kann Cutover gar nicht starten).
- **Pitfall (Flag-Race im Reporting):** Wenn ein Live-Reporting-Request während der Cutover-Tx läuft, könnte er (je nach SQLite-Isolation) entweder den alten oder neuen Flag-Wert sehen. Standard-SQLite-Isolation (SERIALIZABLE für die Cutover-Tx) sollte das verhindern; Plan-Phase sollte das aber explizit testen oder dokumentieren.
- **Pitfall (HR-Verwirrung über GET /extra-hours nach Cutover):** Migrierte Rows sind `deleted IS NOT NULL`, also unsichtbar in der Standard-Read-Query. Quarantäne-Rows bleiben sichtbar. HR könnte verwirrt sein "wo sind meine Vacation-Einträge geblieben?" — Plan-Phase sollte einen Hinweis im Frontend-Workstream-Brief vorbereiten oder einen Read-Endpunkt für migrierte Rows als Folgephase deferren.
- **Pitfall (Idempotenz vs. Quarantäne):** Re-Run der Migration nach manueller Quarantäne-Auflösung (HR hat die ambigue Row manuell zu einer AbsencePeriod gemacht und die ursprüngliche extra_hours-Row gelöscht) — die Quarantäne-Tabelle hat jetzt einen "stale" Eintrag mit verschwundener `extra_hours_id`. Plan-Phase entscheidet: Stale-Cleanup automatisch (DELETE WHERE NOT EXISTS extra_hours) oder manuell (HR löscht). Vorgabe: erstes Re-Run löscht stale Quarantäne-Rows automatisch.
- **Pitfall (Tx-Länge bei großem Bestand):** Bei N=10000+ Bestand-Rows könnte die Cutover-Tx mehrere Sekunden dauern. SQLite blockiert während dieser Zeit andere Schreib-Operationen (eine WriteLock-Tx). Plan-Phase sollte einen Smoke-Test mit großer Fixture durchführen, um die Worst-Case-Tx-Länge zu messen.
- **Pitfall (uuid-Feature-Drift):** Wave-0 (D-Phase4-15) MUSS vor Wave 1 grün sein, sonst spätere Standalone-Tests in dao/dao_impl_sqlite würden fail'en. Phase-4-Wave-Reihenfolge: Wave 0 (Hygiene + Migrations + Insta) → Wave 1 (CutoverService + Heuristik) → Wave 2 (Gate + REST) → Wave 3 (E2E-Tests).

</code_context>

<specifics>
## Specific Ideas

- **Diff-Report-JSON-Schema (verbatim für Plan-Phase):**
  ```json
  {
    "gate_run_id": "01234567-89ab-cdef-0123-456789abcdef",
    "run_at": "2026-05-03T14:23:00Z",
    "dry_run": true,
    "drift_threshold": 0.01,
    "total_drift_rows": 3,
    "drift": [
      {
        "sales_person_id": "uuid",
        "sales_person_name": "Erika Mustermann",
        "category": "Vacation",
        "year": 2024,
        "legacy_sum": 120.0,
        "derived_sum": 112.0,
        "drift": 8.0,
        "quarantined_extra_hours_count": 2,
        "quarantine_reasons": ["amount_below_contract_hours", "weekend_entry_with_workday_only_contract"]
      }
    ],
    "passed": false
  }
  ```

- **Quarantäne-Reason-Strings (Plan-Phase finalisiert):**
  - `"amount_below_contract_hours"` — extra_hours.amount < contract_hours_at(day) (z.B. 4h bei 8h-Vertrag)
  - `"amount_above_contract_hours"` — extra_hours.amount > contract_hours_at(day) (z.B. 10h bei 8h-Vertrag, Plus-Stunden)
  - `"contract_hours_zero_for_day"` — Wochenend-Eintrag bei Mo-Fr-Vertrag, oder Krankheit am Feiertag mit 0h-Anspruch
  - `"contract_not_active_at_date"` — kein gültiger EmployeeWorkDetails am date_time (z.B. vor Vertragsbeginn)
  - `"iso_53_week_gap"` — ISO-Woche-53-Edge-Case (Plan-Phase entscheidet ob das überhaupt möglich ist)

- **Permission-Pattern-Beispiel (analog Phase 2 D-Phase2-07):**
  ```rust
  // service_impl/src/cutover.rs (sketch)
  async fn run(&self, dry_run: bool, ctx: Authentication<Context>, tx: Option<Tx>) -> Result<CutoverRunResult, ServiceError> {
      self.permission_service
          .check_privilege(if dry_run { HR_PRIVILEGE } else { CUTOVER_ADMIN_PRIVILEGE }, &ctx)
          .await?;
      let tx = self.transaction_dao.use_transaction(tx).await?;
      // ... Migration + Gate + (if !dry_run && gate_passed) Cleanup + Carryover + Flag-Flip ...
      if dry_run || !gate_passed {
          self.transaction_dao.rollback(tx).await?;
      } else {
          self.transaction_dao.commit(tx).await?;
      }
      Ok(result)
  }
  ```

- **Cluster-Algorithmus-Skelett (verbatim für Plan-Phase, C-Phase4-03):**
  ```rust
  // 1. Read all extra_hours WHERE category IN (Vacation, SickLeave, UnpaidLeave) AND deleted IS NULL,
  //    sortiert by (sales_person_id, category, date_time)
  // 2. Iteriere; pro (sp, category) gruppe:
  //    a. Lade EmployeeWorkDetails für sp (alle aktiven Verträge)
  //    b. Iteriere Tag für Tag (sortiert)
  //    c. Bestimme contract_hours_at(day) per gültigen Vertrag am day
  //    d. WENN amount != contract_hours OR contract_hours == 0: quarantine, break cluster
  //    e. WENN amount == contract_hours UND day == previous_day + 1_workday: erweitere cluster
  //    f. WENN amount == contract_hours UND nicht-konsekutiv: schließe vorherigen Cluster ab, starte neuen
  // 3. Pro abgeschlossenem Cluster: INSERT INTO absence_period + N INSERTs INTO absence_period_migration_source
  ```

- **OpenAPI-Snapshot-Test-Skelett (verbatim für Plan-Phase, D-Phase4-11):**
  ```rust
  // rest/tests/openapi_snapshot.rs
  use rest::ApiDoc;
  use utoipa::OpenApi;

  #[test]
  fn openapi_snapshot_locks_full_api_surface() {
      let openapi = ApiDoc::openapi();
      // Sortierte JSON-Repräsentation, damit Diff-Order stabil ist
      insta::assert_json_snapshot!(openapi);
  }
  ```

- **Frontend-Migration-Hinweis** (für später, NICHT Phase-4-Scope): nach Phase-4-Cutover-Vollendung muss das Frontend (`shifty-dioxus`) den `/extra-hours`-POST für Vacation/SickLeave/UnpaidLeave durch `/absence-period`-POST ersetzen. Andernfalls erlebt der User unerklärliches `403`. Operations-Verantwortung: Hinweis im PR-Description, README-Update, separater Issue im Frontend-Repo.

</specifics>

<deferred>
## Deferred Ideas

- **REST-Endpunkt zum Auflisten der Quarantäne-Rows** (`GET /admin/cutover/quarantine`) — bei Bedarf in Folgephase als HR-Admin-Surface, damit HR die ambiguen Einträge browse + manuell auflösen kann.
- **Auto-Cleanup der Quarantäne-Rows** — nicht in Phase 4. HR muss manuell entscheiden: entweder die ursprüngliche extra_hours-Row akzeptieren als unveränderten Bestand, oder eine `/absence-period`-POST anlegen und die alte Row manuell soft-deleten.
- **Restore-Endpunkt aus `employee_yearly_carryover_pre_cutover_backup`** — bei Bedarf in Folgephase. Phase 4 schreibt nur das Backup; Restore ist manuell (SQL-Pfad oder Folgephase-Endpunkt).
- **Frontend-Migration der `/extra-hours`-POST-Calls auf `/absence-period`** — Frontend-Workstream-Verantwortung. Dauert separat.
- **Bulk-Carryover-Rebuild-Endpoint für Operations** — bei Bedarf in Folgephase als HR-Admin-Surface (z.B. nach manuellen Daten-Korrekturen).
- **Read-Compat-Shim für `/extra-hours`-Vacation-POSTs nach Cutover** — bewusst NICHT Phase 4 (D-Phase4-09 wählt Hard-403). Falls Frontend-Migration sich stark verzögert, könnte eine Folgephase einen kurzen Shim einführen, der intern `/absence-period` aufruft.
- **Audit-Trail für `feature_flag`-Flips** (wer hat wann geflippt) — schon in Phase 2 deferred; nicht in Phase 4.
- **REST-Endpunkte für `feature_flag` mit OpenAPI** — falls Frontend einen Admin-Screen bekommt; nicht in dieser Iteration.
- **Migration weiterer ExtraHours-Kategorien zu range-basiert** (z.B. Holiday als Range-Kategorie) — in v1 explizit out of scope; Phase 4 ist nur für Vacation/SickLeave/UnpaidLeave.
- **CarryoverService-Tier-Wechsel** (Basic → Business-Logic) — entstanden aus C-Phase4-02. Falls Plan-Phase entscheidet, dass `rebuild_for_year` als neuer Service `CarryoverRebuildService` (Business-Logic, separat) gebaut wird, bleibt `CarryoverService` strikt basic. Alternative: existing `CarryoverService` wird Business-Logic — Tier-Wechsel sollte explizit dokumentiert werden.
- **Quarantäne-Reason-i18n** — Frontend braucht Übersetzungen der `reason`-Strings; Frontend-Workstream-Verantwortung.

</deferred>

---

*Phase: 4-Migration-Cutover*
*Context gathered: 2026-05-03*
