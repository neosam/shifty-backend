---
phase: 57-f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus
plan: 03
subsystem: rebooking-automation
tags: [rebooking, auto-cron, backfill, balance-cap, tdd, reb-auto-06]
status: complete
requires:
  - 57-02-SUMMARY.md (der Filter-Fix — erst dadurch bildet `balance_hours` die Rebooking-Wirkung ab)
  - service/src/reporting.rs (ReportingService::get_report_for_employee_range, bereits DI-Dependency)
  - shifty-utils (ShiftyDate::first_day_in_year, ShiftyDate::new)
provides:
  - "Vier-Parameter-`week_excess` mit Balance-Deckel (REB-AUTO-06, Invarianten I-3/I-4)"
  - "Year-to-date-Balance-Read in `run_for_week` (include_carryover = true, D-57-07)"
  - "Derselbe Read in `backfill_range` — Preview == Apply (Invariante I-7)"
  - "`reporting_service(balance: f32)`-Helper in rebooking_backfill.rs und voluntary_rebooking_scheduler.rs"
  - "`run_for_week_balance_cap_binds` — CI-Wächter gegen das Kopieren der suggest()-Range (Pitfall 3)"
affects:
  - Plan 04 (Ende-zu-Ende-Referenz-Szenario D-57-02 inkl. Gegenprobe Übererfüllung)
  - Plan 06 (Docs-Update F14 §3 beschreibt die Deckel-Formel)
tech-stack:
  added: []
  patterns:
    - "Deckel als reine Formel im pure-fn-Kern (`min(unconstrained, headroom)`), Balance-Beschaffung an der Call-Site"
    - "mockall `.withf()` als struktureller Range-Wächter statt Prosa im Kommentar"
    - "Zwei sich widersprechende Preview-Tests (3.0 bei Balance −8, 2.0 bei Balance −2) als Platzhalter-Falle"
key-files:
  created: []
  modified:
    - service_impl/src/rebooking_reconciliation.rs
    - service_impl/src/test/voluntary_rebooking.rs
    - service_impl/src/test/rebooking_backfill.rs
    - service_impl/src/test/voluntary_rebooking_scheduler.rs
decisions:
  - "D-57-07 umgesetzt: Balance-Quelle ist year-to-date ab `first_day_in_year(iso_year)` bis Sonntag der verarbeiteten ISO-Woche mit `include_carryover = true` — NICHT die isolierte Wochen-Range von suggest()/approve()"
  - "Der Balance-Read steht bewusst NACH dem Toggle- und dem Locked-Gate: ausgeschalteter Cron / gesperrte Woche kehren weiterhin ohne Report-Roundtrip zurück"
  - "Kein gemeinsamer Helper für die beiden Balance-Reads — der Plan pinnt `get_report_for_employee_range` == 4 und `first_day_in_year` == 2 als Akzeptanzkriterium; die Call-Sites bleiben lokal lesbar"
  - "Chronologische Backfill-Konsistenz über die geteilte `read_tx` (SQLite-Sichtbarkeit innerhalb der Transaktion) statt eigener inkrementeller Buchführung — D-57-07 erlaubt, aber fordert die Optimierung nicht"
  - "REB-AUTO-06 wurde NICHT als complete markiert — die Gegenprobe des Requirements ist das Ende-zu-Ende-Szenario, das Plan 04 schreibt (dessen Frontmatter REB-AUTO-06 ebenfalls führt)"
metrics:
  duration: ~20min
  completed: 2026-07-31
---

# Phase 57 Plan 03: Balance-Deckel für den Auto-Cron Summary

Die Automatik gleicht das bezahlte Stundenkonto nur bis 0 aus: `week_excess` deckelt
den Wochen-Überschuss auf den Headroom der year-to-date-Balance — im Schreib-Pfad
`run_for_week` und im Dry-Run-Preview `backfill_range` mit demselben Wert.

## Was gebaut wurde

### Task 1 — RED: Formel-Matrix und drei Deckel-Verhaltenstests (Commit `90941312`)

**(a) `week_excess_matrix` auf vier Parameter erweitert.** Die fünf Bestandszeilen
bekamen `-8.0` als Balance — ein Headroom von 8, bei dem der Deckel in keiner von
ihnen bindet, sodass die Erwartungswerte 5.0 / 0.0 / 0.0 / 0.0 / 0.0 unverändert
gültig bleiben. Vier neue Zeilen kamen hinzu:

| Zeile | Erwartung | Belegt |
|---|---|---|
| `week_excess(15.0, 0.0, 5.0, -2.0)` | 2.0 | Deckel bindet (unconstrained wäre 10), Gegenprobe D-57-02, Invariante I-3 |
| `week_excess(10.0, 0.0, 5.0, 0.0)` | 0.0 | Balance 0 → kein Headroom (I-4) |
| `week_excess(10.0, 0.0, 5.0, 3.0)` | 0.0 | positive Balance → kein Headroom (I-4) |
| `week_excess(10.0, 0.0, 5.0, f32::NAN)` | 0.0 | Guard deckt den vierten Parameter ab (T-57-07) |

Der Doc-Kommentar wurde von „max(ist − (soll + committed), 0)" auf die Deckel-Formel
umgeschrieben und referenziert REB-AUTO-06 neben REB-AUTO-01.

**(b) `run_for_week_balance_cap_binds`.** Vorlage ist `run_for_week_books_excess`,
abweichend `stats_service(15.0, 5.0)` und ein eigens gebauter `MockReportingService`
mit `.withf(…)` **vor** dem `.returning(…)`:

```rust
.withf(|sales_person_id, from, to, include_carryover, _ctx, _tx| {
    *sales_person_id == sp_id()
        && *from == ShiftyDate::first_day_in_year(2026)
        && *to == ShiftyDate::new(2026, 12, DayOfWeek::Sunday).expect("KW 12/2026 has a Sunday")
        && *include_carryover
})
```

Das ist der strukturelle Schutz gegen Pitfall 3: `suggest()` und `approve()` lesen die
Balance für die **isolierte** ISO-Woche ohne Carryover. Wer dieses naheliegende Muster
nach `run_for_week` kopiert, färbt den Test rot, statt still den falschen Deckel zu
bauen. Assertionen: `Booked { hours: 2.0 }`, genau zwei ExtraHours-Rows, −2.0
VolunteerWork / +2.0 ExtraWork.

**(c) `run_for_week_non_negative_balance_skips`.** Vorlage
`run_for_week_no_excess_skips` (beide Write-Mocks ohne Expect → jeder Write paniked),
`stats_service(15.0, 5.0)` + `reporting_service(0.0)` → `Skipped { NoExcess }`.

**(d) `rebooking_backfill.rs` parametrisiert.** `reporting_service()` →
`reporting_service(balance: f32)`, alle sechs Aufrufstellen auf `-8.0` — dadurch
bleibt insbesondere `excess = 8-5 = 3` in `backfill_dry_run_writes_nothing` gültig
(Headroom 8). Neu: `backfill_dry_run_preview_respects_balance_cap` mit
`stats_service(15.0, 5.0)` + `reporting_service(-2.0)` → genau ein
`BackfillPreviewItem` mit `excess == 2.0` und `would_book == true`.

**Gemessener RED-Zustand:** exakt 9 Fehler, alle
`error[E0061]: this function takes 3 arguments but 4 arguments were supplied` — je
eine pro Matrix-Zeile, keine weiteren Fehlerklassen. Das ist zugleich der Beleg, dass
die `.withf`-Signatur und die neuen Testrümpfe typkorrekt sind: sie hätten sonst
zusätzliche Fehlercodes produziert.

### Task 2 — GREEN: Deckel + zwei Balance-Reads (Commit `5432b9c1`)

**(a) `week_excess` auf vier Parameter.**

```rust
pub(crate) fn week_excess(ist: f32, soll: f32, committed_voluntary: f32, balance: f32) -> f32 {
    if !ist.is_finite() || !soll.is_finite() || !committed_voluntary.is_finite() || !balance.is_finite() {
        return 0.0;
    }
    let unconstrained = ist - (soll + committed_voluntary);
    let unconstrained = if unconstrained > 0.0 { unconstrained } else { 0.0 };
    let headroom = if -balance > 0.0 { -balance } else { 0.0 };
    unconstrained.min(headroom)
}
```

Der Doc-Kommentar erklärt den Deckel in Kontobegriffen („die Automatik gleicht das
bezahlte Stundenkonto aus, hebt es aber nie über 0") und grenzt den HR-Pfad
ausdrücklich ab: `proposed_hours = min(|balance|, ist)` bleibt ungedeckelt, weil ein
Mensch die Entscheidung trägt (D-57-01, Invariante I-5).

**(b) `run_for_week` — Balance-Read.** Der alte Kommentar („wir lesen ihn NICHT, um
den Write-Pfad schlank zu halten") ist entfallen; an seiner Stelle steht die
Begründung nach D-57-07 plus der ausdrückliche Pitfall-3-Warnhinweis. Der Read steht
nach dem Locked-Gate und vor `week_voluntary_figures`:

```rust
let year_start = ShiftyDate::first_day_in_year(iso_year);
let week_end = ShiftyDate::new(iso_year, iso_week, DayOfWeek::Sunday)
    .map_err(|_| ServiceError::InternalError)?;
let report = self.reporting_service
    .get_report_for_employee_range(&sales_person_id, year_start, week_end, true,
        Authentication::Full, Some(tx.clone())).await?;
let balance = report.balance_hours;
```

Die Position nach den beiden Gates ist bewusst: ausgeschalteter Cron und gesperrte
Woche kehren weiterhin ohne Report-Roundtrip zurück (belegt durch die unverändert
grünen `run_for_week_toggle_off_skips` / `..._locked_*`-Tests, deren
`MockReportingService` keine Expectation trägt).

**(c) `backfill_range` — derselbe Read** mit `Some(read_tx.clone())`, plus ein
Kommentar, der den Platzhalter explizit verbietet und auf den Doc-Kommentar von
`BackfillPreviewItem.excess` verweist. Die chronologische Korrektheit über mehrere
Backfill-Wochen ergibt sich ohne Zusatzaufwand aus der geteilten `read_tx`.

`suggest`, `approve`, `reject`, `rebook`, `make_batch`, `week_voluntary_figures`,
`shifty_bin/src/main.rs` und `service_impl/src/reporting.rs` blieben unangetastet.

## Verifikation

### Hard Gates

| Gate | Ergebnis |
|---|---|
| `cargo build` | ✅ Finished |
| `cargo test --workspace` | ✅ 0 failed (u. a. 829 in `service_impl`, 66 Integration) |
| `cargo clippy --workspace -- -D warnings` | ✅ exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ exit 0 (Testcode mit abgedeckt) |
| `cargo run` (Smoke, Timeout) | ✅ DI-Verdrahtung + Migrationen + Scheduler-Registrierung durchlaufen; Bind auf :3000 scheitert nur, weil bereits eine Instanz läuft (Umgebung, kein Code-Befund) |

### Akzeptanzkriterien

| Kriterium | Erwartet | Gemessen |
|---|---|---|
| `grep -c 'pub(crate) fn week_excess(ist: f32, soll: f32, committed_voluntary: f32, balance: f32) -> f32'` | 1 | 1 |
| `grep -c 'balance.is_finite()'` | 1 | 1 |
| `grep -c 'get_report_for_employee_range'` (rebooking_reconciliation.rs) | 4 | 4 |
| `grep -c 'first_day_in_year'` (rebooking_reconciliation.rs) | 2 | 2 |
| `grep -c 'week_excess(15.0, 0.0, 5.0, -2.0)'` | 1 | 1 |
| `grep -c 'week_excess(10.0, 0.0, 5.0, 0.0)'` | 1 | 1 |
| `grep -c 'fn run_for_week_balance_cap_binds'` | 1 | 1 |
| `grep -c 'fn run_for_week_non_negative_balance_skips'` | 1 | 1 |
| `grep -c 'withf'` (voluntary_rebooking.rs) | ≥ 1 | 3 |
| `grep -c 'first_day_in_year'` (voluntary_rebooking.rs) | ≥ 1 | 1 |
| `grep -c 'fn backfill_dry_run_preview_respects_balance_cap'` | 1 | 1 |
| `grep -c 'fn reporting_service(balance: f32)'` (rebooking_backfill.rs) | 1 | 1 |
| `grep -c 'reporting_service()'` (rebooking_backfill.rs) | 0 | 0 |
| `git status --porcelain shifty_bin/src/main.rs` | leer | leer |
| `git status --porcelain service_impl/src/reporting.rs` | leer | leer |

### Testläufe

| Suite | Ergebnis |
|---|---|
| `cargo test -p service_impl week_excess_matrix` | ✅ 1 passed |
| `cargo test -p service_impl run_for_week` | ✅ 11 passed (inkl. beide neuen Fälle und alle sieben Bestandsfälle) |
| `cargo test -p service_impl backfill` | ✅ 8 passed (inkl. `backfill_dry_run_writes_nothing` mit unveränderten 3.0) |

### TDD-Gate-Sequenz

| Gate | Commit | Beleg |
|---|---|---|
| RED | `90941312` | 9× E0061 „this function takes 3 arguments but 4 arguments were supplied", keine anderen Fehlerklassen |
| GREEN | `5432b9c1` | 11 run_for_week- + 8 backfill- + Matrix-Tests grün, Workspace grün, Clippy grün |

Ein REFACTOR-Gate war nicht nötig — die GREEN-Implementierung ist bereits die
Zielform aus 57-PATTERNS.md §Ziel-Signatur.

## Prohibitions — Status

| Prohibition | Status | Beleg |
|---|---|---|
| Die isolierte Wochen-Range von `suggest()`/`approve()` NICHT für den Deckel kopieren (D-57-07, Pitfall 3) | eingehalten | `run_for_week_balance_cap_binds` pinnt per `.withf` `from == first_day_in_year(2026)`, `to == Sonntag KW 12/2026`, `include_carryover == true`; der Test ist grün |
| `backfill_range` darf den vierten Parameter nicht als Platzhalter übergeben (Pitfall 2) | eingehalten | `backfill_dry_run_writes_nothing` (3.0 bei Balance −8) und `backfill_dry_run_preview_respects_balance_cap` (2.0 bei Balance −2) sind beide grün — ein 0.0-Platzhalter macht beide rot |
| `proposed_hours` im HR-Pfad bekommt KEINEN Deckel (I-5, D-57-01) | eingehalten | `service_impl/src/reporting.rs` ist unverändert (`git status --porcelain` leer); die suggest/approve-Tests laufen unangetastet grün |
| Keine DI-Änderung, kein neuer Service-Graph-Kante | eingehalten | `shifty_bin/src/main.rs` unverändert; `ReportingService` war bereits Dependency im `gen_service_impl!`-Block |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] `voluntary_rebooking_scheduler.rs` Test-Harness um den Balance-Mock ergänzt**

- **Gefunden bei:** Task 2, `cargo test --workspace`
- **Problem:** `run_once_now_books_excess_for_vorwoche` baut den ECHTEN
  `RebookingReconciliationServiceImpl` über `build_recon`, das den
  `ReportingService` hartkodiert als `MockReportingService::new()` (ohne jede
  Expectation) verdrahtete — bis Phase 57 korrekt, weil `run_for_week` den Report
  nie las. Mit dem neuen Balance-Read paniked der Mock:
  `MockReportingService::get_report_for_employee_range(…, ShiftyDate { 2026, week 1,
  Thursday }, ShiftyDate { 2026, week 11, Sunday }, true, Full, …): No matching
  expectation found`. Das ist unmittelbar durch diesen Task verursacht, also
  in-scope.
- **Fix:** `make_report(balance)`- und `reporting_service(balance: f32)`-Helper
  ergänzt (spiegelbildlich zu den beiden anderen Test-Dateien), `build_recon` nimmt
  den `ReportingService` jetzt als Parameter statt ihn hartzukodieren, Case 1
  übergibt `-8.0` (Headroom 8 > Überschuss 3 → Deckel bindet nicht, erwartete 3.0
  unverändert). Die beiden anderen Tests der Datei konstruieren den Impl direkt und
  erreichen den Read nie (Toggle-off bzw. uncapped-Filter) — sie blieben unberührt.
- **Datei:** `service_impl/src/test/voluntary_rebooking_scheduler.rs` (nicht in
  `files_modified` des Plans — der Plan hat diese dritte Call-Site des Motors
  übersehen)
- **Commit:** `5432b9c1`
- **Nebenbefund:** Die Panik-Meldung ist zugleich ein unabhängiger Beleg für die
  korrekte Range — `first_day_in_year(2026)` fällt auf einen Donnerstag in ISO-Woche 1,
  Ende ist Sonntag KW 11, `include_carryover = true`.

Keine weiteren Abweichungen — Formel und beide Call-Sites folgen exakt dem
Plan-Sketch aus 57-PATTERNS.md.

## Auth-Gates

Keine.

## Known Stubs

Keine.

## Requirements-Status

`requirements mark-complete` wurde für **REB-AUTO-06 bewusst NICHT** ausgeführt.
Dieser Plan liefert die Formel, beide Call-Sites und die Unit-Ebene der
Akzeptanzbedingung (Deckel bindet, Balance ≥ 0 skippt, Preview == Apply). Die im
Requirement wörtlich geforderte **Gegenprobe** („Woche 2 mit 20 h statt 17 h ergibt
einen Wochen-Überschuss von 10; der Übertrag bleibt trotzdem 2 und die Balance landet
auf 0") ist ein Ende-zu-Ende-Szenario über den Shiftplan-Cap-Überlauf — das schreibt
**Plan 04**, dessen Frontmatter REB-AUTO-06 ebenfalls führt. Das Abhaken gehört
dorthin; eine frühere Markierung würde Coverage behaupten, die noch nicht gemessen
ist. Dasselbe Muster hat Plan 02 für VOL-ACCT-04 angewandt.

## Docs-Freshness-Gate

Dieser Plan berührt keine Trigger-Datei aus der CLAUDE.md-Tabelle:
`shifty_bin/src/main.rs`, `service/**`, `dao/**`, `migrations/`,
`service_impl/src/permission.rs`, `service_impl/src/billing_period_report.rs` und
`service_impl/src/reporting.rs` sind alle unverändert. Geändert wurde ausschließlich
`service_impl/src/rebooking_reconciliation.rs` (keine Trait-Signatur, keine
DI-Kante) plus Testdateien. Die inhaltliche Beschreibung der Deckel-Formel in
`docs/features/F14-rebooking(_de).md` ist trotzdem angebracht und **Plan 06 (Wave 5)**
zugewiesen — sie steht dort bereits in `files_modified`.

## Threat Flags

Keine neue Angriffsfläche. Der Register-Status:

- **T-57-07** (nicht-endliche Balance) — mitigiert: der Guard deckt alle vier
  Parameter ab, CI-Gate ist die Matrix-Zeile `week_excess(10.0, 0.0, 5.0, f32::NAN)`.
- **T-57-08** (Platzhalter-Balance im Backfill) — mitigiert: zwei sich
  widersprechende Preview-Tests.
- **T-57-09** (zusätzlicher Report-Read im Backfill) — akzeptiert wie geplant: der
  Read sitzt nach dem HR-Privilege-Gate, mit `Authentication::Full` und der bereits
  offenen Transaktion.
- **T-57-10** (ein Report-Read pro (SalesPerson, Woche) im Backfill-Preview) —
  akzeptiert wie geplant; die Obergrenze auf die letzte abgeschlossene Vorwoche
  (T-56-05) bleibt in Kraft.
- **T-57-SC** — `Cargo.toml` unverändert, keine Paket-Installation.

## Anschluss

- **Plan 04** kann das Referenz-Szenario D-57-02 Ende-zu-Ende schreiben; Leseseite
  (Plan 02) und Deckel (dieser Plan) stehen beide.
- **Plan 06** dokumentiert die Deckel-Formel und die Abgrenzung zum HR-Pfad in
  F14 §3.

## Commits

| Commit | Inhalt |
|---|---|
| `90941312` | `test(57-03)`: RED — Balance-Deckel-Matrix und drei Verhaltenstests |
| `5432b9c1` | `feat(57-03)`: GREEN — Balance-Deckel fuer den Auto-Cron (REB-AUTO-06) |

## Self-Check: PASSED

- `service_impl/src/rebooking_reconciliation.rs` — FOUND
- `service_impl/src/test/voluntary_rebooking.rs` — FOUND
- `service_impl/src/test/rebooking_backfill.rs` — FOUND
- `service_impl/src/test/voluntary_rebooking_scheduler.rs` — FOUND
- `.planning/phases/57-.../57-03-SUMMARY.md` — FOUND
- Commit `90941312` — FOUND
- Commit `5432b9c1` — FOUND
- `cargo build` / `cargo test --workspace` / `cargo clippy --workspace -- -D warnings` — alle grün
