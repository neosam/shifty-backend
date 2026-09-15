---
title: Benutzerverwaltung und Sales-Person-Verwaltung zu einer Personenverwaltung zusammenlegen
trigger_condition: nächster Milestone nach v2.6 (User-Festlegung 2026-09-15)
planted_date: 2026-09-15
---

# Personenverwaltung zusammenlegen

## Auslöser

User-Festlegung vom 2026-09-15: *„Wir müssen im nächsten Meilenstein die UI für
die Benutzerverwaltung und Sales Person Verwaltung zusammenlegen. Das ist
aktuell extrem verwirrend für die Benutzer."*

→ Kern-Scope für den nächsten `/gsd-new-milestone`.

## Ist-Zustand (verifiziert am 2026-09-15)

**Die Listen sind bereits zusammengelegt, die Detailseiten nicht.**
`/user_management/` (`page/user_management.rs:40`) hat eine Tab-Leiste mit
`UserMgmtTab::SalesPersons` (Default) und `UserMgmtTab::Users`, lädt beim Mount
beides (`LoadAllUsers` + `LoadAllSalesPersons`) und zeigt je eine durchsuchbare
Liste.

Der Bruch passiert beim Klick in die Tiefe — dieselbe reale Person zerfällt in
**drei** Detailseiten mit je eigener Route, eigenem Zurück-Ziel und eigenem
Datenausschnitt:

| Route | Datei | Zeigt |
|---|---|---|
| `/sales_person_details/:sales_person_id/` | `page/sales_person_details.rs` | Stammdaten (Name, …), Shiftplan-Zuordnungen |
| `/user_details/:user_id/` | `page/user_details.rs` | Rollen/Berechtigungen, Einladungen, Session-Revoke |
| `/employees/:employee_id/` | `page/employee_details.rs` | Report-Sicht: Balance, Stunden, Freiwilligen-Konto, Verträge |

Wer eine Person anlegt und einsatzbereit machen will, muss also mindestens zwei
Objekte an zwei Orten pflegen und dazwischen über die Tab-Leiste navigieren —
ohne dass die UI irgendwo sagt, dass es dieselbe Person ist.

## Das Datenmodell trägt die Zusammenlegung bereits

`sales_person_user` (Migration `20240506114107_add_sales_person.sql:14`) ist eine
Verknüpfungstabelle mit `UNIQUE(sales_person_id)` **und** `UNIQUE(user_id)` — also
eine echte **1:1-Beziehung**, die es seit 2024 gibt. Beide Seiten sind optional:
eine SalesPerson ohne Zeile in `sales_person_user` hat kein Login, ein User ohne
Zeile keine Schichtidentität — genau die drei Ausprägungen, die unten festgelegt
sind. `SalesPersonDao` hat schon
`find_sales_person_by_user_id` (`dao/src/sales_person.rs:71`), Service-seitig
existieren `assign_user_to_sales_person` / Pendant
(`service/src/sales_person.rs:109,115`).

**Korrektur (2026-09-15, nach der Person-Entscheidung unten):** Die erste
Einschätzung „reine Frontend-Phase, keine Migration" ist hinfällig. Mit einem
eigenen Fachobjekt `Person` wird das eine **Data-Model-first-Phase** mit
Migration und Backfill — Frontend ist die zweite Hälfte, nicht die ganze
Aufgabe.

## Festgelegt (User, 2026-09-15)

**Es gibt genau einen Eintragstyp, und eine Komponente deckt ihn ab.** Ob dahinter
ein Benutzer, eine Sales Person oder beides verknüpft steckt, ist eine
Eigenschaft des Eintrags — kein eigener Seitentyp. Der Regelfall ist „beides
verknüpft"; die beiden anderen Ausprägungen sind gleichwertige, normale Fälle,
keine Sonderfälle:

| Ausprägung | Beispiel | Was der Eintrag hat |
|---|---|---|
| User + SalesPerson (Regelfall) | Mitarbeiter mit Login | Stammdaten, Shiftplan-Zuordnung, Rollen, Einladungen |
| nur User | reiner HR- oder Admin-Account | Rollen, Einladungen, Login-Verwaltung — keine Schichtdaten |
| nur SalesPerson | Freiwillige/r ohne Login | Stammdaten, Shiftplan-Zuordnung — keine Rollen, kein Login |

Die Komponente blendet also Abschnitte je nach Ausprägung ein oder aus, statt auf
verschiedene Seiten zu verzweigen. Daraus folgt direkt:

- „Login hinzufügen" (zu einer SalesPerson) und „Person hinzufügen" (zu einem
  User) sind **Aktionen auf dem Eintrag**, keine Neuanlage an anderer Stelle.
  Sie überführen eine Ausprägung in eine andere.
- Die Liste unter `/user_management/` braucht keine Tabs mehr, sondern **eine**
  Liste aller Einträge mit erkennbarer Ausprägung (Badge/Spalte) und Filter.

## Entscheidung: eigenes Fachobjekt `Person` (User, 2026-09-15)

Von den drei Identitäts-Varianten fällt die Wahl auf die dritte: **ein neues
Fachobjekt `Person`** als Aggregat über User und SalesPerson. Damit hat der
Eintrag eine echte, stabile Identität statt einer aus zwei Fremd-PKs
zusammengesetzten.

### Was dabei nicht angetastet werden darf

`sales_person.id` bleibt. Die UUID wird von **zwölf Tabellen** referenziert
(`booking`, `extra_hours`, `working_hours`, `absence_period`,
`sales_person_unavailable`, `sales_person_shiftplan`, `shiftplan`,
`billing_period_sales_person`, Carryover-Backups, Quarantäne-Tabellen …) —
darunter `billing_period_sales_person`, also die **write-once-Snapshots**. Eine
Umschlüsselung der SalesPerson-ID würde historische Abrechnungsdaten berühren
und ist damit ausgeschlossen.

Ebenso bleibt `user.name` als PK: daran hängen `user_role` (und darüber die
Privilegien), `user_invitation` und `sales_person_user`, dazu die Session-/
Auth-Schicht.

**Person kommt also obendrauf, nicht an deren Stelle.** `sales_person` und
`user` bleiben die Aggregate, an denen die Fachdaten hängen; `Person` bündelt
sie und trägt die Identität des Eintrags.

### Offene Modellierungsfrage

Zwei Varianten, beide additiv:

- **A — Person als Elterntabelle:** neue Tabelle `person(id, …)`, dazu
  `sales_person.person_id` und `user.person_id` als nullable FKs.
  `sales_person_user` wird überflüssig und aufgelöst. Fachlich am saubersten
  (Person ist das Aggregat, User/SalesPerson sind Facetten), verlangt aber
  Spalten-Migrationen an zwei bestehenden Tabellen.
- **B — `sales_person_user` zu `person` weiterentwickeln:** die bestehende
  Verknüpfungstabelle bekommt eine eigene `id` und beide Spalten werden
  nullable, sodass sie auch Einträge ohne Gegenstück trägt. Migrationsärmer,
  aber „Person" bleibt strukturell eine Verknüpfungstabelle mit Identität.

Für beide gilt: **Backfill-Migration** nötig — für jede bestehende SalesPerson
und jeden bestehenden User eine Person anlegen, verknüpfte Paare zu *einer*
Person zusammenführen. Die 1:1-Constraints auf `sales_person_user` garantieren,
dass dabei keine Mehrdeutigkeit entsteht.

### Weitere zu klärende Punkte

- **Wo lebt der Name?** Heute doppelt: `sales_person.name` (Anzeigename) und
  `user.name` (PK = Login-Kennung). Mit einer Person wäre der Anzeigename
  natürlicherweise an ihr — was aber die Frage aufwirft, was mit
  `sales_person.name` passiert (Migration der Werte? Spalte behalten und
  spiegeln? deprecaten?). `user.name` bleibt in jedem Fall die Login-Kennung.
- **Snapshot-Schema-Version:** voraussichtlich **kein** Bump. Person ändert
  keine Computation und keinen persistierten `value_type`; solange
  `billing_period_sales_person.sales_person_id` unverändert bleibt, ist der
  Snapshot-Vertrag nicht berührt. In der discuss-phase explizit gegen die
  CLAUDE.md-Klausel prüfen und die Begründung festhalten (Präzedenz: v1.4
  no-bump-Justification).
- **Docs-Trigger:** eine Migration zieht `docs/architecture/03-data-model.md` +
  `diagrams/db-schema-er.mmd` + die passenden `docs/features/F*.md` nach sich,
  jeweils EN **und** DE (hartes Gate beim Milestone-Close).

## Vor dem Planen zu klären

1. ~~Woran wird ein Eintrag identifiziert?~~ **Entschieden:** eigenes Fachobjekt
   `Person` (siehe oben). Offen bleibt nur noch Variante A vs. B der
   Modellierung.
2. **Gehört `/employees/:id` mit hinein** oder bleibt die Report-Sicht (Balance,
   Stunden, Freiwilligen-Konto) getrennt? Sie ist HR-gated und dient der
   Auswertung, nicht der Verwaltung — Argument für getrennt lassen, dann aber mit
   klarem Absprung zwischen beiden Sichten.
3. **Abschnittsweises Rollen-Gating.** Rollen/Einladungen/Impersonation sind heute
   Admin-Gebiet (`/user_details`), Stammdaten/Shiftplan shiftplanner-nah
   (`/sales_person_details`). Auf einer gemeinsamen Seite muss das pro Abschnitt
   greifen, nicht pro Seite.
4. **Wer liefert die Union?** Nach „Fat Backend, Thin Client" sollte das Backend
   die Eintragsliste fertig ausgeprägt liefern (inkl. Ausprägungs-Marker), statt
   das FE zwei Listen mergen zu lassen. Präzedenz: v2.5 VAA
   (`sales_person_absences` als Union bezahlt + freiwillig).

## Vorhandene Bausteine

- Impersonation („Act as this person", D-32-07 / IMP-01) hängt heute am
  Users-Tab — in einer Personen-Sicht wäre das eine Aktion auf der Person.
- `USER_MANAGEMENT_STORE` + `UserManagementAction` bedienen bereits beide
  Entitäten; der Store müsste nicht neu geschnitten werden.
- Präzedenz für BE-liefert-fertiges-Aggregat: v2.5 VAA
  (`sales_person_absences` als Union bezahlt + freiwillig, FE macht reinen
  Merge).

## Verwandt

- Todo `2026-07-03-schmale-admin-rolle-mit-user-impersonation.md` — berührt
  dieselbe Rollen-/Berechtigungsecke, sollte beim Zuschnitt mitgelesen werden.
- Todo `2026-05-08-admin-rolle-bekommt-alle-privilegien.md` — dito.
