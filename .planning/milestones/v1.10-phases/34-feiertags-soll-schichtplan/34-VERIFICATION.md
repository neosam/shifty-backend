---
phase: 34-feiertags-soll-schichtplan
verified: 2026-06-30T14:00:00Z
status: passed
score: 6/6 must-haves verified
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: passed
  previous_score: 6/6
  gaps_closed:
    - "CR-01: holiday_derived_gated floss in apply_weekly_cap-Baseline ein → holiday leckte in volunteer_hours / swallowed balance credit (cap-active + overbooked Woche)"
  gaps_remaining: []
  regressions: []
---

# Phase 34: Feiertags-Soll im Schichtplan — Verifikationsbericht

**Phase-Ziel:** Ein automatisch angerechneter Feiertag reduziert das angezeigte Soll in der Wochentabelle unter dem Schichtplan — konsistent zum Stundenkonto —, während die Kapazitätsbänder (`paid_hours`/`dynamic_hours`/`committed_voluntary`/`volunteer`) unverändert bleiben (D-25-08-Grenze).
**Verifiziert:** 2026-06-30 (initiale Verifikation) + 2026-06-30 (Re-Verifikation nach CR-01-Fix)
**Status:** passed
**Re-Verifikation:** Ja — nach Post-Verifikations-Code-Review BLOCKER CR-01

---

## Ziel-Erreichung

### Beobachtbare Wahrheiten (Success Criteria aus ROADMAP.md)

| # | Wahrheit | Status | Nachweis |
|---|----------|--------|---------|
| 1 | In der Wochentabelle unter dem Schichtplan reduziert ein automatisch angerechneter Feiertag das angezeigte Soll (`available_hours`/`expected_hours`) eines Mitarbeiters; der Wert stimmt mit dem Stundenkonto überein. (HSP-01) | VERIFIED | `reporting.rs:1122`: `let expected_hours = expected_hours_for_cap - holiday_derived_gated;` — Test `test_holiday_auto_credit_get_week_reduces_soll_bands_unchanged` assertiert `expected_hours == 32.0` (40 − 8). Test `test_hsp03_cap_active_holiday_no_band_leak` assertiert zusätzlich `balance_hours == 8.0` (Stundenkonto-Konsistenz im cap-aktiven Fall). GRUEN. |
| 2 | Die abgeleiteten Feiertags-Stunden (`holiday_hours`) erscheinen pro Mitarbeiter in der Schichtplan-Tabelle. (HSP-02) | VERIFIED | `reporting.rs:1101`: `let holiday_hours = holiday_hours + holiday_derived_gated;` — Tests assertieren `holiday_hours == 8.0`. GRUEN. |
| 3 | Die Kapazitätsbänder (`paid_hours`/`dynamic_hours`/`committed_voluntary`/`volunteer`) sind in derselben Woche vor und nach der Änderung identisch (Regressions-Guard). (HSP-03) | VERIFIED | `reporting.rs:1129`: `let dynamic_hours = dynamic_hours - abense_hours_for_balance - absence_derived_balance_total;` — `holiday_derived_gated` fehlt bewusst (Band-Guard). `test_holiday_auto_credit_get_week_reduces_soll_bands_unchanged` assertiert `dynamic_hours == 40.0`. `test_hsp03_cap_active_holiday_no_band_leak` assertiert zusätzlich `volunteer_hours == 0.0` und `dynamic_hours == 40.0` im cap-aktiven Fall. GRUEN. |
| 4 | Ein Feiertag vor dem konfigurierten Stichtag bleibt wirkungslos, und ein manueller `ExtraHours(Holiday)` wird nicht doppelt gezählt — identisch zum Stundenkonto (Wiederverwendung von `build_derived_holiday_map`). (HSP-04) | VERIFIED | `reporting.rs:1084–1092`: `self.build_derived_holiday_map(...)` aufgerufen — derselbe Pfad wie Injektionspunkte 1a/1b. `test_hsp04_before_cutoff`: expected_hours==40, holiday_hours==0. `test_hsp04_manual_wins`: holiday_hours==8.0 (nicht 16.0). Beide GRUEN. |
| 5 | D-34-01: `holiday_derived_gated` nur in `expected_hours` subtrahiert; `booking_information.rs:517`-Block (per-Tag Mo–So) nicht angefasst. | VERIFIED | `reporting.rs:1129` zeigt `dynamic_hours`-Formel ohne `holiday_derived_gated`. `booking_information.rs` nicht in `key_files.modified` der SUMMARY; per Grep keine Änderung an Zeile 517. |
| 6 | D-34-04: `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12 — `billing_period_report.rs` ruft `get_report_for_employee_range`, nicht `get_week`. | VERIFIED | `billing_period_report.rs:117`: `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;`. Grep über `billing_period_report.rs`: kein Aufruf von `get_week` oder `build_derived_holiday_map` — nur `get_report_for_employee_range` (4 Aufrufstellen, Zeilen 146/157/168/179). |

**Score:** 6/6 Wahrheiten verifiziert (0 behavior_unverified)

---

### Erforderliche Artefakte

| Artefakt | Erwartet | Status | Details |
|----------|----------|--------|---------|
| `service_impl/src/reporting.rs` | 4. Injektionspunkt: `holiday_derived_gated`-Term in `get_week`; CR-01-Fix: separater `expected_hours_for_cap` für `apply_weekly_cap` | VERIFIED | Zeilen 1072–1129: Injektionspunkt + CR-01-Fix dokumentiert und implementiert. `expected_hours_for_cap` (Zeile 1113) exkludiert `holiday_derived_gated`; `apply_weekly_cap` verwendet `expected_hours_for_cap` (Zeile 1115-1116); `expected_hours` = `expected_hours_for_cap − holiday_derived_gated` (Zeile 1122); `dynamic_hours` unberührt (Zeile 1129). |
| `service_impl/src/test/reporting_holiday_auto_credit.rs` | HOL-03 umgebaut; `test_hsp04_before_cutoff`; `test_hsp04_manual_wins`; neu: `test_hsp03_cap_active_holiday_no_band_leak`; Umbenennung WR-03 | VERIFIED | 929 Zeilen. Alle 8 Tests GRUEN (Zeile 546: `test_holiday_auto_credit_get_week_reduces_soll_bands_unchanged`, Zeile 641: `test_hsp04_before_cutoff`, Zeile 726: `test_hsp04_manual_wins`, Zeile 821: `test_hsp03_cap_active_holiday_no_band_leak`). |

---

### Key-Link-Verifikation

| Von | Zu | Via | Status | Details |
|-----|----|-----|--------|---------|
| `get_week` | `build_derived_holiday_map` | `self.build_derived_holiday_map(ShiftyWeek::new(year, week).as_date(Monday), ..., &employee_extra_hours_owned, context.clone())` | WIRED | `reporting.rs:1084–1092`. |
| `report.holiday_hours` | `WorkingHoursPerSalesPerson.holiday_hours` | Propagation via `booking_information.rs:331` (automatisch, kein Eingriff) | WIRED | `booking_information.rs` nicht angefasst; Propagationspfad unverändert. |
| `report.expected_hours` | `WorkingHoursPerSalesPerson.available_hours` | `available_hours = report.expected_hours` in `booking_information.rs:322` | WIRED | Automatische Propagation; `booking_information.rs` nicht angefasst. |
| `report.dynamic_hours` | `paid_hours`-Band (HSP-03-Grenze) | `paid_hours = Σ report.dynamic_hours` in `booking_information.rs` | WIRED, INVARIANT | `holiday_derived_gated` wird NICHT von `dynamic_hours` subtrahiert — Band bleibt konstant. |

---

### Behavioral Spot-Checks

| Verhalten | Befehl | Ergebnis | Status |
|-----------|--------|----------|--------|
| 8 Holiday-Tests grün, inkl. HSP-01/02/03/04 + CR-01-Regression | `SQLX_OFFLINE=true cargo test -p service_impl reporting_holiday` | 8 passed, 0 failed | PASS |

Ausgabe des Test-Runs (Re-Verifikation):
```
running 8 tests
test test::reporting_holiday_auto_credit::test_hsp04_manual_wins ... ok
test test::reporting_holiday_auto_credit::test_hsp04_before_cutoff ... ok
test test::reporting_holiday_auto_credit::test_hsp03_cap_active_holiday_no_band_leak ... ok
test test::reporting_holiday_auto_credit::test_holiday_auto_credit_get_week_reduces_soll_bands_unchanged ... ok
test test::reporting_holiday_auto_credit::test_holiday_auto_credit_basic ... ok
test test::reporting_holiday_auto_credit::test_holiday_before_cutoff_skipped ... ok
test test::reporting_holiday_auto_credit::test_holiday_manual_wins ... ok
test test::reporting_holiday_auto_credit::test_holiday_auto_credit_equivalence ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 510 filtered out; finished in 0.00s
```

---

### Anforderungs-Abdeckung

| Anforderung | Plan | Beschreibung | Status | Nachweis |
|-------------|------|--------------|--------|---------|
| HSP-01 | 34-01 | Feiertag reduziert `expected_hours` in `get_week` | SATISFIED | `reporting.rs:1122`; Tests assertieren `expected_hours==32.0` + `balance_hours==8.0`. |
| HSP-02 | 34-01 | `holiday_hours` per Mitarbeiter befüllt | SATISFIED | `reporting.rs:1101`; Tests assertieren `holiday_hours==8.0`. |
| HSP-03 | 34-01 | Kapazitätsbänder unverändert (Regressions-Guard) | SATISFIED | `reporting.rs:1129` ohne `holiday_derived_gated`; Tests assertieren `dynamic_hours==40.0`, `volunteer_hours==0.0` auch im cap-aktiven Fall. |
| HSP-04 | 34-01 | Stichtag-Gate + manual-wins via `build_derived_holiday_map` | SATISFIED | `reporting.rs:1084–1092`; `test_hsp04_before_cutoff` + `test_hsp04_manual_wins` GRUEN. |

Alle 4 HSP-Anforderungen in `REQUIREMENTS.md` als `✅ complete` markiert.

---

### Anti-Pattern-Scan

| Datei | Befund | Bewertung |
|-------|--------|-----------|
| `service_impl/src/reporting.rs` | Keine TBD/FIXME/XXX ohne Issue-Referenz; CR-01-Fix klar mit Rationale-Kommentar versehen (Zeilen 1103–1112). | Sauber |
| `service_impl/src/test/reporting_holiday_auto_credit.rs` | Keine Stubs; alle Tests mit konkreten Asserts und Fehlermeldungen. Testdoc auf CR-01 und D-25-08-Grenze referenziert. | Sauber |

Keine Blocker. Keine Warnungen.

---

### Snapshot-Schema-Verifikation (D-34-04)

`billing_period_report.rs:117`:
```rust
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;
```

Grep-Ergebnis: `billing_period_report.rs` enthält keinen Aufruf von `get_week` oder `build_derived_holiday_map`. Alle 4 Aufrufstellen des Reporting-Service in `billing_period_report.rs` nutzen `get_report_for_employee_range` (Zeilen 146, 157, 168, 179). Ein Bump auf 13 war nicht erforderlich.

---

### Bekannte Pre-Existing-Issues (nicht durch Phase 34 verursacht)

| Issue | Datei | Einschätzung |
|-------|-------|--------------|
| `test_seed_twice_is_additive` schlägt mit `ValidationError([Duplicate])` fehl | `shifty_bin/src/integration_test/dev_seed.rs:94` | Pre-existing seit commit `8ead369` (März 2026); Phase 34 nicht kausal. Commit `8379eea` adressiert diesen Regression-Fix (liegt in der Git-History des aktuellen Stands). |

---

## Gap-Closure (CR-01)

### Hintergrund

Die initiale Verifikation (7/7 Tests, status: passed) wurde durch einen Post-Verifikations-Code-Review mit BLOCKER CR-01 nachbearbeitet. Der Befund: Im ursprünglichen Code wurde `holiday_derived_gated` in `abense_hours_for_balance` eingerechnet, was bedeutete, dass der `apply_weekly_cap`-Aufruf gegen eine bereits um den Feiertag reduzierte Baseline cappte. Im cap-aktiven + überbuchten Fall (Schichtplan > raw-Soll) konvertierte die Differenz fälschlicherweise in `auto_volunteer_hours` — der Feiertag leckte in das `volunteer`-Band und der Balance-Kredit wurde verschluckt.

### Geänderter Code

**Commits: e8da4d8 (RED) + 67f774a (GREEN)**

`service_impl/src/reporting.rs`, Zeilen 1113–1122 (nach Fix):

```rust
// HSP-03 band guard / CR-01: the derived holiday must NOT enter the
// apply_weekly_cap baseline. ...
let expected_hours_for_cap =
    planned_hours - abense_hours_for_balance - absence_derived_balance_total;
let (shiftplan_hours, auto_volunteer_hours) =
    apply_weekly_cap(cap_active, raw_shiftplan_hours, expected_hours_for_cap);
// HSP-01: the displayed/balance expected is the cap baseline minus the derived
// holiday credit.
let expected_hours = expected_hours_for_cap - holiday_derived_gated;
```

`holiday_derived_gated` wird ausschliesslich auf `expected_hours` (Zeile 1122) und `holiday_hours` (Zeile 1101) angewendet. Es fliesst NICHT in `expected_hours_for_cap` ein und NICHT in `dynamic_hours` (Zeile 1129).

### Konsistenz mit dem Jahr-View

Der autorisierte Jahr-View (`get_reports_for_all_employees`) ruft `apply_weekly_cap` mit dem rohen `expected_hours` auf (Zeile 428) und wendet den Feiertag separat auf die Balance an. `get_week` verhält sich nach dem Fix identisch: cap gegen raw-expected, Feiertag danach als reiner Balance-/Display-Abzug. Die Inkonsistenz zwischen beiden Pfaden ist damit geschlossen.

### Verifikation des Fixes

**Test `test_hsp03_cap_active_holiday_no_band_leak` (`reporting_holiday_auto_credit.rs:821`):**

Setup: `cap_planned_hours_to_expected=true`, 40h Schichtplan in KW23/2024, 8h Feiertag (Montag), Toggle aktiv.

| Assertion | Erwarteter Wert | Bedeutung |
|-----------|----------------|-----------|
| `volunteer_hours` | 0.0 | Band-Guard: Feiertag erzeugt kein Auto-Ehrenamt |
| `dynamic_hours` | 40.0 | Band-Guard: paid_hours-Band unveraendert |
| `expected_hours` | 32.0 | HSP-01: Soll um 8h Feiertag reduziert |
| `holiday_hours` | 8.0 | HSP-02: derived-Holiday-Beitrag sichtbar |
| `balance_hours` | 8.0 | HSP-01 Stundenkonto-Konsistenz: 40 gearbeitet − 32 Soll |

Alle 5 Assertions GRUEN. Ohne den Fix waere `volunteer_hours == 8.0` und `balance_hours == 0.0` (der Feiertag-Kredit wuerde durch die fehlgeleitete Cap-Berechnung verschluckt).

**WR-03 Umbenennung:** `test_holiday_auto_credit_no_year_view_impact` → `test_holiday_auto_credit_get_week_reduces_soll_bands_unchanged`. Name beschreibt jetzt das direkt getestete Verhalten statt einer Referenz auf einen anderen View. Kein Logik-Change.

**Gesamtergebnis Test-Run:** 8/8 GRUEN (war 7/7 vor CR-01-Fix-Zyklus):
- `test_hsp03_cap_active_holiday_no_band_leak` — NEU, gezielt fuer CR-01
- `test_holiday_auto_credit_get_week_reduces_soll_bands_unchanged` — umbenannt (WR-03)
- 6 bestehende Tests unveraendert GRUEN (kein Regress)

**Snapshot-Version:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12. `get_week` hat keine Billing-Period-Snapshot-Abhaengigkeit; der Fix aendert keine value_type-Semantik in `billing_period_report.rs`.

### Ergebnis

CR-01 ist geschlossen. Alle 4 HSP-Anforderungen erfullt. Phase-34-Ziel bestaetigt.

---

## Zusammenfassung

Phase 34 hat ihr Ziel vollstaendig erreicht. Der 4. Injektionspunkt in `get_week` (`reporting.rs:1072–1129`) verdrahtet `build_derived_holiday_map` korrekt mit CR-01-konformem Cap-Handling:

- `expected_hours` wird um `holiday_derived_gated` reduziert — NACH dem Cap-Schritt (HSP-01).
- `holiday_hours` enthalt den derived-Beitrag (HSP-02).
- `apply_weekly_cap` cappt gegen `expected_hours_for_cap` (ohne Holiday) — konsistent mit dem Jahr-View (HSP-03, CR-01).
- `dynamic_hours` (und damit `paid_hours`-Band) bleibt strikt unangetastet (HSP-03).
- Stichtag-Gate und manual-wins werden aus dem bestehenden Phase-25-Helfer ubernommen — keine Neu-Implementierung, keine Doppelzahlung (HSP-04).
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12 (D-34-04).
- Clippy sauber (vom Orchestrator bestatigt: `cargo clippy --workspace -- -D warnings` clean).
- 8/8 Holiday-Tests GRUEN.

---

_Initiale Verifikation: 2026-06-30_
_Re-Verifikation (CR-01 Gap-Closure): 2026-06-30_
_Verifier: Claude (gsd-verifier)_
