# Phase 60: Person-Datenmodell + Backfill - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-16
**Phase:** 60-person-datenmodell-backfill
**Areas discussed:** Identität bei Löschen/Neuanlage, Produktions-Datenlage & Ausprägungen, Sicherung vor der Migration

**Verfahren:** Textmodus, alle Fragen in einer Runde (User-Präferenz 2026-09-16).
Vier ursprünglich vorbereitete Graubereiche wurden auf drei User-Fragen
eingedampft; die rein technischen Punkte (Spaltenzuschnitt, Zeitpunkt der
`person_id`-Vergabe, Verifikations-Mechanik, Test-Ablage) hat Claude selbst
entschieden und zur Kontrolle offengelegt — gemäß der bestehenden Rückmeldung
„wenige, einfach formulierte Fragen, technische Details selbst entscheiden".

---

## Identität bei Löschen und Neuanlage

Sales Persons werden soft-gelöscht, weil Abrechnungsdaten daran hängen. Frage:
behält der gelöschte Eintrag seine Person-Kennung, oder wird sie wieder frei?

| Option | Description | Selected |
|--------|-------------|----------|
| Kennung behalten (voller Unique-Index) | Gelöschte Sales Person belegt ihre Person dauerhaft. Historische Abrechnungs-Snapshots lösen eindeutig auf. Preis: ein zurückkehrender Mitarbeiter ist zunächst zwei Personen. | ✓ |
| Kennung freigeben (partieller Index `WHERE deleted IS NULL`) | Ein neuer Eintrag könnte dieselbe Person übernehmen. Entspricht dem sonst dominanten Repo-Muster (`rebooking_batch_week_unique_idx`). Risiko: zwei verschiedene Menschen unter einer Identität, alte Snapshots zeigen auf die falsche Person. | |

**User's choice:** Kennung behalten (Empfehlung bestätigt).
**Notes:** Beantwortet die von `PITFALLS.md` Pitfall 3 ausdrücklich als
Produktentscheidung markierte Frage. Die Abweichung vom Repo-Standardmuster
braucht einen erklärenden Kommentar in der Migration, sonst liest sie sich wie
ein Versehen. Das manuelle Zusammenführen zweier Personen wurde als Folgeidee
notiert, nicht eingeplant.

---

## Produktions-Datenlage & Ausprägungen

Gefragt war die grobe Größenordnung von Benutzern ohne Sales Person, Sales
Persons ohne Login und soft-gelöschten Sales Persons mit alten Abrechnungen.

| Option | Description | Selected |
|--------|-------------|----------|
| Grobe Zahlen liefern | Plan kann sich auf Mengenordnungen stützen. | |
| „Keine Ahnung, prüf's beim Deploy" | Plan muss ohne Mengenannahme auskommen, Zählprobe wird die Absicherung. | |
| Freitext-Antwort des Users | Keine Zahlen, dafür eine fachliche Festlegung zur Optionalität. | ✓ |

**User's choice:** Freitext — keine Zahlen, stattdessen eine Klarstellung zum
Sollverhalten.
**Notes:** Wörtlich sinngemäß: *„es gibt HR Benutzer und die soll es auch
weiterhin geben und das soll auch weiterhin möglich sein. Es soll nicht immer
automatisch beides angelegt werden. Benutzer oder Sales Person bei einer Person
soll optional sein."* — Deckt sich mit der bereits im Seed gesperrten
Drei-Ausprägungen-Festlegung; wurde dem User so zurückgespiegelt und bestätigt.
Wichtig für die Planung: die Optionalität ist eine dauerhafte fachliche Zusage,
kein Übergangszustand, und der eindeutige Index darf sie nicht einschränken
(er verbietet nur **zwei** Träger derselben Seite, nicht **null**). Da keine
Zahlen vorliegen, darf sich der Plan auf keine Mengenannahme stützen — alle drei
Fälle brauchen eine eigene Fixture.

---

## Sicherung vor der Migration

| Option | Description | Selected |
|--------|-------------|----------|
| Datei-Backup der SQLite-Datei genügt | Migration fügt nur Spalten hinzu, überschreibt keine Daten. Rollback = Datei zurückspielen. | ✓ |
| Zusätzliche Backup-Tabelle in der DB | Präzedenz aus v1.0 (`…pre-cutover-backup.sql`). Dort wurden allerdings Daten umgerechnet — hier nicht. Wäre totes Gewicht mit späterem Aufräumbedarf. | |

**User's choice:** Datei-Backup genügt (Empfehlung bestätigt).
**Notes:** Der Rollback-Pfad ist damit explizit „Datei zurückspielen", nicht
„Down-Migration".

---

## Claude's Discretion

Vom User bestätigt, ohne Widerspruch:

- Spaltenzuschnitt von `person`: nur Identität + übliche Änderungs-Spalten,
  **keine** `deleted`-Spalte (D-60-04).
- Zeilen, die nach der Migration entstehen, bekommen in Phase 60 noch kein
  `person_id`; die Vergabe beim Anlegen ist Phase-61-Scope. Kein DB-Trigger,
  kein DAO-Eingriff (D-60-10).
- Die 1:1-Garantie kommt als `CREATE UNIQUE INDEX` — technischer Zwang, weil
  SQLite bei `ALTER TABLE … ADD COLUMN` kein `UNIQUE` erlaubt (D-60-05).
- Der DB-Level-Ablehnungstest läuft über den Index, nicht über Fremdschlüssel:
  `PRAGMA foreign_keys` ist in diesem Repo nirgends gesetzt, FK-Enforcement in
  SQLite also aus (D-60-06). Wird in der Doku festgehalten.
- Die Migration prüft die Zählprobe selbst und bricht bei Abweichung ab, statt
  halb migrierte Daten zu hinterlassen (D-60-07).
- Der Apply-twice-Nachweis ist ein echter Test mit Vorher/Nachher-Vergleich, kein
  „läuft ohne Fehler durch" (D-60-08).
- Exakte Audit-Spaltenwahl, Migrations-Dateiname, konkrete Form der
  Selbst-Assertion, Test-Ablage und Index-Namen bleiben dem Planer überlassen.

## Deferred Ideas

- Zusammenführen zweier Personen (zurückkehrender Mitarbeiter) — direkte Folge
  der Entscheidung oben, kein v2.7-Scope.
- „Person löschen" inkl. `deleted`-Spalte auf `person` — frühestens Phase 63,
  falls dort überhaupt gebraucht.
- Echte Personenstammdaten an `person` (E-Mail, Telefon, Geburtsdatum) —
  PERS-F-01, Future.
- `PRAGMA foreign_keys = ON` aktivieren — projektweiter Blast Radius,
  eigenständige Änderung.
