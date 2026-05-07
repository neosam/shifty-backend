# Phase 2: Reporting Integration & Snapshot Versioning - Context

**Gathered:** 2026-05-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Der Reporting-Pfad bekommt eine **zweite Quelle** für Vacation/Sick/UnpaidLeave-Stunden — Per-Tag-Derivation aus `AbsencePeriod` gegen den am jeweiligen Tag gültigen `EmployeeWorkDetails`-Vertrag, mit korrekter Feiertags-Orthogonalität (0 Urlaubsstunden am Feiertag), hinter Feature-Flag `absence_range_source_active` versteckt. `CURRENT_SNAPSHOT_SCHEMA_VERSION` wird gleichzeitig 2 → 3 gebumpt — im **selben Commit** wie der Reporting-Switch (CLAUDE.md-Pflicht).

**In Scope (Phase 2):**
- `AbsenceService::derive_hours_for_range(from, to, sales_person_id) -> BTreeMap<Date, AbsenceCategoryHours>` — single source of truth für Cross-Category-konflikt-aufgelöste Tagesstunden
- Cross-Category-Resolver mit deterministischer Priorität `SickLeave > Vacation > UnpaidLeave` (BUrlG §9-konform)
- Feiertags-0-Stunden-Auflösung pro Tag (`special_day`-aware)
- Vertragswechsel-mid-range-Korrektheit (per-Tag gegen am Tag gültigen `EmployeeWorkDetails`)
- Neuer `BillingPeriodValueType::UnpaidLeave`-Variante in `service/src/billing_period.rs`
- Snapshot-Builder schreibt `UnpaidLeave` aus `report_delta.unpaid_leave_hours`
- Snapshot-Read-Pfad: v2-Snapshots ohne `unpaid_leave`-Spalte werden weiterhin gelesen (fehlend = 0.0)
- `feature_flag(key TEXT PK, enabled BOOLEAN NOT NULL DEFAULT 0, description TEXT, ...)`-Tabelle (neue Migration)
- `FeatureFlagService` (Trait + Impl + Mock + DI) mit `feature_flag_admin`-Privileg
- Migration seedet `('absence_range_source_active', 0, ...)` als disabled
- Reporting-Switch: wenn Flag an, wechselt `ReportingService` für Vacation/Sick/UnpaidLeave atomar zur AbsencePeriod-derived-Quelle
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2 → 3 im selben Commit
- Hybrid-Locking-Test (`service_impl/src/test/billing_period_report.rs`):
  - **Pin-Map** über alle 12 `BillingPeriodValueType`-Varianten mit konkreten Werten aus deterministischer Fixture
  - **Compiler-Exhaustive-Match** über alle Varianten — neue Variante zwingt zur Test-Anpassung
- Bit-Identitäts-Test bei Flag = aus (Snapshot vor und nach Phase-2-Code = identisch)

**Strikt nicht in Scope (Phase 2):**
- Booking-Konflikt-Detection / Forward-/Reverse-Warnings (Phase 3)
- Shift-Plan-Integration (Phase 3)
- Migrations-Heuristik aus `ExtraHours` (Phase 4)
- **Phase-4-Cutover-Gate** (`MIG-02`: pro `(sales_person, kategorie)` `sum(derive_hours_for_range) == sum(extra_hours_legacy)`) — anderer Mechanismus, andere Phase
- Atomares Flippen des Flags (Phase 4 in derselben Tx wie MIG-01/MIG-04)
- Carryover-Refresh (Phase 4)
- REST-Endpoints für `FeatureFlagService` (Service-Layer reicht für Phase 2; REST kann später nachgezogen werden)
- Frontend (separater Workstream)

</domain>

<decisions>
## Implementation Decisions

### Cross-Category-Auflösung im Reporting (BUrlG §9)

- **D-Phase2-01:** **Sick gewinnt bei Vacation∩SickLeave (BUrlG §9-konform).** An jedem Tag, an dem sowohl eine `Vacation`- als auch eine `SickLeave`-AbsencePeriod aktiv ist, werden die Vertragsstunden ausschließlich der `SickLeave`-Kategorie zugerechnet. `Vacation` produziert für diesen Tag 0 Stunden — Urlaub bleibt unverbraucht. Begründung: §9 BUrlG („Erkrankung während des Urlaubs") fordert, dass Krankheitstage nicht vom Urlaubskonto abgezogen werden.
- **D-Phase2-02:** **Cross-Category-Resolver lebt in `AbsenceService::derive_hours_for_range`.** Single source of truth: der Service bekommt alle 3 Kategorien gleichzeitig und gibt eine **bereits konflikt-aufgelöste** Tagesliste zurück (z.B. `BTreeMap<Date, AbsenceCategoryHours>` mit pro Tag genau einer dominanten Kategorie + ihrer Stundenzahl). Phase-4-Migrations-Gate wird **dieselbe** Logik fahren — keine Duplizierung.
- **D-Phase2-03:** **Deterministische Prioritäts-Reihenfolge: `SickLeave > Vacation > UnpaidLeave`.** Über alle 3 Pair-Kombinationen anwendbar. UnpaidLeave verliert immer (Mitarbeiter „nicht-bezahlt frei" — wenn er stattdessen Urlaub nimmt, gewinnt Vacation; wenn er krank wird, gewinnt Sick). Deterministisch und re-runnable für Phase-4-Validation-Gate.

### UnpaidLeave Snapshot-Mapping

- **D-Phase2-04:** **Neuer `BillingPeriodValueType::UnpaidLeave`-Variante** in `service/src/billing_period.rs`. Dazu:
  - Neuer Match-Arm in `as_str()` → `"unpaid_leave"`
  - Neuer Match-Arm in `FromStr::from_str` → `"unpaid_leave" => Ok(UnpaidLeave)`
  - Snapshot-Builder (`service_impl/src/billing_period_report.rs`) schreibt einen neuen `BillingPeriodValueType::UnpaidLeave`-Eintrag aus `report_delta.unpaid_leave_hours` (analog zu `VacationHours` aktuell)
  - Schließt die bestehende Snapshot-Lücke: heute fließt `unpaid_leave_hours` nur indirekt in `total_absence_days_per_week` rein, ist aber nicht persistiert
  - Phase-4-Cutover-Gate kann pro Kategorie inkl. UnpaidLeave validieren (MIG-03 sauber abgedeckt)
- **D-Phase2-05:** **Snapshot-Read v2 → fehlende `unpaid_leave`-Spalte wird als 0.0 interpretiert** (versions-agnostic). Begründung: vor Phase 2 wurde UnpaidLeave nicht persistiert — „fehlender Eintrag" ist semantisch korrekt „unbekannt = 0". Reader bleibt einheitlich; kein neuer versions-aware Code-Pfad. Erfüllt SC-5 ("v2-Snapshots bleiben lesbar") trivial.

### Feature-Flag-Mechanik

- **D-Phase2-06:** **Eigene generische Tabelle `feature_flag(key TEXT PK, enabled BOOLEAN NOT NULL DEFAULT 0, description TEXT, ...)`.** Bewusst **nicht** Reuse von `ToggleService`/`toggle`-Tabelle — semantische Trennung: Feature-Flags sind **Architektur-/Migrations**-Schalter, Toggles sind User-Toggles. Generisches Key-Value-Schema future-proof für weitere Architektur-Flags. Phase 2 seedet `('absence_range_source_active', 0, 'Use AbsencePeriod-derived hours instead of ExtraHours for Vacation/Sick/UnpaidLeave reporting')` als disabled. Phase 4 flippt via `UPDATE feature_flag SET enabled = 1 WHERE key = 'absence_range_source_active'` in derselben Tx wie MIG-01/MIG-04 (Phase-4-SC-4 abgedeckt).
- **D-Phase2-07:** **Eigener `FeatureFlagService` (Trait + Impl + Mock + DI) mit eigenem Privileg `feature_flag_admin`.**
  - API: `is_enabled(key, ctx, tx) -> Result<bool, ServiceError>`, `set(key, value, ctx, tx) -> Result<(), ServiceError>` (HR-Permission via neues `feature_flag_admin`-Privileg, nicht reuse `toggle_admin`)
  - REST-Endpoints sind **out of scope** für Phase 2 — Service-Layer reicht; `ReportingService` ruft intern `is_enabled` auf, Phase 4 ruft `set` aus der Migration
  - Test-Setup: `MockFeatureFlagService` erlaubt `expect_is_enabled().returning(|_, _, _| Ok(false))` für bit-identitäts-Tests bei Flag = aus

### Reporting-Switch

- **D-Phase2-08-A:** Der Switch-Mechanismus lebt in **`ReportingService`** (nicht im DAO). Pro Report-Range:
  1. `feature_flag_service.is_enabled("absence_range_source_active", ...)` einmal lesen
  2. Wenn `false`: bestehender ExtraHours-Pfad für Vacation/Sick/UnpaidLeave (heute = always)
  3. Wenn `true`: `absence_service.derive_hours_for_range(...)` für Vacation/Sick/UnpaidLeave; ExtraHours-Quelle für diese 3 Kategorien wird komplett ignoriert (atomarer Switch — keine Mischung)
  - ExtraWork, Volunteer, Holiday, CustomExtraHours bleiben **immer** ExtraHours-Quelle (sie sind nicht range-basiert)
  - `BillingPeriodReportService` ruft `ReportingService` wie bisher; der Switch ist unter dem ReportingService-API verborgen

### Locking-Test (SNAP-02)

- **D-Phase2-08-B:** **Hybrid Locking-Test** in `service_impl/src/test/billing_period_report.rs`:
  1. **Pin-Map-Test** (`test_snapshot_v3_pinned_values`): deterministische Fixture (1 Sales-Person, 1 Vertrag, definierte ExtraHours, definierte AbsencePeriods); pro `BillingPeriodValueType` ein `assert_eq!` auf den erwarteten konkreten Stunden-/Tage-Wert. Bei Drift: `cargo test` rot.
  2. **Compiler-Exhaustive-Match-Test** (`test_billing_period_value_type_surface_locked`): `match value_type { Overall => …, Balance => …, … }` über **alle** Varianten. Bei neuer Enum-Variante zwingt der Compiler zur Test-Anpassung.
  - **Header-Kommentar** auf beiden Tests: *"Wenn dieser Test fehlschlägt: bist du sicher, dass du `CURRENT_SNAPSHOT_SCHEMA_VERSION` bumpen wolltest? Siehe CLAUDE.md → 'Billing Period Snapshot Schema Versioning'."*
- **D-Phase2-09:** **Pin-Map-Scope: alle 12 `BillingPeriodValueType`-Varianten** (Overall, Balance, ExpectedHours, ExtraWork, VacationHours, SickLeave, **UnpaidLeave (neu)**, Holiday, Volunteer, VacationDays, VacationEntitlement, CustomExtraHours). Universeller Drift-Schutz — auch Refactors außerhalb des Phase-2-Scopes brechen den Test. Das ist der intended pain.

### Snapshot-Bump

- **D-Phase2-10:** **`CURRENT_SNAPSHOT_SCHEMA_VERSION = 2 → 3`** in `service_impl/src/billing_period_report.rs:37`. **Im selben Commit** wie:
  - Neuer `BillingPeriodValueType::UnpaidLeave` (D-Phase2-04)
  - Reporting-Switch zu `derive_hours_for_range` (D-Phase2-08-A)
  - Pin-Map mit neuen Phase-3-Werten (D-Phase2-08-B)
  - Migration für `feature_flag` + Seed (D-Phase2-06)
  - Per `CLAUDE.md` Pflicht: Bump erfolgt im selben Commit wie Input-Set-Änderung der Snapshot-Berechnung.

### Claude's Discretion (Plan-Phase entscheidet)

- **C-Phase2-01:** **`derive_hours_for_range`-Return-Type-Detail.** Vorschlag: `Result<BTreeMap<time::Date, ResolvedAbsence>, ServiceError>` mit `struct ResolvedAbsence { category: AbsenceCategory, hours: f32 }`. Plan-Phase darf alternativ `Vec<(Date, AbsenceCategory, f32)>` oder pro-Kategorie-Map wählen, falls SQL-Pattern oder Performance es nahelegt. **Constraint:** der Resolver-Output muss bereits konflikt-aufgelöst sein (D-Phase2-02).
- **C-Phase2-02:** **Feiertags-0-Auflösung** — konkrete Wochenend-Logik. Vorgabe: an Tagen, die in `special_day` als Feiertag markiert sind, **0 Urlaubsstunden**. Frage offen: gilt dasselbe für Wochenenden (Sa/So) generell? Plan-Phase prüft `EmployeeWorkDetails.workdays`-Feld und setzt für Tage, an denen der Vertrag 0 Stunden vorsieht (`workdays_per_week`-Inversion), auf 0. **Pragma:** "Vertragsstunden des Tages = 0 ⇒ Absence-Stunden des Tages = 0" — fällt aus dem Per-Tag-Vertrag-Lookup natürlich raus.
- **C-Phase2-03:** **`FeatureFlagService`-DAO-Surface**. Plan-Phase entscheidet zwischen einem `FeatureFlagDao`-Trait + `FeatureFlagDaoImpl` (vollständiges DAO-Pattern, analog `ToggleDao`) und einem schmaleren `FeatureFlagDao` mit nur 2 Methoden (`get(key)` / `set(key, value)`). Vorgabe: schmaleres DAO reicht für Phase 2; Erweiterung in Phase 5+ wenn nötig.
- **C-Phase2-04:** **Pin-Map-Fixture** — konkrete Werte (Vertragsstunden, Datum-Ranges, ExtraHours, AbsencePeriods) entscheidet Plan-Phase. Vorgabe: 1 Sales-Person, 8h/Tag-Vertrag, 5 Werktage, 1-Wochen-Range mit 1 Vacation-AbsencePeriod über 3 Werktage + 1 Sick-Tag der mit Vacation überlappt (BUrlG §9-Test) + 1 ExtraWork-Eintrag. Die Werte folgen aus den Inputs deterministisch.
- **C-Phase2-05:** **DI-Reihenfolge für `FeatureFlagService`** — mechanische Erweiterung in `shifty_bin/src/main.rs`; Reporting bekommt den `FeatureFlagService` als neue Dependency.
- **C-Phase2-06:** **Naming des Toggle-Keys**: `absence_range_source_active` als Schlüsselstring (snake_case, ohne Punkt — der `absence.range_source_active`-Stil aus ROADMAP.md ist Prosa-Notation, nicht der Storage-Key). Plan-Phase darf das Naming feinjustieren, sollte aber konsistent zwischen Migration-Seed, Service-Calls und Tests bleiben.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project-Level Spezifikationen
- `.planning/ROADMAP.md` § Phase 2 — Goal, Depends-on Phase 1, Success-Criteria 1-5, Discuss-Carry-Overs.
- `.planning/STATE.md` — Aktuelle Position, Architektur-Decisions (Hybrid materialize-on-snapshot / derive-on-read; Bump 2→3 im selben Commit).
- `shifty-backend/CLAUDE.md` § "Billing Period Snapshot Schema Versioning" — die einzige Quelle für die Bump-Pflicht; Trigger-Bedingungen; Begründung warum SNAP-02 build-time-fail sein muss.
- `shifty-backend/CLAUDE.local.md` — VCS via `jj` (alle Commits manuell durch User; GSD `commit_docs: false`); NixOS-Hinweise (`nix-shell` für `sqlx-cli`).
- `~/.claude/CLAUDE.md` — Tests sind Pflicht für jede Änderung.

### Vorphase-Outputs
- `.planning/phases/01-absence-domain-foundation/01-CONTEXT.md` — D-01..D-17 als Fundament (insbesondere D-02 Kategorien-Liste, D-12 Self-Overlap kategorie-scoped, D-16 DateRange-API mit `iter_days`/`day_count` schon verfügbar).
- `.planning/phases/01-absence-domain-foundation/01-PATTERNS.md` — Pattern-Mapping aus Phase 1 (Code-Templates für AbsenceService bestehen).
- `.planning/phases/01-absence-domain-foundation/01-RESEARCH.md` — Range-Overlap-SQL-Idiom; AbsenceService-Trait-Shape; Stack-Begründung.
- `.planning/phases/01-absence-domain-foundation/01-VALIDATION.md` — Pinned Discretion Items aus Plan-Phase 1 (z.B. TO inline statt eigene Datei).

### Code-Templates für Phase 2
- `service/src/billing_period.rs:36-92` — `BillingPeriodValueType`-Enum + `as_str()`/`FromStr` (UnpaidLeave-Variante hier hinzufügen).
- `service_impl/src/billing_period_report.rs:37` — `CURRENT_SNAPSHOT_SCHEMA_VERSION` (Bump-Stelle).
- `service_impl/src/billing_period_report.rs:109-228` — Snapshot-Builder (`build_billing_period_report_for_sales_person`); UnpaidLeave-Insert hier hinzufügen.
- `service_impl/src/billing_period_report.rs:300-324` — `build_and_persist_billing_period_report` (schreibt `snapshot_schema_version`).
- `service_impl/src/test/billing_period_report.rs:1090-1149` — bestehende Tests für Schema-Version-Stamp; Locking-Test ergänzt diese Datei.
- `service/src/reporting.rs:88-180` — `EmployeeReport`-Struktur mit `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours`/`holiday_hours`-Feldern (relevant für Switch-Mechanik).
- `service_impl/src/reporting.rs:285-378, 563-739, 958-963` — bestehende ExtraHours-Aggregations-Pfade (Switch-Stellen für `derive_hours_for_range`).
- `service/src/toggle.rs` + `service_impl/src/toggle.rs` + `dao/src/toggle.rs` + `dao_impl_sqlite/src/toggle.rs` — **Template** für `FeatureFlagService` (parallele Struktur, eigenes Privileg).
- `migrations/sqlite/20260105000000_app-toggles.sql` — Schema-Template für `feature_flag`-Tabelle (analoges Pattern).
- `service/src/employee_work_details.rs` (zu lesen in Plan-Phase) — `EmployeeWorkDetails` mit `workdays_per_week`/`hours_per_week` (für Per-Tag-Vertragsstunden-Lookup in `derive_hours_for_range`).
- `service/src/special_day.rs` (zu lesen in Plan-Phase) — `SpecialDay`-Struktur (für Feiertags-0-Auflösung).
- `service/src/absence.rs` + `service_impl/src/absence.rs` (aus Phase 1) — `AbsenceService` wird hier um `derive_hours_for_range` erweitert.
- `service/src/lib.rs:121-128` — `ServiceError`-Surface (potentielle neue Variante `FeatureFlagNotFound(Arc<str>)` für `is_enabled`-Misses; Plan-Phase entscheidet).
- `shifty_bin/src/main.rs` — DI-Verdrahtung; `FeatureFlagService` als neue Dependency in `ReportingService` und Phase-4-Migrations-Code.
- `shifty-utils/src/date_range.rs` (aus Phase 1) — `DateRange::iter_days()` für Per-Tag-Iteration in `derive_hours_for_range`.

### Permission/RBAC
- `service_impl/src/permission.rs` — `HR_PRIVILEGE` und Privileg-Konstanten-Konvention.
- `migrations/sqlite/20260105000000_app-toggles.sql:30` — Privileg-Insert-Pattern (`INSERT INTO privilege (name, update_process) VALUES ('toggle_admin', 'initial')`); analog für `feature_flag_admin` in der neuen Migration.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`AbsenceService` aus Phase 1** (`service/src/absence.rs`) — wird um `derive_hours_for_range`-Methode erweitert; nutzt bestehende DAO-Methoden (`find_by_sales_person_in_range` oder neu) und `DateRange::iter_days`.
- **`shifty_utils::DateRange::iter_days()`/`day_count()`** (aus Phase-1-D-16/17) — bereits da, ready to use für Per-Tag-Iteration.
- **`gen_service_impl!`-Macro** — DI-Pattern für `FeatureFlagServiceImpl` direkt übertragbar aus `service_impl/src/toggle.rs:gen_service_impl!`-Block.
- **`ToggleService`-Architektur** (`service/src/toggle.rs`, `service_impl/src/toggle.rs`, `dao_impl_sqlite/src/toggle.rs`) — als **strukturelles Vorbild** (nicht reuse). Schema-Template, Service-Trait-Shape, DAO-Trait-Shape, Privileg-Insert-Pattern, Test-Setup-Pattern alle 1:1 spiegelbar.
- **`MockToggleService` via `#[automock]`** — Mock-Pattern direkt für `MockFeatureFlagService` adaptierbar.
- **`special_day`-Tabelle/Service** (Migration `20241020064536_add-special-day-table.sql`; `service/src/special_day.rs`) — Read-API für Feiertags-Lookup; Plan-Phase prüft, ob ein eigener Helper für „is_holiday(date)" nötig ist oder ob `SpecialDayService::find_by_date` ausreicht.
- **`EmployeeWorkDetailsService`** (`service/src/employee_work_details.rs`) — liefert den am Tag gültigen Vertrag mit `find_active_for(sales_person_id, date)` (oder vergleichbar) — für Per-Tag-Vertragsstunden-Lookup.

### Established Patterns
- **Layered Architecture**: REST → Service-Trait → DAO-Trait → SQLx. `FeatureFlagService` folgt dem; REST ist out-of-scope für Phase 2.
- **Soft-Delete-Konvention** (`WHERE deleted IS NULL`) gilt für `feature_flag` **nicht** — Feature-Flags werden nicht gesoft-deletet, sondern direkt gelöscht oder enabled/disabled.
- **Migration-Seed-Pattern**: `INSERT INTO ... VALUES (...)` direkt in der `up.sql` (siehe `20260105000000_app-toggles.sql:30` für `toggle_admin`-Privileg).
- **Snapshot-Schema-Versions-Disziplin** (`CLAUDE.md`): jeder Bump-Trigger erfordert Bump im selben Commit. Phase-2-Trigger:
  - Neuer `value_type` (UnpaidLeave): **Trigger 1** (additiv neuer value_type)
  - Geänderte Berechnungs-Inputs für VacationHours/SickLeave/UnpaidLeave (von ExtraHours auf AbsencePeriod-derived hinter Flag): **Trigger 4** (Input-Set-Änderung)
- **Hybrid Materialize-vs-Derive**: Live-Reports derive-on-read über `ReportingService::get_report_for_employee_range`; Snapshots werden bei `build_and_persist_billing_period_report` einmal materialisiert. Phase 2 ändert die **Quelle** der Berechnung (hinter Flag), nicht das Hybrid-Modell.

### Integration Points
- **`service/src/billing_period.rs:36-47`**: `BillingPeriodValueType`-Enum erweitern um `UnpaidLeave`-Variante (+ `as_str` + `FromStr`).
- **`service_impl/src/billing_period_report.rs:37`**: `CURRENT_SNAPSHOT_SCHEMA_VERSION = 2` → `= 3`.
- **`service_impl/src/billing_period_report.rs:155-163`**: nach dem `SickLeave`-Insert einen analogen `UnpaidLeave`-Insert aus `report_delta.unpaid_leave_hours`.
- **`service_impl/src/reporting.rs`**: alle Stellen, an denen `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours` aus `extra_hours` aggregiert werden — hier den Switch via `feature_flag_service.is_enabled(...)` einbauen.
- **`service/src/absence.rs`** (aus Phase 1): neue Trait-Methode `derive_hours_for_range(from, to, sales_person_id, ctx, tx)`.
- **`service_impl/src/absence.rs`** (aus Phase 1): Implementation der neuen Methode mit Cross-Category-Resolver + Per-Tag-Vertrags-Lookup + Feiertags-0-Auflösung.
- **`shifty_bin/src/main.rs`**: `FeatureFlagServiceDependencies`-Block analog `ToggleServiceDependencies`; `ReportingService` und `AbsenceService` bekommen `FeatureFlagService` als Dep injiziert (nur `ReportingService` ruft `is_enabled`; `AbsenceService` braucht ihn nicht direkt — `derive_hours_for_range` ist Flag-agnostic).
- **Neue Migration `migrations/sqlite/<timestamp>_add-feature-flag-table.sql`** mit:
  - `CREATE TABLE feature_flag(key TEXT PK, enabled INTEGER NOT NULL DEFAULT 0, description TEXT, update_timestamp TEXT, update_process TEXT NOT NULL)`
  - `INSERT INTO feature_flag (key, enabled, description, update_process) VALUES ('absence_range_source_active', 0, '...', 'phase-2-migration')`
  - `INSERT INTO privilege (name, update_process) VALUES ('feature_flag_admin', 'initial')`

### Risiken / Pitfalls für Phase 2
- **Pitfall (Snapshot-Versioning, CLAUDE.md):** Bump und Berechnungs-Änderung MÜSSEN im selben Commit landen. `D-Phase2-10` und `D-Phase2-08-B` zusammen erzwingen das.
- **Pitfall (Bit-Identität bei Flag = aus, SC-2):** Wenn der Flag aus ist, MÜSSEN existierende Snapshots/Reports bit-identische Werte produzieren. Plan-Phase-Test: Snapshot über bekanntes Fixture vor und nach dem Phase-2-Patch (mit Flag = aus) muss byte-identisch sein. Risiko: das Schema-Bump wird trotzdem geschrieben, also ist der Snapshot-Output **nicht** bit-identisch — nur die `values`-Map ist es. Bit-Identitäts-Test muss spezifisch nur die `values`-Map vergleichen, nicht das `snapshot_schema_version`-Feld.
- **Pitfall (Vertragswechsel mid-range):** Per-Tag-Lookup gegen `EmployeeWorkDetails`. Wenn jemand am 15.06. den Vertrag wechselt von 8h auf 4h, muss `derive_hours_for_range(2024-06-01, 2024-06-30)` für 14.06. = 8h liefern, für 16.06. = 4h. Plan-Phase prüft, ob `EmployeeWorkDetailsService::find_active_for(date)` (oder Äquivalent) existiert.
- **Pitfall (Resolver-Performance):** `derive_hours_for_range` ruft pro Tag `EmployeeWorkDetailsService::find_active_for` und `SpecialDayService::is_holiday`. Bei einem Jahres-Range = 365 Calls — Plan-Phase entscheidet, ob batch-Lookup nötig ist (`find_all_active_in_range` + `find_holidays_in_range`) oder ob per-Tag mit Caching reicht. Vorgabe: erst messen, dann optimieren.
- **Pitfall (FeatureFlag-Caching):** `is_enabled` wird pro Report **einmal** gelesen, nicht pro Tag. Plan-Phase muss sicherstellen, dass der Wert für die Dauer eines Reports konsistent ist (kein Race mit gleichzeitigem `set`).

</code_context>

<specifics>
## Specific Ideas

- **Pin-Map-Fixture-Vorschlag:** 1 Sales-Person, 1 Vertrag (`8h/Tag, Mo-Fr`), Range `2024-06-03 (Mo) bis 2024-06-09 (So)`:
  - 1 Vacation-AbsencePeriod `2024-06-03..2024-06-05` (3 Werktage Mo-Mi)
  - 1 SickLeave-AbsencePeriod `2024-06-04..2024-06-04` (1 Tag Di — überlappt mit Vacation → BUrlG-§9-Test: Di zählt nur als Sick, Vacation produziert für Di 0h)
  - 1 ExtraHours-Eintrag `2024-06-06: ExtraWork +2h`
  - Erwartete Werte (Flag = an): VacationHours.delta = 16h (Mo+Mi), SickLeave.delta = 8h (Di), UnpaidLeave.delta = 0h, ExtraWork.delta = 2h, Holiday.delta = 0h, ExpectedHours.delta = 40h, Balance.delta = -14h, Overall.delta = 26h, ...
  - Der **gleiche** Test mit Flag = aus (Bit-Identitäts-Test) prüft, dass die Werte denen der heutigen ExtraHours-Quelle entsprechen.
- **`description`-Feld in `feature_flag`:** Einlogger-freundliche Erklärung für HR-User: `"When ON, range-based AbsencePeriods are the source of truth for Vacation/Sick/UnpaidLeave hours instead of ExtraHours. Flip atomically with Phase-4 migration; do NOT flip manually."`
- **Locking-Test-Header-Kommentar (verbatim für Plan-Phase):**
  ```rust
  /// LOCKING TEST — DO NOT NAIVELY UPDATE.
  ///
  /// If this test fails after a code change:
  ///   - Did you intentionally change the snapshot computation?
  ///   - If yes, you MUST also bump CURRENT_SNAPSHOT_SCHEMA_VERSION
  ///     in service_impl/src/billing_period_report.rs.
  ///   - See CLAUDE.md § "Billing Period Snapshot Schema Versioning"
  ///     for the bump-trigger rules.
  ```
- **Compiler-Exhaustive-Match-Skelett (für Plan-Phase):**
  ```rust
  fn ensure_value_type_surface_locked(value_type: &BillingPeriodValueType) {
      match value_type {
          BillingPeriodValueType::Overall => {}
          BillingPeriodValueType::Balance => {}
          BillingPeriodValueType::ExpectedHours => {}
          BillingPeriodValueType::ExtraWork => {}
          BillingPeriodValueType::VacationHours => {}
          BillingPeriodValueType::SickLeave => {}
          BillingPeriodValueType::UnpaidLeave => {} // NEU in v3
          BillingPeriodValueType::Holiday => {}
          BillingPeriodValueType::Volunteer => {}
          BillingPeriodValueType::VacationDays => {}
          BillingPeriodValueType::VacationEntitlement => {}
          BillingPeriodValueType::CustomExtraHours(_) => {}
          // Wenn der Compiler hier eine fehlende Variante meldet:
          // bist du sicher, dass du nicht CURRENT_SNAPSHOT_SCHEMA_VERSION bumpen wolltest?
      }
  }
  ```

</specifics>

<deferred>
## Deferred Ideas

- **REST-Endpoints für `FeatureFlagService`** — Phase 5+ oder bei Bedarf. Phase 2 reicht der Service-Layer.
- **`feature_flag`-Audit-Trail** (wer hat wann geflippt) — nicht in Phase 2; bei Bedarf später als zusätzliche Spalten oder Audit-Tabelle.
- **Weitere Architektur-Feature-Flags** — die `feature_flag`-Tabelle ist generisch; zukünftige Flags können einfach gesedet werden.
- **Phase-4-Cutover-Gate (MIG-02/MIG-03)** — pro `(sales_person, kategorie)` `sum(derive_hours_for_range) == sum(extra_hours_legacy)`. Anderer Mechanismus, Phase 4. Phase 2 stellt aber sicher, dass `derive_hours_for_range` die Logik-Surface bietet, die Phase 4 dafür aufrufen kann.
- **Atomares Feature-Flag-Flippen in derselben Tx wie MIG-01/MIG-04** — Phase 4. Phase 2 stellt das Schema und die `set`-API bereit; das atomare Flippen findet in Phase 4 statt.
- **Carryover-Refresh nach Flag-Flip** — Phase 4 (MIG-04).
- **REST-Endpoints für `feature_flag` mit OpenAPI** — falls Frontend einen Admin-Screen bekommt; nicht in dieser Iteration.
- **Insta-Snapshot-Tooling** — bewusst nicht eingeführt; Pin-Map + Compiler-Exhaustive-Match reicht. Könnte Future-Phase sein, falls Snapshot-Surface stark wächst.
- **Holiday-Anrechnung als 0-Stunden-Nuance** — die genaue Frage „was passiert wenn AbsencePeriod über einen Feiertag läuft, der ein Wochentag ist" ist durch C-Phase2-02 und das „Vertragsstunden des Tages = 0 ⇒ 0" Pragma abgedeckt; Plan-Phase finalisiert die Implementation.
- **`derive_hours_for_range` für andere Konsumenten als Reporting** (z.B. Phase 3 Booking-Konflikt-Detection) — Phase 3 nutzt `find_overlapping`-Style-API, nicht `derive_hours_for_range`. Kein Konflikt.

</deferred>

---

*Phase: 2-Reporting-Integration-Snapshot-Versioning*
*Context gathered: 2026-05-01*
