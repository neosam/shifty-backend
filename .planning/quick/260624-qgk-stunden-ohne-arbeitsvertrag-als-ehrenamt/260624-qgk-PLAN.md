---
task: 260624-qgk
title: Ehrenamt ohne Arbeitsvertrag verbuchbar + Ehrenamt-Stunden unter "Soll" anzeigen (Schwelle >= 0.5)
type: quick
mode: quick
files_modified:
  - service_impl/src/reporting.rs
  - service/src/reporting.rs
  - rest-types/src/lib.rs
  - shifty-dioxus/src/loader.rs
  - shifty-dioxus/src/state/employee_work_details.rs
  - shifty-dioxus/src/component/working_hours_mini_overview.rs
  - shifty-dioxus/src/i18n/mod.rs
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/cs.rs
autonomous: true
snapshot_schema_version_bump: false
---

<objective>
Drei Anforderungen aus dem User-Task:

1. **Ehrenamt-Stunden ohne Arbeitsvertrag verbuchbar** — Kann man Stunden ohne
   Arbeitsvertrag als Ehrenamt verbuchen? **Befund: JA, bereits möglich.** Die
   freiwillige Zusage (`committed_voluntary`) lebt als Feld auf
   `EmployeeWorkDetails`. Eine Person braucht also einen
   `EmployeeWorkDetails`-Record, der aber **`expected_hours = 0`** tragen darf —
   das ist genau die "kein Arbeitsvertrag"-Repräsentation. Die Reporting-Gates
   in `booking_information.rs` (Z. 212/224) UND die Editor-Sichtbarkeit in
   `contract_modal.rs` (Z. 173, `show_committed`) verwenden bereits explizit
   `wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0` (D-05:
   "cap || rein-freiwillig"). Damit fließt `committed_voluntary` einer reinen
   Freiwilligen-Zeile (expected_hours = 0, cap = false) bereits korrekt in die
   verfügbaren Stunden ein, OHNE dass ein bezahlter Vertrag besteht.
   → Scope für Req 1 = **bestätigender Regressionstest** (kein Verhaltens-Change),
   damit dieser Pfad gegen Regression gepinnt ist.

2. **Ehrenamt-Stunden unter "Soll" anzeigen** — Die "Soll"-Spalte (DE "Soll",
   EN "Target", CS "Cíl") lebt in `working_hours_mini_overview.rs`
   (`Key::WorkingHoursTableTarget`, Tabellen- und Card-Layout). Sie rendert das
   `dynamic_hours`-Feld (= Vertrags-Sollstunden) je Mitarbeiter. Die
   committed_voluntary-Stunden sollen **als separate Zusatzzeile/Badge NEBEN der
   Soll-Zahl** ausgewiesen werden (additive Anzeige, NICHT in die Soll-Zahl
   eingerechnet — die Soll-Zahl bleibt die reine Vertrags-Sollstundenzahl,
   damit Auslastung/Balance unverändert bleiben). Dafür muss
   `committed_voluntary_hours` je Person durch den Short-Week-Report
   (`get_week` + `get_reports_for_all_employees`) → `ShortEmployeeReportTO` →
   `WorkingHoursMini` geschleust werden.
   **Annahme (explizit):** separate Anzeige neben Soll, additiv, kein Reinrechnen
   in die Soll-Zahl. Begründung: Soll = vertragliche Pflichtstunden; Ehrenamt ist
   freiwillig obendrauf (v1.4-Design D-01: committed_voluntary ist von
   expected_hours entkoppelt). Ein Reinrechnen würde Balance/Auslastung
   verfälschen.

3. **Schwelle: Ehrenamt nur anzeigen wenn >= 0.5** — Reiner Display-Gate. Wenn
   `committed_voluntary_hours < 0.5`, wird die Ehrenamt-Anzeige komplett
   ausgeblendet; bei `>= 0.5` sichtbar. Gilt an ALLEN Render-Sites in
   `working_hours_mini_overview.rs` (Card-Layout + Table-Layout-Row; die
   Total-Zeile zeigt die Summe nur, wenn die Gesamt-Ehrenamt-Summe >= 0.5).

Purpose: Sichtbarkeit der freiwillig zugesagten Stunden in der Mitarbeiter-
Wochenübersicht, mit Rausch-Unterdrückung kleiner Werte.
Output: committed_voluntary je Person bis in die Soll-Tabelle durchgereicht +
geschwellte separate Anzeige + Tests + i18n (3 Locales).

**Snapshot-Schema-Version:** KEIN Bump nötig. Begründung: Req 2 berührt
ausschließlich den Live-Short-Report-Pfad (`get_week` /
`get_reports_for_all_employees`), der NICHT persistiert wird. Es wird kein
`billing_period`-`value_type` neu berechnet, hinzugefügt, entfernt oder in seinem
Input-Set verändert. `billing_period_report.rs` Z. 74 dokumentiert bereits, dass
die committed_voluntary-Zwei-Band-Arbeit (Phase 15) "KEIN Bump — Achse-B-only"
war; diese Anzeige-Erweiterung ist erst recht snapshot-neutral.
`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt unverändert.
</objective>

<vcs_constraint>
**jj-only, KEINE Commits durch das Tooling.** Dieses Repo ist jj-co-located; der
USER committet alles manuell. Der Executor dieses Plans führt KEINE git-/jj-
Commit-/Add-Befehle aus. Plan-Tasks = ausschließlich Code-Edits + Tests +
Verifikations-Builds.
</vcs_constraint>

<context>
@.planning/STATE.md

<interfaces>
<!-- Backend domain report (service-tier). committed_voluntary fehlt hier noch. -->
From service/src/reporting.rs (struct ShortEmployeeReport, Z. 147-161):
```rust
pub struct ShortEmployeeReport {
    pub sales_person: Arc<SalesPerson>,
    pub balance_hours: f32,
    pub dynamic_hours: f32,     // <- die "Soll"-Zahl
    pub expected_hours: f32,
    pub overall_hours: f32,
    pub vacation_hours: f32,
    pub sick_leave_hours: f32,
    pub holiday_hours: f32,
    pub unavailable_hours: f32,
    pub unpaid_leave_hours: f32,
    pub volunteer_hours: f32,
    pub custom_absence_hours: Arc<[CustomExtraHours]>,
}
```

From rest-types/src/lib.rs (ShortEmployeeReportTO, Z. 371-393):
```rust
pub struct ShortEmployeeReportTO {
    pub sales_person: SalesPersonTO,
    pub balance_hours: f32,
    pub expected_hours: f32,
    pub dynamic_hours: f32,
    pub overall_hours: f32,
    #[serde(default)]            // <- gleiches Pattern für committed_voluntary_hours verwenden
    pub volunteer_hours: f32,
}
// impl From<&service::reporting::ShortEmployeeReport> for ShortEmployeeReportTO { ... }
```

Bereits existierender Aggregat-Helper (NUTZEN, nicht neu bauen):
From service_impl/src/reporting.rs (Z. 101-110):
```rust
pub fn committed_voluntary_for_calendar_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> f32  // summiert committed_voluntary aller aktiven Rows der KW (CVC-03 / D-OVERLAP-AGG = SUM)
```
ACHTUNG: dieser Helper filtert NICHT auf cap/expected==0. Für die Anzeige unter
"Soll" ist das ok (wir wollen die zugesagte Zahl der Person zeigen), aber der
per-Person-Filter muss vor dem Aufruf auf die Rows dieser Person eingegrenzt
sein. In `get_week` liegen die Rows bereits person-gruppiert vor
(`working_hours: HashMap<sales_person_id, Vec<EmployeeWorkDetails>>`, Z. 742);
in `get_reports_for_all_employees` ist `working_hours` bereits per
`paid_employee.id` gefiltert (Z. 177-181).

From shifty-dioxus/src/state/employee_work_details.rs (WorkingHoursMini, Z. 7-30):
```rust
pub struct WorkingHoursMini {
    pub sales_person_id: Uuid,
    pub sales_person_name: ImStr,
    pub expected_hours: f32,
    pub dynamic_hours: f32,      // <- gerendert als "Soll"/Target
    pub actual_hours: f32,
    pub balance_hours: f32,
    pub background_color: ImStr,
    // committed_voluntary_hours: f32  <- NEU hinzufügen, Default 0.0
}
```

From shifty-dioxus/src/loader.rs (build_working_hours_mini, Z. 451-477):
baut WorkingHoursMini aus ShortEmployeeReportTO. Hier
`committed_voluntary_hours: report.committed_voluntary_hours` ergänzen.

From shifty-dioxus/src/component/working_hours_mini_overview.rs:
- CardsLayout (Z. 98-159): rendert "{actual_hours_str} / {dynamic_hours_str}h"
  als Soll-Zeile.
- TableLayout (Z. 161-296): Spalte `WorkingHoursTableTarget` rendert `target_str`
  (= dynamic_hours, Z. 243-246); Total-Zeile summiert dynamic_hours (Z. 285-288).
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Backend — committed_voluntary_hours in ShortEmployeeReport + TO durchreichen, plus bestätigender No-Contract-Test</name>
  <files>service/src/reporting.rs, service_impl/src/reporting.rs, rest-types/src/lib.rs</files>
  <behavior>
    Req 1 (bestätigend, KEIN Verhaltens-Change):
    - Test: Eine reine Freiwilligen-Zeile (`EmployeeWorkDetails` mit
      `expected_hours = 0.0`, `cap_planned_hours_to_expected = false`,
      `committed_voluntary = 5.0`) liefert über
      `committed_voluntary_for_calendar_week` 5.0 — d.h. ohne bezahlten Vertrag
      ist committed_voluntary verbuchbar/aggregierbar. (Pinnt den D-05-Pfad.)
    Req 2 (Datentransport):
    - `ShortEmployeeReport` trägt neues Feld `committed_voluntary_hours: f32`.
    - `get_week` befüllt es je Person aus den person-gruppierten work_hours der KW
      via `committed_voluntary_for_calendar_week(&person_rows, year, week)`.
    - `get_reports_for_all_employees` befüllt es analog (working_hours ist dort
      bereits per paid_employee.id gefiltert; year-Report → die committed-Summe
      der KW(s) im Range — konsistent zur Wochen-Semantik: für den Jahres-Report
      die committed-Summe der `until_week` verwenden, gleicher Aufruf
      `committed_voluntary_for_calendar_week(&working_hours, year, until_week)`).
    - `ShortEmployeeReportTO` trägt `#[serde(default)] committed_voluntary_hours: f32`
      und die `From`-Impl mappt es 1:1.
    - Test (rest-types): TO-Roundtrip mit committed_voluntary_hours = 2.5
      überlebt; Legacy-JSON ohne das Feld deserialisiert zu 0.0 (Backward-Compat,
      analog zu `committed_voluntary_hours_defaults_to_zero_when_absent` Z. 2299).
  </behavior>
  <action>
    1. `service/src/reporting.rs`: In `struct ShortEmployeeReport` (Z. 147-161)
       Feld `pub committed_voluntary_hours: f32,` ergänzen.
    2. `service_impl/src/reporting.rs`:
       a) In `get_week` (push bei Z. 892): vor dem push die committed-Summe der
          Person berechnen —
          `let committed_voluntary_hours = committed_voluntary_for_calendar_week(&working_hours, year, week);`
          (`working_hours` ist hier die person-lokale Vec aus dem Loop-Tuple,
          Z. 742) und ins Struct setzen.
          HINWEIS: `committed_voluntary_for_calendar_week` erwartet `&[EmployeeWorkDetails]`;
          die person-lokale `working_hours` ist bereits genau das.
       b) In `get_reports_for_all_employees` (push bei Z. 456): `working_hours`
          ist dort die per-`paid_employee.id` gefilterte `Arc<[EmployeeWorkDetails]>`
          (Z. 177-181). Setze
          `committed_voluntary_hours: committed_voluntary_for_calendar_week(&working_hours, year, until_week),`.
       c) Alle weiteren `ShortEmployeeReport { ... }`-Konstruktionen in Test-Modulen
          dieser Datei um `committed_voluntary_hours: 0.0,` ergänzen (rustc
          erzwingt Exhaustivität — alle Stellen finden via `cargo build`).
    3. `rest-types/src/lib.rs`: In `ShortEmployeeReportTO` (Z. 371-379)
       `#[serde(default)] pub committed_voluntary_hours: f32,` ergänzen; in der
       `From`-Impl (Z. 382-392) `committed_voluntary_hours: report.committed_voluntary_hours,`
       mappen. Alle Test-Fixtures in dieser Datei, die `ShortEmployeeReportTO { ... }`
       voll konstruieren, ergänzen (rustc-geführt).
    4. Tests:
       - `service_impl/src/reporting.rs` Modul
         `test_committed_voluntary_for_calendar_week` (ab Z. 1717) um Test
         `committed_voluntary_bookable_without_paid_contract` erweitern: eine Row
         mit `expected_hours = 0.0`, `cap_planned_hours_to_expected = false`,
         `committed_voluntary = 5.0` aktiv in (2026, KW 10) → Helper liefert 5.0.
         (Bestätigt Req 1: Ehrenamt ohne Vertrag verbuchbar/aggregierbar.)
       - `rest-types/src/lib.rs`: Test
         `short_employee_report_committed_voluntary_roundtrip` (TO mit 2.5
         serialisiert→deserialisiert unverändert) + Test
         `short_employee_report_committed_voluntary_defaults_zero_when_absent`
         (Legacy-JSON ohne Feld → 0.0).
  </action>
  <verify>
    <automated>cd /home/neosam/programming/rust/projects/shifty/shifty-backend && nix develop --command cargo test -p service_impl reporting committed_voluntary 2>&1 | tail -20 && nix develop --command cargo test -p rest-types short_employee_report 2>&1 | tail -20</automated>
  </verify>
  <done>
    `cargo build --workspace` grün; neue Tests grün; `ShortEmployeeReportTO`
    serialisiert `committed_voluntary_hours`; No-Contract-Aggregations-Test grün
    (Req 1 gepinnt); kein Snapshot-Version-Bump.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Frontend — committed_voluntary_hours in WorkingHoursMini + geschwellte (>=0.5) separate Anzeige unter "Soll" + i18n</name>
  <files>shifty-dioxus/src/state/employee_work_details.rs, shifty-dioxus/src/loader.rs, shifty-dioxus/src/component/working_hours_mini_overview.rs, shifty-dioxus/src/i18n/mod.rs, shifty-dioxus/src/i18n/de.rs, shifty-dioxus/src/i18n/en.rs, shifty-dioxus/src/i18n/cs.rs</files>
  <behavior>
    Req 2 + Req 3 (Anzeige + Schwelle):
    - Pure Helper `show_committed_voluntary(committed: f32) -> bool` gibt
      `committed >= 0.5` zurück. Tests: 0.49 → false, 0.5 → true, 0.0 → false,
      5.0 → true. (Req-3-Schwelle, exakt >= 0.5.)
    - TableLayout: in der Target/Soll-Spalte (Z. 243-246) wird UNTER der
      `{target_str}h`-Zeile eine zweite, kleine Zeile gerendert mit dem
      Ehrenamt-Wert (z.B. `"+ {committed}h {LabelEhrenamt}"`) — NUR wenn
      `show_committed_voluntary(row.committed_voluntary_hours)` true ist.
      Die Soll-Zahl selbst (`target_str`) bleibt unverändert (additiv, kein
      Reinrechnen).
    - CardsLayout: analog eine kleine Ehrenamt-Zeile unter
      "{actual}/{dynamic}h", nur bei >= 0.5.
    - Total-Zeile (Z. 285-288): Ehrenamt-Summe nur anzeigen, wenn
      `show_committed_voluntary(Σ committed_voluntary_hours)` true.
    - i18n: neuer Key `Key::CommittedVoluntaryShort` (Kurzlabel "Ehrenamt" /
      "Volunteer" / "Dobrovolnictví") in allen 3 Locales.
  </behavior>
  <action>
    1. `shifty-dioxus/src/state/employee_work_details.rs`: In `WorkingHoursMini`
       (Z. 7-16) Feld `pub committed_voluntary_hours: f32,` ergänzen; in
       `Default` (Z. 18-30) `committed_voluntary_hours: 0.0,`. Alle Test-Konstruktionen
       von `WorkingHoursMini` workspace-weit (rustc-geführt) ergänzen — insbesondere
       die `make_row` + `WorkingHoursMini { ... }`-Literale in
       `working_hours_mini_overview.rs` (Z. 368, 461, 492, 607, 632, 662, 714, 723,
       798, 829).
    2. `shifty-dioxus/src/loader.rs`: In `build_working_hours_mini` (Z. 465-476)
       `committed_voluntary_hours: report.committed_voluntary_hours,` ergänzen.
       Loader-Test-Fixture `make_report` (Z. 485-501) um
       `committed_voluntary_hours: 0.0,` ergänzen (ShortEmployeeReportTO hat
       `#[serde(default)]`, aber das Test-Literal konstruiert voll → rustc-geführt).
    3. `shifty-dioxus/src/component/working_hours_mini_overview.rs`:
       a) Pure Helper hinzufügen:
          `pub(crate) fn show_committed_voluntary(committed: f32) -> bool { committed >= 0.5 }`
       b) TableLayout (Z. 243-246): die Target-`td` so erweitern, dass unter
          `{target_str}h` bedingt eine zweite Zeile rendert:
          `if show_committed_voluntary(working_hour.committed_voluntary_hours) { div { class: "text-small font-normal text-good", "+ {format_hours(working_hour.committed_voluntary_hours, 1)}h {committed_label}" } }`
          (committed_label = `i18n.t(Key::CommittedVoluntaryShort)`).
       c) CardsLayout (Z. 137-143): analoge bedingte Zusatzzeile unter der
          hours-Zeile.
       d) Total-Zeile (Z. 285-288): `let total_committed: f32 = props.rows.iter().map(|r| r.committed_voluntary_hours).sum();`
          und in der Target-Total-`td` bedingt `if show_committed_voluntary(total_committed) { ... }` anzeigen.
       e) Tests im `#[cfg(test)] mod tests` ergänzen:
          - `show_committed_voluntary_threshold`: 0.49→false, 0.5→true, 0.0→false,
            5.0→true.
          - `committed_voluntary_line_rendered_when_at_or_above_threshold`: Row mit
            committed_voluntary_hours = 2.0 → gerenderte HTML enthält
            "2.0" + committed_label (Table-Layout SSR-Render, analog bestehendem
            Test-Pattern in dieser Datei).
          - `committed_voluntary_line_hidden_below_threshold`: Row mit
            committed_voluntary_hours = 0.3 → HTML enthält NICHT die Ehrenamt-Zeile.
    4. i18n: in `shifty-dioxus/src/i18n/mod.rs` `Key::CommittedVoluntaryShort`
       zur `Key`-Enum hinzufügen; in `de.rs`/`en.rs`/`cs.rs` jeweils
       `i18n.add_text(Locale::De, Key::CommittedVoluntaryShort, "Ehrenamt");`
       (DE), `"Volunteer"` (EN), `"Dobrovolnictví"` (CS). Auf korrektes
       `Locale::*`-Tag je Datei achten (Pitfall: kein `Locale::En` in `de.rs`).
       Falls in dieser Datei ein `i18n_*_present_in_all_locales`-Test existiert,
       läuft der neue Key automatisch mit; sonst keine zusätzliche Test-Pflicht.
  </action>
  <verify>
    <automated>cd /home/neosam/programming/rust/projects/shifty/shifty-backend/shifty-dioxus && cargo test show_committed_voluntary 2>&1 | tail -20 && cargo test working_hours_mini 2>&1 | tail -20 && cargo test i18n 2>&1 | tail -15</automated>
  </verify>
  <done>
    `cargo test` (shifty-dioxus) grün inkl. neuer Schwellen- und Render-Tests;
    Ehrenamt-Zeile erscheint unter Soll nur bei >= 0.5; i18n-Key in allen 3
    Locales; Soll-Zahl selbst unverändert (kein Reinrechnen).
  </done>
</task>

<task type="auto">
  <name>Task 3: Verifikation — Workspace-Build, Tests, WASM-Build-Gate</name>
  <files>(keine Code-Änderung; Verifikation)</files>
  <action>
    Vollständige Verifikation, dass beide Layer sauber bauen und testen und der
    WASM-Build-Gate hält (CLAUDE.md-Pflicht für Frontend-Änderungen):
    1. Backend: `cargo build --workspace` + `cargo test --workspace`.
    2. Frontend: `cargo test` in `shifty-dioxus/`.
    3. WASM-Build-Gate: `cargo build --target wasm32-unknown-unknown` in
       `shifty-dioxus/` (ggf. via `nix develop` für die wasm32-Toolchain).
    Bei Fehlern: fixen, bis alle drei grün. KEINE Commits (jj-only, User
    committet manuell).
  </action>
  <verify>
    <automated>cd /home/neosam/programming/rust/projects/shifty/shifty-backend && nix develop --command cargo test --workspace 2>&1 | tail -25 && cd shifty-dioxus && nix develop --command bash -c "cargo test 2>&1 | tail -15 && cargo build --target wasm32-unknown-unknown 2>&1 | tail -10"</automated>
  </verify>
  <done>
    `cargo build --workspace` + `cargo test --workspace` grün (Backend);
    `cargo test` grün (shifty-dioxus); `cargo build --target wasm32-unknown-unknown`
    exit 0. Keine offenen Compile-Fehler. Nichts committet.
  </done>
</task>

</tasks>

<verification>
- Req 1: Aggregations-Test beweist, dass committed_voluntary einer Zeile mit
  `expected_hours = 0` / `cap = false` (= kein bezahlter Vertrag) verbucht wird
  → bereits möglich, jetzt gegen Regression gepinnt.
- Req 2: `committed_voluntary_hours` reist Backend-Report → TO → WorkingHoursMini
  → Render unter "Soll" (separate additive Zeile, Soll-Zahl unverändert).
- Req 3: `show_committed_voluntary(committed) == (committed >= 0.5)` gated alle
  Render-Sites (Card, Table-Row, Total).
- Snapshot: kein `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump (Live-Report-only, kein
  persistierter value_type berührt).
- i18n: `Key::CommittedVoluntaryShort` in de/en/cs.
- jj-only: keine Commits durch das Tooling.
</verification>

<success_criteria>
- `cargo build --workspace` + `cargo test --workspace` (Backend) grün.
- `cargo test` (shifty-dioxus) grün, inkl. Schwellen- + Render- + Roundtrip-Tests.
- `cargo build --target wasm32-unknown-unknown` exit 0.
- Ehrenamt-Stunden erscheinen als separate Zeile unter "Soll" in der
  Mitarbeiter-Wochenübersicht, NUR wenn >= 0.5.
- Soll-Zahl, Auslastung und Balance bleiben rechnerisch unverändert.
- Keine Commits durch den Executor (User committet via jj).
</success_criteria>

<output>
Nach Abschluss: `.planning/quick/260624-qgk-stunden-ohne-arbeitsvertrag-als-ehrenamt/260624-qgk-SUMMARY.md`
erstellen (was geändert, Test-Zahlen, Bestätigung Req-1-Befund, kein
Snapshot-Bump). KEINE Commits.
</output>
