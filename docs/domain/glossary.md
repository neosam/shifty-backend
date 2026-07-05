# Glossar — Domain-Begriffe in Shifty

Dieses Glossar ist die Wahrheitsquelle für Begriffe. Wenn zwei
Dokumente denselben Begriff unterschiedlich verwenden, hat dieses hier
Recht — oder es ist ein Widerspruch, den es aufzulösen gilt.

## A

**Absence.** Range-basierte Abwesenheit (v1.0+), z.B. Urlaub, Krank,
Unbezahlt. Ersetzt Single-Day-Extra-Hours nach Cutover. Range ist
**inklusiv beidseitig** `[from, to]`. Detail: [F05](../features/F05-absence-system.md).

**Absence Period.** Konkrete Zeile in der `absence_period`-Tabelle,
identifiziert einen Range + Kategorie + Sales Person.

**Aggregat.** Fachliche Gruppierung mehrerer Entities, die zusammen
konsistent sein müssen (klassisch nach DDD). Siehe
[Architektur-Diagramme](../architecture/diagrams/domain-aggregates.mmd).

**Authentication::Full.** Auth-Enum-Variante, die alle
Permission-Checks passieren lässt. **Ausschließlich für interne
Aggregat-Reads** durch Business-Logic-Services. Details:
[04-auth.md](../architecture/04-auth.md).

## B

**Balance / Balance-Hours.** Die berechnete Differenz aus tatsächlich
gearbeiteten und vertraglich erwarteten Stunden, plus/minus Extras
(Urlaub, Krank, Feiertag). Formel:
`balance = worked − expected + carryover`. Details:
[time-accounting.md](./time-accounting.md).

**Basic Service.** Service-Klasse, die genau ein Fach-Objekt verwaltet
und keine anderen Domain-Services konsumiert. Siehe
[02-service-tiers.md](../architecture/02-service-tiers.md).

**Billing Period.** Abrechnungsperiode, in der die Balance/Urlaub/Stunden
eines Sales Person zu einem Snapshot eingefroren wird. Details:
[billing-period.md](./billing-period.md) und
[F08](../features/F08-billing-period.md).

**Block.** Zeitscheibe für Reports (üblicherweise Kalenderwoche).
`My Block` = benutzereigene Sicht. `Block Report` = HR-Sicht mit
Aggregation.

**Booking.** Zuweisung *Sales Person × Slot × Datum* — der Kern der
Schichtplanung. Details: [F03](../features/F03-booking.md).

**Booking Log.** Read-only Audit-Trail auf `bookings_view`, inklusive
Soft-Deletes. Zeigt, wer eine Booking wann angelegt/geändert hat.

**Business-Logic Service.** Service-Klasse, die mehrere Aggregate
kombiniert oder Cross-Entity-Invarianten pflegt. Darf andere
Domain-Services konsumieren.

## C

**Carryover.** Am Jahresende persistierter Saldo (Stunden und/oder
Urlaubstage), der ins Folgejahr rollt. Wird vom Scheduler wöchentlich
für Vor- und aktuelles Jahr aktualisiert. Vermeidet Rekalkulation
historischer Zeiträume.

**Contract.** Eine Zeile in `employee_work_details` mit Wochenstunden,
Wochentagen, gültig ab/bis. Ein Sales Person kann mehrere Contract-Zeilen
über die Zeit haben.

**Custom Extra Hours.** Vom Betrieb definierte Extra-Kategorie
zusätzlich zu den Standard-Enum-Werten. Wird pro Zeile referenziert.

## E

**Expected Hours.** Vertraglich erwartete Stunden pro Zeitraum. Ergibt
sich aus Contract × Tage − SpecialDays − UnpaidLeave.

**Extra Hours (Legacy).** Single-Day-Zeitzeilen für Überstunden,
Urlaub, Krank etc. Wird nach Cutover durch das Absence-System ersetzt,
existiert aber weiter für historische Daten. Kategorien:
`ExtraWork`, `Vacation`, `SickLeave`, `Holiday`, `Unavailable`,
`UnpaidLeave`, `VolunteerWork`, `CustomExtraHours`.

## F

**Fat Backend, Thin Client.** Kernprinzip: Business-Logik ausschließlich
im Backend. Frontend rendert nur.

**Feature Flag.** Statischer / boolean-orientierter Schalter, meist
vom Admin gesetzt. Unterschied zum Toggle: kein Stichtag, kein
User-Kontext. Detail: [F13](../features/F13-system-infrastructure.md).

## G

**gen_service_impl!.** Makro (`service_impl/src/macros.rs`), das
Service-Implementierungen mit ihren typisierten Dependencies
verdrahtet.

## H

**HR-Gate.** Auth-Regel, die eine Operation auf User mit HR-Rolle
einschränkt (z.B. Anlegen von Billing-Perioden, Bearbeiten fremder
Absences).

## I

**Impersonation.** Admin-Feature, mit dem sich ein Support-User als
anderer User verhält, um dessen Sicht zu reproduzieren. Session hält
`impersonate`-Flag.

## O

**OIDC.** OpenID Connect. Produktions-Auth-Modus.

## P

**Permission Service.** Zentrale Prüfstelle für Rollen-basierte
Autorisierung. Kern-Bypass: `Authentication::Full`.

## R

**Report.** Aggregat aus Bookings + Extra Hours + Absence + Carryover +
Special Days, das eine Balance und weitere Kennzahlen liefert. Details:
[F07](../features/F07-reporting-balance.md).

**Re-Point.** Datenumzug: Bookings werden von einem Slot auf einen
anderen umgehängt (z.B. bei Slot-Split). MUSS atomar in einer TX
laufen, sonst Doppelzählung.

**RBAC.** Role-Based Access Control. Shifty-Rollen sind in Migrations
definiert, Details: [F12](../features/F12-auth-session.md).

## S

**Sales Person.** Mitarbeiter-Entität mit Contract-Historie, Farbwahl,
Verfügbarkeits-Fenster. Details: [F01](../features/F01-employee-management.md).

**Session.** Login-Zustand des Users. Cookie-basiert, ggf.
`impersonate`-markiert. 365-Tage-Expiry.

**Shiftplan.** Aggregat aus Slots + Special Days + Katalog + Editor.
Kein Domain-Objekt, sondern die Anwendungssicht auf "Wer arbeitet
wann".

**Shiftplan Edit.** Business-Logic-Service zum Bearbeiten von
Shiftplänen inkl. Slot-Split, Booking-Migration, Wochen-Sperre-Check.

**Slot.** Zeitfenster mit Kapazität (`min_resources`,
`max_paid_employees`) pro Wochentag. Buchungen füllen Slots.

**Snapshot.** Eingefrorene Sicht auf Balance/Stunden/Urlaub in einer
Billing Period. Write-once, mit
`snapshot_schema_version` versioniert. Der Vertrag bei Formeländerung
ist streng — siehe
[billing-period.md](./billing-period.md).

**Snapshot Schema Version.** `pub const u32` in
`service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION`.
Aktuell **12**. Bump-Regeln in
[F08](../features/F08-billing-period.md).

**Soft-Delete.** Löschung durch Setzen einer `deleted`-Timestamp-Spalte
statt `DELETE FROM`. Reader filtern `WHERE deleted IS NULL`.

**Special Day.** Feiertag oder betrieblicher Sondertag, der die
Expected-Hours-Berechnung beeinflusst.

**Stichtag-Rollout.** Muster für Toggle-Features: Ab Datum X gilt neue
Semantik; davor bleibt alte Semantik gültig. Reporting muss beide
Semantiken über die Zeitachse hinweg konsistent behandeln.

## T

**Toggle.** User- und/oder datumsabhängiger Schalter, oft für
Stichtag-Rollouts (z.B. D-51-07). Unterschied zum Feature Flag:
zeit- und kontextabhängig.

**Transaction (Option\<Transaction\>).** Muster, in dem jede
Service-Methode `Option<Self::Transaction>` akzeptiert — öffnet
selbst, wenn `None`, fährt in äußerer TX mit, wenn `Some`.

## U

**UnpaidLeave.** Extra-Hours-Kategorie mit spezieller Semantik:
**senkt Erwartung, addiert nichts** auf die Ist-Seite. Andere Kategorien
(Vacation, SickLeave, Holiday) senken NICHT die Erwartung, sondern
addieren auf die Ist-Seite.

## V

**Vacation Balance.** Aktueller Urlaubsstand: Anspruch + Übertrag −
Verbraucht − Geplant. Formel:
`balance = entitled + carryover(year−1) − (used + planned)`. Details:
[F06](../features/F06-vacation-management.md).

**Vacation Entitlement Offset.** Manuelle Korrektur des
Urlaubsanspruchs (Boni, Abzüge). HR-only änderbar, nur HR-sichtbar.

**value_type.** Enum-Spalte in `billing_period_sales_person`, die
identifiziert, was für ein Wert eine Zeile trägt (z.B.
`WorkedHours`, `VacationDaysUsed`, `Balance`). Erweiterungen erzwingen
Snapshot-Version-Bump.

## W

**Week Message.** Info-Text pro Kalenderwoche, wird im Shiftplan
angezeigt.

**Week Status.** Freigabezustand einer Woche (`Unset`, `Planned`,
`Locked`, `Released`). Steuert, wer noch Änderungen machen darf.

**Working Days.** Wochentag-Flags im Contract, die definieren, an
welchen Tagen der Sales Person grundsätzlich arbeitet.

**Working Hours.** Vertraglich erwartete Stunden pro Woche.
