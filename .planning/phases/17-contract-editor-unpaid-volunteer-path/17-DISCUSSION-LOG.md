# Phase 17: Contract editor input + „alle"-Filter / unpaid-volunteer path - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-24
**Phase:** 17-contract-editor-unpaid-volunteer-path
**Areas discussed:** Editor-Feld committed_voluntary, „alle"-Filter Semantik + UI, Unpaid-volunteer Record-Pfad, is_paid-Gating + Blank/0-Anzeige, Reporting-Gate-Reconciliation

---

## Editor-Feld committed_voluntary

| Option | Description | Selected |
|--------|-------------|----------|
| Nur bei cap=true sichtbar | Feld erscheint erst wenn Cap-Toggle an | |
| Immer sichtbar | dauerhaft sichtbar, ggf. disabled wenn cap=false | |

**User's choice (free text):** „Bei cap=true ODER wenn die Sollstunden = 0 sind."
**Notes:** Verfeinerung gegenüber den vorgelegten Optionen → Sichtbarkeitsbedingung `cap_planned_hours_to_expected == true || expected_hours == 0` (D-01). Deckt sowohl gedeckelte Personen als auch rein freiwillige 0-Sollstunden-Personen ab.

---

## „alle"-Filter Semantik + UI

| Option | Description | Selected |
|--------|-------------|----------|
| Default paid, „alle" zeigt unbezahlte | Standard zeigt bezahlte; Toggle deckt unbezahlte Freiwillige auf; inaktive separat | ✓ |
| „alle" = unbezahlte + inaktive | kombinierte „wirklich alles"-Sicht | |
| Zwei getrennte Toggles | separate Schalter für unbezahlte / inaktive | |

**User's choice:** Default paid, „alle" zeigt unbezahlte (D-03).
**Notes:** Inaktive bleiben über den bestehenden `!inactive`-Filter ausgeblendet, nicht mit „alle" kombiniert.

---

## Unpaid-volunteer Record-Pfad

| Option | Description | Selected |
|--------|-------------|----------|
| Über bestehenden Vertrags-Editor | is_paid (SalesPerson) false, expected=0, committed>0 | ✓ |
| SalesPerson.is_paid zuerst setzen | is_paid getrennt, Editor schreibt nur Record | |
| Separater Onboarding-Pfad | eigener „Freiwillige hinzufügen"-Flow | |

**User's choice:** Über bestehenden Vertrags-Editor (D-04).
**Notes:** Code-Scout ergab: `is_paid` ist bereits in `sales_person_details.rs` per Checkbox editierbar (nicht im contract_modal). Damit ist der gewählte Pfad sauber: is_paid wie bisher in sales_person_details setzen, Work-Details-Record (expected=0, committed>0) via contract_modal. Kein neues is_paid-Control im Vertrags-Editor.

---

## Blank/0-Anzeige (Mitarbeiteransicht)

| Option | Description | Selected |
|--------|-------------|----------|
| Auch hier schlicht „0" | konsistent mit Jahresansicht D-03 | ✓ |
| Blank/Strich bei 0 | Strich/leer statt „0" zur semantischen Unterscheidung | |

**User's choice:** Auch hier schlicht „0" (D-07).
**Notes:** Schließt die aus Phase 16 hierher verschobene blank/Strich-Idee final ab (verworfen).

---

## Reporting-Gate-Reconciliation

| Option | Description | Selected |
|--------|-------------|----------|
| Nein — Gate bleibt cap=true | Phase-15-Reporting unverändert; unbezahlte committed zählt nicht zur Jahresansicht | |
| Ja — Gate auf cap ODER expected=0 | Read-Gate erweitern, damit unbezahlte Freiwillige in der Jahresansicht-Kapazität erscheinen | ✓ |

**User's choice:** Ja — Gate auf `cap || expected==0` (D-05).
**Notes:** Symmetrie zur Editor-Sichtbarkeit (D-01). Erfordert Anpassung des Phase-15-Pfads (`get_weekly_summary`) + neue Fixtures. KEIN Snapshot-Bump (Achse-B-only, Version bleibt 7). Interaktion mit is_paid-Gate als D-06 dokumentiert: committed zählt auf der freiwilligen Achse, paid-Stunden lecken nicht.

---

## Claude's Discretion

- **is_paid-Gating-Stil:** User wählte „Du entscheidest (Planner)" — Planner wählt zentraler Helper vs. inline pro Site nach lokalem Fit. Invariante: jede at-risk-Site auf `sales_person.is_paid` gegatet + `get_week`-Integrationstest (D-GATING-STYLE).

## Deferred Ideas

- Inline-Banner „Zusage nicht erfüllt" → v1.5 (CVC-F-01).
- Blank/Strich-Darstellung → final verworfen (D-07).
