# Plan: Phase 58 (Feature) — Optionale Stundenverteilung pro Werktag (Snapshot-Bump 13→14)

Repo: `~/programming/projects/shifty-backend` | Branch: `jj`-Branch `weekday-hours-distribution` vom aktuellen Stand
Kanban-Task: `t_16fdb494` | Enabler für: `t_ae384342` (Vertragszeit-Warnung, folgt nach)

## Kontext & Ist-Zustand

- `employee_work_details` (ehem. `working_hours`) ist **pro Vertragsversion** versioniert
  (`from_year/from_calendar_week/from_day_of_week` … `to_*`) und trägt bereits boolean-Flags
  `monday..sunday` + `workdays_per_week` + `expected_hours`
  (`service/src/employee_work_details.rs:14-42`, Migration `20241023062246`). Die
  Werktage-Stunden reiten als **7 optionale `REAL`-Spalten** auf derselben Vertragszeile —
  der bestehende Vertragswechsel-Mechanismus erledigt „Gültigkeit ab KW/Datum" gratis.
- Pausedelte Tagesstunde wird heute an **zwei** Stellen aus dem Vertrag abgeleitet:
  - `EmployeeWorkDetails::hours_per_day()` = `expected_hours / workdays_per_week`
    (`service/src/employee_work_details.rs:108`)
  - `EmployeeWorkDetails::holiday_hours()` = `expected_hours / potential_days_per_week()`
    (`service/src/employee_work_details.rs:112`, nutzt die Wochentags-Flags, NICHT `workdays_per_week`)
- Calculation-Sites, die taggenau werden müssen (Blast Radius, grep-verifiziert):
  - **Site A — Holiday-Derivation** (v1.7): `build_derived_holiday_map_for_week_pure`
    (`service_impl/src/reporting.rs:564`, konsumiert `holiday_hours()` auf Zeile 599) +
    async-Zwilling `build_derived_holiday_map` (Zeile 621).
  - **Site B — Absence-Zählung**: `service_impl/src/absence.rs:517-548` (gezahlte
    Urlaubstage × `hours_per_day`, pro-Wochen-Cap Zeile 769-786), `days_from_hours`
    (Zeile 809) für die Anzeige-Umrechnung konkreter Stundenposten.
  - **Site C — Urlaubsanspruch**: `service_impl/src/vacation_balance.rs`
    (`representative_hours_per_day` Zeile 78, Modell A: ein `hours_per_day` pro Jahr).
  - **Site D — Snapshot**: `service_impl/src/billing_period_report.rs`
    (Computation-Doku Zeile 64, `CURRENT_SNAPSHOT_SCHEMA_VERSION = 13` Zeile 140).
- REST: `rest/src/employee_work_details.rs` (POST/PUT/DELETE + Read), TO
  `EmployeeWorkDetailsTO` in `rest-types/src/lib.rs:747`. FE: `contract_modal.rs` +
  `employee_work_details_form.rs` (i18n via `i18n::Key`).
- PO-Spec (Board-Comment `t_16fdb494`): Summenvalidierung über **aktive** Werkstage,
  `NULL`-Verteilung bleibt **bit-exakt** im alten Pfad (Regression je Site), Bump 13→14
  Planbestand, FE-Editor im Vertrags-Modal, i18n de/en/cs.

## Definitions of Done (PO-Scope)

- Verteilung = 7 Spalten `monday_hours..sunday_hours` (`REAL NULL`), je aktive
  Vertragsversion. **„Aktive Verteilung"** = alle flagierten Werktage haben einen
  Nicht-NULL-Wert. (Annahme, im Plan fixiert: Teilweise Verteilung = inaktiv → Fallback;
  Validierung erzwingt Vollständigkeit.)
- Validierung (Service-Layer, bei Create/Update): Summe über **aktive** (flagierte)
  Werkstage == `expected_hours` (f32-Epsilon 1e-3), sonst `ServiceError` → HTTP 422.
  Nicht-aktive Wochentage dürfen NULL (oder 0.0) sein.
- Ohne aktive Verteilung: alle Berechnungspfade **bit-exakt** identisch zu heute
  (jeweils Regressionstest).

## Schritte

1. **Migration + DAO + Entity.**
   - Migration `migrations/sqlite/20260824000000_add-weekday-hours-to-employee-work-details.sql`:
     7× `ALTER TABLE employee_work_details ADD COLUMN <weekday>_hours REAL` (nullable,
     kein DEFAULT → NULL = nicht gesetzt).
   - DAO-Entity `dao_impl_sqlite/src/employee_work_details.rs` + SELECT-Liste erweitern,
     `.sqlx`-Offline-Cache via `sqlx prepare` neu generieren (CI-Präzedenz Phase 34:
     fehlender Cache = rote CI).
   - `service/src/employee_work_details.rs`: 7 Felder `Option<f32>`, `From`-Impl
     ergänzen; neue pure Helper `weekday_hours(&self, w: Weekday) -> Option<f32>`
     (Some nur bei aktiver Verteilung und flagiertem Tag) +
     `has_active_distribution(&self) -> bool`.
   - Verifikation: `cargo build --workspace` + neue Unit-Test Roundtrip
     (NULL-Vertrag → `weekday_hours == None`; vollständige Verteilung → `Some(x)`).
2. **Summenvalidierung (Service) + 422.** `service_impl/src/employee_work_details.rs`
   (Create/Update-Pfad): Prädikat als pure fn `validate_weekday_distribution(
   expected_hours, workday_flags, weekday_hours) -> Result<(), ValidationError>` —
   TDD: Tests für Summe ≠ Soll (422), Teil-Verteilung (inaktiv, OK), Epsilon-Fall,
   nicht-aktiver Tag mit Wert (definiert: wird ignoriert bzw. abgelehnt — im Code
   kommentieren).
   - Verifikation: `cargo test employee_work_details` grün inkl. 4 neuer Fälle.
3. **Site A — Holiday-Derivation taggenau.** In `build_derived_holiday_map_for_week_pure`
   (+ async-Zwilling): `wh.holiday_hours()` →
   `wh.weekday_hours(weekday).unwrap_or(wh.holiday_hours())`.
   - Verifikation: neue pure-Tests (Feiertag am 4h-Werktag rechnet 4h an; Feiertag an
     nicht-aktiver Verteilung byte-identisch zum alten Wert) + existierende v1.7-Tests
     bleiben grün (bit-exakte Regression, NULL-Verteilung).
4. **Site B — Absence-Zählung taggenau.** `absence.rs`: gezählte Urlaubstage nutzen
   `weekday_hours(Wochentag des Tages).unwrap_or(hours_per_day())` statt konstantem
   `hours_per_day`; pro-Wochen-Cap (Zeile 769) zählt weiterhin **Tage** (Semantik
   unverändert, nur Stundenwert pro Tag wird taggenau); `days_from_hours`-Aufrufe
   (Zeile 642, 727) für konkrete Posten: Wochentag bekannt → taggenau, sonst Fallback.
   - Verifikation: `cargo test absence` grün + 3 neue Regressionstests
     (Urlaub auf 4h-Di vs. 8h-Mo; NULL-Verteilung bit-exakt; Cap-Verhalten unchanged).
5. **Site C — Urlaubsanspruch.** `vacation_balance.rs`: `representative_hours_per_day`
   bleibt Modell A — bei aktiver Verteilung ist der Tages-Mittelwert über aktive
   Werktage per Konstruktion gleich `expected_hours/workdays_per_week` =
   `hours_per_day()`, d. h. **keine Änderung nötig**; stattdessen: Dokumentation des
   Sachverhalts + Regressionstest, der beides für aktive Verteilung belegt
   (Summe aktive Tage × 52 == `expected_hours × 52`).
   - Verifikation: `cargo test vacation_balance` grün, Test neu.
6. **Site D — Snapshot + Bump 13→14.** `billing_period_report.rs`: prüfen, ob die
   Computation direkt `hours_per_day`/`holiday_hours` liest oder über das korrigierte
   Reporting-Datum (Erwartung: über Site-A/B-Ergebnisse durchgereicht → keine
   Formeländerung nötig, aber die **Computation ergibt andere Werte** bei aktiver
   Verteilung → Bump fällig). `CURRENT_SNAPSHOT_SCHEMA_VERSION` 13 → 14; die
   fünf Snapshot-Locking-Gates (u. a. `pdf_export_config`, `shiftplan_report` pinnen die
   Konstante absolut, Präzedenz Phase 57) + Bump-Historie pflegen.
   - Verifikation: `cargo test billing_period` + Snapshot-Gates grün; Straddling-Golden-
     Snapshot-Test: NULL-Verteilung → byte-identisch zu v13 (Präzedenz D-01, Phase 56).
7. **REST + TO + OpenAPI.** `EmployeeWorkDetailsTO` +7 `Option<f32>`-Felder
   (`rest-types/src/lib.rs:747`) mit `ToSchema`-Doku; Create/Update-TOs + Handler
   `rest/src/employee_work_details.rs` durchreichen; utoipa-Annotationen.
   - Verifikation: OpenAPI-Surface-Contract-Test (Präzedenz v2.2 HYG-05) grün;
     `curl`-Roundtrip Create→Read mit Verteilung.
8. **FE-Editor im Vertrags-Modal.** `shifty-dioxus/src/component/contract_modal.rs` +
   `employee_work_details_form.rs`: optionale Stundenfelder für aktive Werktage
   (ausgegraut für nicht-aktive Wochentage), Live-Summenanzeige mit Soll-Vergleich,
   Disable-Save bei Summenabweichung (Backend bleibt autoritativ); i18n-Keys in
   de/en/cs (bestehendes `i18n::Key`-Schema; FE-i18n-Tests wie IMP-05 erweitern).
   Caveat D-25-06-Klasse: controlled Inputs, kein programmatisches Signal-Setzen in Tests.
   - Verifikation: `cargo test -p shifty-dioxus` grün (inkl. i18n-Key-Parität),
     WASM-Build warnungsfrei, FE-Clippy `-D warnings`.
9. **Docs-Freshness (EN+DE im selben Commit).** Trigger `migrations/sqlite/*.sql` +
   `service/**` + `billing_period_report.rs`:
   - `docs/architecture/03-data-model.md` + `diagrams/db-schema-er.mmd` (7 neue Spalten)
   - F07-Reporting-Balance-Doku (taggenaue Anrechnung, beide Sprachen)
   - F05/Vertrags-Doku, die die `working_hours`/`employee_work_details`-Tabelle beschreibt
   - `F08-billing-period.md` + `docs/domain/billing-period.md`: Bump-Historie 13→14
     + Version-Tabelle (welche `value_type`s betroffen — messen wie Phase 57, nicht raten)
   - Verifikation: `gsd-docs-update`-Drift-Check bzw. manuelle Gegenüberstellung;
     EN/DE synchron (Drift = Gate-Verstoß).
10. **Harte Gates (eigener Schritt, keine Fußnote).**
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace -- -D warnings`
    - `cargo test --workspace`
    - FE: `cargo build --target wasm32-unknown-unknown` + `cargo test -p shifty-dioxus`
      + `cargo clippy -p shifty-dioxus -- -D warnings`
    - Verifikation: alle vier grün in Folge (GSD-Auto-Commit bleibt aktiv, `jj` importiert).

## Abhängigkeiten

- 1 vor 2 (Spalten existieren), 2 vor 3–6 (Helper + Validierung), 3–5 parallelisierbar
  nach 2, 6 nach 3+4 (Computation liest deren Output), 7 nach 1 (TO-Felder), 8 nach 7
  (API-Shape), 9 nach 6+8 (Bump-Werte + FE-Final-Shape), 10 zum Schluss und vor Commit.
- **`t_ae384342` (Vertragszeit-Warnung) baut auf diese Phase auf** — Board-Link existiert
  schon; darf erst danach starten.

## Risiken

- **f32-Präzision bei der Summenvalidierung** → Summen über 7 f32-Werte driften
  → Gegenmaßnahme: Epsilon-Validierung (1e-3) im Service, FE zeigt Abweichung ab 0.1h.
- **`.sqlx`-Offline-Cache vergessen** → CI rot bei grünem lokalen Build (Präzedenz
  Phase 34) → `sqlx prepare` ist Schritt-1-Teil, Verifikation explizit.
- **Absence-Semantik: „Tage" vs. „Stunden"** — pro-Wochen-Cap zählt Tage; Verteilung
  ändert nur den Stundenwert pro Tag. Wenn der PO semantisch „Stunden-Cap" meint, wäre
  das eine andere Spec → im Review explizit als Annahme markiert.
- **vacation_balance Modell A** — bei aktiver Verteilung ist der Jahres-Representative
  per Konstruktion unverändert (Mathematik: Durchschnitt über aktive Tage ==
  `hours_per_day`), aber nur, wenn `workdays_per_week` == Anzahl flagierter Tage
  bleibt. Wenn ein Nutzer `workdays_per_week` ≠ Anzahl Flags setzt (heute schon
  möglich), existiert eine Diskrepanz → Regressionstest deckt das auf; PO-Annahme:
  `workdays_per_week` bleibt Definition für das Cap, Verteilung überschreibt nur Werte.
- **Doku-Drift EN/DE** — 5+ Dateien in 2 Sprachen → Schritt 9 ist ein eigener,
  letzter Schritt; Reviewer-Regel (Trigger-Dateien im Diff) beachten.
- **Snapshot-Bump-Wert falsch angegeben** → Bump-Historie erst nach Schritt 6 mit
  Messung schreiben (Präzedenz Phase 57: „Overall war im Plan übersehen").

## Nicht enthalten (Scope)

- `t_ae384342` Warnung außerhalb Vertragszeiten (folgt als eigene Phase).
- Keine Änderung der `is_dynamic`-/Cap-Logik, kein neues Privileg, keine FE-Seite
  außerhalb des Vertrags-Modals.
- Keine Retro-Aktivierung: bestehende Verträge bleiben ohne Verteilung (NULL →
  altes Verhalten), kein Backfill.
