# Phase 17: Contract editor input + „alle"-Filter / unpaid-volunteer path - Context

**Gathered:** 2026-06-24
**Status:** Ready for planning

<domain>
## Phase Boundary

`committed_voluntary` wird im Vertrags-Editor (`contract_modal.rs`) als numerisches Feld editierbar, durch beide `TryFrom`-Richtungen in `state/employee_work_details.rs` gefädelt (heute hardcoded `0.0` — Phase-17-Gap-Kommentar). Die Mitarbeiteransicht bekommt einen einblendbaren „alle"-Filter, über den rein unbezahlte Freiwillige (`SalesPerson.is_paid = false`) mit einem `EmployeeWorkDetails`-Record (`expected_hours = 0`, `committed_voluntary > 0`) sichtbar/auswählbar werden. Jede work-details-iterierende paid-only-Site wird explizit auf `sales_person.is_paid` gegatet (nicht auf Record-Präsenz) — kein Leak in `paid_hours`/Billing/Year-Summary. Zusätzlich wird der Reporting-Read-Gate für `committed_voluntary` von `cap == true` auf `cap == true || expected_hours == 0` erweitert (Achse B / `get_weekly_summary`), damit auch rein unbezahlte Freiwillige mit ihrer Zusage in der Jahresansicht-Kapazität erscheinen.

**NICHT in dieser Phase (v1.5):** Inline-Banner „Zusage nicht erfüllt" (CVC-F-01); eigenes committed-Band-Refinement über das in Phase 16 ausgelieferte hinaus.

</domain>

<decisions>
## Implementation Decisions

### Editor-Feld (`contract_modal.rs`)
- **D-01 (Sichtbarkeits-Bedingung):** Das numerische `committed_voluntary`-Feld ist sichtbar/editierbar, wenn `cap_planned_hours_to_expected == true` **ODER** `expected_hours == 0`. Bei einer gedeckelten Person ist es die Zusage-obendrauf; bei einer 0-Sollstunden-Person (rein freiwillig) ist es der einzige Kapazitätswert. Sonst ausgeblendet (würde wirkungslos sein). Vorlage: der bestehende `expected_hours`-`TextInput` direkt darüber (`input_type="number"`, `step="0.01"`).
- **D-02 (State-Threading):** `committed_voluntary` wird als Feld auf die Frontend-`EmployeeWorkDetails`-State-Struct gezogen und in **beiden** `TryFrom`-Richtungen (`state/employee_work_details.rs`) durchgezogen. Der bestehende `committed_voluntary: 0.0`-Hardcode (HEAD-Gap-Kommentar ~Zeile 218) wird durch den echten Feldwert ersetzt. Open→Save-unverändert-Round-Trip muss den Backend-Wert bewahren (CVC-09). Numerisches Input → der `<input type=date>`-Signal-Caveat gilt hier NICHT.

### „alle"-Filter (Mitarbeiteransicht)
- **D-03 (Filter-Semantik):** Default zeigt **bezahlte** Mitarbeiter; ein einblendbarer „alle"-Toggle deckt **zusätzlich rein unbezahlte Freiwillige** (`is_paid = false`) auf. Inaktive Mitarbeiter bleiben über den bestehenden, separaten `!sales_person.inactive`-Filter ausgeblendet (KEINE Kombination mit „alle"). Der Paid-Default sitzt vermutlich im Loader-/Lade-Pfad (`employees_list.rs` filtert heute nur `!inactive`) — Planner verifiziert, wo die Paid-Restriktion greift.

### Unpaid-volunteer Record-Pfad (D-UNPAID-RECORD)
- **D-04 (Erzeugungs-Pfad):** Über den **bestehenden Vertrags-Editor**. `SalesPerson.is_paid = false` wird wie bisher in `sales_person_details.rs` gesetzt (bereits vorhandene Checkbox — **kein** neues is_paid-Control im `contract_modal`). Der `EmployeeWorkDetails`-Record (`expected_hours = 0`, `committed_voluntary > 0`) entsteht über das `contract_modal`. Anti-Pattern (aus ROADMAP-Notes): unpaid Volunteers KEINEN paid-style-Record geben — `expected_hours > 0` würde sie in paid-Loops flippen.

### Reporting-Gate-Erweiterung (Achse B)
- **D-05 (Read-Gate erweitern, KEIN Snapshot-Bump):** Der `committed_voluntary`-Read-Gate in `booking_information.rs::get_weekly_summary` (Achse B, Phase 15) wird von `cap_planned_hours_to_expected == true` auf `cap_planned_hours_to_expected == true || expected_hours == 0` erweitert. Damit fließt auch die Zusage rein unbezahlter Freiwilliger in die aggregierte Jahresansicht-Kapazität ein. Erfordert Anpassung des Phase-15-Pfads + neue/erweiterte Fixtures. **KEIN `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump:** die Erweiterung bleibt — wie der ganze committed-Pfad — Achse-B-only (`get_weekly_summary`/Jahresansicht) und berührt keinen persistierten `BillingPeriodValueType`; Version bleibt **7** (konsistent mit Phase-15-D-01 + CLAUDE.md § Snapshot Versioning).
- **D-06 (Interaktion committed-Read vs. is_paid-Gate — kritisch):** Der `committed_voluntary`-Read-Gate (D-05, Achse B) ist **unabhängig** vom is_paid-Gate. Eine unbezahlte Person (`is_paid = false`) trägt ihre Zusage auf der **freiwilligen** Achse (Band 1, committed) zur Jahresansicht bei, während ihre (0) bezahlten Stunden durch das is_paid-Gating NICHT in `paid_hours`/Billing lecken. Die at-risk-Sites iterieren unbezahlte Personen bereits (`reporting::get_week` nutzt `all_for_week`, nicht paid-gefiltert — die Haupt-Überraschung) — daher ist das per-Site-is_paid-Gating Pflicht, der committed-Read aber gewollt.

### Blank/0-Anzeige (Mitarbeiteransicht)
- **D-07 (schlicht „0"):** `committed == 0` wird in der Mitarbeiteransicht schlicht als `0` (bzw. `🎯0.00`) gezeigt — KEINE blank/Strich-Sonderlogik. Konsistent mit Phase-16-D-03 (Jahresansicht). Die in Phase 16 hierher verschobene blank/Strich-Idee wird damit abschließend verworfen.

### Claude's Discretion / Planner
- **is_paid-Gating-Stil (D-GATING-STYLE):** Planner entscheidet site-by-site nach lokalem Fit (zentraler Helper/Filter vs. inline pro Site). **Invariante:** jede enumerierte at-risk-Site MUSS auf `sales_person.is_paid` gegatet sein (nicht auf Record-Präsenz), und ein `get_week`-Seiteneffekt-Integrationstest sichert ab: kein Leak in `paid_hours`/Billing/Year-Summary + Personen-Set-Konsistenz über year-summary / all-employees-report / Billing (CVC-10).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap / Requirements
- `.planning/ROADMAP.md` § „Phase 17: Contract editor input + „alle"-Filter / unpaid-volunteer path" — Goal, Success Criteria, **Notes for plan-phase** (enumerierte at-risk-Sites für is_paid-Gating).
- `.planning/REQUIREMENTS.md` — CVC-09 (Editor-Feld + Round-Trip), CVC-10 (alle-Filter + unpaid-Record + is_paid-Gating + get_week-Test).

### Vorgänger-Phasen (Pflichtlektüre)
- `.planning/phases/15-reporting-no-double-count-snapshot-bump-same-commit/15-CONTEXT.md` — Zwei-Band-Modell (Band 1 = committed, Band 2 = surplus), Cap-Gating-Logik (die D-05 hier erweitert), no-double-count-Invariante, KEIN-Snapshot-Bump-Begründung (Achse B).
- `.planning/phases/16-jahresansicht-display/16-CONTEXT.md` — D-03 (blank/Strich → „0", hierher verschoben → in D-07 abschließend entschieden), `committed_voluntary_hours`-Display-Pfad TO→State→Render.
- `.planning/phases/14-data-model-foundation-backend/14-CONTEXT.md` — `committed_voluntary`-Feld-Foundation (DAO/Service/TO), Carry-Forward-Semantik, D-OVERLAP-AGG.

### Snapshot-Versioning
- `shifty-backend/CLAUDE.md` § „Billing Period Snapshot Schema Versioning" — begründet, warum D-05 (Achse-B-only) KEINEN Bump braucht; Version bleibt 7.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `shifty-dioxus/src/component/contract_modal.rs`: Der `expected_hours`-`TextInput` (Zeilen ~298–319, `input_type="number"`, `step="0.01"`, parse `f32` → dispatch) ist die 1:1-Vorlage für das `committed_voluntary`-Feld. Der Cap-Toggle (`FormCheckbox`, ~Zeile 381) liefert das `cap_planned_hours_to_expected`-Signal für die D-01-Sichtbarkeitsbedingung.
- `shifty-dioxus/src/page/sales_person_details.rs`: bestehende `is_paid`-Checkbox (~Zeile 152, `UpdateSalesPerson`) — der Pfad zum Setzen von `is_paid = false` (D-04). KEIN Duplikat im contract_modal nötig.
- `shifty-dioxus/src/component/employee_view.rs`: `is_paid`-Pill/Label (`Key::Paid` / unpaid) — bestehendes Muster zur Visualisierung des Paid-Status in der Mitarbeiteransicht.

### Established Patterns
- `shifty-dioxus/src/state/employee_work_details.rs`: zwei `TryFrom`-Richtungen (`&EmployeeWorkDetailsTO → EmployeeWorkDetails` ~Zeile 145, `&EmployeeWorkDetails → EmployeeWorkDetailsTO` ~Zeile 185). Der `committed_voluntary: 0.0`-Hardcode mit „Phase 17 scope"-Kommentar (~Zeile 213–218) markiert exakt, wo D-02 greift. `cap_planned_hours_to_expected` ist das Zeile-für-Zeile-Präzedenz-Feld.
- `shifty-dioxus/src/component/employees_list.rs`: Filter-Kette `.filter(|e| !e.sales_person.inactive).filter(matches_search)` (~Zeile 82–88) — Einhängepunkt für den „alle"-Toggle (D-03). `is_paid` liegt auf `e.sales_person`.

### Integration Points
- Backend at-risk-Sites (Pflicht-Gating, ROADMAP-Notes): `reporting::get_week` (`all_for_week`, nicht paid-gefiltert — Haupt-Überraschung), `booking_information` `paid_hours`-Akkumulation + day-level loop, `reporting::get_reports_for_all_employees` (is_paid-gefiltert → Personen-Set-Inkonsistenz-Risiko), `billing_period_report::build_new_billing_period` (`get_all`, kein paid-Filter), `vacation_balance` (`get_all_paid` — verifizieren, dass nichts direkt work-details liest), ggf. `loader.rs` (Paid-Default des Filters).
- `booking_information.rs::get_weekly_summary`: der committed-Read-Gate (D-05) — von `cap` auf `cap || expected_hours == 0` erweitern.

</code_context>

<specifics>
## Specific Ideas

- Reporting-Gate-Bedingung soll exakt der Editor-Sichtbarkeitsbedingung entsprechen (`cap || expected_hours == 0`) — symmetrisch, damit „editierbar" ⇔ „wird gezählt" gilt (D-01 ⇔ D-05).
- is_paid wird bewusst NICHT im Vertrags-Editor dupliziert — bestehender `sales_person_details`-Pfad bleibt die einzige is_paid-Quelle.

</specifics>

<deferred>
## Deferred Ideas

- Inline-Banner „Zusage nicht erfüllt" (committed > actual) → **v1.5 (CVC-F-01)**.
- Blank/Strich-Darstellung statt „0" → final verworfen (D-07), nicht in v1.5 weitertragen.

### Reviewed Todos (not folded)
- `.planning/todos/pending/2026-06-22-committed-voluntary-capacity-jahresansicht.md` — der v1.4-Umbrella-Todo (deckt das gesamte Milestone 14–17 ab, nicht Phase-17-spezifisch). Wird beim Milestone-Abschluss geschlossen, nicht hier gefoldet.

</deferred>

---

*Phase: 17-contract-editor-unpaid-volunteer-path*
*Context gathered: 2026-06-24*
