# Phase 62: Personen-Verzeichnis — eine Liste statt Tabs - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-19
**Phase:** 62-Personen-Verzeichnis — eine Liste statt Tabs
**Areas discussed:** Navigation aus der Liste, Umfang der Listen-Aktionen, Darstellung bei zwei Namen, Sichtbarkeit und Soft-Delete

---

## Navigation aus der Liste (→ D-62-07)

| Option | Description | Selected |
|--------|-------------|----------|
| Bestehende Detail-Routen beibehalten | Zeile verlinkt je nach Ausprägung auf `SalesPersonDetails` bzw. `UserDetails`; Phase 63 ersetzt beides durch eine Route | ✓ |
| Leere `/person/{id}`-Hülle schon in 62 | Neue Route wird angelegt, aber erst in 63 gefüllt | |

**User's choice:** Default (bestehende Routen beibehalten).
**Notes:** „Muss man dann sehen" — die endgültige Navigationsform entsteht mit der Detailansicht in Phase 63; Phase 62 investiert nichts in eine Route, die 63 sofort umbaut.

---

## Umfang der Listen-Aktionen (→ D-62-06)

| Option | Description | Selected |
|--------|-------------|----------|
| Alle heutigen Aktionen erhalten | „Neuen Benutzer anlegen", Löschen mit Bestätigung, „Act as this person", Rollen-Spalte wandern in die vereinte Liste — sichtbar nur bei Benutzer-Ausprägung | ✓ |
| Liste in 62 rein lesend | Aktionen erst mit Phase 63; zwischenzeitlich fehlt das Benutzer-Anlegen | |

**User's choice:** Default (alle Aktionen erhalten).
**Notes:** Die Seite ist produktiv — ein Umbau darf zwischen Phase 62 und 63 keine Funktionslücke reißen.

---

## Darstellung bei zwei Namen (→ D-62-03)

| Option | Description | Selected |
|--------|-------------|----------|
| `display_name` + Username als eigene Spalte | Verkäufername als Zeilen-Titel, Username zusätzlich sichtbar; Rollen-Spalte bleibt | ✓ |
| Nur Anzeigename + Ausprägungs-Badge | Schlanker, verliert aber Username und Rollen aus der Übersicht | |

**User's choice:** Freitext — „Vielleicht sollte irgendwie beides angezeigt werden."
**Notes:** Als Entscheidung gepinnt: das TO trägt `display_name` **und** `username` als getrennte Felder, beide werden gerendert. Bei einem reinen Benutzerkonto tragen beide denselben Wert; das Frontend unterdrückt nichts und leitet nichts ab (PADM-06 / Fat Backend).

---

## Sichtbarkeit und Soft-Delete (→ D-62-04, D-62-05)

| Option | Description | Selected |
|--------|-------------|----------|
| Aktive + inaktive Verkäufer zeigen, soft-gelöschte nicht; WR-02 hier mitfixen | Heutiges Sichtbarkeitsverhalten bleibt; der Soft-Delete-Filter wandert in die `ON`-Klausel des Kandidaten-Joins, plus Test und Doku-Korrektur | ✓ |
| WR-02 auf Phase 63 schieben | Fix erst dort, wo der Kandidaten-Picker gebaut wird | |

**User's choice:** Default (WR-02 hier mitfixen).
**Notes:** Begründung für den Ort: derselbe Outer-Join entsteht in Phase 62 ohnehin; danach sind beide Kandidatenpfade symmetrisch. Ohne den Fix fällt ein Benutzer, dessen einzige Sales Person soft-gelöscht ist, aus beiden Kandidatenlisten und ließe sich über den Phase-63-Picker nie wieder verknüpfen.

---

## Claude's Discretion

Vom User nicht aufgemacht, in CONTEXT.md §„Claude's Discretion" ausgeführt:

- Routen-Präfix und Endpoint-Zuschnitt des Aggregat-`GET` (mit Auflage: `Dioxus.toml` im selben Commit)
- Name und Modul-Ablage des Business-Logic-Services; neue DAO-Methode vs. Kombination bestehender
- Serde-Form des `kind`-Enums und die Feldnamen im `PersonEntryTO`
- Sortierung der Liste; optionaler Ausprägungs-Filter neben der Suche
- Ob die Suche im Backend oder im Frontend filtert (FE-Filter vertretbar — reine UI-State-Logik)
- Ob die Rollen im Aggregat mitkommen (Auflage: kein Per-Zeile-Nachladen)
- i18n-Key-Namen; Verbleib von `UserManagementTabBar` / `UserMgmtTab`
- Wellen-Schnitt, Plan-Anzahl, Testablage

Vorab ohne Rückfrage entschieden (in der Präsentation offengelegt):

- **D-62-01** — explizites `kind`-Enum statt `Option`-Ableitung im Frontend (Roadmap-Discuss-Point, SUMMARY-Frage 8)
- **D-62-02** — Lesequelle ist der Outer-Join `sales_person`/`user`, nicht `person` (Roadmap-Discuss-Point bestätigt)

---

## Deferred Ideas

Siehe CONTEXT.md §`<deferred>`. Kein Scope-Creep in dieser Diskussion — alle
Abgrenzungen folgen dem in der Roadmap gesetzten Schnitt zwischen Phase 62
(Lesen) und Phase 63 (Detail + Mutationen).

Sieben Todo-Treffer aus `todo.match-phase 62` wurden geprüft und **nicht**
gefoldet (Keyword-Zufall über „admin"/„frontend"/„user") — Liste in CONTEXT.md
§„Reviewed Todos (not folded)".
