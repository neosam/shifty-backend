---
phase: 57-f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus
plan: 02
subsystem: reporting
tags: [rebooking, reporting, balance, voluntary-hours, tdd, filter-fix]
status: complete
requires:
  - 57-01-SUMMARY.md (VOL-ACCT-04 als Spec-Grundlage, Bestandsdaten-Pre-Flight grün)
  - service/src/extra_hours.rs (ExtraHoursCategory::as_report_type / ::availability, ExtraHoursSource)
provides:
  - "Wirksamer Übertrag freiwillig→bezahlt auf der Leseseite (VOL-ACCT-04, Invarianten I-1/I-2)"
  - "assert_delta-Test-Helper in rebooking_neutrality.rs"
  - "rebooking_effective_in_weekly_summary — CI-Wächter für Site 1 (assemble_weeks)"
  - "build_reporting_service_for_week — Mock-Harness für den get_week-Pfad"
affects:
  - Plan 03 (Balance-Deckel liest die jetzt korrigierte Balance)
  - Plan 04 (Ende-zu-Ende-Test gegen Referenz-Szenario D-57-02 baut hierauf auf)
  - Plan 05 (Snapshot-Bump 12→13 — ein Pair ist ab jetzt nicht mehr snapshot-neutral)
  - Plan 06 (Docs-Update F14 §3 / F07 / time-accounting beschreibt diesen Stand)
tech-stack:
  added: []
  patterns:
    - "Konjunktiv-negiertes Ausnahme-Prädikat (Kategorie UND Source) statt pauschalem Source-Vorfilter"
    - "assert_delta(before, after, expected_delta, aggregate) mit #[track_caller] neben assert_bit_eq"
    - "Getrennter Mock-Harness pro Report-Pfad (Range vs. Woche), weil get_week fünf zusätzliche Service-Aufrufe macht"
key-files:
  created: []
  modified:
    - service_impl/src/reporting.rs
    - service_impl/src/test/rebooking_neutrality.rs
    - service_impl/src/test/rebooking_reconciliation.rs
decisions:
  - "D-57-06 umgesetzt: Sites 2/3/4 verlieren das source-Prädikat ersatzlos, Site 1 bekommt stattdessen eine gezielte VolunteerWork-UND-Rebooking-Ausnahme an der availability()-basierten Summe"
  - "ExtraHoursSource bleibt als Provenienz-Feld erhalten und wird für die Aggregation nur noch an genau einer Stelle herangezogen (assemble_weeks :876)"
  - "RISK-01 (Alt-Inkonsistenz gewöhnlicher VolunteerWork zwischen Wochen- und Jahressicht) bleibt unverändert bestehen — bewusst nicht Scope"
  - "Der Datei-Doku-Satz 'NICHT die Assertion anpassen' aus Phase 56 ist ersatzlos entfallen — er hat den Bug ein ganzes Milestone lang zementiert"
metrics:
  duration: ~35min
  completed: 2026-07-31
---

# Phase 57 Plan 02: Filter-Fix — Rebooking auf der Leseseite wirksam Summary

Der Übertrag freiwillig→bezahlt wirkt jetzt: ein Pair über `h` hebt die Balance um
`h` und senkt das Ehrenamt-Ist um `h` — in beiden Report-Pfaden, mit unverändertem
VOL-ACCT-01-Soll und ohne die Alt-Inkonsistenz RISK-01 anzufassen.

## Was gebaut wurde

### Task 1 — RED: die drei bug-zementierenden Assertions invertiert (Commit `55e86ec3`)

**(a) Datei-Doku-Kopf `rebooking_neutrality.rs` neu geschrieben.** Der alte Kopf
behauptete Bit-Identität als Erfolgskriterium und verbot ausdrücklich, eine
fehlschlagende Assertion anzupassen („echte Entdeckung … NICHT die Assertion
anpassen"). Genau dieser Satz ist der Grund, warum der Bug ein Milestone überlebt
hat — er ist ersatzlos entfallen. Der neue Kopf beschreibt die Lage nach D-57-08:
Balance `+h` (I-1), Ehrenamt-Ist `−h` (I-2), Soll bit-identisch (I-6, weil es aus
`committed_voluntary` stammt und keine `extra_hours` liest). Er benennt außerdem,
dass diese Datei ab jetzt der CI-Wächter gegen ein erneutes Wirkungslos-Werden ist
— die Rolle, die per D-57-04 bewusst NICHT dem HR-Alert-Banner zugewiesen wurde.

**(b) `assert_delta` ergänzt.** `#[track_caller]`-Helper mit derselben
NaN-Vorprüfung wie `assert_bit_eq`, dann `((after − before) − expected_delta).abs()
< 1e-3`. Die Fehlermeldung nennt Aggregat, before, after, erwartetes und
tatsächliches Delta. `assert_bit_eq` bleibt unverändert erhalten — es wird von
`test_neutrality_f2_soll` weiterhin gebraucht.

**(c) Zwei Tests umgestellt.** `test_neutrality_balance` → `assert_delta(…, 2.0,
"balance_hours")`, `test_neutrality_f1_ist` → `assert_delta(…, −2.0, "f1_ist
(volunteer_hours)")`. Die 2.0 stammt aus der bestehenden Fixture
`with_rebooking_pair()` (−2.0 VolunteerWork / +2.0 ExtraWork), sie wurde nicht neu
gewählt. `test_neutrality_f2_soll` blieb wörtlich unverändert; sein Doc-Kommentar
hält jetzt fest, dass er der einzige verbliebene Neutralitäts-Test ist.

**(d) `rebooking_neutrality_single_manual` invertiert.** Das Epsilon-Muster war
dort bereits vorhanden, nur die Zielwerte änderten sich auf `bal_before + 2.0`
bzw. `vol_before − 2.0`. `rebooking_neutrality_roundtrip` blieb wörtlich
unverändert (zwei Pairs heben sich algebraisch auf, filterunabhängig korrekt) —
im Diff ist der Rumpf nachweislich unberührt.

**Gemessener RED-Zustand:** `before=-53, after=-53, expected delta=2, actual delta=0`
— die Zahl `−53` deckt sich exakt mit dem empirischen Befund aus dem Phase-Kontext.

### Task 2 — RED: Wochenübersichts-Wächter für Site 1 (Commit `8ad35a29`)

`build_reporting_service_for_week` spiegelt `build_reporting_service` und ergänzt
genau die fünf Mock-Erwartungen, die der `get_week`-Pfad zusätzlich braucht:
`all_for_week` → die vorhandene `work_details()`-Row,
`extract_shiftplan_report_for_week` → leer (die Wirkung soll ausschließlich aus den
ExtraHours kommen), `find_by_week` → die übergebene Liste, `get_all` →
`make_sales_person()`, `find_all` → leere Absence-Liste. Weitere Stubs waren nicht
nötig: `special_day_service` wird bei `cutoff == None` und leeren Absences gar
nicht aufgerufen (`need_special_days` ist false), und `has_pending_rebooking` ist
im `assemble_weeks`-Pfad hart `false` — der `RebookingBatchService`-Lookup sitzt
ausschließlich in `get_reports_for_all_employees`.

`rebooking_effective_in_weekly_summary` ruft `get_week(2026, 12, …)` zweimal
(Baseline vs. Pair), sucht den Eintrag mit `sales_person.id == sp_id()` und
assertiert `+2.0` auf `balance_hours` und `−2.0` auf `volunteer_hours`. Jahr/Woche
folgen der Fixture: `eh()` datiert alle Rows auf Mi 2026-03-18 (KW 12/2026),
`work_details()` deckt KW 10..=20/2026 ab.

**Gemessener RED-Zustand:** `before=0, after=0, actual delta=0`. Die Baseline-Balance
von exakt 0 ist selbst der Beleg für RISK-01: in der Wochensicht zählt die
gewöhnliche 3.0-VolunteerWork-Row über `availability() == Available` mit (3.0 + 2.0
gegen 5.0 erwartete Stunden), in der Jahressicht nicht.

### Task 3 — GREEN: die vier Filter-Sites korrigiert (Commit `d0550077`)

| Site | Ort | Eingriff |
|---|---|---|
| 2 | `get_reports_for_all_employees` | `.filter(source == Manual)` + Phase-55-Kommentar ersatzlos gestrichen |
| 3 | `overall_extra_work_hours` | dito |
| 4 | `hours_per_week` | dito |
| 1 | `assemble_weeks` | Source-Prädikat am Bucket-Load gestrichen, dafür an der `extra_working_hours`-Summe eine gezielte Ausnahme |

Für Sites 2–4 trägt die Kategorie-Semantik die Fachlogik bereits korrekt: der
jeweilige Downstream-Filter über `as_report_type() == ReportType::WorkingHours`
lässt `ExtraWork` in die Balance und `VolunteerWork` nicht. Der Source-Marker hatte
sie nur pauschal vorgeschaltet blockiert.

Site 1 ist der Sonderfall (D-57-06): Die `extra_working_hours`-Summe filtert über
`availability()`, nicht über `as_report_type()`, und `VolunteerWork.availability()`
ist `Available`. Das neue Prädikat verwirft daher konjunktiv genau die Kombination
`VolunteerWork` UND `Rebooking`:

```rust
.filter(|eh| {
    !(eh.category == ExtraHoursCategory::VolunteerWork
        && eh.source == ExtraHoursSource::Rebooking)
})
```

Ohne diese Ausnahme würde das `−h`-Bein hier mitgezählt und die Anhebung des
`+h`-Beins netto neutralisieren — der Bug bliebe an genau dieser einen Stelle live.
Die Einschränkung auf `source == Rebooking` ist zwingend: gewöhnliche
VolunteerWork-Rows bleiben unverändert Teil dieser Summe (RISK-01).

`manual_volunteer_hours` blieb wörtlich unverändert — dort MUSS die `−h`-Row drin
bleiben, denn genau sie senkt das Ehrenamt-Ist. Der Kopf-Kommentar (früher Zeilen
185–198) wurde auf den Stand nach Phase 57 umgeschrieben: Kategorie-Semantik
entscheidet, `ExtraHoursSource` ist reine Provenienz und wirkt für die Aggregation
nur noch an der einen `availability()`-basierten Summe; mit Referenz auf
VOL-ACCT-04, D-57-06 und RISK-01.

## Verifikation

### Hard Gates

| Gate | Ergebnis |
|---|---|
| `cargo build` | ✅ Finished |
| `cargo test --workspace` | ✅ alle Suites ok, 0 failed (u. a. 826 in `service_impl`, 66 Integration) |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ exit 0 |

### Akzeptanzkriterien

| Kriterium | Erwartet | Gemessen |
|---|---|---|
| `grep -c 'fn assert_delta' rebooking_neutrality.rs` | 1 | 1 |
| `grep -c 'fn assert_bit_eq' rebooking_neutrality.rs` | 1 | 1 |
| `grep -c 'VOL-ACCT-04' rebooking_neutrality.rs` | ≥ 1 | 11 |
| `grep -c 'fn rebooking_effective_in_weekly_summary'` | 1 | 1 |
| `grep -c 'fn build_reporting_service_for_week'` | 1 | 1 |
| `grep -c 'get_week' rebooking_neutrality.rs` | ≥ 1 | 5 |
| `grep -c 'eh.source == ExtraHoursSource::Manual' reporting.rs` | 0 | 0 |
| `grep -c 'ExtraHoursSource::Rebooking' reporting.rs` | ≥ 1 | 1 |
| Diff-Rumpf `rebooking_neutrality_roundtrip` | unverändert | kein Treffer im Diff |
| Diff berührt `alert_predicate` / `proposed_hours` / `manual_volunteer_hours` / `has_pending_rebooking` | nein | kein Treffer im Diff |

### TDD-Gate-Sequenz

| Gate | Commit | Beleg |
|---|---|---|
| RED (1) | `55e86ec3` | drei Assertion-Fehlschläge (kein Compile-Fehler), Clippy grün |
| RED (2) | `8ad35a29` | `rebooking_effective_in_weekly_summary` FAILED mit Delta 0.0, Clippy grün |
| GREEN | `d0550077` | 68 Rebooking-Tests grün, `volunteer_extra_hours_excluded_from_balance_and_overall` grün, Workspace grün |

Die vier Sites sind vollständig abgedeckt: Sites 3/4 über
`get_report_for_employee_range` (`test_neutrality_balance`,
`test_neutrality_f1_ist`, `rebooking_neutrality_single_manual`), Site 1 über
`get_week` (`rebooking_effective_in_weekly_summary`), Site 2 über den
Short-Report-Pfad, dessen Downstream-Semantik der unveränderte
`volunteer_extra_hours_excluded_from_balance_and_overall` festnagelt.

## Prohibitions — Status

| Prohibition | Status | Beleg |
|---|---|---|
| Site-1-Eingriff nicht auf gewöhnliche VolunteerWork ausweiten (RISK-01) | eingehalten | Prädikat verknüpft Kategorie UND Source konjunktiv; `grep -c 'ExtraHoursSource::Rebooking'` = 1; `volunteer_extra_hours_excluded_from_balance_and_overall` unverändert grün |
| `alert_predicate` / `proposed_hours` nicht ändern (I-5, D-57-01) | eingehalten | `git diff service_impl/src/reporting.rs` enthält keinen Treffer auf beide Symbole; die HR-Formel `min(\|balance\|, ist)` steht unverändert |
| Kein Eingriff am HR-Alert-Banner (D-55-07, D-57-04) | eingehalten | Diff berührt weder die `has_pending_rebooking`-Berechnung noch den `RebookingBatchService`-Lookup |
| `manual_volunteer_hours` unverändert | eingehalten | kein Diff-Treffer; die `−h`-Rebooking-Row bleibt dort drin und senkt das Ehrenamt-Ist |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 — Doku-Korrektheit] Datei-Doku-Kopf von `rebooking_reconciliation.rs` mit-korrigiert**

- **Gefunden bei:** Task 1 (d)
- **Problem:** Der Plan schrieb für diese Datei nur den Doc-Kommentar über
  `rebooking_neutrality_single_manual` vor. Der Datei-Doku-Kopf (Test-Achse 1)
  behauptete jedoch weiterhin wörtlich, eine einzelne Umbuchung lasse
  `balance_hours` UND `volunteer_hours` „UNVERAENDERT" — nach der Inversion eine
  aktiv falsche Aussage direkt über dem geänderten Test.
- **Fix:** Achse 1 auf VOL-ACCT-04 / D-57-08 umgeschrieben, mit Verweis auf
  `test_neutrality_f2_soll` als verbliebenen Neutralitäts-Beleg und auf die
  weiterhin algebraisch neutrale Roundtrip-Eigenschaft.
- **Datei:** `service_impl/src/test/rebooking_reconciliation.rs`
- **Commit:** `55e86ec3`

**2. [Rule 3 — Blocking] Clippy `doc_lazy_continuation` an einem Fixture-Kommentar**

- **Gefunden bei:** Task 1, Clippy-Gate
- **Problem:** Die neu formulierte Zeile `/// + +2 ExtraWork, …` wurde von Clippy
  als Listen-Fortsetzung ohne Einrückung gewertet — `-D warnings` schlug fehl.
- **Fix:** Formulierung auf „-2 VolunteerWork und +2 ExtraWork" umgestellt.
- **Datei:** `service_impl/src/test/rebooking_reconciliation.rs`
- **Commit:** `55e86ec3`

Keine weiteren Abweichungen — die vier Sites wurden exakt nach dem Plan-Sketch
korrigiert.

## Auth-Gates

Keine.

## Known Stubs

Keine.

## Requirements-Status

`requirements mark-complete` wurde nur für **VOL-ACCT-03** ausgeführt: dessen
Neufassung (Soll-Schutz) ist mit diesem Plan vollständig belegt —
`test_neutrality_f2_soll` bleibt bit-identisch grün, und mehr fordert das
Requirement nach D-57-09 nicht.

**VOL-ACCT-04 wurde bewusst NICHT als complete markiert.** Dieser Plan liefert die
Leseseite (Invarianten I-1 und I-2); die Akzeptanzbedingung des Requirements ist
aber das vollständige Referenz-Szenario D-57-02, das erst **Plan 04** als
Ende-zu-Ende-Test schreibt. Das Abhaken gehört dorthin — eine frühere Markierung
würde Coverage behaupten, die noch nicht gemessen ist.

## Docs-Freshness-Gate

Dieser Plan berührt `service_impl/src/reporting.rs` (Kategorie-Semantik der
Aggregation) und löst damit laut CLAUDE.md die Trigger-Zeile für
`docs/features/F07-reporting-balance.md` + `docs/domain/time-accounting.md`
(beide Sprachen) aus. Diese Aktualisierung ist **Plan 06 (Wave 5)** zugewiesen —
`57-06-PLAN.md` führt genau diese vier Dateien plus `F14-rebooking(_de).md` in
`files_modified` und fordert in seinen `must_haves` wörtlich die Beschreibung der
Marker-Regel im Stand nach D-57-06. Das Gate ist also auf Phasenebene abgedeckt
und wird beim Milestone-Close von Plan 06 geschlossen, nicht hier dupliziert.

## Threat Flags

Keine neue Angriffsfläche. T-57-04 (zu weit gefasste Ausnahme an Site 1) ist wie
im Threat-Register vorgesehen mitigiert: das Prädikat ist konjunktiv, und beide
Seiten sind getestet (`rebooking_effective_in_weekly_summary` für die Wirksamkeit,
`volunteer_extra_hours_excluded_from_balance_and_overall` für die unveränderte
RISK-01-Semantik). T-57-05 (rückwirkende Aggregations-Änderung) ist durch den
grünen Bestandsdaten-Pre-Flight aus Plan 01 abgesichert. `Cargo.toml` ist
unverändert — keine Paket-Installation.

## Anschluss

- **Plan 03** kann den Balance-Deckel gegen eine Balance implementieren, die
  Rebooking-Wirkung jetzt tatsächlich abbildet.
- **Plan 05** hat mit diesem Commit den harten Beleg für den Snapshot-Bump 12→13:
  ein Rebooking-Paar ist ab jetzt nicht mehr snapshot-neutral.
- **Plan 06** dokumentiert den hier hergestellten Stand (F14 §3, F07,
  time-accounting).

## Commits

| Commit | Inhalt |
|---|---|
| `55e86ec3` | `test(57-02)`: RED — Neutralitäts-Assertions auf VOL-ACCT-04-Wirksamkeit invertieren |
| `8ad35a29` | `test(57-02)`: RED — Wochenübersichts-Wächter für Site 1 (assemble_weeks) |
| `d0550077` | `fix(57-02)`: GREEN — Rebooking auf der Leseseite wirksam machen (VOL-ACCT-04) |

## Self-Check: PASSED

- `service_impl/src/reporting.rs` — FOUND
- `service_impl/src/test/rebooking_neutrality.rs` — FOUND
- `service_impl/src/test/rebooking_reconciliation.rs` — FOUND
- `.planning/phases/57-.../57-02-SUMMARY.md` — FOUND
- Commit `55e86ec3` — FOUND
- Commit `8ad35a29` — FOUND
- Commit `d0550077` — FOUND
- `cargo build` / `cargo test --workspace` / `cargo clippy --workspace -- -D warnings` — alle grün
