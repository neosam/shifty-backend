# Billing Period — Snapshot & Versionierung (Fachlich)

Diese Datei erklärt das Billing-Period-Konzept aus Fach-Sicht. Für die
technische Referenz siehe
[F08 Billing Period](../features/F08-billing-period.md).

## Was ist eine Billing Period?

Eine **Billing Period** ist ein abgegrenzter Abrechnungszeitraum,
für den zu einem bestimmten Zeitpunkt ein **Snapshot** der
Balance-, Stunden- und Urlaubs-Zahlen jedes Sales Person erzeugt und
in der Datenbank eingefroren wird.

**Zweck:** Eine Abrechnung / ein Ausdruck / eine Auszahlung soll nicht
davon abhängen, wann der Report genau geöffnet wird. Wenn die
HR-Person am 3. des Monats den Snapshot für den Vormonat erzeugt hat,
soll er am 30. desselben Monats denselben Wert zeigen — auch wenn in
der Zwischenzeit rückwirkend Bookings korrigiert wurden.

## Vertrag: Write-Once + Versioniert

Zwei Regeln machen den Snapshot verlässlich:

### Regel 1: Write-Once

Wenn eine Billing Period einmal geschrieben ist, ist ihr Inhalt fix.
Nachträgliche Bookings, Absences, Extra-Hours werden **nicht**
zurück in den Snapshot geschrieben. Das ist absichtlich.

**Konsequenz für User:** Der Live-Report (Weekly Overview, My Shifts)
kann von der Billing-Period abweichen, wenn nachträglich Daten
geändert wurden. Beide Zahlen sind valide — die eine ist "was am 3.
war", die andere "was heute ist".

### Regel 2: Versionierung (`snapshot_schema_version`)

Jede Billing-Period-Zeile trägt eine Versions-Nummer:
`snapshot_schema_version: u32`, aktuell **12**.

Diese Zahl sagt: "Zum Zeitpunkt, als dieser Snapshot geschrieben
wurde, galten diese Rechenregeln."

**Wenn sich die Rechenregeln ändern** (neue Kategorie, andere Formel,
anderer Input-Set), muss die Version um 1 erhöht werden. Der Grund:
Ein Validator, der Live-Rechnung gegen Snapshot vergleicht, kann
sonst nicht unterscheiden zwischen:

- "Der Snapshot ist falsch" (echter Datenbug) und
- "Der Snapshot wurde unter alten Regeln geschrieben" (erwarteter
  Diff wegen Regeländerung).

Ohne Version wären alle Alt-Snapshots nach einer Formeländerung
plötzlich "falsch" — jedes Post-Mortem wäre sinnlos.

**Mit Version:** Validator liest die Version des Snapshots. Wenn
niedriger als aktuelle Konstante → alte Regeln, Diff erwartet.
Wenn gleich → aktuelle Regeln, Diff ist ein Bug.

## Wann eine Bump-Pflicht besteht

**Bumpe die Version wenn du:**

1. Einen neuen persistierten `value_type` zu `billing_period_sales_person`
   hinzufügst.
2. Einen bestehenden `value_type` entfernst oder umbenennst.
3. Die Berechnung eines existierenden `value_type` änderst.
4. Den Input-Set änderst (z.B. eine neue Extra-Hours-Kategorie mit-
   aggregierst).

**Bumpe NICHT wenn du:**

1. Rein additive Änderungen machst, die keinen `value_type` berühren
   (neue REST-Endpoints, Frontend-Änderungen, neue Spalten auf
   unrelated Tabellen).
2. Interne Refactorings machst, die identisches Ergebnis liefern.

**Technische Referenz:** Die Konstante ist
`service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION`
und wird vom Writer in
`build_and_persist_billing_period_report()` gestempelt. Siehe
[F08](../features/F08-billing-period.md) für den Code-Nachweis.

## Vier Report-Sichten pro Snapshot

Ein Snapshot enthält vier Aggregat-Sichten:

- **`value_ytd_from`** — Year-to-date bis zum Start der Periode.
- **`value_ytd_to`** — Year-to-date bis zum Ende der Periode.
- **`value_full_year`** — Ganzjahressumme.
- **`value_delta`** — Differenz Ende − Start (= das, was in der Periode
  passierte).

Für jeden `value_type` (Balance, Worked Hours, Vacation Used, …) gibt
es diese vier Sichten.

## Wer welchen Snapshot lesen darf

- **HR:** Alle Billing Periods sehen und anlegen.
- **Sales Person:** Nur eigene Historie in Billing Period Details.
- **[Zu prüfen]** genaue Auth-Gates — siehe F08.

## Wann Snapshots ins Bild kommen

- **Monatsende / Quartalsende:** HR erzeugt den Snapshot für die
  vergangene Periode. Ab dem Moment ist die Zahl "offiziell".
- **Jahresende:** Der Ganzjahres-Snapshot dient als Basis für den
  Carryover ins Folgejahr.
- **Ad-hoc:** HR kann jederzeit einen Zeitraum-Snapshot erzeugen — für
  Zeugnisse, Abrechnungen, Kontrollen.

## Was NICHT im Snapshot ist

- **Booking-Details** — es wird nur die Aggregat-Stunde persistiert,
  keine Zeile-für-Zeile-Sicht.
- **Absence-Details** — nur die Summen.
- **Textliche Kommentare** — Warnings, Week Messages sind live-only.

## Randfall-Referenzen

Siehe [`edge-cases.md#3-billing-period--snapshots`](./edge-cases.md#3-billing-period--snapshots)
für die scharfen Kanten:

- Alter Snapshot auf neuem Code.
- Bump-Vergessen nach Formeländerung.
- Race Snapshot ↔ paralleler Write.
- Kein Snapshot vorhanden — Live-Rechnung fällt ein.
- Toggle-basierte Semantik-Änderung — MUSS bumpen.

## PR-Review-Muster

**Verpflichtender Check bei Änderungen an `billing_period_report.rs`:**

1. Wurde `CURRENT_SNAPSHOT_SCHEMA_VERSION` gebumpt?
2. Wenn ja, ist im PR-Text dokumentiert, warum?
3. Wenn nein, ist geklärt, dass die Änderung wirklich additiv ist?

Ohne diese Prüfung driftet die Snapshot-Semantik still.
