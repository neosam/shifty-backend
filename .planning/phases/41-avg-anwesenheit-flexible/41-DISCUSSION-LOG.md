# Phase 41 — Discussion Log

**Date:** 2026-07-02
**Mode:** discuss (manuell geführt — gsd-tools-Shim defekt, siehe reference_gsd_tools_roster_broken)

## Verlauf

Erste Fragerunde (7 Grau-Bereiche aus ROADMAP D-AVG-01..08) war zu abstrakt — User
fehlte der Feature-Kontext. Nach Codebase-Verortung (bestehender HR-Statistik-Bereich im
Mitarbeiter-Report zeigt bereits A-22-1 „Ø Std/Woche") wurde das Feature greifbar.

### Entscheidungen

| Frage | Optionen präsentiert | User-Wahl |
|-------|----------------------|-----------|
| Zeitraum | freie from/to · Abrechnungsperiode · Jahr-bis-heute | **angezeigter Report-Zeitraum** |
| Kennzahl-Art | Std/Woche (existiert) · tagebasiert | **tagebasiert** — „auf Wochentage" |
| Konkrete Kennzahl | Ø Anwesenheitstage/Woche · Anwesenheitsquote % · Ø Std/Anwesenheitstag | **Ø Stunden pro Anwesenheitstag** |
| Anwesenheitstag-Def | Tag mit echter Arbeit (Shiftplan/ExtraWork/VolunteerWork, h>0) | **bestätigt (A)** |
| Exclusion-Set | nur Urlaub · alle Absencen · — | **löst sich auf**: nur echte Arbeitstage im Nenner ⇒ jede Absence draußen |
| Anzeige-Ort | eigene Sicht · bestehender Report-Statistik-Bereich | **bestehender Bereich** |
| Schwelle | — | **mind. 2 Anwesenheitstage**, sonst Leerzustand |

### Aufgelöste Verwirrungen
- „Redundanz zur A-22-1-Zahl?" → nein: neue Zahl ist tagebasiert (Std/Tag), alte ist
  Std/Woche. Bewusst getrennt.
- „Nur Urlaub vs. alle Absencen raus?" → durch die Std/Anwesenheitstag-Definition
  gegenstandslos: nur echte Arbeitstage zählen im Nenner.

### Bestätigend gelockt (kein Rütteln)
- Scope `is_dynamic == true`, server-seitiger Filter, HR-gated.
- Reines Read-Aggregat, kein Snapshot-Bump (Version bleibt 12), A-22-1 unberührt.
- i18n de/en/cs.

## Deferred
- AVG-04 (Trend über Perioden), AVG-05 (konfigurierbare Exclusion), Multi-MA-Liste.
