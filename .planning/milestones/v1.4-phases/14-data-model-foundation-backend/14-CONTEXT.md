# Phase 14: Data-model foundation (backend) - Context

**Gathered:** 2026-06-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Das zeit-versionierte Feld `committed_voluntary: f32` (D-01 / Variante B — nur die freiwillige Zusage obendrauf, **entkoppelt** von `expected_hours`) existiert durchgängig auf `EmployeeWorkDetails` über alle Backend-Layer: SQLite-Migration → DAO (Entity/Row + `TryFrom`) → Service (Struct + beide Konversionen) → `rest-types` (`EmployeeWorkDetailsTO`). Das Feld ist in dieser Phase **inert** — es transportiert und persistiert, wird aber **nirgends gelesen** und hat keine Reporting-/Display-Wirkung. Es legt die Foundation, auf der Phase 15 (Reporting) liest und Phase 16/17 (Frontend) konsumiert/editiert.

**Reine Backend-Phase, kein Frontend-Anteil** (Feld inert; Frontend folgt in Phase 16/17 — begründeter Skip im Sinne der GSD-Scope-Regel). **Kein REST/OpenAPI-Change** — diese Endpoint-Familie hat bewusst keine `#[utoipa::path]`/`ToSchema` (serde-transparent); kein Phantom-OpenAPI-Task.

**Nicht in dieser Phase:** jede Lese-/Aggregations-Logik an einem Produktions-Read-Site (Phase 15), Snapshot-Bump (Phase 15), Display (Phase 16), Editor-Input + „alle"-Filter + unpaid-volunteer-Pfad (Phase 17).

</domain>

<decisions>
## Implementation Decisions

### Overlap-Aggregation
- **D-OVERLAP-AGG (SUM):** Liegen zwei überlappende aktive `EmployeeWorkDetails`-Rows in derselben ISO-Woche (Daten-Anomalie — Versionen sind normalerweise sequenziell, aber `find_working_hours_for_calendar_week` kann mehrere Rows liefern), wird `committed_voluntary` über **SUM** aggregiert — `find_working_hours_for_calendar_week(...).map(|wh| wh.committed_voluntary).sum()`. Begründung: konsistent mit dem `expected_hours`-Präzedenzfall (`reporting.rs:240-254`, gleicher `.fold(acc + a)`-Pfad, **dieselbe** Selektion), folgt dem line-für-line-Prinzip der ganzen Phase, minimaler Sonderfall-Code. Das Boolean-`.any()`-Pattern des Cap-Flags (`reporting.rs:264-265`) generalisiert **nicht** auf einen numerischen Wert und ist hier explizit NICHT zu kopieren.
- **D-OVERLAP-AGG-TEST:** Die SUM-Semantik wird in Phase 14 per Unit-Test gepinnt (zwei überlappende Rows in derselben ISO-Woche, z.B. 5h + 5h → 10h), auch wenn noch **kein** Produktions-Read-Site existiert (Feld inert). Der eigentliche Read-/Aggregations-Pfad landet in Phase 15.

### Field-Threading (auf Milestone/ROADMAP-Ebene gelockt — hier zur Klarheit zitiert)
- **D-01 / Variante B:** Nur die freiwillige Zusage obendrauf, entkoppelt von `expected_hours`. KEIN `committed >= expected`-Invariant (Variante A wurde verworfen).
- **Migration:** `ALTER TABLE … ADD COLUMN committed_voluntary REAL NOT NULL DEFAULT 0` — additiv via `sqlx migrate run` (**niemals** `sqlx database reset` — destruktiv, braucht User-Confirmation). `.sqlx`-Offline-Cache via `cargo sqlx prepare` regenerieren (erster harter Compile-Gate). Auf NixOS: `nix develop`.
- **Wire-Backward-Compat:** `EmployeeWorkDetailsTO` bekommt `#[serde(default)] committed_voluntary: f32` (kein `ToSchema`, passend zum umgebenden Struct).
- **Beide Konversionsrichtungen an jeder Boundary:** DAO `TryFrom`, Service↔DAO, TO↔Service — keine Omission-Lücke (eine fehlende Konversion = silent `0.0` oder Compile-Error).
- **Carry-Forward (CVC-02):** Beim Rotieren einer Vertrags-Version (Update/Rotate-Pfad, Struct-Update-Spread) wird `committed_voluntary` mitgeführt, nicht still auf Default zurückgesetzt — per Test verifiziert.
- **DI:** `EmployeeWorkDetailsService` bleibt **Basic Service** — das Feld ist Daten, keine Dependency; keine neue DI-Verdrahtung.

### Claude's Discretion
- Ob die SUM-Aggregation in Phase 14 bereits als wiederverwendbarer Accessor/Helper (z.B. `committed_voluntary_for_calendar_week`) eingeführt oder nur als getestete, dokumentierte Semantik ohne Produktions-Reader gepinnt wird — Planner-Entscheidung. SC#4 verlangt nur, dass die Semantik definiert + per Test gepinnt ist; der Produktions-Read-Site gehört zu Phase 15.
- Genaue Test-Datei-/Modul-Platzierung des Round-Trip- und Carry-Forward-Tests (Erweiterung des bestehenden `employee_work_details_update`-Integrationstests vs. neues Test-Modul).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone-Research (v1.4) — Pflichtlektüre
- `.planning/research/SUMMARY.md` — Executive Summary + Reuse-Map; die Zwei-Achsen-Erkenntnis (Achse A `reporting.rs` vs. Achse B `booking_information.rs`); Build-Order; Phase-A-Abgrenzung.
- `.planning/research/PITFALLS.md` — P3 (Forward-Default-Migration), P4 (Time-Version-Skew: Wert entkoppelt, Zeitfenster geteilt; D-OVERLAP-AGG), die enumerierte at-risk-Site-Liste (relevant ab Phase 15/17).
- `.planning/research/STACK.md` — Line-für-line-Reuse-Tabelle (wie `cap_planned_hours_to_expected` durch jeden Layer gefädelt wurde).
- `.planning/research/ARCHITECTURE.md` — Touch-Boundaries + Build-Order.

### Roadmap / Requirements
- `.planning/ROADMAP.md` § „Phase 14: Data-model foundation (backend)" — Goal, Success Criteria 1–4, Notes for plan-phase.
- `.planning/REQUIREMENTS.md` — CVC-01, CVC-02, CVC-03 (Wortlaut).

### Präzedenz-Code (line-für-line kopieren)
- `service_impl/src/reporting.rs:77-86` — `find_working_hours_for_calendar_week` (die Overlap-Selektion).
- `service_impl/src/reporting.rs:240-265` — `expected_hours`-SUM-`.fold`-Präzedenz (D-OVERLAP-AGG-Vorlage) vs. `cap_planned_hours_to_expected`-`.any()` (NICHT generalisierbar).
- `service/src/employee_work_details.rs` — Service-Struct + Konversionen (`cap_planned_hours_to_expected`-Threading als Vorlage, Z.26/56/207).
- `service_impl/src/employee_work_details.rs:248` — Update/Rotate-Struct-Spread (Carry-Forward-Vorlage).
- Sekundäre Präzedenz: `is_dynamic` (analoges Feld-Threading vor v1.3).

### Projekt-Regeln
- `CLAUDE.md` § „Billing Period Snapshot Schema Versioning" — relevant für Phase 15, hier nur Kontext: in Phase 14 **kein** Bump (Feld inert, kein persistierter `value_type` ändert sich).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `cap_planned_hours_to_expected` (v1.3) + `is_dynamic`: zwei exakte Präzedenzfälle für genau diesen Feld-Add auf genau dieser Entity — Migration, DAO-Row/Entity, Service-Struct, DTO, beide Konversionen, alle SELECT/INSERT/UPDATE. Line-für-line kopieren.
- `find_working_hours_for_calendar_week` (`reporting.rs:77`): liefert die Iterator-Selektion über alle in der Woche aktiven Rows — die Basis für die SUM-Aggregation (D-OVERLAP-AGG).
- Bestehender `employee_work_details_update`-Integrationstest: Erweiterungs-Anker für den fraktionalen Open→Save→Reload-Round-Trip.

### Established Patterns
- `EmployeeWorkDetails` ist zeit-versioniert (from/to ISO-Woche). `committed_voluntary` reitet auf **derselben** Row → erbt das Zeitfenster gratis; nur der **Wert** ist entkoppelt, nicht das Zeitfenster (PITFALLS P4).
- `.sqlx`-Offline-Cache ist compile-time-checked → erster Compile-Gate nach Migration + `cargo sqlx prepare`.
- Diese Endpoint-Familie ist serde-transparent ohne `#[utoipa::path]`/`ToSchema` → kein OpenAPI-Task.

### Integration Points
- Migration (`migrations/sqlite`) → DAO (`dao` + `dao_impl_sqlite`) → Service (`service/src/employee_work_details.rs` + `service_impl/src/employee_work_details.rs`) → `rest-types` (`EmployeeWorkDetailsTO`). REST-Handler: unverändert.

</code_context>

<specifics>
## Specific Ideas

- Aggregations-Vorlage exakt: `find_working_hours_for_calendar_week(...).map(|wh| wh.committed_voluntary).sum()` — gleiche Selektion + gleicher Reduktions-Stil wie der `expected_hours`-`.fold`.

</specifics>

<deferred>
## Deferred Ideas

- Reporting-Integration (Achse B, `booking_information.rs::get_weekly_summary`, per-Woche `max(committed, actual)`) + Snapshot-Bump 7→8 → **Phase 15**.
- Jahresansicht-Display (dritter „zugesagt"-Token, Überschuss-Anzeige, i18n) → **Phase 16**.
- Vertrags-Editor-Input + „alle"-Filter + unpaid-volunteer-Record (`is_paid`-Gating aller at-risk-Sites) → **Phase 17**.
- Inline-Banner „Zusage nicht erfüllt" + eigenes committed-Band im Chart → **v1.5** (CVC-F-01 / CVC-F-02).

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 14-data-model-foundation-backend*
*Context gathered: 2026-06-22*
