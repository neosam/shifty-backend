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
- `PersonService` — Entity-Manager für die Identitäts-Elterntabelle
  `person` (v2.7 Phase 61). Alle vier Methoden sind HR-gated; zwei davon
  sind die „unverknüpfte Kandidaten"-Lesepfade, die Phase 63 für den
  Kandidaten-Picker braucht. Deps: `PersonDao`, `PermissionService`,
  `ClockService`, `UuidService`, `TransactionDao` — bewusst kein
  Domain-Service, weil ein solcher über `PermissionService` einen
  DI-Zyklus schließen würde. Konstruktion im Basic-Block von
  `shifty_bin/src/main.rs`, strikt vor der Business-Logic-Schicht. Seit
  v2.7 Phase 62 existiert der erste Konsument-Kontext:
  `PersonDirectoryService` liegt eine Schicht darüber und liest dieselben
  Tabellen über einen eigenen DAO — der Basic-Service selbst bleibt
  unverändert und HR-gegated. Siehe
  Feature [F01](../features/F01-employee-management_de.md) §4.5.
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
- `RebookingReconciliationService` — F3/F5-Rebooking-Write-Orchestrator
  (v2.6 Phase 55). Besitzt den einzigen internen Pair-Write-Path,
  geteilt von der F3-manuellen-One-Shot- und der F5-approve-Lifecycle.
  Konsumiert `ExtraHoursService`, `RebookingBatchService`,
  `ReportingService`, `VoluntaryStatsService` (+ `PermissionService`,
  `TransactionDao`) — klar ein Business-Logic-Service (vier
  Domain-Service-Deps). Kein Zyklus: keiner dieser vier kennt ihn
  zurück. Alle vier Methoden HR-gated (T-55-05). DI-konstruiert in
  `shifty_bin/src/main.rs` **nach** allen Basic-Services und nach
  `ReportingService` + `VoluntaryStatsService` (Reihenfolge:
  `extra_hours` → `rebooking_batch` → `reporting` → `voluntary_stats`
  → `rebooking_reconciliation`, zuletzt). Siehe Feature
  [F14 §8](../features/F14-rebooking.md).
- `VoluntaryRebookingScheduler` — F4-Auto-Cron-Scheduler (v2.6
  Phase 56). Ein schlanker `tokio-cron-scheduler`-Job (Montag 03:15,
  6-Feld-Cron `0 15 3 * * 1`), der die vorige ISO-Woche berechnet und
  jede gedeckelte Person an
  `RebookingReconciliationService::run_for_week` delegiert. Konsumiert
  `RebookingReconciliationService` (die Engine) +
  `EmployeeWorkDetailsService` (die Quelle der gedeckelten Personen) (+
  `PermissionService`, `ClockService`, `TransactionDao`) — ein
  Business-Logic-Service (zwei Domain-Service-Deps). Spiegelt den
  `PdfExportScheduler` ohne `reload_from_db` (der Stichtag-Toggle wird
  bei jedem Lauf frisch in `run_for_week` gelesen). Gestartet in
  `shifty_bin/src/main.rs` **nach** `pdf_export_scheduler.start()` am
  Ende von `main()` — der DI-Verdrahtungs-Trigger für dieses Dokument.
  Keine REST-Oberfläche (der Endpoint `/admin/rebooking/backfill` hängt
  am `RebookingReconciliationService`). Siehe Feature
  [F14 §9](../features/F14-rebooking.md).
- `PersonDirectoryService` — Lese-Aggregat hinter der einen
  Personen-Liste, die die bisherigen Tabs der Benutzerverwaltung ablöst
  (v2.7 Phase 62). Deps: `PersonDirectoryDao`, `PermissionService`,
  `TransactionDao` — **kein** einziger Domain-Service. Er ist trotzdem
  Business-Logic, weil er die beiden Träger-Aggregate einer Person
  (`sales_person` und `user`) zu einer Sicht zusammenführt und über sie
  hinweg die `PersonKind`-Invariante pflegt; siehe die Klarstellung
  unter Regel 3 weiter unten. `list` ist auf das Literal `"admin"`
  gegated und **propagiert** den Fehler — bewusst anders als
  `VoluntaryStatsService`, der auf `None`-Felder redigiert: ein
  redigiertes Verzeichnis wäre von einem leeren ununterscheidbar.
  DI-konstruiert in `shifty_bin/src/main.rs` **nach** dem Basic-Block,
  insbesondere strikt nach `PersonService`, direkt im Anschluss an
  `voluntary_stats_service`. Siehe Feature
  [F01](../features/F01-employee-management_de.md) §4.6.
- `PersonDetailService` — das Aggregat hinter der einen
  Personen-Detailseite und ihren Ausprägungs-Aktionen (v2.7 Phase 63).
  Deps: `PersonDirectoryDao`, `PersonDao`, `PermissionDao`,
  `SalesPersonDao`, `PersonService`, `SalesPersonService`,
  `PermissionService`, `TransactionDao` — **acht**, und zwei davon sind
  Basic-Services. Zweifelsfrei Business-Logic: er orchestriert über zwei
  Fach-Objekte hinweg (Sales Person und Benutzerkonto) und pflegt mit dem
  Ausprägungs-Lebenszyklus eine Cross-Entity-Invariante — Anlegen,
  Zuschalten, Entfernen einer Seite, plus dem Abbau der trägerlos
  gewordenen Person [D-63-01].
  **Warum zwei Basic-Service-Deps keinen Zyklus schließen:** die
  Tier-Konvention erlaubt der Business-Logic-Schicht ausdrücklich, Basic-
  Services zu konsumieren; verboten ist nur die Gegenrichtung.
  `PersonService` und `SalesPersonService` sind beide Entity-Manager (nur
  DAO + Permission + Clock + Uuid + Transaction) und kennen dieses Aggregat
  nicht — es gibt also keine Kante zurück. Auch sonst konsumiert es kein
  Service; es hängt ausschließlich an den REST-Handlern.
  Alle sieben Methoden sind auf das Literal `"admin"` gegated, als erste
  Anweisung, und damit bewusst **strenger** als die HR-Gates der
  darunterliegenden Basic-Services; der Service ruft diese intern deshalb mit
  `Authentication::Full` auf — erreichbar erst nach dem eigenen Gate.
  DI-konstruiert in `shifty_bin/src/main.rs` **nach** dem Basic-Block —
  insbesondere strikt nach `person_service`, `sales_person_service` und
  `permission_service` — im Business-Logic-Abschnitt direkt hinter
  `person_directory_service`. Siehe Feature
  [F01](../features/F01-employee-management_de.md) §4.7.

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

   **Die Faustregel ist ein Indikator, nicht die Entscheidung.** Es
   entscheidet die inhaltliche Frage: verwaltet der Service genau *ein*
   Aggregat? Ein Lese-Aggregat, das mehrere Fach-Objekte zu einer Sicht
   zusammenführt, ist Business-Logic — auch wenn seine Dependency-Liste wie
   die eines Basic-Service aussieht. `PersonDirectoryService` (v2.7
   Phase 62) ist genau dieser Fall: drei Dependencies, kein Domain-Service
   — und trotzdem Business-Logic, weil er `sales_person` und `user` zu
   einer Personen-Sicht verbindet.

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
