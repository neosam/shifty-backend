# Phase 61: PersonService + Auflösung von `sales_person_user` - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-17
**Phase:** 61-personservice-aufl-sung-von-sales-person-user
**Areas discussed:** Verknüpfungs-Semantik, Lösen und Person-Identität, Deploy-Zustand von Phase 60

---

## Verknüpfungs-Semantik — was passiert, wenn beide Seiten schon eine Person haben?

**Ausgangslage, die die Frage erzwingt:** Nach dem Backfill aus Phase 60 hat jede
bestehende Sales Person und jeder bestehende Benutzer bereits eine eigene Person.
Ein Admin, der einen Bestands-Benutzer mit einer Bestands-Sales-Person verknüpft,
zeigt damit zwangsläufig auf zwei verschiedene Personen. Success Criterion 4 der
Roadmap liest sich so, als müsse genau das mit 409 abgelehnt werden.

| Option | Description | Selected |
|--------|-------------|----------|
| Modell A — immer eine Person, Verknüpfen = Merge | `person_id` wird bei jeder Neuanlage vergeben, ist nie NULL. Verknüpfen verschmilzt zwei Personen, die absorbierte Zeile wird gelöscht. 409 nur bei echter Mehrdeutigkeit (die Gegenseite hat ihre Ausprägung schon belegt). | ✓ |
| Modell B — NULL bis zur Verknüpfung | Neuanlagen bekommen keine Person; Verknüpfen zweier NULL-Objekte legt eine gemeinsame Person an; 409 sobald beide Seiten schon auf verschiedene Personen zeigen. Deckt sich wörtlich mit Success Criterion 4. | |

**User's choice:** Modell A.

**Notes:** Ausschlaggebend war der Vergleich mit dem heutigen Verhalten. Heute
löscht `set_user` die alte `sales_person_user`-Zeile und legt die neue an: bei
freiem Benutzer geht das durch, bei belegtem Benutzer schlägt der
`UNIQUE(user_id)`-Constraint zu und der Aufrufer sieht einen 500er. Modell B wäre
ein Funktionsverlust gegenüber heute — Bestandsobjekte ließen sich nie wieder
verknüpfen — und machte PADM-04 (Phase 63, „zwei bereits getrennt bestehende
Objekte ausnahmsweise verknüpfen") für alle Bestandsdaten unmöglich. Modell A
erhält den funktionierenden Fall exakt und ersetzt den 500er durch einen sauberen,
unterscheidbaren 409.

Zu Success Criterion 4: Modell A erfüllt es beobachtbar („Verknüpfen zweier noch
unverknüpfter Objekte legt genau eine gemeinsame Person an") — der Ergebniszustand
ist identisch, nur der Mechanismus ist Verschmelzen statt Neuanlegen.
Folgeentscheidung ohne eigene Frage: „unverknüpft" heißt damit nicht
`person_id IS NULL`, sondern „die eigene Person trägt die Gegenseite nicht"
(D-61-02).

---

## Lösen — wer behält die Person?

**Ausgangslage:** `set_user(sp, None)` trennt heute nur die Verknüpfungszeile,
beide Objekte überleben. Im Person-Modell muss eine Seite eine neue Identität
bekommen.

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Sales Person behält | Die Sales Person behält die bestehende Person, der Benutzer bekommt eine frisch angelegte. | ✓ |
| (b) Benutzer behält | Umgekehrt — der Benutzer behält die bestehende Person. | |
| (c) `person_id = NULL` | Die abgetrennte Seite wird wieder zum „unverknüpften Kandidaten" mit NULL. | |

**User's choice:** (a).

**Notes:** Begründung: an `sales_person.id` hängen Buchungen, `extra_hours` und
write-once-Abrechnungs-Snapshots — das ist die Seite mit historischem Gewicht,
ihre Person-Identität soll stabil bleiben. Nebenwirkung, die (a) besonders
attraktiv macht: es entsteht nie eine Person ohne Träger, womit Pitfall 5 Fall 1
(„unlink to zero" → Ghost-Einträge in der Personen-Liste) strukturell
ausgeschlossen ist — eine Produktfrage weniger für Phase 62. (c) wurde verworfen,
weil es NULLs wieder einführt, die D-61-01 gerade beseitigt.

---

## Deploy-Zustand von Phase 60

**Ausgangslage:** Zeilen, die zwischen dem Phase-60- und dem Phase-61-Deploy
entstehen, hätten laut D-60-10 noch kein `person_id`; ihre Verknüpfung lebte nur
in `sales_person_user` und ginge beim DROP still verloren.

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 60 bereits produktiv | Nachzieh-Backfill + Selbst-Assertion **und** eigene Fixture-Testmatrix für den Zwischenzustand. | |
| Phase 60 noch nicht deployt | Nachzieh-Backfill + Selbst-Assertion bleiben als billige Absicherung, aber ohne eigene Fixture-Matrix. | ✓ |

**User's choice:** Noch nicht deployt — Phase 60 und 61 gehen zusammen live.

**Notes:** Der Guard bleibt trotzdem drin: er kostet fast nichts, und die
Entwicklungs-Datenbanken haben den Zwischenzustand real durchlaufen. Ein Test, der
den Zustand herstellt und nachweist, dass der Nachzug greift, genügt (D-61-06).

---

## Claude's Discretion

Vom User ausdrücklich an Claude delegiert („technische Details entscheide ich
selbst"):

- `ServiceError`-Variante für den Verknüpfungs-Konflikt und ihr 409-Mapping —
  einzige Vorgabe: unterscheidbar von `EntityConflicts` (Versions-Konflikt).
- API-Zuschnitt von `PersonDao`/`PersonService` — keine Vorratshaltung für 62/63.
- Ob Merge- und Konflikt-Logik im DAO oder im `PersonService` sitzen.
- Migrations-Dateiname, Index-Namen, Form der Selbst-Assertion, Test-Ablage.
- Ob `BasicDaoImpl::clear_all` zusätzlich `DELETE FROM person` bekommt.
- Wellen-Schnitt, Task-Granularität und Plan-Anzahl.

## Deferred Ideas

- Merge-Bedienoberfläche für zusammengeführte Personen — Mechanik entsteht hier,
  UI nicht; kein v2.7-Scope.
- Kandidaten-Picker-UI (PADM-04) — Lesepfade hier, Oberfläche Phase 63.
- Person löschen inkl. `deleted`-Spalte — frühestens Phase 63.
- Echte Personenstammdaten an `person` — PERS-F-01, Future.
- `person_id` schema-seitig auf `NOT NULL` ziehen — bräuchte Table-Rebuild in
  SQLite; Invariante wird vorerst vom Code getragen.
- `FOREIGN KEY`-Deklaration auf den `person_id`-Kanten — offene Frage aus Phase 60.

## Reviewed Todos (nicht eingefaltet)

`todo.match-phase` meldete drei Treffer mit Score 0.9, alle Keyword-Zufallstreffer
über „auth"/„user". Keiner wurde eingefaltet; der User hat der Einschätzung nicht
widersprochen.

- `.planning/todos/pending/2026-05-08-admin-rolle-bekommt-alle-privilegien.md` —
  Rollen-/Privilegien-Modell, kein Bezug zur Person-Verknüpfung.
- `.planning/todos/pending/2026-05-08-cutover-ui-admin-feature.md` —
  Frontend-Feature, diese Phase ist backend-only.
- `.planning/todos/pending/2026-07-03-schmale-admin-rolle-mit-user-impersonation.md`
  — würde genau das Self-Gate verändern, dessen Nicht-Regression diese Phase
  nachweisen muss. Bewusst getrennt gehalten.
