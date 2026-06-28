# Requirements: Shifty — v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit

> Definiert 2026-06-28 zum Milestone-Start (`/gsd-new-milestone`). Phasen-Nummerierung
> läuft fort (letzte abgeschlossene Phase: 24 → v1.7 ab Phase 25).
> Roadmap: [`ROADMAP.md`](ROADMAP.md).

**Core Value:** Abwesenheiten werden automatisch und korrekt in Reports und
Jahresansicht berücksichtigt — Feiertage müssen nicht mehr manuell pro Mitarbeiter
gepflegt werden, und Urlaub von Freiwilligen verzerrt die geplante Verfügbarkeit
nicht mehr.

## Maßgebliche Design-Vorgaben

- **Feiertags-Referenz-Logik:** Die Auto-Anrechnung repliziert **exakt** die Wirkung
  eines manuellen `ExtraHours` mit Kategorie `Holiday` — nicht mehr und nicht weniger.

- **Asymmetrie (bewusst):** Feiertage lassen die freiwillige committed-Zusage 🎯
  unberührt (wie Holiday-`ExtraHours`); **Urlaub/Abwesenheit** eines Freiwilligen
  reduziert sie.

- **Stichtag:** Die Feiertags-Automatik greift erst ab einem konfigurierbaren Datum;
  die Vergangenheit (Snapshots, evtl. manuelle Einträge) bleibt unberührt.

## v1.7 Requirements

### Automatische Feiertags-Anrechnung (HOL) — Achse A (Reporting)

- [x] **HOL-01**: Pro `Holiday` aus `special_day` wird für jeden Mitarbeiter, der am
  betreffenden Wochentag laut am Tag gültigem Vertrag arbeitet, das Feiertags-
  Stundenäquivalent (`hours_per_holiday` = Wochenstunden / Arbeitstage) automatisch
  angerechnet — ohne manuellen `ExtraHours`-Eintrag.

- [x] **HOL-02**: Die Wirkung der Auto-Anrechnung ist **identisch** zu einem manuellen
  `ExtraHours(Holiday)`: reduziert `expected_hours`, erhöht die Balance, erscheint als
  `holiday_hours`-Spalte in Report und Jahresansicht-Display. Verifiziert per
  Vergleichstest gegen einen äquivalenten manuellen Eintrag.

- [x] **HOL-03**: Die Verfügbarkeits-Rechnung der Jahresansicht (`paid_hours`,
  `committed_voluntary_hours` 🎯, `volunteer_hours` 🤝) bleibt durch die Feiertags-
  Automatik **unverändert** (Regressions-Guard).

### Freiwilligen-Abwesenheit in der Jahresansicht (VFA) — Achse B (Booking-Information)

- [x] **VFA-01**: Urlaub/Abwesenheit eines Freiwilligen (`is_paid=false`,
  `committed_voluntary>0`) **reduziert** seine committed-Zusage 🎯 in der Jahresansicht
  (`booking_information.rs::get_weekly_summary`) für die von der Abwesenheit betroffenen
  Arbeitstage/Wochen.

- [x] **VFA-02**: Feiertage reduzieren die committed-Zusage bewusst **nicht** (Asymmetrie
  zu VFA-01, konsistent mit HOL-03) — Regressions-Guard.

### Stichtag & Konfiguration (HCFG)

- [x] **HCFG-01**: Ein global konfigurierbares „Feiertags-Automatik aktiv ab"-Datum
  steuert, ab wann die Auto-Anrechnung greift. Feiertage **vor** dem Stichtag werden von
  der Automatik nicht angerechnet (Bestand/manuelle Einträge geschützt).

- [ ] **HCFG-02**: Admin-gated Settings-UI zum Setzen/Ändern des Stichtag-Datums (analog
  `/settings/`-Seite aus v1.6), persistiert über die bestehende Konfig-/Toggle-Infra.
  i18n de/en/cs.

- [x] **HCFG-03**: Keine Doppelzählung ab Stichtag — ein Feiertag wird entweder durch die
  Automatik **oder** durch einen evtl. vorhandenen manuellen `ExtraHours(Holiday)`
  angerechnet, nicht beides. (Konfliktregel final in discuss-phase.)

### Snapshot-Korrektheit (HSNAP)

- [ ] **HSNAP-01**: `CURRENT_SNAPSHOT_SCHEMA_VERSION` wird bei Bedarf gebumpt (10 → 11),
  falls sich die Holiday-Computation/Input-Set ändert; historische Snapshots vor dem
  Stichtag bleiben reproduzierbar/unverändert.

### Cross-Navigation (NAV) — Frontend

- [ ] **NAV-01**: Die Abwesenheitsansicht (`/absences`) und der Mitarbeiterreport /
  Jahresansicht verlinken **gegenseitig** pro Mitarbeiter (Deep-Link mit vorbelegtem
  Personen-Kontext). Beide Report-Einstiege werden berücksichtigt:

  - **(a) Sales-Rolle** → „Mein Zeitkonto" (eigener Report) ↔ eigene Abwesenheiten.
  - **(b) HR-Rolle** → Mitarbeiterseite (Report pro Mitarbeiter) ↔ Abwesenheiten des
    jeweiligen Mitarbeiters.
  i18n de/en/cs.

## Future Requirements (deferred)

- **ShortDay / Kurztage** automatisch anrechnen (anteilig / `time_of_day`) — separate Story.
- **Volle Urlaubsverwaltung für Freiwillige** — Vacation-Balance/Anspruch (`get_all_paid`
  um `is_paid=false` erweitern). v1.7 liefert nur die Jahresansicht-Reduktion.

## Out of Scope (bestätigt)

| Feature | Grund |
|---------|-------|
| ShortDay / Kurztage | User-Entscheidung — separate Future-Story |
| Eigenständige committed-Reduktion durch Feiertage | Verworfen — Referenz ist exakt `ExtraHours(Holiday)` (HOL-03 / VFA-02) |
| Rückwirkende Feiertags-Anrechnung vor Stichtag | Stichtag schützt Vergangenheit bewusst |
| Urlaubsanspruch/-Balance-Verwaltung für Freiwillige | Nur Jahresansicht-Reduktion in v1.7 (VFA-01) |

## Offene Design-Fragen (discuss-phase, keine Requirements)

- materialize (echte `ExtraHours`-Rows) vs. derive-on-read für die Feiertags-Anrechnung.
- Welche Absence-Kategorien VFA-01 abdeckt (nur Vacation oder auch SickLeave/UnpaidLeave).
- Ob der Stichtag (HCFG-01) auch VFA-01 steuert (historische Jahresansicht-Stabilität).
- Exakter Nenner / Tag-Auswahl der Reduktionen (`has_day_of_week`, `hours_per_holiday`).
- Speicherort der Stichtag-Konfiguration (Toggle-/Settings-Infrastruktur aus v1.6).
- Konfliktregel HCFG-03 (manuell vor Automatik vs. Automatik ersetzt).
- NAV-01: genaue Link-Platzierung + Deep-Link-Parameter (Personen-Filter/Jahr) — UI-phase.

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| HOL-01 | Phase 25 | ⏳ |
| HOL-02 | Phase 25 | ⏳ |
| HOL-03 | Phase 25 | ⏳ |
| VFA-01 | Phase 26 | ⏳ |
| VFA-02 | Phase 26 | ⏳ |
| HCFG-01 | Phase 25 | ⏳ |
| HCFG-02 | Phase 25 | ⏳ |
| HCFG-03 | Phase 25 | ⏳ |
| HSNAP-01 | Phase 25 | ⏳ |
| NAV-01 | Phase 26 | ⏳ |

**Coverage:** 10/10 (Roadmap complete — alle v1.7 Requirements gemappt).

---
*Requirements defined: 2026-06-28 (v1.7 milestone start)*
*Traceability filled: 2026-06-28 (Roadmap created — Phasen 25–26)*
