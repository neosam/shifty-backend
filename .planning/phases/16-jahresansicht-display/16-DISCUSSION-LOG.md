# Phase 16: Jahresansicht display - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-24
**Phase:** 16-jahresansicht-display
**Areas discussed:** Verfügbar-Semantik, Token & Überschuss-Notation, Blank/0-Regel, Chart-Behandlung

---

## Verfügbar-Semantik

| Option | Description | Selected |
|--------|-------------|----------|
| Ja — zählt mit (Backend) | overall_available_hours = paid + committed + surplus; Diff/Chart wachsen konsistent; Summe im Backend (year-view-only, nicht persistiert). | ✓ |
| Nein — rein informativ | committed nur Anzeige-Token; available/diff/chart unverändert (paid + surplus). | |

**User's choice:** Ja — zählt mit (Backend).
**Notes:** Löst den Phase-15-TODO (`booking_information.rs:270-273` „Phase 16 will sum both bands for display") ein. Snapshot-neutral, da WeeklySummary nicht persistiert.

---

## Token & Überschuss-Notation

| Option | Description | Selected |
|--------|-------------|----------|
| Drei getrennte Tokens | 💰paid \| 🎯zugesagt \| 🤝surplus. „5+2" = 🎯5 + 🤝2; gedeckt = 🎯5. Nutzt bestehenden 🤝-Token (= Band 2) weiter. | ✓ |
| Kombinierter zugesagt-Token | Ein Token zeigt inline „5 + 2"; separater 🤝-Surplus-Token entfällt. | |
| Du entscheidest | Token-Form Claude überlassen. | |

**User's choice:** Drei getrennte Tokens.
**Notes:** Exaktes Icon (🎯 Vorschlag) + Header-Anordnung bleibt Claude's Discretion.

---

## Blank/0-Regel

| Option | Description | Selected |
|--------|-------------|----------|
| Nur committed-Token „–" bei 0 | committed-Token blank/Strich bei 0; paid/volunteer numerisch. | |
| committed + Surplus „–" bei 0 | beide blank bei 0. | |
| Einfach „0" zeigen | committed-Token zeigt 0.00 wie paid/volunteer; keine Sonderlogik. | ✓ |

**User's choice:** Einfach „0" zeigen (User-Rückfrage: „Warum nicht einfach bei 0 bleiben?").
**Notes:** Revidiert die „blank/Strich, nicht 0"-Formulierung in CVC-07 / ROADMAP-SC#2. Blank/Strich-Idee gehört allenfalls in die Mitarbeiteransicht (Phase 17), nicht in die aggregierte Jahresansicht.

---

## Chart-Behandlung

| Option | Description | Selected |
|--------|-------------|----------|
| Balkenhöhe folgt available, kein Extra-Segment | Balken-Total wächst, aber committed ohne eigene Farbe (committed-Band erst v1.5). | |
| Drei Farben in Phase 16 | Gestapelter Balken paid/committed/surplus; holt CVC-F-02 aus v1.5 vor; Required-Linie bleibt; Tooltip nennt alle drei. | ✓ |
| Chart komplett unangetastet | Balken bleibt paid + surplus trotz committed in Diff-Spalte (bewusste Inkonsistenz). | |

**User's choice:** Drei Farben in Phase 16 ("Es sind drei Farben").
**Notes:** User bestätigte zudem, dass im Surplus (Band 2) committed bereits per-Person abgezogen ist → drei Segmente stapeln sich ohne Doppelzählung (paid + committed + surplus = paid + max(committed, actual)). Zieht CVC-F-02 bewusst aus v1.5 in Phase 16.

---

## Claude's Discretion

- Token-Emoji/Icon für „zugesagt" + genaue Spalten-/Header-Anordnung.
- committed-Chart-Segment-Farbe (token-basiert, keine Hardcoded-Hex).
- Exakte i18n-Label-Texte + ob `PaidVolunteer`-Header-Key erweitert oder neuer Key.
- Test-Platzierung (SSR, Chart-Segment, From-Mapping, Per-Locale-Matcher).

## Deferred Ideas

- Editor-Input + „alle"-Filter + unpaid-volunteer-Record + is_paid-Gating → Phase 17.
- Blank/Strich statt „0" → ggf. Mitarbeiteransicht (Phase 17), nicht Jahresansicht.
- Inline-Banner „Zusage nicht erfüllt" → v1.5 (CVC-F-01).
- Research-Flag: zwei `get_weekly_summary`-Varianten, zweite mit committed=0.0-Placeholder — Year-View-Pfad klären.
