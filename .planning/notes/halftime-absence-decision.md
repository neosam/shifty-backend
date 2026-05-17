---
title: Halbtag-Abwesenheiten — Decision Log
date: 2026-05-17
context: explore session — Revision der v1.3-Out-of-Scope-Entscheidung "Halbtage / Stundenebene für Abwesenheiten" nach konkretem Cutover-Blocker (Heiligabend + Silvester nicht abbildbar)
---

# Halbtag-Abwesenheiten — Decision Log

## Trigger

Beim Vorbereiten des `extra_hours → absence_period`-Cutovers (Phase 8.1) ist
aufgefallen, dass bestehende **halbe Urlaubstage** an Heiligabend (24.12.) und
Silvester (31.12.) nicht ins neue Range-basierte `AbsencePeriod`-Modell überführt
werden können — das Backend modelliert nur Ganztage. Das war in `REQUIREMENTS.md`
+ `PROJECT.md` als Out-of-Scope notiert ("Backend modelliert nur Ganztage;
Halbtag-Modell wäre Backend-Änderung"). Diese Entscheidung wird revidiert.

## Decisions

### D-01 — Halbtag ist Eigenschaft der **Absence**, nicht des **Tages**

Variante B aus der Diskussion. Ein Halbtag drückt aus, dass eine **bestimmte
Buchung** nur einen halben Vacation-Day vom Kontingent verbraucht. Das ist
nicht dasselbe wie ein "halber Arbeitstag" als `SpecialDay`-Eigenschaft
(Variante A), die die Soll-Arbeitszeit reduzieren würde.

**Begründung:** Heiligabend/Silvester sind im Betrieb **volle Arbeitstage**,
von denen 0,5 Vacation-Days verbraucht und die anderen 4 h gearbeitet (bzw.
als geschenkte Freistellung gewertet) wurden. Das ist eine Eigenschaft der
**Vacation-Buchung**, nicht des Tages.

### D-02 — Granularität: nur `Full | Half`, kein AM/PM, keine Stundenebene

`day_fraction` ist ein zweiwertiges Konzept (ganz oder halb). Es gibt keine
Unterscheidung Vormittag/Nachmittag und kein generelles Stundenmodell.

**Begründung:**
- Stundenebene wäre Over-Engineering für ein Problem, das in der Praxis hauptsächlich
  Heiligabend + Silvester betrifft, und würde komplette Service-Schicht,
  Reporting-Arithmetik und Frontend-UI umbauen.
- AM/PM bringt Konfliktauflösung mit Bookings am selben Tag, ist aber für den
  konkreten Anwendungsfall (Heiligabend/Silvester) irrelevant, weil die Tage
  ohnehin schichtfrei sind. Kann später ergänzt werden, wenn Bedarf entsteht.

### D-03 — Implementierung **vor** finalem Cutover-Switch

Eigene neue Phase **8.3** als Vorlauf vor dem finalen Cutover-Switch (Plan
08.1-12 — Phase-8-HUMAN-UAT-Subsumption). Die Phase erweitert:
- Backend-Datenmodell (`AbsencePeriod` + `absence_period`-Tabelle).
- Service + REST + DTOs.
- Frontend-CRUD (Phase 8 Page) um Halbtag-Eingabe.
- Cutover-Migration-UI (Phase 8.1 — bereits 11/12 fertig) um Halbtag-Mapping
  pro Migrationseintrag.

**Begründung:** Datenkorrektur auf bereits gecutoverten Live-Daten ("Vacation
versehentlich als 1,0 statt 0,5 verbucht") ist deutlich schmerzhafter als ein
verzögerter Cutover-Switch. Der Cutover ist ein Einmalereignis — eine Phase
Verzögerung kostet weniger als nachträgliche Datenarchäologie für alle
betroffenen Mitarbeiter.

### D-04 — Cutover-Migration-UI muss Halbtag-Einstellung haben

Die in Phase 8.1 bereits gebaute Drift-Resolution-Liste (Per-Eintrag-Aktionen
Convert / Edit / Delete / Skip) muss um eine Halb/Ganz-Auswahl pro Eintrag
erweitert werden. Vorgegeben durch User: "Das sollte ich dann in der
Datenmigration UI auch einstellen können."

**Offen für Plan-Phase 8.3:**
- Automatische Halbtag-Erkennung anhand Alt-Daten-Stunden (z. B. 4 h statt
  8 h → Halbtag-Vorschlag) **vs.** rein manuelle Toggle pro Eintrag.
- Erweiterung der bereits gebauten `convert_quarantine_entry`- und
  `bulk_convert_quarantine_rows`-Endpoints (Plan 08.1-02 / 08.1-03) und des
  `ManualConvertModal` aus Phase 8.2 (Plan 08.2-02).

## Revision

Der Out-of-Scope-Eintrag in `REQUIREMENTS.md:96` und der entsprechende
Bullet-Point in `PROJECT.md` (`Halbtage / Stundenebene für Abwesenheiten —
Backend modelliert nur Ganztage`) ist **hinfällig**. Cleanup-Schritt erfasst als
`.planning/todos/pending/2026-05-17-revise-halftime-scope.md`.

## Offene Punkte für die Plan-Phase

- **Datenmodell-Form:** Enum `DayFraction { Full, Half }` (Rust-idiomatisch,
  Type-Safe) vs. `day_fraction: f32` (flexibel für später, aber sloppy).
  Empfehlung: Enum + DB-Spalte als `TEXT` mit Check-Constraint, oder `INTEGER`
  (0=Full, 1=Half).
- **Migration bestehender `absence_period`-Daten:** Default-Wert `Full` für
  alle bestehenden Einträge. Spalte `NOT NULL DEFAULT 'full'`.
- **Reporting-Auswirkungen:** `derive_hours_for_range` muss `day_fraction`
  berücksichtigen — bei `Half` halbe Arbeitsstunden anrechnen.
  `BillingPeriodValueType` für Vacation-Aggregate: ob ein neuer `value_type`
  nötig ist oder die bestehenden Werte über `day_fraction` korrigiert werden.
  **Achtung Snapshot-Schema-Versioning:** Da die Vacation-Aggregation-Logik sich
  ändert, muss `CURRENT_SNAPSHOT_SCHEMA_VERSION` gebumpt werden (siehe
  `shifty-backend/CLAUDE.md` § "Billing Period Snapshot Schema Versioning").
- **Frontend-UI:** Checkbox "halber Tag" im Absence-Modal vs. Dropdown
  `Full | Half`. Bei Range > 1 Tag: gilt `day_fraction` für alle Tage der
  Range, oder nur für Start/Ende?
- **Konflikte mit Bookings:** Halbtag-Absence + Booking am selben Tag wird
  als zulässig betrachtet (kein Konflikt-Warning), oder weiter geprüft?
- **i18n:** De / En / Cs für neue Labels ("Halber Tag", "Half day", "Půldenní
  dovolená" o. ä.).

## Verweis

- Requirement: `REQUIREMENTS.md` — **FUI-A-10**
- Phase: `ROADMAP.md` — **Phase 8.3: Halbtag-Support für Absences**
- Todo (Cleanup): `.planning/todos/pending/2026-05-17-revise-halftime-scope.md`
