# Phase 29: Urlaubs-Balken-Konsistenz (FE) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-29
**Phase:** 29-Urlaubs-Balken-Konsistenz (FE)
**Areas discussed:** Überzug-Visualisierung

---

## Überzug-Visualisierung (`used + planned > total`)

| Option | Description | Selected |
|--------|-------------|----------|
| Volle Breite + Warnfarbe | Balken `(used+planned)/total`, bei Überzug auf 100% gekappt + amber; negative Resturlaub-Zahl signalisiert Überzug. Einzeiler, kein Layout-Risiko, nutzt vorhandene `<=3`-Farb-Logik. | ✓ |
| Physischer Überlauf | `overflow-hidden` entfernt, Breite ungekappt (>100%), Balken ragt über Track hinaus. Wörtliche SC2-Erfüllung, aber Layout-Risiko (Kartenkante). | |

**User's choice:** Volle Breite + Warnfarbe (→ D-29-02)
**Notes:** Design-System hat nur `good`/`warn`-Tokens (kein drittes „danger"). ROADMAP-SC2
„über 100% hinaus / kein Clamp" wurde entsprechend als Farb-Signal-Interpretation nachgezogen
(voller amber Balken + negative Zahl statt physischem Überlauf).

---

## Claude's Discretion

- Code-Form des Einzeilers (lokale Bindung vs. Inline-Summe) — Static-class-Pattern muss bleiben.
- Test-Form: reine Berechnung bevorzugt in pure Helfer-Funktion extrahieren → per `cargo test`
  abdeckbar (Browser-Render-Tests laut Projekt-Memory unzuverlässig).
- Warnfarben-Schwelle bleibt `remaining <= 3.0` (single flag, erfüllt SC3, D-29-03).

## Deferred Ideas

- Zwei-Segment-Urlaubsbalken (genommen vs. geplant) — Future, in REQUIREMENTS.md deferred.
