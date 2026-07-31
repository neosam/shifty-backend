---
phase: 57-f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus
plan: 04
subsystem: reporting
tags: [rebooking, reporting, balance, voluntary-hours, tdd, acceptance-test, cap-overflow]
status: complete
requires:
  - 57-02-SUMMARY.md (Filter-Fix — erst dadurch wirkt das Rebooking-Paar auf der Leseseite)
  - 57-03-SUMMARY.md (Balance-Deckel — liefert die gebuchten 2 h, die dieser Test als Paar setzt)
  - service_impl/src/test/rebooking_neutrality.rs (Harness-Bauvorlage)
  - service/src/shiftplan_report.rs (ShiftplanReportDay-Shape)
provides:
  - "Referenz-Szenario D-57-02 als ausführbarer Ende-zu-Ende-Test (VOL-ACCT-04)"
  - "Gegenprobe Übererfüllung 3h/20h (REB-AUTO-06)"
  - "Shiftplan-getriebener Reporting-Harness build_reporting_service(extra_hours, week_10_hours, week_11_hours)"
affects:
  - Plan 05 (Snapshot-Bump 12→13 — dieser Test belegt die geänderte Computation an konkreten Zahlen)
  - Plan 06 (Docs-Update F14 §3 / F07 kann die Zahlen 8/12/−2 → 10/10/0 zitieren)
tech-stack:
  added: []
  patterns:
    - "Ehrenamt-Stunden im Test aus dem Shiftplan-Cap-Überlauf erzeugen statt aus manuellen VolunteerWork-ExtraHours — übt den Produktions-Pfad"
    - "Kontostände statt Deltas assertieren: der Test nennt die Zahlen, mit denen der User den Fehler beschrieben hat"
    - "Doc-Kommentar als Herleitungs-Tabelle (Woche → raw_shiftplan → Cap → bezahlt/Ehrenamt)"
key-files:
  created:
    - service_impl/src/test/rebooking_reference_scenario.rs
  modified:
    - service_impl/src/test/mod.rs
decisions:
  - "Bezahlte Stunden werden über EmployeeReport::overall_hours assertiert (gedeckelte Shiftplan-Stunden + balance-relevante ExtraHours); balance_hours + expected_hours dient im Basis-Test als zusätzliche Gegenprobe desselben Werts"
  - "include_carryover = false — das Szenario hat keine Vorjahresgeschichte; der Carryover-Mock liefert ohnehin None"
  - "Der Test assertiert bewusst NICHT auf week_excess — die Schreibseite ist mit denselben Zahlen in run_for_week_balance_cap_binds (Plan 03) abgedeckt"
  - "VOL-ACCT-04 und REB-AUTO-06 sind mit diesem Plan vollständig belegt; beide Checkboxen standen seit Plan 01 bereits auf [x] (Requirements-Neufassung), inhaltlich ist die Coverage erst jetzt gemessen"
metrics:
  duration: ~15min
  completed: 2026-07-31
---

# Phase 57 Plan 04: Referenz-Szenario D-57-02 als Ende-zu-Ende-Test Summary

Das Szenario, mit dem der User den Fehler beschrieben hat, ist als Test hinterlegt
und grün — beide Hälften, mit den konkreten Kontoständen 8/12/−2 → 10/10/0 statt
mit abstrakten Deltas, und mit Ehrenamt-Stunden, die aus dem Shiftplan-Cap-Überlauf
entstehen statt aus direkt gesetzten ExtraHours.

## Was gebaut wurde

### Task 1 — Shiftplan-getriebener Harness + Basis-Szenario 3 h / 17 h (Commit `e2b6be8d`)

**Neues Testmodul** `service_impl/src/test/rebooking_reference_scenario.rs`,
registriert in `service_impl/src/test/mod.rs` direkt hinter
`rebooking_neutrality` (Gruppierung der `rebooking_*`-Module erhalten).

**Datei-Doku-Kopf.** Zitiert D-57-02 im Wortlaut des Users und benennt den
Unterschied zu `rebooking_neutrality.rs`: dort entstehen die Ehrenamt-Stunden aus
direkt gesetzten `VolunteerWork`-Rows, hier ausschließlich aus dem
Cap-Überlauf (`cap_planned_hours_to_expected = true` →
`apply_weekly_cap` → `auto_volunteer_hours` in `hours_per_week`) — also über den
Pfad, der sie in der Produktion tatsächlich erzeugt.

**Fixtures**, gespiegelt aus `rebooking_neutrality.rs` mit eigenem
UUID-Präfix `0x…_0057_…` (keine Kollision): `sp_id()`, `work_details()`
(inhaltlich unverändert — 5 h/Woche Mo–Fr, KW 10..=20/2026, Cap aktiv,
`committed_voluntary: 5.0`), `make_sales_person()`, `range()`.

`range()` liefert Mo 2026-03-02 .. So 2026-03-15. Nachgerechnet: der 2026-03-02
ist Montag der ISO-Woche 10/2026 (ISO-Woche 1 beginnt am 2025-12-29, +63 Tage),
der 2026-03-15 ist Sonntag der KW 11 — exakt zwei Wochen, also `expected_hours`
von 10.0 über die Range.

**Harness** `build_reporting_service(extra_hours, week_10_hours, week_11_hours)`.
Aufbau und `ReportingDeps` von `rebooking_neutrality.rs` übernommen; einziger
inhaltlicher Unterschied ist der `MockShiftplanReportService`, der zwei echte
`ShiftplanReportDay`-Einträge liefert (KW 10 / KW 11, beide `year: 2026`,
`day_of_week: Monday`). Die Tages-Granularität ist irrelevant, weil
`hours_per_week` pro ISO-Woche summiert. `extract_shiftplan_report_for_week`
liefert dieselbe Liste, damit ein späterer Wochenpfad denselben Fixture-Stand
sieht.

**Rebooking-Fixture** `rebooking_pair(h)` — genau zwei Rows, beide
`source: ExtraHoursSource::Rebooking`, datiert auf Mi 2026-03-11 (KW 11/2026):
`-h` `VolunteerWork` und `+h` `ExtraWork`. Das sind die **einzigen**
ExtraHours-Rows der Datei.

**Test `rebooking_reference_scenario_balances_account`** mit
Herleitungs-Tabelle im Doc-Kommentar:

| Woche | `raw_shiftplan` | `expected` | Cap? | bezahlt | Ehrenamt |
|---|---|---|---|---|---|
| KW 10 | 3 | 5 | nein | 3 | 0 |
| KW 11 | 17 | 5 | ja | 5 | 12 |
| Summe | 20 | 10 | | **8** | **12** |

Assertiert wird zuerst die Ausgangslage (`expected_hours` 10.0,
`overall_hours` 8.0, `volunteer_hours` 12.0, `balance_hours` −2.0, plus die
Gegenprobe `balance + expected == 8.0`), dann derselbe Report mit
`rebooking_pair(2.0)`: `overall_hours` 10.0, `volunteer_hours` 10.0,
`balance_hours` 0.0. Alle Vergleiche über den `#[track_caller]`-Helper
`assert_close` mit Toleranz 1e-3 und einer Meldung, die Soll und Ist nennt.

### Task 2 — Gegenprobe Übererfüllung 3 h / 20 h (Commit `1fdc8a1f`)

**Test `rebooking_reference_scenario_overachievement_keeps_cap`**, identisch
aufgebaut, aber mit `week_11_hours = 20.0`:

| Woche | `raw_shiftplan` | `expected` | Cap? | bezahlt | Ehrenamt |
|---|---|---|---|---|---|
| KW 10 | 3 | 5 | nein | 3 | 0 |
| KW 11 | 20 | 5 | ja | 5 | 15 |
| Summe | 23 | 10 | | **8** | **15** |

Erste Hälfte: 8.0 / 15.0 / −2.0 — die Balance bleibt trotz drei zusätzlicher
Arbeitsstunden unverändert, weil der Cap die bezahlten Stunden der KW 11 bei 5 h
hält und der gesamte Mehraufwand ins Ehrenamt-Ist läuft. Zweite Hälfte mit
`rebooking_pair(2.0)`: 10.0 / 13.0 / 0.0.

Zusätzlich eine explizite Fachaussage-Assertion: die Freiwilligen-Zusage über die
Range beträgt `2 × committed_voluntary` = 10.0 h; `volunteer_hours` nach dem
Übertrag minus dieser Zusage ergibt 3.0 — drei Stunden stehen weiterhin über der
Zusage freiwillig zu Buche, der Übertrag hat sie also nicht unterschritten.

**Zahlen-Trennung im Doc-Kommentar** (Prohibition, s. u.): der Wochen-Input des
Auto-Cron für KW 11 ist ein unbeschränkter Überschuss von **10** (`ist` 15 −
`committed` 5, diese eine Woche); die in der ROADMAP genannte „Überschuss 5" ist
das **Perioden-Delta** `Ist − Soll` über beide Wochen (15 − 10). Beide Zahlen sind
korrekt, beziehen sich aber auf verschiedene Ranges; gebucht werden in beiden
Lesarten 2 h, begrenzt durch den Headroom aus Balance −2.

Der Datei-Doku-Kopf wurde um den Verweis auf die Schreibseite ergänzt:
`service_impl/src/test/voluntary_rebooking.rs::run_for_week_balance_cap_binds`
(Plan 03) deckt dieselben Zahlen auf der Write-Seite ab.

## Erstlauf-Befund

Der Plan verlangte ausdrücklich, einen roten Erstlauf als echte Entdeckung zu
melden statt die Erwartung anzupassen (T-57-11). **Beide Tests waren beim ersten
Lauf grün** — der Filter-Fix aus Plan 02 und der Balance-Deckel aus Plan 03 tragen
das Referenz-Szenario vollständig. Keine Assertion wurde nachträglich verändert;
die Erwartungswerte stammen unverändert aus D-57-02 und der Nachrechnung in
57-RESEARCH.md §„Balance-Deckel: Nachrechnung des Referenz-Szenarios".

## Verifikation

### Hard Gates

| Gate | Ergebnis |
|---|---|
| `cargo build` | ✅ Finished |
| `cargo test --workspace` | ✅ 0 failed (831 in `service_impl` — 829 vorher + 2 neue) |
| `cargo clippy --workspace -- -D warnings` | ✅ exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ exit 0 (Testcode mit abgedeckt) |

### Akzeptanzkriterien

| Kriterium | Erwartet | Gemessen |
|---|---|---|
| `test -f service_impl/src/test/rebooking_reference_scenario.rs` | OK | OK |
| `grep -c 'pub mod rebooking_reference_scenario;' test/mod.rs` | 1 | 1 |
| `grep -c 'fn rebooking_reference_scenario_balances_account'` | 1 | 1 |
| `grep -c 'fn rebooking_reference_scenario_overachievement_keeps_cap'` | 1 | 1 |
| `grep -c 'ExtraHoursSource::Manual'` | 0 | 0 |
| `grep -c 'ShiftplanReportDay'` | ≥ 1 | 5 |
| `grep -c 'cap_planned_hours_to_expected: true'` | 1 | 1 |
| `grep -c 'committed_voluntary: 5.0'` | 1 | 1 |
| `grep -c 'run_for_week_balance_cap_binds'` | ≥ 1 | 2 |
| `cargo test -p service_impl rebooking_reference_scenario` | 2 passed | 2 passed |

### TDD-Gate-Sequenz

Dieser Plan ist ein reiner **Acceptance-Test-Plan** — er ändert keinen
Produktionscode. Die RED/GREEN-Sequenz liegt entsprechend nicht in diesem Plan,
sondern in den Plänen 02 (`55e86ec3` RED → `d0550077` GREEN) und 03
(`90941312` RED → `5432b9c1` GREEN); dieser Plan misst deren GREEN-Zustand am
Referenz-Szenario. Beide Commits hier sind daher korrekt vom Typ `test(...)`.
Ein `feat(...)`-Gate wäre hier nur dann fällig gewesen, wenn der Erstlauf rot
gewesen wäre — er war grün (s. o.).

## Prohibitions — Status

| Prohibition | Status | Beleg |
|---|---|---|
| Wochen-Input des Auto-Cron (10) NICHT mit dem Perioden-Delta `Ist − Soll` (5) verwechseln | eingehalten | Der Doc-Kommentar von `..._overachievement_keeps_cap` benennt beide Zahlen mit ihrer Range; der Testkörper assertiert ausschließlich auf `EmployeeReport`-Felder über die 2-Wochen-Range, nie auf `week_excess` |
| Ehrenamt-Stunden NICHT über direkt gesetzte VolunteerWork-ExtraHours simulieren | eingehalten | `grep -c 'ExtraHoursSource::Manual'` = 0; die einzigen ExtraHours-Rows sind die beiden Beine aus `rebooking_pair()`, beide `source: Rebooking`; die 12 bzw. 15 Ehrenamt-Stunden entstehen in `apply_weekly_cap` |
| Erwartung nicht an die Implementierung anpassen (T-57-11) | eingehalten | Erstlauf beider Tests grün; keine Assertion nachträglich geändert |

## Deviations from Plan

Keine — der Plan wurde exakt wie geschrieben ausgeführt.

Zwei Präzisierungen, die der Plan ausdrücklich offengelassen hatte:

1. **Welches Feld die bezahlten Stunden trägt.** Der Plan schrieb „den im Report
   tatsächlich vorhandenen Ausdruck verwenden und im Kommentar benennen". Das ist
   `EmployeeReport::overall_hours` (= `shiftplan_hours_by_week +
   overall_extra_work_hours`, `reporting.rs:1827`). Der Basis-Test assertiert
   zusätzlich `balance_hours + expected_hours == 8.0` als Gegenprobe desselben
   Werts.
2. **`extract_shiftplan_report_for_week` mit-gestubbt.** Der Plan sah das für den
   Fall vor, dass der Report-Pfad ihn erwartet. `get_report_for_employee_range`
   ruft ihn nicht, der Stub ist also derzeit ungenutzt — er bleibt drin, damit ein
   späterer Wochenpfad denselben Fixture-Stand sieht statt einer leeren Liste
   (Konsistenz-Falle vermieden).

## Auth-Gates

Keine.

## Known Stubs

Keine.

## Requirements-Status

`requirements mark-complete VOL-ACCT-04 REB-AUTO-06` gemeldet:
`already_complete` für beide. Die Checkboxen stehen seit Plan 01
(`f169b176`, Requirements-Neufassung) auf `[x]` — dort wurden sie beim Anlegen
gesetzt, nicht beim Erfüllen. **Inhaltlich sind beide Requirements erst mit
diesem Plan gemessen:**

- **VOL-ACCT-04** — Akzeptanzbedingung ist das Referenz-Szenario D-57-02; Plan 02
  lieferte die Invarianten I-1/I-2 als Deltas, dieser Plan die Kontostände.
- **REB-AUTO-06** — die im Requirement wörtlich geforderte Gegenprobe („Woche 2
  mit 20 h … Übertrag bleibt 2, Balance landet auf 0") ist
  `rebooking_reference_scenario_overachievement_keeps_cap` auf der Leseseite,
  zusammen mit `run_for_week_balance_cap_binds` (Plan 03) auf der Schreibseite.

**Nebenbefund, out of scope:** Die Traceability-Tabelle in
`.planning/REQUIREMENTS.md` führt *alle* 19 Requirements als `Pending` — auch die
in den Phasen 54–56 abgeschlossenen. Das ist eine projektweite Alt-Drift der
Tabelle, nicht durch diesen Plan verursacht; nicht angefasst.

## Docs-Freshness-Gate

Dieser Plan berührt keine Trigger-Datei aus der CLAUDE.md-Tabelle: geändert wurden
ausschließlich `service_impl/src/test/rebooking_reference_scenario.rs` (neu) und
`service_impl/src/test/mod.rs` (eine Modul-Zeile). Kein `service/**`, kein
`dao/**`, keine Migration, kein `main.rs`, kein `reporting.rs`,
kein `billing_period_report.rs`. Das Docs-Update der Phase liegt unverändert bei
**Plan 06 (Wave 5)**.

## Threat Flags

Keine neue Angriffsfläche. Register-Status:

- **T-57-11** (Test passt sich der Implementierung an) — mitigiert: Erstlauf beider
  Tests grün, keine Assertion nachträglich verändert; die Erwartungswerte stammen
  aus D-57-02 / 57-RESEARCH.md.
- **T-57-12** (Fixture setzt Ehrenamt-Stunden direkt) — mitigiert:
  `ExtraHoursSource::Manual` kommt in der Datei null Mal vor.
- **T-57-13** (Testdaten) — akzeptiert: synthetische UUIDs, Name „Reference 57".
- **T-57-SC** — `Cargo.toml` unverändert, keine Paket-Installation.

## Wave-Disziplin

Innerhalb der deklarierten `files_modified` geblieben:
`service_impl/src/billing_period_report.rs` und die billing-period-Testmodule
(Plan 05, Wave 4) wurden nicht angefasst — `git status --porcelain` war nach
beiden Commits leer.

## Anschluss

- **Plan 05** hat mit diesem Test den harten Beleg für den Snapshot-Bump 12→13 an
  konkreten Zahlen: dieselbe Periode liefert vor und nach dem Rebooking-Paar
  unterschiedliche Balance-/Volunteer-Werte.
- **Plan 06** kann die Zahlen 8/12/−2 → 10/10/0 (und 8/15/−2 → 10/13/0) direkt in
  F14 §3 / F07 zitieren, mit Verweis auf diese Testdatei als lebenden Beleg.

## Commits

| Commit | Inhalt |
|---|---|
| `e2b6be8d` | `test(57-04)`: Referenz-Szenario D-57-02 als Ende-zu-Ende-Test (VOL-ACCT-04) |
| `1fdc8a1f` | `test(57-04)`: Gegenprobe Uebererfuellung — Deckel haelt, Zusage bleibt (REB-AUTO-06) |

## Self-Check: PASSED

- `service_impl/src/test/rebooking_reference_scenario.rs` — FOUND
- `service_impl/src/test/mod.rs` — FOUND
- `.planning/phases/57-.../57-04-SUMMARY.md` — FOUND
- Commit `e2b6be8d` — FOUND
- Commit `1fdc8a1f` — FOUND
- `cargo build` / `cargo test --workspace` / `cargo clippy --workspace -- -D warnings` — alle grün
