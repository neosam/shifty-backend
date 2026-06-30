# Requirements: Shifty — v1.10 Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz

> **Versions-Hinweis:** `v1.10` ist das interne GSD-Planungs-Label. Die reale
> Release-Version vergibt der User datumsbasiert via `./cli-update-version`.

**Core Value:** Feiertage durchgängig korrekt machen — Special Days über die UI
pflegbar **und** ihre Soll-Wirkung auch in der Schichtplan-Wochentabelle sichtbar,
konsistent zur bereits korrekten Stundenkonto-Anrechnung (Phase 25, derive-on-read).

**Research:** keine (übersprungen) — beide Features bauen auf bestehenden Code-Mustern
auf, keine neue Tech/Library. Grundlage sind die beiden Analyse-Todos vom 2026-06-30
(`special-days-ui-bearbeiten-einstellungen`, `feiertag-soll-abzug-schichtplan-tabelle`).

**Ausgangslage (code-verifiziert):**

- Backend-CRUD für `special_day` existiert bereits: `GET /special-days/for-week/{year}/{week}`,
  `POST /special-days/`, `DELETE /special-days/{id}` (`rest/src/special_day.rs`). Das
  Frontend kann sie heute nur **lesen** (`get_special_days_for_week`).

- Die Feiertags-Automatik (`build_derived_holiday_map`, `service_impl/src/reporting.rs:151`)
  läuft heute in `get_reports_for_all_employees`/`get_report_for_employee_range`/`hours_per_week`,
  **nicht** in `get_week` — daher fehlt der Soll-Abzug in der Schichtplan-Tabelle
  (`booking_information` liest `report.expected_hours`/`report.holiday_hours` aus `get_week`).

**i18n (querschnittlich):** Alle neuen benutzersichtbaren Texte in de/en/cs — gilt für SPD-01..04.

## v1.10 Requirements

### Special-Days-UI (SPD)

- [x] **SPD-01**: Shiftplanner kann einen Special Day per **Kalenderdatum** anlegen (Typ
  `Holiday` oder `ShortDay`; bei `ShortDay` mit Uhrzeit). Persistiert als
  `(year, calendar_week, day_of_week)` über die bestehende `POST /special-days/`-CRUD;
  Datum→ISO-Woche/Wochentag-Mapping im Frontend (`time::Date::from_iso_week_date` /
  `as_shifty_week`). WASM-Datepicker-Caveat (D-25-06) beim Submit beachten.

- [x] **SPD-02**: Shiftplanner sieht die vorhandenen Special Days als **Liste** mit Datum im
  locale-üblichen Format **plus abgeleitetem Kontext** in Klammern, z. B.
  `15.08.2026 (Samstag, KW 33, 2026)`. Wochentag/KW/Jahr werden aus dem Datum berechnet
  und mitübersetzt.

- [x] **SPD-03**: Shiftplanner kann einen vorhandenen Special Day **löschen** (Frontend gegen
  `DELETE /special-days/{id}` verdrahtet; Liste aktualisiert sich).

- [x] **SPD-04**: Die Special-Days-Pflege ist **shiftplanner-gated** auf beiden Flächen
  (deckungsgleich zur bestehenden Special-Day-CRUD und Slot-Struktur-CRUD, die beide auf
  `SHIFTPLANNER_PRIVILEGE` gaten; FE-Gate `has_privilege("shiftplanner")`, kein 403-Mismatch);
  **alle Texte i18n de/en/cs**. *(Korrektur in discuss-phase: ursprünglich „admin-gated (Muster
  Phase 24/25)" — code-verifiziert auf `shiftplanner` geändert, da Special Days = Schichtplan-
  Struktur. Siehe D-33-01/02.)*

### Feiertags-Soll im Schichtplan (HSP)

- [ ] **HSP-01**: In der Wochentabelle unter dem Schichtplan reduziert ein automatisch
  angerechneter Feiertag das angezeigte **Soll** (`available_hours`/`expected_hours`) pro
  Mitarbeiter — **konsistent zum Stundenkonto**. Umsetzung: `get_week`
  (`service_impl/src/reporting.rs:884`) erhält den derived-Holiday-Beitrag (vierter
  Injektionspunkt analog 1a/1b/1c).

- [ ] **HSP-02**: Die abgeleiteten Feiertags-Stunden (`holiday_hours`) erscheinen in der
  Schichtplan-Tabelle (`booking_information` `WorkingHoursPerSalesPerson.holiday_hours`).

- [ ] **HSP-03**: Die Kapazitätsbänder (`paid_hours`/`dynamic_hours`/`committed_voluntary`/
  `volunteer`) bleiben **unverändert** — Regressions-Guard für die Bänder; der durch
  D-25-08 geschützte Kern (`dynamic_hours` → `paid_hours`) wird nicht angetastet.

- [ ] **HSP-04**: **Stichtag-Gate** und **Konfliktregel** (manueller `ExtraHours(Holiday)`
  gewinnt, keine Doppelzählung) wirken in der Schichtplan-Tabelle **identisch** zum
  Stundenkonto — durch Wiederverwendung von `build_derived_holiday_map`.
  *Offene Decision (discuss-phase, D-NN):* Anpassung/Neuformulierung des HOL-03-
  Regressionstests `test_holiday_auto_credit_no_year_view_impact`, der den `get_week`-
  Holiday-Aufruf aktuell ausdrücklich verbietet — künftig: Bänder unverändert, aber
  expected/available reduziert.

### Slot-Werte nur für eine Woche ändern (SWO)

- [ ] **SWO-01**: Im Slot-Editor wählt ein Shiftplanner explizit zwischen **„nur diese
  Woche"** und **„ab dieser Woche"**. „nur diese Woche" ändert die Slot-Werte als einmalige
  Ausnahme **ausschließlich** in der gewählten KW; die wiederkehrende Struktur bleibt
  davor und danach unverändert.

- [ ] **SWO-02**: Mechanik = **Split + Re-Merge** auf Basis von `ShiftplanEditService::modify_slot`
  (`service_impl/src/shiftplan_edit.rs:51`): drei Slot-Versionen — Segment 1 (Original bis
  `KW-1`), Segment 2 (`valid_from = Montag KW`, `valid_to = Sonntag KW`, neue Werte), Segment 3
  (`valid_from = KW+1`, Original-Werte bis ursprüngliches `valid_to`). Buchungen der KW werden
  auf Segment 2 re-pointed, Buchungen ab `KW+1` auf Segment 3 (gleiche Delete+Create-Re-Point-
  Logik wie heute). Editierbare Werte wie bei „ab KW": `min_resources`, `max_paid_employees`,
  `from`, `to`.

- [ ] **SWO-03**: Der gesamte Vorgang (alle Slot-Schnitte + alle Booking-Re-Points) läuft in
  **einer einzigen Transaktion**. Bei jedem Fehler erfolgt ein vollständiger Rollback — der
  Zustand ist exakt wie vorher. (`modify_slot` ist bereits atomar; die Erweiterung bleibt in
  derselben `tx`, ein `commit` am Ende.)

- [ ] **SWO-04**: Die Booking-Neuzuweisungen sind durch **harte Tests** abgesichert — keine
  doppelten oder verwaisten Buchungen, **nichts** doppelt in Reports/Balance. Gate =
  `shiftplan.edit` (konsistent zu `modify_slot`). Neue UI-Texte (Modus-Wahl) i18n de/en/cs.

## Future Requirements (deferred)

| Requirement | Begründung |
|---|---|
| ShortDay/Kurztage-Soll-Automatik im Report (anteilig, `time_of_day`) | Future-Story, schon in Phase 25 bewusst außer Scope. |
| Hover-Tooltip auf Feiertags-Zelle in der Schichtplan-Tabelle | Differentiator zu HSP-02, rein additiv. |
| ~~Multi-Wochen-Read-Endpoint für Special Days~~ | **In Phase 33 gezogen** (discuss-phase D-33-05): neuer Range/Jahr-Read-Endpoint speist die Settings-Übersicht. Nicht mehr deferred. |

## Out of Scope (bestätigt)

- **Snapshot-Schema-Version-Bump:** voraussichtlich **nicht** nötig — `billing_period`-
  Snapshots speisen sich aus dem `reporting.rs`-`holiday_hours`-Pfad, nicht aus
  `get_week`/`booking_information`. In der HSP-Phase verifizieren (Default: kein Bump).

- **Backlog-Phase 999.1** (Breaking/Major Dependency-Migration) — bleibt separat.

## Traceability

| Requirement | Phase | Status |
|---|---|---|
| SPD-01 | Phase 33 | 📋 planned |
| SPD-02 | Phase 33 | 📋 planned |
| SPD-03 | Phase 33 | 📋 planned |
| SPD-04 | Phase 33 | 📋 planned |
| HSP-01 | Phase 34 | 📋 planned |
| HSP-02 | Phase 34 | 📋 planned |
| HSP-03 | Phase 34 | 📋 planned |
| HSP-04 | Phase 34 | 📋 planned |
| SWO-01 | Phase 35 | 📋 planned |
| SWO-02 | Phase 35 | 📋 planned |
| SWO-03 | Phase 35 | 📋 planned |
| SWO-04 | Phase 35 | 📋 planned |

**Coverage:** 12/12 v1.10 Requirements gemappt (keine Orphans, keine Doppelzuordnung).
Phase 33 = Special-Days-UI (Frontend-zentriert, Backend-CRUD existiert + neuer Range/Jahr-Read).
Phase 34 = Feiertags-Soll im Schichtplan (Backend-zentriert, kein neuer Frontend-Anteil).
Phase 35 = Slot-Werte nur für eine Woche ändern (BE+FE; Split+Re-Merge auf `modify_slot`,
bewusst Schichtplan-Struktur, am 2026-06-30 in v1.10 aufgenommen).
