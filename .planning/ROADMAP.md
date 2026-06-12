# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ◆ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8 (✓), 8.1 (⊘ superseded), 8.2–8.3 (✓), 8.4–8.6 (neues Koexistenz-Modell), 9–13 (active, started 2026-05-07)

## Phases

### v1.3 Frontend Abwesenheiten + UI-Closure-Restanten (active)

- [x] **Phase 8: Absence-CRUD-Page Foundation** (Frontend) — completed 2026-05-08, 9 plans, code-side fertig, int-UAT deferred zu Phase 8.1
  Neue Top-Level-Route `absences` mit CRUD gegen `/absence-period`, HR/Employee-Sicht aus Auth-Context, Form (Range-Picker + Kategorie + Description) und Forward-Warnings-Anzeige.
  Requirements: FUI-A-01, FUI-A-02, FUI-A-03, FUI-A-04
  Success Criteria:
  1. Route `/absences` ist via Menü erreichbar; HR-Privileg-Check schaltet Filter über alle Mitarbeiter frei (Auth-Context, kein User-Toggle)
  2. Form erlaubt CRUD eines `AbsencePeriodTO` mit Datum-Range-Picker (Ganztage), Kategorie-Dropdown (`Vacation`/`SickLeave`/`UnpaidLeave`), Description; Self-Overlap-`422` wird als Validation-Error gerendert
  3. `AbsencePeriodCreateResultTO.warnings[]` aus POST/PUT-Antwort wird als nicht-blockierende Hinweisliste angezeigt
  4. `cargo build --target wasm32-unknown-unknown` grün; UAT-Smoke gegen Integrationsumgebung (HR + Employee Login je einmal Anlage + Edit + Delete) — **deferred, siehe 08-HUMAN-UAT.md + Phase 8.1**

- [~] **Phase 8.1: Cutover-Migration-UI** (Frontend) — ⊘ **SUPERSEDED 2026-06-09**
  > **⊘ Abgelöst durch Phasen 8.4–8.6 (per-row Koexistenz-Modell).** Der Batch-Cutover-Wizard wird nicht fertiggestellt und nicht ausgeliefert. Statt einer Big-Bang-Migration mit ratender Heuristik + Quarantäne-Gate setzt v1.3 auf **dauerhafte additive Koexistenz** von `extra_hours` (manuelle Stunden-Ebene) und `absence_period` (strukturierte Ranges): Read-Projektion der `extra_hours` auf der Absence-Seite + manueller **HR-Einzel-Convert** (Range vom Menschen, kein Heuristik-Raten). Plan 08.1-12 (Phase-8-HUMAN-UAT-Subsumption / finaler Flag-Switch) ist damit gegenstandslos. Begründung-Stack siehe 08.4-CONTEXT.md (entstanden aus Design-Diskussion 2026-06-09). Historischer Inhalt bleibt unten zu Referenzzwecken erhalten.
  Admin-UI für die `extra_hours` → `absence_period`-Datenmigration. 3-Stage-Wizard (Profile → Dry-Run → Commit) mit Drift-Resolution-Liste, Per-Eintrag-Aktionen (Delete / Edit / Convert-to-Range / Skip) und Bulk-Aktionen. Schließt den Phase-8-int-UAT-Block, der durch reale Buchungs-Pattern-Diversität entstanden ist (Auto-Heuristik in Plan 08-09 deckt nicht alle Patterns ab — siehe 08-HUMAN-UAT.md gap-1).
  Requirements: (Closure-Phase, kein neues FUI-Requirement; löst Phase-8-Adoption-Block)
  Success Criteria:
  1. Admin-Route mit `cutover_admin`-Privileg-Gate; 3 sichtbare Stages; Profile + Dry-Run liefern strukturiertes Ergebnis-Display (Quarantine-Counts, Carryover-Diff)
  2. Drift-Resolution-Liste rendert pro `quarantined_entry` Datum/Wochentag/Stunden/Reason-Text/Suggested-Action (alles aus inline `gate_drift_report` aus Plan 08-08); Per-Eintrag-Aktionen für Delete / Edit-extra_hours / Convert-to-AbsencePeriod / Skip
  3. Bulk-Aktion "Alle Wochenpauschalen für (sales_person, category, year) konvertieren" verfügbar
  4. Cutover-Commit erst aktiv wenn `quarantined_rows == 0`; Confirmation-Dialog vor destruktivem Commit; Idempotenz-Hinweis nach Abschluss
  5. Optional: Feiertag-Konsistenz-Fix in `detect_weekly_lump_sum` (Plan 08-09 Inkonsistenz mit `derive_hours_for_range` — Group D Drifts wie Sonja Vac 2026)
  6. Phase-8-HUMAN-UAT (35 Schritte) wird auf int durchlaufen und gemeinsam mit Phase 8.1 closed; gap-1 in 08-HUMAN-UAT.md auf `resolved` gesetzt

- [x] **Phase 8.2: Manual-Range-Convert für Quarantäne** (Backend + Frontend, gap-1a-Closure) — completed 2026-05-10
  Erweitert die Cutover-UI um manuelles Konvertieren: Wenn die Heuristik einen Quarantäne-Eintrag nicht auflösen kann (Karin-Pattern, gap-1a — Vertragswechsel mit differing `hours_per_day` mid-week), gibt der Admin/HR den `absence_period`-Zeitraum (start/end) selbst vor. Backend erweitert `convert_quarantine_entry` um optionales `manual_range`, skipt die Heuristik und schreibt direkt. Frontend ersetzt das stub-bleibende `EditExtraHoursModal` durch ein `ManualConvertModal` mit Date-Range-Picker.
  Requirements: (Closure-Phase, schließt gap-1a aus 08.1-10-SUMMARY)
  Success Criteria:
  1. `CutoverConvertQuarantineEntryPayload.manual_range: Option<{ start, end }>` neu; bei `Some` skipt die Heuristik und nutzt den gegebenen Range; gleicher `synthetic_run_id`-Pfad wie heuristischer Convert
  2. Per-Eintrag-Modal in der Cutover-Page: Date-Range-Picker (von / bis), Category read-only, Submit dispatcht `CutoverAction::ManualConvert` und liefert `refreshed_drift_report` inline (D-08-Pattern aus 8.1)
  3. Karin-Diagnose-Test (`diagnose_int_drift_pattern_karin_*`) plus 1 neuer Test: manual_range löst die Karin-Quarantäne ohne Backend-Heuristik-Anpassung
  4. WASM-Build + Backend cargo test workspace grün; Privilege bleibt `cutover_admin OR hr` (D-23 aus 8.1)

- [ ] **Phase 8.3: Halbtag-Support für Absences** (Backend + Frontend + Cutover-UI-Erweiterung, Scope-Revision)
  Erweitert `AbsencePeriod` um `day_fraction: Full | Half`, damit halbe Urlaubstage (klassisch: Heiligabend + Silvester) abgebildet werden können. Vorlauf-Phase **vor** dem finalen Cutover-Switch (Plan 08.1-12), damit bestehende Halbtag-Buchungen im Cutover korrekt überführt werden. Datenkorrektur auf bereits gecutoverten Live-Daten wäre nachträglich schmerzhaft. Granularität bewusst zweiwertig — kein AM/PM, keine Stundenebene; Stundenebene bleibt out-of-scope. Revidiert die alte Out-of-Scope-Notiz "Halbtage / Stundenebene für Abwesenheiten" aus REQUIREMENTS.md.
  Requirements: FUI-A-10
  Success Criteria:
  1. `absence_period`-Tabelle + DTO + Service + REST + DAO erweitert um `day_fraction`; bestehende Einträge bleiben `Full` (no-drift-Garantie)
  2. Reporting (`derive_hours_for_range`, Vacation-Aggregation) berücksichtigt Halbtage; `CURRENT_SNAPSHOT_SCHEMA_VERSION` wird gebumpt
  3. Frontend `AbsenceModal` + `CutoverAdminPage`-Drift-Resolution + `ManualConvertModal` bekommen Halb/Ganz-Eingabe pro Eintrag
  4. i18n De / En / Cs für neue Labels; OpenAPI-Surface-Test grün; WASM-Build + `cargo test --workspace` grün; keine Regression in bestehenden Billing-Period-Snapshots

- [x] **Phase 8.4: Reporting-Additiv-Merge + Deprecation-Rückbau** (Backend) — ✓ verified passed 2026-06-09 (9/9 must-haves; Truth 9 dynamic-contract Balance-Parity geschlossen via Gap-Closure 05) — *neues Koexistenz-Modell, ersetzt 8.1-Cutover-Prämisse* — **Plans:** 5/5 complete
  `extra_hours` (Vacation/SickLeave/UnpaidLeave) bleibt ein **dauerhaft erlaubter** manueller Eingabeweg neben `absence_period`. Reporting summiert beide Quellen **additiv** (Modell A: keine globale Quellen-Umschaltung, keine Doppelzähl-Sperre per Flag — konvertierte/soft-deleted Rows tragen die per-row Quelle selbst). Der globale Flag `absence_range_source_active` und die Schreibsperre (D-Phase4-09) werden zurückgebaut.
  Requirements: (Modell-Revision; hebt Cutover-Prämisse aus v1.0 Phase 4 / 08.1 auf)
  Success Criteria:
  1. `reporting.rs` summiert `absence_period`-derived (`derive_hours_for_range`) **plus** lebende `extra_hours` (Vacation/Sick/Unpaid) additiv; der globale Flag-Filter (`reporting.rs:489`) entfällt; konvertierte (soft-deleted, `deleted IS NOT NULL`) `extra_hours` zählen nicht doppelt (per-row Quelle via `deleted IS NULL`)
  2. Schreibsperre in `extra_hours.rs` (`absence_range_source_active`-Gate, ~Z. 206) entfernt — neue Urlaubs-/Krank-/Unpaid-`extra_hours` wieder anlegbar
  3. **Snapshot-Schema-Version-Bump:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` +1 (Input-Menge der Vacation/Sick/Unpaid-Computation ändert sich — siehe `CLAUDE.md` § Snapshot Versioning)
  4. `cargo test --workspace` grün; Billing-Period-Snapshot-Regression sauber (alte Snapshots als „older schema" markiert)
  Plans:
  - [x] 08.4-01-PLAN.md — Wave 0: destruktive Test-Bereinigung (Flag/Gate-Tests löschen) + neue additive Test-Datei + Snapshot-Locking-Pin auf 5
  - [x] 08.4-02-PLAN.md — Wave 2: additiver Reporting-Merge + Snapshot-Bump 4→5 (ein jj-Commit) + extra_hours-Schreibsperre-Rückbau + Full-Suite-Gate
  - [x] 08.4-03-PLAN.md — Gap-Closure Wave 1 (Gap 1/CR-01+IN-03): additiver Display-Merge in get_reports_for_all_employees + get_week (Year-Bounds-Scoping) + WR-02 Gleichtags-Overlap-Test
  - [x] 08.4-04-PLAN.md — Gap-Closure Wave 2 (Gap 2/WR-01): absence-derived Stunden bewegen balance/expected symmetrisch in allen 3 Methoden + Snapshot-Bump 5→6 + Balance-Parity-Test + WR-03/IN-01/IN-02 Cleanup + Full-Suite-Gate
  - [x] 08.4-05-PLAN.md — Gap-Closure (Truth 9 / CR-01+WR-01): per-Woche-gegatete absence-Balance-Reduktion in get_reports_for_all_employees + get_week (dynamic-contract Parity), is_dynamic=true Fixture + 3 dynamische Balance-Parity-Tests, Snapshot bleibt 6 (kein Bump), Full-Suite-Gate

- [ ] **Phase 8.5: Read-Projektion + HR-Inline-Convert auf der Absence-Seite** (Backend + Frontend) — *Sichtbarkeit + reversibler manueller Convert*
  Die Absence-Liste blendet lebende `extra_hours`-Urlaub/Krank/Unpaid **read-only** mit „stundenbasiert"-Label ein (Read-Projektion — zeigt den Roh-Eintrag, **rekonstruiert keine Range**, daher driftfrei). HR kann einen stundenbasierten Eintrag per Inline-Aktion mit **selbst eingegebenem Zeitraum** in ein `absence_period` umwandeln. Wiederverwendet die in **Phase 8.2** gebaute atomare Convert-Tx (`manual_range` + `absence_period_migration_source`-Backlink + Soft-Delete) — nur aus dem Cutover-Namespace herausgelöst. Enthält den Working-Hours-Dialog-Umbau.
  Requirements: FUI-A-08 (revidiert — Soft-Migration statt Deprecation)
  Success Criteria:
  1. Absence-Read-Endpoint(s) (`GET /absence-period*`) liefern zusätzlich eine read-only Projektion lebender `extra_hours` (Vacation/Sick/Unpaid) als Tages-/Stunden-Marker; Frontend rendert sie mit sichtbarem **„stundenbasiert"**-Label + Edit-Deep-Link zur Working-Hours-Seite
  2. Neuer Convert-Endpoint außerhalb des `cutover`-Namespace (z.B. `POST /extra-hours/{id}/convert-to-absence`, Body `{ start, end, day_fraction }`) nutzt die 8.2-Tx-Logik (absence_period anlegen + extra_hours soft-delete + migration_source schreiben); **Heuristik nicht beteiligt**; Privileg **`hr`** (reversibel via Backlink)
  3. Inline-Aktion „In Zeitraum umwandeln" auf stundenbasierten Absence-Einträgen (HR-sichtbar) öffnet Range-Modal (von/bis + Halb/Ganz, reuse `ManualConvertModal` aus 8.2); Liste aktualisiert nach Convert
  4. **Dialog-Umbau** `add_extra_hours_form.rs`: Von/Bis-Range-Felder + `VacationDays`-Branch + `add_vacation`-Range-Call entfernt (nur noch Stunden-Eintrag); bei Vacation/SickLeave **Warnung + Empfehlung**, ganze Zeiträume auf der Absence-Seite zu erfassen (kein Block — Modell A)
  5. i18n De/En/Cs für neue Labels/Warnungen; `cargo build --target wasm32-unknown-unknown` grün; `cargo test --workspace` grün
  6. OpenAPI-`#[utoipa::path]` + `ToSchema` + Surface-Test für den neuen Convert-Endpoint
  **Plans:** 7 plans across 7 waves (sequenziell — Compile-Dependency-Kette Backend→Frontend)
  Plans:
  - [x] 08.5-01-PLAN.md — MigrationSourceDao (Trait+SQLite-Impl) + DB-Migration (cutover_run_id raus)
  - [x] 08.5-02-PLAN.md — AbsenceConversionService (BL-Tier, lean 3-write Convert-Tx) + Tests
  - [x] 08.5-03-PLAN.md — Convert-Endpoint + 3 rest-types-DTOs + DI-Wiring + Surface-Test
  - [x] 08.5-04-PLAN.md — Read-Projektion (beide GET-Handler augmentiert) + Integration-Tests
  - [x] 08.5-05-PLAN.md — Frontend-Daten-Schicht (api/state/loader/service) + AbsenceConvertModal-Extraktion
  - [x] 08.5-06-PLAN.md — absences.rs HourlyMarkerRow inline + Convert/Edit-Verdrahtung + i18n + SSR-Tests
  - [x] 08.5-07-PLAN.md — Working-Hours-Dialog-Umbau + Soft-Migration-Hinweis + i18n + Full-Suite/WASM-Gate

- [x] **Phase 8.6: Cutover-Abriss** (Backend + Frontend) — *Entfernung der Batch-Maschinerie* — ✓ Complete 2026-06-11
  Die Batch-Cutover-Maschinerie wird **ersatzlos entfernt**. Erhalten bleibt nur das per-row Convert-Plumbing (jetzt in 8.5: `absence_period_migration_source` + Soft-Delete-on-Convert).
  Requirements: (Aufräum-Phase)
  Success Criteria:
  1. Frontend: `page/cutover_admin.rs`, die `/admin/cutover`-Route (`app.rs`), der Menü-Eintrag und die Cutover-i18n-Keys entfernt
  2. Backend: `rest/src/cutover.rs` (alle 5 Handler: gate-dry-run / commit / profile / convert-quarantine-entry / bulk-convert-quarantine-rows) entfernt; `CutoverServiceImpl` Gate-/Quarantäne-/Profile-/Commit-/Bulk-Logik + Heuristik (`detect_weekly_lump_sum` / `iso_week_range` / `lookup_active_contract`) entfernt; obsolete Diagnose-Tests (Karin/Lila/Anina) entfernt
  3. Drop-Migration für die Tabelle `absence_migration_quarantine` (alte Migration unverändert lassen); `absence_period_migration_source` **bleibt**
  4. Feature-Flag `absence_range_source_active` vollständig entfernt (kein Reader mehr in `reporting.rs` / `extra_hours.rs` / `carryover_rebuild.rs`)
  5. OpenAPI-Surface-Test angepasst (Cutover-Schemas raus); `cargo test --workspace` + WASM-Build grün; kein toter Code (`cargo check --workspace` ohne Warnungen auf entfernte Symbole)
  Plans: 5 plans
  Plans:
  - [x] 08.6-01-PLAN.md — Backend-Abriss: CUTOVER_ADMIN_PRIVILEGE inlinen, Cutover+CarryoverRebuild-Crates loeschen, main.rs-DI entkoppeln, feature_flag-Tests re-keyen
  - [x] 08.6-02-PLAN.md — Frontend-Abriss: Cutover-Page/Route/i18n/api raus, top_bar absences-Gate auf logged_in (D-03), FeatureFlagsState Shell-Trim (D-02)
  - [x] 08.6-03-PLAN.md — Forward-only Drop-Migrations (quarantine + carryover-backup) + Seed-DELETE + .sqlx-Regeneration
  - [x] 08.6-04-PLAN.md — OpenAPI-Surface-Test: Cutover-Pfade/-Schemas raus, /feature-flag/{key} bleibt (D-08)
  - [x] 08.6-05-PLAN.md — Verifikations-Gate: cargo test/check --workspace + WASM-Build + Negative-Space/Survivor-Asserts (D-09)

- [x] **Phase 9: Booking-Flow Reverse-Warnings** (Frontend) — *Copy-Week descoped 2026-06-11 (siehe 09-CONTEXT.md)*
  Shiftplan-Editor-Buchungen laufen über `POST /shiftplan-edit/booking` mit Reverse-Warnings-Confirm-Dialog (Optimistic-Create + Rollback bei Abbruch). **Copy-Week wurde aus dieser Phase descoped** — das Feature war vom User bewusst per Commit `294566f` ("feat: Remove copy last week feature") entfernt; FUI-A-06 ist dropped, der tote Copy-Week-Code wird hier aufgeräumt. Backend `POST /shiftplan-edit/copy-week` + `CopyWeekResultTO` bleiben unangetastet.
  Requirements: FUI-A-05 (FUI-A-06 → dropped)
  Success Criteria:
  1. Booking aus Shiftplan-Editor postet auf `/shiftplan-edit/booking`; `BookingCreateResultTO.warnings[]` löst Dioxus-Confirm-Dialog aus (kein `window.confirm`); "Abbrechen" macht die optimistisch erstellte Buchung per DELETE rückgängig, "Trotzdem buchen" behält sie
  2. Toter Copy-Week-Frontend-Code entfernt: `ShiftPlanAction::CopyFromPreviousWeek` + Handler, `api::copy_week`, `loader::copy_from_previous_week`, ungenutzter i18n-Key `ShiftplanTakeLastWeek` (en/de/cs). Backend-Endpoint bleibt
  3. Alter `POST /booking` bleibt parallel verfügbar (verifiziert durch grep-Check, dass alte Call-Sites unverändert sind)
  **Plans:** 2 plans across 2 waves
  Plans:
  - [x] 09-01-PLAN.md — Wave 1: Foundation — conflict-aware api/loader fns + shared component/warning_list.rs (3 booking variants + person_name prop + SSR tests) + 7 i18n keys × 3 locales (parity/reference-matcher tests)
  - [x] 09-02-PLAN.md — Wave 2: Wiring — shiftplan.rs RollbackBooking action + Dioxus confirm-dialog (optimistic+rollback) + Copy-Week dead-code cleanup + ROADMAP/REQUIREMENTS doc-sync + WASM/workspace gate

- [ ] **Phase 10: Shiftplan-View Unavailability-Marker** (Frontend)
  Shiftplan-Wochen-View visualisiert `UnavailabilityMarkerTO` farbig pro Tag pro Person mit drei Visual-States.
  Requirements: FUI-A-07
  Success Criteria:
  1. Wochen-View nutzt per-sales-person Endpoint `/shiftplan-info/{shiftplan_id}/{year}/{week}/sales-person/{sales_person_id}`
  2. `UnavailabilityMarkerTO::AbsencePeriod` mit Kategorie-Farbe gerendert (Vacation = grün, SickLeave = orange, UnpaidLeave = grau — Final-Farben in UI-SPEC)
  3. `UnavailabilityMarkerTO::ManualUnavailable` neutral gerendert; `UnavailabilityMarkerTO::Both` mit eigener Visual-Indication (signalisiert redundanten manuellen Eintrag nach Cutover, optional Aufräum-Button)

- [~] **Phase 11: Migrations-Hinweis-UX + Deprecation-Handling** (Frontend) — ⊘ **SUPERSEDED 2026-06-09**
  > **⊘ Abgelöst durch das Koexistenz-Modell (8.4–8.6).** `extra_hours`-Urlaub/Krank wird **nicht** mehr deprecated → SC 2 (`403 ExtraHoursCategoryDeprecated` abfangen) und SC 3 (Flag-Defensive) sind gegenstandslos. Der einzige überlebende Scope (SC 1: Soft-Hinweis/Empfehlung auf der Stunden-Maske) ist vollständig in **Phase 8.5 SC 4** (Dialog-Umbau + Warnung) gefaltet. Kein Rest-Scope verbleibt. Historischer Inhalt bleibt unten als Referenz erhalten.
  Alte `extra_hours`-basierten "Urlaub eintragen"-Eingangswege werden auf neue Maske umgelenkt; nach Cutover wird `403 ExtraHoursCategoryDeprecatedErrorTO` mit User-Hinweis abgefangen.
  Requirements: FUI-A-08
  Success Criteria:
  1. `add_extra_hours_form.rs`, `extra_hours_modal.rs`, `add_extra_days_form.rs`, `add_extra_hours_choice.rs` verlinken für `Vacation` / `SickLeave` / `UnpaidLeave` auf `/absences` (Soft-Migration vor Cutover)
  2. `403 ExtraHoursCategoryDeprecatedErrorTO`-Response wird abgefangen, Toast/Banner mit Migrations-Hinweis und Link zur neuen Maske gerendert
  3. Cutover-Flag-Status (`absence_range_source_active`) wird defensiv gehandhabt: lesen immer aus `/absence-period`; Schreiben über alte Maske nur falls Flag noch aus, sonst Redirect

- [ ] **Phase 12: UI-Closure v1.1/v1.2-Restanten** (Frontend)
  Schließe sichtbares `current_paid_count`/`max_paid_employees`-Rendering, Capacity-Editor in Slot-Settings, sichtbare `VolunteerWork`/`UnpaidLeave`-Kategorien und `cap_planned_hours_to_expected`-Settings.
  Requirements: FUI-01, FUI-02, FUI-03, FUI-04
  Success Criteria:
  1. `current_paid_count` ist im Shiftplan-Week-View pro Slot sichtbar; mit Layout-Variante `2/3 bezahlt` wenn `max_paid_employees` konfiguriert; `Warning::PaidEmployeeLimitExceeded` wird visuell hervorgehoben
  2. Slot-Settings haben Capacity-Editor mit Clear-Button für `None` (kein Limit); Round-Trip-Test (open → save unverändert) bewahrt den Backend-Wert
  3. `VolunteerWork` / `UnpaidLeave` werden in Extra-Hours-Listen sichtbar gerendert (kein `rsx! { "" }` mehr aus v1.2 Plan 06-04); Kategorien sind in der Anlage-Form auswählbar (sofern Cutover-Flag-Konsistenz erlaubt)
  4. `cap_planned_hours_to_expected` ist im Sales-Person-Settings-UI als Toggle editierbar; Server-Round-Trip verifiziert

- [ ] **Phase 13: i18n-Vollständigkeits-Audit + v1.3 Smoke-Closure** (Subsumption-Closure)
  Cross-Phase i18n-Audit: alle in v1.3 hinzugefügten Strings sind in De / En / Cs vollständig. Plus Final-UAT auf Integrationsumgebung (Subsumption-Pattern wie v1.2 Phase 7).
  Requirements: FUI-A-09
  Success Criteria:
  1. Alle in Phasen 8–12 hinzugefügten i18n-Keys sind in `en.rs`, `de.rs`, `cs.rs` vollständig (kein Locale::En-statt-Locale::De-Bug); diff-Audit dokumentiert
  2. Final-UAT: HR-Login + Employee-Login je einmal durch alle drei Locales (Page-Load, Form-Anlage, Warning-Render, Deprecation-Toast)
  3. Backend `cargo check --workspace` + `cargo test --workspace` re-verifiziert (keine Regression durch Frontend-Phasen-Coupling)
  4. WASM-Build `cargo build --target wasm32-unknown-unknown` grün als finaler Compile-Gate

<details>
<summary>✅ v1.0 Range-Based Absence Management (Phasen 1–4) — SHIPPED 2026-05-03</summary>

- [x] **Phase 1: Absence Domain Foundation** (5/5 plans) — completed 2026-05-01
  Neue parallele `absence` Domain (DAO + Service + REST + Permission), additiv, ohne Reporting-Wirkung
- [x] **Phase 2: Reporting Integration & Snapshot Versioning** (4/4 plans) — completed 2026-05-02
  `derive_hours_for_range` + Reporting-Switch hinter Feature-Flag, `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2 → 3 im selben Commit
- [x] **Phase 3: Booking & Shift-Plan Konflikt-Integration** (6/6 plans) — completed 2026-05-02
  Forward/Reverse Booking-Warnings + Shift-Plan-Anzeige aus AbsencePeriod ohne Doppel-Eintragung
- [x] **Phase 4: Migration & Cutover** (8/8 plans) — completed 2026-05-03
  Heuristik-Migration, Validierungs-Gate (< 0.01h Drift-Toleranz), atomarer Feature-Flag-Flip mit Carryover-Refresh, REST-Deprecation. Plus Bonus-Recovery von `extra_hours.update` mit logical_id-Versionierung.

**Full milestone archive:** [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)

</details>

<details>
<summary>✅ v1.1 Slot Capacity & Constraints (Phase 5) — SHIPPED 2026-05-04</summary>

- [x] **Phase 5: Slot Paid Capacity Warning** (6/6 plans) — completed 2026-05-04
  Slots erhalten ein optionales `max_paid_employees: Option<u8>` Capacity-Limit. Backend emittiert nicht-blockierende `Warning::PaidEmployeeLimitExceeded` (a) im `BookingCreateResult.warnings` im Conflict-Aware-Booking-Flow und (b) als `current_paid_count` per Slot im Shiftplan-Week-View. 461 Tests grün; 16/16 D-decisions verified. Frontend (shifty-dioxus) out of scope.

**Full milestone archive:** [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)

</details>

<details>
<summary>✅ v1.2 Frontend rest-types Konsolidierung (Phasen 6–7) — SHIPPED 2026-05-07</summary>

- [x] **Phase 6: rest-types Unification & Frontend Compile-Through** (5/5 plans) — completed 2026-05-07
  Backend-`rest-types` als single source of truth verdrahtet, Frontend-Fork gelöscht, 17 fehlende TOs/Enum-Varianten + 4 fehlende Felder + Match-Arme adressiert; `cargo build --target wasm32-unknown-unknown` grün; 466 Backend-Tests ohne Regression. 8/8 V-Truths verified.
- [x] **Phase 7: Runtime Smoke & Regression Safety** (1/1 plan) — completed 2026-05-07
  Frontend-Boot, Login und Shiftplan-Navigation auf Integrationsumgebung verifiziert (User-UAT 2026-05-07); Backend `cargo check --workspace` + `cargo test --workspace` re-verifiziert (Subsumption von Phase-6 V-Truth #6 + #7 plus lokaler Re-Run). 4/4 Success Criteria verified.

**Full milestone archive:** [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)

</details>

## Phase Details

### Phase 8: Absence-CRUD-Page Foundation

**Goal:** Neue Top-Level-Route `absences` in `shifty-backend/shifty-dioxus` ist via Menü erreichbar und bietet vollständiges CRUD gegen `/absence-period`. HR-vs-Employee-Sicht kommt aus dem Auth-Context (kein User-Toggle). Die Form bietet Datum-Range-Picker (Ganztage), Kategorie-Dropdown (`Vacation` / `SickLeave` / `UnpaidLeave`) und Description; nicht-blockierende `AbsencePeriodCreateResultTO.warnings[]` werden gerendert. Zusätzlich wird ein neuer Backend-Resturlaubs-Endpoint nachgezogen, weil `VacationEntitlementCard` und `VacationPerPersonList` aus dem Mockup einen autoritativen Resturlaubs-Wert anzeigen (siehe CONTEXT.md D-02/D-03).

**Depends on:** v1.0 Phase 1 (Absence-Backend-Domain), v1.0 Phase 4 (Cutover-Surface), v1.2 Phase 6 (rest-types-Unification — `AbsencePeriodTO` / `AbsenceCategoryTO` / `AbsencePeriodCreateResultTO` / `WarningTO` aus zentralem `rest-types` referenzierbar), v1.2 Phase 7 (WASM-Compile + Runtime-Smoke grün).

**Requirements:** FUI-A-01, FUI-A-02, FUI-A-03, FUI-A-04

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. Route `/absences` ist via Menü erreichbar; HR-Privileg-Check schaltet Filter über alle Mitarbeiter frei (Auth-Context, kein User-Toggle) (FUI-A-01).
2. Form erlaubt CRUD eines `AbsencePeriodTO` mit Datum-Range-Picker (Ganztage), Kategorie-Dropdown (`Vacation` / `SickLeave` / `UnpaidLeave`), Description; Self-Overlap-`422` aus Backend wird als Validation-Error gerendert (FUI-A-02, FUI-A-03).
3. `AbsencePeriodCreateResultTO.warnings[]` aus POST/PUT-Antwort wird als nicht-blockierende Hinweisliste gerendert (FUI-A-04).
4. Neuer Backend-Resturlaubs-Endpoint (Shape Plan-Phase-Decision) liefert für `(sales_person_id [, year])` einen Wert mit entitled / used / planned / remaining (oder semantisch äquivalent); `hr ∨ self`-Permission analog zu `/absence-period`; OpenAPI-`#[utoipa::path]`-Annotation; `ToSchema` auf der DTO. Frontend-Komponenten `VacationEntitlementCard` (eigener User) und `VacationPerPersonList` (HR-Übersicht) konsumieren diesen Endpoint.
5. `cargo build --target wasm32-unknown-unknown` im `shifty-backend/shifty-dioxus/`-Subordner liefert Exit-Code 0 ohne Errors; `cargo check --workspace` + `cargo test --workspace` im Backend-Root grün (Backend-Erweiterung darf keine Regression verursachen); UAT-Smoke gegen Integrationsumgebung (HR + Employee Login je einmal Anlage + Edit + Delete + Resturlaubs-Anzeige) erfolgreich.

**Plans:** 9 plans (3 Backend-Waves + 3 Frontend-Waves + 1 Gap-Closure-Plan + 1 Cutover-Response-Polish-Plan + 1 Cutover-Heuristik-Plan)

- [x] 08-01-PLAN.md — Service-Trait + Domain-Struct (`VacationBalanceService`) + DTO (`VacationBalanceTO`) — Wave 1, BL-Tier interface foundation (completed 2026-05-08)
- [x] 08-02-PLAN.md — Service-Impl (BL-Tier per gen_service_impl!) + 7 Unit-Tests + REST-Endpoints (utoipa) + DI-Wiring in main.rs — Wave 2 (completed 2026-05-08)
- [x] 08-03-PLAN.md — OpenAPI Surface-Assertion-Test (`rest/tests/openapi_surface.rs`; Option-B-Pivot vom flaky insta-snapshot weg; pinnt Pfad-Liste + Schema-Namen + VacationBalance-Tag; 3-run-determinism) — Wave 3 (completed 2026-05-08)
- [x] 08-04-PLAN.md — Frontend Foundation: api.rs (8 fns) + ShiftyError::Validation + state-types + loader + service-coroutines + 60 i18n-Keys (de/en/cs) + Dx-Proxy-Einträge — Wave 4 (completed 2026-05-08)
- [x] 08-05-PLAN.md — AbsencesPage + AbsenceModal + 9 inline components (WarningList + CategoryBadge + StatusPill + VacationEntitlementCard + VacationPerPersonList + AbsenceList + AbsenceFilterBar + StatsGrid + DeleteConfirmDialog) + Route::Absences + TopBar entry + 11 dioxus-ssr snapshot tests + WASM-Build-Gate — Wave 5; closes Wave-0-Item-3 in VALIDATION.md (nyquist_compliant: true) (completed 2026-05-08)
- [x] 08-06-PLAN.md — UAT-Smoke (HR + Employee) + Final-Regression-Gates (cargo test --workspace + WASM-Build) — Wave 6 closure
- [x] 08-07-PLAN.md — Gap-Closure: admin-auto-grant trigger sqlx-Migration + GET /feature-flag/{key} REST-Endpoint + Frontend FeatureFlagsState + TopBar Cutover-Gate + HR-Submenu + Responsive AbsencesPage Layout (completed 2026-05-08)
- [x] 08-08-PLAN.md — Cutover-Response Drift-Details: QuarantineReason::human_text + suggested_action + CutoverQuarantineEntryTO + CutoverRunResultTO.gate_drift_report inline DTO + per-entry quarantined_entries; failed-Gate liefert interpretierbare Antwort ohne File-Lookup (completed 2026-05-08)
- [x] 08-09-PLAN.md — Cutover Wochenpauschalen-Heuristik: detect_weekly_lump_sum + iso_week_range + lookup_active_contract; Migration-Loop bekommt Pre-Check (a.5) VOR Workday/Strict-Match-Quarantäne; 1× extra_hours-Row mit `amount ≈ Σ hours_per_day für Vertragstage der ISO-Woche` → absence_period {Mo, So}; Live-Szenario Max-Schmidt 20h@Friday bei 3-Tage-Vertrag migriert sauber (gate passes, drift=0); 7 unit + 1 integration test (completed 2026-05-08)

**UI hint**: yes

**Notes for plan-phase:** Misch-Phase Backend + Frontend im Monorepo (`shifty-backend/`). **Frontend-Schwerpunkt** (`shifty-dioxus/`): Page + Modal + Service + State + Loader + API-Layer + i18n; Backend-Endpoints `/absence-period` (GET-list, GET-by-id, POST, PUT, DELETE, GET-by-sales-person) sind in v1.0 Phase 1 geshipped (`rest/src/absence.rs`); DTOs (`AbsencePeriodTO`, `AbsenceCategoryTO`, `AbsencePeriodCreateResultTO`, `WarningTO`) liegen in `rest-types/src/lib.rs:1565..2040`. **Backend-Erweiterung neu in Scope:** Resturlaubs-Endpoint + neuer DTO `VacationBalanceTO` (Name + Shape Plan-Phase-Decision) + neuer Service. Erwartete Tier-Klassifizierung: **Business-Logic-Tier** (kombiniert `WorkingHoursService` + `AbsenceService`/`AbsenceDao`, ggf. `SpecialDayService`). Permission `hr ∨ self`. Siehe `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen". Mockup-Quellen: `shifty-backend/shifty-dioxus/shifty-design/project/absences.jsx` (729 Zeilen, `AbsencePage` + `AbsenceModal` + `WarningList` + `CategoryBadge` + `StatusPill` + `VacationEntitlementCard` + `VacationPerPersonList` — alles im Phase-8-Scope) und Integrations-Brief `shifty-backend/shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md`. Tweak `viewAs` aus dem Mockup ist NICHT zu übernehmen — Sicht kommt aus Auth-Context (`hr`-Privileg). Confirm-Dialog im Mockup verwendet `window.confirm`; im echten Frontend ist Dioxus-Dialog-Komponente zu nutzen (`shifty-backend/shifty-dioxus/src/component/dialog.rs`). i18n De / En / Cs ist Teil dieser Phase (Page-Titel, Kategorie-Labels, Form-Labels, Warning-Texte) — kein nachgelagerter Audit, sondern direkt mit der Implementierung. Out-of-Scope-Mockup-Komponenten: `UnavailabilityChip` → Phase 10; Deprecation-Banner für legacy `extra_hours` → Phase 11. Vollständige Decision-Liste: `.planning/phases/08-absence-crud-page-foundation/08-CONTEXT.md` (D-01..D-14). Plan-phase legt fest, ob api/loader/state/page-Komponenten und Backend-Erweiterung in einer oder mehreren Waves laufen.

---

### Phase 8.1: Cutover-Migration-UI

**Goal:** Admin-UI im Monorepo (`shifty-backend/shifty-dioxus/`) für die `extra_hours` → `absence_period`-Datenmigration. 3-Stage-Stepper-Wizard (Profile → Dry-Run → Commit) mit Drift-Resolution-Liste und Per-Eintrag-Aktionen (Delete / Edit / Convert-to-Range / Skip) sowie Bulk-Aktionen pro `(sales_person, category, year)`-Gruppe. Schließt den Phase-8-int-UAT-Block, der durch reale Buchungs-Pattern-Diversität entstanden ist (Auto-Heuristik in Plan 08-09 deckt nicht alle Patterns ab — siehe `08-HUMAN-UAT.md` gap-1). Backend-additiv: zwei neue atomic-tx Endpoints (`POST /admin/cutover/convert-quarantine-entry`, `POST /admin/cutover/bulk-convert-quarantine-rows`) auf `CutoverServiceImpl` (Business-Logic-Tier), die `extra_hours`-Soft-Delete + `absence_period`-Insert unter `cutover_admin`-Privileg zusammenführen. Schließungs-Phase für Phase 8 — nach Cutover läuft der dort deferred 35-Schritt-HUMAN-UAT als Subsumption-Plan.

**Depends on:** v1.0 Phase 4 (Cutover-Surface — `CutoverServiceImpl`, `CutoverRunResultTO`, `CUTOVER_ADMIN_PRIVILEGE`), Phase 8 Plan 08-07 (Feature-Flag-Endpoint `GET /feature-flag/{key}` + `FeatureFlagsState` + "Verwaltung"-Submenu-Pattern), Phase 8 Plan 08-08 (`gate_drift_report`-Inline-Shape, `CutoverQuarantineEntryTO`, `QuarantineReason::{human_text, suggested_action}`), Phase 8 Plan 08-09 (`detect_weekly_lump_sum` + `iso_week_range` + `lookup_active_contract`-Helper), v1.2 Phase 6 (rest-types-Cross-Crate-Konstruktion).

**Requirements:** Closure-Phase — kein neues FUI-Requirement; löst Phase-8-Adoption-Block (`08-HUMAN-UAT.md` gap-1). Pflicht-Locale-Coverage (FUI-A-09) gilt für neu hinzugefügte i18n-Keys.

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. Admin-Route `/admin/cutover` mit `cutover_admin`-Privileg-Gate (HR sieht Page + Profile + Dry-Run, nur `cutover_admin` darf Commit); 3 sichtbare Stages; Profile + Dry-Run liefern strukturiertes Ergebnis-Display (Quarantine-Counts, Per-Person-Stats, Carryover-Diff).
2. Drift-Resolution-Liste rendert pro `quarantined_entry` ISO-Datum + Wochentag-Code + Hours + Reason-Text + Suggested-Action (alles aus inline `gate_drift_report` von Plan 08-08); gegliedert nach `(sales_person, category, year)`; Per-Eintrag-Aktionen Convert / Edit-extra_hours / Delete / Skip in Action-Spalte.
3. Bulk-Aktion "Alle Wochenpauschalen für (sales_person, category, year) konvertieren" verfügbar je Gruppe; ruft `POST /admin/cutover/bulk-convert-quarantine-rows` (single-Tx, atomar pro Gruppe).
4. Cutover-Commit erst aktiv wenn `quarantined_rows == 0`; Type-to-confirm-Dialog ("CUTOVER") + Migration-Summary vor destruktivem Commit; Idempotenz-Hinweis nach Abschluss + Permanent-Banner bei Re-Open via `absence_range_source_active`-Flag-Check (Plan 08-07 `FeatureFlagsState`).
5. Backend: zwei neue Endpoints (`POST /admin/cutover/convert-quarantine-entry`, `POST /admin/cutover/bulk-convert-quarantine-rows`) mit `#[utoipa::path]`, `ToSchema` auf neuen DTOs, `cutover_admin`-Privilege-Check, atomic-tx (`extra_hours`-Soft-Delete + `absence_period`-Insert in einer Tx); `EXPECTED_PATHS`/`EXPECTED_SCHEMAS` in `rest/tests/openapi_surface.rs` ergänzt; Unit-Tests für beide Service-Methoden.
6. Diagnose-Plan für `08-HUMAN-UAT.md` gap-1 (a) (Vertragsdaten-Edge-Case Lila/Anina/Karin): Reproduce mit Test-Fixtures + Hypothesen (mid-week-Vertragswechsel, Hire-Date-Edge-Cases, Inactive-Contract-Tage in `lookup_active_contract`); Fix wenn klar, sonst dokumentierter bleibender gap.
7. **Optional:** Feiertag-Konsistenz-Fix in `detect_weekly_lump_sum` (gap-1 (c)) ist explizit OUT OF SCOPE (`derive_hours_for_range` skipt Holidays bewusst, `service_impl/src/absence.rs:483-485`); Operator löst manuell via Edit oder Convert + Edit.
8. i18n De / En / Cs vollständig für Page-Chrome (Stage-Labels, Stat-Box-Titel, Action-Buttons, Confirm-Dialog-Texte, Banner-Texte, Toast/Error-Texte); `QuarantineReason`-Texte (`reason_text`, `suggested_action`) bleiben Englisch und werden unverändert gerendert (Plan 08-08-Konvention). Per-Locale-Reference-Matcher-Tests gegen `Locale::En`-statt-`Locale::De`-Bug analog Plan 08-04 D-26.
9. Eigener 8.1-UAT-Plan für die Cutover-UI selbst (Wizard-Stages, Drift-Resolution-Aktionen alle vier je einmal, Bulk-Convert auf Group-Section, Type-to-confirm-Verhalten, Idempotenz-State nach Commit).
10. Phase-8-HUMAN-UAT (35 Schritte, `08-HUMAN-UAT.md`) wird auf int durchlaufen NACH 8.1-UI-Cutover und gemeinsam mit Phase 8.1 closed; gap-1 in `08-HUMAN-UAT.md` auf `resolved` gesetzt.
11. `cargo build --target wasm32-unknown-unknown` im `shifty-backend/shifty-dioxus/`-Subordner liefert Exit-Code 0 ohne Errors; `cargo check --workspace` + `cargo test --workspace` im Backend-Root grün (Backend-Convert-Endpoints + Frontend dürfen keine Regression verursachen).

**Plans:** 12 plans across 6 waves
- [ ] 08.1-01-PLAN.md — rest-types DTOs (4 Request/Response + CutoverConvertErrorTO) — Wave 1
- [ ] 08.1-02-PLAN.md — Service convert_quarantine_entry + compute_gate_diagnostic helper + 4 mockall tests + From-Impl — Wave 1
- [ ] 08.1-03-PLAN.md — Service bulk_convert_quarantine_rows (strict-atomic) + 4 mockall tests + From-Impl — Wave 1
- [ ] 08.1-04-PLAN.md — REST handlers (2) + ApiDoc + OpenAPI surface entries + 5 integration tests — Wave 1
- [ ] 08.1-05-PLAN.md — Frontend api.rs (5 cutover_* fns) + Dioxus.toml proxy entry — Wave 2
- [ ] 08.1-06-PLAN.md — i18n: 33 Cutover* keys × 3 locales + 4 reference-matcher tests — Wave 2
- [ ] 08.1-07-PLAN.md — Router::AdminCutover + page-stub + TopBar Verwaltung-Submenu entry + 4 nav tests — Wave 2
- [ ] 08.1-08-PLAN.md — state/cutover_state + service/cutover (CUTOVER_STORE + CUTOVER_DRIFT_REFRESH + Coroutine) + 5 tests — Wave 2
- [ ] 08.1-09-PLAN.md — page/cutover_admin.rs Single-File-Composition (11 components) + 11 dioxus-ssr snapshot tests + WASM-Build-Gate — Wave 3
- [ ] 08.1-10-PLAN.md — Diagnose-Plan gap-1 (a): Lila/Anina/Karin contract edge-case tests + optional fix — Wave 4
- [ ] 08.1-11-PLAN.md — 8.1-eigener UAT (D-21) — Wave 5 (manual checkpoint)
- [ ] 08.1-12-PLAN.md — Phase-8-HUMAN-UAT-Subsumption (D-20) + final regression gates — Wave 6 (manual checkpoint)

**UI hint**: yes (Frontend-Schwerpunkt + Backend-additiv)

**Notes for plan-phase:** Misch-Phase wie Phase 8 (Backend + Frontend im selben Monorepo). **Vollständige Decision-Liste D-01..D-27 + Phase-Boundary + Out-of-Scope:** `.planning/phases/8.1-cutover-migration-ui/8.1-CONTEXT.md` (CANONICAL — alle Detail-Decisions inkl. Convert-Endpoint-Shapes, Stepper-Topologie D-07/D-08, Drift-Listen-Gliederung D-11..D-14, Type-to-confirm D-15, Idempotenz-Detection D-17, Privilege-Gate D-23, i18n-Pattern D-26/D-27). **Service-Tier:** Convert-Endpoints sind Business-Logic-Tier auf existing `CutoverServiceImpl` (Cross-Aggregat: extra_hours + absence_period + working_hours; reuse `detect_weekly_lump_sum` + `iso_week_range` + `lookup_active_contract`). **Idempotenz-Pattern:** `Option<bool>::None`-Default in `FeatureFlagsState` verhindert Banner-Flackern (Plan 08-07-Pattern). **Auto-Re-Run:** Nach jeder Resolve-Aktion `gate-dry-run` triggern (D-08); Plan-Phase entscheidet ob Backend `refreshed_drift_report` inline mitliefert oder Frontend separat fetched. **OpenAPI-Surface-Test:** `rest/tests/openapi_surface.rs` (`EXPECTED_PATHS` + `EXPECTED_SCHEMAS`) muss um die zwei neuen Pfade + neue DTOs ergänzt werden (Plan 08-03-Pattern). **Snapshot-Schema-Versioning:** 8.1 berührt keine `BillingPeriodValueType`-Erweiterung — `CURRENT_SNAPSHOT_SCHEMA_VERSION` braucht KEINEN Bump. **VCS:** jj-only (siehe `CLAUDE.local.md`); Plans dürfen keine `git commit`-Befehle planen. **Out-of-Scope explizit:** Backend-Heuristik-Fix für Feiertage (D-06), Audit-Log-UI, Cutover-History-Page, Multi-Tenant, Force-Commit-Override (siehe CONTEXT.md `<domain>` "Out of Scope").

---

### Phase 8.2: Manual-Range-Convert für Quarantäne

**Goal:** Closure-Phase für `08.1-10`-gap-1a (Karin-Pattern). Erweitert die in Phase 8.1 etablierte Convert-API um einen manuellen Pfad: Wenn die Heuristik einen Quarantäne-Eintrag nicht auflösen kann (Vertragswechsel mit differing `hours_per_day` mid-week, Hire/End-Date-Edge-Cases ohne weekly-lump-sum-Match), gibt der Admin/HR den Ziel-`absence_period`-Zeitraum direkt vor und das Backend schreibt ohne weitere Mustererkennung. Frontend ersetzt das in 8.1-09 als bekannten Stub belassene `EditExtraHoursModal` durch ein `ManualConvertModal` mit Date-Range-Picker. Audit-Pfad bleibt identisch zum heuristischen Convert (gleicher `synthetic_run_id`-Flow, inline `refreshed_drift_report` per D-08).

**Depends on:** Phase 8.1 (Cutover-Migration-UI — `convert_quarantine_entry`, `compute_gate_diagnostic`, `CutoverAdminPage`-State + Coroutine, `CutoverConvertQuarantineEntryRequest`/`Response`-DTOs).

**Requirements:** Closure-Phase — schließt gap-1a aus `08.1-10-SUMMARY.md` (Karin-Pattern, bleibender gap dokumentiert für Operator-Resolution). Pflicht-Locale-Coverage (FUI-A-09) für neu hinzugefügte i18n-Keys (Date-Range-Picker-Labels, Modal-Titel, Hilfetext).

**Success Criteria:**

1. `CutoverConvertQuarantineEntryRequest` erhält optionales `manual_range: Option<{ start_date, end_date }>`. Bei `Some` skipt `convert_quarantine_entry` die Heuristik (`detect_weekly_lump_sum` + `lookup_active_contract`-Match) und schreibt direkt eine `absence_period` mit dem gegebenen Zeitraum + Soft-Delete der zugehörigen `extra_hours`. Same-Tx, gleicher `synthetic_run_id`, `refreshed_drift_report` inline.
2. Backend-Validation: `start_date <= end_date`; beide Daten innerhalb des Quarantäne-Eintrag-Jahres; Kategorie unverändert (read-only übernommen aus dem Quarantäne-Eintrag); `cutover_admin OR hr` Privilege-Check (D-23 aus 8.1).
3. Karin-Diagnose-Test (`diagnose_int_drift_pattern_karin_*` aus 8.1-10) wird durch einen neuen Test ergänzt: `convert_quarantine_entry` mit `manual_range = Some(...)` löst Karin-Quarantäne ohne Heuristik-Anpassung; `derive_hours_for_range` über die manuell gesetzte Range matcht den `legacy_sum`.
4. Frontend: `EditExtraHoursModal`-Stub aus 8.1-09 wird zum `ManualConvertModal`. Eingabefelder: Datum-von, Datum-bis (`<input type="date">` reicht), Kategorie read-only, Stunden read-only (informativ — die werden vom Backend aus Range + Contract abgeleitet). Submit dispatcht `CutoverAction::ManualConvert { extra_hours_id, manual_range }`. `refreshed_drift_report` aus Response landet im `CUTOVER_STORE` (selbe Mechanik wie 8.1-09 Convert/Bulk-Convert).
5. OpenAPI-Surface-Test bleibt grün — die Änderung ist additiv (neues optionales Feld); ein neuer Schema-Drift-Eintrag für das DTO bestätigt das `manual_range`-Feld.
6. WASM-Build (`cd shifty-dioxus && cargo build --target wasm32-unknown-unknown`) + Backend `cargo test --workspace` grün; Snapshot-Schema-Version unverändert (kein neuer `BillingPeriodValueType`).

**Plans:** 2 plans across 2 waves (sequenziell — Frontend-Plan wartet auf Backend-DTO)
- [x] 08.2-01-PLAN.md — Backend `manual_range`-Branch + DTO + 4 mockall + 1 integration test + OpenAPI-Schema-Update — Wave 1 (completed 2026-05-10, see 08.2-01-SUMMARY.md)
- [x] 08.2-02-PLAN.md — Frontend ManualConvertModal + Coroutine-Action + 8 i18n keys × 3 locales + 4 dioxus-ssr snapshots + WASM-Build-Gate — Wave 2 (completed 2026-05-10, see 08.2-02-SUMMARY.md)

**UI hint**: yes (Backend additiv + Frontend Modal-Erweiterung).

**Notes for plan-phase:** Sehr kleine Phase — voraussichtlich 1-2 Plans (Backend-Erweiterung + Frontend-Modal-Replacement, gegebenenfalls in einer Welle parallelisierbar wenn rest-types-Änderung in Plan 1 gemacht wird). Reuse 8.1-Patterns: Service-Tier-Klassifikation (Business-Logic), `compute_gate_diagnostic` für `refreshed_drift_report`, jj-only Commit-Politik. **Out-of-Scope:** Generelles Edit der Stunden eines `extra_hours`-Eintrags (sofern doch gewünscht: separate Phase oder Kombi-Modal mit Tab-Switch). **Karin-Test wird zur Verifikation des Manual-Convert-Pfads**; bleibender gap aus 8.1 wird als `resolved` markiert in 8.2 SUMMARY.

---

### Phase 8.3: Halbtag-Support für Absences

**Goal:** Backend-Datenmodell + Service + REST + Frontend-CRUD + Cutover-Migration-UI um halbe Urlaubstage erweitern (`day_fraction: Full | Half` auf `AbsencePeriod`). Vorlauf-Phase **vor** dem finalen Cutover-Switch (Plan 08.1-12, Phase-8-HUMAN-UAT-Subsumption), damit bestehende halbe Urlaubstage (Heiligabend 24.12., Silvester 31.12.) im Cutover korrekt nach `absence_period` überführt werden können — Datenkorrektur auf bereits gecutoverten Live-Daten ist deutlich schmerzhafter als ein verzögerter Switch. Revidiert die v1.3-Out-of-Scope-Entscheidung "Halbtage / Stundenebene für Abwesenheiten" aus `REQUIREMENTS.md`. Granularität ist bewusst zweiwertig (ganz oder halb), kein AM/PM-Modell, keine Stundenebene — Stundenebene bleibt out-of-scope.

**Depends on:** Phase 8 (Absence-CRUD-Page Foundation — `AbsenceModal`, `AbsenceService`, `absence_period`-Tabelle, `AbsencePeriodTO`-DTO), Phase 8.1 Plans 01-11 (Cutover-Migration-UI bereits gebaut; 8.3 erweitert die existierende Drift-Resolution-Liste + `convert_quarantine_entry`/`bulk_convert_quarantine_rows`-Endpoints), Phase 8.2 (`ManualConvertModal` bekommt Halb/Ganz-Auswahl).

**Blocks:** Phase 8.1 Plan 08.1-12 (Phase-8-HUMAN-UAT-Subsumption / finaler Switch) — läuft erst, wenn 8.3 durch ist.

**Requirements:** **FUI-A-10** (Halbtag-Abwesenheiten — Backend-Modell + Frontend-Eingabe + Cutover-Mapping). Pflicht-Locale-Coverage (FUI-A-09) für neue i18n-Keys.

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. `absence_period`-Tabelle erweitert um `day_fraction`-Spalte (Migration additiv: `NOT NULL DEFAULT 'full'`). DAO + Entity-Mapping unterstützt das Feld. Plan-Phase entscheidet zwischen `TEXT`-Enum (`'full'|'half'`) und `INTEGER` (0/1).
2. `AbsencePeriodTO` (`rest-types/src/lib.rs:1565..2040`) bekommt `day_fraction: DayFractionTO`-Feld (Enum-DTO mit `ToSchema`); `AbsencePeriodCreateResultTO` unverändert. `AbsenceService::create_absence_period` + `update_absence_period` propagieren das Feld unverändert nach DAO.
3. `derive_hours_for_range` (`service_impl/src/absence.rs:483-…`) und Reporting-Aggregation berücksichtigen `day_fraction` — bei `Half` werden halbe Soll-Stunden pro Tag angerechnet. **Snapshot-Schema-Version-Bump:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` in `service_impl::billing_period_report` wird um eins erhöht (Begründung: Vacation-Computation-Logik ändert sich, alte Snapshots würden bei Re-Computation drift erzeugen — siehe `shifty-backend/CLAUDE.md` § "Billing Period Snapshot Schema Versioning").
4. Frontend `AbsenceModal` (`shifty-backend/shifty-dioxus/`) bekommt Halb/Ganz-Eingabe pro Buchung (UI-Form Plan-Phase-Decision: Checkbox vs. Dropdown). Bei Range > 1 Tag gilt `day_fraction` einheitlich für alle Tage der Range (klassischer Anwendungsfall: Range = 1 Tag).
5. Cutover-Migration-UI (`/admin/cutover`, Phase 8.1) bekommt pro Drift-Resolution-Eintrag Halb/Ganz-Auswahl. Per-Eintrag-Convert (`POST /admin/cutover/convert-quarantine-entry`) + Bulk-Convert (`POST /admin/cutover/bulk-convert-quarantine-rows`) + Manual-Convert (`POST /admin/cutover/convert-quarantine-entry` mit `manual_range`, Phase 8.2) akzeptieren `day_fraction`. Plan-Phase entscheidet zwischen Auto-Vorschlag aus Alt-Daten-Stunden (≈ 4 h → Halbtag) und rein manueller Toggle.
6. Konflikt-Logik: Halbtag-Absence + Booking am selben Tag wird **nicht** als Konflikt gewarnt (Booking gilt für die andere Tageshälfte). Plan-Phase entscheidet, ob `WarningTO` einen informativen Hinweis liefert oder schweigt.
7. OpenAPI-Surface-Test (`rest/tests/openapi_surface.rs` — `EXPECTED_SCHEMAS`) ergänzt um `DayFractionTO` + `day_fraction`-Feld auf `AbsencePeriodTO`. Schema-Drift-Test grün.
8. i18n De / En / Cs für Halb/Ganz-Labels in `AbsenceModal` + `CutoverAdminPage`-Drift-Resolution + `ManualConvertModal`. Per-Locale-Reference-Matcher-Tests analog Plan 08-04 D-26.
9. Backfill-Daten-Test: Bestehende `absence_period`-Einträge (vor Migration) bleiben unverändert (`Full`); ein Integration-Test verifiziert, dass `derive_hours_for_range` mit `day_fraction = Full` identische Resultate liefert wie vor der Schema-Erweiterung (no-drift-Garantie für bestehende Daten).
10. `cargo build --target wasm32-unknown-unknown` im `shifty-backend/shifty-dioxus/`-Subordner liefert Exit-Code 0 ohne Errors; `cargo check --workspace` + `cargo test --workspace` im Backend-Root grün. Keine Regression in Billing-Period-Snapshots existierender Phasen (alte Snapshots haben alte `snapshot_schema_version` und werden vom Validator korrekt als "older schema" markiert).

**UI hint**: yes (Backend-Erweiterung + Frontend-CRUD-Modal + Cutover-UI-Erweiterung).

**Notes for plan-phase:** Misch-Phase Backend + Frontend wie 8 / 8.1. Open Questions aus `.planning/notes/halftime-absence-decision.md` (Datenmodell-Form Enum vs. f32, Cutover-Auto-Vorschlag vs. manuelle Toggle, Frontend-UI-Pattern Checkbox vs. Dropdown, Konflikt-Warning-Verhalten, i18n-Keys) sind in der Plan-Phase zu entscheiden. **Service-Tier:** Erweiterung des bestehenden `AbsenceService` (Business-Logic-Tier, schon klassifiziert in Phase 8). `CutoverServiceImpl`-Erweiterung bleibt Business-Logic-Tier. **Snapshot-Schema-Versioning:** **Pflicht-Bump** der `CURRENT_SNAPSHOT_SCHEMA_VERSION` — Vacation-Aggregation ändert sich. **Reuse-Patterns:** 8.1-Drift-Resolution-Liste (`page/cutover_admin.rs`, Plan 08.1-09), `compute_gate_diagnostic` für `refreshed_drift_report`, 8.2-`ManualConvertModal` als Vorlage für Form-Erweiterung. **Out-of-Scope explizit:** AM/PM-Disambiguierung (separater Halbtag-Vormittag vs. Halbtag-Nachmittag), Stundenebene generell, Konflikt-Warning-Logik für Halbtag-Booking-Overlap (toleriert ohne Warning), Edit-Pfad für `day_fraction` auf bereits gecutoverten `absence_period`-Einträgen vor Phase-8.3-Schema (alle bestehenden Einträge sind `Full`; explizite Korrektur erfolgt über normalen Edit-Pfad). **VCS:** jj-only (siehe `CLAUDE.local.md`); Plans dürfen keine `git commit`-Befehle planen. **Cutover-Reihenfolge:** Plan-Phase muss klären, ob 8.3 vor Plan 08.1-12 als separate Phase abgeschlossen wird oder ob 08.1-12 in 8.3 subsumiert wird (vermutlich separat, weil 8.3 das Schema voraussetzt für die HUMAN-UAT-Re-Run).

**Plans (6 plans):**
- [x] 08.3-01-PLAN.md — Foundation: Migration + DAO/Service/DTO-Enums + OpenAPI-Surface + i18n-Key-Enum-Slots
- [x] 08.3-02-PLAN.md — DAO-SQLite threading: 6 SELECTs + INSERT + TryFrom + Service-CRUD + 2 In-Memory-Tests (no-drift + half-round-trip)
- [x] 08.3-03-PLAN.md — i18n: 13 add_text Bodies × 3 Locales (De/En/Cs) + 2 Presence-Tests + 6 Reference-Matcher-Tests (Pitfall-2-Guard)
- [x] 08.3-04-PLAN.md — Reporting Hot-Path: derive_hours_for_range Halbtag-Multiplikation + CURRENT_SNAPSHOT_SCHEMA_VERSION 3→4 + 3 Mockall-Tests
- [x] 08.3-05-PLAN.md — Cutover Backend: Request-DTOs + Service-Traits + Impls + REST-Handler + 3 Mockall-Tests + 1 REST-Integration-Test
- [x] 08.3-06-PLAN.md — Frontend: AbsenceModal + DriftEntryRow + ManualConvertModal + CutoverAction-Migration + 4 SSR-Snapshot-Tests + WASM-Build-Gate

**Wave-Struktur:**
- Wave 1 (parallel-eligible standalone): 08.3-01 (Foundation)
- Wave 2 (parallel — beide depend_on 08.3-01, keine file-overlap): 08.3-02 (Backend DAO+Service-Threading) ∥ 08.3-03 (i18n-Bodies + Tests)
- Wave 3 (parallel — disjoint Dependencies + Files): 08.3-04 (Reporting Hot-Path; depends_on 08.3-02) ∥ 08.3-05 (Cutover Backend; depends_on 08.3-01+02)
- Wave 4 (sequential — depends_on 08.3-01+03+05): 08.3-06 (Frontend + WASM-Gate)


---

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1 — Absence Domain Foundation | v1.0 | 5/5 | Complete | 2026-05-01 |
| 2 — Reporting Integration & Snapshot Versioning | v1.0 | 4/4 | Complete | 2026-05-02 |
| 3 — Booking & Shift-Plan Konflikt-Integration | v1.0 | 6/6 | Complete | 2026-05-02 |
| 4 — Migration & Cutover | v1.0 | 8/8 | Complete | 2026-05-03 |
| 5 — Slot Paid Capacity Warning | v1.1 | 6/6 | Complete | 2026-05-04 |
| 6 — rest-types Unification & Frontend Compile-Through | v1.2 | 5/5 | Complete | 2026-05-07 |
| 7 — Runtime Smoke & Regression Safety | v1.2 | 1/1 | Complete | 2026-05-07 |
| 8 — Absence-CRUD-Page Foundation | v1.3 | 8/9 | In Progress | — |
| 8.1 — Cutover-Migration-UI | v1.3 | 11/12 | ⊘ Superseded | 2026-06-09 |
| 8.2 — Manual-Range-Convert für Quarantäne | v1.3 | 2/2 | Complete | 2026-05-10 |
| 8.3 — Halbtag-Support für Absences | v1.3 | 6/6 | Complete | — |
| 8.4 — Reporting-Additiv-Merge + Deprecation-Rückbau | v1.3 | 0/? | Pending | — |
| 8.5 — Read-Projektion + HR-Inline-Convert | v1.3 | 0/? | Pending | — |
| 8.6 — Cutover-Abriss | v1.3 | 5/5 | Complete | 2026-06-11 |
| 9 — Booking-Flow Reverse-Warnings (Copy-Week descoped) | v1.3 | 2/2 | Complete | 2026-06-12 |
| 10 — Shiftplan-View Unavailability-Marker | v1.3 | 0/? | Pending | — |
| 11 — Migrations-Hinweis-UX + Deprecation-Handling | v1.3 | 0/? | ⊘ Superseded | 2026-06-09 |
| 12 — UI-Closure v1.1/v1.2-Restanten | v1.3 | 0/? | Pending | — |
| 13 — i18n-Vollständigkeits-Audit + v1.3 Smoke-Closure | v1.3 | 0/? | Pending | — |

---

*Last updated: 2026-06-09 — **Modell-Re-Scope:** Phase 8.1 (Batch-Cutover-Wizard) als ⊘ SUPERSEDED markiert; neue Phasen 8.4 (Reporting-Additiv-Merge + Deprecation-Rückbau), 8.5 (Read-Projektion + HR-Inline-Convert), 8.6 (Cutover-Abriss) eingesetzt. Grund: dauerhafte additive Koexistenz von `extra_hours` (manuelle Stunden-Ebene) + `absence_period` (Ranges) statt Big-Bang-Migration mit ratender Heuristik — eliminiert die Cutover-Unzuverlässigkeit (Karin-Pattern) an der Wurzel. Phase 11 (Deprecation-Handling) ebenfalls ⊘ SUPERSEDED (Rest vollständig in 8.5 SC 4 gefaltet). Decision-Stack → 08.4-CONTEXT.md (`/gsd:discuss-phase 8.4`). Phasen 10/12 referenzieren das alte Modell punktuell — bei Plan-Phase prüfen. — Vorheriger Stand: Phase 8.2 verified passed (6/6 must-haves, gsd-verifier 08.2-VERIFICATION.md). Plan 08.2-02 (Frontend ManualConvertModal) complete: shifty-dioxus api::cutover_convert_quarantine_entry um `manual_range: Option<ManualRangeTO>` erweitert (existing ConvertSingle call-site auf `None` migriert); neue `CutoverAction::ConvertSingleManualRange { extra_hours_id, start_date, end_date }`-Variante mit Coroutine-Branch (formatiert dates via time::macros::format_description, baut ManualRangeTO, ruft Backend, P-6 fallback auf separate gate-dry-run wenn refreshed_drift_report.is_none(), schreibt CUTOVER_STORE.last_dry_run + bump_cutover_refresh); ManualConvertModal-Component ersetzt EditExtraHoursModal-Stub (Custom-Backdrop, 2× `<input type="date">`, read-only amount + category als spans D-31/D-32, inline error-rendering, P-7 defense — kein unwrap_or_else hardcoded fallback); DriftEntryRow Edit-Button öffnet ManualConvertModal mit `category: drift_row_meta.0.category` als read-only Quelle (CutoverQuarantineEntryTO hat kein category-Feld); on_submit dispatcht ConvertSingleManualRange + close-on-submit. 8 neue i18n-Keys × 3 Locales (DE/EN/CS — `CutoverManualConvert{ModalTitle,Help,StartLabel,EndLabel,BtnSubmit,ErrStartAfterEnd,ErrYearMismatch,ErrOverlap}`) + Per-Locale-Reference-Matcher-Tests erweitert (Pitfall-2-Guard). 4 neue dioxus-ssr Snapshot-Tests (`manual_convert_modal_renders_two_date_inputs` / `manual_convert_modal_renders_validation_error_when_start_after_end` / `manual_convert_modal_not_rendered_when_closed` / `manual_convert_modal_dispatches_action_on_valid_submit`) ersetzen Test 11 aus 8.1-09 (`edit_extra_hours_modal_renders_amount_and_date_only`); 536/536 shifty-dioxus binary tests grün; cargo check --workspace grün; WASM-Build-Gate (`nix-shell -p openssl pkg-config lld --command "cargo build --target wasm32-unknown-unknown"`) exit 0. 3 jj-commits (`feat`/`feat`/`feat`). Phase 8.2 Plans complete (2/2). Karin-Pattern (gap-1a) jetzt operativ end-to-end auflösbar — UAT-bereit.*
