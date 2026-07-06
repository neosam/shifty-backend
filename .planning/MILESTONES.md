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

## v1.6 — Paid-Capacity-Durchsetzung & Konfiguration

**Shipped:** 2026-06-27
**Phases:** 24 (1 phase, 5 plans, 2 Waves)
**Closeout:** override_closeout (kein formaler Milestone-Audit; ein Human-UAT-Item bewusst deferred)
**Archive:** [`milestones/v1.6-ROADMAP.md`](milestones/v1.6-ROADMAP.md)

**Delivered:**
Die Paid-Capacity-Grenze (`max_paid_employees` pro Slot/Woche) wurde von einem rein visuellen Soft-Hinweis (v1.1/Phase 5, Phase 23) zu einem **global konfigurierbar durchsetzbaren Limit**. Ein admin-schaltbarer globaler Toggle (`paid_limit_hard_enforcement` über den bestehenden `ToggleService`, Default = weich → keine Regression) bestimmt, ob das Buchen über das Limit hinaus (a) hart blockiert wird — außer für die Shiftplanner-Rolle — oder (b) wie zuvor nur eine nicht-blockierende Warnung erzeugt. Der Hard-Block läuft pre-persist im Business-Logic-Tier (`ShiftplanEditService` mit frisch gelesenem Toggle vor `booking_service.create`), liefert einen unterscheidbaren `ServiceError::PaidLimitExceeded` (HTTP **409**, nicht 403) und eine lokalisierte Inline-Meldung. Eine persistente Overage-Warn-Sektion über dem Wochenplan macht Überschreitungen für **alle Rollen** sichtbar. Mitgefixt: das Buchungs-Permission-Gate von `HR ∨ self` auf `Shiftplanner ∨ self` (D-24-04). Alles für En/De/Cs lokalisiert.

**Key accomplishments:**

1. **24-01 — Error-Contract + Seed** — `ServiceError::PaidLimitExceeded { current, max }` → HTTP 409 in `rest/src/lib.rs` (+ OpenAPI-409-Annotation); Seed-Migration `20260627000000_seed-paid-limit-toggle.sql` (`INSERT OR IGNORE`, `enabled=0` = weich).
2. **24-02 — Enforcement + Gate-Fix** — Pre-Persist-Hard-Block in `book_slot_with_conflict_check` (`prospective > max`, Shiftplanner-Bypass, nur bezahlte zählen); `ToggleService` in `ShiftplanEditService` verdrahtet (D-24-08, Basic-vor-Business DI); Gate `HR ∨ self` → `Shiftplanner ∨ self`; 4 neue Hard-Block-Tests + migrierte Gate-Tests.
3. **24-03 — i18n** — 9 neue Keys (Settings-Toggle, Overage-Sektion, Block-Meldung) in En/De/Cs + Present-in-all-locales-Guard-Test.
4. **24-04 — Settings-Seite** — neue admin-gated `/settings/`-Route (`SettingsPage`, Component-Guard `has_privilege("admin")`) mit genau einem Paid-Limit-Toggle (`aria-pressed`, „Saved."/„Could not save setting."-Inline-Feedback); Toggle-REST-Client (`api`/`loader`) + Nav-Gating.
5. **24-05 — Shiftplan-UI** — Inline-409-Hard-Block-Meldung (D-24-05) + persistente Overage-Warn-Sektion über `ShiftplanTabBar` für alle Rollen (D-24-03).

**Test verification:** Backend `cargo build` + `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` grün; Frontend `cargo build --target wasm32-unknown-unknown` grün; 24-VERIFICATION.md 7/7 must-haves verified; Human-UAT 3/4 PASS. Git: Commit `e4ffbba`, 53 Dateien, +6855/−106.

**Known deferred items (acknowledged at close, 2026-06-27):**

- **Human-UAT #1 — Inline-Block-Platzierung** — die 409-Inline-Meldung rendert global unter der WeekView statt an der Slot-Zelle. Bewusst nicht im Browser getestet (User-Entscheidung „#2 fertig, #1 weglassen"); Backend-409-Logik durch 4 Unit-Tests abgedeckt. Nachbesserung der Platzierung offen.
- **Carry-over Deferred Items aus v1.4/v1.5** (historischer Quick-Task-/Todo-Ballast, Nyquist-VALIDATION-Lücken) — weiterhin deferred, siehe STATE.md → Deferred Items.

---

## v1.7 — Automatische Feiertage & Freiwilligen-Abwesenheit

**Shipped:** 2026-06-29 (Phasen complete & verified 2026-06-28; Milestone-Close 2026-06-29)
**Phases:** 25–26 (2 phases, 7 plans)
**Closeout:** override_closeout (Carry-over Deferred Items acknowledged; kein neuer Blocker)
**Archive:** [`milestones/v1.7-ROADMAP.md`](milestones/v1.7-ROADMAP.md) · [`milestones/v1.7-REQUIREMENTS.md`](milestones/v1.7-REQUIREMENTS.md)

**Delivered:**
Feiertage werden automatisch (statt manuell pro Mitarbeiter) im Mitarbeiterreport
angerechnet — mit **identischer** Wirkung zu einem manuellen `ExtraHours(Holiday)`
(reduziert `expected_hours`, erhöht Balance, `holiday_hours`-Spalte) und einem
admin-konfigurierbaren „aktiv ab"-Stichtag, der Vergangenheit (Snapshots, manuelle
Einträge) schützt und Doppelzählung verhindert. Berechnung **derive-on-read** (Toggle-
`value`-Spalte mit ISO-Cutoff + `SpecialDay`-Tabelle, keine `ExtraHours`-Rows). Urlaub/
Abwesenheit eines Freiwilligen (`is_paid=false`, `committed_voluntary>0`) reduziert seine
committed-Zusage 🎯 in der Jahresansicht (whole-week-out in `get_weekly_summary`);
Feiertage tun das bewusst **nicht** (Asymmetrie, per CI-Guard gepinnt). Bidirektionale
Deep-Links zwischen `/absences` und Mitarbeiterreport/Jahresansicht. Snapshot-Schema-
Version 10 → 11.

**Key accomplishments:**

1. **Phase 25-01** — Toggle-`value`-Spalte (nullable `TEXT`) end-to-end durch DAO/Service/REST + `holiday_auto_credit`-Seed; `GET/PUT/DELETE /toggle/{name}/value` toggle_admin-gated + ISO-Date-validiert; value-Presence treibt `enabled` (D-25-05).
2. **Phase 25-02** — derive-on-read Holiday-Auto-Credit in `ReportingService` (`build_derived_holiday_map`, 3 Injektionspunkte), Dual-Write `holiday_hours`+`absense_hours`, Snapshot-Bump 10→11, main.rs-DI.
3. **Phase 25-03/04** — admin-gated Settings-Date-Input (Save/Clear/Inline-Feedback) + 5 i18n-Keys de/en/cs; behaviorale Acceptance-Tests inkl. derived-vs-manuell-Vergleich + HOL-03-Regressions-Guard.
4. **Phase 26-01** — `AbsenceService`-DI in `BookingInformationService` + `period_overlaps_week`-Pure-Helper + whole-week-out-Reduktion (beide Bänder) in `get_weekly_summary`; 8 VFA-01-Tests.
5. **Phase 26-02/03** — VFA-02-Asymmetrie als full-service-Regressionstest + No-Snapshot-Bump-Guard (`==11`); Route `/absences/:employee_id` + `AbsencesFor`-Preselect (GlobalSignal) + 4 Ghost-Button-Cross-Links + 4 i18n-Keys de/en/cs.

**Test verification:** Backend `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` grün; Frontend WASM-Build grün. Beide Phasen `passed` (complete & verified 2026-06-28).

**Known deferred items (acknowledged at close, 2026-06-29):**

- **NAV-01-Deep-Links + Feiertags-Anzeige** — zum Phasen-Abschluss automatik-grün, aber nicht separat human-bestätigt (Carry-over im Deferred-Items-Pool).
- **REQUIREMENTS.md-Body-Checkboxen** (HCFG-02/HSNAP-01/NAV-01) blieben optisch `[ ]` trotz Verifikation — beim Close auf `[x]` nachgezogen (Doc-Drift).
- **Carry-over Deferred Items aus v1.4–v1.6** — weiterhin deferred, siehe STATE.md → Deferred Items.

---

## v1.8 — Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX)

**Shipped:** 2026-06-29 (beide Phasen VERIFIED inkl. Live-HR-Browser-Smokes)
**Phases:** 27–28 (2 phases, 5 plans)
**Closeout:** override_closeout (Milestone-Audit `passed`; Carry-over Deferred Items acknowledged)
**Audit:** ✅ passed (2/2 Requirements, 100% Integration, 2/2 Flows)
**Archive:** [`milestones/v1.8-ROADMAP.md`](milestones/v1.8-ROADMAP.md) · [`milestones/v1.8-REQUIREMENTS.md`](milestones/v1.8-REQUIREMENTS.md) · [`milestones/v1.8-MILESTONE-AUDIT.md`](milestones/v1.8-MILESTONE-AUDIT.md)

**Delivered:**
HR-UX rund um Abwesenheiten/Urlaub. Freiwillige (`is_paid=false`) sind in den
Abwesenheits-Selektoren auswählbar — gruppiert (native `optgroup` Angestellte/
Freiwillige) in **beiden** Call-Sites (AbsenceModal + AbsenceFilterBar) über einen
gemeinsamen Helfer, inaktive ausgeblendet, leere Gruppen ausgelassen, de/en/cs.
HR kann den berechneten Jahres-Urlaubsanspruch per signed **Offset (Korrektur-Delta)**
anpassen: `entitled_effective = round(berechnet) + offset`, pro Person+Jahr persistiert,
überlebt Vertragsänderungen (Delta statt Override), HR-gated CRUD + immer sichtbares
Inline-Editor-Feld („berechnet {n} + Offset [x]"); für normale User unsichtbar
(**API-level** Hiding — Self-View bekommt `offset`/`computed == None`). Begleitend:
Off-by-one-Proration-Fix (`vacation_days_for_year` year-START) + Snapshot-Schema-
Version-Bump 11 → 12 (`BillingPeriodValueType::VacationEntitlement`).

**Key accomplishments:**

1. **Phase 27-01** — Pure `grouped_selectable` + `PersonGroup` (Employees|Volunteers) + RSX-Helfer `grouped_person_options` (zwei `<optgroup>`s), beide Call-Sites umgestellt (kein Copy-Paste); 2 i18n-Keys de/en/cs; 5 Pure-Function-Tests. `is_selectable_employee` NICHT gelockert (D-27-02), Gruppierung nutzt eigenes `!inactive`-Predicate.
2. **Phase 28-01** — additive Migration + Tabelle `vacation_entitlement_offset` (partial unique index `WHERE deleted IS NULL`) + DAO + Basic HR-gated `VacationEntitlementOffsetService` (HR_PRIVILEGE auf jeder Methode) + CRUD/HR-gate-Tests.
3. **Phase 28-02** — Offset nach `.round()` addiert (Integer-Day-Korrektur, fließt in `remaining_days`); API-level Hiding (HR-only breakdown); `VacationBalanceTO`-Felder; HR-gated REST CRUD + ApiDoc; DI BL→Basic (kein Cycle).
4. **Phase 28-03** — Off-by-one year-START-Fix (`ordinal-1`) + Pflicht-Snapshot-Bump 11→12 + Guard-/Regressions-Tests.
5. **Phase 28-04** — FE Inline-HR-Offset-Editor (signed, on-blur/Enter, year-scoped) + User-Seite effective-only + `SaveOffset`-Action + i18n de/en/cs.

**Bonus-Bugfixes (Live-Smoke, committet):** `fix(28)` `/vacation-entitlement-offset` im `Dioxus.toml`-Dev-Proxy ergänzt (FE-Save lief auf 405); `fix` AbsenceModal schloss nach sauberem Create/Update nicht (`on_close` im No-Warnings-Zweig nachgezogen).

**Test verification:** Backend `cargo test --workspace` (inkl. neuer Offset/Balance/Off-by-one/Snapshot-Guard-Tests) + `cargo clippy --workspace -- -D warnings` grün; Frontend WASM-Build + 678 FE-Tests grün; `.sqlx`-Offline-Cache regeneriert; Migration additiv. Beide Live-HR-Browser-Smokes bestätigt (`behavior_unverified: 0`).

**Known deferred items (acknowledged at close, 2026-06-29):**

- **Phase-28-SUMMARY-Frontmatter** ohne `requirements-completed: [VAC-OFFSET-01]` (kosmetisch; voll durch 28-VERIFICATION abgedeckt).
- **DAO `find_by_id`** auf `VacationEntitlementOffsetDao` unkonsumiert — Forward-Hook für künftiges `DELETE /{id}`.
- **Carry-over Deferred Items aus v1.4–v1.6** (carryover-absence-vs-report awaiting_human_verify, Phase-24-Human-UAT #1, historischer Quick-Task-/Todo-Ballast, Nyquist-Lücken) — weiterhin deferred, siehe STATE.md → Deferred Items.

---

## v1.9 — Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation

**Shipped:** 2026-06-29 (autonomer Run; alle 4 Phasen VERIFIED, 2 optionale Browser-Smokes deferred)
**Phases:** 29–32 (4 phases, 6 plans)
**Closeout:** override_closeout (Milestone-Audit `passed`; Carry-over Deferred Items acknowledged)
**Audit:** ✅ passed (7/7 Requirements, 4/4 Integration, 4/4 E2E-Flows)
**Archive:** [`milestones/v1.9-ROADMAP.md`](milestones/v1.9-ROADMAP.md) · [`milestones/v1.9-REQUIREMENTS.md`](milestones/v1.9-REQUIREMENTS.md) · [`milestones/v1.9-MILESTONE-AUDIT.md`](milestones/v1.9-MILESTONE-AUDIT.md)

**Delivered:**
Drei betrieblich aufgefallene Schichtplan-/Urlaubs-UX-Lücken geschlossen und eine
vollwertige (lesend **und** schreibend) Admin-Impersonation mit Audit der echten
Admin-Identität ergänzt — alles frontend-zentriert, keine neuen Dependencies, kein
Snapshot-Bump, keine Migration. Der Pro-Person-Urlaubsbalken misst jetzt dieselbe Größe
wie die Resturlaub-Zahl (`(used+planned)/total`, Überzug per Farb-Signal). Die
Wochen-Summary-Karten zeigen beim schnellen Wochenwechsel nur noch die aktuelle Woche
(geteilter `(year,week)`-Staleness-Guard über alle Loader). Eigene/ausgewählte
Abwesenheits-Tage erscheinen proaktiv als „Nicht Verfügbar" im Schichtplan-Grid
(Kategorie-Set identisch zur Buchungs-Warnung, kein Drift). Admins können aus der
Users-Liste heraus impersonieren — persistenter, nicht-schließbarer Banner auf jeder
Seite (überlebt Reload), zentrale Audit-Middleware loggt jede schreibende Aktion mit
echter Admin-Identität, sauberer Store-Teardown beim Beenden, Admin-Gate gegen die rohe
Session-Identität (kein Privilege-Leak).

**Key accomplishments:**

1. **Phase 29 (VAC-01)** — Pure Helfer `compute_vacation_bar((used+planned)/total, clamp, low-flag)` aus dem Dioxus-Render extrahiert + verdrahtet in `PersonVacationCard`; Überzug per voller amber Balken + negativer Zahl (Farb-Signal, D-29-02); 6 Unit-Tests; Static-class-Pitfall-5 bewahrt.
2. **Phase 30 (SHP-02)** — Geteilter `(year,week)`-Guard: neues `week_guard.rs` (`SELECTED_WEEK` GlobalSignal + pure `is_current_selection`), synchron-vor-Dispatch gesetzt, Drop-on-Mismatch in **allen vier** Summary-Loadern (Code-Review fand den 4., `working_hours_mini`) + Render-Guard; 4 Prädikat-Tests.
3. **Phase 31 (SHP-01)** — `absence_marker.rs` pure Helfer `absence_periods_to_discourage_days` (alle 3 Kategorien, nur Ganztags — **byte-genau** zur `BookingOnAbsenceDay`-Warnung, null Drift) + guarded `reload_absence_days` (reused Phase-30-Guard) an 4 Triggern + Union-Merge in `discourage_weekdays`; 8 Tests; Scope = `current_sales_person`.
4. **Phase 32 (IMP-01..04)** — Backend: `RealUser`-Newtype in **beiden** `context_extractor`-Varianten injiziert + zentrale `audit_impersonated_writes`-Tower-Middleware (nach `context_extractor`) loggt `real_user`+`acting_as` für jede mutierende Anfrage + Start/Stop-Tracing + Two-Path/P10-Doku; Admin-Gate gegen rohe `session.user_id` (kein Privilege-Leak); 3 Integration-Tests (SC3/SC5/P10), 11 Session-Unit-Tests. Frontend: 3 `api.rs`-Calls + `service/impersonate.rs` (Store + `status_from_to` + Full-Reload-Teardown) + nicht-schließbarer Amber-Banner in `app.rs` (First-Init `LoadStatus`, überlebt Reload) + Users-Tab-„Act as"-Einstieg + i18n de/en/cs. **Keine** `Authentication<Context>`-Signatur-Änderung.

**Code review (alle adressiert):** P30 WR-01 (4. Loader `working_hours_mini` ungeschützt) gefixt; P31 IN-02 (symmetrischer Test) + IN-01 (Kommentar) gefixt; P32 WR-01 (Start-Tracing nach Erfolg), WR-02 (Stop-Tracing mit Target), WR-03 (Test-Kommentar) gefixt.

**Test verification:** Backend `cargo test --workspace` (inkl. 3 Impersonation-Integration- + 11 Session-Unit-Tests) + `cargo clippy --workspace -- -D warnings` grün (unabhängig re-verifiziert); Frontend `cargo build --target wasm32-unknown-unknown` + 705 FE-Tests grün. Kein Snapshot-Bump (bleibt 12), keine Migration, keine neuen Deps.

**Known deferred items (acknowledged at close, 2026-06-29):**

- **2 optionale Browser-Smokes** (P30 schnelles Wochen-Klicken, P32 Impersonation-Roundtrip) — nicht pixel-/timing-automatisierbar; strukturelle Korrektheit voll verifiziert; user-akzeptiert als deferred UAT.
- **Carry-over Deferred Items aus v1.4–v1.8** (carryover-absence-vs-report, Phase-24-UAT/Verification, historischer Quick-Task-/Todo-Ballast, Nyquist-Lücken) — weiterhin deferred, siehe STATE.md → Deferred Items. Keines v1.9-spezifisch.

**Hinweis:** v1.9 ist das interne Planungs-Label; der gesamte Code liegt uncommitted im Arbeitsbaum (jj manueller Commit durch User). Reale Release-Version datumsbasiert via `cli-update-version`.

---

## v1.10 — Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz

**Shipped:** 2026-06-30 (autonomer Run; alle 3 Phasen VERIFIED, SWO-01 live-browser-bestätigt)
**Phases:** 33–35 (3 phases, 8 plans)
**Closeout:** override_closeout (Milestone-Audit `passed`; Carry-over Deferred Items acknowledged)
**Audit:** ✅ passed (12/12 Requirements, Integration clean, 2/2 E2E-Flows)
**Archive:** [`milestones/v1.10-ROADMAP.md`](milestones/v1.10-ROADMAP.md) · [`milestones/v1.10-REQUIREMENTS.md`](milestones/v1.10-REQUIREMENTS.md) · [`v1.10-MILESTONE-AUDIT.md`](v1.10-MILESTONE-AUDIT.md)

**Delivered:**
Feiertage durchgängig korrekt gemacht — Special Days über die UI pflegbar **und** ihre
Soll-Wirkung auch in der Schichtplan-Wochentabelle sichtbar — plus eine Einzelwochen-Slot-
Ausnahme. Special Days (Holiday/ShortDay) sind **shiftplanner-gated** auf zwei Flächen voll-CRUD
pflegbar (Schichtplan-Wochenraster Per-Tag-Dropdown + Settings-Kalenderdatum-Picker + nach Jahr
gruppierte Liste mit abgeleitetem Kontext `15.08.2026 (Samstag, KW 33, 2026)`) gegen die
bestehende REST-CRUD plus einem neuen `for-year`-Read-Endpoint. Ein automatisch angerechneter
Feiertag reduziert das angezeigte Soll (`expected_hours`/`available_hours`/`holiday_hours`) auch
in der Wochentabelle unter dem Schichtplan — `get_week` bekommt einen 4. Injektionspunkt via
`build_derived_holiday_map` (derive-on-read, identisch zum Stundenkonto), während die
Kapazitätsbänder (`paid_hours`/`dynamic_hours`/`committed_voluntary`/`volunteer`) unangetastet
bleiben (D-25-08-Grenze). Slot-Werte (Kapazität/Zeiten) lassen sich für **genau eine KW** als
einmalige Ausnahme ändern via 3-Segment-Split+Re-Merge (atomar, eine Transaktion/Rollback,
Buchungs-Re-Point ohne Doppelzählung) mit UI-Wahl „nur diese Woche"/„ab dieser Woche".
**Kein Snapshot-Bump** (bleibt 12), **keine** Migration, **keine** neuen Dependencies.

**Key accomplishments:**

1. **Phase 33 (SPD-01..04)** — FE-Special-Days-Pflege auf zwei Flächen (Settings-Card-3
   Datepicker-Create + Jahres-Liste + Delete; Schichtplan Per-Tag-Dropdown Feiertag/Kurzer
   Tag/Nichts + ShortDay-Inline-Prompt), neuer Backend-`GET /special-days/for-year/{year}`-Read,
   18 i18n-Keys de/en/cs, shiftplanner-gated. **create-Pfad-Bug gefunden+gefixt** (FE POSTete
   `/special-days/` → Axum-0.8-404 → fix `/special-days`).

2. **Phase 34 (HSP-01..04)** — `get_week` 4. Injektionspunkt `holiday_derived_gated` reduziert
   nur `expected_hours`/`holiday_hours`, Bänder per Regressions-Guard geschützt; HOL-03-Test
   neu formuliert; HSP-04-Subtests (Stichtag-Gate + manueller-Holiday-gewinnt); Snapshot bleibt
   12 (grep-verifiziert). Re-verified nach CR-01-Gap-Closure (Cap-aktiver Band-Leak gefixt).

3. **Phase 35 (SWO-01..04)** — Backend `modify_slot_single_week` (3-Segment-Split + Booking-
   Partition + Atomarität + Gate `shiftplan.edit`) + REST-Route; FE single_week-State + api/loader-
   Pfad + Modus-Radiogruppe + 4 i18n-Keys de/en/cs. SWO-01 live-browser-bestätigt (3-Segment
   4/5/4, nur KW 27 geändert).

**Test verification:** Backend `cargo test --workspace` grün (526 Tests) + `cargo clippy
--workspace -- -D warnings` sauber; FE `cargo test -p shifty-dioxus` + `cargo build --target
wasm32-unknown-unknown` grün. **Regression-Gate während Phase 34 fand+fixte einen committeten
Cross-Phase-Blocker aus Phase 33** (`find_by_year`-Query fehlte im `.sqlx`-Offline-Cache → CI
wäre rot gewesen).

**Known deferred items (acknowledged at close, 2026-06-30):**

- **5 rein-visuelle Phase-33-Smokes** (WASM-Datepicker-Signal D-25-06, Add-Button-Disabled-
  Rendering, Jahres-Liste-Badges, Dropdown-onclick-Roundtrip, ShortDay-Inline-Prompt) — Backend-
  CRUD voll verifiziert, Dioxus-Interaktion nicht zuverlässig automatisierbar.

- **WR-02** (Code-Review WARNING, pre-existing) — `save_slot_edit` hält Write-Borrow über
  `.await`; nicht von Phase 35 eingeführt, als deferred FE-Borrow-Todo erfasst.

- **Carry-over Deferred Items aus v1.4–v1.9** — weiterhin deferred, siehe STATE.md. Keines
  v1.10-spezifisch.

**Hinweis:** v1.10 ist das interne Planungs-Label. Reale Release-Version datumsbasiert via
`cli-update-version` (kein v1.x git tag).

---

## v1.11 — Stabilisierung & UX-Politur

**Shipped:** 2026-07-01
**Phases:** 36–38 (3 phases, 6 plans)
**Archive:** [`milestones/v1.11-ROADMAP.md`](milestones/v1.11-ROADMAP.md) · [`milestones/v1.11-REQUIREMENTS.md`](milestones/v1.11-REQUIREMENTS.md) · [`milestones/v1.11-MILESTONE-AUDIT.md`](milestones/v1.11-MILESTONE-AUDIT.md)

**Delivered:**
Konsolidierung nach der v1.7–v1.10-Feature-Welle: vier gemeldete Bugs abgeräumt und der Frontend-Build warnungsfrei gemacht. Keine neuen Fähigkeiten, kein Snapshot-Bump (bleibt 12), keine Migration, keine neuen Deps.

**Key accomplishments:**

1. **Phase 36 (SDF-01/02)** — Special-Days-Bugfixes: der `create`-Service-Pfad ersetzt einen bestehenden gleich-datierten Special-Day-Eintrag jetzt atomar per in-place UPDATE (statt `ValidationError(Duplicate)`/HTTP 422), sodass der Feiertag↔Kurzer-Tag-Wechsel im Schichtplan fehlerfrei durchläuft (TDD, beide Richtungen). Settings-`SelectInput` bekam einen optionalen controlled `value`-Prop + Card-3-`sd_type`-Bindung, sodass der „Anlegen"-Button nach jedem Create wieder aktiv ist.
2. **Phase 37 (MOD-01/02)** — Modal-UX: zentrale drag-sichere Backdrop-Schließ-Logik in `dialog.rs` (`BackdropPress`-Signal-Flag-State-Machine, deckt alle 9 Dialog-Nutzer ab) + identischer Inline-Fix im eigenen Backdrop von `absence_convert_modal.rs`; ein innen begonnener, außen losgelassener Drag schließt nicht mehr. Arbeitsvertrag-Modal bekam pro Feld (außer Von/Bis) einen Erklärungssatz über 6 neue `*Help`-i18n-Keys in de/en/cs.
3. **Phase 38 (HYG-01/02)** — Frontend-Build-Hygiene: `shifty-dioxus` ist `cargo build`-warnungsfrei (14 auto-fix, 2 deprecated `time::parse` → `parse_borrowed::<2>`, ~34 Dead-Code gelöscht / 11 begründete `#[allow(dead_code)]`); Backend bleibt `cargo clippy --workspace -- -D warnings` grün. dioxus-Clippy bewusst out-of-scope.

**Test verification:** Backend `cargo test --workspace` (528 unit + 64 integration) + `cargo clippy --workspace -- -D warnings` grün; FE `cargo build` 0 Warnings + `cargo test -p shifty-dioxus` (727 pass; 1 pre-existing OOS) + WASM-Build grün. Milestone-Audit `passed` (6/6 Requirements, Integration clean, 4/4 Flows). Pro Phase Code-Review (0 Blocker).

**Known deferred items (acknowledged at close, 2026-07-01, override_closeout):**

- **SDF-02 + MOD-01 Browser-Smokes** (D-25-06-Klasse) — strukturell per SSR/Unit-Tests verifiziert; live-WASM-Interaktion optional deferred (User akzeptierte strukturelle Verifikation).
- **WR-01/36** (special-day replace nicht transaktional / kein UNIQUE-Index) — Fix bräuchte Migration → out-of-scope. **WR-02/36** (stale „already exists"-Hinweis) + **WR-02/37** (pre-existing `cancel_label`-Conditional) — Fix bräuchte i18n-Copy → out-of-scope; Folge-Fix-Kandidaten.
- **Pre-existing** FE-Test `i18n_impersonation_keys_match_german_reference` + ~198 dioxus-Clippy-Lints (dioxus aus CI-Clippy-Gate) — weiterhin deferred.
- **Carry-over Deferred Items aus v1.4–v1.10** — weiterhin deferred, siehe STATE.md. Keines v1.11-spezifisch.

**Hinweis:** v1.11 ist das interne Planungs-Label. Reale Release-Version datumsbasiert via `cli-update-version`/`/release-version` (kein v1.x git tag).

---

## v2.1 — Schichtplan- & Reporting-Erweiterungen

**Shipped:** 2026-07-02
**Phases:** 39–42 (4 phases, 14 plans)
**Closeout:** override_closeout (Milestone-Audit `passed`; Carry-over Deferred Items acknowledged)
**Audit:** ✅ passed (9/9 Requirements, Integration clean, 3/3 Flows)
**Archive:** [`milestones/v2.1-ROADMAP.md`](milestones/v2.1-ROADMAP.md) · [`milestones/v2.1-REQUIREMENTS.md`](milestones/v2.1-REQUIREMENTS.md) · [`milestones/v2.1-MILESTONE-AUDIT.md`](milestones/v2.1-MILESTONE-AUDIT.md)

**Delivered:**
Zwei neue Steuerungs-/Auswertungs-Fähigkeiten für die Schichtplanung plus ein isolierter
Settings-Bugfix. KW-Status (`None / In Planung / Geplant / Gesperrt`) pro ISO-(Jahr, Woche) —
shiftplanner-gated CRUD, farbkodiertes Badge für alle Rollen, Locked-Woche sperrt alle 6
Schreibpfade (`book_slot_with_conflict_check`, `modify_slot`, `modify_slot_single_week`,
`remove_slot`, `copy_week_with_conflict_check`, `delete_booking` inkl. REST-Re-Routing) via
TOCTOU-sicherem `assert_week_not_locked` in derselben Transaktion (HTTP 423); Schichtplaner
behält Vollzugriff. HR-gated Ø-Anwesenheit pro flexiblem Mitarbeiter (`is_dynamic == true`)
über einen Zeitraum, Urlaub aus Nenner herausgerechnet (reines Read-Aggregat, kein Snapshot-Bump,
Snapshot bleibt 12). Special-Days-„Anlegen"-Button-Fix (Option-2-Reset-Removal). Migration
nur in Phase 39 (`week_status`-Tabelle, partial UNIQUE). Keine neuen Dependencies.

**Key accomplishments:**

1. **Phase 39 (WST-01/02/05)** — `week_status`-Tabelle + Migration (partial UNIQUE) + `WeekStatusDao` + `WeekStatusService` (Basic-Tier, TDD: Permission-Gate, Upsert/Soft-Delete, KW-53); `WeekStatusTO` + REST-CRUD + ApiDoc + DI-Wiring; FE: `WeekStatus`-Enum (4 Varianten inkl. `Unset`, i18n de/en/cs), Fresh-Fetch-Store, `WeekStatusBadge` + `WeekStatusDropdown` in der Schichtplan-Wochenansicht. `should_show_badge` pure-fn. Code-Review 0 Blocker.
2. **Phase 40 (WST-03/04)** — `ServiceError::WeekLocked` → HTTP 423 + OpenAPI-Annotation; `assert_week_not_locked`-Helper in allen 6 Schreibpfaden in-Transaktion (kein TOCTOU); neue `ShiftplanEditService::delete_booking`-Methode + REST-Re-Routing `DELETE /booking/{id}` (schließt Basic-Tier-Bypass); Shiftplanner-Bypass; FE read-only Locked-Woche + 423-Inline-Banner; i18n de/en/cs. **CRITICAL CR-01 (Privileg-Mismatch) via Code-Review gefunden + gefixt** + Regressionstest. Test-Matrix 6 Pfade × {gesperrt, offen}.
3. **Phase 41 (AVG-01/02/03)** — Pure fn `average_hours_per_attendance_day` (eigener Struct `EmployeeAttendanceStatistics`, DISTINCT-Date-BTreeSet, ≥1 work-category, <2 Tage → None); `ReportingService::get_employee_attendance_statistics` (HR-Gate als erste await-Op, `is_dynamic`-Filter, `until_week`-Clamp); `EmployeeAttendanceStatisticsTO` + HR-gated Endpoint + ApiDoc; FE Ø-Anwesenheit-Sektion im HR-Report mit Leerzustand; i18n de/en/cs. Snapshot bleibt 12 (grep-verifiziert).
4. **Phase 42 (SDF-01)** — Reset-Block `settings.rs:458-459` entfernt (Option 2); reine Validitäts-/Retention-Fns `is_special_day_form_valid` + `special_day_form_after_create` extrahiert + unit-getestet; stale Doc-Comment gefixt. SSR-Mount-Test begründet übersprungen (D-42-06).

**Test verification:** Backend `cargo test --workspace` (569 service_impl + 64 rest + weitere, 0 Failures) + `cargo clippy --workspace -- -D warnings` clean; FE `cargo build --target wasm32-unknown-unknown` warnungsfrei + `cargo test -p shifty-dioxus` 752 grün. Audit `passed` (9/9 Requirements, Integration clean, 3/3 E2E-Flows, Nyquist compliant).

**Known deferred items (acknowledged at close, 2026-07-02, override_closeout):**

- **3 optionale D-25-06-Browser-Smokes** (Phase 40: +/- Buttons weg in Locked-Woche; Phase 41: Ø-Anwesenheits-Zahl im HR-Report; Phase 42: Button bleibt aktiv nach Create) — strukturell via pure-fn + Endpoint-Tests verifiziert; live-WASM-Interaktion optional deferred.
- **WR-03 (akzeptiert):** `is_dynamic`-Filter ohne Report-Perioden-Bezug — konsistent mit `billing_period_report.rs`-Muster; zeitraum-bewusste Vertragshistorie out-of-scope (AVG-05 Backlog).
- **PRÄ-v2.1 Carry-over** (i18n_impersonation_keys_match_german_reference aus v1.11/Phase 37-02, Commit 83a0d91): De-Label '🥸 Agieren' vs Test-Referenz 'Als diese Person agieren' — Produkt-Copy-Entscheidung offen; Todo: `.planning/todos/pending/2026-07-02-i18n-impersonation-key-test-mismatch.md`.
- **Carry-over Deferred Items aus v1.4–v1.11** — weiterhin deferred, siehe STATE.md. Keines v2.1-spezifisch.

**Hinweis:** v2.1 ist das interne Planungs-Label. Reale Release-Version datumsbasiert via `cli-update-version`/`/release-version` (kein git tag).

---

## v2.2 — Aufräumen, WebDAV-Export & Wochentag-Muster

**Shipped:** 2026-07-03
**Phases:** 43–48 (6 phases, 16 plans)
**Closeout:** override_closeout (Milestone-Audit `passed`; 13 Carry-over-Items aus dem Pre-Close-Audit acknowledged, keines v2.2-spezifisch)
**Audit:** ✅ passed (16/16 Requirements, Integration clean, 3/3 E2E-Flows)
**Archive:** [`milestones/v2.2-ROADMAP.md`](milestones/v2.2-ROADMAP.md) · [`milestones/v2.2-REQUIREMENTS.md`](milestones/v2.2-REQUIREMENTS.md) · [`milestones/v2.2-MILESTONE-AUDIT.md`](milestones/v2.2-MILESTONE-AUDIT.md)

**Delivered:**
Aufräum-Milestone mit zwei substantiellen Features. Verbliebene v1.x-Carry-over-Bugs
(SDF-03/04/05 Special-Days-Feintuning; BUG-01/02/03 Frontend-Korrektheit) und
Hygiene-Themen (HYG-03 Dioxus-Warnings, HYG-04 „Edit structure"-i18n, HYG-05
REST-Content-Type-Drift-Guard, IMP-05 i18n-Impersonation-Test) geschlossen. Die
v2.1-Ø-Std/Anwesenheitstag-Kennzahl wurde durch eine pro-Wochentag-Muster-Anzeige
(count + %) im HR-Stats-Block ersetzt (RPT-01/02/03, reines Read-Aggregat, Snapshot
bleibt 12). Neu: regelmäßiger Nextcloud-PDF-Export via WebDAV (EXP-01/02/03) —
Backend-Task rendert pro Kalenderwoche ein PDF (`printpdf`, deterministisch), pusht
per WebDAV (`reqwest_dav`, MKCOL+PUT, 3× Retry 2s/4s/8s), Cron-Scheduler
(`tokio-cron-scheduler`), admin-gated Settings-Card mit Config + „Jetzt
exportieren"-Button + Status. Migration nur in Phase 48 (`pdf_export_config`
Single-Row). Snapshot bleibt 12 (kein Bump).

**Key accomplishments:**

1. **Phase 43 (SDF-03/04/05)** — Special-Days-Feintuning: `sd_year_after_create`
   Kalenderjahr-Loader-Fix (`date.year()` statt `iso_year` → 1.1.-Anzeige-Bug
   behoben), i18n-Copy Duplikat-Hinweis auf Replace-Verhalten umgestellt (de/en/cs,
   Anti-Wording-Test), Feiertag↔Kurzer-Tag-Umschalter im Wochenraster wirft keinen
   Fehler mehr (Backend-Roundtrip-Test kettet zwei Ersetzungen, FE
   `special_day_error_after_create` pure fn).

2. **Phase 44 (BUG-01/02/03)** — Frontend-Korrektheit: `save_slot_edit` auf
   Snapshot-vor-`.await` + Pure-fn-Outcome-Apply refaktoriert (kein
   `SLOT_EDIT_STORE`-Write-Borrow über `.await`, 6 Regressionstests),
   `ShiftyError::InvitationParse(String)` + Inline-Banner + i18n für sichtbaren
   Invitation-Parse-Fehler (kein silent-empty), durable Grep-Invariant-Test
   `mod backdrop_invariant` in `dialog.rs` (jedes Modal mit `fixed inset-0` muss
   `BackdropPress` nutzen).

3. **Phase 45 (HYG-03)** — shifty-dioxus Warnings-Aufräumen: 177 → 0 Clippy-Warnings
   (auto-fix erste Welle, dann manuelle Kategorien mit begründeten `#[allow]`s bei
   API-brechenden Fällen); FE-Clippy-Gate `-D warnings` erstmals scharfgestellt;
   Backend-Clippy-Gate unberührt grün.

4. **Phase 46 (HYG-04/HYG-05/IMP-05)** — Backend-Hygiene & i18n: 3 neue i18n-Keys
   (`Shiftplan{EditStructure,NormalMode,NewSlot}`) in de/en/cs + Call-Sites in
   `page/shiftplan.rs`; OpenAPI-Reflection-Test `rest/tests/content_type_surface.rs`
   iteriert 120 utoipa-Operationen + Whitelist (application/json + text/plain) +
   Grandfather-Liste für 13 pre-existing Handler (Content-Type-Drift-Guard);
   `i18n_impersonation_keys_match_german_reference`-Test auf shipped 🥸-Copy
   angepasst (3/3 Impersonation-Tests grün).

5. **Phase 47 (RPT-01/02/03)** — Wochentag-Anwesenheits-Muster: pure fn
   `weekday_attendance_distribution` + Endpoint-Umbau
   (`EmployeeAttendanceStatisticsTO.attendance_by_weekday + counted_calendar_weeks`),
   HR-Gate + `is_dynamic`-Filter preserved, alte `average_hours_per_attendance_day`
   grep-verifiziert entfernt (0 Hits); FE-Formatter `format_weekday_attendance_line`
   („Mo: 8 (80 %) · …") + i18n de/en/cs (9 neue Keys: 7 Wochentag-Shortcuts,
   Tooltip, Leerzustand) + SSR-Tests; Snapshot bleibt 12 (grep-verifiziert).

6. **Phase 48 (EXP-01/02/03)** — Nextcloud-PDF-Export via WebDAV: Migration
   `pdf_export_config` (Single-Row, fixe UUID-PK, INSERT-OR-IGNORE-Seed) + Basic-Tier
   `PdfExportConfigService` (admin-gated get/update, Full-Auth-only
   record_success/record_error) + REST GET/PUT mit Token-Maskierung;
   `pdf_render.rs` mit `printpdf` (Landscape A4, deterministisch, 10 TDD-Tests);
   `webdav_client.rs` mit `reqwest_dav` (MKCOL+PUT, 3× Exp-Backoff 2s/4s/8s,
   Token-Leak-Guard im Debug-Impl, rustls-only, wiremock-Tests);
   `PdfExportSchedulerImpl` mit `tokio-cron-scheduler` (Config-Reload via PUT-Hook,
   POST /trigger-Endpoint, Boot-Wiring, 6 behavior-Unit-Tests + 1 E2E
   `boot_trigger_reload_flow`); admin-gated Settings-Card 4 in FE (Toggle + 6
   Felder + Save + „Jetzt exportieren" + Status-Anzeige, 19 neue i18n-Keys in
   de/en/cs).

**Test verification:** Backend `cargo test --workspace` grün; Backend
`cargo clippy --workspace -- -D warnings` clean; FE `cargo build --target
wasm32-unknown-unknown` warnungsfrei; FE `cargo test -p shifty-dioxus` **787
grün** (inkl. der zuvor gebrochenen `i18n_impersonation_keys_match_german_reference`
via IMP-05); FE `cargo clippy -p shifty-dioxus -- -D warnings` erstmals grün.
Audit `passed` (16/16 Requirements, Integration clean, 3/3 E2E-Flows). Neue Deps
(`printpdf`, `reqwest_dav`, `tokio-cron-scheduler`), eine neue Migration
(`pdf_export_config`), kein Snapshot-Bump.

**Known deferred items (acknowledged at close, 2026-07-03, override_closeout):**

- **Pre-existing `doc_lazy_continuation`-Clippy-Warning** (Phase-40-Ursprung, aus
  v2.1) in `service_impl/src/test/shiftplan_edit_lock.rs:6` — fires nur mit
  `--all-targets`, nicht Teil des Standard-Clippy-Gates. Non-blocking, trivialer
  Folge-Fix.
- **Phase 45 Scope-Caveat:** 13 pre-existing `#[allow(dead_code)]`/
  `#[allow(non_snake_case)]` ohne reason-Kommentar bleiben — Clippy `-D warnings`
  trotzdem grün. Optionale Nach-Hygiene-Runde.
- **Phase 48 Deviation:** PDF-Determinismus-Test normalisiert die `/ID`-Array-Bytes
  test-seitig (dokumentiert im SUMMARY) — Payload/Layout byte-gleich, nur der
  printpdf-internal `/ID`-Fingerprint runspezifisch.
- **Pre-Close-Audit 2026-07-03**: 13 Carry-over-Items (1 debug session
  `awaiting_human_verify`, 7 Quick-Tasks, 5+ Pending Todos aus Mai/Juni 2026) —
  historischer Ballast aus v1.4–v2.1, in v2.2 nicht in Scope gezogen, weiterhin in
  STATE.md → Deferred Items.

**Post-Ship-Bugfixes (2026-07-03, nach Milestone-Close, vor Release-Tag):**

Drei UAT-Findings vom User nachträglich in v2.2 gefixt (kein separater
Milestone, weil noch nicht getagged):

1. **SDF-03 Kalender-Jahr-Semantik (nachziehen)** — der Phase-43-Fix hatte nur
   `sd_year.set()` nach einem Create korrigiert, nicht den DB-Filter.
   Ein 01.01.2027-Eintrag wurde als ISO-Wochenjahr `(2026, W53, Fri)` gespeichert
   und war im 2026er-Filter sichtbar, im 2027er-Filter unsichtbar.
   Fix: `SpecialDayServiceImpl::get_by_year(year)` lädt jetzt `year` UND
   `year - 1`, filtert per `ShiftyDate::to_date().year() == year` (Kalenderjahr)
   und sortiert nach Kalenderdatum aufsteigend, sodass 01.01.YYYY oben statt am
   Ende der Liste steht. Tests: `test_get_by_year_delegates_and_maps` (Mid-Year-
   Basis-Fall angepasst auf 2026-W10-Mo) + neuer
   `test_get_by_year_returns_new_year_day_under_calendar_year` (positive:
   01.01.2027 ist in `get_by_year(2027)` sichtbar UND sortiert VOR 11.01.2027;
   negative: NICHT sichtbar in `get_by_year(2026)`).
   Files: `service_impl/src/special_days.rs`, `service_impl/src/test/special_days.rs`.

2. **RPT-02 Sichtbarkeit + Label** — die Wochentag-Zeile wurde nur für flexible
   Employees mit HR-Rolle gerendert (`is_dynamic`-Filter). User wollte sie für
   ALLE Employees sehen. Fix: `is_dynamic`-Gate in
   `ReportingServiceImpl::get_employee_attendance_statistics` entfernt (HR-Gate
   bleibt). Zusätzlich UX-Fix: neuer i18n-Key `WeekdayAttendanceLabel`
   (de: „Anwesenheit / Tag", en: „Attendance / day", cs: „Docházka / den") als
   kurzes TupleRow-Label; der lange Text bleibt als `title`-Tooltip. Test
   `attendance_statistics_returns_none_for_static` → durch
   `attendance_statistics_returns_some_for_static_after_rpt02` ersetzt.
   Files: `service_impl/src/reporting.rs`,
   `service_impl/src/test/reporting_attendance_gate.rs`,
   `shifty-dioxus/src/component/employee_view.rs`,
   `shifty-dioxus/src/i18n/{mod,de,en,cs}.rs`.

3. **+ Button Refresh (nicht reproduzierbar)** — User meldete Full-Page-Reload
   beim + Button auf Absent-Markierten Montags-Slots. Bei Repro in Chrome (via
   `mcp__claude-in-chrome`) trat der Reload weder mit programmatischem
   `.click()` noch mit voller MouseEvent-Chain auf. Handler-Kette geprüft:
   `r#type: "button"`, `evt.stop_propagation()`, kein Ancestor-`<form>`/`<a>`,
   `update_shiftplan()` nutzt keinen Router-Push, `block_error` rendert nur
   Banner ohne Redirect. Nach kurzem Retest durch den User selbst nicht mehr
   auftretend — vermutlich transient (dx-serve Live-Reload während meiner
   Backend-Änderungen). Kein Code-Fix; falls Bug wiederkehrt: Console-Log beim
   Click liefert den Panic-Ort.

Alle Gates nach Post-Ship-Fixes: Backend `cargo test --workspace` grün,
`cargo clippy --workspace -- -D warnings` grün, `cargo build
--target wasm32-unknown-unknown` grün.

**Hinweis:** v2.2 ist das interne Planungs-Label. Reale Release-Version via
`/release-version` (SemVer-Tag wird dort gesetzt, `git.create_tag=false` in GSD-Config).

---

## v2.3 — PDF-Export: Browser-Look & Download-Button

**Shipped:** 2026-07-04
**Phases:** 49–50 (2 phases, 8 plans, 8 SUMMARYs)
**Archive:** [`milestones/v2.3-ROADMAP.md`](milestones/v2.3-ROADMAP.md)

**Delivered:**
Kleiner Fix-Milestone auf dem v2.2-PDF-Export. Der v2.2-Renderer produzierte
praktisch unlesbare PDFs (starres mm-Absolut-Layout, keine sichtbaren Slot-
Zellen, keine Uhrzeiten). v2.3 tauschte den Renderer gegen eine browser-
ähnliche Wochenansicht (Slots als Zellen mit Uhrzeit-Label + Namen, sieben
Wochentag-Spalten, Landscape A4, Header „Schichtplan KW {NN} ({JJJJ})",
Renderzeitpunkt „Erstellt am DD.MM.YYYY HH:MM Uhr" auf jeder Seite) und
legte einen On-Demand-Download-Button neben dem iCal-Button auf die
Schichtplan-Seite (visibility-gated auf `WeekStatus ∈ {Planned, Locked}`).
WebDAV-Scheduler aus Phase 48 nutzt den neuen Renderer automatisch via
`PdfShiftplanService`-Delegation. Kein Snapshot-Bump (bleibt 12), keine
Migration, keine neue Cargo-Dep — nur das `local-offset`-Feature auf
existierendem `time`-Crate.

**Key accomplishments:**

1. **Phase 49 — On-Demand-Download-Button (BE + FE)** — Neuer REST-Endpoint
   `GET /shiftplan/{id}/{year}/{week}/pdf` mit Auth-Gate (kein Admin-Gate,
   Employee-Auth liefert 200) und `WeekStatus`-Defense-in-Depth-409
   (`week-not-releasable` JSON-Error-Code, D-49-03).
   `PdfShiftplanService` als Business-Logic-Tier-Assembler (ShiftplanView +
   SalesPerson + WeekStatus + pdf_render); `PdfExportScheduler` refactored
   zu Delegation an denselben Service (D-49-08). Frontend-Anchor neben iCal
   mit pure Predicate `should_show_pdf_button(status, shiftplan_id) -> bool`
   (8-case Test-Matrix, D-49-13); i18n-Key `PdfDownload` in de/en/cs.
   Dateiname `schichtplan-{JJJJ}-KW{NN}.pdf` via `filename_for()` DRY-Helper.

2. **Phase 50 — PDF-Renderer neu: Browser-Look + Timestamp** — Kompletter
   Rewrite von `service_impl/src/pdf_render.rs`. Neue 5-Parameter-Signatur
   `render_shiftplan_week_pdf(week, sales_persons, header_year, header_week,
   render_timestamp: OffsetDateTime)` (D-50-11) — Renderer bleibt pure Funktion.
   Hybrid-Stack-Layout (base + duration_hours × step, D-50-01/02); sichtbare
   Slot-Rahmen via `add_rect(Rect::with_mode(PaintMode::Stroke))` +
   `save_graphics_state`/`set_outline_thickness`/`restore_graphics_state`
   (D-50-10); dynamische Sonntag-Spalte (D-50-08); Header-Timestamp
   oben-rechts (D-50-09); alphabetische Namen (D-50-06/07); Overflow-Marker
   „+ N weitere" (D-50-03/04). `resolve_render_timestamp()` mit
   `time::OffsetDateTime::now_local()` + UTC-Fallback + `tracing::warn!`-Log
   bei `IndeterminateOffset` (D-50-12). `local-offset`-Cargo-Feature auf
   existierendem `time` — keine neue Crate.

3. **Human UAT bestätigt (D-50-17)** — visueller Layout-Check gegen reale
   Woche via Phase-49-Button. Erfolgskriterium PDF-01 „Ausdruck ohne
   Digital-Referenz nutzbar" erreicht. Post-UAT-User-Feedback in Fix-Commits
   umgesetzt: `+ N weitere` Overflow-Marker (D-50-03/04) entfernt (Boxen
   wachsen mit, Namen komma-separiert); `(freiwillig)`-Suffix (D-50-06/07)
   entfernt (paid/unpaid irrelevant im PDF, Fix-Commit `a484f74`); Slot-Boxen
   row-aligned über alle Tages-Spalten (`e8c4a83`); tighterer Layout für
   ~2 mehr Slots pro Spalte (`1c1f5df`).

4. **Post-Ship-Hotfix v2.3.1** — `fix(pdf-export): tolerate per-week
   ValidationError + fix cron seed to 6-field` (Commit `754f94f`).
   WebDAV-Scheduler ignoriert jetzt einzelne Wochen mit `ValidationError`
   statt komplett zu fallen; Cron-Seed-Format korrigiert auf 6-Feld
   (`tokio-cron-scheduler`-Erwartung).

5. **Byte-Determinismus bewusst gebrochen** — v2.2-Vertrag (fixe Metadata
   2000-01-01) durch PDF-02 (Timestamp) obsolet. WebDAV-Overwrite bleibt
   korrekt; kein Scheduler-Code-Change nötig. `printpdf`-Crate unverändert,
   keine neuen Deps.

**Test verification:** 781 Tests grün workspace-wide.
`cargo clippy --workspace -- -D warnings` grün. Verifier PASSED 14/14 in
Phase 50. Human UAT D-50-17 bestätigt 2026-07-04.

**Known deferred items:**

Keine v2.3-spezifischen. Historische Deferred-Items siehe
`.planning/STATE.md` „Deferred Items"-Sektion.

**Hinweis:** v2.3 ist das interne Planungs-Label. Reale Release-Versionen
`v2.3.0` und `v2.3.1` (Hotfix) via `/release-version` gesetzt
(`git.create_tag=false` in GSD-Config, Tags aus SemVer-Flow).

---

## v2.4 — Kurzer-Tag-Slot-Kürzung

**Shipped:** 2026-07-05
**Phases:** 51 (1 phase, 8 plans, 8 SUMMARYs)
**Archive:** [`milestones/v2.4-ROADMAP.md`](milestones/v2.4-ROADMAP.md)

**Delivered:**
Fokus-Milestone auf einer einzelnen Semantik-Ergänzung. An Kurzen Tagen
(`special_day.ShortDay` mit Cutoff-Uhrzeit) werden Slots, die den Cutoff
überlappen, dynamisch auf `[slot.start, cutoff]` gekürzt — in Rendering
(WeekView + PDF) und Ist-Stunden-Berechnung (Reporting +
Booking-Information + Balance). Slots komplett hinter dem Cutoff
verschwinden. Soll-Stunden bleiben unverändert (Balance-Konto sammelt
Minusstunden an Kurzen Tagen). Ein admin-konfigurierbarer Stichtag
(`shortday_slot_clipping_active_from`) schützt historische
Balance-Views: ohne Wert deaktiviert, mit Wert wirkt Kürzung nur für
`booking_date >= active_from`. View-layer / dynamisch — keine
DB-Änderung an Slots oder Bookings; kein Snapshot-Bump; nur additive
Toggle-Seed-Migration; keine neue Cargo-Dep. Fat Backend, Thin Client
(D-51-02): das Frontend enthält null Clip-Logik.

**Key accomplishments:**

1. **Kanonische `Slot::clip_to` als pure Value-Methode** auf
   `service::slot::Slot` (D-51-01), mit vier D-04-Grenzfall-Tests
   (Slot vor Cutoff / endet exakt am Cutoff / überlappt / komplett
   hinter Cutoff). Keine Panics, kein `unwrap`.

2. **Toggle-Stichtag + `shortday_gate`-Helper** (SHC-06 BE) — additive
   Migration `20260704000001_seed-shortday-slot-clipping-toggle.sql`
   (`INSERT OR IGNORE`, `enabled=0`, `value=NULL` → Rollout-Default
   „Kürzung aus"). `shortday_gate::{parse_active_from, should_clip,
   resolve_active_from_for_week, clip_slot_for_week, ClipOutcome}` als
   pure Rust-Helper (kein async, kein DAO); Präzedenz HCFG-02 aus v1.7.

3. **Vier BE-Aggregat-Ketten clippen konsistent** — Chain A' BlockService
   (iCal + insufficient), Chain B `build_shiftplan_day` (WeekView + PDF
   via `ShiftplanSlot.effective_to`, ersetzt Filter-statt-Clip-Bug),
   Chain C `booking_information` (ersetzt denselben Bug), Chain D
   `ShiftplanReport` als Rust-Layer-Refactor (raw-row DAO +
   Aggregation im Service, entfernt SUM-Queries mit pre-existing
   `/60.0`-Bug). Alle vier Ketten gaten am `shortday_gate::should_clip`.

4. **DTO-Wrapper-Field `ShiftplanSlotTO.effective_to`** (D-51-09) —
   trägt den geclippten Wert ans FE + PDF. `SlotTO` bleibt bidirektional
   roh (POST/PUT `/slot`-Roundtrip byte-clean, per compile-time-Test
   gepinnt). FE-Loader kopiert `effective_to` in `state::Slot.to` beim
   Shiftplan-Load; WeekView + PDF-Renderer sehen automatisch geclippte
   Werte. Fat Backend, Thin Client (D-51-02): grep-verifiziert kein
   `clip_to`-Call im `shifty-dioxus/src/`.

5. **Admin-Settings-UI Card 2b** (SHC-06 FE) — admin-gated Datepicker in
   Settings-Page, strukturell identisch zum HCFG-02-Blueprint (Save +
   Clear + Feedback + UnsetHint), 6 neue i18n-Keys de/en/cs. Pure
   `is_within_shortday_gate`-Validator als `#[cfg(test)]`-
   Kontraktspiegel.

6. **Drei pre-existing Bugs mitgeräumt** — (a) Filter-statt-Clip in
   `shiftplan.rs` + `booking_information.rs` (ShortDay-Slot wurde ganz
   ausgefiltert statt am Cutoff gekürzt); (b) `/60.0`-SQL-Bug in alten
   Chain-D-SUM-Queries (via Delete-Branch beim Rust-Layer-Refactor);
   (c) `ToggleService`-Full-Context-Bypass für internal-Aggregate-
   Konsumenten (nachträglich als Gap-Closure gefixt in Commits
   `f654613`, `7f21bd4`, `1b863e8`, `5aee47e`, `9cbe151`).

**Test verification:** Phase-51-Verifier PASS (6/6 must-haves,
`behavior_unverified: 0`); Milestone-Audit `passed` (6/6 Requirements,
6/6 Cross-Phase Wirings, 6/6 E2E-Flows, 2 non-blocking Warnings W1+W2).
Backend `cargo test --workspace` + `cargo clippy --workspace -- -D
warnings` grün; FE `cargo build --target wasm32-unknown-unknown` +
FE-Clippy `-D warnings` grün. Snapshot-Schema-Version bleibt 12
(grep-verifiziert).

**Known deferred items:**

- W1 (cosmetic): P07-SUMMARY-Doc-Drift (nennt pdf_render-Fns die nicht
  existieren; Runtime korrekt via loader-Collapse). Kein Blocker.
- W2 (latent): `shifty-dioxus/src/state/shiftplan.rs:199-214`
  `From<&SlotTO> for Slot` ignoriert `effective_to`; heute nur im
  Slot-Edit-Form-Pfad (raw korrekt). Empfehlung: Doc-Warnkommentar
  oder Rename zu `Slot::from_edit_to(SlotTO)`.
- Pre-existing 4 `dbg!`-Makros in `service_impl/src/block.rs:71-91`
  (kein Clippy-Verstoß, in VERIFICATION.md dokumentiert).
- Historische Deferred-Items siehe `.planning/STATE.md` „Deferred
  Items"-Sektion.

**Hinweis:** v2.4 ist das interne Planungs-Label. Reale Release-Version
via `/release-version` (`git.create_tag=false` in GSD-Config,
Tags aus SemVer-Flow).

---

## v2.5 — Weekly-Overview Performance & Freiwilligen-Abwesenheiten

**Shipped:** 2026-07-06
**Phases:** 52–53 (2 phases, 8 plans + 3 Follow-Ups, 11 SUMMARYs)
**Archive:** [`milestones/v2.5-ROADMAP.md`](milestones/v2.5-ROADMAP.md)
**Audit:** [`milestones/v2.5-MILESTONE-AUDIT.md`](milestones/v2.5-MILESTONE-AUDIT.md) — status `passed` (9/9 Requirements, 1 SC-Override formal aufgelöst)

**Delivered:**
Zwei zusammenhängende Ergänzungen an der Jahresübersicht. **Performance:**
`get_weekly_summary` konsumiert Jahres-Aggregate statt sequenzieller
Wochen-Service-Calls; drei Chain-A/B/C-Preloads (`special_day`, `toggle`,
`absence_period`) auf Year-Scope gehoben; 26 000 SQLite-Roundtrips pro
Anfrage eliminiert; End-to-End-Median **2.33s → 0.12s (19.4×)**, WOP-04
<500ms um Faktor 4 übertroffen; Byte-Identität durch 8 Golden-Snapshot-
Fixtures über Feiertage/ShortDays/Volunteer-Absencen/CVC-06-Cap/`shortday_gate.active_from`-on/off gewährleistet; Jahresübergangs-Bug
(paid_hours-Drift KW1/KW53 durch Kalender-Jahr vs. ISO-Woche) in
Follow-up #3 mit drei `_iso_year`-Bulk-Methoden geschlossen (16 neue
Regressions-Gates). **VAA:** Freiwillige mit aktiver Vacation/SickLeave/
UnpaidLeave-Period erscheinen in `sales_person_absences` neben bezahlten
Mitarbeitern; Backend liefert Name + cap-gated `committed_voluntary` im
DTO; FE macht reinen Union-Merge + case-insensitive Sort; Rendering-Zeile
wörtlich unverändert; INT-Browser-Sightcheck durch User bestätigt. Kein
Snapshot-Bump (bleibt 12), keine Migration, keine neuen Cargo-Deps.

**Key accomplishments:**

1. **Wave 1 Golden-Snapshot-Baseline** (Plan 52-01) — 8 byte-identische
   Fixture-Tests + Pre-Refactor-Latenz-Baseline 2.33s als hartes Gate für
   alle folgenden Waves. Verhinderte stillen Semantik-Drift bei drei
   Chain-Optimierungen.

2. **Wave 2+3+4 additive Batch-Trait-Methoden** (Plans 52-02, 52-03,
   52-04) — `assemble_weeks(weeks, ...)` als `pub(crate) async fn` in
   `reporting.rs` (Wave 2, reiner Extract); `ExtraHoursService::find_by_year`
   + `ShiftplanReportService::extract_shiftplan_report_for_year` mit
   `sqlx prepare` (Wave 3); `ReportingService::get_year` mit drei
   Bulk-Load-Roundtrips + Vec-Delegation (Wave 4). `get_week`-Signatur
   unverändert — reiner Wrapper.

3. **Wave 5 Bulk-Load-Präambel im Konsumenten** (Plan 52-05) — 7
   konstante Bulk-Loads vor der Wochen-Schleife in
   `BookingInformationServiceImpl::get_weekly_summary`; ersetzt die ~55×3
   Per-Woche-Chains durch In-Memory-Filter. 8/8 Wave-1-Fixtures grün,
   Latenz 2.33s → 1.13s (Faktor 2.07×).

4. **Follow-Ups #1+#2 Chain-A/B/C-Elimination** (52-followup-wop04 +
   52-followup2-wop04) — `sales_person` load-once via HashMap + `working_hours`
   per-sp-gebucketet (F#1: 2.07× → 2.40×), dann Chain A (`special_day`),
   Chain B (`toggle`), Chain C (`absence_period.derive_hours_for_range`)
   in `assemble_weeks` auf Year-Scope-Preloads gehoben plus zwei pure
   In-Memory-Helper (`derive_hours_for_week_pure`,
   `build_derived_holiday_map_for_week_pure`). **19.4× kumulativ,
   WOP-04-Ziel um 4× übertroffen (0.12s Median).**

5. **Follow-up #3 Jahresübergangs-Fix** (52-followup3-year-boundary-fix) —
   User-Report reproduziert: paid_hours/required_hours-Drift in KW 1 / KW 53
   durch drei kalender-jahr-scharfe Bulk-Methoden mit falscher Range vs.
   `booking(year, calendar_week)`-ISO-Semantik. Fix: neue `_iso_year`-
   Varianten mit `[ISO-Mo(Y,1), ISO-Su(Y,weeks(Y))+1d]`; alte kalender-jahr-
   Methoden bei ExtraHours + ShiftplanReport gelöscht (grep-verifiziert);
   16 Regressions-Gates in `reporting_year_boundary.rs` +
   `booking_information_weekly_summary_year_boundary*.rs`. Löste den
   Phase-52-SC#4-Override formal auf.

6. **VAA Datenkontrakt + zwei Fill-Sites** (Plans 53-01 + 53-02) —
   `SalesPersonAbsence` (service) + `SalesPersonAbsenceTO` (rest-types)
   Twin-Struct mit `#[serde(default)]`-Guard für Legacy-Wire-Compat;
   `WeeklySummary{,TO}.sales_person_absences: Arc<[...]>` additiv;
   D-53-02 cap-gated `committed_voluntary`-Formel an beiden Fill-Sites
   (`get_weekly_summary` + `get_summery_for_week`) identisch gepinnt.
   Sichtbarkeitskriterium = exakt `absent_volunteer_ids` aus VFA-01
   whole-week-out (Phase 26). 3/3 `vaa03_*`-Backend-Tests grün.

7. **VAA FE Union-Merge** (Plan 53-03) — `impl From<&WeeklySummaryTO> for
   state::WeeklySummary` als Block-Expression: Bezahlten-Loop
   unverändert (Regression-Lock VAA-03 #3), Freiwilligen-`extend` aus
   DTO-Feld, case-insensitive Name-Sort. Rendering-Zeile in
   `page/weekly_overview.rs:126` wörtlich unverändert (grep-verifiziert).
   INT-Browser-Sightcheck durch User bestätigt.

**Test verification:** Backend `cargo test --workspace` = 713+ Unit +
64+ Integration + kleinere Suites, 0 failed. Backend `cargo clippy
--workspace -- -D warnings` = 0 warnings. FE `cargo test` = 802 passed,
0 failed. FE WASM-Build = exit 0. Byte-Identity-Fixtures = 8/8 grün.
VAA-Tests = 3/3 grün. Regression-Locks (VFA-01, Chain-C, Legacy-Wire-Compat) alle grün.

**Known deferred items (Tech-Debt für v2.6+ Backlog):**

- SDF-03 Semantik-Cleanup: `SpecialDayService::get_by_iso_year` als
  Follow-up, um die 55 `special_day.get_by_week`-Calls in `assemble_weeks`
  auf konstant 2 zu reduzieren. Nicht latenz-relevant.
- DB-Indices: `booking(year, calendar_week)`, `extra_hours(date_time)`,
  `working_hours(from_year, to_year)` — RESEARCH-Q3 v2.5. Kein Bottleneck
  mehr nach Follow-up #2.
- F07-Doku: Follow-Ups #1+#2 haben die neuen Pure-Helper
  (`derive_hours_for_week_pure`, `build_derived_holiday_map_for_week_pure`)
  NICHT in `docs/features/F07-reporting-balance.{md,_de.md}` dokumentiert;
  Balance-Formel selbst ist unverändert dokumentiert.
- Historische Deferred-Items siehe `.planning/STATE.md` „Deferred
  Items"-Sektion.

**Hinweis:** v2.5 ist das interne Planungs-Label. Reale Release-Version
via `/release-version` (`git.create_tag=false` in GSD-Config, Tags aus
SemVer-Flow).

---
