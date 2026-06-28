# Phase 26: Freiwilligen-Abwesenheit & Cross-Navigation - Discussion Log

> **Audit trail only.** Decisions are captured in CONTEXT.md.

**Date:** 2026-06-28
**Phase:** 26-Freiwilligen-Abwesenheit & Cross-Navigation
**Areas discussed:** VFA-Kategorien, VFA-Stichtag, VFA-Reduktions-Formel, NAV-Deep-Link
**Mode:** Interactive (gsd-autonomous --interactive), Batch-Entscheidung mit Empfehlungen

---

## VFA-01 Absence-Kategorien
| Option | Selected |
|--------|----------|
| Alle drei (Vacation + SickLeave + UnpaidLeave) | ✓ |
| Nur Vacation | |

**Choice:** Alle drei (= Empfehlung). Abwesenheit = nicht verfügbar, egal aus welchem Grund.

## VFA-01 Stichtag
| Option | Selected |
|--------|----------|
| Immer aktiv, kein Stichtag | ✓ |
| Stichtag-gated (wie HCFG-01) | |

**Choice:** Immer aktiv (= Empfehlung). Year-View ist live/nicht persistiert → kein Snapshot-Bump, keine Config-UI. Löst D-25-09.

## VFA-01 Reduktions-Formel
| Option | Selected |
|--------|----------|
| Pro abwesendem Arbeitstag anteilig | |
| Ganze Woche raus bei Abwesenheit | ✓ |

**Choice:** Ganze Woche raus (überstimmt meine anteilige Empfehlung). Irgendeine Abwesenheit in der Woche → committed_voluntary dieser Person/Woche = 0. Simpler.

## NAV-01 Deep-Link-Mechanismus
| Option | Selected |
|--------|----------|
| Route-Param /absences/:employee_id | ✓ |
| Interner Selektor via Query/State | |

**Choice:** Route-Param (= Empfehlung). Bookmarkbar, konsistent mit /employees/:employee_id; belegt den bestehenden Personen-Selektor vor.

## Claude's Discretion
- Wochen-Overlap via find_overlapping_for_booking; DI AbsenceService → BookingInformationService (kein Zyklus); separate Route vs optionaler Param; i18n-Texte + Link-Platzierung (UI-Phase).

## Deferred
- Volle Urlaubsverwaltung für Freiwillige; anteilige Pro-Tag-Reduktion; per-Mitarbeiter-Jahresansicht-Deep-Link vom weekly_overview.
