---
created: 2026-06-27T13:23:20.083Z
title: Urlaub für Freiwillige in Absence-Tagen eintragen
area: absence
files:
  - service/src/absence.rs
  - service_impl/src/absence.rs
  - shifty-dioxus/src/page/absences.rs
---

## Problem

Im Absence-/Abwesenheits-Modul lässt sich derzeit Urlaub (und andere Absence-Kategorien)
nur für reguläre/bezahlte Mitarbeiter eintragen. Es fehlt die Möglichkeit, **Urlaub für
Freiwillige** zu erfassen.

Hintergrund: Freiwillige (unbezahlte SalesPersons, `is_paid = false`) sind ein eigenständiger
Personenkreis (vgl. v1.4 Committed-Voluntary-Capacity, „alle"-Filter mit `is_paid`-Gating).
Wenn Freiwillige Urlaub/Abwesenheit haben, soll das ebenfalls im Absence-System eintragbar
sein — heute ist dieser Pfad nicht abgedeckt bzw. auf bezahlte Personen beschränkt.

## Solution

TBD — vor Umsetzung klären:
- **Scope der Kategorien:** Nur Urlaub, oder alle Absence-Kategorien (krank/unbezahlt/…) auch
  für Freiwillige?
- **Auswirkung auf Berechnungen:** Freiwillige haben keinen Stunden-Vertrag → wie wirkt sich
  eine Absence auf Balance/Reporting aus? Vermutlich rein informativ/kapazitätsbezogen, NICHT
  in die bezahlte Stundenbilanz einfließen lassen (Doppelzählung/Leak vermeiden — vgl.
  v1.4 `is_paid`-Gating).
- **Backend:** Prüfen, ob `AbsenceService` / DAO die Person-Auswahl auf bezahlte Personen
  einschränkt (Permission-/Filter-Gate) und ob das für Freiwillige geöffnet werden muss.
- **Frontend:** Personen-Auswahl in der Absence-Eingabe muss Freiwillige anbieten (analog
  zum „alle"-Filter aus v1.4), inkl. i18n + Sichtbarkeits-/Rollen-Gating.

Verwandt: v1.4 Committed-Voluntary-Capacity (unpaid-volunteer `is_paid`-Gating), v1.6 Phase 24
(Paid-Capacity-Durchsetzung).
