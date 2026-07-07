# Service-Tiers — Basic vs Business-Logic

Shifty trennt Service-Implementierungen in zwei Schichten. Diese Trennung
verhindert zyklische DI-Kopplung und hält die Konstruktionsreihenfolge in
`shifty_bin/src/main.rs` deterministisch.

## Basic Services (Entity-Manager)

Ein **Basic Service** verwaltet genau ein Fach-Objekt:

- CRUD + Validation + Permission-Gates für sein Aggregat.
- Konsumiert **nur** DAOs, `PermissionService`, `TransactionDao`.
- Konsumiert **keine** anderen Domain-Services.

**Beispiele:**

- `BookingService`
- `RebookingBatchService` — Entity-Manager für `rebooking_batch` +
  `rebooking_batch_entry` (v2.6 Phase 54). Ausschließlich HR-gated
  CRUD; erster Konsument folgt in Phase 55
  (`RebookingReconciliationService`). Deps: `RebookingBatchDao`,
  `PermissionService`, `ClockService`, `UuidService`,
  `TransactionDao`. Siehe Feature
  [F14](../features/F14-rebooking.md).
- `SalesPersonService`
- `SalesPersonUnavailableService`
- `SlotService`
- `ShiftplanService` (Stamm-Daten)
- `SpecialDayService`

## Business-Logic Services

Ein **Business-Logic Service** kombiniert mehrere Aggregate oder pflegt
Cross-Entity-Invarianten:

- Konsumiert Basic Services und andere Business-Logic Services — solange
  kein zyklisches Coupling entsteht.
- Aggregiert oft Read-Only-Daten aus mehreren Basic-Services (dann
  meist mit `Authentication::Full` intern, siehe
  [`04-auth.md`](./04-auth.md)).

**Beispiele:**

- `AbsenceService` — Multi-Tag-Range, Kategorie-Logik, Konflikt-Lookups.
- `ShiftplanViewService` — Read-Aggregat über Slot + Booking + Absence.
- `ShiftplanEditService` — Write-Aggregat mit Booking-Migration bei
  Slot-Änderungen.
- `ReportingService` — Balance-Rechnung über Booking + ExtraHours +
  Absence + Carryover + SpecialDay.
- `BookingInformationService` — Angereicherte Booking-Ansichten.
- `CarryoverService` — Jahresend-Snapshot mit Cross-Year-Konsistenz.
- `WorkingHoursService` — Erwartungs-Rechnung.
- `BillingPeriodReportService` — Snapshot-Erzeugung.
- `VoluntaryStatsService` — Read-only F1/F2-Aggregat auf Basis von
  `ExtraHoursService` + `EmployeeWorkDetailsService` +
  `SalesPersonService` (v2.6 Phase 54). HR-only via API-Level
  None-Redaktion (Non-HR erhält alle Felder `None`, kein 403).
  Siehe Feature [F14](../features/F14-rebooking.md).

## Regeln

1. **Wenn zwei Services sich gegenseitig brauchen:** Einer ist Basic,
   einer ist Business-Logic. Der Basic kennt den Business-Logic-Service
   nicht. Bei Bedarf wandert die Cross-Entity-Operation in einen
   dritten Service eine Schicht höher.
2. **DI-Konstruktion in `main.rs`:** erst alle Basic Services, dann die
   Business-Logic-Schicht — keine `OnceLock`-/Forward-Decl-Tricks.
3. **Faustregel zur Klassifizierung:** Dependencies zählen.
   - Nur DAOs + Permission + Transaction → Basic.
   - Sobald ein anderer Domain-Service als Dep auftaucht → Business-Logic.

## Warum zwei Tiers?

**Ohne die Trennung** landet man schnell in zyklischen Deps:

- `BookingService` will beim Löschen einen Absence-Konflikt prüfen →
  ruft `AbsenceService`.
- `AbsenceService` will beim Löschen einer Absence checken, ob eine
  Booking sich darauf bezieht → ruft `BookingService`.

Mit Tiers wird der Zyklus explizit gebrochen: `BookingService` bleibt
Basic, kennt `AbsenceService` **nicht**. Der Cross-Entity-Check wandert
in einen dritten Service (z.B. `ShiftplanEditService`), der beide
konsumiert.

## Der Service-Graph

Die tatsächliche DI-Verdrahtung aus `shifty_bin/src/main.rs` ist als
Mermaid-Diagramm in
[`diagrams/service-graph-runtime.mmd`](./diagrams/service-graph-runtime.mmd)
generiert. Die Trait-Deklarations-Version (was jeder Service als
Abhängigkeit **fordert**, unabhängig von der Reihenfolge) liegt in
[`diagrams/service-graph-traits.mmd`](./diagrams/service-graph-traits.mmd).

## Historie

Die Tier-Konvention wurde nachträglich formalisiert, nachdem zwei
Refactoring-Zyklen mit versteckten Zyklen gescheitert waren. Sie steht
verbindlich in `shifty-backend/CLAUDE.md` und wird bei
Service-Reviews aktiv geprüft.
