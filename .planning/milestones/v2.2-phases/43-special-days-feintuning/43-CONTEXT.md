# Phase 43: Special-Days-Feintuning (FE + BE) - Context

**Gathered:** 2026-07-02
**Status:** Ready for planning
**Mode:** Autonomous — Cleanup/Bugfix-Phase, Grey Areas trivial (Success Criteria präzise), Claude's Discretion für Umsetzungsdetails.

<domain>
## Phase Boundary

Drei präzise Bug-/UX-Korrekturen am Special-Days-Feature (Fortsetzung SDF-01/02 aus v1.11):

- **SDF-03**: Special-Days-Loader in `settings.rs` lädt nach **Kalenderjahr** (`date.year()`), nicht nach ISO-Wochenjahr — behebt 1.1.-Anzeige-Bug.
- **SDF-04**: i18n-Copy des „already exists"-Hinweises in de/en/cs an Replace-Verhalten anpassen (kein „blockiert"-Text mehr, weil SDF-01 v1.11 auf in-place-Replace umgestellt hat).
- **SDF-05**: Schichtplan-Wochenraster-Dropdown Feiertag↔Kurzer-Tag umschalten produziert keine 422/UI-Fehlermeldung mehr — analog SDF-01 atomarer in-place Replace, nicht create.

Kein Snapshot-Bump, keine Migration, keine neuen Deps.

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion
Alle Umsetzungsdetails an Claude — Success Criteria in ROADMAP.md sind hart und präzise. Insbesondere:

- **SDF-03**: `sd_year.set(iso_year)` → `sd_year.set(date.year() as u32)` in `page/settings.rs` (Zeile ~635 im create-Handler und ~635-Umgebung im Date-Picker-onchange). Regressionstest per pure-fn: Datumsstich am Jahreswechsel (z.B. 2026-01-01 = ISO-Woche 1 in Jahr 2025) → `sd_year == 2026`.
- **SDF-04**: i18n-Keys für Special-Days-Duplikat-Hinweis suchen (`grep -rn "already exists\|existiert bereits\|již existuje"` in `shifty-dioxus/i18n/` oder `strings.rs`). Copy neutralisieren: "wird ersetzt" / "will be replaced" / "bude nahrazen" o.ä. i18n-Presence-Test grün.
- **SDF-05**: Schichtplan-Dropdown greift nicht die REST-Route für Special-Days-Create, sondern schaltet auf den in-place-Replace-Pfad, den SDF-01/Phase-42 etabliert hat. HTTP 201 statt 422, keine UI-Fehlermeldung.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `shifty-dioxus/src/page/settings.rs`: `sd_year` Signal, `parse_date_to_iso_parts()` helper, `is_duplicate_special_day()` prüft on-the-fly, `special_day_form_after_create()` reset-Verhalten.
- `SDF-01` (Phase 42/36): atomarer in-place-Replace für Special-Days via `create_or_replace_special_day` (grep im Service-Impl bestätigt Pattern).
- i18n-Ressourcen: `shifty-dioxus/src/i18n/` (de/en/cs) oder `strings.rs` — Presence-Test pattern in v1.11-HYG-Phasen etabliert.

### Established Patterns
- Pure-fn Unit Tests für FE-Logik in `#[cfg(test)] mod tests` innerhalb der jeweiligen `page/*.rs` (siehe `special_day_iso_date_round_trip` etc.).
- Backend-Test-Muster: `service_impl/src/test/` — Integration tests mit in-memory SQLite.

### Integration Points
- **FE:** `shifty-dioxus/src/page/settings.rs` (Settings-Card Special-Days) + Schichtplan-Wochenraster (`page/shiftplan.rs` oder `component/shiftplan_*`).
- **BE:** `service_impl/src/special_day.rs` + `dao_impl_sqlite/src/special_day.rs` (bereits vorhanden aus v1.10/v1.11).
- **REST:** vermutlich keine neuen Routen — Reuse `PUT /special-days/{id}` oder in-place-Replace via `create_or_replace`.

</code_context>

<specifics>
## Specific Ideas

- SDF-04 Copy: neutrale, replace-taugliche Wording — nicht „blockiert" / „already exists", sondern etwa „wird ersetzt" / „will be replaced" / „bude nahrazen". Exakte Wording an Claude — de-Referenz gewinnt (analog IMP-05 Präzedenz).
- SDF-05: Regressionstest muss den Umschalt-Roundtrip (Holiday → ShortDay → Holiday) auf einem existierenden Special-Day-Slot durchspielen und HTTP 201/Erfolg + korrekten `day_type` in DB verifizieren.

</specifics>

<deferred>
## Deferred Ideas

Nichts — Scope-treu innerhalb Phase 43.

</deferred>
