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
