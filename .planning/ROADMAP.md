# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- 🚧 **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (in planning, started 2026-05-07)

## Phases

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
  Slots erhalten ein optionales `max_paid_employees: Option<u8>` Capacity-Limit. Backend emittiert nicht-blockierende `Warning::PaidEmployeeLimitExceeded` (a) im `BookingCreateResult.warnings` im Conflict-Aware-Booking-Flow und (b) als `current_paid_count` per Slot im Shiftplan-Week-View. Buchen bleibt erlaubt (D-07: kein Rollback). NULL = kein Limit. Limit-Check lebt im Business-Logic-Tier (`ShiftplanEditService`); Legacy `POST /booking` + `BookingService::create` UNVERÄNDERT (D-Phase3-18 Regression-Lock gehalten). 461 Tests green (+6); 16/16 D-decisions verified. Frontend (shifty-dioxus) out of scope.

**Full milestone archive:** [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)

</details>

### 🚧 v1.2 Frontend rest-types Konsolidierung (in flight)

- [ ] **Phase 6: rest-types Unification & Frontend Compile-Through** — Backend-`rest-types` als single source of truth verdrahten, Frontend-Fork löschen, alle 17 fehlenden TOs/Enum-Varianten + 4 fehlenden Felder + Match-Arme im Frontend-Code adressieren bis `cargo build --target wasm32-unknown-unknown` grün ist.
- [ ] **Phase 7: Runtime Smoke & Regression Safety** — `dx serve` startet ohne Runtime-Panics; Login + Shiftplan-Navigation manuell verifiziert; Backend-Workspace `cargo check` + `cargo test` ohne Regression nach Cargo-Feature-Umbau in `rest-types`.

## Phase Details

### Phase 6: rest-types Unification & Frontend Compile-Through

**Goal:** Backend-`rest-types` ist die einzige Quelle der Wahrheit für API-DTOs im Repository. Frontend-Fork ist gelöscht; `shifty-dioxus` kompiliert (WASM-Target) gegen den realen Backend-API-Stand.

**Depends on:** Nothing (additive consolidation; baut technisch auf v1.1 D-09/D-10 + v1.0 Phase-3-Warning-Surface auf, aber kein Code-Dependency)

**Requirements:** RT-01, RT-02, RT-03, FC-01, FC-02

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. `shifty-dioxus/Cargo.toml` deklariert `rest-types = { path = "../rest-types", default-features = false }` und KEINE `path = "rest-types"`-Selbstreferenz mehr (RT-01).
2. Verzeichnis `shifty-dioxus/rest-types/` existiert nicht mehr im Tree; eine Repo-weite Suche `find . -type d -name rest-types` liefert genau einen Treffer (`./rest-types`) (RT-02).
3. Frontend-Code (`shifty-dioxus/src/**/*.rs` inkl. `state/`, `loader.rs`, `api.rs`, `component/`, `page/`) kompiliert ohne `unresolved import`/`no variant`-Fehler gegen alle in CONCERNS.md §1 katalogisierten 17 fehlenden Structs/Enums (`WarningTO`, `AbsencePeriodTO`, `UnavailabilityMarkerTO`, Cutover-DTOs, `ToggleTO`/`ToggleGroupTO`, `ImpersonateTO`, `BookingCreateResultTO`, `CopyWeekResultTO`, `AbsencePeriodCreateResultTO`, `AbsenceCategoryTO`, `ShiftplanAssignmentTO`, `ExtraHoursCategoryDeprecatedErrorTO`) plus 4 fehlenden Felder (`SlotTO.max_paid_employees`, `ShiftplanSlotTO.current_paid_count`, `ShiftplanDayTO.unavailable`, `BillingPeriodTO.snapshot_schema_version`) (RT-03).
4. Alle `match`-Ausdrücke gegen Backend-Enums (insbesondere `WarningTO` mit 5 Varianten, `ExtraHoursCategoryTO` mit `UnpaidLeave`/`VolunteerWork`, `InvitationStatus`, `UnavailabilityMarkerTO` mit 3 Varianten) sind erschöpfend; keine `panic!`-on-unknown-variant-Pfade verbleiben in `state/employee.rs` und `state/shiftplan.rs` für die jetzt-bekannten Varianten (FC-01). Minimal/no-op-Rendering ist akzeptabel (z. B. `WarningTO::PaidEmployeeLimitExceeded => rsx! { "" }`); UI-Closure ist explizit v1.3-Scope.
5. `cargo build --target wasm32-unknown-unknown` im `shifty-dioxus/`-Subordner liefert Exit-Code 0 ohne Errors; Warnings sind toleriert (FC-02).

**Plans:** 5 plans

Plans:
- [x] 06-00-PLAN.md — Wave 0 Backend-rest-types Vorbereitung (InvitationStatus-Familie + ShiftplanTO Derives + shifty_utils Feature-Gate)
- [x] 06-01-PLAN.md — Wave 1 Cargo-Swap + Fork-Delete (RT-01 + RT-02)
- [x] 06-02-PLAN.md — Wave 2 Slot-Capacity State-Mirror (Cluster E + Weekday-Panic-Defense)
- [x] 06-03-PLAN.md — Wave 2 Invitation-Surface Cutover (Cluster F: redeemed_at-String-Migration)
- [x] 06-04-PLAN.md — Wave 3 WASM-Compile-Closure (residual fixes + FC-02 phase gate)

**UI hint**: yes

**Notes for plan-phase:** Die Drift-Inventur in CONCERNS.md §1 clustert sich entlang von Feature-Domänen (Absence-Stack aus v1.0 / Slot-Capacity aus v1.1 / Cutover-Surface / Feature-Flag-Toggles / User-Invitations / Shiftplan-Assignments). Plans innerhalb dieser Phase können in parallelen Waves laufen, wobei jede Wave ein Feature-Cluster fehlender TOs adressiert (Match-Arme + `loader.rs`-Mappings + ggf. `api.rs`-Wrapper-Shape-Anpassungen). Die mechanische "swap dep + delete fork"-Operation (RT-01 + RT-02) sollte als eigener Plan früh in der Phase laufen — sie öffnet die Compile-Error-Welle, die dann von den Cluster-Plans abgearbeitet wird. Wave-Topologie-Vorschlag: Wave 1 = Cargo-Swap + Fork-Delete (1 Plan, sequenziell); Wave 2 = Feature-Cluster-Plans parallel (3–5 Plans, partitioniert nach disjunkten Modul-Mengen in `src/state/`, `src/loader.rs`-Sektionen, `src/page/`); Wave 3 = Compile-Closure (residual Match-Arm-Cleanup, falls Wave 2 Restdrift hinterlässt). Plan-phase entscheidet die finale Wave-Struktur.

### Phase 7: Runtime Smoke & Regression Safety

**Goal:** Verifizieren, dass das vereinheitlichte `rest-types` zur Laufzeit funktioniert (Frontend-Boot + Login + Hauptseiten-Navigation) und dass die Cargo-Feature-Umbauten an `rest-types` keine Backend-Regression verursacht haben.

**Depends on:** Phase 6 (rest-types-Unification ist Voraussetzung; Phase 7 prüft den vereinheitlichten Stand)

**Requirements:** FC-03, RC-01

**Success Criteria** (was muss WAHR sein, nachdem die Phase abgeschlossen ist):

1. `dx serve --hot-reload` im `shifty-dioxus/`-Subordner startet das Frontend auf Port 8080 ohne WASM-Init-Panic; Browser-DevTools-Console zeigt keine `RuntimeError`/`unreachable executed`-Einträge beim ersten Paint (FC-03 Boot-Gate).
2. Manuelle UAT: Login-Flow gegen den lokalen Backend (`cargo run` auf Port 3000) erfolgreich; Browser-Session-Cookie wird gesetzt; Navigation zur Shiftplan-Seite (`/shiftplan/<id>/<year>/<week>`) rendert eine Week-View ohne Panic; Slot mit `max_paid_employees`-Limit oder mit Absence-Marker auf einem Tag rendert ohne Crash (FC-03 Navigation-Gate).
3. Backend-Workspace `cargo check --workspace` im Repo-Root liefert Exit-Code 0; vergleichbare Wall-Clock-Zeit zur v1.1-Baseline (32 s ± 30 %) — kein neuer Compile-Pfad durch `default-features = false`-Umbau (RC-01 Compile-Gate).
4. Backend-Workspace `cargo test --workspace` im Repo-Root: 461 Tests grün (= v1.1-Baseline) oder mehr; KEINE Tests in `service_impl/src/test/` oder `dao_impl_sqlite/src/test/` rot durch unerwartete Feature-Flag-Effekte am `rest-types`-Crate (RC-01 Test-Gate).

**Plans:** TBD

**UI hint**: yes

**Notes for plan-phase:** Phase 7 ist klein — wahrscheinlich 1–2 Plans. Ein Plan kann reichen, falls die manuelle UAT-Checkliste (FC-03) im selben Plan wie die `cargo check`/`cargo test`-Regressionsläufe (RC-01) abgearbeitet wird. Falls UAT separate Aufmerksamkeit braucht (z. B. wenn unerwartete Runtime-Issues auftauchen, die im Phase-7-Scope adressiert werden müssen statt zurück nach Phase 6 zu eskalieren), kann sie in einen eigenen Plan ausgegliedert werden. Falls in Phase-6-Plans `panic!`-Pfade in `state/employee.rs:89/151` oder `state/shiftplan.rs:59` (CONCERNS §9) zurückgelassen wurden, ist hier der letzte Catch-Point — entweder UAT manuell auslösen lassen oder defensiv eine Fall-Back-Variant einführen. Plan-phase entscheidet.

---

## Progress

| Phase | Milestone | Plans Complete | Status   | Completed  |
|-------|-----------|----------------|----------|------------|
| 1 — Absence Domain Foundation | v1.0 | 5/5 | Complete | 2026-05-01 |
| 2 — Reporting Integration & Snapshot Versioning | v1.0 | 4/4 | Complete | 2026-05-02 |
| 3 — Booking & Shift-Plan Konflikt-Integration | v1.0 | 6/6 | Complete | 2026-05-02 |
| 4 — Migration & Cutover | v1.0 | 8/8 | Complete | 2026-05-03 |
| 5 — Slot Paid Capacity Warning | v1.1 | 6/6 | Complete | 2026-05-04 |
| 6 — rest-types Unification & Frontend Compile-Through | v1.2 | 0/5 | Planned | — |
| 7 — Runtime Smoke & Regression Safety | v1.2 | 0/0 | Not started | — |

---

*Last updated: 2026-05-07 — v1.2 milestone in planning. Phase 6: 5 plans across 3 waves (Wave 0 backend prep, Wave 1 cargo-swap, Wave 2 cluster-fixes, Wave 3 wasm-closure); Phase 7 plan-decomposition pending.*
