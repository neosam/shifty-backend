# Milestones — Shifty Backend

Historischer Index aller geshipten Milestones. Jeder Eintrag verlinkt auf die Detail-Archive in `.planning/milestones/`.

---

## v1.0 — Range-Based Absence Management

**Shipped:** 2026-05-03
**Phases:** 1–4 (4 phases, 23 plans, 22 SUMMARYs)
**Archive:** [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)

**Delivered:**
Zeitraum-basierte Abwesenheits-Domäne (Vacation/SickLeave/UnpaidLeave) ersetzt die wochenweise `ExtraHours`-Buchhaltung. Per-Tag-Stunden werden aus dem am jeweiligen Tag gültigen Vertrag abgeleitet (`derive_hours_for_range`); doppelte Eintragung zwischen ExtraHours und Shift-Plan entfällt. Komplette Bestandsdaten-Migration mit atomarer Cutover-Tx hinter Validierungs-Gate (Toleranz < 0.01h Drift).

**Key accomplishments:**

1. **Phase 1** — `absence_period`-Schema + DAO + Service + REST + 8 Integration-Tests; additiv ohne Reporting-Effekt; logical_id-Versionierung etabliert (später auch in `extra_hours` übernommen)
2. **Phase 2** — `derive_hours_for_range` (per-Tag-Vertrags-Lookup mit Feiertags-Orthogonalität), FeatureFlagService-Infrastruktur, atomarer Snapshot-Bump 2→3 + Reporting-Switch in einem Commit
3. **Phase 3** — Forward-/Reverse-Booking-Warnings über AbsencePeriod, ShiftplanView-Marker per-sales-person, BookingService-Files unangetastet (D-Phase3-18 Regression-Lock 0-Diff)
4. **Phase 4** — Heuristik-Cluster-Migration (Strict-Match + 5 Quarantäne-Reasons), atomarer Cutover (Backup → Carryover-Rebuild → Soft-Delete → Flag-Flip), 3 REST-Endpoints `/admin/cutover/{gate-dry-run,commit,profile}`, OpenAPI-Snapshot-Pin (160 KB, 3-Run-deterministic)
5. **Phase 4 Bonus** — `ExtraHoursService::update` mit logical_id-Rotation + REST `PUT /extra-hours/{id}` (recovered via jj-rebase nach fälschlicher phantom-Migration-Diagnose in Plan 04-02)

**Test verification:** 458+ tests green workspace-wide. OpenAPI snapshot deterministic. Cold-start smoke pass.

**Known deferred items:**

- 04-UAT Test 8 (idempotenter Re-Run nach Commit): manuell 403 erhalten, vermutlich Setup-Issue (kein cutover_admin-Grant in dev-DB); Code-Pfad abgedeckt durch passing Integration-Test `test_idempotence_rerun_no_op`.
- `/gsd:secure-phase 04` wurde nicht ausgeführt — als bewusstes Skip akzeptiert (Threats in Plan-SUMMARYs durchgängig als mitigated/accepted dokumentiert).

**Recovery note:**
Während Plan 04-02 wurde der frühere Commit `fe744df` (logical_id für extra_hours + PUT-Endpoint) fälschlich als "phantom from never-committed branch" interpretiert und seine Migration aus dem Workspace entfernt. Während des UAT-Reviews bemerkt, recovered via `jj rebase -r fe744dff -d @-`, Konflikte gelöst, OpenAPI-Snapshot re-akzeptiert. Im aktuellen v1.0-Lineup als Commit `psknryoq` (vor dem Phase-4-Verifikations-Commit) integriert.

---

## v1.1 — Slot Capacity & Constraints

**Shipped:** 2026-05-04
**Phases:** 5 (1 phase, 6 plans, 6 SUMMARYs)
**Archive:** [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)

**Delivered:**
Slots erhalten ein optionales Capacity-Limit `max_paid_employees: Option<u8>`. Wenn der Live-Count an aktiven Bookings im Slot mit `sales_person.is_paid = true` das konfigurierte Limit übersteigt, emittiert das Backend nicht-blockierende `Warning::PaidEmployeeLimitExceeded` über (a) `BookingCreateResult.warnings` im Conflict-Aware-Booking-Endpoint und (b) `current_paid_count` neben `max_paid_employees` per Slot im Shiftplan-Week-View. Buchen bleibt erlaubt — die Warning ist informativ (D-07: kein Tx-Rollback). `NULL`-Konfiguration bedeutet "kein Limit". Backend-additiv über alle Layer (SQLite-Migration, DAO, Service, REST). Frontend out of scope.

**Key accomplishments:**

1. **DAO Foundation** (Plan 05-01) — Nullable `slot.max_paid_employees INTEGER` ohne DEFAULT/NOT NULL, `SlotEntity.max_paid_employees: Option<u8>`, 4 SELECTs + INSERT + UPDATE in `dao_impl_sqlite/src/slot.rs`; Forward-Compat-Shims (Rule 3) für sequenzielle-Wave-2-Compile-Blocker.
2. **Service-Tier Wiring** (Plans 05-03, 05-04) — `service::slot::Slot.max_paid_employees`, `From` impls bridge DAO ↔ Service; `ShiftplanSlot.current_paid_count: u8` inline-derived in `build_shiftplan_day` aus bereits resolvten Bookings (Read-Aggregation-Pattern); Fixture-Migration in 5 Test-Files.
3. **Warning Surface** (Plans 05-02, 05-05) — 5. `Warning::PaidEmployeeLimitExceeded`-Variante mit 6 strukturierten Feldern (slot_id, booking_id, year, week, current_paid_count, max_paid_employees); REST-DTO-Mirror auf `SlotTO`/`WarningTO`/`ShiftplanSlotTO` mit `#[serde(default)]`-Backward-Compat (Wire-Tier-Mirror-Pattern); Wave-Coupling-Pattern für Producer-Consumer-Plans.
4. **Warning Emission** (Plan 05-06) — `ShiftplanEditService::book_slot_with_conflict_check` emittiert `Warning::PaidEmployeeLimitExceeded` nach Persistence (D-07: kein Rollback) wenn `current_paid_count > max` (D-06 strict, D-15 NULL-skip). Privater Helper `count_paid_bookings_in_slot_week` lebt im Business-Logic-Tier (D-12 + CLAUDE.md Service-Tier-Konvention + v1.0 D-Phase3-18 Regression-Lock).
5. **Regression-Lock gehalten** — Legacy `POST /booking` + `BookingService::create` UNVERÄNDERT (D-16, D-Phase3-18); verifiziert via `grep -c "PaidEmployeeLimitExceeded" service_impl/src/booking.rs` = 0 + `rest/src/booking.rs` = 0.

**Test verification:** 461 tests green workspace-wide (+6 über v1.0-Baseline 455). `cargo build --workspace` GREEN. `cargo run` boots cleanly to `127.0.0.1:3000`. `05-VERIFICATION.md`: 16/16 D-decisions verified, status: passed (gaps_remaining = []).

**Known deferred items:**

- **Frontend-Workstream (shifty-dioxus)** — bewusst out of scope. Capacity-Editor in Slot-Settings + UI-Anzeige `current_paid_count` / `max_paid_employees` folgen in separatem Repo.
- **Min-Paid-Capacity / Skill-Matching** — gemerkt für künftiges Backend-Milestone.

---

## v1.2 — Frontend rest-types Konsolidierung

**Shipped:** 2026-05-07
**Phases:** 6–7 (2 phases, 6 plans, 6 SUMMARYs)
**Archive:** [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)

**Delivered:**
Backend-`rest-types` ist die einzige Quelle der Wahrheit für API-DTOs. Der parallele Frontend-Fork `shifty-dioxus/rest-types/` ist gelöscht; `shifty-dioxus` zieht via Cross-Workspace-Path-Dep (`path = "../rest-types"`, `default-features = false`) gegen die Backend-Crate. Alle in CONCERNS.md §1 katalogisierten 17 fehlenden TOs/Enum-Varianten und 4 fehlenden Felder sind im Frontend referenzierbar; Match-Arme sind exhaustiv (rustc-enforced); `cargo build --target wasm32-unknown-unknown` liefert Exit-Code 0. Backend-Workspace bleibt regression-frei (466 Tests grün, ohne Re-Run-Effekt aus dem Cargo-Feature-Umbau). Keine User-facing Features hinzugefügt — explizit "Compile-Pfad freimachen für v1.3+ UI-Closure"-Scope.

**Key accomplishments:**

1. **Wave-0 Backend-Prep** (Plan 06-00) — Invitation-DTO-Familie (`InvitationStatus`, `InvitationResponse`, `GenerateInvitationRequest`) nach `rest-types` migriert mit konsistentem `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema`-Derive-Set; `shifty_utils`-Import feature-gated; `ShiftplanTO` mit `PartialEq, Eq` ergänzt. Backend swap-ready für Wave 1.
2. **Wave-1 Cargo-Swap + Fork-Delete** (Plan 06-01) — `shifty-dioxus/Cargo.toml` deklariert `[dependencies.rest-types] path = "../rest-types" default-features = false`; Verzeichnis `shifty-dioxus/rest-types/` aus dem Tree gelöscht (RT-01 + RT-02 erfüllt). Wave 2 öffnet sich mit den erwarteten Compile-Errors aus dem CONCERNS-§1-Katalog.
3. **Wave-2 Cluster-Fixes** (Plans 06-02, 06-03) — Frontend `Slot`-State erweitert um `max_paid_employees: Option<u8>` + `current_paid_count: u8` (state-only, kein Rendering — UI-Closure ist v1.3-Scope); `Weekday::from_num_from_monday`-Panic durch defensiven Fallback ersetzt; Cluster F (`invitation.redeemed_at`) auf Borrow-Form umgestellt nach Wave-0-`Option<OffsetDateTime> → Option<String>`-Migration.
4. **Wave-3 WASM-Compile-Closure** (Plan 06-04) — Cluster E (`SlotEditItem`-State-Mirror für `max_paid_employees` mit beiden `From`-Richtungen, preserve-on-edit-roundtrip) und Cluster H (`TemplateEngineTO PartialEq, Eq, Copy` für `assert_eq!`-Frontend-Tests) gefixt. WASM-Build grün. Visuelles Delta über die gesamte Phase 6 = 0 (UI-SPEC Regel 4 verifiziert via `jj diff` über `tailwind.config.js`, `input.css`, `src/i18n/`).
5. **Phase 7 — Runtime Smoke + Regression Safety** (Plan 07-00) — Alle 4 Phase-7-Success-Criteria verifiziert: dx serve / Login / Shiftplan-Navigation auf Integrationsumgebung (User-UAT 2026-05-07); Backend `cargo check --workspace` + `cargo test --workspace` re-verifiziert (Subsumption von Phase-6-VERIFICATION V-Truth #6 + #7 plus lokaler Re-Run zur Phase-Closure-Zeit). Subsumption-Pattern für reine Closure-Phasen etabliert.

**Test verification:** 466 tests green workspace-wide. `cargo build --target wasm32-unknown-unknown` GREEN. WASM-Artefakt 149 MB unter `shifty-dioxus/target/wasm32-unknown-unknown/debug/`. Frontend rendert auf Integrationsumgebung ohne Panic. 8/8 V-Truths verified (Phase 6) + 4/4 Success Criteria verified (Phase 7).

**Known deferred items (deferred to v1.3+):**

- **Frontend User-facing Closure (FUI-01..04)** — sichtbares `current_paid_count`/`max_paid_employees`-Rendering, Capacity-Editor in Slot-Settings, `VolunteerWork`/`UnpaidLeave`-UI, `cap_planned_hours_to_expected`-Settings. Compile-Pfad ist jetzt frei; v1.3 baut die UI darauf.
- **Frontend Abwesenheiten-Maske (FUI-A-01..09)** — Top-Level-Maske "Abwesenheiten" mit HR-Sicht + Employee-Self-Service gegen `/absence-period` REST-API. Mockup vorhanden (`shifty-dioxus/shifty-design/project/absences.jsx`, 729 Zeilen JSX). Briefing in `notes/abwesenheiten-frontend-context.md`; Seed `seeds/abwesenheiten-frontend-milestone.md` matcht beim `/gsd-new-milestone v1.3`-Start automatisch.
- **Min-Paid-Capacity / Skill-Matching (SC-01, SC-02)** — Backend-Slot-Constraints für künftiges Backend-Milestone.

---

## v1.4 — Committed Voluntary Capacity

**Shipped:** 2026-06-25
**Phases:** 14–17 (4 phases, 11 plans, 26 tasks)
**Archive:** [`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md) · [`milestones/v1.4-REQUIREMENTS.md`](milestones/v1.4-REQUIREMENTS.md) · [`milestones/v1.4-MILESTONE-AUDIT.md`](milestones/v1.4-MILESTONE-AUDIT.md)

**Delivered:**
Im Voraus zugesagte freiwillige Stunden-Kapazität wird pro Mitarbeiter über ein zeit-versioniertes Feld `committed_voluntary: f32` auf `EmployeeWorkDetails` (Variante B — entkoppelt von `expected_hours`) erfasst und in der Jahresansicht-Verfügbarkeit **ohne Doppelzählung** als separat ausgewiesene Kapazität ausgewertet. Die Zwei-Band-Dekomposition (Band 1 = cap-gated Σ Zusage, Band 2 = Σ Überschuss `max(actual−committed,0)`) lebt ausschließlich in Achse B (`booking_information.rs::get_weekly_summary`) und berührt keinen persistierten `BillingPeriodValueType` → **kein** Snapshot-Schema-Bump durch v1.4. Rein unbezahlte Freiwillige (`is_paid=false`) können einen Vertrags-Record halten und sind via „alle"-Filter sichtbar, ohne in `paid_hours`/Billing/Year-Summary zu leaken.

**Key accomplishments:**

1. **Phase 14** — Additive SQLite-Spalte `committed_voluntary REAL NOT NULL DEFAULT 0` end-to-end durch DAO (f64-Row + `as f32` TryFrom + 4 SELECT/INSERT/UPDATE), Service-Struct + beide Konversionen, `EmployeeWorkDetailsTO` (`#[serde(default)]`) + beide From-Impls; CVC-02 Carry-Forward-Spread bei Versions-Rotation; CVC-03 SUM-Overlap-Aggregation (`committed_voluntary_for_calendar_week`) gepinnt. Feld zunächst inert.
2. **Phase 15** — No-double-count: separater `committed_voluntary_hours`-Term in `booking_information.rs` (Achse B, FORMULA B), per-Person-Überschuss-Reduktion für `volunteer_hours` (Band 2), cap-gated (CVC-06); KEIN Snapshot-Bump durch v1.4 (CVC-05); 9 deterministische Fixtures + Regressionstest.
3. **Phase 16** — Jahresansicht zeigt drittes Token 🎯 „zugesagt" (Desktop + Mobile), drittes gestapeltes Chart-Segment `var(--good)`, Überschuss sichtbar; `committed_voluntary_hours` durch `WeeklySummaryTO` → Frontend-State → Render gefädelt; i18n De/En/Cs vollständig (CVC-07/08).
4. **Phase 17** — `committed_voluntary` im Vertrags-Editor editierbar (Round-Trip-bewahrend, CVC-09); einblendbarer „alle"-Filter für unbezahlte Freiwillige; jede paid-only work-details-Site explizit auf `sales_person.is_paid` gegated — kein Leak (CVC-10). Human-UAT live im Browser bestätigt.

**Test verification:** Backend `cargo check --workspace` GREEN, `cargo test -p service_impl` 451/451 + `rest-types` 3/3; Frontend `cargo check --target wasm32-unknown-unknown` GREEN, `cargo test` 628/628. Audit `passed` (10/10 Requirements, Integration intakt).

**Known deferred items (deferred to v1.5+):**

- **2 Human-UAT-Checks (Phase 16)** — visuelle Drei-Farben-Chart-Lesbarkeit + Czech-Übersetzungsqualität; nicht test-automatisierbar, bewusst beim Close acknowledged (siehe `milestones/v1.4-MILESTONE-AUDIT.md`).
- **CVC-F-01 / CVC-F-02** — Inline-Banner „Zusage nicht erfüllt"; eigenes committed-Band im Chart (CVC-F-02 wurde teilweise in Phase 16 vorgezogen).
- **AVG-01** — Auswertung „durchschnittliche Anwesenheit bei flexiblen Stunden" (eigene discuss-Phase).
- **Tech-Debt:** Nyquist-VALIDATION für Phasen 14/15/17 unvollständig (Discovery-only).

---

## v1.5 — Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen

**Shipped:** 2026-06-27
**Phases:** 18–23 (6 phases, 11 plans)
**Closeout:** override_closeout (acknowledged open items — siehe STATE.md Deferred Items)
**Archive:** [`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md) · [`milestones/v1.5-REQUIREMENTS.md`](milestones/v1.5-REQUIREMENTS.md)

**Delivered:**
Die verbliebenen Korrektheits- und Bedienprobleme der Abwesenheits-/Urlaubsverwaltung geschlossen: Carryover-Resturlaub stimmt jetzt zwischen Vacation-Balance und Report-Service überein (`year-1`-Quelle gepinnt), und `vacation_days` bleibt nach extra_hours→Absence-Konvertierung korrekt (derived Absences werden in die per-Woche-Kategorien gemergt, Single Source `by_week`, ohne Doppelzählung → Snapshot-Bump 9→10). Der „In Zeitraum umwandeln"-Dialog belegt das bis-Datum arbeitstagbasiert vor und erkennt den exakten 1-Wochen-Fall. Die Mitarbeiter-Jahresansicht ist schneller zuordenbar (KW+Datum-Hover/-Labels, gestapelte Freiwilligen-Stunden), HR bekommt eine HR-only Ø-Stunden/Woche-Statistik pro Person (urlaubsbereinigt), und zwei Tabellen wurden lesbarer (max-width + Zebra, schmalere Mitarbeiter-Spalte). Mitgeliefert: Frontend-UI für die Slot-Paid-Capacity (Editor + Overage-Warnfarbe) inkl. `modify_slot`-Bugfix.

**Key accomplishments:**

1. **Phase 18** — Carryover auf `year-1` gepinnt + per-Mock-Matcher gegen Reversion verriegelt (UV-04); derived Absences in per-Woche-`vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours` gemergt, Jahreslumpen-Doppelzählung entfernt (Single Source `by_week`), Snapshot-Bump 9→10 (UV-05).
2. **Phase 19** — `suggested_end` + `is_full_week` auf `ExtraHoursMarkerTO`, Backend-`suggest_convert_ranges_for_markers` (Arbeitstag/Feiertag/Wochen-Cap + Exakt-Wochen-Soll); Frontend belegt bis vor + „1 Woche"/„N Tage"-Anzeige (UV-01/02).
3. **Phase 20** — ⚠️-Indikator bei stundenbasierten Markern (UV-03); Histogramm mit KW+Datum-Hover/-Labels und gestapelten `volunteer_hours` + separatem Wert in der KW-Liste (YV-01/02/03).
4. **Phase 21** — `WorkingHoursMiniOverview` max-width + Zebra (UI-01); `/absences`-Mitarbeiter-Spalte `1.5fr`→`200px` an allen drei grid-cols (UI-02).
5. **Phase 22** — HR-gated `EmployeeWeeklyStatistics` + REST `GET /report/{id}/weekly-statistics` (Regel A-22-1: Jahr bis heute, voll-abwesende Wochen raus) + HR-only Frontend-Block (STAT-01/02; setzt Todo AVG-01 um).
6. **Phase 23** — Slot-Capacity-Editor (`max_paid_employees`, NULL = kein Limit) + Overage-Warnfarbe im Week-View; UAT-Bugfix: `modify_slot` ließ `max_paid_employees` fallen → gefixt + Regressionstest.

**Test verification:** Backend `cargo test --workspace` grün (inkl. neuer Regressionstests UV-04/UV-05/A-22-1); Frontend `cargo build --target wasm32-unknown-unknown` grün; v1.5-UAT-Polish im Browser bestätigt (Tabellenbreite, Wochen-Start Montag, Histogramm-Hover).

**Known deferred items (acknowledged at close, 2026-06-27):**

- **`carryover-absence-vs-report`** — Code-Fix ist drin (`vacation_balance.rs:225` → `year-1`, Tests grün) + Phase-18-Mock-Lock; Debug-Session-Status nur noch `awaiting_human_verify` (Browser-Bestätigung ausstehend, kein offener Code).
- **Historischer Quick-Task-/Todo-Ballast (pre-v1.4)** — seit v1.4-Close deferred, kein v1.5-Scope.
- **Tech-Debt:** Nyquist-VALIDATION einzelner v1.5-Frontend-Phasen optional/discovery-only.

---
