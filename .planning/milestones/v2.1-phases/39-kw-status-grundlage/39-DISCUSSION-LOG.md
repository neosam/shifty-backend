# Phase 39: KW-Status Grundlage — Discussion Log

**Date:** 2026-07-01
**Mode:** discuss (text-style, per user preference)

> Human reference only — nicht von downstream-Agenten (researcher/planner/executor) konsumiert.

## Präsentierte Graubereiche

1. Berechtigung & Status-Übergänge
2. Leer-Status: Enum-Name + Persistenz-Modell
3. UI-Muster der Set-Aktion
4. Badge-Farbkodierung + Labels

## Verlauf

- **User wählte aktiv Bereich 3 (UI)** und legte fest: Nicht-Schichtplaner sehen das Badge nur bei gesetztem Status; bei `Unset` gar nichts. Schichtplaner erhält ein Dropdown zur Auswahl, positioniert oberhalb der Wochenansicht. „Der Rest ist Default."
- **Bereiche 1, 2, 4** → Claude setzte begründete Defaults und legte sie dem User zur Bestätigung vor:
  - 1: nur Schichtplaner, alle Übergänge frei, kein Entsperr-Sondergate.
  - 2: Enum-Variante `Unset` (nicht `None`/`Open`); Persistenz = Zeilen-Abwesenheit (keine Zeile = `Unset`, Reset = soft-delete).
  - 4: Gesperrt=rot / Geplant=grün / In Planung=amber / Unset=grau; Labels de/en/cs tabelliert.
- **User: „Passt."** → alle Entscheidungen bestätigt (D-39-01 … D-39-12).

## Deferred (redirected)

- Sperr-Durchsetzung → Phase 40 (bereits geroadmappt).
- Bulk-Status → WST-06 (v2-Backlog). Publish-Notification → WST-07 (v2-Backlog).
