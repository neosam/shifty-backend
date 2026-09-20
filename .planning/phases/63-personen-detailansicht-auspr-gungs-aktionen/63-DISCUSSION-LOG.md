# Phase 63: Personen-Detailansicht + Ausprägungs-Aktionen - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-20
**Phase:** 63-Personen-Detailansicht + Ausprägungs-Aktionen
**Areas discussed:** Route und Rückbau der alten Detailseiten, Lebenszyklus der
`person`-Zeile, Bedeutung von „Ausprägung entfernen", Freischaltung des
Verkäufer-Löschens, Umfang des Anlege-Dialogs, Trennung neu-anlegen /
verknüpfen, Inhalt der Lösch-Bestätigung

**Modus:** Alle sieben Fragen wurden auf ausdrücklichen Wunsch des Nutzers als
**ein** Textblock gestellt („Bitte in einem einzigen Block abfragen. Nicht das
Abfragetool nutzen. Ich will auf einen Schlag antworten."), jede mit
Empfehlung. Der Nutzer hat den Block als Ganzes bestätigt: „Ok, dann klingt die
Empfehlung überall gut."

**Zwischenfrage des Nutzers:** „Hast du bei F2 gefragt, ob Sales Persons gelöscht
werden sollen?" — Nein; F2 betraf ausschließlich die `person`-Identitätszeile.
Die Löschbarkeit der Verkäufer-Seite war F4, die Unterscheidung lösen/löschen
F3. Nach der Klarstellung folgte die Gesamtzustimmung.

---

## F1 — Route und Rückbau der beiden alten Detailseiten → D-63-04

| Option | Beschreibung | Gewählt |
|--------|-------------|---------|
| Neue Route `/person/:person_id/`, alte Routen ersatzlos entfernen | `person_id` liegt im `PersonEntryTO` bereits vor; grep zeigt nur `user_management.rs` und `top_bar.rs` als Aufrufer | ✓ |
| Alte Routen als Weiterleitungs-Hüllen stehen lassen | schützt gespeicherte Lesezeichen, kostet zwei tote Routen | |

**User's choice:** harter Rückbau.
**Notes:** Nachträgliches Nachrüsten von Weiterleitungen bleibt möglich und steht
als Deferred Idea. Nachweis-Auflage: abschließender Grep auf `UserDetails` /
`SalesPersonDetails` muss leer sein.

---

## F2 — Lebenszyklus der `person`-Zeile → D-63-01 (ROADMAP-Discuss-Point D-63-01)

| Option | Beschreibung | Gewählt |
|--------|-------------|---------|
| `person`-Zeile beim Wegfall der letzten Ausprägung mitlöschen | PITFALLS-Empfehlung, einfachste Variante; sicher wegen D-60-04 | ✓ |
| Zeile liegen lassen | erzeugt trägerlose Waisen, die über den Outer-Join ohnehin unsichtbar wären | |
| Waisen in einer eigenen Admin-Sicht zeigen | zusätzliche Sicht ohne erkennbaren Nutzen | |

**User's choice:** mitlöschen.
**Notes:** Abgrenzung im Log festgehalten — beim **Lösen** entsteht nach D-61-05
weiterhin keine trägerlose Person; D-63-01 greift nur beim **Löschen** des
letzten Trägers. Diese Frage betraf ausdrücklich nicht die Löschbarkeit von
Sales Persons (siehe Zwischenfrage oben).

---

## F3 — Bedeutung von „Ausprägung entfernen" → D-63-05

| Option | Beschreibung | Gewählt |
|--------|-------------|---------|
| „Verknüpfung lösen" und „löschen" getrennt und unterschiedlich beschriftet, **kein** Sammel-„Eintrag löschen" | drei klar benannte Aktionen; PADM-07 über die Einzelaktionen erfüllt | ✓ |
| Zusätzlicher Sammelknopf „Eintrag löschen" | bündelt auf einem verknüpften Eintrag Hard Delete (Benutzer) und Soft Delete (Verkäufer) in einer Geste | |

**User's choice:** getrennt, kein Sammelknopf.
**Notes:** Auslöser war der Code-Befund, dass `delete_user`
(`service_impl/src/permission.rs:135-143`) hart löscht, `SalesPersonService::delete`
(`sales_person.rs:253+`) dagegen soft.

---

## F4 — Freischaltung des Verkäufer-Löschens im Frontend → D-63-06

| Option | Beschreibung | Gewählt |
|--------|-------------|---------|
| Löschpfad freischalten, in der Bestätigung auf „inaktiv setzen" hinweisen | ohne Freischaltung bliebe PADM-07 für die Verkäufer-Seite eine leere Zusage | ✓ |
| Beim Status quo bleiben — Verkäufer verschwinden nur über `inactive` | kein neuer destruktiver Pfad, aber Requirement nur halb erfüllt | |

**User's choice:** freischalten, mit Hinweis auf die schonende Alternative.
**Notes:** Belegt per grep — im Frontend existiert heute kein einziger Aufruf von
`delete_sales_person`; nur `delete_user` ist verdrahtet.

---

## F5 — Umfang des Anlege-Dialogs → D-63-07 (+ D-63-03 aus der ROADMAP)

| Option | Beschreibung | Gewählt |
|--------|-------------|---------|
| Schlank: Anzeigename, zwei Umschalter, Benutzername nur wenn Login an; Rest danach in der Detailansicht | Dialog leitet nach dem Speichern direkt in die Detailansicht weiter | ✓ |
| Vollformular: Farbe, bezahlt, inaktiv, Schichtplan gleich im Dialog | baut die abgelöste Verkäufer-Detailseite im Dialog nach | |

**User's choice:** schlank.
**Notes:** Der Rückbau der beiden heutigen Knöpfe („Neuen Verkäufer erstellen",
„Neuen Benutzer hinzufügen", `user_management.rs:296-315`) war als D-63-03 bereits
in der ROADMAP gepinnt und stand nicht zur Wahl — er wurde in der Frage nur als
Rahmen genannt. Zusatz-Auflage aus der Diskussion: mindestens ein Umschalter muss
an sein.

---

## F6 — Trennung neu-anlegen / verknüpfen → D-63-02 (ROADMAP-Discuss-Point D-63-02)

| Option | Beschreibung | Gewählt |
|--------|-------------|---------|
| Zwei getrennt beschriftete Aktionen, Kandidaten-Picker nur an der zweiten | FEATURES-Empfehlung, Keycloak-Präzedenz | ✓ |
| Ein Dialog mit Auswahlliste plus „ist nicht dabei, neu anlegen" | verwischt genau die Unterscheidung, die getrennt werden soll | |

**User's choice:** zwei Aktionen.
**Notes:** Serverseitig bleibt D-61-04 unverändert — bereits verknüpfte Ziele
werden mit 409 abgelehnt, auch wenn sie im Picker nicht erscheinen. Success
Criterion 4 verlangt beides.

---

## F7 — Inhalt der Lösch-Bestätigung → D-63-08

| Option | Beschreibung | Gewählt |
|--------|-------------|---------|
| Fester, ausprägungs-spezifischer Text ohne Zahlen | zwei Bausteine, nach Verkäufer-/Benutzer-Seite unterschieden | ✓ |
| Backend zählt die echte Betroffenheit vor dem Löschen | ehrlicher, kostet einen eigenen Zähl-Endpoint samt Tests | |

**User's choice:** statischer Text.
**Notes:** Die gezählte Variante ist als Deferred Idea festgehalten, nicht
verworfen.

---

## Claude's Discretion

Vom Nutzer nicht adressiert und damit ausdrücklich offen für den Planer: Zuschnitt
des Detail-Aggregats (ein Endpoint vs. Kern plus Nachladen), Name und Modul-Ablage
des Business-Logic-Services, Form des Anlege-Endpoints (transaktional vs.
Frontend-Orchestrierung — transaktional stark empfohlen), Routen-Präfixe,
Darstellung und Reihenfolge der Abschnitte, Form des Kandidaten-Pickers,
Durchsetzung der „mindestens ein Umschalter"-Regel, i18n-Key-Namen,
Wellen-Schnitt und Testablage.

## Deferred Ideas

Gezählte Betroffenheit in der Lösch-Bestätigung; Weiterleitungs-Hüllen für die
beiden alten Routen; abschnittsweises Rollen-Gating (PADM-F-02); HR-Report
`/employees/:id` einfalten (PADM-F-01); echte Personenstammdaten (PERS-F-01);
UI zum Zusammenführen zweier Personen; Planungs-Soll (Phase 64); die offenen
Befunde `WR-01`, `WR-03`, `WR-07`, `IN-01..IN-07` aus `61-REVIEW.md`.

Aus `todo.match-phase 63`: acht Treffer, alle Keyword-Zufall, keiner gefaltet —
dieselbe Menge, die Phase 62 bereits geprüft und verworfen hat.
