# Phase 15: Reporting no-double-count + snapshot bump (SAME commit) - Context

**Gathered:** 2026-06-23
**Status:** Ready for planning

> ⚠ **Phasenname/Scope überholt durch D-01 (unten).** Der Phasentitel "+ snapshot bump (SAME commit)" und CVC-05 sind nach der Diskussion **nicht mehr gültig**: Phase 15 berührt **keinen** persistierten `value_type`, also **kein** Snapshot-Bump. Der Planner MUSS ROADMAP-Phasentitel + REQUIREMENTS (CVC-05) entsprechend nachziehen.

<domain>
## Phase Boundary

Die zugesagte freiwillige Kapazität (`committed_voluntary`, in Phase 14 end-to-end gefädelt) wird **ohne Doppelzählung** in die **Jahresansicht-Verfügbarkeit** eingerechnet — als **separater** `committed_voluntary_hours`-Term in `service_impl/src/booking_information.rs::get_weekly_summary` (**Achse B**), per ISO-Woche via `counted_volunteer = max(committed_voluntary, actual_volunteer)`, **erst pro Woche max, dann über das Jahr summiert** (nie `max(Σ, Σ)`); gegated auf `cap_planned_hours_to_expected = true` (cap=false ⇒ `0.0`, Backward-Compat).

> ⚠ **Modell verfeinert durch D-05 (unten, User-Klärung 2026-06-23):** Es sind **zwei gestapelte Bänder** — Band 1 `committed_voluntary_hours` (Zusage, neue Farbe) + Band 2 `volunteer_hours` (Surplus über Zusage, bestehende Farbe), pro Person via `committed + max(actual − committed, 0)`. Das ersetzt das obige „ein-max-Term"-Framing; Multi-Person ⇒ **Formel B** (= 8, nicht 5). Der **bestehende** `volunteer_hours`-Term wird dafür pro Person reduziert.

**Reine Backend-/Berechnungs-Phase.** Phase 15 ändert nur die Service-seitige Berechnung + die `WeeklySummary`-Service-Struct und pinnt die Semantik per Test. **NICHT in dieser Phase:** `reporting.rs` (Achse A — Balance/Billing/persistierter Snapshot), Snapshot-Versions-Bump, `WeeklySummaryTO`, Frontend-Display (Phase 16), Editor-Input / „alle"-Filter / unpaid-volunteer-Pfad (Phase 17).

</domain>

<decisions>
## Implementation Decisions

### Snapshot & Achse-Scope (CVC-04 vs CVC-05 Widerspruch aufgelöst)
- **D-01 (Achse B only, KEIN Snapshot-Bump):** `committed_voluntary` ist eine reine **Jahresansicht-Verfügbarkeitskapazität** (vorausschauende Zusage, KEINE geleistete Stunde). Sie fließt **ausschließlich** in `booking_information.rs::get_weekly_summary` (Achse B) ein und **nicht** in `reporting.rs` (Achse A → Balance/Billing/persistierter `BillingPeriodValueType::Volunteer`). Verifiziert beim Code-Scout: der Billing-Snapshot speist `Volunteer` aus `reporting_service`-Reports (`billing_period_report.rs:240-247`); `WeeklySummary` wird in `billing_period_report.rs` **nirgends** referenziert (year-view-only, nicht persistiert). **Folge:** kein persistierter `value_type` ändert sich → die CLAUDE.md-Bump-Regel verlangt **KEINEN** Bump → `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **7**. CLAUDE.md deckt das explizit ab ("purely additive changes that do not touch the snapshot's value_types").
- **D-01-Konsequenz für REQUIREMENTS/ROADMAP:** **CVC-05 entfällt** als "Bump 7→8"-Anforderung. Ersetzen durch eine explizite **no-bump-Begründung** (begründen, dass Phase 15 keinen persistierten `value_type` berührt; Version bleibt 7). Phasentitel "+ snapshot bump (SAME commit)" ist überholt. Planner zieht ROADMAP + REQUIREMENTS nach (Begründung statt Bump). Begründung warum *nicht* gebumpt wird gehört trotzdem dokumentiert (Audit-Trail).

### Zwei-Band-Dekomposition (D-05, User-Klärung 2026-06-23 — ÜBERSCHREIBT das „ein-max-Term"-Framing)
- **D-05 (zwei gestapelte Bänder, per-Person-Abzug):** Statt eines einzelnen aggregierten `max(committed, actual)`-Terms gibt es **zwei separate, gestapelte Bänder** (in Phase 16 mit **getrennten Farben** gerendert):
  - **Band 1 „zugesagt" (`committed_voluntary_hours`, neue Farbe):** die zugesagte freiwillige Kapazität = `committed` (flat pro aktiver ISO-Woche, cap-gated). Berechnung = `Σ_Woche Σ_Person committed` — direkt via Phase-14-Helper `committed_voluntary_for_calendar_week`.
  - **Band 2 „freiwillig darüber" (bestehender `volunteer_hours`-Term, bestehende Farbe):** alles _über_ der Zusage. Der **bestehende** `volunteer_hours` wird **pro Person pro ISO-Woche** auf den Surplus reduziert: `volunteer_hours = Σ_Woche Σ_Person max(actual_volunteer_p − committed_p, 0)`.
  - **No-double-count-Invariante:** Summe pro Person/Woche = `committed + max(actual − committed, 0) = max(committed, actual)`. Die beiden Bänder überschneiden sich nie.
- **D-05-Konsequenz — Personen-Überlapp & Per-Person-Pflicht:** Zusage und Freiwilligen-Stunden **können dieselbe Person betreffen** (auch eine `is_paid=false`-Person kann ein `committed_voluntary` haben). Deshalb ist der Surplus-Abzug **zwingend per-Person** (nicht auf Summen-Ebene — `max` ist nichtlinear). Der Phase-14-Helper liefert nur die Personen-**Summe** des `committed` → für den Surplus von Band 2 ist eine **Per-Person-Iteration** über das `is_paid=false`-Set mit deren jeweiligem `committed` nötig. `committed=0` ⇒ Abzug ist No-op ⇒ Band 2 identisch zu heute (Backward-Compat). Für `is_paid=true`-gedeckelte Personen ist `actual_volunteer` in Achse B = 0 (Research Option b) ⇒ deren Band 2 = 0, Band 1 = `committed`.
- **D-05-Konsequenz — Multi-Person-Semantik = Formel B (nicht A):** Die Dekomposition ist mathematisch `Σ_Person max(committed_p, actual_p)` = **Formel B**. Worked Example (A: cap, c=5, a=0 / B: c=0, a=3): `committed_voluntary_hours = 5`, `volunteer_hours = 3`, Gesamt = **8** (NICHT 5). Die frühere RESEARCH-Empfehlung „Formel A / max-of-sums = 5" ist durch diese User-Klärung **revidiert**.

### no-double-count Test-Fixture (maximale Coverage gewünscht)
- **D-02 (alle Kategorien + mehr):** User-Vorgabe „Alles was geht! Mehr Tests sind immer gut!". Pflicht-Fälle:
  - **Kern max-pro-Woche:** `committed=5/actual=7 → 7` (Überschuss zählt) und `committed=5/actual=3 → 5` (Zusage deckt) — pinnt `counted = max(committed, actual)` pro Woche.
  - **Summe über Jahr (nie `max(Σ,Σ)`):** mehrere Wochen mit unterschiedlichen `committed`/`actual` → erst pro Woche `max`, DANN summieren. Wichtigster struktureller Schutz gegen die Hauptfalle.
  - **`cap=false → 0.0`:** für nicht-gedeckelte Personen trägt die Zusage `0.0` bei → Ergebnis identisch zu heute (Backward-Compat-Regressionslock).
  - **Mehrere Personen aggregiert:** `get_weekly_summary` summiert über alle Personen; Test mit 2+ Personen (eine gedeckelt mit Zusage, eine normal) stellt korrektes Zusammenspiel von per-Person-`max` und Personen-Aggregation sicher.
  - **Zusätzliche sinnvolle Fälle (Discretion, gewünscht):** Single-Week, leere/keine aktive Row in der Woche (→ `0.0`), Boundary (`committed == actual`), `committed=0` ⇒ keine Wirkung. Float-Vergleiche via Epsilon, nicht `==`.

### Teilwochen-Gewichtung (Default, Research-empfohlen — nicht separat besprochen)
- **D-03 (flat pro aktiver ISO-Woche):** Die Zusage zählt **flat** pro aktiver ISO-Woche, **kein** Pro-Rating nach `weight_for_week` (anders als `expected_hours`). Begründung (Research D-PARTIAL-WEEK): Pro-Rating würde eine „5h-Zusage" in Rand-/Teilwochen still schrumpfen; flat passt außerdem zum bestehenden, in Phase 14 getesteten SUM-Helper `committed_voluntary_for_calendar_week` (der nicht gewichtet).

### Phase-15/16-Schnitt (Default — nicht separat besprochen)
- **D-04 (Service-Struct + Berechnung in 15; TO + Frontend in 16):** Phase 15 fügt den separaten Term `committed_voluntary_hours: f32` auf der **Service-Struct** `service::booking_information::WeeklySummary` hinzu und implementiert die Berechnung in `get_weekly_summary` (test-gepinnt). **`WeeklySummaryTO` + `From`-Mapping + Frontend-Display** folgen in **Phase 16**. (Entspricht der ROADMAP-Phasenstruktur.)

### Claude's Discretion
- Genaue Test-Modul-/Datei-Platzierung (eigenes Modul in `booking_information.rs` `#[cfg(test)]` vs. bestehendes Test-Modul).
- Ob ein kleiner privater Helper für die per-Woche-`max`-vor-Summe-Reduktion eingeführt wird (analog `committed_voluntary_for_calendar_week` aus Phase 14, das hier wiederverwendet werden kann) oder inline.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone-Research (v1.4) — Pflichtlektüre
- `.planning/research/SUMMARY.md` — Zwei-Achsen-Erkenntnis (Achse A `reporting.rs` vs. Achse B `booking_information.rs`); die `max(committed, actual)`-per-Woche-Closed-Form; D-FORMULA-PATH / D-SCOPE-GATE / D-SNAPSHOT (siehe D-01: die D-SNAPSHOT-Empfehlung "bump" ist durch die Diskussion **revidiert** → kein Bump, da reine Achse B).
- `.planning/research/PITFALLS.md` — P1 (Doppelzählung: replacement-not-addition), P2 (Snapshot-Drift — hier durch D-01 entschärft: kein value_type-Change), P5 (unpaid-volunteer-Leak, relevant Phase 17), P4 (Time-Version-Skew / Overlap-Aggregation).
- `.planning/research/ARCHITECTURE.md`, `.planning/research/STACK.md` — Touch-Boundaries + Reuse-Map.

### Roadmap / Requirements
- `.planning/ROADMAP.md` § „Phase 15: Reporting no-double-count + snapshot bump (SAME commit)" — Goal + Success Criteria (⚠ SC#3/Bump überholt durch D-01).
- `.planning/REQUIREMENTS.md` — CVC-04 (gilt), CVC-05 (entfällt/umformulieren per D-01), CVC-06 (cap=false → 0.0).

### Vorgänger-Phase
- `.planning/phases/14-data-model-foundation-backend/14-CONTEXT.md` + `14-01/14-02-SUMMARY.md` — `committed_voluntary` ist durchgängig gefädelt; SUM-Helper `committed_voluntary_for_calendar_week` (`reporting.rs:101-109`) existiert + ist getestet und hier wiederverwendbar.

### Präzedenz-/Integrations-Code (verifiziert beim Scout)
- `service_impl/src/booking_information.rs::get_weekly_summary` (`:95-218`, zweite Variante `:243-330`) — **der Integrations-Site (Achse B)**. Heute: `volunteer_hours` = Summe gebuchter Stunden der `is_paid=false`-Personen (`volunteer_ids`); `overall_available_hours = volunteer_hours + paid_hours` (`:197`/`:309`).
- `service/src/booking_information.rs:38-53` — `WeeklySummary`-Struct (Ziel für neues Feld `committed_voluntary_hours`).
- `service_impl/src/reporting.rs:101-109` — `committed_voluntary_for_calendar_week` (SUM-Helper aus Phase 14, wiederverwendbar) + `:77-86` `find_working_hours_for_calendar_week` (Selektion).
- `service_impl/src/billing_period_report.rs:74` (`CURRENT_SNAPSHOT_SCHEMA_VERSION = 7`, bleibt) + `:240-247` (`Volunteer` value_type aus `reporting_service`-Reports — **nicht** aus `WeeklySummary`).

### Projekt-Regeln
- `shifty-backend/CLAUDE.md` § „Billing Period Snapshot Schema Versioning" — die Bump-Regel; per D-01 hier NICHT ausgelöst (kein persistierter value_type ändert sich) → no-bump-Begründung dokumentieren.
- `CLAUDE.local.md` + `.planning/STATE.md` "Constraints In Force" — jj-only Commits (kein git/jj-Commit aus Agents), NixOS `nix develop`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `committed_voluntary_for_calendar_week` (`reporting.rs:101-109`, Phase 14): SUM über die in der Woche aktiven Rows — direkt nutzbar, um den per-Person-`committed`-Wert pro ISO-Woche zu holen.
- `find_working_hours_for_calendar_week` (`reporting.rs:77-86`): die Wochen-Selektion (dieselbe wie `expected_hours`).
- `get_weekly_summary` (`booking_information.rs:95+`): bestehender per-Woche-Aggregations-Pfad; der neue Term wird hier eingehängt.

### Established Patterns
- `WeeklySummary` ist year-view-only und wird **nicht** persistiert (kein Billing-Snapshot-Konsum) → additives Feld ohne Snapshot-Wirkung.
- `is_paid` lebt auf `SalesPerson`, NICHT auf `EmployeeWorkDetails`; `get_weekly_summary`'s `volunteer_hours` keyt heute auf `is_paid=false` (`volunteer_ids`).

### Integration Points
- `EmployeeWorkDetails.committed_voluntary` (per Woche via Helper) → neuer `committed_voluntary_hours`-Term auf `WeeklySummary` in `get_weekly_summary`, gegated auf `cap_planned_hours_to_expected=true`, per-Woche `max(committed, actual)` vor Jahres-Summe.

</code_context>

<specifics>
## Specific Ideas

- No-double-count closed form: `counted_volunteer = max(committed_voluntary, actual_volunteer)` **pro ISO-Woche**, dann summiert — niemals `max(Σ, Σ)`.
- `committed=0` ⇒ Ergebnis identisch zu heute (Backward-Compat-Garantie).

</specifics>

<deferred>
## Deferred Ideas

- **WeeklySummaryTO + `From`-Mapping + Frontend „zugesagt"-Token + Überschuss-Anzeige + i18n (de/en/cs)** → **Phase 16**.
- **Editor-Input (`contract_modal.rs`) + „alle"-Filter + unpaid-volunteer-Record + `is_paid`-Gating aller at-risk-Sites** → **Phase 17**.
- **Inline-Banner „Zusage nicht erfüllt" + committed-Band im Chart** → **v1.5** (CVC-F-01/CVC-F-02).

### Research Flag (für gsd-phase-researcher / gsd-planner)
- **„actual_volunteer" für eine bezahlt-gedeckelte Person in Achse B ist undefiniert/zu klären:** `get_weekly_summary`'s `volunteer_hours` zählt heute NUR `is_paid=false`-Personen. Die Motivations-Person („5h bezahlt + 5h zugesagt") ist `is_paid=true` — ihre freiwillige Mehrleistung/Cap-Überlauf taucht in Achse B heute gar nicht auf (der Cap-Überlauf lebt in Achse A `reporting.rs` `auto_volunteer_hours`). **Offene Mechanik-Frage:** Woher kommt das „actual" im `max(committed, actual)` für eine `is_paid=true, cap=true`-Person in `get_weekly_summary`? Optionen, die der Researcher prüfen muss: (a) für gedeckelte paid-Personen ist `actual_volunteer` = deren Cap-Überlauf (müsste in Achse B verfügbar gemacht/berechnet werden); (b) `actual_volunteer` bleibt 0 für paid-Personen in Achse B → der Term reduziert sich auf reines `committed`. Diese Wahl bestimmt die exakte Formel und die worked-example-Fixtures (D-02). **Höchst-Leverage-Gap (deckt sich mit Research „D-SCOPE-GATE landing").**

### Reviewed Todos (not folded)
- `2026-06-09-auswertung-durchschnittliche-anwesenheit-flexible-stunden.md` — per PROJECT.md bewusst NICHT in v1.4 (zu viele offene Definitionsfragen). Keyword-Match (area: reporting), aber out-of-scope.
- `2026-05-08-cutover-ui-admin-feature.md`, `2026-05-05-booking-log-service-500.md`, `2026-05-05-warnung-eintrag-ausserhalb-vertragszeiten.md`, `2026-05-07-review-list-user-invitations-silent-empty.md` — Keyword-False-Positives, kein Bezug zu committed_voluntary/Reporting-Doppelzählung.

</deferred>

---

*Phase: 15-reporting-no-double-count-snapshot-bump-same-commit*
*Context gathered: 2026-06-23*
