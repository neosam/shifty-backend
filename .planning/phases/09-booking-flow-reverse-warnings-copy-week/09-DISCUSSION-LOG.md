# Phase 9: Booking-Flow Reverse-Warnings + Copy-Week - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-11
**Phase:** 09-booking-flow-reverse-warnings-copy-week
**Areas discussed:** Confirm-Dialog-Semantik, Copy-Week-Trigger & -Scope, Aggregierte Warnings-Anzeige, Warning-Component & Texte, Rollback-Fehlerbehandlung, Dialog-Wording & Buttons, Test-Strategie / SSR, 403/422-Verhalten

---

## Confirm-Dialog-Semantik

| Option | Description | Selected |
|--------|-------------|----------|
| Optimistic + Rollback | POST bucht sofort; bei Warnings → Dialog; "Abbrechen" → DELETE der Buchung, "Bestätigen" behält | ✓ |
| Acknowledge-only | POST bucht sofort; Dialog nur mit "Verstanden", kein Rollback | |

**User's choice:** Optimistic + Rollback
**Notes:** Der Endpoint ist kein Dry-Run (create-then-warn). Optimistic+Rollback kommt "Confirm vor finaler Buchung" am nächsten — Planer kann zurück. `api::remove_booking` (api.rs:227) existiert, `BookingTO.id` liefert die Rollback-ID.

---

## Copy-Week-Trigger & -Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Vorwoche → aktuell, Button aktivieren | Dormanten Pfad reaktivieren + sichtbar machen | |
| Nur Endpoint umstellen, Button dormant | Logik umstellen, kein Button | |
| From/To-Wochen-Picker | Beliebige Quell-/Ziel-Woche | |
| (Other / free text) | "Was hat der Copy-week Button auf sich? Den habe ich extra mal entfernt." | ✓ |

**User's choice:** Freitext — User hat den Copy-Week-Button bewusst entfernt.
**Notes:** Historie bestätigt: Commit `294566f` "feat: Remove copy last week feature". Übrig blieb toter Code (Action-Variante + Handler, kein Button, ungenutzter i18n-Key). Führte zur Folge-Entscheidung über den Phasen-Scope (siehe nächster Block).

---

## Copy-Week Scope-Auflösung (FUI-A-06)

| Option | Description | Selected |
|--------|-------------|----------|
| Copy-Week descopen | Aus Phase 9 streichen, FUI-A-06 dropped, toten Code aufräumen, ROADMAP/REQUIREMENTS nachziehen | ✓ |
| Wieder einführen (konflikt-bewusst) | Feature reaktivieren auf neuem Endpoint | |
| Nur Cleanup, kein Feature | Descopen + Code jetzt entfernen | |

**User's choice:** Copy-Week descopen
**Notes:** Bereiche "Copy-Week-Trigger" und "aggregierte Warnings-Anzeige" damit gegenstandslos. Backend-Endpoint + `CopyWeekResultTO` bleiben funktionsfähig. ROADMAP-Titel/SC + REQUIREMENTS FUI-A-06 vom User autorisiert nachzuziehen.

---

## Warning-Component & Texte

| Option | Description | Selected |
|--------|-------------|----------|
| Neue geteilte Warning-Komponente | component/warning_list.rs, kennt alle Varianten | |
| Booking-spezifische Inline-Komponente | Inline in shiftplan.rs, minimaler Blast-Radius | |
| AbsenceWarningDisplay erweitern + teilen | Bestehende erweitern, aus absences.rs nach component/ herauslösen | ✓ |

**User's choice:** AbsenceWarningDisplay erweitern + teilen

| Option (Text-Detail) | Description | Selected |
|--------|-------------|----------|
| Datum + Grund, ohne Personname | Person redundant im Single-Booking-Flow | |
| Person + Datum + Grund | Voll inkl. Mitarbeitername | ✓ |
| Kompakt, eine Zeile pro Warning | Knappste Variante | |

**User's choice:** Person + Datum + Grund
**Notes:** Erfordert person_name-Auflösung (Side-Join-Pattern Plan 08-04). Nur Booking-Pfad-Varianten relevant: BookingOnAbsenceDay, BookingOnUnavailableDay, PaidEmployeeLimitExceeded.

---

## Rollback-Fehlerbehandlung

| Option | Description | Selected |
|--------|-------------|----------|
| Error-Toast + Reload | Fehler über error_handler + Reload (echter Zustand sichtbar) | ✓ |
| Still + Reload | Fehler schlucken, nur Reload | |
| Retry-once, dann Toast | Einmal retry, dann Toast | |

**User's choice:** Error-Toast + Reload

---

## Dialog-Wording & Buttons

| Option | Description | Selected |
|--------|-------------|----------|
| "Trotzdem buchen" / "Abbrechen" + Sing/Plural-Header | Handlungsbezogen, Header mit {count} analog AbsenceWarning | ✓ |
| "Bestätigen" / "Verwerfen" | Neutraler | |
| Du entscheidest beim Planen | Wording dem Plan überlassen | |

**User's choice:** "Trotzdem buchen" / "Abbrechen" + Singular/Plural-Header

---

## Test-Strategie / SSR

| Option | Description | Selected |
|--------|-------------|----------|
| Voller Satz (analog Phase 8) | SSR-Snapshots je Variante, leeres-Array, Rollback-Dispatch, Per-Locale-Matcher | ✓ |
| Minimal | Nur Render + leeres-Array | |

**User's choice:** Voller Satz (analog Phase 8)

---

## 403/422-Verhalten

| Option | Description | Selected |
|--------|-------------|----------|
| 403 still, 422 als Error | 403 still schlucken (wie Bestand), 422 über error_handler | ✓ |
| 403 dezenter Hinweis | Toast bei 403 | |

**User's choice:** 403 still, 422 als Error

---

## Claude's Discretion

- Exaktes Tailwind-Styling des Dialogs.
- Name der geteilten Komponente (warning_list.rs vs. AbsenceWarningDisplay umbenennen/verschieben).
- Genaue i18n-Key-Namen.
- Ob `api::add_booking` erweitert oder neue Funktion angelegt wird.
- Wave-/Plan-Aufteilung (kleine Phase, vermutlich 1–2 Plans).

## Deferred Ideas

- Copy-Week mit Konflikt-Awareness (UI-Reaktivierung): Backend bleibt funktionsfähig; eigene Phase falls je gewünscht. FUI-A-06 vorerst dropped.
- Optionale Migration von absences.rs auf die geteilte Warning-Komponente über das Nötige hinaus.
