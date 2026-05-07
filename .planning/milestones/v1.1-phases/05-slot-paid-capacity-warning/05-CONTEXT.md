# Phase 5: Slot Paid Capacity Warning - Context

**Gathered:** 2026-05-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Slots erhalten ein **optionales** Capacity-Limit für bezahlte Mitarbeiter:innen (`slot.max_paid_employees: Option<u8>`). Wenn der aktuelle Live-Count an aktiven Bookings im Slot mit `sales_person.is_paid = true` das konfigurierte Limit übersteigt, emittiert das Backend nicht-blockierende **Warnings** über zwei Kanäle:

1. **Booking-Create**: Warning-Variant in `BookingCreateResult.warnings` (etabliertes Pattern aus v1.0 Phase 3).
2. **Shiftplan-Week-View**: Pro Slot werden `max_paid_employees` (Konfig) und `current_paid_count` (berechnet) im Read-DTO mitgeliefert. Konsumenten (späteres Frontend) können `current > max` selbst auswerten.

Buchen bleibt **erlaubt** — die Warning ist informativ, nicht blockierend. `NULL`-Konfiguration bedeutet „kein Limit" (kein Check, keine Warning, keine Read-Felder).

**In Scope (Phase 5):**

- **Schema-Migration**: neue nullable Spalte `slot.max_paid_employees INTEGER` (default `NULL`).
- **`SlotEntity`-Erweiterung**: `pub max_paid_employees: Option<u8>` plus DAO-Read/Write-Adaption.
- **Slot-REST-DTOs** (Create/Update/Read): neues Feld `max_paid_employees: Option<u8>` mit utoipa-`ToSchema`.
- **Slot-REST-Update-Permission**: Rolle `shiftplanner` (analog `min_resources`-Update).
- **ShiftplanEditService Limit-Check**: Nach erfolgreichem Booking im conflict-aware Pfad (`POST /shiftplan-edit/booking`) wird der Live-Paid-Count im Ziel-Slot+Woche berechnet; `current > max` ergibt Warning-Variant. Lebt im Business-Logic-Tier (`ShiftplanEditService` hat bereits `BookingService`- und `SalesPersonService`-Deps), nicht im Basic-Tier `BookingService`.
- **Warning-Variant**: neuer Variant im bestehenden Booking-Warning-Enum, z. B. `Warning::PaidEmployeeLimitExceeded { slot_id, current_paid_count, max_paid_employees }`. Strukturiert (keine i18n-Strings im Backend).
- **Shiftplan-Week-View-Aggregation**: pro Slot werden `max_paid_employees` und `current_paid_count` als `Option<…>` / Field im Read-DTO geliefert (nur wenn `max_paid_employees` gesetzt; sonst beide Felder absent oder `None`).
- **Pflicht-Tests**:
  - DAO-Roundtrip für Slot mit/ohne `max_paid_employees`.
  - Service-Test: Booking eines bezahlten Mitarbeiters in Slot mit `max=2` und 2 bestehenden bezahlten Bookings → Warning.
  - Service-Test: Booking eines unbezahlten Mitarbeiters → keine Warning.
  - Service-Test: Booking mit `max_paid_employees = NULL` → kein Check, keine Warning.
  - Service-Test: Booking eines bezahlten Mitarbeiters, der gleichzeitig in einer Absence-Periode liegt → zählt trotzdem als „eingetragen", Warning wird ausgelöst sobald Limit überschritten ist.
  - Read-Test: Shiftplan-Week-View liefert `current_paid_count` korrekt pro Slot/Woche.
  - REST-Test: Slot-Update mit `max_paid_employees` als `shiftplanner` (200) und ohne Permission (403).
  - REST-Test: Booking via `POST /shiftplan-edit/booking` in überlasteten Slot liefert `BookingCreateResult.warnings` mit dem neuen Variant.

**Strikt nicht in Scope (Phase 5):**

- **Frontend (shifty-dioxus)**: separater Workstream, anderes Repo. UI-Konfiguration des Limits, visuelle Indikatoren im Grid und i18n-Strings für Warnings werden im Frontend-Repo umgesetzt.
- **Hartes Buchungs-Limit / Reject-Pfad**: das Limit ist ausschließlich Soft-Warning. Kein 4xx-Status, kein Tx-Rollback.
- **Legacy-Booking-Endpoint (`POST /booking`)**: keine Warning-Emission. Pfad hat keinen `BookingCreateResult`-Wrapper, Frontend migriert sowieso auf `/shiftplan-edit/booking` (v1.0-Phase-4-Verantwortlichkeit).
- **OpenAPI-Snapshot-Test**: existiert nicht mehr. `rest/tests/openapi_snapshot.rs` wurde in Commit `fdb70b5` (`test(rest): remove flaky openapi snapshot test`) entfernt. Keine Snapshot-Update-Pflicht in Phase 5.
- **`update_slot` DAO-Fix für `min_resources`**: existierender Gap (`min_resources` wird im UPDATE nicht persistiert) bleibt unberührt. Phase 5 macht `update_slot` für `max_paid_employees` von Anfang an korrekt; `min_resources` ist out of scope.
- **Min-Paid-Capacity / Skill-Matching / weitere Slot-Constraints**: reserviert für spätere Phasen v1.x.
- **Historische Backfill-Migration**: keine bestehenden Slots werden berührt — alle bestehenden Rows bekommen `NULL`.
- **`CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump**: nicht betroffen — Phase 5 ändert keine `BillingPeriodValueType`-Berechnung und keine Snapshot-Inputs.
- **Absence-Bereinigung des Counts**: explizit verworfen — eingetragen ist eingetragen, Absence-Status ist orthogonal.

</domain>

<decisions>
## Implementation Decisions

### Datenmodell
- **D-01:** Neue nullable Spalte `slot.max_paid_employees INTEGER` (DEFAULT `NULL`, kein NOT NULL). `NULL` = kein Limit. Bestehende Rows bleiben unverändert.
- **D-02:** Entity-Feld `SlotEntity.max_paid_employees: Option<u8>` (analog zu `min_resources: u8`, aber `Option` weil nullable).
- **D-03:** Geltungsbereich des Limits = pro **konkreter Wochen-Instanz** des wiederkehrenden Slot-Patterns. Es wird gegen die Bookings *einer* (year, week, slot)-Kombination gezählt.

### Capacity-Berechnung
- **D-04:** Gezählt werden alle aktiven Bookings (`bookings.deleted IS NULL`) im Slot dieser (year, week)-Kombination, deren Sales Person aktuell `is_paid = true` hat. Soft-deleted Sales Persons (`sales_person.deleted IS NOT NULL`) zählen nicht.
- **D-05:** Absence-Status der gebuchten Person ist **irrelevant** — wer eingetragen ist, zählt. (Ein bezahlter Mitarbeiter in Vacation, der trotzdem im Slot gebucht ist, wird mitgezählt.)
- **D-06:** Schwellwert-Semantik: Warning wird emittiert wenn `current_paid_count > max_paid_employees` (strikt größer). Bei `current == max` gibt es keine Warning — das Limit ist „erlaubtes Maximum".

### Backend-API
- **D-07:** Booking-Create-Pfad: nach erfolgreichem Insert über den **conflict-aware Endpoint** `POST /shiftplan-edit/booking` wird der neue Paid-Count gezählt; wenn `max_paid_employees` gesetzt UND `current > max`, wird Warning in `BookingCreateResult.warnings` angehängt. Buchung wird **nicht** zurückgerollt. Legacy `POST /booking` bleibt unverändert (kein Warning-Wrapper).
- **D-08:** Neuer Warning-Variant: `Warning::PaidEmployeeLimitExceeded { slot_id: Uuid, current_paid_count: u8, max_paid_employees: u8 }` (strukturiert, ohne Text — Übersetzung im Frontend). Erweitert das bestehende `service::warning::Warning` Enum (4 → 5 Varianten) plus den `WarningTO` in `rest-types`.
- **D-09:** Shiftplan-Week-View liefert pro Slot zusätzlich `max_paid_employees: Option<u8>` (Konfig) und `current_paid_count: u8` (berechnet, immer wenn das Read-DTO einen Slot enthält — keine extra Roundtrip-Optimierung, weil `sales_person.is_paid` per Booking schon im View-Aggregator vorliegt). Pattern-Analog: `build_shiftplan_day` im View-Service.
- **D-10:** Slot-Create/Update DTO bekommt Feld `max_paid_employees: Option<u8>` (utoipa-ToSchema annotated). OpenAPI-Snapshot-Test existiert nicht mehr (siehe Out-of-Scope) — keine Snapshot-Aktualisierung nötig.

### Berechtigungen
- **D-11:** Update von `slot.max_paid_employees` braucht Rolle **`shiftplanner`** (gleiche Permission wie aktueller Slot-Update-Pfad mit `min_resources`). Kein neues Privileg.

### Architektur
- **D-12:** Limit-Check (Booking-Pfad) lebt im **`ShiftplanEditService`** (Business-Logic-Tier), nicht im `BookingService`. Begründung: Cross-Entity-Logik (braucht Slot + Booking + Sales-Person `is_paid`) — die Service-Tier-Konvention aus `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen" verbietet das im Basic-Tier `BookingService`. `ShiftplanEditService` hat bereits `BookingService`- und `SalesPersonService`-Deps (etabliert in v1.0 Phase 3). Read-Aggregation für die Shiftplan-Week-View lebt analog im bestehenden Shiftplan-View-Service (Business-Logic-Tier). **Kein neuer Service nötig.**
- **D-13:** Warning-Pattern baut 1:1 auf v1.0 Phase 3 auf (`BookingCreateResult { booking, warnings }`). Neuer Variant erweitert das bestehende `Warning`-Enum. Kein neuer Wrapper, keine neue Surface.
- **D-14:** Snapshot-Versionierung **nicht** betroffen — kein Bump von `CURRENT_SNAPSHOT_SCHEMA_VERSION`. Begründung: kein `BillingPeriodValueType` wird hinzugefügt/geändert, keine Reporting-Computation berührt.
- **D-16:** **Endpoint-Scope:** Warning-Emission ausschließlich auf dem conflict-aware Pfad `POST /shiftplan-edit/booking`. Legacy `POST /booking` bleibt unverändert. Frontend migriert auf den conflict-aware Pfad (v1.0-Phase-4-Verantwortlichkeit, separater Workstream).

### Migration / Backfill
- **D-15:** Keine Backfill-Logik. Migration ist additiv (`ALTER TABLE slot ADD COLUMN max_paid_employees INTEGER`); alle bestehenden Rows haben implizit `NULL`. Keine Drift, kein Cutover-Gate, kein Versionierungs-Stamp.

### Claude's Discretion
- Exakte Methoden-Signatur des neuen Limit-Check-Helpers im `ShiftplanEditService` (z. B. private `count_paid_bookings_in_slot_week(slot_id, year, week, ctx, tx) -> Result<u8, ServiceError>`).
- Ob die Read-Aggregation den Paid-Count inline in `build_shiftplan_day` berechnet (per-Slot Iteration über schon geladene Bookings) oder einen separaten Batch-Query nutzt — Pattern-Mapper empfiehlt inline, da `is_paid` per Booking schon resolved ist.
- Ob `SalesPersonService::all_paid()` als Service-Wrapper neu eingeführt werden muss oder ob ein bestehender Lookup-Pfad ausreicht (DAO-Methode existiert).

</decisions>

<specifics>
## Specific Ideas

- **Pattern-Vorbild aus dem Codebase:**
  - `slot.min_resources` (Migration `20240813080347_add-column-min-resources.sql`) — gleiche Tabelle, gleicher konzeptioneller Slot-Capacity-Knob, aber NOT NULL/default 2. Wir spiegeln das Pattern, aber nullable.
  - `sales_person.is_paid` (Migration `20240618125847_paid-sales-persons.sql`) — bereits etabliertes Konzept; DAO hat sogar `all_paid()`-Methode. Kein neues Feld nötig.
  - `BookingCreateResult { booking, warnings }` aus v1.0 Phase 3 — exaktes Wrapper-Pattern, neuer Warning-Variant fügt sich nahtlos ein.

- **Naming-Wahl:** `max_paid_employees` (DB-Spalte + Entity-Feld + DTO-Feld), nicht `paid_employee_limit` oder `max_paid`. Begründung: konsistent mit `min_resources`-Pluralform, klar dass es um bezahlte Mitarbeiter geht.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Architektur & Konventionen
- `shifty-backend/CLAUDE.md` § "Service-Tier-Konventionen: Basic vs. Business-Logic Services" — Slot-DAO/Service liegt im Basic-Tier; BookingService ist Basic-Tier; Read-Aggregation für Shiftplan-View lebt im bereits existierenden Business-Logic-Tier-Service. Kein neuer Service nötig.
- `shifty-backend/CLAUDE.md` § "Billing Period Snapshot Schema Versioning" — Bestätigt: kein Bump für Phase 5 (keine `value_type`-Änderung, keine Reporting-Computation berührt).
- `shifty-backend/CLAUDE.md` § "Implementation Patterns" → `gen_service_impl!`-Macro für DI; `Option<Transaction>`-Pattern für alle Service-Methoden; `WHERE deleted IS NULL` in jeder DAO-Read-Query.
- `shifty-backend/CLAUDE.md` § "OpenAPI Documentation" — Pflicht: `#[utoipa::path]` an jedem REST-Handler; DTOs in `rest-types` mit `ToSchema`; OpenAPI-Snapshot-Test (`rest/tests/openapi_snapshot.rs`) wird aktualisiert.

### Vorbild-Migrationen / Pattern-Anker
- `migrations/sqlite/20240618125847_paid-sales-persons.sql` — `is_paid`-Konzept-Etablierung; SalesPerson-Schema-Erweiterung als Pattern-Vorbild für Slot-Erweiterung.
- `migrations/sqlite/20240813080347_add-column-min-resources.sql` — direkter Pattern-Vorbild für ALTER TABLE slot ADD COLUMN.
- `dao/src/slot.rs` — `SlotEntity` aktuelle Form, `SlotDao`-Trait für Erweiterung.
- `dao/src/sales_person.rs` — `SalesPersonEntity.is_paid`, `SalesPersonDao::all_paid()` als Lookup-Hilfe.

### Warning-Pattern (v1.0 Vorbild)
- `.planning/milestones/v1.0-ROADMAP.md` — v1.0 Phase 3 dokumentiert das `BookingCreateResult { booking, warnings }`-Wrapper-Pattern und die Forward/Reverse-Warning-Variant-Konventionen.
- `service/src/booking.rs` (oder das Modul, in dem `BookingCreateResult` lebt) — bestehende Warning-Enum als Erweiterungspunkt.

### Constraints In Force
- **VCS**: Repository wird mit `jj` (co-located mit git) verwaltet. Commits manuell durch User. GSD-Auto-Commit deaktiviert (`commit_docs: false`).
- **NixOS-Tooling**: `sqlx-cli` via `nix develop` (NICHT `nix-shell`). Migration testen: `sqlx migrate run` (additiv, nicht-destructive).
- **i18n**: Backend ist sprachneutral. Warning-Variants tragen nur strukturierte Daten — Übersetzung passiert im Frontend-Repo (en/de/cs).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `SalesPersonDao::all_paid(tx)` — direkter Lookup für „alle aktiven bezahlten Sales Persons", verwendbar wenn ein Bulk-Filter gebraucht wird.
- `SalesPersonEntity.is_paid: bool` — Per-Person-Flag, nutzbar in Joins/Filtern.
- `slot.min_resources` — Pattern-Vorbild für Slot-Capacity-Spalte (NOT NULL/default), unsere Variante ist die nullable Schwester.
- `BookingCreateResult { booking, warnings }` aus v1.0 Phase 3 — der gesuchte Wrapper für nicht-blockierende Warnings ist bereits etabliert.
- `gen_service_impl!`-Macro — für DI im neuen oder erweiterten Service.

### Established Patterns
- **Soft-Warning ohne Tx-Rollback**: v1.0 Phase 3 hat das Pattern eingeführt. Phase 5 erweitert es um einen neuen Variant — keine Architektur-Änderung.
- **`Option<Transaction>` in jeder Service-Methode** + `transaction_dao.use_transaction(tx)` + `commit(tx)` am Ende.
- **utoipa-Annotations** an jedem REST-Handler + `ToSchema`-Derive an jedem DTO + OpenAPI-Snapshot-Test.
- **Service-Tier-Konvention**: Basic-Tier (Slot/Booking) konsumiert nur DAO + Permission + Transaction. Read-Aggregation für Shiftplan-View lebt im bestehenden Business-Logic-Tier.

### Integration Points
- `dao/src/slot.rs` — `SlotEntity` + `SlotDao::create_slot` + `SlotDao::update_slot` + alle Read-Methoden.
- `dao_impl_sqlite/src/slot.rs` — SQLite-Reads/Writes; `TryFrom`-Implementations für Row → Entity.
- `service/src/slot.rs` + `service_impl/src/slot.rs` — Slot-Service-Surface; neue Field-Validation (`max_paid_employees: Option<u8>` ist immer valid, kein Range-Check nötig — `u8` reicht).
- `service/src/warning.rs` (oder wo `Warning`-Enum lebt) — neuer Variant `PaidEmployeeLimitExceeded`.
- `service_impl/src/shiftplan_edit.rs` — Limit-Check + Warning-Emission im `book_slot_with_conflict_check` (oder analoger Methode, vom Planner zu verifizieren). KEINE Änderung am `BookingService` selbst.
- Shiftplan-Week-View-Service (existierender Business-Logic-Service, exakter Modul-Name vom Planner zu identifizieren) — Read-Aggregation pro Slot in `build_shiftplan_day`.
- `rest-types/src/lib.rs` — Slot-DTOs erweitern (`max_paid_employees: Option<u8>`), `WarningTO` um neuen Variant erweitern, Shiftplan-Week-View Slot-DTO um `current_paid_count: u8` und `max_paid_employees: Option<u8>` erweitern.
- `rest/src/slot.rs` (oder analoger Pfad) — utoipa-Annotations für Slot-Update bleiben unverändert (Feld kommt durchs DTO).
- `migrations/sqlite/` — neue Migration `YYYYMMDDHHMMSS_add-max-paid-employees-to-slot.sql`. Spalten-Definition: `ALTER TABLE slot ADD COLUMN max_paid_employees INTEGER` (kein `DEFAULT`, kein `NOT NULL`).

</code_context>

<deferred>
## Deferred Ideas

- **Frontend-Implementation (shifty-dioxus)**: UI für Limit-Konfiguration im Slot-Edit-Dialog, visuelle Indikatoren im Shiftplan-Grid, Toast-Anzeige für Booking-Warnings, i18n-Strings (en/de/cs). Separater Workstream im Frontend-Repo, vermutlich unmittelbar nach Phase-5-Backend-Ship.
- **Min-Paid-Capacity / Soll-Besetzung**: analoges Feld `min_paid_employees` für Untergrenze. Wenn gewünscht — Folge-Phase v1.x.
- **Skill-Matching / weitere Slot-Constraints**: andere Capacity-Dimensionen (z. B. Mindest-Anzahl mit bestimmter Qualifikation). Reserviert für spätere v1.x-Phasen.
- **HR-Admin-Surface**: Übersicht aller Slots, deren Limit aktuell überschritten ist (Reporting-View). Nicht Teil von Phase 5.
- **Hartes Reject-Pattern**: falls später eine Variante mit blockierendem Limit gebraucht wird (z. B. „max harte Anzahl"), wird das als separates Feld in einer Folge-Phase modelliert. Phase 5 ist explizit Soft-Warning.

</deferred>

---

*Phase: 05-slot-paid-capacity-warning*
*Context gathered: 2026-05-03*
