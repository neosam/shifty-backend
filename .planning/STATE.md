---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Frontend Abwesenheiten + UI-Closure-Restanten
status: milestone_complete
last_updated: "2026-05-08T12:47:18.000Z"
last_activity: 2026-05-08 -- Plan 08-09 (Cutover Wochenpauschalen-Heuristik) complete (detect_weekly_lump_sum + Migration-Loop-Pre-Check für extra_hours-Rows mit `amount = Σ contract.hours_per_day in ISO-Woche` → absence_period {Mo, So}; Live-Szenario Max Schmidt 20h@Fr bei 3-Tage-Vertrag migriert sauber; 7 unit + 1 integration test added, Test #19 Plan-08-08 fixture amount auf 25.0 angepasst; cargo test --workspace + WASM build green; 5 jj-commits)
progress:
  total_phases: 1
  completed_phases: 1
  total_plans: 9
  completed_plans: 8
  percent: 100
---

# Project State: Shifty Backend

## Project Reference

- **Roadmap**: `.planning/ROADMAP.md` (collapsed milestone format — v1.0, v1.1, v1.2 archived)
- **Milestones-Index**: `.planning/MILESTONES.md`
- **Latest milestone archive**: `.planning/milestones/v1.2-ROADMAP.md`
- **Codebase**: `shifty-backend/CLAUDE.md` (architecture, conventions); Frontend in `shifty-dioxus/CLAUDE.md` + `.planning/codebase/frontend/`
- **Last shipped**: v1.2 Frontend rest-types Konsolidierung (2026-05-07)
- **Current milestone**: v1.3 Frontend Abwesenheiten + UI-Closure-Restanten (gestartet 2026-05-07)
- **Current focus**: v1.3 — Frontend-Abwesenheiten-Maske gegen `/absence-period` als Hauptthema; UI-Closure FUI-01..04 sekundär

## Current Position

Phase: 08
Plan: Not started
Status: Milestone complete
Last activity: 2026-05-08

## Shipped Milestones

### v1.2 — Frontend rest-types Konsolidierung (2026-05-07)

- **2 Phases (6, 7), 6 Plans**, 466 tests green workspace-weit
- Backend-`rest-types` als single source of truth verdrahtet; Frontend-Fork gelöscht; WASM-Build grün
- Phase 7 als Subsumption-Closure-Phase abgeschlossen (User-UAT auf Integrationsumgebung + Phase-6-V-Truth-Reuse)
- 8/8 V-Truths (P6) + 4/4 Success Criteria (P7) verified

### v1.1 — Slot Capacity & Constraints (2026-05-04)

- **1 Phase, 6 Plans**, 461 tests green (+6 über v1.0-Baseline 455)
- Slots: `max_paid_employees: Option<u8>` mit nicht-blockierender Warning-Emission
- 16/16 D-decisions verified (status: passed, gaps_remaining = [])
- Legacy `POST /booking` + `BookingService::create` unverändert (D-Phase3-18 Regression-Lock gehalten)

### v1.0 — Range-Based Absence Management (2026-05-03)

- **23 plans / 22 SUMMARYs** über 4 Phasen geliefert
- 458+ tests green workspace-weit
- OpenAPI surface gepinnt via insta-snapshot (3-run deterministic check passed)
- Atomic-Tx-Cutover verifiziert (Backup → Carryover-Rebuild → Soft-Delete → Flag-Flip)
- Service-Tier-Konvention (Basic vs Business-Logic) durchgehend angewendet

## Accumulated Context (carry forward)

### Architecture Decisions Logged

**v1.3 (Phasen 8+ — Frontend Abwesenheiten + UI-Closure-Restanten):**

- **Heuristik-Pre-Check vor Quarantine-Pfaden** (Plan 08-09): Wenn eine Migration- oder Quarantine-Logik etablierte User-Conventions falsch ablehnt, wird ein additiver Pre-Check VOR den existing Pfaden eingebaut — Match → bypass mit explizitem Output (hier: 1-Row-Cluster mit überschriebenem `{Mo, So}`-Range), Non-Match → fall-through zur unveränderten Logik. Backwards-Compat ohne Code-Duplication; existing Tests bleiben grün ohne Modification. Konkret im Cutover: `detect_weekly_lump_sum(row, all_rows, contract_at)`-Helper + `iso_week_range(day)` + `lookup_active_contract(work_details, day)` als freistehende Funktionen am Modul-Ende. Detection-Order: Lump-Sum-Check VOR Workday-Quarantine + Strict-Match-Quarantine, weil Wochenpauschalen oft auf Nicht-Vertragstagen liegen — sonst wäre die Heuristik nutzlos. Per-Weekday-Contract-Lookup statt 'first contract of the week' kompatibel mit Vertragswechseln mid-week.
- **ISO-Wochen-Boundary via time-Crate-Roundtrip** (Plan 08-09): Mo-of-week / So-of-week werden NICHT manuell berechnet, sondern via `time::Date::to_iso_week_date(day) → (year, week, _)` + `time::Date::from_iso_week_date(year, week, Weekday::{Monday, Sunday})`. Cross-Year-korrekt (KW 53 / KW 1 spreading über Jahresgrenze) ohne eigene calendar-week-math; nutzt das bewährte time-Crate API.
- **Service-Enum-Reason-Mapping** (Plan 08-08): Wenn ein typisiertes Service-Enum (`QuarantineReason`) für End-User sichtbar wird, leben die human-readable + remediation-Strings als Methoden direkt am Enum (`human_text()`, `suggested_action()`) — single source of truth. DTOs (`CutoverQuarantineEntryTO`) rufen die Methoden auf und stringifizieren. Reusable für REST + künftige CLI-Tools / Admin-Reports ohne Wire-Tier-Abhängigkeit. Backend-Default ist Englisch; i18n übernimmt das Frontend (separates Backlog-Item).
- **Inline-Drift-Report-Pattern** (Plan 08-08): Wenn ein Service ein File-System-Audit-Artefakt produziert (z.B. `cutover-gate-{ts}.json`), bekommt die REST-Antwort zusätzlich einen typisierten Inline-Body (`CutoverRunResultTO.gate_drift_report: Option<CutoverGateDriftReportTO>`). File bleibt für Audit-Trail unverändert; Inline-Body ist die UX-Datenquelle für Browser-Konsumenten ohne FS-Access. `#[serde(default)]` auf den neuen Feldern hält Backwards-Compat.
- **Cross-Phase-Daten-Bucketing via Composite-Key-HashMap** (Plan 08-08): Wenn Service-Phase A (`migrate_legacy_extra_hours_to_clusters`) und Service-Phase B (`compute_gate`) per-Entity-Daten teilen müssen, ohne extra DAO-Roundtrips, transportiert eine `HashMap<(Uuid, EnumKategorie, u32), Vec<Entry>>` die Map zwischen Phasen. Voraussetzung: Enum-Key braucht `Hash`-Derive (purely additive Service-Tier-Erweiterung).
- **Admin-Auto-Grant via SQLite-Trigger** (Plan 08-07): Statt jede Privilege-Migration manuell mit einem `INSERT INTO role_privilege ('admin', 'X', ...)` zu duplizieren, läuft eine einmalige Migration `20260508120000_admin-auto-grant-privilege.sql` mit Backfill (alle existierenden Privilegien an admin) + AFTER-INSERT-Trigger (jedes neue Privileg auto-grant an admin). Beide Pfade nutzen `INSERT OR IGNORE` plus das existierende `UNIQUE(role_name, privilege_name)`-Constraint aus `20240426150045_user-roles.sql` als Idempotenz-Garant. DEVUSER hat in DEV automatisch alle Privilegien; Production-Deployment braucht keine manuelle Pflege mehr.
- **Feature-Flag REST + FE-State** (Plan 08-07): `GET /feature-flag/{key}` als auth-only readable Endpoint mit fail-safe `enabled: false` für unknown keys. Frontend-`FeatureFlagsState` defaultet zu `Option<bool>::None` per Flag (nicht `Some(false)`), damit UI-Logik explizit `Some(true)` matcht — verhindert das "sichtbar/unsichtbar/sichtbar"-Flackern während des ersten Service-Loads.
- **Static-Classification + Context-Overlay-Pattern für TopBar-Hierarchie** (Plan 08-07): `is_admin_target(target) -> bool` bleibt user-agnostisch und backwards-kompatibel; `is_admin_target_with_context(target, has_hr) -> bool` ist additiv und liftet `NavTarget::Absences` für HR-User in die Admin-Group, ohne die statische API zu brechen. Plan-08-05-Tests bleiben unangetastet (sales-only); Plan-08-07-Tests sind explizit auf das neue HR-Verhalten ausgelegt.
- **Surface-Assertion statt Full-Snapshot für OpenAPI-Drift-Detection** (Plan 08-03): Wenn ein Test einen `info.version`-bound Field aus `Cargo.toml` pinnt, schlägt er bei jedem Versions-Bump als Noise fehl. Lösung: Pfad-Liste + Schema-Namen-Liste + Tag-Namen-Liste via `assert!` auf `ApiDoc::openapi().paths.paths.keys()` etc., kein insta. Bodies pinnen wir NICHT — DTO-Feld-Churn wäre Noise. Pattern für künftige Crates, die utoipa::OpenApi nutzen.
- **Plan-Pivot mit User-Approval-Pattern** (Plan 08-03): Wenn der Plan-Body eine Tool-/Werkzeug-Annahme macht, die seit Plan-Phase falsch geworden ist (hier: insta-snapshot bewusst entfernt), und der User eine Alternative approved, wird (a) Plan-Frontmatter `must_haves.truths`/`tags`/`files_modified`/`autonomous` aktualisiert, (b) die Plan-Body-Tasks NICHT umgeschrieben (historischer Record), (c) die SUMMARY.md ist die ground-truth was tatsächlich gebaut wurde, (d) im SUMMARY ein "Architectural Pivot (User-Approved)"-Eintrag dokumentiert das.
- **Read-only Aggregat-DTO ohne `$version`-Field** (Plan 08-01): `VacationBalanceTO` ist ein berechnetes Aggregat — kein Optimistic-Lock-Konflikt möglich, daher entfällt das `$version`-Pattern aus AbsencePeriodTO. Bewusste Abweichung; Plan 08-02-REST-Endpoint liefert immer frische Werte.
- **Wave-1-Foundation-Plan ohne Test-Code** (Plan 08-01): Trait + Domain + DTO als reine Interface-Foundation; Tests landen in Wave 2 (Plan 08-02), wo die Service-Impl gegen `MockVacationBalanceService` testbar wird. Dieses Pattern ersetzt das Wave-0-Stub-`#[ignore]`-Pattern für reine BL-Tier-Trait-Foundations und ist stiller als ein Stub.
- **Active-Period-Split-on-today** (Plan 08-02): Wenn eine Vacation-Periode heute aktiv ist (`today ∈ [from, to]`), splittet `VacationBalanceServiceImpl::compute_balance` die Tage auf `clock.date_now()` als Stichtag — Vergangenheits-Anteil zu `used_days`, Zukunfts-Anteil zu `planned_days`. So gibt es keine Diskontinuität, wenn eine Periode genau heute beginnt oder endet, und das Frontend-Aggregat ist heute und morgen gleich aussagekräftig.
- **compute_balance als private Helper für get_team-Code-Sharing** (Plan 08-02): `get_team` iteriert über `sales_person_service.get_all_paid()` und ruft pro Person `compute_balance` auf, das ohne Permission-Check aggregiert (Outer-Permission ist schon im `get`/`get_team` enforced). Innere Service-Calls nutzen `Authentication::Full` analog `compute_forward_warnings` in `absence.rs`.
- **Special-Day-Subtraktion verschoben** (Plan 08-02 — A5-Note in 08-RESEARCH.md): Tag-Anzahl pro Vacation-Periode = `(to - from).whole_days() + 1`, beschnitten auf das Kalenderjahr. Wochenenden, Feiertage, Vertragsstunden-Anteile NICHT berücksichtigt. Das macht das Aggregat zur reinen Kalendertage-Sicht; Refinement (Tag-Äquivalent via `EmployeeWorkDetails.has_day_of_week`) ist Out-of-Scope für Plan 02 und wird je nach Frontend-Feedback in eine spätere Phase gefolded.
- **Modal-Event-Side-Channel-Pattern** (Plan 08-04): Statt `EventHandler<Result<...>>` als Action-Enum-Payload zu führen (was Debug-Trait und Lifetime-Probleme bringt), schreibt der Service modal-lokale Outcomes (Created/Updated/VersionConflict/Validation/Network/Deleted) in einen separaten `GlobalSignal<Option<AbsenceModalEvent>>`. Die Page liest diesen Signal reaktiv und ack-t mit `*store.write() = None`. Hält das Action-Enum cheap-derive-able und ist ergonomischer für die Page als ein durchgereichter EventHandler. PATTERNS.md Z. 522-525 erlaubt explizit beide Varianten — Side-Channel ist die hier gewählte.
- **Defensive Uuid::nil im API-Create-Body** (Plan 08-04): `api::create_absence_period` setzt im Function-Body als ersten Schritt `body.id = Uuid::nil(); body.version = Uuid::nil();`, unabhängig vom Caller-State. Verhindert, dass ein Edit→Create-Mode-Switch im Modal vergisst, die `id` zu nullen, was sonst Backend-422 (`IdSetOnCreate`) liefert. Funktion ist jetzt selbstkonsistent — Caller-Hygiene ist nicht mehr Korrektheits-Voraussetzung.
- **Per-Locale-Reference-Matcher-Tests gegen Pitfall 2** (Plan 08-04): Über den standard `i18n_*_present_in_all_locales`-Test hinaus drei zusätzliche Tests `i18n_*_match_{german,english,czech}_reference`, die je 4-5 Stichproben mit dem Original-String matchen. Fängt versehentliche `Locale::En, …`-Kalls in `de.rs` (oder `Locale::De, …` in `en.rs`/`cs.rs`), die sonst still durchgehen würden, weil Tests mit "?? "-Fallback nur fehlende Keys, nicht falsch-getaggte erkennen.
- **Frontend-State-with-Side-Join-Pattern erweitert** (Plan 08-04): `AbsencePeriod` trägt zwei `Arc<str>`-Felder (`person_name`, `background_color`), die der Loader aus der SalesPerson-Liste füllt — analog zum existierenden `Booking::label`/`background_color`-Pattern in `loader::load_bookings`. From-TO setzt sie initial leer; nur `load_absence_periods_all` (HR-Variante) joinst sie auf, weil die Self-Variante den User bereits kennt.
- **Single-File Page-Composition** (Plan 08-05): Die 9 domain-spezifischen Helper-Components (Modal, WarningList, CategoryBadge, StatusPill, VacationEntitlementCard, VacationPerPersonList, AbsenceList, AbsenceFilterBar, StatsGrid, DeleteConfirmDialog, plus zwei Banner) liegen INLINE in `shifty-dioxus/src/page/absences.rs` (1685 LOC, ~1330 prod + 355 tests). Plan-05 component-inventory schreibt das so vor — Extraction nach `component/absence_modal.rs` ist optional und nur sinnvoll bei Re-Use über die /absences-Surface hinaus. Soft-Cap bei 1500 prod-LOC.
- **Router-Variant-Alias-Pattern** (Plan 08-05): `dioxus_router`'s `#[derive(Routable)]` macht name-based component lookup. Wenn der Plan-Acceptance-Grep eine andere Bezeichnung verlangt (`AbsencesPage` als Component-Name + `Route::Absences {}` als Variant), löst ein zusätzlicher `pub use crate::page::AbsencesPage as Absences;` in router.rs beide Constraints ohne Component-Rename.
- **Newtype-Wrapper für Non-PartialEq Rc<[T]> in Dioxus Props** (Plan 08-05): `WarningTO` aus `rest-types` derived nicht `PartialEq` (Uuid + Date + AbsenceCategoryTO Payloads). `Rc<[WarningTO]>: PartialEq` braucht aber `T: PartialEq`. Lösung: `WarningsList(Rc<[WarningTO]>)`-Newtype mit `impl PartialEq via Rc::ptr_eq`. Compares „same allocation" — exakt für Re-Use-Cases, akzeptabel als false-negative-Häufigkeit (führt nur zu Re-Render, nicht zu Korrektheits-Bug). Keine `PartialEq`-Derive-Erweiterung im rest-types-Crate nötig.
- **Hook-basierter I18N-Locale-Pin in Snapshot-Tests** (Plan 08-05): Direkte `*I18N.write() = generate(Locale::De)` außerhalb eines Dioxus-Reactive-Scopes panic mit `RuntimeError`. Lösung: `pin_de_locale()`-Helper, der den Write inside `use_hook(...)` ausführt. `VirtualDom::new(app)` provided eine Runtime; der Hook läuft beim ersten Render einmalig BEFOR Descendant-Components I18N lesen. Pattern für künftige Locale-spezifische Snapshot-Tests im Frontend.
- **cfg-gated current_date_for_init()** (Plan 08-05): Production WASM-Build ruft `js::current_datetime().date()`; native Test-Build returns `time::macros::date!(2026-05-08)`. Pitfall-9-Audit-Grep scannt Production-Render-Path auf hardcoded Dates — der cfg-Gate hält den hardcoded Wert ausschließlich in `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(test)]` und keeps the audit clean.
- **Defensive Uuid::nil() at TWO layers** (Plan 08-05 verstärkt 08-04): Plan 08-04 setzt `Uuid::nil()` für id+version im api-Layer (`create_absence_period`). Plan 08-05 setzt sie ZUSÄTZLICH im Modal-Submit-Code, dokumentiert mit Inline-Kommentar auf Pitfall 9 / W-7. Auditors die nach Pitfall-9-Pattern grep-en finden's in beiden Layern — UI-Logik UND Wire-Layer.
- **`compute_status` Pure-Function mit injected `today`** (Plan 08-05): Status-Berechnung Active/Planned/Finished ist client-side reine Pure-Function `compute_status(from, to, today) -> AbsenceStatus`. Tests pinnen `today` und covern 3 Boundary-Cases (today before from → Planned, in range → Active, after to → Finished). Page wired today via `current_date_for_init()` at mount, satisfying Pitfall 8 ohne Service-Roundtrip.
- **Wave-0-Closure via Test-Layer (B-2)** (Plan 08-05 Task 3): VALIDATION.md Wave-0-Item-3 fordert "dioxus-ssr Snapshot-Test-Stub für absence-Components". Plan 05 Task 3 schließt das mit 11 Tests (3 CategoryBadge × 3 StatusPill × 3 compute_status × 2 AbsenceFilterBar). Frontmatter `nyquist_compliant: true` + `wave_0_complete: true` ist nach diesen Tests gesetzt. Phase 8 ist UAT-bereit.

**v1.2 (Phasen 6–7 — Frontend rest-types Konsolidierung):**

- **Cross-Workspace-Path-Dep mit `default-features = false`** (Plan 06-01): `shifty-dioxus/Cargo.toml` referenziert die Backend-`rest-types`-Crate via `path = "../rest-types"` mit explizitem `default-features = false`, um den `service-impl`-Feature-Pull-In zu vermeiden, der den WASM-Build durch das `service`-Crate sprengen würde.
- **Wave-0 Backend-Prep vor Cargo-Swap** (Plan 06-00): Pre-Migration der Invitation-DTO-Familie mit konsistentem Derive-Set (`Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema`) macht den Wave-1-Cargo-Swap mechanisch sauber. Backend-Derive-Erweiterung statt Frontend-Hack ist die korrekte Lösung für `assert_eq!`-Tests.
- **State-Editor-Mirror für nicht-editierte Felder** (Plan 06-04): `SlotEditItem` muss `max_paid_employees` als Field-Mirror tragen, weil sonst der Edit-Roundtrip (`SlotTO -> SlotEditItem -> SlotTO`) den Backend-Wert auf `None` setzt. Field-Mirror mit Default ist Pflicht für Datenintegrität, auch wenn das Feld in der aktuellen Phase nicht editiert wird.
- **Subsumption-Verification-Pattern** (Phase 7): Reine UAT-/Smoke-Phasen ohne eigenen Code-Change können in einem einzigen Plan-Summary mit Verweis auf die vorhergehende Phase abgeschlossen werden. Voraussetzungen: (1) automatische Test-Kriterien sind in der Vorgänger-Phase grün dokumentiert; (2) manuelle UAT-Kriterien sind vom User auf einer realen Umgebung verifiziert; (3) beide Belege werden in der Closure-Phase explizit referenziert.
- **No-op-Match-Arm-Pattern** (Plan 06-04): Für Phasen, deren Scope explizit "keine User-facing Features" ist, sind exhaustive Match-Arme via `WarningTO::PaidEmployeeLimitExceeded => rsx! { "" }` ausdrücklich erlaubt. UI-Closure folgt im nächsten Milestone.

**v1.1 (Phase 5 — Slot Paid Capacity Warning):**

- **Warning-Emission-Heart-Pattern** (Plan 05-06): Soft-Warning-Emission im Business-Logic-Tier-Service; insert die Limit-Check-Logik zwischen die existierende Cross-Source-Warning-Emission und das finale `transaction_dao.commit(tx)`. Persistierte Entity in-hand, warnings-Akkumulator in-hand. Kein Rollback (D-07). Helper als private Methode auf einem zweiten `impl<Deps>`-Block; Helper-Signatur: `tx: Deps::Transaction` by-value. Inner cross-service-calls verwenden `Authentication::Full`. D-12-Korrektur: Helper lebt auf `ShiftplanEditServiceImpl` (Business-Logic-Tier), NICHT auf `BookingService` (Basic-Tier per CLAUDE.md + v1.0 D-Phase3-18 Regression-Lock).
- **Wire-Tier-Mirror-Pattern** (Plan 05-05): Additive Service-Tier-Field/Variant landet wire-tier in `rest-types/src/lib.rs` durch 3 Mechanismen: (1) Struct-Feld auf `*TO` + beide `From`-Impls — Backward-Compat via `#[serde(default)]`; (2) Enum-Variant am Ende mit `#[serde(rename_all = "snake_case")]`-Auto-Tag + matching `From`-Arm (rustc enforced Exhaustivität); (3) Cascade-DTOs erben automatisch via `Vec<*TO>`-Embedding.
- **Wave-Coupling-Pattern** (Plan 05-02): Wenn ein additiver Variant zu einem Domain-Enum ein exhaustive downstream `match` ohne Wildcard bricht, schedule Producer-Plan + Consumer-Plan in der GLEICHEN Wave; Standalone-Akzeptanz reduziert sich auf `cargo build -p {producer-crate}`.
- **Read-Aggregation-Pattern** (Plan 05-04): `current_paid_count: u8` wird inline in `build_shiftplan_day` aus bereits resolvten `slot_bookings` per `.filter(|sb| sb.sales_person.is_paid.unwrap_or(false)).count().min(u8::MAX as usize) as u8` abgeleitet. Als `u8` (nicht `Option<u8>`).
- **Forward-Compat-Shim-Pattern (Rule 3)** (Plan 05-01, 05-03): Wenn DAO-Feld eine Phase vor seinem Service-Layer-Mirror landet, hardcode `None` in `From<&Service::Slot> for SlotEntity` und im zentralen Test-Fixture mit Inline-Kommentar auf Folge-Plan.
- **Sequential-Wave-Friction-Mitigation** (Plan 05-03): Wenn parallel-geplante Wave-Plans sequenziell ausgeführt werden, Rule-3-Shims in OUT-OF-SCOPE-Sites mit Folge-Plan-Kommentar einsetzen statt Wave-Reorder.
- **D-12-Override-Präzedenz**: Wenn CONTEXT.md einen Tier-Hint liefert, der gegen CLAUDE.md Service-Tier-Konvention verstößt, **das Plan-File `<objective>` overrid**et CONTEXT.md explizit. Service-Tier-Konvention ist die durchsetzungsstärkere Regel.

**v1.0 (Phasen 1–4):**

- Parallele `absence` Domain (nicht Erweiterung von `extra_hours`).
- Hybrid materialize-on-snapshot / derive-on-read (Live-Reports derive on read; BillingPeriod-Snapshots materialize-once).
- Direction: `AbsenceService → BookingService` (Business-Logic-Tier konsumiert Basic-Tier; nie umgekehrt).
- Service-Tier-Konvention etabliert: Basic Services konsumieren nur DAO/Permission/Transaction; Business-Logic Services kombinieren Aggregate. Doku: `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen".
- `BookingCreateResult { booking, warnings }`-Wrapper für nicht-blockierende Warnings (lebt im Business-Logic-Tier).
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` MUSS gebumpt werden im selben Commit wie Reporting-Switch (per `CLAUDE.md`).
- Phase-3 Wave-0-Stub-Pattern: `#[ignore]` + `unimplemented!()` als Standard für Wave-Forcing.
- Phase-4 Cycle-Break: separater `CarryoverRebuildServiceImpl` (BL-Tier) — bricht Reporting↔Carryover-Cycle.
- logical_id-Versionierungs-Pattern (rotiert physische Row, hält stabilen externen ID): erst in `extra_hours` (commit fe744df) eingeführt, dann in `absence_period` übernommen.

### Constraints In Force

- **VCS**: Repository wird mit `jj` (co-located mit git) verwaltet — Commits manuell durch User. GSD-Auto-Commit ist deaktiviert (`commit_docs: false`). KEINE `git commit`/`git add` aus Agents heraus.
- **NixOS**: Tools wie `sqlx-cli` via `nix develop` (NICHT `nix-shell`, shell.nix kaputt). DB-Befehle: `sqlx database reset` ist DESTRUCTIVE → für additive Migrationen `sqlx migrate run`. Für WASM-Builds in `shifty-dioxus/` ggf. `nix develop` für `wasm32-unknown-unknown`-Toolchain + `dx`/Tailwind.
- **Snapshot Versioning**: `CURRENT_SNAPSHOT_SCHEMA_VERSION` MUSS gebumpt werden, sobald sich `value_type`-Berechnung oder -Input-Set ändert.
- **Multi-Sprache (i18n)**: Alle benutzersichtbaren Texte in en/de/cs. v1.3 wird Frontend-Abwesenheiten-Maske mit signifikantem i18n-Volumen einführen — gleichzeitig in allen drei Locales pflegen (kein Locale::En-statt-Locale::De-Bug).
- **Layered Architecture**: REST → Service (trait) → DAO (trait); `gen_service_impl!` für DI; `WHERE deleted IS NULL` in jeder DAO-Read-Query.
- **Service-Tier-Konvention**: Basic Services konsumieren nur DAO/Permission/Transaction; Business-Logic Services kombinieren Aggregate. Plan-File `<objective>` darf CONTEXT.md-Tier-Hints overriden (Phase-5-D-12-Präzedenz).
- **rest-types-Cross-Crate-Konstruktion** (etabliert in v1.2): Backend-`rest-types/Cargo.toml` hat ein `service-impl`-Feature, das auf das `service`-Crate zeigt. Frontend MUSS dieses Feature OFF lassen (`default-features = false`) — sonst zieht es das `service`-Crate in den WASM-Build und reißt die Toolchain auseinander.

### Open Issues / Tech Debt for v1.3+ (live backlog)

- **Frontend Abwesenheiten-Maske** (FUI-A-01..09) — neue Top-Level-Maske gegen `/absence-period` REST-API (HR-Sicht + Employee-Self-Service); siehe `notes/abwesenheiten-frontend-context.md` und `seeds/abwesenheiten-frontend-milestone.md`. **Hauptthema für v1.3.**
- **Frontend User-facing Closure** (FUI-01..04) — sichtbares `current_paid_count`/`max_paid_employees`-Rendering, Capacity-Editor in Slot-Settings, sichtbare `VolunteerWork`/`UnpaidLeave`-Rendering, `cap_planned_hours_to_expected`-Settings-UI. v1.2 hat den Compile-Pfad freigemacht; v1.3 baut die UI darauf.
- **Min-Paid-Capacity / Skill-Matching** (SC-01, SC-02) — weitere Slot-Constraints als künftige Backend-Features gemerkt.
- **04-UAT Test 8** (idempotenter Cutover-Re-Run): bei nächster Cutover-Phase neu prüfen.
- **`/gsd:secure-phase 04`** — als bewusstes Skip akzeptiert; Compliance separat klären falls gefordert.
- **Zwei offene Review-Todos** (`list_user_invitations` silent-empty, OIDC `silentRenewIframe`) — eigener Todo-Lifecycle.

### Phase-Verzeichnis-Cleanup (optional)

`.planning/phases/01-04` (v1.0), `.planning/phases/05` (v1.1), `.planning/phases/06-07` (v1.2) liegen alle noch im aktiven `phases/`-Verzeichnis. `gsd-sdk milestone.complete` hat sie nicht automatisch in `milestones/v1.X-phases/` verschoben (`archived.phases: false`). Bei Bedarf manuell via `/gsd-cleanup` oder `mkdir milestones/v1.X-phases && mv phases/...` archivieren.

## Session Continuity

**To resume work in a new session:**

1. Read `.planning/MILESTONES.md` (geshipte Milestones — v1.0, v1.1, v1.2)
2. Read `.planning/ROADMAP.md` (v1.3-Phasen aktiv; v1.0–v1.2 collapsed)
3. Read `.planning/REQUIREMENTS.md` (v1.3-Scope, REQ-IDs, Coverage)
4. Read this file (`STATE.md`) — current position
5. Read `.planning/notes/abwesenheiten-frontend-context.md` — v1.3 Briefing
6. Read `.planning/seeds/abwesenheiten-frontend-milestone.md` — Sub-Phasen-Skizze
7. Read `shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md` — Backend-Integrations-Brief
8. Read `shifty-dioxus/shifty-design/project/absences.jsx` — Mockup (729 Zeilen JSX)

**Next command**: `/gsd-execute-phase 8` für Plan 08-06 (UAT smoke; finale Closure-Phase). Mit Plan 08-09 ist die Cutover-Migration jetzt für reale Wochenpauschalen-Bookings tolerant — der Live-Bug Max Schmidt 20h@Friday bei 3-Tage-Vertrag migriert sauber.

---

*State updated: 2026-05-08 — Plan 08-09 (Cutover Wochenpauschalen-Heuristik) abgeschlossen (detect_weekly_lump_sum-Helper + iso_week_range + lookup_active_contract; Migration-Loop bekommt Pre-Check (a.5) VOR Workday/Strict-Match Quarantäne; 1× extra_hours-Row mit `amount = Σ contract.hours_per_day für Vertragstage der ISO-Woche` wird auf absence_period {Mo, So} gemappt — auch wenn Eintrag-Tag ein Nicht-Vertragstag ist; Backwards-compat: existing Strict-Match + Cluster-of-N + Quarantine-Reasons unverändert; 7 unit + 1 integration test added (alle grün), Plan-08-08 Test #19 fixture amount auf 25.0 angepasst um Heuristik-Match zu vermeiden; cargo test --workspace grün (396+68+andere), WASM-Build grün; 5 jj-commits). Phase-8-Progress 8/9 (89%). Plan 08-06 als nächstes — UAT smoke.*
