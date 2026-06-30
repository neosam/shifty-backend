---
phase: 34-feiertags-soll-schichtplan
verified: 2026-06-30T12:00:00Z
status: passed
score: 6/6 must-haves verified
behavior_unverified: 0
overrides_applied: 0
---

# Phase 34: Feiertags-Soll im Schichtplan — Verifikationsbericht

**Phase-Ziel:** Ein automatisch angerechneter Feiertag reduziert das angezeigte Soll in der Wochentabelle unter dem Schichtplan — konsistent zum Stundenkonto —, während die Kapazitätsbänder (`paid_hours`/`dynamic_hours`/`committed_voluntary`/`volunteer`) unverändert bleiben (D-25-08-Grenze).
**Verifiziert:** 2026-06-30
**Status:** passed
**Re-Verifikation:** Nein — initiale Verifikation

---

## Ziel-Erreichung

### Beobachtbare Wahrheiten (Success Criteria aus ROADMAP.md)

| # | Wahrheit | Status | Nachweis |
|---|----------|--------|---------|
| 1 | In der Wochentabelle unter dem Schichtplan reduziert ein automatisch angerechneter Feiertag das angezeigte Soll (`available_hours`/`expected_hours`) eines Mitarbeiters; der Wert stimmt mit dem Stundenkonto überein. (HSP-01) | VERIFIED | `reporting.rs:1107`: `let expected_hours = planned_hours - abense_hours_for_balance - absence_derived_balance_total - holiday_derived_gated;` — Test `test_holiday_auto_credit_no_year_view_impact` assertiert `expected_hours == 32.0` (40 − 8). GRUEN. |
| 2 | Die abgeleiteten Feiertags-Stunden (`holiday_hours`) erscheinen pro Mitarbeiter in der Schichtplan-Tabelle. (HSP-02) | VERIFIED | `reporting.rs:1101`: `let holiday_hours = holiday_hours + holiday_derived_gated;` — Test assertiert `holiday_hours == 8.0`. GRUEN. |
| 3 | Die Kapazitätsbänder (`paid_hours`/`dynamic_hours`/`committed_voluntary`/`volunteer`) sind in derselben Woche vor und nach der Änderung identisch (Regressions-Guard). (HSP-03) | VERIFIED | `reporting.rs:1116`: `let dynamic_hours = dynamic_hours - abense_hours_for_balance - absence_derived_balance_total;` — `holiday_derived_gated` fehlt bewusst (Kommentar: "band guard"). Test assertiert `dynamic_hours == 40.0`. GRUEN. |
| 4 | Ein Feiertag vor dem konfigurierten Stichtag bleibt wirkungslos, und ein manueller `ExtraHours(Holiday)` wird nicht doppelt gezählt — identisch zum Stundenkonto (Wiederverwendung von `build_derived_holiday_map`). (HSP-04) | VERIFIED | `reporting.rs:1084–1092`: `self.build_derived_holiday_map(...)` aufgerufen — derselbe Pfad wie Injektionspunkte 1a/1b (Zeilen 361, 754). `test_hsp04_before_cutoff`: expected_hours==40, holiday_hours==0. `test_hsp04_manual_wins`: holiday_hours==8.0 (nicht 16.0). Beide GRUEN. |
| 5 | D-34-01: `holiday_derived_gated` nur in `expected_hours` subtrahiert; `booking_information.rs:517`-Block (per-Tag Mo–So) nicht angefasst. | VERIFIED | `reporting.rs:1116` zeigt `dynamic_hours`-Formel ohne `holiday_derived_gated`. `booking_information.rs` nicht in `key_files.modified` der SUMMARY; per Grep keine Änderung an Zeile 517. |
| 6 | D-34-04: `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12 — `billing_period_report.rs` ruft `get_report_for_employee_range`, nicht `get_week`. | VERIFIED | `billing_period_report.rs:117`: `pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 12;`. Grep über `billing_period_report.rs`: kein Aufruf von `get_week` oder `build_derived_holiday_map` — nur `get_report_for_employee_range` (4 Aufrufstellen, Zeilen 146/157/168/179). |

**Score:** 6/6 Wahrheiten verifiziert (0 behavior_unverified)

---

### Erforderliche Artefakte

| Artefakt | Erwartet | Status | Details |
|----------|----------|--------|---------|
| `service_impl/src/reporting.rs` | 4. Injektionspunkt: `holiday_derived_gated`-Term in `get_week` | VERIFIED | Zeilen 1072–1107: Injektionspunkt dokumentiert und implementiert. `build_derived_holiday_map` aufgerufen, `holiday_derived_gated` an `expected_hours` und `holiday_hours` addiert/subtrahiert, `dynamic_hours` unberührt. |
| `service_impl/src/test/reporting_holiday_auto_credit.rs` | HOL-03 in-place umgebaut + `test_hsp04_before_cutoff` + `test_hsp04_manual_wins` | VERIFIED | 795 Zeilen. `test_holiday_auto_credit_no_year_view_impact` (Zeilen 543–627) mit positiven Asserts HSP-01/02/03. `test_hsp04_before_cutoff` (Zeilen 639–712). `test_hsp04_manual_wins` (Zeilen 724–794). Alle drei Tests GRUEN. |

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
| 7 Holiday-Tests grün, inkl. HSP-01/02/03/04 | `SQLX_OFFLINE=true cargo test -p service_impl reporting_holiday` | 7 passed, 0 failed | PASS |
| Clippy sauber | `SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings` | Finished — no warnings | PASS |

Ausgabe des Test-Runs:
```
running 7 tests
test test::reporting_holiday_auto_credit::test_hsp04_manual_wins ... ok
test test::reporting_holiday_auto_credit::test_holiday_manual_wins ... ok
test test::reporting_holiday_auto_credit::test_holiday_auto_credit_no_year_view_impact ... ok
test test::reporting_holiday_auto_credit::test_hsp04_before_cutoff ... ok
test test::reporting_holiday_auto_credit::test_holiday_auto_credit_basic ... ok
test test::reporting_holiday_auto_credit::test_holiday_before_cutoff_skipped ... ok
test test::reporting_holiday_auto_credit::test_holiday_auto_credit_equivalence ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 510 filtered out
```

---

### Anforderungs-Abdeckung

| Anforderung | Plan | Beschreibung | Status | Nachweis |
|-------------|------|--------------|--------|---------|
| HSP-01 | 34-01 | Feiertag reduziert `expected_hours` in `get_week` | SATISFIED | `reporting.rs:1107`; Test-Assert `expected_hours==32.0`. |
| HSP-02 | 34-01 | `holiday_hours` per Mitarbeiter befüllt | SATISFIED | `reporting.rs:1101`; Test-Assert `holiday_hours==8.0`. |
| HSP-03 | 34-01 | Kapazitätsbänder unverändert (Regressions-Guard) | SATISFIED | `reporting.rs:1116` ohne `holiday_derived_gated`; Test-Assert `dynamic_hours==40.0`. |
| HSP-04 | 34-01 | Stichtag-Gate + manual-wins via `build_derived_holiday_map` | SATISFIED | `reporting.rs:1084–1092`; `test_hsp04_before_cutoff` + `test_hsp04_manual_wins` GRUEN. |

Alle 4 HSP-Anforderungen in `REQUIREMENTS.md` als `✅ complete` markiert.

---

### Anti-Pattern-Scan

| Datei | Befund | Bewertung |
|-------|--------|-----------|
| `service_impl/src/reporting.rs` | Keine TBD/FIXME/XXX ohne Issue-Referenz; Injektionspunkt klar kommentiert (HSP-01/02/03-Labels). | Sauber |
| `service_impl/src/test/reporting_holiday_auto_credit.rs` | Keine Stubs; alle neuen Tests mit konkreten Asserts und Fehlermeldungen. | Sauber |

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

## Zusammenfassung

Phase 34 hat ihr Ziel vollständig erreicht. Der 4. Injektionspunkt in `get_week` (`reporting.rs:1072–1107`) verdrahtet `build_derived_holiday_map` korrekt:

- `expected_hours` wird um `holiday_derived_gated` reduziert (HSP-01).
- `holiday_hours` enthält den derived-Beitrag (HSP-02).
- `dynamic_hours` (und damit `paid_hours`-Band) bleibt strikt unangetastet (HSP-03).
- Stichtag-Gate und manual-wins werden aus dem bestehenden Phase-25-Helfer übernommen — keine Neu-Implementierung, keine Doppelzählung (HSP-04).
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12 (D-34-04).
- Clippy sauber; 7/7 Holiday-Tests grün.

---

_Verifiziert: 2026-06-30_
_Verifier: Claude (gsd-verifier)_
