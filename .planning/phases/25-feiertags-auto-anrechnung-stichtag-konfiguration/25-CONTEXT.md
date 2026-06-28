# Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration - Context

**Gathered:** 2026-06-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Feiertage aus `special_day` (Typ `Holiday`) werden **automatisch** im Mitarbeiterreport
angerechnet — mit **identischer Wirkung** zu einem manuellen `ExtraHours(Holiday)`-Eintrag
(reduziert `expected_hours`, erhöht Balance, erscheint als `holiday_hours`-Spalte). Ein
Admin setzt über eine admin-gated Settings-UI einen **Aktivierungsstichtag** ("aktiv ab"),
der steuert, ab wann die Automatik greift; die Vergangenheit (Snapshots, manuelle Einträge)
bleibt unberührt.

**Achse:** Reporting (Achse A). Backend **und** Frontend.

**Requirements:** HOL-01, HOL-02, HOL-03, HCFG-01, HCFG-02, HCFG-03, HSNAP-01.

**Liefert NICHT:**
- Keine ShortDay/Kurztage-Automatik (separate Future-Story).
- Keine eigenständige committed-Reduktion durch Feiertage (Referenz ist exakt
  `ExtraHours(Holiday)`; Freiwilligen-Asymmetrie = VFA, Phase 26).
- Keine Urlaubsverwaltung/-Balance für Freiwillige (Phase 26 liefert nur Jahresansicht-Reduktion).
- Keine Cross-Navigation (NAV-01 = Phase 26).

</domain>

<decisions>
## Implementation Decisions

### Architektur der Anrechnung (HOL-01/HOL-02)
- **D-25-01 (Derive-on-read, KEINE materialisierten Rows):** Die Feiertagsstunden werden
  **beim Report-Erstellen** direkt aus `special_day` + dem am Tag gültigen Vertrag berechnet
  und in dieselbe `holiday_hours`-Aggregation injiziert, die heute manuelle
  `ExtraHoursCategory::Holiday`-Einträge summiert (`service_impl/src/reporting.rs:402-406`).
  Es werden **keine** echten `ExtraHours`-DB-Zeilen erzeugt. Vorteil: Stichtag- und
  Konfliktregel werden zu simplen Read-Filtern; kein Write-Side-/Cleanup-Aufwand bei
  Feiertags- oder Vertragsänderungen; keine Backfill-Migration; keine Auto-vs-Manuell-Markierung.

### Stunden-Formel & Tag-Auswahl (HOL-01)
- **D-25-02 (Bestehenden Helper wiederverwenden):** Anrechnungsbetrag pro Feiertag =
  `EmployeeWorkDetails::holiday_hours()` (= `expected_hours / potential_days_per_week()`,
  `service/src/employee_work_details.rs:112-114`). Angerechnet wird **nur**, wenn der
  Mitarbeiter laut am Feiertag gültigem Vertrag an diesem Wochentag arbeitet —
  `EmployeeWorkDetails::has_day_of_week(holiday.day_of_week)` true
  (`service/src/employee_work_details.rs:146-156`). Nenner ist bewusst
  `potential_days_per_week()` (Anzahl gesetzter Wochentag-Flags), konsistent mit dem
  existierenden `holiday_hours()`-Helper — **nicht** `workdays_per_week`.

### Konfliktregel manuell vs. automatisch (HCFG-03)
- **D-25-03 (Manuell gewinnt):** Existiert für einen Mitarbeiter bereits ein **manueller**
  `ExtraHours(Holiday)`-Eintrag, der denselben Feiertag abdeckt, wird die Automatik für
  diesen Mitarbeiter+Tag **übersprungen** (keine Doppelzählung). Bewusste manuelle
  Eingaben/Korrekturen bleiben unangetastet und haben Vorrang. Granularität: pro
  Mitarbeiter + konkretem Feiertags-Tag.

### Stichtag-Speicherung & Settings-UI (HCFG-01/HCFG-02)
- **D-25-04 (Toggle-Tabelle um `value`-Spalte erweitern):** Der Stichtag wird über die
  **bestehende `toggle`-Infrastruktur** persistiert, die dafür um eine **nullable
  `value TEXT`-Spalte** erweitert wird (Migration). Der Stichtag liegt als **ISO-Datum-String**
  im `value`. *(User-Entscheidung — bewusst gegen die alternative neue Key-Value-Tabelle;
  reuse der in Phase 24 etablierten Toggle-Infra, siehe `dao/src/toggle.rs`,
  `service/src/toggle.rs`, `rest/src/toggle.rs`, `migrations/sqlite/20260105000000_app-toggles.sql`.)*
  DAO/Service/REST/Frontend werden entsprechend um Lesen/Schreiben des `value` ergänzt.
- **D-25-05 (Toggle-Semantik & Default = aus):** Ein dedizierter Toggle-Key (Name = Claude's
  Discretion, z.B. `holiday_auto_credit`). Der **`value` (ISO-Datum) ist der autoritative
  Gate**: ein Feiertag wird nur automatisch angerechnet, wenn ein Stichtag gesetzt ist UND
  `feiertag_datum >= stichtag`. **Default = kein Stichtag gesetzt → Automatik aus**
  (keine Regression auf Bestandsdaten, konsistent mit Phase-24-Prinzip „Default ändert
  kein Verhalten"). Die `enabled`-Spalte wird konsistent mitgeführt; exakte Verdrahtung
  (separater Master-Schalter vs. aus `value`-Präsenz abgeleitet) = Claude's Discretion.
- **D-25-06 (Settings-UI):** Admin-gated Settings-Seite analog Phase 24
  (`shifty-dioxus/src/page/settings.rs`, `is_admin_target`-Muster, `toggle_admin`-Gate)
  bekommt ein **Datums-Eingabefeld** zum Setzen/Ändern des Stichtags; nach Reload wieder
  auffindbar (persistiert). Alle Texte **i18n de/en/cs**. Konkretes Layout (Date-Input/
  Datepicker) wird in der UI-/Plan-Phase final.

### Snapshot-Korrektheit (HSNAP-01)
- **D-25-07 (Bump 10 → 11):** `CURRENT_SNAPSHOT_SCHEMA_VERSION` wird von **10 auf 11**
  gebumpt (`service_impl/src/billing_period_report.rs:101`), weil derive-on-read die
  Computation des persistierten `BillingPeriodValueType::Holiday` für Perioden **ab Stichtag**
  verändert (neue Input-Quelle `special_day`). Historische Snapshots **vor** dem Stichtag
  bleiben reproduzierbar/unverändert, weil der Stichtag-Filter sie nicht berührt. (Pflicht-Bump
  laut `shifty-backend/CLAUDE.md` § "Billing Period Snapshot Schema Versioning".)

### Regressions-Guard Jahresansicht (HOL-03)
- **D-25-08 (Year-View unangetastet):** Die Feiertags-Automatik fließt **ausschließlich** in
  den `holiday_hours`/`expected_hours`/`balance`-Pfad (`service_impl/src/reporting.rs`).
  Sie verändert **nicht** `dynamic_hours`/`paid_hours`/`committed_voluntary_hours`/
  `volunteer_hours` der Jahresansicht (`service_impl/src/booking_information.rs:203-273`).
  Wird per **Regressionstest** abgesichert (Jahresansicht-Werte vor/nach Automatik identisch).

### Stichtag-Scope (Phase-Grenze zu Phase 26)
- **D-25-09 (Stichtag gated nur HOL):** In Phase 25 steuert der Stichtag **ausschließlich**
  die Feiertags-Automatik (HOL). Ob derselbe Stichtag auch VFA-01 (Freiwilligen-Abwesenheits-
  Reduktion) gated, ist bewusst eine **Phase-26-Entscheidung** und hier nicht festgelegt.

### Claude's Discretion
- Exakter Toggle-Key-Name; ob `enabled` als separater Master-Schalter dient oder aus der
  `value`-Präsenz abgeleitet wird.
- Ableitung des absoluten Feiertags-Datums aus `(year, calendar_week, day_of_week)` für den
  Stichtag-Vergleich; internes Vergleichsformat des Stichtags (ISO-Datum vs. year+week).
- Genaue Detektion „manueller Holiday-Eintrag deckt diesen Feiertag ab" (Match-Kriterium:
  Mitarbeiter + Datum bzw. Woche+Wochentag).
- Alle i18n-Labels/Texte (de/en/cs) für Settings-Datumsfeld und etwaige Statusanzeige.
- Konkretes Settings-UI-Layout (Date-Input vs. Datepicker), am bestehenden Settings-/Token-Set ausgerichtet.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Projektregeln
- `.planning/REQUIREMENTS.md` — v1.7 Requirements HOL-01/02/03, HCFG-01/02/03, HSNAP-01;
  "Maßgebliche Design-Vorgaben" (exakte ExtraHours(Holiday)-Referenz, Asymmetrie, Stichtag).
- `.planning/ROADMAP.md` § "Phase 25" — Goal + 5 Success Criteria.
- `shifty-backend/CLAUDE.md` § "Billing Period Snapshot Schema Versioning" — **Pflicht-Regel**
  für den `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump (HSNAP-01).
- `.planning/phases/24-paid-capacity-enforcement-config/24-CONTEXT.md` — etabliertes
  Settings-/Toggle-Muster (D-24-01a Reuse ToggleService, D-24-06 neue `/settings/`-Route,
  D-24-07 Toggle-Seeding via Migration).

### Referenz-Pfad: ExtraHours(Holiday) → Report (HOL-01/02 Vorbild)
- `service_impl/src/reporting.rs:402-406` — heutige `holiday_hours`-Summe über manuelle
  `ExtraHoursCategory::Holiday`; **Injektionspunkt** für derive-on-read.
- `service_impl/src/reporting.rs:495-498` — `expected_hours`/`balance_hours`-Berechnung
  (`holiday_hours` ist Teil von `absense_hours`).
- `service/src/reporting.rs:111-116` — `GroupedReportHours::hours_per_holiday()`
  (`contract_weekly_hours / days_per_week`).
- `service/src/reporting.rs:147-161` — `ShortEmployeeReport.holiday_hours`.

### Stunden-Formel & Vertrag
- `service/src/employee_work_details.rs:104-156` — `holiday_hours()`,
  `hours_per_day()`, `potential_days_per_week()`, `has_day_of_week(weekday)`.
- `service_impl/src/reporting.rs:77-86` — `find_working_hours_for_calendar_week()`
  (am Tag gültigen Vertrag selektieren).

### Holiday-Quelle (special_day)
- `dao/src/special_day.rs:8-38` — `SpecialDayTypeEntity` (Holiday|ShortDay),
  `SpecialDayEntity` (year, calendar_week, day_of_week, day_type, time_of_day),
  `find_by_week`.
- `service/src/special_days.rs:35-90` — `SpecialDay`, `SpecialDayService::get_by_week`.
- `dao_impl_sqlite/src/special_day.rs:86-107` — SQLite-Query.
- `migrations/sqlite/20241020064536_add-special-day-table.sql` — Schema.

### Toggle-/Settings-Infrastruktur (HCFG-02, zu erweitern)
- `dao/src/toggle.rs:6-11` — `ToggleEntity` (name, enabled, description) — um `value` erweitern.
- `service/src/toggle.rs` — `ToggleService`-Trait.
- `rest/src/toggle.rs:17-27` — `GET /toggle/{name}/enabled`, `PUT .../enable|disable` —
  um value-get/set erweitern.
- `migrations/sqlite/20260105000000_app-toggles.sql` — Toggle-Tabelle (neue Migration
  für `value`-Spalte).
- `shifty-dioxus/src/page/settings.rs:1-149` — admin-gated Settings-Seite (Muster).
- `shifty-dioxus/src/api.rs:1577-1595` — `get_toggle_enabled`/`set_toggle` (Frontend-API).

### Snapshot-Versionierung (HSNAP-01)
- `service_impl/src/billing_period_report.rs:101` — `CURRENT_SNAPSHOT_SCHEMA_VERSION` (10 → 11).
- `service_impl/src/billing_period_report.rs:241` — Schreibstelle `BillingPeriodValueType::Holiday`.
- `service/src/billing_period.rs:34-51` — `BillingPeriodValueType`-Enum.

### Regressions-Guard Jahresansicht (HOL-03)
- `service/src/booking_information.rs:96-134` — `WeeklySummary`, `get_weekly_summary`.
- `service_impl/src/booking_information.rs:203-273` — `paid_hours` (dynamic_hours),
  `committed_voluntary_hours`, `volunteer_hours` — **muss unverändert bleiben**.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`EmployeeWorkDetails::holiday_hours()` / `has_day_of_week()` / `find_working_hours_for_calendar_week()`** —
  liefern Betrag + Tag-Auswahl + am-Tag-gültigen-Vertrag direkt; keine neue Formel nötig.
- **`holiday_hours`-Aggregation in `reporting.rs:402-406`** — derive-on-read addiert die
  abgeleiteten Stunden in dieselbe Summe; restlicher Report-/Balance-Pfad bleibt unverändert.
- **Toggle-DAO/Service/REST/Frontend-Kette** (Phase 24) — als Vorlage für value-Erweiterung
  und die Settings-Seite.
- **`special_day` find_by_week** — Holidays pro Woche; bei Bedarf week-weise iterieren statt
  neue Range-Query.

### Established Patterns
- **Service-Tier-Konvention** (`shifty-backend/CLAUDE.md`): `ToggleService` ist Basic-Tier
  (nur DAO+Permission+Transaction). Der Report-/Reporting-Service ist Business-Logic und darf
  ToggleService konsumieren (für den Stichtag-Read) — keine Zyklen.
- **Snapshot-Bump-Regel** (CLAUDE.md): Änderung der Computation eines persistierten
  `value_type` (hier `Holiday`) ⇒ `CURRENT_SNAPSHOT_SCHEMA_VERSION` +1.
- **`WHERE deleted IS NULL`** Soft-Delete-Konvention in allen DAOs.
- **i18n de/en/cs** für alle neuen UI-Texte (Key::Settings existiert bereits).

### Integration Points
- **Read des Stichtags im Report-Pfad:** Reporting-Service liest den Toggle-`value` (ISO-Datum)
  und filtert Holidays mit `holiday_datum >= stichtag`.
- **Konflikt-Check:** Im Report-Pfad pro Mitarbeiter+Tag prüfen, ob ein manueller
  `ExtraHours(Holiday)` denselben Feiertag abdeckt → falls ja, abgeleitete Anrechnung skippen.
- **Settings-UI ↔ Backend:** Datums-Eingabe schreibt via erweitertem `PUT /toggle/.../value`
  (o.ä.), liest via erweitertem GET.

</code_context>

<specifics>
## Specific Ideas

- **Identitäts-Verifikation (HOL-02):** Vergleichstest, der für denselben Mitarbeiter/Feiertag
  zeigt: derive-on-read-`holiday_hours` == `holiday_hours` bei äquivalentem manuellem
  `ExtraHours(Holiday)`-Eintrag (gleicher `expected_hours`/`balance`-Effekt).
- **Stichtag-Schutz (HCFG-01):** Feiertag vor Stichtag ⇒ kein Auto-Eintrag; bestehende
  manuelle Einträge + historische Snapshots unberührt — per Test abgesichert.
- **Keine-Doppelzählung (HCFG-03):** Feiertag mit manuellem Eintrag erscheint genau einmal
  im Report (manuell), nicht zusätzlich automatisch.

</specifics>

<deferred>
## Deferred Ideas

- **ShortDay / Kurztage automatisch anrechnen** (anteilig / `time_of_day`) — Future-Story,
  bewusst außer Scope (Out-of-Scope bestätigt in REQUIREMENTS.md).
- **Volle Urlaubsverwaltung/-Balance für Freiwillige** — Future-Story; v1.7/Phase 26 liefert
  nur die Jahresansicht-Reduktion (VFA-01).
- **Ob der Stichtag auch VFA-01 gated** — bewusst nach Phase 26 verschoben (D-25-09).

### Reviewed Todos (not folded)
- `2026-05-05-warnung-eintrag-ausserhalb-vertragszeiten.md` ("Warnung bei Eintrag außerhalb
  der Vertragszeiten") — nur Stopwort-Keyword-Match, inhaltlich Shiftplan-Warnung, **nicht**
  Feiertags-/Settings-Scope. Nicht gefoldet.
- `2026-05-07-review-frontend-list-user-invitations-silent-empty-fallback.md` — Frontend-API-
  Parsing-Review, unverwandt mit Phase 25. Nicht gefoldet.

</deferred>

---

*Phase: 25-Feiertags-Auto-Anrechnung & Stichtag-Konfiguration*
*Context gathered: 2026-06-28*
