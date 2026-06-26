# Phase 23: Frontend — Slot Paid-Capacity UI - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-26
**Phase:** 23-frontend-slot-paid-capacity-ui
**Areas discussed:** Paid-Count in Zelle, Reichweite & Editor-Hinweis, Warn-Optik (Folge-Klärung)

---

## Gray-Area-Auswahl (multiSelect)

| Option | Description | Selected |
|--------|-------------|----------|
| NULL-Eingabe im Editor | Leeres Feld vs. Checkbox vs. 0-Sentinel für „kein Limit" | |
| Warn-Optik Week-View | warn-soft schon für Unterbesetzung belegt; eigene Farbe/Icon/Rahmen? | (indirekt via Folge-Frage) |
| Paid-Count in Zelle | „bezahlt X/Y"-Badge vs. nur Färbung | ✓ |
| Reichweite & Editor-Hinweis | Wer sieht Warnung; Editor-Verhalten bei Limit < Count | ✓ |

---

## Paid-Count in Zelle

| Option | Description | Selected |
|--------|-------------|----------|
| Eigenes Badge 'paid X/Y' | Zusätzliches Badge, nur bei gesetztem Limit | |
| Nur Färbung/Icon | Keine Zahl, nur Farbe/Icon | (~) |
| Bestehendes Badge erweitern | „filled/need · 💶X/Y" in einer Zeile | |

**User's choice:** „Wir brauchen keine Anzeige" — keine numerische Anzeige in der Zelle.
**Notes:** Die Farbe ist das einzige Signal. Damit wurde die Warn-Optik (Farbwahl) zur
entscheidenden Folge-Frage.

---

## Reichweite (wer sieht die Warn-Färbung)

| Option | Description | Selected |
|--------|-------------|----------|
| Nur Shiftplanner | Nur bei is_shiftplanner sichtbar | |
| Alle Nutzer | Alle Rollen sehen die Überschreitungs-Färbung | ✓ |

**User's choice:** Alle Nutzer.
**Notes:** „Das sind in erster Linie bezahlte, die sich zu viel eintragen. Die sollen
sofort den Effekt sehen."

---

## Editor-Hinweis bei Limit < current_paid_count

| Option | Description | Selected |
|--------|-------------|----------|
| Inline-Hinweis, nicht blockierend | Warn-Banner im Editor, Speichern bleibt möglich | ✓ |
| Kein Hinweis | Keine Reaktion im Editor | |

**User's choice:** Inline-Hinweis, nicht blockierend.
**Notes:** Konsistent mit der bekannten Präferenz „Inline-Warnungen statt Dialoge".

---

## Warn-Optik (Folge-Klärung, da Farbe = einziges Signal)

| Option | Description | Selected |
|--------|-------------|----------|
| Eigene Farbe (bad/rot) als Hintergrund | Deutlich anderer Ton als warn-soft; Vorrang bei Doppelzustand | ✓ |
| Roter Rahmen zusätzlich | Bestehender BG bleibt, zusätzlicher Border | |
| Warn-Icon in der Zelle | ⚠️-Icon (zählt als kleine Anzeige) | |

**User's choice:** Eigene Farbe (bad/rot) als Hintergrund.
**Notes:** Paid-Überschreitung erhält Vorrang vor reiner Unterbesetzung (warn-soft).

---

## Claude's Discretion

- NULL-Eingabe im Editor (nicht zur Diskussion gewählt) → Default: leeres Feld = `None`/kein Limit.
- Konkrete Tailwind-`bad`-Tokens, Label-/Hinweis-Texte, Banner-Platzierung.
- Ob die Editor-Validierung `current_paid_count` aus dem Slot-State oder einem eigenen Prop zieht.

## Deferred Ideas

- Numerische Paid-Count-Anzeige in der Zelle — bewusst verworfen.
- Hartes Blockieren des Save bei Limit-Verletzung — bewusst verworfen.
