# Phase 36: Special-Days-Bugfixes (BE+FE) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-01
**Phase:** 36-special-days-bugfixes
**Areas discussed:** SDF-01 Fix-Ort & Semantik, SDF-02 Fix-Ansatz, Abdeckung Settings-Karte, Test-/Verifikations-Strategie

---

## SDF-01 Fix-Ort & Semantik

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Backend-Upsert im create | Existierenden Same-Date-Eintrag atomar ersetzen statt Duplicate-Error; zentral, jeder Caller profitiert | ✓ |
| (b) Neuer Backend-Update/PUT-Endpoint | Sauberste REST-Semantik, aber mehr Fläche (Service+DAO+Route) | |
| (c) Frontend delete-then-create | Minimal, aber 2 Calls, nicht-atomar (Delete-OK/Create-Fail lässt Tag leer) | |

**User's choice:** (a) Backend-Upsert
**Notes:** Root cause bestätigt durch Code-Scout — kein DB-Constraint, sondern Service-Duplicate-Guard (`special_days.rs:142-153`); kein Update-Endpoint existiert. Atomarität passt zur etablierten Re-Point-Regel.

---

## SDF-02 Fix-Ansatz

| Option | Description | Selected |
|--------|-------------|----------|
| (a) SelectInput controlled | value/selected-Binding in shared inputs.rs → Root-Cause, alle Dropdowns immun | ✓ |
| (b) sd_type nach Create behalten | Minimal, lokal, nur Datum neu | |

**User's choice:** (a) controlled SelectInput
**Notes:** Scout bestätigte uncontrolled `<select>` (`inputs.rs:82-102`, kein value/selected). Optionales value-Prop wahrt Rückwärtskompatibilität.

---

## Abdeckung Settings-Karte (SDF-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Zentraler Backend-Fix deckt beide Flächen | Schichtplan + Settings rufen denselben create-Pfad | ✓ |
| Pro-Fläche separat | — | |

**User's choice:** ja, zentral (folgt aus SDF-01 (a))
**Notes:** —

---

## Test-/Verifikations-Strategie

| Option | Description | Selected |
|--------|-------------|----------|
| cargo-Tests als harte Gates, Browser optional | Backend-Switch-Test + Settings-SSR-Test; Browser manuell wegen D-25-06 | ✓ |
| Browser-e2e als Pflicht-Gate | — | |

**User's choice:** cargo-Tests hart, Browser optional/manuell
**Notes:** D-25-06 — programmatisches Setzen von date/select triggert Dioxus-Signale nicht zuverlässig → Anzeige-/Reset-Logik per cargo-Test.

---

## Claude's Discretion

- Exakte DAO-Mechanik für den atomaren Ersatz (soft-delete-then-insert vs. DAO-`update` erweitern).
- Konkreter Typ des `value`-Props am `SelectInput`.

## Deferred Ideas

None — Diskussion blieb im Phasen-Scope. MOD-01/02 → Phase 37, HYG-01/02 → Phase 38.
