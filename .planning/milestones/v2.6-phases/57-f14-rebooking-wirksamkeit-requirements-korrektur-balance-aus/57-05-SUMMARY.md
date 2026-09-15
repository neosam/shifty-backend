---
phase: 57-f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus
plan: 05
subsystem: billing-period
tags: [snapshot-versioning, rebooking, billing-period, locking-tests, golden-fixture]
status: complete
requires:
  - 57-02-SUMMARY.md (Filter-Fix — erst dadurch ist ein Rebooking-Paar nicht mehr snapshot-neutral)
  - 57-03-SUMMARY.md (Balance-Deckel — Wave-3-Vorgänger)
  - CLAUDE.md § "Billing Period Snapshot Schema Versioning" (Bump-Trigger-Regeln)
provides:
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION = 13 mit v13-Historien-Absatz (D-57-10)"
  - "test_straddling_snapshot_rebooking_shifts_values — empirischer Beleg der ±h-Verschiebung"
  - "assert_snapshot_shift / compute_snapshot_for / straddling_extra_hours_without_rebooking"
  - "Range-treuer ExtraHours-Mock in der Straddling-Fixture"
affects:
  - Plan 06 (Docs-Update F08 §7.4 v13-Zeile, docs/domain/billing-period Snapshot-Version 13)
tech-stack:
  added: []
  patterns:
    - "Golden-Fixture mit zwei getrennten Beweislasten in einer Datei (Toggle-Unabhängigkeit vs. Datensatz-Verschiebung), im Doku-Kopf explizit auseinandergehalten"
    - "assert_snapshot_shift: erwartete Verschiebung je value_type mit Toleranz, Byte-Identität für den Rest — eine Verschiebung von 0 ist die Fehlermeldung"
    - "Mock-Erwartungen müssen ihre Range-Parameter respektieren, sobald der Produktionscode dieselbe fn mit mehreren Ranges aufruft"
key-files:
  created: []
  modified:
    - service_impl/src/billing_period_report.rs
    - service_impl/src/test/billing_period_straddling.rs
    - service_impl/src/test/billing_period_snapshot_locking.rs
    - service_impl/src/test/booking_information_vfa.rs
    - service_impl/src/test/booking_information.rs
    - service_impl/src/test/pdf_export_config.rs
    - service_impl/src/test/shiftplan_report.rs
decisions:
  - "D-57-10 umgesetzt: CURRENT_SNAPSHOT_SCHEMA_VERSION 12 → 13, begründet mit dem CLAUDE.md-Trigger 'Change the input set the computation reads from'"
  - "Der v13-Absatz nennt VIER betroffene value_types statt der drei geplanten — Overall verschiebt sich mit, weil overall_hours den overall_extra_work_hours-Summanden enthält (Messbefund, nicht Annahme)"
  - "D-57-12 eingehalten: kein Recompute, keine Migration bestehender v12-Rows; migrations/ ist unberührt"
  - "Fünf statt drei Locking-Gates nachgezogen — pdf_export_config und shiftplan_report pinnen die Konstante ebenfalls absolut und waren im Plan nicht erfasst"
  - "Der Funktionsname snapshot_schema_version_pinned_at_10 bleibt (historisch, wird aus booking_information_vfa.rs namentlich zitiert); stattdessen ein Kommentar, der den Namen als historisch ausweist"
metrics:
  duration: ~25min
  completed: 2026-07-31
---

# Phase 57 Plan 05: Snapshot-Bump 12→13 und invertierte Golden-Fixture Summary

Die Snapshot-Version steht auf 13 mit einer Begründung nach dem Muster der
v9..v12-Kette, und die Straddling-Fixture beweist jetzt beides: die
Toggle-Unabhängigkeit der Leseseite (unverändert) UND die Verschiebung, die ein
Rebooking-Paar im Snapshot erzeugt — ohne dass eine bestehende Snapshot-Row
angefasst wurde.

## Was gebaut wurde

### Task 1 — Straddling-Fixture invertieren (Commit `ec35e0e0`)

**(a) Datei-Doku-Kopf neu geschrieben.** Der alte Kopf erklärte die
Byte-Identität der drei Toggle-Szenarien zur Beweislast für den „Nein"-Zweig von
D-01 und dafür, dass die Version bei 12 bleibt — mit der Begründung, der
`source == Manual`-Filter sei toggle-unabhängig. Beides ist überholt. Der neue
Kopf trennt zwei Beweislasten sauber:

| | Beweislast 1 | Beweislast 2 |
|---|---|---|
| Aussage | Der Stichtag-Toggle steuert nur die Schreibseite (I-8) | Ein Rebooking-Paar ist seit Phase 57 im Snapshot sichtbar |
| Variabel | Toggle-Wert (pre / post / straddling / dormant) | Datensatz (mit / ohne Paar) |
| Erwartung | byte-identisch | Verschiebung um genau ±h |
| Träger | die drei bestehenden Tests, unverändert | `test_straddling_snapshot_rebooking_shifts_values` |

Die Alarm-Richtung ist für Beweislast 2 umgedreht: Alarm ist jetzt, wenn der
with-vs-without-Vergleich **keine** Verschiebung zeigt — dann filtert eine Kette
das Paar wieder heraus und der Fix aus 57-02 ist wirkungslos geworden.

**(b) Kommentar an `build_reporting_service` korrigiert.** Er begründete die
Toggle-Irrelevanz mit dem Marker-Filter, den es seit 57-02 nicht mehr gibt. Der
neue Grund ist einfacher: der Stichtag wirkt nur auf den Auto-Cron-Write, die
Read-Aggregation liest ihn für Rebooking-Rows gar nicht.

**(c) `straddling_extra_hours_without_rebooking()`** liefert dieselbe manuelle
Baseline (3.0 `VolunteerWork`, 2.0 `ExtraWork`, beide `Manual`, Tags 1 und 2)
ohne die beiden Rebooking-Beine. `straddling_extra_hours()` blieb wörtlich
unverändert — der Diff enthält genau zwei `eh(`-Zeilen, beide additiv.

**(d) `compute_snapshot_for(extra_hours, toggle_value)`** übernimmt den Rumpf;
`build_billing_period_service` nimmt den Datensatz jetzt als Parameter statt ihn
hartzukodieren. `compute_snapshot(toggle_value)` bleibt als dünner Wrapper mit
`straddling_extra_hours()` — dadurch sind die drei bestehenden Testkörper
nachweislich unberührt.

**(e) `assert_snapshot_shift`** (`#[track_caller]`) prüft zuerst
`sales_person_id` und Schlüsselsatz, dann je `value_type`: steht er in
`expected_shifts`, muss die Delta-Differenz dem erwarteten Betrag entsprechen
(Toleranz 1e-3) und `value_ytd_from` byte-identisch sein; sonst müssen alle vier
Felder byte-identisch sein. Die Fehlermeldung nennt beide Rohwerte und den
Hinweis, dass eine Verschiebung von 0 auf einen wiedereingeführten Filter
hindeutet.

**(f) `test_straddling_snapshot_rebooking_shifts_values`** vergleicht beide
Datensätze beim selben Toggle-Wert (`2026-06-01`, die PRE-Referenz) und
assertiert zusätzlich explizit die Existenz des `Volunteer`-Keys auf beiden
Seiten (Pitfall-5-Wächter mit sprechender Meldung).

### Task 2 — Bump auf 13 und fünf Locking-Gates (Commit `d7afe0c7`)

**(a) `billing_period_report.rs`.** Neuer `/// - v13:`-Absatz, im Aufbau dem
v12-Absatz nachempfunden: was sich geändert hat (Marker-Filter entfernt, D-57-06),
welche `value_type`s betroffen sind (Overall, Balance, ExtraWork, Volunteer), wie
Validatoren mit v12 umgehen müssen („older schema", nicht revalidieren), welche
`value_type`s ausdrücklich NICHT betroffen sind (`ExpectedHours`,
`VacationEntitlement`, `VacationDays`, Absence-Kategorien — `rebook()` schreibt
nur `VolunteerWork` und `ExtraWork`, und das Soll stammt aus dem Vertrag). Dazu
der Hinweis auf die gekippte Vorgängerentscheidung D-01 (Phase 56, REB-AUTO-05),
der Verweis auf den Beweis-Test und die ausdrückliche Feststellung, dass
bestehende v12-Rows unangetastet bleiben (D-57-12). Konstante auf 13.

**(b) Locking-Gates.** Der Bump brach **fünf** Tests, nicht drei:

| Datei | Test | Nachgezogen |
|---|---|---|
| `billing_period_snapshot_locking.rs` | `test_snapshot_schema_version_pinned` | Wert + mehrzeiliger Begründungstext + Modul-Doku |
| `booking_information_vfa.rs` | `phase26_vfa_no_snapshot_bump` | Wert + Doc-Kommentar + Begründungssatz; die Aussage „VFA darf nicht bumpen" bleibt |
| `booking_information.rs` | `snapshot_schema_version_pinned_at_10` | Wert + v13-Absatz in der Historien-Kette + Modul-Doku; Funktionsname unverändert, dafür ein Kommentar, der ihn als historisch ausweist |
| `pdf_export_config.rs` | `snapshot_version_unchanged_grep_gate` | Grep-Needle `= 13;` + Doc-Kommentar (nicht im Plan erfasst) |
| `shiftplan_report.rs` | `test_snapshot_schema_version_unchanged` | Wert + Doc-Kommentar + Meldung, die den Wert Phase 57 zuordnet (nicht im Plan erfasst) |

Bei allen fünf wurde nicht nur die Zahl getauscht, sondern auch die Begründung —
das war die ausdrückliche Auflage aus T-57-15.

## Erstlauf-Befunde

Der Plan verlangte, einen Befund zu melden statt die Erwartung abzuschwächen.
Zwei Befunde traten auf, beide **verschärfend**:

**1. `Overall` verschiebt sich mit (+2.0).** Der Plan erwartete drei betroffene
`value_type`s; gemessen wurde ein vierter:

```
value_type `Overall` field `value_delta` diverges: actual=4, expected=2
```

`report_delta.overall_hours` ist `shiftplan_hours_by_week +
overall_extra_work_hours` — das `+h ExtraWork`-Bein hebt den zweiten Summanden
und damit auch Overall. Das deckt sich mit Plan 04 (dort 8.0 → 10.0 auf
`overall_hours`). Die Erwartung wurde **erweitert**, nicht abgeschwächt: der Test
pinnt jetzt vier Verschiebungen, und der v13-Absatz nennt vier value_types.

**2. Der ExtraHours-Mock ignorierte seine Range-Parameter.** Zweiter roter Lauf:

```
value_type `ExtraWork` field `value_ytd_from` diverges: actual=4, expected=2
```

Ursache war nicht die Produktion, sondern die Fixture: `find_by_sales_person_id_and_year_range`
gab per `.returning(move |_, _, _, _, _| …)` bei **jeder** Range alle Rows
zurück. Der Billing-Period-Report ruft die fn aber viermal mit verschiedenen
Ranges (`ytd_from` = Jahresstart .. Perioden-Start−1, `ytd_to`, `full_year`,
`delta`). Dadurch sah auch `report_start` (bis 2026-03-01) das Paar vom
2026-04-15 — und die Datums-Position der Rebooking-Woche, die der Datei-Kopf seit
Phase 56 ausdrücklich behauptet („in der SPÄTEREN Hälfte des Zeitraums"), war
wirkungslos. Siehe Deviations.

## Verifikation

### Hard Gates

| Gate | Ergebnis |
|---|---|
| `cargo build` | ✅ Finished |
| `cargo test --workspace` | ✅ 0 failed (832 in `service_impl` — 831 vorher + 1 neuer, 66 Integration) |
| `cargo clippy --workspace -- -D warnings` | ✅ exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ exit 0 (Testcode mit abgedeckt) |

### Akzeptanzkriterien

| Kriterium | Erwartet | Gemessen |
|---|---|---|
| `grep -c 'fn straddling_extra_hours_without_rebooking'` | 1 | 1 |
| `grep -c 'fn compute_snapshot_for'` | 1 | 1 |
| `grep -c 'fn assert_snapshot_shift'` | 1 | 1 |
| `grep -c 'fn test_straddling_snapshot_rebooking_shifts_values'` | 1 | 1 |
| `grep -c 'BillingPeriodValueType::Volunteer'` (straddling) | ≥ 1 | 4 |
| `cargo test -p service_impl billing_period_straddling` | 4 passed | 4 passed |
| Diff in den drei bestehenden Testkörpern | keine | keine (Diff enthält keine `compute_snapshot(Some…` / `assert_snapshot_bit_eq(` / `contains_key`-Zeile) |
| Diff an den Beträgen in `straddling_extra_hours()` | keine | keine (nur zwei additive `eh(`-Zeilen der neuen Fixture) |
| `grep -c 'pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13;'` | 1 | 1 |
| `grep -c '/// - v13:'` | 1 | 1 |
| v13-Absatz enthält `Balance` / `ExtraWork` / `Volunteer` / `D-57-10` | ja | ja (plus `Overall`) |
| `git status --porcelain migrations/` | leer | leer |

### Testläufe

| Suite | Ergebnis |
|---|---|
| `cargo test -p service_impl billing_period_straddling` | ✅ 4 passed (3 Toggle-Tests unverändert + Shift-Test) |
| `test_snapshot_schema_version_pinned` | ✅ |
| `phase26_vfa_no_snapshot_bump` | ✅ |
| `snapshot_schema_version_pinned_at_10` | ✅ |
| `snapshot_version_unchanged_grep_gate` | ✅ |
| `test_snapshot_schema_version_unchanged` | ✅ |

### TDD-Gate-Sequenz

Task 1 lief als TDD-Task. Die Sequenz ist dokumentiert durch zwei rote Läufe mit
konkreten Zahlen (Overall-Divergenz 4 vs. 2, dann ExtraWork-`value_ytd_from` 4
vs. 2) vor dem grünen Lauf. Beide RED-Zustände führten zu einer **Korrektur der
Erwartung nach oben bzw. einer Fixture-Reparatur**, nicht zu einer Abschwächung —
begründet in „Erstlauf-Befunde".

Der Commit-Typ von Task 1 ist `test(...)`, weil ausschließlich Testcode geändert
wurde; Task 2 ist `feat(...)`, weil dort Produktionscode
(`CURRENT_SNAPSHOT_SCHEMA_VERSION`) bewegt wird.

## Prohibitions — Status

| Prohibition | Status | Beleg |
|---|---|---|
| Keine Repair-Migration, kein Recompute bestehender v12-Rows (D-57-12) | eingehalten | `git status --porcelain migrations/` leer; `billing_period_report.rs` hat keine Recompute-Funktion, der Diff besteht dort aus dem Kommentarblock plus dem Konstantenwert |
| Fixture-Beträge von `straddling_extra_hours()` unverändert (Pitfall 5) | eingehalten | Diff enthält nur zwei additive `eh(`-Zeilen (3.0 Manual VolunteerWork, 2.0 Manual ExtraWork) in der NEUEN fn; die vier Zeilen der Bestands-Fixture sind unberührt. Der Volunteer-Key existiert beidseitig (1.0 bzw. 3.0) und wird explizit assertiert |
| Die drei bestehenden Toggle-Tests in ihren Assertions unverändert | eingehalten | `git diff -U0` zeigt keine ± Zeile mit `compute_snapshot(Some`, `assert_snapshot_bit_eq(` oder `reference.values.contains_key`; alle drei Tests bleiben grün |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] Vierter betroffener `value_type`: `Overall`**

- **Gefunden bei:** Task 1, erster Testlauf
- **Problem:** Der Plan (und damit die geplante `expected_shifts`-Liste sowie der
  geplante v13-Text) nannte nur `Balance`, `ExtraWork` und `Volunteer`. Gemessen
  verschiebt sich `Overall` ebenfalls um +2.0, weil
  `report_delta.overall_hours = shiftplan_hours_by_week + overall_extra_work_hours`
  das `+h ExtraWork`-Bein enthält.
- **Fix:** `Overall` in `expected_shifts` aufgenommen, in die Herleitungstabelle
  des Test-Doc-Kommentars und in den v13-Historien-Absatz sowie in die
  Begründungstexte aller nachgezogenen Locking-Gates. Damit weiß ein Validator,
  dass er für v12-Snapshots auch `Overall` nicht revalidieren darf — eine
  Auslassung wäre hier ein stiller Falsch-Positiv-Generator gewesen.
- **Dateien:** `service_impl/src/test/billing_period_straddling.rs`,
  `service_impl/src/billing_period_report.rs`
- **Commits:** `ec35e0e0`, `d7afe0c7`

**2. [Rule 1 — Bug] Der ExtraHours-Mock respektierte seine Range-Parameter nicht**

- **Gefunden bei:** Task 1, zweiter Testlauf
  (`ExtraWork.value_ytd_from` divergierte 4 vs. 2)
- **Problem:** `expect_find_by_sales_person_id_and_year_range()` gab per
  `.returning(move |_, _, _, _, _| Ok(eh_arc.clone()))` bei jeder Range **alle**
  Rows zurück. Der Billing-Period-Report ruft dieselbe fn aber viermal mit
  verschiedenen Ranges. Konsequenz: `report_start` (Jahresstart .. 2026-03-01)
  enthielt die Rebooking-Rows vom 2026-04-15, obwohl die Produktion sie
  DAO-seitig herausfiltert. Damit war die must_have-Aussage „`value_ytd_from`
  bleibt byte-identisch, weil die Rebooking-Woche strikt innerhalb des Zeitraums
  liegt" in dieser Fixture nicht prüfbar — und die seit Phase 56 im Datei-Kopf
  behauptete Datums-Position der Rebooking-Woche war folgenlos.
- **Fix:** Der Mock filtert jetzt `eh.date_time.date()` gegen
  `from_date.to_date() ..= to_date.to_date()`. Der Kommentar darüber erklärt,
  warum das nötig ist. Die drei Toggle-Tests bleiben grün (sie vergleichen
  Szenarien untereinander, nicht gegen Absolutwerte).
- **Datei:** `service_impl/src/test/billing_period_straddling.rs`
- **Commit:** `ec35e0e0`

**3. [Rule 3 — Blocking] Zwei weitere Locking-Gates nachgezogen**

- **Gefunden bei:** Task 2, `cargo test --workspace`
- **Problem:** Der Plan nannte drei Tests, die die Konstante festnageln.
  Tatsächlich sind es fünf: `pdf_export_config::snapshot_version_unchanged_grep_gate`
  (Grep-Gate auf den Literal-String `… u32 = 12;`) und
  `shiftplan_report::test_snapshot_schema_version_unchanged` (Phase-51-Chain-D-Gate)
  pinnen den Absolutwert ebenfalls. Beide sind unmittelbar durch den Bump
  gebrochen, also in-scope.
- **Fix:** Beide auf 13 nachgezogen, jeweils mit einem Doc-Absatz, der klarstellt,
  dass der gepinnte Wert der jeweils aktuelle Konstantenwert ist und aus Phase 57
  stammt — nicht aus der Phase, die das Gate ursprünglich aufgestellt hat. Der
  Zweck der Gates („Phase 48 / Chain D darf nicht bumpen") bleibt unverändert.
- **Dateien:** `service_impl/src/test/pdf_export_config.rs`,
  `service_impl/src/test/shiftplan_report.rs` (beide nicht in `files_modified`
  des Plans)
- **Commit:** `d7afe0c7`

**4. [Rule 2 — Doku-Korrektheit] Fehlermeldung von `assert_f32_bit_eq` generalisiert**

- **Gefunden bei:** Task 1 (e)
- **Problem:** Der Plan listet `assert_f32_bit_eq` als bewusst unverändert. Seine
  Meldung endete jedoch mit „a chain leaked the rebooking pair into the snapshot
  => D-01 flips to a 12->13 bump" — nach dem Bump aktiv falsch, und seit Task 1
  hat der Helper zwei Aufrufer mit gegensätzlicher Semantik.
- **Fix:** Nur der Meldungstext benennt jetzt beide Deutungen (Toggle-Leck über
  `assert_snapshot_bit_eq` vs. unerlaubte Verschiebung über
  `assert_snapshot_shift`). Signatur und Vergleichslogik sind unverändert.
- **Datei:** `service_impl/src/test/billing_period_straddling.rs`
- **Commit:** `ec35e0e0`
- **Präzedenz:** Plan 02 hat dieselbe Art Korrektur an einem irreführenden
  Datei-Doku-Kopf als Rule-2-Abweichung geführt.

## Auth-Gates

Keine.

## Known Stubs

Keine.

## Requirements-Status

`requirements mark-complete VOL-ACCT-04` gemeldet: `already_complete` — die
Checkbox steht seit Plan 01 (Requirements-Neufassung) auf `[x]`, inhaltlich
gemessen wurde das Requirement mit Plan 04 (Referenz-Szenario). Dieser Plan
liefert die Snapshot-Konsequenz derselben Änderung; er erweitert die Coverage von
VOL-ACCT-04 um die Persistenz-Seite, ohne ein neues Requirement zu schließen.

## Docs-Freshness-Gate

Dieser Plan berührt `service_impl/src/billing_period_report.rs` und darin
ausdrücklich `CURRENT_SNAPSHOT_SCHEMA_VERSION` — das ist eine Trigger-Zeile der
CLAUDE.md-Tabelle und verlangt `docs/features/F08-billing-period.md` +
`docs/domain/billing-period.md` (beide Sprachen), Bump-Historie und
Version-Tabelle.

Diese Aktualisierung ist **Plan 06 (Wave 5)** zugewiesen und dort verifiziert
abgedeckt: `57-06-PLAN.md` führt alle vier Dateien in `files_modified` und fordert
in seinen `must_haves` wörtlich „F08 §7.4 enthält eine v13-Zeile in der
Bump-Historie" sowie „docs/domain/billing-period nennt die aktuelle
Snapshot-Version 13". Plan 06 hängt per `depends_on` an diesem Plan. Das Gate ist
also auf Phasenebene geschlossen und wird hier nicht dupliziert.

**Nachtrag für Plan 06:** Der v13-Absatz nennt **vier** betroffene value_types
(Overall, Balance, ExtraWork, Volunteer), nicht die drei, die 57-06-PLAN.md
antizipiert. Die F08-Bump-Historie sollte Overall mit aufführen.

## Threat Flags

Keine neue Angriffsfläche — keine neuen Felder, keine neuen Leser, keine Änderung
an Lese-Berechtigungen. Register-Status:

- **T-57-14** (unterlassener oder unbegründeter Bump) — mitigiert: der Bump ist
  vollzogen und durch `test_straddling_snapshot_rebooking_shifts_values` empirisch
  belegt; der Historien-Absatz nennt Trigger, betroffene value_types,
  Validator-Anweisung und die gekippte Vorgängerentscheidung.
- **T-57-15** (naives Durchwinken der Locking-Tests) — mitigiert: alle fünf Gates
  haben aktualisierte Begründungstexte, kein reiner Zahlentausch; die zwei vom
  Plan nicht erfassten Gates wurden gefunden und gleich behandelt.
- **T-57-16** (Recompute/Migration bestehender v12-Rows) — mitigiert:
  `git status --porcelain migrations/` leer, keine Recompute-Funktion.
- **T-57-17** (Information Disclosure) — akzeptiert wie geplant.
- **T-57-SC** — `Cargo.toml` unverändert, keine Paket-Installation.

## Wave-Disziplin

Innerhalb der Wave-4-Grenzen geblieben: `service_impl/src/test/rebooking_reference_scenario.rs`
und `service_impl/src/test/mod.rs` (Plan 04) wurden nicht angefasst. Über die
deklarierten `files_modified` hinaus wurden nur `pdf_export_config.rs` und
`shiftplan_report.rs` berührt — beides zwingend, weil ihre Locking-Gates durch den
Bump brachen (Rule 3), und beides Testcode ohne Überschneidung mit Plan 04.

## Anschluss

- **Plan 06** dokumentiert den Bump in F08 §7.4 und `docs/domain/billing-period`
  (beide Sprachen) — mit dem Nachtrag oben: vier value_types, nicht drei.

## Commits

| Commit | Inhalt |
|---|---|
| `ec35e0e0` | `test(57-05)`: Straddling-Fixture invertieren — Rebooking verschiebt den Snapshot |
| `d7afe0c7` | `feat(57-05)`: CURRENT_SNAPSHOT_SCHEMA_VERSION 12 → 13 (VOL-ACCT-04 / D-57-10) |

## Self-Check: PASSED

- `service_impl/src/billing_period_report.rs` — FOUND
- `service_impl/src/test/billing_period_straddling.rs` — FOUND
- `service_impl/src/test/billing_period_snapshot_locking.rs` — FOUND
- `service_impl/src/test/booking_information_vfa.rs` — FOUND
- `service_impl/src/test/booking_information.rs` — FOUND
- `service_impl/src/test/pdf_export_config.rs` — FOUND
- `service_impl/src/test/shiftplan_report.rs` — FOUND
- `.planning/phases/57-.../57-05-SUMMARY.md` — FOUND
- Commit `ec35e0e0` — FOUND
- Commit `d7afe0c7` — FOUND
- `cargo build` / `cargo test --workspace` / `cargo clippy --workspace -- -D warnings` — alle grün
