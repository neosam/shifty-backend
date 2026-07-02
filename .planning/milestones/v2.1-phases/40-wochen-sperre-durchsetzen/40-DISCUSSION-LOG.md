# Phase 40: Wochen-Sperre durchsetzen (BE+FE) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-01
**Phase:** 40-Wochen-Sperre-durchsetzen
**Areas discussed:** HTTP-Status-Code, Frontend-Durchsetzung, Umfang der Sperre (Selbst-Ausbuchen), Banner/Hinweis

---

## HTTP-Status-Code für Locked-Write

| Option | Description | Selected |
|--------|-------------|----------|
| 423 Locked | Semantisch exakt; erster seiner Art im Codebase; klar in OpenAPI | ✓ |
| 409 Conflict | Konsistent mit allen bestehenden Conflict-Fehlern (PaidLimitExceeded etc.) | |

**User's choice:** 423.
**Notes:** Scout zeigte: alle bestehenden Conflict-Fehler mappen auf 409, 423 existiert nirgends. Da das FE den Sperr-Zustand direkt am Wochen-Status abliest, ist der HTTP-Fehler reines Sicherheitsnetz; semantische Präzision überwog. Kosten null (Match-Arm ohnehin compiler-erzwungen). → D-40-01.

---

## Frontend-Durchsetzung der Read-only-Woche

| Option | Description | Selected |
|--------|-------------|----------|
| Proaktiv Controls deaktivieren/ausblenden | +/- Buttons für Nicht-Schichtplaner in Gesperrt-Woche ausblenden | ✓ |
| Nur reaktiv (Banner bei 423) | Optimistisch lassen, erst auf Server-Fehler reagieren | |
| Beides | Controls disabled + Banner-Fallback | |

**User's choice:** „Controls deaktivieren oder einfach die + und - Buttons ausblenden." → +/- Buttons ausblenden.
**Notes:** Server-Gate bleibt die eigentliche Durchsetzung; Ausblenden ist reine UX (schützt nicht gegen direkte API-Calls). → D-40-03.

---

## Umfang der Sperre — inkl. Selbst-Ausbuchen?

| Option | Description | Selected |
|--------|-------------|----------|
| Harte Sperre inkl. Entfernen | Auch eigenes Ausbuchen/Buchung-Löschen blockiert | ✓ |
| Ausnahme fürs Selbst-Ausbuchen | Nutzer darf sich weiterhin selbst aus der Gesperrt-Woche entfernen | |

**User's choice:** „Soll auch für das Entfernen gelten."
**Notes:** Bewusste Konsequenz: Nutzer kommt ohne Schichtplaner nicht mehr aus einer gesperrten Woche. Betrifft den neuen `delete_booking`-Pfad. → D-40-02.

---

## Banner / Hinweis bei Gesperrt-Woche

| Option | Description | Selected |
|--------|-------------|----------|
| A) Nur Phase-39-Badge + ausgeblendete Buttons | Kein zusätzlicher Hinweis | ✓ |
| B) Zusätzlicher informativer Inline-Hinweis | „Diese Woche ist gesperrt …" permanent oben | |
| C) Reaktiver Toast/Streifen nur bei 423 | Sicherheitsnetz-Meldung | |

**User's choice:** „Es braucht keinen Hinweis. Finde A auch gut."
**Notes:** Rotes „Gesperrt"-Badge aus Phase 39 + fehlende Buttons sind selbsterklärend; zusätzlicher Banner wäre Doppelung. SC1-Formulierung „Inline-Banner bei 423" bewusst auf „Badge + ausgeblendete Buttons" reduziert. → D-40-04.

---

## Claude's Discretion

- Name/Signatur/Ort des `assert_week_not_locked`-Helpers (freie Funktion vs. Methode), solange in derselben Transaktion.
- Signatur/Verhalten der neuen `ShiftplanEditService::delete_booking` (BookingService::delete-Semantik erhalten + Lock-Gate).
- Verdrahtung des Week-Status-Read als Dep in `ShiftplanEditServiceDeps`.
- Wortlaut der de/en/cs-Sperr-Meldung.

## Deferred Ideas

- Bulk-KW-Sperre → WST-06 (Backlog).
- Publish-Notification → WST-07 (Backlog).
- Sperre weiterer Nicht-Shiftplan-Schreibpfade (Absence/Unavailable) → außerhalb dieser Phase.
