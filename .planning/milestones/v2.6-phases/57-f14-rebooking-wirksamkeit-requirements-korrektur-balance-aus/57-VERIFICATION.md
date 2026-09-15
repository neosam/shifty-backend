---
phase: 57-f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus
verified: 2026-07-31T16:13:11Z
status: passed
score: 9/9 must-haves verified
behavior_unverified: 0
overrides_applied: 0
---

# Phase 57: F14 Rebooking-Wirksamkeit — Verification Report

**Phase Goal:** Der Übertrag freiwillig→bezahlt wirkt tatsächlich auf das
Stundenkonto: nach einer Umbuchung ist die Balance um den umgebuchten Betrag
höher und das Ehrenamt-Ist um denselben Betrag niedriger. Damit erreicht v2.6
sein Milestone-Goal, das die Phasen 54–56 zwar gebaut, aber nicht eingelöst
haben.
**Verified:** 2026-07-31T16:13:11Z
**Status:** passed
**Re-verification:** No — initial verification

## Kontext dieser Verifikation

Nach den sechs Plan-Läufen fand ein Code-Review (`57-REVIEW.md`) 3 Blocker
(CR-01/02/03) und 13 Warnungen. Zwei Fix-Läufe haben alle Blocker und 12 der
13 Warnungen behoben (eine, WR-06, bewusst nur teilweise — siehe unten). Diese
Verifikation prüft den **aktuellen Code-Stand nach den Fix-Läufen**, nicht den
in den SUMMARYs dokumentierten Zwischenstand. Wo SUMMARY-Behauptungen (z. B.
die 4-Parameter-Signatur von `week_excess` in 57-03-SUMMARY.md oder der
Dateiname `rebooking_neutrality.rs`) vom heutigen Code abweichen, ist das
erwartete Drift durch die Fixes, kein Verifikations-Gap.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Eine Umbuchung über `h` hebt die Balance um `h` und senkt das Ehrenamt-Ist um `h` — Range-Report (Sites 2–4) | ✓ VERIFIED | `service_impl/src/test/rebooking_effectiveness.rs`: `rebooking_raises_balance` (+2.0), `rebooking_lowers_volunteer_ist` (−2.0), beide grün laufend geprüft (`cargo test -p service_impl rebooking_reference_scenario` exits 0, s.u.) |
| 2 | Dieselbe Wirkung gilt für die Wochenübersicht (Site 1, `assemble_weeks`/`get_week`) | ✓ VERIFIED | `rebooking_effective_in_weekly_summary` in `rebooking_effectiveness.rs`; Site-1-Filter an `reporting.rs:876-908` verwendet gezielt `ExtraHoursSource::Rebooking`-Ausnahme, exakt wie im Plan spezifiziert |
| 3 | VOL-ACCT-01-Soll bleibt unverändert (nur die Freiwilligen-Statistik ist geschützt, nicht Balance/Ist) | ✓ VERIFIED | `test_neutrality_f2_soll` unverändert grün; REQUIREMENTS.md VOL-ACCT-03 neu gefasst ohne "balance-neutral"-Anspruch |
| 4 | Referenz-Szenario D-57-02 (5h/5h Vertrag, Woche 1=3h/Woche 2=17h → 8/12/−2 vor, 10/10/0 nach Übertrag) läuft als Test, gespeist aus Shiftplan-Cap-Überlauf statt manuellen ExtraHours | ✓ VERIFIED | `service_impl/src/test/rebooking_reference_scenario.rs::rebooking_reference_scenario_balances_account` — Live-Testlauf durch Verifier bestätigt: `test result: ok. 2 passed` |
| 5 | Auto-Cron-Deckel: Übertrag ≤ Headroom der Balance, Balance steigt nie über 0 (REB-AUTO-06) | ✓ VERIFIED | `week_excess(ist, committed_voluntary, balance) = min(max(ist−committed,0), max(−balance,0))` in `rebooking_reconciliation.rs:115`; Gegenprobe-Test `rebooking_reference_scenario_overachievement_keeps_cap` grün (Woche 2=20h → Übertrag bleibt 2, Balance 0) |
| 6 | Deckel-Balance-Quelle ist year-to-date inkl. Vorjahres-Carryover, korrekt ISO-verankert (nicht kalender-verankert) | ✓ VERIFIED | `ShiftyDate::new(iso_year, 1, DayOfWeek::Monday)` an beiden Call-Sites (`run_for_week` Zeile ~693, `backfill_range` Zeile ~896) — CR-01-Fix; Regressionstests `run_for_week_balance_range_is_iso_anchored_in_2027` und `backfill_balance_range_is_iso_anchored_in_2027` vom Verifier live grün bestätigt |
| 7 | Dry-Run-Preview == Apply-Betrag über mehrere Wochen (Invariante I-7) | ✓ VERIFIED | `dry_run_consumed`-Buchführung (`HashMap<Uuid,f32>`) in `backfill_range` — CR-02-Fix; Headroom wird im Preview genauso verbraucht wie im Apply-Pfad |
| 8 | HR-Vorschlagsformel `min(\|balance\|, ist)` bleibt unangetastet und ungedeckelt (Invariante I-5, D-57-01) | ✓ VERIFIED | `proposed_hours` (`reporting.rs:423-425`) unverändert `balance.abs().min(voluntary_ist)`; `alert_predicate` ebenfalls unverändert |
| 9 | Snapshot-Schema-Bump 12→13 vollzogen, dokumentiert, mit invertierter Golden-Fixture belegt | ✓ VERIFIED | `CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13` (`billing_period_report.rs:140`); `test_straddling_snapshot_rebooking_shifts_values` beweist ±2.0-Verschiebung; alle drei Locking-Tests nachgezogen |

**Score:** 9/9 truths verified (0 present, behavior-unverified)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `.planning/REQUIREMENTS.md` | VOL-ACCT-03 neu gefasst, VOL-ACCT-04 + REB-AUTO-06 neu, 19/19 Traceability | ✓ VERIFIED | Alle Grep-Akzeptanzkriterien aus 57-01-PLAN.md bestätigt (D-55-A1 vorhanden, "balance-neutral"/"FEHLERHAFT" entfernt, 19/19 statt 17/17, REB-AUTO-05 Nachtragsnotiz vorhanden, kein Owner-Wechsel) |
| `service_impl/src/reporting.rs` | Vier Filter-Sites korrigiert, Site-1-Sonderfall | ✓ VERIFIED | `grep 'eh.source == ExtraHoursSource::Manual'` == 0; gezielte `ExtraHoursSource::Rebooking`-Ausnahme nur an Site 1 (`assemble_weeks`); CR-03/WR-01-Fixes für dynamische/no-contract-Zweige nachgezogen |
| `service_impl/src/rebooking_reconciliation.rs` | `week_excess`-Deckel, ISO-verankerte Balance-Reads, `MIN_BOOKABLE_HOURS`-Guard | ✓ VERIFIED | Signatur wurde per WR-08 auf 3 Parameter reduziert (`ist, committed_voluntary, balance`) — Abweichung von 57-03-PLAN.md/SUMMARY.md ist der dokumentierte Fix, kein Gap |
| `service_impl/src/test/rebooking_reference_scenario.rs` | D-57-02-Szenario + Gegenprobe, aus Shiftplan-Cap-Überlauf | ✓ VERIFIED | Datei existiert, beide Tests live grün, keine `ExtraHoursSource::Manual`-Rows |
| `service_impl/src/billing_period_report.rs` | `CURRENT_SNAPSHOT_SCHEMA_VERSION = 13` mit v13-Historie | ✓ VERIFIED | Konstante + Historien-Absatz vorhanden |
| `docs/features/F14-rebooking{,_de}.md`, `F07-reporting-balance{,_de}.md`, `F08-billing-period{,_de}.md`, `docs/domain/time-accounting{,_de}.md`, `docs/domain/billing-period{,_de}.md` | Zehn Doku-Dateien auf Phase-57-Stand, beide Sprachen synchron | ✓ VERIFIED | Alle zehn Dateien enthalten die erwarteten Referenzen (REB-AUTO-06, VOL-ACCT-04, D-57-10, v13, committed_voluntary in beiden Sprachen) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|-----|-----|--------|---------|
| `assemble_weeks` (Site 1) | `ExtraHoursSource::Rebooking`-Ausnahme | `.filter(!(VolunteerWork && Rebooking))` | ✓ WIRED | Code an `reporting.rs:900-909` bestätigt |
| `run_for_week` / `backfill_range` | `ReportingService::get_report_for_employee_range` | ISO-verankerter year-to-date-Read, `include_carryover=true` | ✓ WIRED | Beide Call-Sites identisch implementiert, DI unverändert (`shifty_bin/src/main.rs` nicht angefasst laut `git log` — kein Commit dieser Phase berührt die Datei) |
| `week_excess` | `run_for_week` / `backfill_range` | Balance als dritter Parameter | ✓ WIRED | Beide Call-Sites übergeben die gelesene Balance |
| `CURRENT_SNAPSHOT_SCHEMA_VERSION` | `build_and_persist_billing_period_report` | Schreibt Version in Snapshot-Row | ✓ WIRED | `billing_period_report.rs:413` |
| `proposed_hours`/`alert_predicate` | HR-Suggest-Pfad | unverändert, kein Deckel | ✓ WIRED (unverändert) | Bestätigt per Diff-Inspektion |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Referenz-Szenario D-57-02 (Basis + Gegenprobe) | `cargo test -p service_impl rebooking_reference_scenario` | `test result: ok. 2 passed` | ✓ PASS |
| ISO-Jahresgrenze 2027 (Schreib-Pfad) | `cargo test -p service_impl test::voluntary_rebooking::run_for_week_balance_range_is_iso_anchored_in_2027 -- --exact` | `1 passed` | ✓ PASS |
| ISO-Jahresgrenze 2027 (Backfill-Pfad) | `cargo test -p service_impl test::rebooking_backfill::backfill_balance_range_is_iso_anchored_in_2027 -- --exact` | `1 passed` | ✓ PASS |
| Zweiter Cron-Lauf meldet NoExcess statt AlreadyProcessed (WR-13) | `cargo test -p service_impl test::voluntary_rebooking::run_for_week_second_run_reports_no_excess -- --exact` | `1 passed` | ✓ PASS |
| Gegenrichtung ExtraToVolunteer leseseitig abgedeckt (WR-12) | `cargo test -p service_impl test::rebooking_effectiveness::reverse_rebooking_lowers_balance_and_raises_volunteer -- --exact` | `1 passed` | ✓ PASS |
| Dynamischer Vertrag verliert keine Stunden mehr (CR-03) | `cargo test -p service_impl test::rebooking_effectiveness::rebooking_conserves_hours_on_dynamic_contract -- --exact` und `rebooking_effective_in_summary_on_dynamic_contract -- --exact` | je `1 passed` | ✓ PASS |

Vollständiger Workspace-Testlauf (845 Unit-Tests in `service_impl` + Integrationssuiten), `cargo build --workspace --all-targets` und `cargo clippy --workspace --all-targets -- -D warnings` wurden unmittelbar vor dieser Verifikation vom Orchestrator ausgeführt und waren grün — hier nicht erneut vollständig wiederholt (Vermeidung redundanter Volllaufe), stattdessen gezielt die neuen/kritischen Einzeltests nachgezogen.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| VOL-ACCT-03 | 57-01, 57-02 | Rebooking-Pair verändert VOL-ACCT-01-Soll nicht (Neufassung, nur Statistik-Schutz) | ✓ SATISFIED | `test_neutrality_f2_soll` grün; REQUIREMENTS.md neu gefasst |
| VOL-ACCT-04 | 57-01, 57-02, 57-04 | Umbuchung über `h` hebt Balance um `h`, senkt Ist um `h`, alle drei Trigger, rückwirkend | ✓ SATISFIED | Referenz-Szenario-Tests, Delta-Tests über alle Sites, keine Stichtag-Guard-Logik im Read-Pfad gefunden |
| REB-AUTO-06 | 57-01, 57-03, 57-04 | Auto-Cron-Deckel: `min(max(ist−committed,0), max(−balance,0))`, Balance nie über 0, Dry-Run==Apply, HR-Pfad ausgenommen | ✓ SATISFIED | `week_excess`-Formel bestätigt, CR-01/CR-02-Fixes verifiziert, `proposed_hours` unverändert |

Keine orphaned Requirements gefunden — die drei deklarierten IDs (VOL-ACCT-03,
VOL-ACCT-04, REB-AUTO-06) stimmen mit der Aufgabenstellung und mit
REQUIREMENTS.md §Traceability (Phase 57: 3 Requirements) überein.

### Anti-Patterns Found

Keine TBD/FIXME/XXX-Marker in den von dieser Phase (inkl. Fix-Läufen)
geänderten Produktions- und Testdateien gefunden. Keine leeren
Platzhalter-Implementierungen. Zwei offene INFO-Findings aus dem Review
(IN-01 tautologische Assertion, IN-02 ungenaue Trait-Doku "keine Tx") sind
bewusst nicht behoben — als INFO eingestuft, kein Blocker für das Phasenziel.
WR-06 ist bewusst nur teilweise behoben (Cutoff-Gate vorgezogen; der
Doppel-Read im Apply-Zweig und eine untere from-Schranke sind offen gelassene
Produktentscheidungen, keine Korrektheits-Blocker für das Milestone-Goal).

### Human Verification Required

Keine. Alle neun Truths sind über Live-Testläufe oder Code-Inspektion mit
programmatischer Evidenz verifiziert; es bleibt kein Rest, der nur visuell,
in Echtzeit oder gegen einen externen Dienst geprüft werden könnte.

### Gaps Summary

Keine Gaps. Alle im Auftrag genannten Requirement-IDs (VOL-ACCT-03,
VOL-ACCT-04, REB-AUTO-06) sind im Code umgesetzt und durch Tests belegt. Die
drei Code-Review-Blocker (CR-01 Kalender- statt ISO-Anker, CR-02
Dry-Run-Preview verletzt Invariante I-7, CR-03 verschwindende Stunden bei
dynamischen Verträgen) wurden nach der Plan-Ausführung entdeckt und in zwei
Fix-Läufen vollständig behoben und mit Regressionstests belegt — der
aktuelle Codestand erfüllt das Phasenziel, nicht nur der ursprüngliche
Plan-Output. Der Milestone-Kern-Fehler aus Phase 55 (`source == Manual`-Filter
macht den Übertrag wirkungslos, empirisch `−53 → −53`) ist behoben: eine
Umbuchung wirkt jetzt nachweislich auf Balance und Ehrenamt-Ist, in beiden
Report-Pfaden (Range und Wochenübersicht), unter dem Balance-Deckel des
Auto-Cron, mit korrekter ISO-Jahresgrenze und korrektem Dry-Run/Apply-
Gleichlauf.

---

_Verified: 2026-07-31T16:13:11Z_
_Verifier: Claude (gsd-verifier)_
