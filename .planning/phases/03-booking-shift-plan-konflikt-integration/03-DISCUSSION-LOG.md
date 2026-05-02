# Phase 3: Booking & Shift-Plan Konflikt-Integration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-02
**Phase:** 3-booking-shift-plan-konflikt-integration
**Areas discussed:** A (Warning API-Migration), B (Cross-Source-Lookup), C (Shift-Plan-Surface), D (Warning-Modell + Doppel-Markierung)

---

## Area A — Warning API-Migration (BookingService/AbsenceService)

### A1: Wie wandeln wir BookingService::create und AbsenceService::create um, damit sie Warnings zurückgeben können?

| Option | Description | Selected |
|--------|-------------|----------|
| Signatur-Bruch (Recommended) | Beide Methoden ändern zu Result<XxxCreateResult, ServiceError> mit struct XxxCreateResult { entity, warnings: Arc<[Warning]> }. Saubere Code, alle Aufrufer müssen die Warnings sehen (Compiler erzwingt es), keine zwei parallelen Pfade. | ✓ |
| Additive parallele Methode | create() bleibt wie es ist, neue Methode create_with_warnings(). Kein Bruch für Bestand, REST kann beide Endpoints anbieten. Risiko: stille Datenverluste. | |
| Hybrid: Bruch im Service, additive REST-Variante | Service-Trait ändert, REST exponiert beide. Kompromiss intern sauber, externe API nicht-bruchend. | |

**User's choice:** Signatur-Bruch. Begründung: Compiler-Druck ist gewünscht; alle Test-Sites + REST-Handler + copy_week + integration-tests werden mechanisch mit-migriert. Frontend-Workstream ist separat und wird sowieso später nachziehen.

---

### A2: Wie behandelt BookingService::copy_week (Loop über create) die neuen Warnings?

| Option | Description | Selected |
|--------|-------------|----------|
| Warnings aggregieren und zurückgeben (Recommended) | copy_week ändert sich auf Result<CopyWeekResult, _> mit copied_bookings + warnings Arc<[Warning]>. Konsistent mit BookingCreateResult. | ✓ |
| Warnings swallowen / Logging | copy_week bleibt Result<(), _>; Warnings werden nur geloggt. Konflikte unsichtbar. | |
| Erste Warning bricht ab | copy_week stoppt beim ersten Konflikt. Konservativ aber bricht idempotente Bulk-Operationen. | |

**User's choice:** Warnings aggregieren. Konsistent mit dem Wrapper-Pattern.

---

### A3: Wie expone ich die Warnings auf der REST-Schicht?

| Option | Description | Selected |
|--------|-------------|----------|
| Wrapper-DTO im Response-Body (Recommended) | POST /booking und POST /absence-period geben jetzt BookingCreateResultTO / AbsencePeriodCreateResultTO zurück — immer 200/201 (auch mit Warnings). Frontend prüft warnings.is_empty(). utoipa-Schema klar. | ✓ |
| Warnings im HTTP-Header (X-Shifty-Warnings) | Body bleibt unverändert; Warnings in Custom-Header. Nicht im OpenAPI-Schema sichtbar, nicht idiomatisch. | |
| Status 200 + Warnings, Status 201 ohne | Konvention: 201 = clean, 200 = mit Warnings. Etwas missbrauch-artig. | |

**User's choice:** Wrapper-DTO im Response-Body. utoipa zeigt den Bruch direkt; OpenAPI-Snapshot dokumentiert die API-Veränderung.

---

### A4: AbsenceService::update kann beim Range-Erweitern neue Booking-Konflikte erzeugen. Soll update auch den Warning-Wrapper liefern?

| Option | Description | Selected |
|--------|-------------|----------|
| Ja, update gibt auch AbsencePeriodCreateResult zurück (Recommended) | Symmetrisch zu create. Warnings werden für ALLE Tage in der NEUEN Range berechnet. Einfaches Mental-Model. | ✓ |
| Nur für neue Tage Warnings (Diff-Modus) | Update vergleicht alte vs. neue Range; gibt nur Warnings für neu hinzugekommene Tage. Diff-Logik komplex und fehlerträchtig. | |
| Update bleibt warning-frei, nur create warnt | Verlust: HR könnte beim Verlängern einer Vacation einen neu betroffenen Booking-Tag übersehen. | |

**User's choice:** Symmetrisch zu create, volle Warnings über die neue Range.

---

## Area B — Cross-Source-Unavailability Lookup

### B1: Wo lebt der Cross-Kategorie-Lookup für 'gibt es eine aktive AbsencePeriod im Range für diesen Mitarbeiter?'

| Option | Description | Selected |
|--------|-------------|----------|
| Neue DAO-Methode mit IN-Clause (Recommended) | AbsenceDao::find_overlapping_for_booking. Eine SQL-Query, single roundtrip, nutzt composite index. | ✓ |
| Service-Helper, der bestehendes find_overlapping 3x ruft | 3 Roundtrips, 3 Queries. | |
| Beide — DAO bekommt neue Methode + Service-Wrapper | Doppel-Surface, mehr Komplexität. | |

**User's choice:** Neue DAO-Methode mit IN-Clause. Single roundtrip; nutzt bestehenden Index aus Phase 1 D-04.

---

### B2: Wo werden AbsencePeriod- und sales_person_unavailable-Quellen für die Reverse-Warning kombiniert?

| Option | Description | Selected |
|--------|-------------|----------|
| Im BookingService::create direkt (Recommended) | BookingService bekommt beide Dependencies; ruft beide separat. Architektur-Direction bleibt erhalten. | ✓ |
| Neuer UnavailabilityService als Aggregator | Neuer Service, neue DI, neue Indirektion. Overkill für 2 Quellen. | |
| AbsenceService aggregiert sales_person_unavailable mit | Bricht Architektur-Direction (AbsenceService kennt jetzt sales_person_unavailable). | |

**User's choice:** BookingService kombiniert direkt.

---

### B3 (Premise-Klärung): Soll BookingService::create wirklich die Reverse-Warning machen?

User hatte zunächst über "Other"-Antwort die Premise hinterfragt: "Was hat booking damit zu tun? Muss da was geändert werden?"

| Option | Description | Selected |
|--------|-------------|----------|
| Ja, BookingService::create gibt die Warning zurück (per SC2) | Wie ursprünglich gefragt. SC2 verlangt explizit den Booking-Service. | ✓ |
| Nein, Reverse-Warning passiert nur passiv beim Shift-Plan-Anzeigen | BookingService bleibt unverändert; SC2 müsste umformuliert werden. | |
| Anders | User erklärt das Modell selbst. | |

**User's choice:** Ja — Bestätigung der ursprünglichen SC2-Lesart.

**Notes:** SC2 sagt "Beim Anlegen eines Bookings ... gibt der Booking-Service eine Warnung zurück". Heute (`service_impl/src/booking.rs:181-303`) gibt es keinen Unavailability-Check; Phase 3 baut den ein.

---

### B4: Wie konvertiert BookingService::create den Buchungstag in eine Date für den AbsencePeriod-Lookup?

| Option | Description | Selected |
|--------|-------------|----------|
| Inline per time::Date::from_iso_week_date (Recommended) | BookingService::create lädt Slot ohnehin; Konversion direkt im create()-Body. Pattern existiert in shiftplan.rs:138. | ✓ |
| Slot-Service liefert die Date | Neue Methode SlotService::resolve_date; semantischer aber neue API für nur diesen Use-Case. | |
| Helper im shifty-utils-Crate | Wrapper um time::Date::from_iso_week_date. Wiederverwendbar aber Overhead. | |

**User's choice:** Inline-Konversion. Slot wird ohnehin geladen (Zeile 260-263); keine neue Surface.

---

## Area C — Shift-Plan-Markierungs-Surface (PLAN-01)

### C1: Wo lebt die Shift-Plan-Markierung von Absence-Tagen für 'einen Mitarbeiter über einen Zeitraum'?

| Option | Description | Selected |
|--------|-------------|----------|
| Erweitere ShiftplanViewService um per-sales-person-Variante (Recommended) | Neue Methode get_shiftplan_week_for_sales_person mit per-Tag-Markierung. ShiftplanDay bekommt Feld unavailable. Additiv. | ✓ |
| Erweitere BookingInformationService::get_booking_conflicts_for_week | Liefert nur Konflikte mit Bookings, nicht 'leere Absence-Tage'. PLAN-01 verlangt aber alle Tage zu markieren. | |
| Neuer eigener UnavailabilityViewService | Sauberer aus Architektur-Sicht aber mehr Surface, mehr REST, mehr DI. | |

**User's choice:** Erweitere ShiftplanViewService. Bestehende globale Methoden bleiben unverändert.

---

### C2: Wie wird die Markierung in der ShiftplanWeek-Struktur exponiert?

| Option | Description | Selected |
|--------|-------------|----------|
| Neues Feld am ShiftplanDay (Recommended) | ShiftplanDay.unavailable: Option<UnavailabilityMarker> mit Varianten AbsencePeriod/ManualUnavailable/Both. Slots bleiben sichtbar (man sieht WAS verloren geht). Additiv. | ✓ |
| Separater Per-Tag-Map zurückgeben | Trennt Wochen-Sicht von Markierungen. Komplexer im REST-DTO. | |
| Markierung am ShiftplanBooking | Nur Bookings auf Unavailable-Tagen tragen Flag. Widerspricht 'alle Tage markieren'. | |

**User's choice:** Neues Feld am ShiftplanDay.

---

### C3: Permission und REST-Surface für get_shiftplan_week_for_sales_person?

| Option | Description | Selected |
|--------|-------------|----------|
| HR ∪ self, neuer Endpoint /shiftplan/{id}/year/{y}/week/{w}/sales-person/{sp} (Recommended) | Eigener REST-Endpoint, eigene Service-Methode. Permission HR ∨ verify_user_is_sales_person. | ✓ |
| Selbe Methode, sales_person_id als Query-Parameter | Spart Endpoint, aber Response-Shape conditional. | |
| Self-Only — nur der eigene Mitarbeiter | HR sieht das nicht (müsste impersonate). | |

**User's choice:** HR ∪ self mit neuem Endpoint.

---

### C4: Welchen Zeitraum nimmt get_shiftplan_*_for_sales_person als Parameter?

| Option | Description | Selected |
|--------|-------------|----------|
| Eine Woche (Recommended, konsistent mit Bestand) | year + calendar_week. Frontend loopt mehrere Wochen. | ✓ |
| Expliziter DateRange | Mehrere Wochen in einem Roundtrip. Bricht Per-Woche-Pattern. | |
| Beide Methoden parallel | Maximale Flexibilität, doppelte API-Surface. | |

**User's choice:** Per-Woche, konsistent mit existierendem get_shiftplan_week.

---

## Area D — Warning-Datenmodell + Doppel-Markierungs-Verhalten

### D1: Welche Form hat Warning?

| Option | Description | Selected |
|--------|-------------|----------|
| Enum mit klaren Varianten (Recommended) | enum Warning { BookingOnAbsenceDay {...}, BookingOnUnavailableDay {...}, AbsenceOverlapsBooking {...}, AbsenceOverlapsManualUnavailable {...} }. Compiler-Druck; Type-safe. | ✓ |
| Generischer Struct mit reason-String | reason: Arc<str> nicht type-safe; i18n schwierig. | |
| Pro Use-Case eigene Result-Types | Duplikation: Booking-Konflikt taucht doppelt auf. | |

**User's choice:** Enum mit klaren Varianten.

---

### D2: Bei einer Mehrtages-Absence über mehrere Bookings: wie wird die Warning-Liste strukturiert?

| Option | Description | Selected |
|--------|-------------|----------|
| Eine Warning pro betroffenem Booking-Tag (Recommended) | Vec<Warning> mit BookingOnAbsenceDay-Variante PRO konfliktiertem Booking. Symmetrisch mit Reverse. | ✓ |
| Eine aggregierte Warning mit Vec<BookingRef> | Schlüssel-Statement aber asymmetrisch zur Reverse-Richtung. | |
| Eine Warning pro Tag | Bei 14 Tagen Absence ohne Bookings = 14 leere Warnings. Noisy. | |

**User's choice:** Eine Warning pro betroffenem Booking-Tag.

---

### D3: Was passiert beim Anlegen einer AbsencePeriod, die bestehende sales_person_unavailable-Einträge überdeckt?

| Option | Description | Selected |
|--------|-------------|----------|
| Warning mit Variant AbsenceOverlapsManualUnavailable, kein Auto-Cleanup (Recommended) | Warning informiert; User entscheidet selbst. Symmetrisch mit Booking-Konflikt-Verhalten. | ✓ |
| Auto-Cleanup: überlappende sales_person_unavailable werden in derselben Tx soft-deleted | Irreversibel; bricht Architektur-Direction; bricht Phase-4-Re-Run-Idempotenz. | |
| Ignorieren — keine Warning, kein Cleanup | User merkt erst beim Anschauen. | |

**User's choice:** Warning, kein Auto-Cleanup.

---

### D4: Bekommt SalesPersonUnavailableService::create auch eine Warning, wenn der Tag bereits durch eine AbsencePeriod abgedeckt ist?

| Option | Description | Selected |
|--------|-------------|----------|
| Nein, sales_person_unavailable.create bleibt unverändert (Recommended) | Phase 3 fokussiert auf BOOK-01/BOOK-02/PLAN-01. Halt die Phase fokussiert. | |
| Ja, symmetrisch — Warning bei Konflikt | Konsequent symmetrisch aber weiterer Service-Bruch, weiterer REST-Wrapper. Scope creep. | |
| Ja, aber als Deferred für eine Folge-Phase | Phase 3 baut die Infrastruktur, aber SalesPersonUnavailableService selbst wird später erweitert. | ✓ |

**User's choice:** Deferred für Folge-Phase. Phase 3 fügt das Pendant-Warning-Variant `ManualUnavailableOnAbsenceDay` NICHT in die Warning-Enum ein (Vermeidung dead enum-arms); die Folgephase wird sowohl Variante als auch Wrapper-Bruch zusammen einführen.

---

## Claude's Discretion (Plan-Phase entscheidet)

Siehe `03-CONTEXT.md` § Decisions § "Claude's Discretion" für die vollständige Liste:

- Modul-Lokation für Warning-Enum + Wrapper-Structs.
- Range-Lookup-API (Loop vs. neue get_for_range-Methode in BookingService).
- build_shiftplan_day_for_sales_person Layout (neuer Helper vs. Optional-Parameter).
- Warning-zu-WarningTO-Conversion (Tag-und-Daten-Schema).
- Test-Fixture-Lokation für Cross-Source-Test.
- Performance-Caching für copy_week (erst messen).
- Kategorie-Trigger-Differenzierung (Default: alle 3 gleich).

## Deferred Ideas

Siehe `03-CONTEXT.md` § Deferred Ideas für die vollständige Liste. Wichtigste Punkte:

- SalesPersonUnavailableService::create Symmetrisierung — Folgephase.
- Auto-Cleanup von überlappenden manual unavailables — bewusst NICHT in Phase 3.
- Multi-Wochen-Range-Variante für ShiftplanView — Future-Phase.
- Performance-Optimierungen (copy_week pre-fetch, BookingService::get_for_range) — erst messen.
- Frontend-Backward-Compat — separater Workstream.
