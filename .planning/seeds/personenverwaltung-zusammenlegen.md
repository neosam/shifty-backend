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
Verknüpfungstabelle mit `UNIQUE(user_id)` und FK auf `user(name)` — also eine
**1:1-Beziehung**, die es seit 2024 gibt. `SalesPersonDao` hat schon
`find_sales_person_by_user_id` (`dao/src/sales_person.rs:71`), Service-seitig
existieren `assign_user_to_sales_person` / Pendant
(`service/src/sales_person.rs:109,115`).

**Konsequenz:** Das ist sehr wahrscheinlich eine reine Frontend-/UX-Phase.
Backend-Änderungen wären allenfalls additiv (z. B. ein Aggregat-DTO, das
Person + User + Rollen in einem Rutsch liefert — Fat Backend, Thin Client).
Keine Migration absehbar, kein Snapshot-Bump.

## Vor dem Planen zu klären

1. **Was ist die führende Entität in der neuen UI?** Kandidaten: „Person" als
   neues UI-Konzept über beiden, oder SalesPerson als Anker mit optionalem
   User-Login-Abschnitt. Die zweite Variante wäre näher am Datenmodell
   (`sales_person_user.user_id` ist optional/`UNIQUE`, nicht jede SalesPerson
   hat einen Login und umgekehrt).
2. **Gehört `/employees/:id` mit in die Zusammenlegung** oder bleibt die
   Report-Sicht bewusst getrennt? Sie ist HR-gated und hat einen ganz anderen
   Zweck (Auswertung statt Verwaltung) — Argument für getrennt lassen, aber
   dann braucht es einen klaren Absprung zwischen beiden.
3. **Was passiert mit Personen ohne Gegenstück?** SalesPerson ohne User
   (Freiwillige ohne Login) und User ohne SalesPerson (reine Admins) müssen in
   der zusammengelegten Sicht sauber darstellbar bleiben — inklusive der
   Aktion „Login hinzufügen" / „Person hinzufügen".
4. **Rollen-/Sichtbarkeitsmatrix.** `/user_details` ist heute admin-gebiet
   (Rollen, Einladungen, Impersonation), `/sales_person_details` ist
   shiftplanner-nah. Eine gemeinsame Seite braucht abschnittsweises Gating.

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
