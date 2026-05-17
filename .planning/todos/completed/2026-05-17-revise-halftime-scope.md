---
created: 2026-05-17T00:00:00.000Z
title: Out-of-Scope-Eintrag "Halbtage / Stundenebene" aus PROJECT.md + REQUIREMENTS.md aufräumen
area: docs
files:
  - .planning/PROJECT.md
  - .planning/REQUIREMENTS.md
---

## Problem

`REQUIREMENTS.md` (Zeile 96) und `PROJECT.md` führen aktuell *Halbtage /
Stundenebene für Abwesenheiten* als explizit ausgeschlossen:

```text
| Halbtage / Stundenebene für Abwesenheiten | Backend modelliert nur Ganztage;
Halbtag-Modell wäre Backend-Änderung, kein Frontend-Scope |
```

Diese Entscheidung wurde am 2026-05-17 in einer Explore-Session revidiert —
halbe Urlaubstage werden in Phase 8.3 implementiert (siehe
`.planning/notes/halftime-absence-decision.md` und FUI-A-10 in REQUIREMENTS.md).
Wenn der alte Out-of-Scope-Eintrag stehenbleibt, widerspricht er der neuen
Phase 8.3 und FUI-A-10 — und sorgt in zwei Jahren für Verwirrung.

## Zu tun

1. **`REQUIREMENTS.md`** (Zeile 96): Den Halbtag-Eintrag aus der
   `## Out of Scope`-Tabelle entfernen ODER zu einem revidierten Eintrag
   umformulieren, etwa:

   > ~~Halbtage / Stundenebene für Abwesenheiten~~ → *Revidiert 2026-05-17:
   > Halbtage werden in Phase 8.3 (FUI-A-10) umgesetzt. Stundenebene bleibt
   > out-of-scope.*

   Empfehlung: Stundenebene als separaten Eintrag explizit drinlassen (bleibt
   out-of-scope), nur den Halbtag-Teil rausnehmen.

2. **`PROJECT.md`**: Bullet-Eintrag *"Halbtage / Stundenebene für
   Abwesenheiten (Backend modelliert nur Ganztage)"* analog anpassen — Halbtag
   raus, Stundenebene drin als out-of-scope.

3. **Verweis hinzufügen:** In beiden Dateien einen kurzen Verweis auf
   `.planning/notes/halftime-absence-decision.md` und Phase 8.3 setzen, damit
   die Revisions-Begründung auffindbar bleibt.

## Wann

Vor Beginn der Plan-Phase 8.3 — die Plan-Phase liest beide Dateien als
Quelle der fachlichen Anforderungen und sollte keine widersprüchliche
Out-of-Scope-Aussage finden.

## Verweise

- Decision Log: `.planning/notes/halftime-absence-decision.md`
- Requirement: REQUIREMENTS.md FUI-A-10
- Phase: ROADMAP.md Phase 8.3

## Resolution

- 2026-05-17 — Erledigt:
  - `REQUIREMENTS.md` Out-of-Scope-Tabelle: Halbtag-Teil entfernt, Stundenebene
    als eigenständiger Out-of-Scope-Eintrag mit Verweis auf FUI-A-10 / Phase 8.3
    und Decision Log neu formuliert.
  - `PROJECT.md` Bullet "Bewusst nicht in v1.3": analog umformuliert, Halbtag
    raus, Stundenebene drin als out-of-scope mit Verweis auf FUI-A-10 / Phase 8.3
    und Decision Log.
