# Phase 10: Shiftplan-View Unavailability-Marker - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-19
**Phase:** 10-shiftplan-view-unavailability-marker
**Areas discussed:** Loading-Strategie, Visual-Placement, Farb-/Border-Mapping, Both-State Aufräum-Button, i18n
**Mode:** --text --batch

---

## ⚖️ Post-Discussion-Revision (2026-06-19) — DSGVO Art. 9

Nach Abschluss der Discussion brachte der User eine **Datenschutz-Revision** ein: Die
Wochenansicht ist für alle Schichtplaner sichtbar; `SickLeave` ist ein Gesundheitsdatum
(DSGVO Art. 9). Der Marker darf den **Grund** einer Abwesenheit **nicht** anzeigen.

**Auswirkung auf die Decisions:**
- **D-07 revidiert:** ein einziger neutraler Chip-Stil statt Kategorie-Token (keine
  `CategoryBadge`-Farben).
- **D-08 revidiert:** `Both` rendert identisch zu den anderen (keine eigene Indikation,
  keine Kategorie-Farbe).
- **D-11 revidiert:** ein einziger neutraler i18n-Key „nicht verfügbar" statt Kategorie-/
  Both-Texte.
- **D-12 neu (governing):** Kategorie/Grund wird in der Wochenansicht nie angezeigt;
  überschreibt ROADMAP-Phase-10 SC 2 + SC 3.
- Unverändert: D-01..D-06 (Loading + Placement), D-09 (jetzt festzurren), D-10 (Aufräum-Button deferred).

Die Tabellen unten dokumentieren die **ursprünglich** gewählten Optionen; maßgeblich ist
die revidierte Fassung in CONTEXT.md.

---

## Loading-Strategie

| Option | Description | Selected |
|--------|-------------|----------|
| (a) | Bestehenden `load_unavailable_sales_person_days_for_week` umbauen → per-sales-person-Endpoint, kleinster Blast-Radius | ✓ |
| (b) | Haupt-`load_shift_plan` bei ausgewählter Person komplett auf per-sales-person-Endpoint umstellen | |

**User's choice:** (a) — D-01
**Notes:** Globaler `load_shift_plan` bleibt unverändert; nur die Marker-Quelle wandert auf den per-sales-person-Endpoint.

### Verhalten ohne ausgewählte Person

| Option | Description | Selected |
|--------|-------------|----------|
| (a) | Keine Marker anzeigen (Grid wie heute) | ✓ |
| (b) | Etwas anderes | |

**User's choice:** (a) — D-03

---

## Visual-Placement im Time-Grid

| Option | Description | Selected |
|--------|-------------|----------|
| (a) | Chip im Tag-Spalten-Header + bestehende Spalten-Tönung beibehalten | ✓ |
| (b) | Nur die `discourage`-Spalten-Tönung umfärben (kein Chip) | |
| (c) | Beides: Chip + kategorie-gefärbte Spalte | (initial) |

**User's choice:** zunächst (c), nach Klärung des Widerspruchs mit der Tönungs-Frage final (a) — D-04
**Notes:** Konflikt 3c↔4a aufgelöst zugunsten „Chip = Kategorie-Farbe, Spalte = neutral bad-soft" (siehe nächster Abschnitt).

### Bedeutung der Spalten-Tönung

| Option | Description | Selected |
|--------|-------------|----------|
| (i) | Chip = Kategorie-Farbe, Spalte = Kategorie-Soft-Farbe getönt | |
| (ii) | Chip = Kategorie-Farbe, Spalte bleibt neutral bad-soft (rot) | ✓ |
| (iii) | Spalte Kategorie-Farbe + rötlicher Akzent bei manual/both | |

**User's choice:** (ii) — D-05
**Notes:** Concern-Trennung: Spalte = „nicht buchen", Chip = „warum". `discourage`-Mechanismus bleibt strukturell.

---

## Farb-/Border-Mapping der drei States

| Option | Description | Selected |
|--------|-------------|----------|
| (a) | Bestehende `CategoryBadge`-Tokens wiederverwenden (STATIC classes, Pitfall-5); manual = neutral + dashed | ✓ |
| (b) | Neue eigene Tokens definieren | |

**User's choice:** (a) — D-07

### Both-State Visual-Indikation

| Option | Description | Selected |
|--------|-------------|----------|
| (a) | Mockup-Semantik: Kategorie-Soft-BG + dashed Border in Kategorie-Farbe + Redundanz-Indikator (`!`) | ✓ |
| (b) | Anders | |

**User's choice:** (a) — D-08

### Final-Farben jetzt oder UI-SPEC

| Option | Description | Selected |
|--------|-------------|----------|
| (a) | Jetzt festzurren (nur bestehende Tokens, kein neues Design-Asset) | ✓ |
| (b) | `/gsd-ui-phase 10` für UI-SPEC laufen lassen | |

**User's choice:** (a) — D-09

---

## Both-State Aufräum-Button

| Option | Description | Selected |
|--------|-------------|----------|
| (a) | Nur visuelle Indikation + Tooltip; Aufräum-Aktion deferred | ✓ |
| (b) | Button mit rein (löscht manuellen Eintrag, Dioxus-Dialog) | |

**User's choice:** (a) — D-10
**Notes:** Mockup hat keinen Button; ROADMAP SC3 nennt ihn „optional". Deferred → siehe CONTEXT.md `<deferred>`.

---

## i18n

| Option | Description | Selected |
|--------|-------------|----------|
| (a) | Kategorie-Namen via bestehende `Key::AbsenceCategory*`; nur 2–3 neue Keys (manual + both-tooltip), De/En/Cs | ✓ |
| (b) | Mehr/andere Texte | |

**User's choice:** (a) — D-11

---

## Claude's Discretion

- Exakte Chip-Geometrie (Höhe/Padding/Icon) + genaue Tooltip-Wortwahl.
- Ob `UnavailabilityChip` als eigenes Rust-Atom oder inline in `DayView` (Plan-Phase-Entscheidung).

## Deferred Ideas

- **`Both`-Aufräum-Button** — Löschen des redundanten manuellen `sales_person_unavailable`-Eintrags
  direkt aus dem Marker. Kandidat für spätere Cutover-Cleanup-Phase oder Phase 12.
