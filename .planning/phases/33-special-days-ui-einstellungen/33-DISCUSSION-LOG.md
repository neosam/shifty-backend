# Phase 33: Special-Days-UI in den Einstellungen - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-30
**Phase:** 33-special-days-ui-einstellungen
**Areas discussed:** Berechtigung / Gate-Abgleich, Listen-Scope (Multi-Woche), Eingabe-Flow & ShortDay, Card-Platzierung & Listen-Format

---

## Berechtigung / Gate-Abgleich

| Option | Description | Selected |
|--------|-------------|----------|
| admin durchgängig (BE ändern) | Card auf admin-SettingsPage, Backend create/delete admin-only | |
| shiftplanner durchgängig | Backend bleibt shiftplanner, Card an shiftplanner-erreichbaren Ort | (✓ resultierend) |
| Beide dürfen (admin ∨ shiftplanner) | Backend + FE auf admin ODER shiftplanner | |

**User's choice:** Freitext-Rückfrage „Gibt es nicht eine Rolle, die die Struktur des Schichtplans ändern kann?"
**Notes:** Code-Verifikation: Slot-Struktur-CRUD (`slot.rs:211/269/293`) und Special-Day-CRUD gaten beide auf `SHIFTPLANNER_PRIVILEGE = "shiftplanner"`. Ergebnis: **shiftplanner durchgängig**, kein Backend-Change. SPD-04 „admin-gated" als ungenaue Annahme korrigiert (D-33-01/02).

---

## Platzierung (Folgefrage)

| Option | Description | Selected |
|--------|-------------|----------|
| Schichtplan-Seite | Sektion auf shiftplan.rs (shiftplanner-gated) | ✓ |
| Eigene shiftplanner-Settings-Seite | Neue Route nur für Struktur-Pflege | |
| Settings-Seite, Card shiftplanner-gated | Seiten-Gate lockern, per-Card gaten | ✓ |

**User's choice:** Freitext — „Schichtplan Seite UND Settings Seite. Schichtplanseite weil man da die Struktur ohnehin ändert (muss interaktiv gehen), Settings Seite für kompakte Übersicht."
**Notes:** Zwei Oberflächen für dieselbe Capability (D-33-03/04).

---

## Aufgabenteilung der Flächen (Folgefrage)

| Option | Description | Selected |
|--------|-------------|----------|
| Schichtplan=anlegen+löschen, Settings=Übersicht+löschen | Datepicker vermeidbar | |
| Beide Flächen voll-CRUD | Schichtplan interaktiv + Settings Datepicker-Create | ✓ |
| Settings=voll-CRUD, Schichtplan=nur Anzeige | Näher am Todo | |

**User's choice:** Beide Flächen voll-CRUD
**Notes:** Datepicker-Caveat (D-25-06) wird auf der Settings-Fläche gelöst.

---

## Listen-Scope (Multi-Woche)

| Option | Description | Selected |
|--------|-------------|----------|
| Neuer Range/Jahr-Read-Endpoint (BE) | Ein Request füllt die Liste | ✓ |
| FE iteriert KW 1..53 pro Jahr | Kein BE-Change, ~53 Requests | |
| Jahr-Picker + rollendes Fenster | Begrenztes Fenster, weniger Requests | |

**User's choice:** Neuer Range/Jahr-Read-Endpoint (BE)
**Notes:** Hebt das in REQUIREMENTS deferierte „Multi-Wochen-Read-Endpoint"-Item in diese Phase (D-33-05).

---

## Eingabe-Flow & ShortDay

| Option | Description | Selected |
|--------|-------------|----------|
| ShortDay-Uhrzeit Pflicht; Duplikat blocken | Uhrzeit required, kein zweiter Eintrag am selben Tag | ✓ |
| ShortDay-Uhrzeit Pflicht; Duplikat erlauben | Mehrere Einträge erlaubt | |
| Du entscheidest | Discretion | |

**User's choice:** ShortDay-Uhrzeit Pflicht; Duplikat blocken
**Notes:** Inline-Hinweis bei Duplikat am selben (Jahr,KW,Wochentag) (D-33-06/07).

---

## Interaktion auf der Schichtplan-Seite

| Option | Description | Selected |
|--------|-------------|----------|
| Per-Tag-Affordance im Wochenraster | Icon/Menü pro Tag | ✓ (als Dropdown) |
| Wochen-Sektion unter dem Raster | Card unter dem Raster | |
| Du entscheidest (UI-Phase) | Discretion | |

**User's choice:** Freitext — „So ein Dropdown-Button, mit dem man generell Einstellungen für einen Tag machen kann. Aktuell geht halt nur Feiertag, kurzer Tag oder nichts."
**Notes:** Generisches „Tag-Einstellungen"-Dropdown pro Wochentag, Optionen Feiertag/Kurzer Tag/Nichts (= entfernen); erweiterbar gedacht (D-33-03).

---

## Card-Platzierung & Listen-Format

| Option | Description | Selected |
|--------|-------------|----------|
| Chronologisch, nach Jahr gruppiert | Jahr-Picker, Gruppierung, Badge, Empty-State | ✓ |
| Flache chronologische Liste | Ohne Gruppierung | |
| Du entscheidest | Discretion | |

**User's choice:** Chronologisch, nach Jahr gruppiert
**Notes:** Typ-Badge, ShortDay zeigt Uhrzeit, Empty-State; Format SPD-02 (D-33-08).

---

## Claude's Discretion

- Exakte Seiten-Gate-Verdrahtung (Page-Gate lockern vs. Card-Guard), solange FE = shiftplanner.
- Name/Pfad + Read-Permission des neuen Range/Jahr-Read-Endpoints.
- Ob Duplikat-Prüfung zusätzlich serverseitig.
- Konkretes UI-Layout (Dropdown, Form, Badge, Empty-State); i18n-Texte de/en/cs.

## Deferred Ideas

- ShortDay-Soll-Automatik im Report (Future).
- Hover-Tooltip auf Feiertags-Zelle (Phase-34-Differentiator).
- Weitere „Tag-Einstellungen" im Dropdown über Special Days hinaus.
</content>
