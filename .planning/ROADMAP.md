# Roadmap: Shifty Backend — Range-Based Absence Management

**Created:** 2026-05-01
**Granularity:** standard
**Coverage:** 19/19 v1 requirements mapped
**Milestone goal:** Replace per-date hour-amount absence accounting (Vacation/Sick/UnpaidLeave) with range-based absences whose per-day hour effects are derived from the contract valid on that day, eliminating double-entry between ExtraHours and the shift plan, and surviving contract changes without manual rework.

## Phases

- [x] **Phase 1: Absence Domain Foundation** — neue parallele `absence` Domain (DAO + Service + REST + Permission), additiv, ohne Reporting-Wirkung (completed 2026-05-01)
- [x] **Phase 2: Reporting Integration & Snapshot Versioning** — `derive_hours_for_range` + Reporting-Switch hinter Feature-Flag, `CURRENT_SNAPSHOT_SCHEMA_VERSION` 2 → 3 im selben Commit (completed 2026-05-02)
- [x] **Phase 3: Booking & Shift-Plan Konflikt-Integration** — Forward/Reverse Booking-Warnings + Shift-Plan-Anzeige aus AbsencePeriod ohne Doppel-Eintragung (completed 2026-05-02, alle 4 SCs erfüllt, D-Phase3-18 Regression-Lock final 0-Diff)
- [ ] **Phase 4: Migration & Cutover** — Heuristik-Migration, Validierungs-Gate, atomarer Feature-Flag-Flip mit Carryover-Refresh, REST-Deprecation

## Phase Details

### Phase 1: Absence Domain Foundation

**Goal**: Eine neue, parallele `absence` Domain existiert end-to-end (Schema, DAO, Service, REST, DI), permission-gated, ohne Auswirkung auf Reporting/Snapshots/Booking-Flows. Entwickler können Absences anlegen, lesen, ändern und (soft-)löschen; alle Tests grün.

**Depends on**: Nothing (Foundation-Phase)

**Requirements**: ABS-01, ABS-02, ABS-03, ABS-04, ABS-05

**Success Criteria** (what must be TRUE):
  1. Ein Vorstand/HR-User kann via REST eine `AbsencePeriod` (Vacation/Sick/UnpaidLeave) mit `from_date`/`to_date` anlegen, abrufen, ändern und soft-löschen — unabhängig vom bestehenden ExtraHours-Pfad.
  2. Ein Mitarbeiter ohne HR-Rechte erhält bei jeder schreibenden Operation auf fremde `AbsencePeriod`s `403 Forbidden`; `_forbidden`-Tests existieren für jede public service method.
  3. Self-Overlap (gleicher Mitarbeiter + gleiche Kategorie + überlappender Zeitraum) wird vom Service erkannt und als `ServiceError`-Variante zurückgewiesen.
  4. `cargo test` und `cargo build` sind grün; ein Integration-Test in `shifty_bin/src/integration_test/` deckt CRUD-Round-Trip einschließlich Soft-Delete + `logical_id`-Update-Pfad ab.
  5. Bestehende Reporting-, Booking- und Snapshot-Pfade liefern bit-identische Ergebnisse wie vor der Phase (Phase ist additiv — Beweis: bestehende Tests unverändert grün).

**Plans**: 5 plans

Plans:
- [x] 01-00-PLAN.md — Wave 0 Foundation: Migration `<TS>_create-absence-period.sql` (CHECK + 3 partial indexes), `shifty_utils::DateRange` Utility, `ValidationFailureItem::OverlappingPeriod(Uuid)` Variante.
- [x] 01-01-PLAN.md — Wave 1 DAO: `dao::absence::AbsenceDao` Trait + Entity + automock; `dao_impl_sqlite::absence::AbsenceDaoImpl` mit 7 SQLx-Methoden inkl. Two-Branch `find_overlapping`.
- [x] 01-02-PLAN.md — Wave 2 Service: `service::absence::AbsenceService` Trait + Domain-Modell; `service_impl::absence::AbsenceServiceImpl` mit `gen_service_impl!`-DI, Range-Validierung, Self-Overlap mit `exclude_logical_id`, HR ∨ self Permission, logical_id-Update; 13+ Service-Tests inkl. `_forbidden` pro public method (D-11/ABS-05).
- [x] 01-03-PLAN.md — Wave 3 REST: `AbsencePeriodTO` + `AbsenceCategoryTO` inline in `rest-types/src/lib.rs`; 6 REST-Handler in `rest/src/absence.rs` mit utoipa; `RestStateDef`-Erweiterung, ApiDoc-Nest, Router-Nest in `rest/src/lib.rs`.
- [x] 01-04-PLAN.md — Wave 4 DI + Integration: `AbsenceServiceDependencies`-Block in `shifty_bin/src/main.rs`; 8 Integration-Tests in `shifty_bin/src/integration_test/absence_period.rs` (CRUD, Schema-Constraints, Self-Overlap, D-12, D-15, Soft-Delete); Final-Smoke-Gate `cargo test --workspace`.

**Discuss-phase carry-overs**: Domain-Naming (Absence vs. AbsencePeriod vs. TimeOff), exakte Kategorie-Liste in Scope (5 vs. 6 vs. 7) — beides muss vor `/gsd:plan-phase 1` geklärt sein, da es Schema und Trait-Namen treibt.

---

### Phase 2: Reporting Integration & Snapshot Versioning

**Goal**: Reporting kann Absence-derived Stunden zusätzlich zu ExtraHours summieren (Feature-Flag-gesteuert), per-Tag gegen den am jeweiligen Tag gültigen Vertrag berechnet, mit korrekter Feiertags-Orthogonalität. `CURRENT_SNAPSHOT_SCHEMA_VERSION` ist auf 3 gebumpt — im **selben Commit** wie der Reporting-Switch.

**Depends on**: Phase 1 (AbsenceService existiert)

**Requirements**: REP-01, REP-02, REP-03, REP-04, SNAP-01, SNAP-02

**Success Criteria** (what must be TRUE):
  1. `AbsenceService::derive_hours_for_range(from, to, sales_person_id)` liefert pro Tag im Range die Vertragsstunden des am Tag gültigen `EmployeeWorkDetails`-Vertrages; Feiertage innerhalb des Ranges liefern 0 Urlaubsstunden (kein Verbrauch); Vertragswechsel mitten im Range produziert für jeden Tag den jeweils gültigen Wert.
  2. Solange `absence.range_source_active` aus ist, liefern alle bestehenden Reports, Bilanzen und Snapshots **bit-identische** Werte wie vor Phase 2 (Test: Snapshot eines Fixtures vor und nach Phase-2-Code = identisch).
  3. Ist der Flag an, wechselt der Reporting-Pfad atomar zur neuen Quelle für Vacation/Sick/UnpaidLeave; ein `EmployeeReport` über einen Range mit `AbsencePeriod`-Einträgen produziert die erwarteten Delta-Stunden (positiver Test mit Fixture).
  4. `CURRENT_SNAPSHOT_SCHEMA_VERSION = 3`; ein Locking-Test (`service_impl/src/test/billing_period_report.rs`) schlägt zur Build-Zeit fehl, wenn die Berechnungs-Logik der Reporting-Inputs sich ändert ohne dass die Konstante sich ändert.
  5. Bestehende Snapshots der Version 2 bleiben lesbar; neue Snapshots (egal ob Flag an oder aus zur Erstellungs-Zeit) tragen Version 3; Validatoren erkennen den Unterschied.

**Plans**: 4 plans

Plans:
- [x] 02-01-PLAN.md — Wave 0 Test-Scaffolding: 5 neue Test-Dateien (Fixtures + Pin/Match-Locking-Test + 3 Stubs), Pin-Test pre-Wave-2 ROT als Wave-2-Forcing. **Completed 2026-05-02 (commits d8dad0aa, f85f4a3f, 0eeff84c, 726e919c).**
- [x] 02-02-PLAN.md — Wave 1 derive_hours_for_range: AbsenceService um Cross-Category-Resolver + Per-Tag-Vertrags-Lookup + Feiertags-0-Auflösung erweitern. **Completed 2026-05-02 (commits 8fafb6ef, 3e371b06, ae7d0642).**
- [x] 02-03-PLAN.md — Wave 1 FeatureFlagService: neuer Service-Trait + Impl + DAO + Schema-Migration + Privileg.
- [x] 02-04-PLAN.md — Wave 2 Atomic Reporting-Switch: Snapshot-Bump 2→3, UnpaidLeave-Variante, Reporting-Switch hinter Flag, Pin-Map-Test (alle 12 Varianten), Match-Test-Aktivierung — alles in einem jj-Commit (D-Phase2-10).

**Discuss-phase carry-overs**: Sick-overlapping-Vacation Policy (BUrlG §9-Konflikt) — muss vor Plan-Phase 2 dokumentiert sein, da `derive_hours_for_range` die Cross-Category-Overlap-Auflösung kennen muss. Liste der `value_type`s im Snapshot (welche werden in der Berechnung berührt) — beeinflusst Locking-Test-Scope.

---

### Phase 3: Booking & Shift-Plan Konflikt-Integration

**Goal**: Bookings und Shift-Plan kennen `AbsencePeriod` als Konflikt-Quelle. Forward-Warning beim Anlegen einer überlappenden Absence, Reverse-Warning beim Anlegen eines Bookings auf einem absence-day, Shift-Plan markiert absence-days automatisch — die bisherige doppelte Eintragung (ExtraHours + sales_person_unavailable) entfällt für die zeitraum-basierten Kategorien.

**Depends on**: Phase 1 (AbsenceService existiert; kann parallel zu Phase 2 entwickelt werden, blockiert aber bis Phase 1 stabil ist)

**Requirements**: BOOK-01, BOOK-02, PLAN-01

**Success Criteria** (what must be TRUE):
  1. Beim Anlegen einer `AbsencePeriod`, die ein bestehendes Booking überlappt, gibt der Service einen `BookingCreateResult`-analogen Wrapper mit `Vec<Warning>` zurück (mit konkreten Booking-IDs und Daten); die Absence wird trotzdem persistiert; kein Auto-Löschen.
  2. Beim Anlegen eines Bookings auf einem Tag, der entweder durch `AbsencePeriod` oder durch `sales_person_unavailable` als nicht verfügbar markiert ist, gibt der Booking-Service eine Warnung zurück; bestehende Reverse-Warnung-Tests via `sales_person_unavailable` bleiben grün (keine Regression).
  3. Eine Shift-Plan-Anzeige für einen Mitarbeiter über einen Zeitraum markiert alle Tage als nicht verfügbar, wenn ein `AbsencePeriod` den Tag enthält **oder** ein `sales_person_unavailable`-Eintrag existiert; manuelle `sales_person_unavailable`-Einträge bleiben für Einzeltage außerhalb fester Zeiträume möglich (kein Konflikt zwischen den Quellen).
  4. Soft-deleted `AbsencePeriod`s triggern keine Warnung und keine Shift-Plan-Markierung (Pitfall-6-Test grün).

**Plans**: 6 plans

Plans:
**Wave 1**
- [x] 03-01-PLAN.md — Wave 0 Test-Scaffolding (Stubs für Reverse-Warning + Cross-Source + Pitfall-1)
- [x] 03-02-PLAN.md — Wave 1 Domain-Surface (Warning-Enum + AbsenceDao::find_overlapping_for_booking + UnavailabilityMarker + ShiftplanDay-Field)

**Wave 2**
- [x] 03-03-PLAN.md — Wave 2 AbsenceService (Sig-Brüche create/update zu AbsencePeriodCreateResult + Forward-Warning-Loop + find_overlapping_for_booking + neue DI-Deps)

**Wave 3**
- [x] 03-04-PLAN.md — Wave 3 ShiftplanEditService Reverse-Warning + ShiftplanViewService per-sales-person + DI-Wiring

**Wave 4**
- [x] 03-05-PLAN.md — Wave 4 REST-Layer (5 Wrapper-DTOs + Wrapper-Body für /absence-period + 2 neue /shiftplan-edit-Endpunkte + 2 neue per-sales-person-Endpunkte unter /shiftplan-info + ApiDoc; D-Phase3-18 Regression-Lock erfüllt)

**Wave 5**
- [x] 03-06-PLAN.md — Wave 5 Tests aktiviert (4 forward-warning-Tests in test/absence.rs, 1 marker_manual_only-Test in test/shiftplan.rs, 4 cross-source Integration-Tests in booking_absence_conflict.rs) + Phase-1-Migration recovered (Bonus: 8 absence_period-Tests jetzt grün) + SC1-SC4 alle erfüllt + D-Phase3-18 Regression-Lock final verifiziert

**Cross-cutting constraints:**
- BookingService-Files (`service/src/booking.rs`, `service_impl/src/booking.rs`, `rest/src/booking.rs`, `service_impl/src/test/booking.rs`) bleiben UNVERÄNDERT (D-Phase3-18 Regression-Lock).

**Discuss-phase carry-overs**: keine offenen Entscheidungen blockierend für diese Phase. Reverse-Warning-Direction ist durch BOOK-02 bereits eindeutig festgelegt (symmetrisch).

---

### Phase 4: Migration & Cutover

**Goal**: Bestehende `ExtraHours`-Einträge der Kategorien Vacation/Sick/UnpaidLeave werden heuristisch zu `AbsencePeriod`-Zeiträumen rekonstruiert. Vor dem Feature-Flag-Flip stellt ein Validierungs-Gate **pro Mitarbeiter und pro Kategorie** sicher, dass die summierten Stunden identisch bleiben. Erst dann wird der Flag in einer atomaren Transaktion geflippt — inklusive Carryover-Refresh. Bestehende ExtraHours-REST-Endpunkte bleiben funktional oder sind klar deprecation-markiert. **Diese Phase ist atomar — MIG-01 bis MIG-04 müssen zusammen committet/deployt werden, weil das Feature dormant bleibt bis das Gate grün ist.**

**Depends on**: Phase 2 (`derive_hours_for_range` muss bewiesen korrekt sein, da Validierung dieselbe Logik fährt) und Phase 1 (Schema existiert). Phase 3 ist nicht harte Voraussetzung, sollte aber gemerged sein damit der Cutover die volle Feature-Surface aktiviert.

**Requirements**: MIG-01, MIG-02, MIG-03, MIG-04, MIG-05

**Success Criteria** (what must be TRUE):
  1. Ein read-only Production-Data-Profile (Histogramm der Bestands-Einträge: Stunden-pro-Eintrag, Bruchstunden-Quote, Wochen mit Vertragswechsel, ISO-Woche-53, Wochenend-Einträge) liegt in `.planning/migration-backup/` vor Beginn jeglicher Migrations-Logik.
  2. Die Heuristik-basierte Migration erzeugt für jede eindeutig konvertierbare ExtraHours-Reihe genau eine `AbsencePeriod`; nicht eindeutig konvertierbare Bestände landen in einer Quarantäne (kein Datenverlust); ein Re-Run der Migration ist idempotent (keyed on `logical_id`).
  3. Das **Cutover-Gate** (MIG-02) prüft pro `(sales_person_id, kategorie)` über alle relevanten Zeiträume: `sum(derive_hours_for_range(...)) == sum(extra_hours_legacy)` mit Toleranz < 0.01h. Eine einzige Abweichung lehnt den Flag-Flip ab und produziert einen strukturierten Diff-Report; der Feature-Flag bleibt aus.
  4. Bei grünem Gate wird `absence.range_source_active` in derselben Transaktion wie MIG-01 (Migration) und MIG-04 (Carryover-Rewrite für jedes betroffene Jahr, mit Pre-Migration-Backup) auf `true` gesetzt. Schlägt irgendein Schritt fehl, wird die gesamte Transaktion zurückgerollt und das Feature bleibt dormant.
  5. Ein Per-Mitarbeiter-Per-Jahr-Per-Kategorie-Invariant-Test (`shifty_bin/src/integration_test/`) bestätigt: Pre-Migration-Stunden-Summe == Post-Migration-derived-Stunden-Summe für jede Kombination.
  6. Bestehende `/extra-hours`-REST-Endpunkte für Vacation/Sick/UnpaidLeave bleiben entweder funktional (read-compat shim) oder sind mit klarer Deprecation-Strategie und Aufrufer-Information abgelöst; ein OpenAPI-Snapshot-Test verhindert ein stilles Breaking Change auf bestehenden Endpunkten.

**Plans**: TBD

**Discuss-phase carry-overs (blockierend für Plan-Phase 4):**
  - Migrations-Heuristik-Spezifika: Werktage = Mo-Fr fix? Verhalten bei Teilwochen? Bruchstunden-Behandlung? Vertragswechsel mitten in der Woche?
  - REST-Transition-Strategie: Read-Compat-Shim auf `/extra-hours` vs. saubere v2-Versionierung — beeinflusst MIG-05-Komplexität.
  - Schicksal der alten ExtraHours-Vacation/Sick/UnpaidLeave-Einträge **nach** Cutover: soft-delete in-place, Move zu Archiv-Tabelle, oder live behalten und Reporting-seitig ignorieren.
  - Carryover-Refresh-Scope: alle Jahre mit migrierten Absences, oder nur das laufende Jahr?

## Build Order Rationale

Die Phasen-Reihenfolge folgt strikt der Research-Empfehlung (`research/SUMMARY.md`, `research/ARCHITECTURE.md`):

1. **Phase 1 ist Foundation** — DAO + Service + REST sind additiv; Bestandscode bleibt unberührt; kann mit hoher Konfidenz gemerged werden bevor irgendein Integrations-Risiko eingeführt wird.

2. **Phase 2 koppelt Reporting + Snapshot-Versionierung in einer Phase** — die `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump-Pflicht (per `CLAUDE.md`) verlangt, dass der Bump im **selben Commit** landet wie die Änderung der Reporting-Inputs. Phase 2 ist bewusst der highest-risk seam dieser Iteration; sie ist aber komplett hinter `absence.range_source_active = false` versteckt — alles bleibt dormant.

3. **Phase 3 (Booking + Shift-Plan)** ist parallel-entwickelbar zu Phase 2, weil beide Phasen nur von Phase 1 abhängen. Sie liefert die User-sichtbare Doppel-Eintragungs-Eliminierung und die Konflikt-Warnings, ohne die Reporting-Berechnung anzufassen.

4. **Phase 4 ist die milestone-finale Verpflichtung** — sie greift die einzige irreversible Operation (Production-Daten-Migration) und enthält das Cutover-Gate. Alle Derivation-Logik (Phase 2) muss bewiesen korrekt sein, weil die Migrations-Validierung dieselbe Logik fährt. **MIG-01..04 sind atomar — der Feature-Flag flippt nur bei grünem Gate, sonst Rollback.** MIG-05 (REST-Deprecation) reist mit, weil sie ohnehin gemeinsam kommuniziert werden muss.

**Granularität-Anmerkung**: Standard-Granularity zielt auf 5-7 Phasen. Hier sind 4 Phasen entstanden — bewusst, weil das Work-Domain genau 4 natürliche Delivery-Grenzen hat (additive Foundation; Reporting-Switch hinter Flag; User-sichtbare Konflikt-Integration; Migration-mit-Cutover). Eine Auf-Splittung von Phase 1 in DAO+Service vs. REST würde horizontale Layer-Phasen erzeugen (Anti-Pattern); eine Auf-Splittung von Phase 4 würde MIG-01..04-Atomarität brechen (Hart-Constraint). Vier Phasen mit jeweils 3-5 Plans erfüllt den Granularity-Geist.

## Coverage Summary

| Phase | Goal | Requirements | # Success Criteria |
|---|---|---|---|
| 1 — Absence Domain Foundation | parallele `absence` Domain end-to-end, additiv | ABS-01, ABS-02, ABS-03, ABS-04, ABS-05 | 5 |
| 2 — Reporting Integration & Snapshot Versioning | derive-on-read hinter Feature-Flag + Snapshot-Bump 2→3 im selben Commit | REP-01, REP-02, REP-03, REP-04, SNAP-01, SNAP-02 | 5 |
| 3 — Booking & Shift-Plan Konflikt-Integration | Forward/Reverse Booking-Warnings + Shift-Plan-Anzeige aus AbsencePeriod ohne Doppel-Eintragung | BOOK-01, BOOK-02, PLAN-01 | 4 |
| 4 — Migration & Cutover | Heuristik-Migration + Validierungs-Gate + atomarer Flag-Flip + REST-Deprecation | MIG-01, MIG-02, MIG-03, MIG-04, MIG-05 | 6 |

**Total v1 requirements:** 19
**Mapped:** 19
**Orphaned:** 0

## Progress

| Phase | Plans Complete | Status | Completed |
|---|---|---|---|
| 1 — Absence Domain Foundation | 5/5 | Complete | 2026-05-01 |
| 2 — Reporting Integration & Snapshot Versioning | 4/4 | Complete | 2026-05-02 |
| 3 — Booking & Shift-Plan Konflikt-Integration | 6/6 | Complete | 2026-05-02 |
| 4 — Migration & Cutover | 0/0 | Not started | — |

## Discuss-Phase Carry-Overs (Aggregated)

Diese offenen Entscheidungen aus `PROJECT.md`/`research/SUMMARY.md` sind **keine Roadmapper-Entscheidungen** — sie werden im `/gsd:discuss-phase` vor der jeweiligen `/gsd:plan-phase N` geklärt. Aufgelistet hier zur Sichtbarkeit.

| # | Decision | Blockierend für Phase |
|---|---|---|
| 1 | Domain-Naming (`Absence` / `AbsencePeriod` / `TimeOff` / …) | 1 |
| 2 | Genauer Scope der Kategorien (5 vs. 6 vs. 7 — `UnpaidLeave` ja, `VolunteerWork`/`Holiday`/`Unavailable` nein per `Out of Scope`) | 1 |
| 3 | Lazy-on-read formell festhalten (Empfehlung aus Research) | 1 / 2 |
| 4 | Sick-overlapping-Vacation Policy (BUrlG §9-Konflikt) | 2 |
| 5 | Liste der berührten `value_type`s im Snapshot (Locking-Test-Scope) | 2 |
| 6 | Migrations-Heuristik-Spezifika (Werktage Mo-Fr, Teilwochen, Bruchstunden, Vertragswechsel-mid-week) | 4 |
| 7 | REST-Transition-Strategie (Read-Compat-Shim vs. saubere v2-Versionierung) | 4 |
| 8 | Schicksal alter `ExtraHours`-Einträge nach Cutover | 4 |
| 9 | Carryover-Refresh-Scope (alle Jahre vs. laufendes Jahr) | 4 |

---

*Roadmap created: 2026-05-01*
*Last updated: 2026-05-02 — Phase 3 complete (all 6 plans done; SC1-SC4 verified; D-Phase3-18 Regression-Lock final 0-diff)*
