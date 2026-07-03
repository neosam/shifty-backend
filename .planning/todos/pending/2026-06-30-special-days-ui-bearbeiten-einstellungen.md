---
created: 2026-06-30T00:00:00
title: Special Days (Feiertage) in der UI bearbeiten — Sektion in den Einstellungen
area: frontend / settings / special-days
files:
  - shifty-dioxus/src/page/settings.rs
  - shifty-dioxus/src/api.rs
  - shifty-dioxus/src/loader.rs
  - rest/src/special_day.rs
---

## Problem

Special Days (Feiertage / Kurztage, `special_day`-Tabelle) lassen sich aktuell
**nicht über die UI pflegen**. Das Frontend kann sie nur **lesen**
(`get_special_days_for_week`, `api.rs:974`), aber nicht anlegen oder löschen.
Sie müssen heute direkt in der DB gesetzt werden — unpraktisch, gerade weil die
Feiertags-Auto-Anrechnung (Phase 25) genau auf diesen Einträgen aufbaut.

User-Wunsch (2026-06-30): Special Days in der UI bearbeitbar machen, z. B. als
**neue Sektion in den Einstellungen** (`/settings`).

### Ist-Stand (verifiziert)

Backend bietet bereits CRUD (`rest/src/special_day.rs`):
- `GET /special-days/for-week/{year}/{week}` (lesen)
- `POST /special-days/` (`create_special_days`)
- `DELETE /special-days/{id}` (`delete_special_day`)

→ Es fehlt **nur** die Frontend-Seite; die REST-Endpoints existieren schon
(create/delete sind noch nicht im Frontend verdrahtet).

`SpecialDay`-Felder: `year`, `calendar_week`, `day_of_week`, `day_type`
(`Holiday` | `ShortDay`), `time_of_day` (nur für ShortDay relevant).

## Solution

TBD — grobe Richtung:

- Neue Sektion/Card auf der bestehenden Settings-Seite
  (`shifty-dioxus/src/page/settings.rs`), analog zum Phase-24/25-Muster
  (admin-gated Cards). Berechtigung klären: vermutlich admin / shiftplanner.
- Frontend-API ergänzen: `create_special_day` / `delete_special_day` in
  `api.rs` + `loader.rs` (POST/DELETE gegen die bereits existierenden
  Endpoints).
- Eingabe-Modell: Der Benutzer wählt und sieht ein **übliches Kalenderdatum**,
  KEINE KW+Wochentag-Eingabe. Beispiel: Auswahl `15.08.2026`.
  - **Eingabe:** normaler Datepicker / Datumsfeld (konkretes Datum).
  - **Anzeige:** das Datum im locale-üblichen Format, gefolgt vom
    abgeleiteten Kontext in Klammern. Beispiel (de):
    `15.08.2026 (Samstag, KW 33, 2026)`. Format pro Locale anpassen
    (de `TT.MM.JJJJ`, en `MM/DD/YYYY` o. ä.); Wochentag + KW + Jahr
    werden aus dem Datum berechnet und mit übersetzt.
  - **Mapping intern:** Datum → `(year, iso_week, weekday)` für die
    `SpecialDay`-Persistenz (vgl. `time::Date::from_iso_week_date` /
    `as_shifty_week`); KW/Wochentag sind reine Ableitung, nicht vom User
    einzugeben.
  - Achtung WASM-Datepicker-Caveat (Phase 25, D-25-06): programmatisches
    Setzen von `<input type=date>` triggert Dioxus-Signale nicht zuverlässig
    → Persistenz-/Anzeige-Loop im echten Browser verifizieren.
- `day_type`-Auswahl (Holiday / ShortDay); bei ShortDay zusätzlich
  `time_of_day`-Feld einblenden.
- Listenansicht der vorhandenen Special Days (z. B. pro Jahr/KW) mit
  Lösch-Möglichkeit. Bedienung über mehrere Wochen hinweg bedenken
  (`for-week` ist wochenweise — evtl. Range-Iteration oder neuer
  Read-Endpoint nötig).
- i18n de/en/cs für alle neuen Labels.
- Tests: Frontend-API-Roundtrip (create → for-week → delete) + WASM-Build-Gate.

Bezug: Voraussetzung/Komfort-Feature für die Feiertags-Automatik aus Phase 25;
verwandt mit [2026-06-30-feiertag-soll-abzug-schichtplan-tabelle].

Zugehörige Phase: noch nicht angelegt.
