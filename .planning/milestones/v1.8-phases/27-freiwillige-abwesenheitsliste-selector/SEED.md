# Phase 27 — Seed: Freiwillige in Abwesenheitsliste auswählbar (FE)

**Milestone**: v1.8 · **Requirement**: VOL-SEL-01 · **Typ**: reines Frontend
**Erstellt**: 2026-06-29

## Problem / Motivation

Auf der Abwesenheitsseite können aktuell nur **Angestellte** (`is_paid == true`)
ausgewählt werden. Der Betrieb braucht aber auch für **Freiwillige**
(`is_paid == false`) die Möglichkeit, Abwesenheiten zu erfassen und zu filtern.

„Freiwillige" sind in Shifty **keine eigene Entität** — es sind Sales Persons mit
`is_paid == false`. (Vgl. v1.4 Phase 17: unbezahlte Personen halten seit dem
`EmployeeWorkDetails`-Records als „rein freiwillige Helfer".)

## Status quo (verifiziert 2026-06-29)

Zentrale Filterstelle (eine Funktion, zwei Call-Sites):

```rust
// shifty-dioxus/src/page/absences.rs:115
pub fn is_selectable_employee(sales_person: &SalesPerson) -> bool {
    sales_person.is_paid && !sales_person.inactive
}
```

Verwendet in:
- **AbsenceModal** Personen-Dropdown — `absences.rs:~1217`
  (`for sp in props.sales_persons.iter().filter(|sp| is_selectable_employee(sp)) { option { … } }`)
- **AbsenceFilterBar** Personenfilter (HR-only) — `absences.rs:~1365`

`SelectInput` (`shifty-dioxus/src/component/form/inputs.rs:83`) rendert
`select { … { props.children } }` — d.h. die Aufrufer übergeben `option`-Children
direkt. **Native `optgroup` ist damit ohne Komponenten-Änderung möglich.**

Sales-Person-Modell:
- `rest-types/src/lib.rs` `SalesPersonTO`: `is_paid: Option<bool>`, `inactive: bool`
- Frontend-State `SalesPerson`: `is_paid: bool`, `inactive: bool`
- Backend hat `get_all_paid()` (nur `is_paid==true`) — Vorbild-Pattern aus Paid-Capacity.

## Entschiedene Konzept-Eckpunkte (User, 2026-06-29)

1. **Selector-UX = gruppierter Dropdown**: zwei `optgroup`s
   („Angestellte" zuerst, dann „Freiwillige"). Kein Toggle, kein Typ-Filter.
2. **Geltungsbereich = Modal UND Filter** (beide Call-Sites).
3. **Gemeinsamer Helfer** für beide Call-Sites (kein Copy-Paste), z.B.
   `grouped_employee_options(sales_persons, selected_id, i18n) -> Element`.
4. **Predicate lockern**: `is_selectable_employee` von `is_paid && !inactive`
   auf `!inactive`; `is_paid` wandert in die Gruppierung.
   → **Achtung**: prüfen, ob `is_selectable_employee` noch anderswo genutzt wird,
     wo die Lockerung NICHT erwünscht ist (dann dort separat halten).
5. **Backend**: keine Änderung (VFA Phase 26 + EmployeeWorkDetails Phase 17
   decken Freiwilligen-Abwesenheiten ab).

## Akzeptanzkriterien (siehe ROADMAP.md Phase 27)

- Freiwillige erscheinen in eigener, beschrifteter Gruppe in Modal + Filter.
- Inaktive bleiben in beiden Selektoren ausgeblendet (beide Gruppen).
- „Alle"-Option im Filter bleibt erhalten.
- 2 neue i18n-Keys `AbsenceGroupEmployees` / `AbsenceGroupVolunteers` in de/en/cs.
- Leere Gruppe wird nicht gerendert (kein leeres `optgroup`).

## Offener Punkt für die Planung

Welche Abwesenheits-**Kategorien** (Vacation / SickLeave / UnpaidLeave) sind für
Freiwillige sinnvoll? Betrifft nur das Kategorie-Dropdown im Modal
(`absences.rs:1229`), nicht den Personen-Selector. Default-Vorschlag: alle
Kategorien gleich anbieten, sofern keine fachliche Einschränkung genannt wird.

## Gates (Definition of Done)

- `cargo build --target wasm32-unknown-unknown` (aus `shifty-dioxus/`, via Backend-Shell wegen lld)
- `cargo test` (FE) inkl. neuer SSR-/Helper-Tests für die Gruppierung
- `cargo clippy --workspace -- -D warnings` (aus Backend-Shell — FE-Clippy nicht CI-gegated, trotzdem prüfen)
- Browser-Smoke: Freiwilliger im Modal + Filter sichtbar/gruppiert (Backend-Roundtrip).
