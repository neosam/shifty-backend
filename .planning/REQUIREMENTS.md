---
type: requirements
milestone: v2.7
defined: 2026-09-16
---

# Requirements: v2.7 Personenverwaltung & Planungs-Soll

**Defined:** 2026-09-16
**Core Value:** Die Verwaltung einer Person passiert an einem Ort, und die Schichtplanung sieht für jeden Vertragstyp ein Stundenziel.

> **Warnung fürs Formulieren (aus `research/PITFALLS.md`, Lehre aus v2.6):**
> Ein Requirement, das eine Invariante *festschreibt* („der Merge ändert nichts",
> „das Planungs-Soll wirkt nie auf die Balance"), ist die gefährliche Klasse — in
> v2.6 haben drei Phasen VOL-ACCT-03 korrekt umgesetzt und dabei verifiziert, dass
> das Feature *nicht* wirkt. Die Erhaltungs-Requirements unten (PERS-05, PSOLL-04)
> sind deshalb bewusst als **beobachtbare Gleichheit vorher/nachher** formuliert,
> nicht als Verbot — und keines von beiden steht dem Milestone-Goal entgegen.

## v1 Requirements

### Person-Datenmodell (PERS)

- [ ] **PERS-01**: Nach der Migration ist jede bestehende Sales Person und jeder bestehende Benutzer genau einer Person zugeordnet; wer heute über `sales_person_user` verknüpft ist, teilt sich danach dieselbe Person.
- [ ] **PERS-02**: Die Backfill-Migration ist wiederholbar — ein zweiter Lauf erzeugt keine zusätzlichen `person`-Zeilen und keine Doppelzuordnung.
- [ ] **PERS-03**: Der Backfill erfasst auch soft-gelöschte Sales Persons, damit Personen, die noch Abrechnungs-Snapshots besitzen, nicht ohne Person dastehen.
- [ ] **PERS-04**: Die Datenbank erzwingt weiterhin, dass eine Person höchstens einen Benutzer und höchstens eine Sales Person hat — die heute von `sales_person_user` getragene 1:1-Garantie geht nicht verloren.
- [ ] **PERS-05**: Ein Mitarbeiter sieht und bucht nach der Umstellung genau dieselben eigenen Daten wie vorher — die Selbst-Zugriffs-Prüfung arbeitet über die neue Person-Verknüpfung statt über `sales_person_user`.
- [ ] **PERS-06**: Bestehende Abrechnungs-Snapshots und Schichtplan-Daten referenzieren nach der Migration dieselben Sales Persons wie vorher (`sales_person.id` und `user.name` bleiben unverändert).

### Personenverwaltung (PADM)

- [ ] **PADM-01**: Ein Admin sieht **eine** Liste aller Einträge — verknüpfte, reine Benutzer und reine Verkäufer — mit erkennbarer Ausprägung und Suche, ohne Tab-Wechsel.
- [ ] **PADM-02**: Beim Anlegen erfasst der Admin **einen** Eintrag und schaltet dabei zu, ob er ein Benutzerkonto und/oder einen Verkäufer-Datensatz bekommt; die nötigen Objekte werden automatisch angelegt und verknüpft.
- [ ] **PADM-03**: Bei einem bestehenden Eintrag kann eine fehlende Ausprägung nachträglich zugeschaltet werden — der Verkäufer-Datensatz bzw. das Benutzerkonto wird angelegt und mit der Person verknüpft.
- [ ] **PADM-04**: Zwei bereits getrennt bestehende Objekte können ausnahmsweise nachträglich verknüpft werden, per Auswahl aus den noch unverknüpften Kandidaten statt durch exaktes Eintippen eines Benutzernamens.
- [ ] **PADM-05**: Eine Detailansicht zeigt alles zu einem Eintrag an einem Ort — Stammdaten und Schichtplan-Zuordnung für die Verkäufer-Seite, Rollen und Einladungen für die Benutzer-Seite — und blendet aus, was die jeweilige Ausprägung nicht hat.
- [ ] **PADM-06**: Der Anzeigename eines Eintrags kommt fertig vom Backend, sodass jeder Eintrag in Liste und Detailansicht beschriftet ist — auch ein reines Benutzerkonto ohne Verkäufernamen.
- [ ] **PADM-07**: Beim Entfernen einer Ausprägung oder beim Löschen eines Eintrags sagt die Bestätigung, was genau verschwindet und was bestehen bleibt.
- [ ] **PADM-08**: Die Personenverwaltung ist vollständig admin-gated — sie öffnet sich nur für Nutzer mit Admin-Rolle.

### Planungs-Soll (PSOLL)

- [ ] **PSOLL-01**: In der Übersicht unterhalb des Schichtplans steht auch bei dynamischen Verträgen ein Sollstunden-Ziel, statt 0.
- [ ] **PSOLL-02**: Das Planungs-Soll kommt fertig berechnet aus dem Backend — das Frontend rechnet keine Fallback-Regel.
- [ ] **PSOLL-03**: Das Planungs-Soll ist in **beiden** Report-Pfaden gefüllt (Wochen-Übersicht und Gesamt-Report), sodass dieselbe Person überall dieselbe Zahl zeigt.
- [ ] **PSOLL-04**: Für dieselben Ausgangsdaten liefert der Stundenkonto-Report nach Einführung des Planungs-Solls unveränderte Balance-Werte — insbesondere bleibt die Jahres-Verfügbarkeitsrechnung (`available_hours`) auf dem bisherigen, bei dynamischen Verträgen genullten Wert.

## Future Requirements

Erfasst, aber bewusst nicht in v2.7:

- **PADM-F-01**: HR-Report (`/employees/:id`) in die Personen-Ansicht einfalten — dient der Auswertung, nicht der Verwaltung; erst bei spürbarer Reibung.
- **PADM-F-02**: Abschnittsweises Rollen-Gating, falls HR später doch Zugriff auf Teile der Personenverwaltung braucht.
- **PERS-F-01**: Echte Personenstammdaten (E-Mail, Telefon, Geburtsdatum) an der `person`-Tabelle — sie ist dafür der natürliche Ort, sobald der Bedarf da ist.

## Out of Scope

| Feature | Grund |
|---------|-------|
| Automatisches Vorschlagen von Verknüpfungen (Namens-Matching) | Identity-Admin-Muster für große Organisationen; bei einer Handvoll Leute unnötig |
| Merge-Historie / Audit-UI für Verknüpfungen | dito — kein Bedarf bei dieser Nutzerzahl |
| Mehrere Benutzerkonten pro Person | Widerspricht der bewusst beibehaltenen 1:1-Beziehung |
| Bulk-Import von Personen | Kein Anwendungsfall |
| Umschlüsselung von `sales_person.id` oder `user.name` | Zwölf Tabellen referenzieren die UUID, darunter write-once-Abrechnungs-Snapshots; der Username trägt die Auth-Schicht |
| Snapshot-Schema-Version-Bump | `person` ist rein administrativ — kein neuer `value_type`, keine geänderte Computation, kein geändertes Input-Set. Version bleibt 13 |
| Phase 58 (Werktags-Stundenverteilung) und Phase 59 (E2E-Playwright) | Vorbereitet, aber bewusst nicht promotet — Nummern bleiben reserviert |

## Bewusste Änderung gegenüber heute

**PADM-08 nimmt HR-Nutzern ohne Admin-Rolle die Sales-Person-Verwaltung.** Heute
sind `create`/`update`/`delete` auf `HR_PRIVILEGE` gegated
(`service_impl/src/sales_person.rs:174,209,261`), während die Benutzerverwaltung
`"admin"` verlangt (`service_impl/src/permission.rs:111-199`). Die zusammengelegte
Seite vereinheitlicht das auf Admin. Vom User am 2026-09-16 explizit so entschieden,
nachdem die Konsequenz benannt wurde.

## Traceability

Gefüllt bei der Roadmap-Erstellung am 2026-09-16 (`.planning/ROADMAP.md`, Milestone-Sektion v2.7).

| Requirement | Phase | Status |
|-------------|-------|--------|
| PERS-01 | Phase 60 | Pending |
| PERS-02 | Phase 60 | Pending |
| PERS-03 | Phase 60 | Pending |
| PERS-04 | Phase 60 | Pending |
| PERS-05 | Phase 61 | Pending |
| PERS-06 | Phase 60 | Pending |
| PADM-01 | Phase 62 | Pending |
| PADM-02 | Phase 63 | Pending |
| PADM-03 | Phase 63 | Pending |
| PADM-04 | Phase 63 | Pending |
| PADM-05 | Phase 63 | Pending |
| PADM-06 | Phase 62 | Pending |
| PADM-07 | Phase 63 | Pending |
| PADM-08 | Phase 62 | Pending |
| PSOLL-01 | Phase 64 | Pending |
| PSOLL-02 | Phase 64 | Pending |
| PSOLL-03 | Phase 64 | Pending |
| PSOLL-04 | Phase 64 | Pending |

**Coverage:**
- v1 requirements: 18 total
- Mapped to phases: 18 ✓
- Unmapped: 0

**Verteilung:**

| Phase | Requirements | Anzahl |
|---|---|---|
| Phase 60 — Person-Datenmodell + Backfill | PERS-01, PERS-02, PERS-03, PERS-04, PERS-06 | 5 |
| Phase 61 — PersonService + Auflösung `sales_person_user` | PERS-05 | 1 |
| Phase 62 — Personen-Verzeichnis (eine Liste statt Tabs) | PADM-01, PADM-06, PADM-08 | 3 |
| Phase 63 — Personen-Detailansicht + Ausprägungs-Aktionen | PADM-02, PADM-03, PADM-04, PADM-05, PADM-07 | 5 |
| Phase 64 — Planungs-Soll für dynamische Verträge | PSOLL-01, PSOLL-02, PSOLL-03, PSOLL-04 | 4 |

Jedes Requirement ist genau einer Phase zugeordnet — keine Waisen, keine Doppelung.

---
*Requirements defined: 2026-09-16 — Traceability gefüllt 2026-09-16 (Roadmap v2.7, Phasen 60–64)*
