# Phase 1: Absence Domain Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-01
**Phase:** 1-Absence Domain Foundation
**Areas discussed:** Naming & Schema-Surface, Update-Semantik & logical_id, Permission-Modell ABS-05, Self-Overlap & DateRange-Utility

---

## Naming & Schema-Surface

### Q1: Welcher Name für die neue Entity / das Modul / die Tabelle?

| Option | Description | Selected |
|--------|-------------|----------|
| AbsencePeriod (Recommended) | Steht so in REQUIREMENTS.md ABS-01 und Research. Modul `absence`, Struct `AbsencePeriod`, Tabelle `absence_period`, REST `/absence-period`. | ✓ |
| Absence (kompakter) | Modul `absence`, Struct `Absence`, Tabelle `absence`, REST `/absence`. Kompakter; das 'Period' liegt implizit. Riskiert Namens-Kollision in v2. | |
| TimeOff | Englisch-idiomatischer; Modul `time_off`. Bricht aus dem bestehenden Domain-Vokabular. | |

**User's choice:** AbsencePeriod (Recommended)

### Q2: Welche Kategorien werden für `AbsencePeriod` in Phase 1 unterstützt?

| Option | Description | Selected |
|--------|-------------|----------|
| 3 Kategorien wie ABS-01 (Recommended) | Vacation / SickLeave / UnpaidLeave — deckt sich mit REQUIREMENTS.md ABS-01 und PROJECT.md Out-of-Scope. v2 erweitert via CAT-01..03. | ✓ |
| 5 Kategorien (zusätzlich Holiday + Unavailable) | PROJECT.md schliesst diese explizit als Out-of-Scope aus. | |
| 6 Kategorien (3 + Holiday + Unavailable + VolunteerWork) | Research-Empfehlung; massiv grösserer Scope; widerspricht PROJECT.md. | |

**User's choice:** 3 Kategorien wie ABS-01 (Recommended)

### Q3: Wo lebt die `AbsenceCategory`?

| Option | Description | Selected |
|--------|-------------|----------|
| Eigene `AbsenceCategory` (Recommended) | Neue, kleine Enum mit nur Vacation/SickLeave/UnpaidLeave in `service/src/absence.rs`. Saubere Domain-Trennung, Compiler-Hilfe. | ✓ |
| Reuse `ExtraHoursCategory` | 1:1-Mapping mit ExtraHours-Domain, aber Laufzeit-Validierung nötig und vermischt Domains. | |

**User's choice:** Eigene `AbsenceCategory` (Recommended)

### Q4: Welche Schema-Constraints und Indexe?

| Option | Description | Selected |
|--------|-------------|----------|
| Vollpaket per Research (Recommended) | CHECK(to_date >= from_date), partial unique on (logical_id) WHERE deleted IS NULL, composite index on (sales_person_id, from_date) WHERE deleted IS NULL, NOT NULL auf Pflichtfeldern. | ✓ |
| Nur Pflicht-Constraints, Indexe später | Spart kurzzeitig Schema-Fläche, bringt später Migration-Folge-Aufwand. | |
| Du entscheidest | Plan-Phase darf entscheiden, default Vollpaket. | |

**User's choice:** Vollpaket per Research (Recommended)

**Notes:** Naming, Categories, Enum-Source und Schema-Constraints sind klar genug; weiter zum nächsten Bereich.

---

## Update-Semantik & logical_id

### Q5: Welche Felder sind auf einem `AbsencePeriod` per Update/PATCH änderbar?

| Option | Description | Selected |
|--------|-------------|----------|
| from_date, to_date, description (Recommended) | Range und Beschreibung änderbar; Kategorie nicht. Match mit User-Erwartung. | |
| from_date, to_date, description, category | Maximaler Mutations-Scope inkl. Kategorie-Umwidmung. Mehr Tests, grösserer Validations-Surface. | ✓ |
| Nur description | Sehr restriktiv; jede Range-Änderung verlangt delete + create. | |

**User's choice:** from_date, to_date, description, category

**Notes:** User wählt maximalen Mutations-Scope. Self-Overlap-Detection beim Update muss auf die neue Kategorie angewendet werden.

### Q6: logical_id und Optimistic Locking analog ExtraHours?

| Option | Description | Selected |
|--------|-------------|----------|
| 1:1 ExtraHours-Pattern (Recommended) | Update soft-deleted alte Row, schreibt neue Row mit selber logical_id, neuer physical id, neuer version. Version-Check bei Drift. | ✓ |
| In-Place Update (kein logical_id) | Einfache UPDATE-Statements ohne tombstone/version. Bricht aus dem Codebase-Pattern. | |
| Du entscheidest | Plan-Phase wählt zwischen 1:1-Pattern und Variante; default 1:1. | |

**User's choice:** 1:1 ExtraHours-Pattern (Recommended)

### Q7: Booking-Konflikt-Detection in Phase 1 — schon mitnehmen oder strikt auf Phase 3 verschieben?

| Option | Description | Selected |
|--------|-------------|----------|
| Strikt Phase 3 (Recommended) | Phase 1 bleibt klar additiv: kein Touch an `BookingService`, kein Warning-Return-Type. | ✓ |
| Return-Type-Stub schon in Phase 1 | Bereits `AbsencePeriodCreateResult { absence, warnings }` einführen; vermeidet API-Bruch in Phase 3. | |
| Komplette Konflikt-Detection in Phase 1 | Verletzt Phase-Trennung, macht Phase 3 überflüssig. | |

**User's choice:** Strikt Phase 3 (Recommended)

---

## Permission-Modell ABS-05

### Q8: Wie wird für WRITE-Operationen die Erlaubnis 'HR oder Mitarbeiter selbst' technisch verifiziert?

| Option | Description | Selected |
|--------|-------------|----------|
| 1:1 ExtraHours-Pattern (Recommended) | `tokio::join!(HR_PRIVILEGE check, verify_user_is_sales_person)` + `.or()`. Bestehende Privileg-Konstanten. | ✓ |
| Nur HR_PRIVILEGE | Mitarbeiter können eigene Abwesenheiten nicht selbst eintragen — widerspricht ABS-05. | |
| Eigenes neues Privileg `ABSENCE_WRITE` | Flexibler, aber grösserer Surface ohne klaren Mehrwert. | |

**User's choice:** 1:1 ExtraHours-Pattern (Recommended)

### Q9: Welche Read-Sicht hat ein Mitarbeiter ohne HR-Rechte?

| Option | Description | Selected |
|--------|-------------|----------|
| Eigene + Schichtplan-Kollegen (Recommended) | HR sieht alle. Mitarbeiter sieht eigene plus die der Schichtplan-Kollegen (analog Booking-Sicht). | ✓ |
| Strikt nur eigene | Maximaler Datenschutz, macht Phase 3 (Shift-Plan-Integration) komplizierter. | |
| HR + alle Sales sehen alles | Einfach zu implementieren; minimaler Datenschutz. | |
| Du entscheidest | Plan-Phase darf entscheiden. | |

**User's choice:** Eigene + Schichtplan-Kollegen (Recommended)

**Notes:** Detail-Ausarbeitung in Plan-Phase basierend auf existierenden Booking-Read-Konventionen.

---

## Self-Overlap & DateRange-Utility

### Q10: Self-Overlap-Detection: per Kategorie oder cross-Kategorie?

| Option | Description | Selected |
|--------|-------------|----------|
| Per (sales_person, kategorie) wie ABS-03 (Recommended) | Vacation und SickLeave dürfen koexistieren (BUrlG §9). Cross-Category-Auflösung ist Phase-2-Reporting-Thema. | ✓ |
| Cross-Kategorie verbieten | Verhindert SickLeave über Vacation, was BUrlG §9 widerspricht. | |
| Cross-Kategorie nur warnen, nicht blockieren | Phase 1 hat keinen Warning-Channel. | |

**User's choice:** Per (sales_person, kategorie) wie ABS-03 (Recommended)

### Q11: Welche `ServiceError`-Variante für Self-Overlap?

| Option | Description | Selected |
|--------|-------------|----------|
| `OverlappingTimeRange` (Recommended) | Existiert bereits; konsistent; HTTP-Mapping etabliert. | |
| Neue Variante `AbsencePeriodOverlap(Uuid, Uuid)` | Praziser, bringt aber neue ServiceError-Variante. | |
| `ValidationError(Arc<[ValidationFailureItem]>)` | Generischer Validation-Channel mit Duplicate o.ä. | ✓ |

**User's choice:** `ValidationError(Arc<[ValidationFailureItem]>)`

**Notes:** Plan-Phase erweitert ggf. `ValidationFailureItem` um eine Overlap-Variante (z.B. `OverlappingPeriod(Uuid)`).

### Q12: Wo lebt die `DateRange`-Utility und wann wird sie eingeführt?

| Option | Description | Selected |
|--------|-------------|----------|
| In Phase 1 in `shifty-utils` (Recommended) | `shifty_utils::DateRange` mit `overlaps()`, `iter_days()`, `day_count()`, `contains()`. Inclusive bounds. | ✓ |
| Erst in Phase 2 einführen | Spart Phase-1-Surface, bedingt aber Code-Dopplung und Refactor. | |
| Lokal in `service`-Crate statt `shifty-utils` | Trennt Utility von Concept, blockiert spätere Wiederverwendung. | |

**User's choice:** In Phase 1 in `shifty-utils` (Recommended)

### Q13: Signatur von `AbsenceDao::find_overlapping`?

| Option | Description | Selected |
|--------|-------------|----------|
| (sales_person_id, category, range) (Recommended) | Direktes Tooling für Self-Overlap; Phase-3-Booking-Integration kann separate Funktion ergänzen. | |
| (sales_person_id, range) only | Cross-kategorie Lookup als Standard; Service muss client-seitig filtern. | |
| Du entscheidest | Plan-Phase darf basierend auf SQL-Index-Optimierung entscheiden. | ✓ |

**User's choice:** Du entscheidest

**Notes:** Hinweis: kategorie-scoped als Standard, separate Funktion für Phase-3-Booking-Lookup wenn nötig.

---

## Meta-Frage des Users vor CONTEXT.md

User fragte, ob Nachteile entstehen, wenn jetzt zu CONTEXT.md gewechselt wird, obwohl ich noch "offene Themen" erwähnt hatte.

Claude antwortete: Die genannten "offenen Themen" (REST-Filter-Query-Params, OpenAPI-Schema-Details, DI-Reihenfolge, description-Pflichtigkeit) sind taktische Details, die die Plan-Phase aus existierenden Patterns (booking.rs, extra_hours.rs) ableiten kann. Keine harten Nachteile. User bestätigte, mit CONTEXT.md weiterzumachen.

---

## Claude's Discretion

- **C-01:** `AbsenceDao::find_overlapping`-Signatur (kategorie-scoped vs. cross-kategorie) — Plan-Phase basierend auf SQL-Index-Optimierung. Vorgabe: Self-Overlap-Detection ist kategorie-scoped (D-12).
- **C-02:** REST-Filter-Query-Params für `GET /absence-period`-Liste — Pattern-Übernahme aus `rest/src/booking.rs` und `rest/src/extra_hours.rs`.
- **C-03:** `description`-Pflichtigkeit (`Arc<str>` mit leerem Default vs. echtes `Option`) — Plan-Phase anhand UX-Argument.
- **C-04:** OpenAPI-Annotationen — Standard `#[utoipa::path]`-Pattern.
- **C-05:** DI-Reihenfolge in `main.rs` — mechanische Erweiterung um `AbsenceServiceDependencies`-Block.

## Deferred Ideas

- Holiday/Unavailable/VolunteerWork als Range — v2 (CAT-01..03)
- Approval-Workflow — v2 (APRV-01)
- Halbtage / Stundengenaue Granularität — v2 (GRAN-01/02)
- Self-Service-Antrag mit Status-Tracking — v2
- Booking-Konflikt-Wrapper-Type — Phase 3 (BOOK-01)
- `find_overlapping_for_booking` (cross-kategorie) — Phase 3
- Reverse-Booking-Warning aus AbsencePeriod-Quelle — Phase 3 (BOOK-02)
- Reporting-Integration / `derive_hours_for_range` — Phase 2
- Frontend (Dioxus) — separater Workstream
