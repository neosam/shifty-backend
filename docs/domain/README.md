# Fach-Referenz — Domain-Modell & Regeln

Diese Sektion beschreibt **das Fach**, nicht die Technik: was ein Booking ist,
wie das Stundenkonto rechnet, wann eine Billing-Period eingefroren wird, was
der Unterschied zwischen Absence und (Legacy-)Extra-Hours ist.

Wenn du fachlich mitreden willst oder eine Business-Regel prüfen musst, sind
das die Dokumente hier.

## Kapitel

- **[glossary.md](./glossary.md)** — Definitionen aller Domain-Begriffe.
- **[time-accounting.md](./time-accounting.md)** — Stundenkonto:
  Expected, Worked, Balance, Carryover — wie es berechnet wird und woher die
  Daten kommen.
- **[billing-period.md](./billing-period.md)** — Billing-Period-Snapshot,
  `snapshot_schema_version`, warum re-computation stabil sein muss.
- **[absence-system.md](./absence-system.md)** — Range-basierte Abwesenheiten
  (v1.0+), Cutover-Historie, Verhältnis zu Legacy-Extra-Hours.
- **[edge-cases.md](./edge-cases.md)** — **Zentrale Randfall-Referenz**.
  Enthält Stundenkonto-Rand­fälle und app-weite Ecken (Auth, Transaktionen,
  Zeit, Zeitzone, Rundung, Toggle-Rollouts, …).

## Warum eine eigene Fach-Sektion

Shifty rechnet nicht triviale Dinge. Ein Booking ist einfach; eine
korrekte Balance über einen Contract-Wechsel, einen Feiertag am Wochenende,
eine periodenübergreifende Krankmeldung und einen Toggle-Rollout mitten im
Zeitraum ist es nicht.

Diese Dokumentation existiert, damit:

- **Fach-Reviewer** (nicht-technische Stakeholder) prüfen können, ob eine
  Regel abgebildet ist, ohne Rust zu lesen.
- **Backend-Entwickler:innen** vor jeder Änderung an der Balance-Rechnung
  wissen, welche Kanten zu prüfen sind (`edge-cases.md`).
- **Zweit-Client-Entwickler:innen** verstehen, was ein zurückgegebener Wert
  bedeutet — ohne die Rechnung selbst zu bauen.
