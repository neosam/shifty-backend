# Phase 53: freiwilligen-abwesenheiten-jahresansicht — Research

**Recherchiert:** 2026-07-06
**Domain:** Rust Backend Service-Erweiterung + Dioxus-FE-Mapper — Neues DTO-Feld mit Union-Merge
**Konfidenz:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-53-01 (G1-a):** Neues Feld `sales_person_absences: Arc<[SalesPersonAbsence]>` auf
  `service::booking_information::WeeklySummary` UND `rest_types::WeeklySummaryTO`.
  Struct-Neuling in **beiden** Ebenen:
  ```rust
  // service/src/booking_information.rs
  #[derive(Clone, Debug, PartialEq)]
  pub struct SalesPersonAbsence {
      pub sales_person_id: Uuid,
      pub name: Arc<str>,
      pub hours: f32,
  }
  // rest-types/src/lib.rs
  #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
  pub struct SalesPersonAbsenceTO {
      pub sales_person_id: Uuid,
      pub name: Arc<str>,
      pub hours: f32,
  }
  ```
- **D-53-02 (G2-a):** Stunden-Wert = Σ über aktive EmployeeWorkDetails-Rows von Person p für
  (year, week) wo `wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0`, von
  `wh.committed_voluntary` — identische Formel zu `booking_information.rs:495–503`, nur
  **ohne** `!absent_volunteer_ids.contains(...)` (hier sind gerade die Abwesenden gemeint).
- **D-53-03 (G3-a):** Sichtbarkeit via `absent_volunteer_ids` — bereits im Code
  (`booking_information.rs:421–444`), Whole-Week-Out-Kriterium (VFA-01, D-26-01).
- **D-53-04 (G4-a):** Backend liefert unsortiert. FE baut Union-Liste aus bezahlten Absencen
  (bestehender Merge) + Freiwilligen aus neuem DTO-Feld; sortiert nach `name.to_lowercase()`.
  Kein visueller Unterschied. Filter `>= 0.1` gilt für Freiwillige (Randfall Zusage=0
  fällt raus).
- **D-53-05 (G5-a):** Fill-Site ist `get_weekly_summary` (Assembly-Loop, Zeile 267+), NICHT
  `assemble_weeks` (D-52-09 MUST-preserve bleibt unberührt).
- **D-53-06 (G6-a):** `get_summery_for_week` (Zeile 643+) füllt das Feld analog; dort muss
  `absent_volunteer_ids` erst noch aufgebaut werden. In-Line für Wave 1; optionaler
  Helper-Extract als Cleanup.

### Claude's Discretion

Keine — alle 6 Gray Areas wurden in discuss-phase fixiert.

### Deferred Ideas (OUT OF SCOPE)

- FE-Filter-Threshold `>= 0.1` als benannte Konstante heben.
- `absent_volunteer_ids_for_week`-Helper-Extraktion (D-53-06) — optional.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Beschreibung | Research-Support |
|----|-------------|------------------|
| VAA-01 | Freiwillige mit aktiver Vacation/SickLeave/UnpaidLeave-Period erscheinen in `sales_person_absences` der Jahresansicht; Backend liefert Name + Stunden fertig im DTO | D-53-01/05/06: DTO-Feld-Design, Fill-Sites vollständig identifiziert |
| VAA-02 | Stunden-Wert = `committed_voluntary` cap-gated (D-53-02-Formel) | Formel verifiziert gegen Code-Zeilen 495–503; exakte Ableitung in §Präzedenzformel |
| VAA-03 | Backend-Test: Freiwilliger mit Period erscheint / ohne Period nicht / Bezahlter unverändert | Test-Skelett in §Validation Architecture; Referenz-Testmodul `booking_information_vfa.rs` analysiert |
| VAA-04 | FE rendert Freiwillige visuell konsistent mit Bezahlten (keine Farbe/Icon/Suffix); bestehende Rendering-Zeile bleibt unverändert | FE-Init-Sites kartiert; page/weekly_overview.rs:121–130 bleibt literal unberührt |
</phase_requirements>

---

## Summary

Phase 53 ergänzt `WeeklySummary` / `WeeklySummaryTO` um ein neues Feld `sales_person_absences:
Arc<[SalesPersonAbsence]>`, das Freiwillige mit aktiver Absence-Period (Vacation/SickLeave/
UnpaidLeave) für die jeweilige Woche enthält — inklusive des `committed_voluntary`-Stundenwertes,
der durch das Whole-Week-Out (VFA-01) aus Band 1 herausgerechnet wird. Die Berechnung ist eine
direkte Ableitung der bereits vorhandenen `committed_voluntary_hours`-Formel (Zeilen 495–503),
nur ohne den Absent-Filter: dort wird der Wert für Abwesende auf 0 gesetzt, hier wird er
explizit gemessen.

Die Implementierung hat zwei Backend-Fill-Sites (`get_weekly_summary` und `get_summery_for_week`)
und eine FE-Umbauung: der bestehende `state::WeeklySummary::from()`-Mapper erweitert seinen
bezahlten-Merge-Loop um eine zweite Iteration über das neue DTO-Feld und sortiert die Union
nach Name. Die Rendering-Zeile in `page/weekly_overview.rs:121` bleibt buchstäblich unverändert
(VAA-04).

Die Phase ist rein additiv: keine Migration, kein Snapshot-Schema-Bump, kein neuer Cargo-Dep,
kein `assemble_weeks`-Touch. Alle benötigten Inputs (`absent_volunteer_ids`,
`all_work_details`, `sales_person_service.get_all()`) sind in `get_weekly_summary` bereits im
Loop verfügbar. Für `get_summery_for_week` muss `absent_volunteer_ids` inline aufgebaut werden
(~15–20 LOC).

**Primäre Empfehlung:** Wave-1 = Struct-Definitionen + `get_weekly_summary`-Fill-Site +
DTO-Mapping. Wave 2 = `get_summery_for_week` + FE-Mapper-Union. Wave 3 = Backend-Test VAA-03.
Kein Redesign, kein Risiko.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Freiwilligen-Absencen berechnen (Stunden-Wert) | API / Backend | — | Fat Backend, Thin Client (D-51-02). Formel identisch zu bestehender `committed_voluntary_hours`-Berechnung — Backend kennt `EmployeeWorkDetails`, FE nicht. |
| `absent_volunteer_ids` bauen (Sichtbarkeit) | API / Backend | — | VFA-01-Logik lebt bereits im Backend (`period_overlaps_week`-Helper, `all_absences`-Load). |
| DTO-Serialisierung | API / Backend | — | `From<&WeeklySummary> for WeeklySummaryTO` in rest-types; keine Logik im REST-Handler. |
| FE-Union-Merge + Sort | Browser / Client | — | Presentation-Layer-Entscheidung (Sortierordnung per D-53-04). FE baut Anzeige-Vec aus `working_hours_per_sales_person` (bezahlt) + neuem DTO-Feld (Freiwillige). |
| FE-Rendering | Browser / Client | — | `page/weekly_overview.rs:121` — bestehende Zeile, kein Touch (VAA-04). |

---

## Standard Stack

Kein neuer Cargo-Dep — alle Werkzeuge bereits im Workspace vorhanden.

### Core (bereits in workspace)

| Crate | Version | Zweck | Relevanz |
|-------|---------|-------|----------|
| `service` (eigener Crate) | — | Trait + Domain-Structs | Neuer `SalesPersonAbsence`-Struct hier |
| `service_impl` (eigener Crate) | — | Fill-Sites | `get_weekly_summary` + `get_summery_for_week` |
| `rest-types` (eigener Crate) | — | DTO-Layer | Neuer `SalesPersonAbsenceTO`-Struct + `WeeklySummaryTO`-Feld |
| `uuid` | 1.x | `Uuid` in Struct | Bereits Dep |
| `serde` | 1.x | Serialize/Deserialize auf TO | Bereits Dep |
| `mockall` | 0.x | Mock-Setup in Backend-Tests | Bereits Dep in service_impl |

**Installation:** Keine — alle Deps sind bereits vorhanden.

---

## Package Legitimacy Audit

> Keine neuen Packages in Phase 53. Kein Cargo-Dep-Add.

**Packages removed due to SLOP verdict:** none
**Packages flagged as suspicious:** none

---

## Architecture Patterns

### System Architecture Diagram

```
FE-Request (LoadYear(year))
        │
        ▼
GET /booking-information/weekly-resource-report/{year}
        │
        ▼
BookingInformationServiceImpl::get_weekly_summary(year)
        │
        ├── [Load-once, vor dem Loop]
        │    ├── sales_person_service.get_all() → volunteer_ids + (NEU) name_map
        │    ├── employee_work_details_service.all() → all_work_details
        │    ├── absence_service.find_all() → all_absences
        │    ├── reporting_service.get_year(year) → year_reports
        │    └── [weitere Phase-52-Bulk-Loads]
        │
        ├── [Per-Woche-Loop: week 1..=55]
        │    ├── absent_volunteer_ids = filter(all_absences, period_overlaps_week) ← VFA-01 vorhanden
        │    │
        │    ├── [NEU: VAA-01/02] sales_person_absences bauen
        │    │    └── für jede id in absent_volunteer_ids:
        │    │         name = name_map[id]
        │    │         hours = Σ find_working_hours(all_work_details, year, week)
        │    │                   .filter(wh.cap || wh.expected==0)
        │    │                   .map(wh.committed_voluntary)  ← Formel D-53-02
        │    │
        │    └── WeeklySummary { ..., sales_person_absences: collected }
        │
        ▼
WeeklySummaryTO::from(&WeeklySummary)   ← Feld-Mapping + From<&SalesPersonAbsence> for SalesPersonAbsenceTO
        │
        ▼
JSON-Response → FE
        │
        ▼
state::WeeklySummary::from(&WeeklySummaryTO)
        │
        ├── bestehender Merge-Loop: working_hours_per_sales_person → bezahlte Absencen (unverändert)
        │
        ├── [NEU] Iteration über sales_person_absences DTO-Feld
        │    └── filter(hours >= 0.1)  →  SalesPersonAbsence { name, absence_hours: hours }
        │
        ├── Union beider Vecs → sort_by(name.to_lowercase())
        │
        ▼
page/weekly_overview.rs:121 → "{name}: {hours} h" (unverändert, VAA-04)
```

### Recommended Project Structure

Keine Strukturänderung — alle Dateien existieren bereits:

```
service/src/booking_information.rs        ← SalesPersonAbsence-Struct + WeeklySummary-Feld (Wave 1)
service_impl/src/booking_information.rs   ← Fill-Sites get_weekly_summary + get_summery_for_week (Wave 1/2)
rest-types/src/lib.rs                     ← SalesPersonAbsenceTO + WeeklySummaryTO-Feld + From-Impls (Wave 1)
shifty-dioxus/src/state/weekly_overview.rs ← FE-Mapper Union-Refactor (Wave 2)
service_impl/src/test/booking_information_vaa.rs  ← Neues Testmodul VAA-03 (Wave 3)
service_impl/src/test/mod.rs              ← pub mod booking_information_vaa (Wave 3)
```

---

## Präzedenzformel-Verifikation (D-53-02) [VERIFIED: Codebase]

### Bestehende Formel in `get_weekly_summary` (Zeilen 509–517)

```rust
// booking_information.rs:509–517 — committed_voluntary_hours (Band 1, bestehend)
let committed_voluntary_hours: f32 = find_working_hours_for_calendar_week(
    &all_work_details,
    year,
    week,
)
.filter(|wh| wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0)
.filter(|wh| !absent_volunteer_ids.contains(&wh.sales_person_id)) // ← VFA-01: Abwesende raus
.map(|wh| wh.committed_voluntary)
.sum();
```

### Neue Formel für VAA-02 (Freiwilligen-Absencen) — Ableitung Stelle für Stelle

```rust
// NEU — VAA-02 fill für sales_person_absences:
//
// Schritt 1: alle_sales_persons wurde bereits über sales_person_service.get_all() geladen.
// In get_weekly_summary müssen wir eine name_map aufbauen (id → name + is_paid).
// Die Loop-Variable `volunteer_ids` enthält nur IDs; wir brauchen zusätzlich die Namen.
//
// Empfehlung: sales_person_service.get_all() liefert Arc<[SalesPerson]>; davon
// volunteer_ids bereits abgeleitet (Zeile 292–297). Den vollen Vec einmalig halten
// und im Loop per id nachschlagen.

let sales_person_absences: Arc<[SalesPersonAbsence]> = absent_volunteer_ids
    .iter()
    .filter_map(|&sp_id| {
        let name = all_sales_persons  // einmalig vor Loop geladen
            .iter()
            .find(|sp| sp.id == sp_id)
            .map(|sp| sp.name.clone())?;
        // D-53-02: identische Cap-Gate-Filterung wie committed_voluntary_hours,
        // aber OHNE absent-Filter — hier sind gerade die Abwesenden gesucht
        let hours: f32 = find_working_hours_for_calendar_week(&all_work_details, year, week)
            .filter(|wh| {
                wh.sales_person_id == sp_id
                    && (wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0)
            })
            .map(|wh| wh.committed_voluntary)
            .sum();
        Some(SalesPersonAbsence { sales_person_id: sp_id, name, hours })
    })
    .collect();
```

**Wichtig:** `find_working_hours_for_calendar_week` ist ein `Iterator` aus `crate::reporting` —
bereits importiert in `booking_information.rs` (Zeile 2). Kein neuer Import.

**Randfall Zusage=0:** Freiwilliger mit `hours = 0.0` wird ins Backend-Feld geschrieben. Der
FE-Filter `>= 0.1` (state/weekly_overview.rs:53) filtert ihn auf FE-Seite raus. Backend liefert
die Zeile bewusst (Fat Backend, D-53-04).

---

## `get_weekly_summary` — Input-Anpassung für `all_sales_persons` [VERIFIED: Codebase]

**Problem:** `get_weekly_summary` baut heute `volunteer_ids: Arc<[Uuid]>` (Zeilen 292–297) aus
`sales_person_service.get_all()` — aber die SalesPerson-Namen werden nicht gehalten.

**Lösung:** Statt eines separaten zweiten `get_all()`-Calls (teuer): Gleichzeitig beim ersten
`get_all()`-Call sowohl `volunteer_ids` als auch `all_sales_persons` aus dem Ergebnis ableiten.

```rust
// Vorher (Zeilen 292–297):
let volunteer_ids: Arc<[Uuid]> = self
    .sales_person_service
    .get_all(Authentication::Full, tx.clone().into())
    .await?
    .iter()
    .filter(|sp| !sp.is_paid.unwrap_or(false))
    .map(|sp| sp.id)
    .collect();

// Nachher — einmal laden, zweimal nutzen:
let all_sales_persons = self
    .sales_person_service
    .get_all(Authentication::Full, tx.clone().into())
    .await?;
let volunteer_ids: Arc<[Uuid]> = all_sales_persons
    .iter()
    .filter(|sp| !sp.is_paid.unwrap_or(false))
    .map(|sp| sp.id)
    .collect();
// all_sales_persons wird im Loop für name-Lookup verwendet (kein extra get_all())
```

**Vorteil:** Kein zusätzlicher Service-Call. `all_sales_persons` ist `Arc<[SalesPerson]>` —
Clone im Loop ist billig (Arc-Clone).

---

## `get_summery_for_week` — absent_volunteer_ids aufbauen (D-53-06) [VERIFIED: Codebase]

`get_summery_for_week` (Zeile 643+) hat heute **kein** `absent_volunteer_ids`. Die Methode
lädt `volunteer_ids` (Zeilen 666–673) und `all_work_details` (implizit über `employee_work_details_service`,
Zeile 772 — bei per-day-hours-Berechnung). VFA-01-Logik fehlt komplett.

### Was die Single-Week-Methode zusätzlich braucht

1. `all_absences` laden — einmalig für diesen Call:
   ```rust
   let all_absences = self
       .absence_service
       .find_all(Authentication::Full, tx.clone().into())
       .await?;
   ```

2. `absent_volunteer_ids` bauen — identisch zu `get_weekly_summary:422–442`:
   ```rust
   let absent_volunteer_ids: std::collections::HashSet<Uuid> =
       if let (Ok(week_monday), Ok(week_sunday)) = (
           time::Date::from_iso_week_date(year as i32, week, time::Weekday::Monday),
           time::Date::from_iso_week_date(year as i32, week, time::Weekday::Sunday),
       ) {
           all_absences
               .iter()
               .filter(|period| {
                   volunteer_ids.contains(&period.sales_person_id)
                       && period_overlaps_week(
                           period.from_date,
                           period.to_date,
                           week_monday,
                           week_sunday,
                       )
               })
               .map(|period| period.sales_person_id)
               .collect()
       } else {
           std::collections::HashSet::new()
       };
   ```

3. `all_sales_persons` laden — wie in `get_weekly_summary` für name-Lookup.

**LoC-Schätzung:** ~20–25 LOC (3 Blöcke: `all_absences`-Load + `absent_volunteer_ids`-Bau +
`sales_person_absences`-Fill), analog zu `get_weekly_summary`.

**Helper-Extraktion:** Optional. Wenn Planner einen `absent_volunteer_ids_for_week`-Helper
extrahieren will: Die Funktion nimmt `(all_absences: &[AbsencePeriod], volunteer_ids: &[Uuid],
year: u32, week: u8) -> HashSet<Uuid>` und kapselt den Blöcke 416–442 aus `get_weekly_summary`.
Beide Fill-Sites rufen ihn auf. Duplikation ist ~15 LOC — vertretbar in-line, aber sauber als
Helper.

---

## FE-Union-Merge in `state/weekly_overview.rs` [VERIFIED: Codebase]

### Bestehender Merge-Loop (Zeilen 47–62)

```rust
// state/weekly_overview.rs:47–62 (heute)
sales_person_absences: summary
    .working_hours_per_sales_person
    .iter()
    .filter_map(|sp| {
        let effective_absence =
            sp.absence_hours - sp.holiday_hours + sp.unavailable_hours;
        if effective_absence >= 0.1 {
            Some(SalesPersonAbsence {
                name: sp.sales_person_name.clone(),
                absence_hours: effective_absence,
            })
        } else {
            None
        }
    })
    .collect(),
```

### Neuer Union-Merge (D-53-04)

```rust
// NEU: Union aus bezahlten Absencen + Freiwilligen-Absencen aus DTO-Feld
let mut all_absences: Vec<SalesPersonAbsence> = {
    // Bezahlte (bestehende Logik, unverändert für Regression-Lock VAA-03 #3)
    let mut v: Vec<SalesPersonAbsence> = summary
        .working_hours_per_sales_person
        .iter()
        .filter_map(|sp| {
            let effective_absence =
                sp.absence_hours - sp.holiday_hours + sp.unavailable_hours;
            if effective_absence >= 0.1 {
                Some(SalesPersonAbsence {
                    name: sp.sales_person_name.clone(),
                    absence_hours: effective_absence,
                })
            } else {
                None
            }
        })
        .collect();
    // Freiwillige aus neuem DTO-Feld (D-53-01)
    v.extend(
        summary
            .sales_person_absences
            .iter()
            .filter(|a| a.hours >= 0.1)  // Randfall Zusage=0 raus (D-53-04)
            .map(|a| SalesPersonAbsence {
                name: a.name.clone(),
                absence_hours: a.hours,
            }),
    );
    // Sort case-insensitive nach Name (D-53-04)
    v.sort_by(|x, y| x.name.to_lowercase().cmp(&y.name.to_lowercase()));
    v
};
```

**Wichtig:** `SalesPersonAbsenceTO` im FE-Kontext hat kein `sales_person_id`-Feld sichtbar im
FE-State-Typ `SalesPersonAbsence` — der FE-Typ behält `{ name, absence_hours }` (zweifeldig
genug für Rendering). Das neue DTO-Feld `sales_person_id` ist im FE-Mapper nicht nötig;
es wird im `SalesPersonAbsenceTO` gehalten falls zukünftige Nutzung es braucht.

---

## FE-Type-Init-Sites vollständig [VERIFIED: Codebase]

`WeeklySummaryTO` bekommt ein neues Feld `sales_person_absences: Arc<[SalesPersonAbsenceTO]>`.
In Rust erfordern alle Stellen, die `WeeklySummaryTO` oder `state::WeeklySummary` direkt
konstruieren, ein `sales_person_absences`-Default.

| Datei | Zeile | Konstruktor | Aktion |
|-------|-------|-------------|--------|
| `shifty-dioxus/src/loader.rs` | 518–534 | `WeeklySummary { ... }` Fallback-Konstruktor | `sales_person_absences: Vec::new()` — **bereits vorhanden** (Zeile 533) |
| `shifty-dioxus/src/component/weekly_overview_chart.rs` | ~173–190 | `sample_week()` Test-Helper | `sales_person_absences: vec![]` — **bereits vorhanden** (Zeile 188) |
| `shifty-dioxus/src/page/weekly_overview.rs` | ~232–248 | `sample_week()` Test-Helper | `sales_person_absences: vec![]` — **bereits vorhanden** (Zeile 247) |
| `shifty-dioxus/src/page/weekly_overview.rs` | ~428–429 | Test `page_absences_row_uses_tokens_and_no_tint` | `sales_person_absences: vec![...]` — **bereits vorhanden** |
| `rest-types/src/lib.rs` | ~1009+ | `From<&WeeklySummary> for WeeklySummaryTO` | Neues Feld muss im From-Impl gemappt werden (wave 1 task) |
| `service_impl/src/booking_information.rs` | ~620–636 | `WeeklySummary { ... }` in get_weekly_summary | Neues Feld `sales_person_absences: sales_person_absences.into()` |
| `service_impl/src/booking_information.rs` | ~901–921 | `WeeklySummary { ... }` in get_summery_for_week | Neues Feld `sales_person_absences: sales_person_absences.into()` |

**FE-Fallback-Konstruktoren (loader.rs + Tests):** Die FE-Test-Helfer und der Fallback in
`loader.rs` konstruieren `state::WeeklySummary`, nicht `WeeklySummaryTO`. Das neue
`state::WeeklySummary`-Feld `sales_person_absences` ist schon vorhanden (`Vec<SalesPersonAbsence>`)
— der Typ ändert sich nicht, nur wie er befüllt wird. **Kein struct-literal breaking change für FE.**

Der einzige breaking-change für FE ist: `WeeklySummaryTO` bekommt ein neues Pflichtfeld. Da
`WeeklySummaryTO` im FE per serde-Deserialisierung befüllt wird (nicht direkt konstruiert),
braucht es ein `#[serde(default)]` auf dem neuen Feld — analog zu `committed_voluntary_hours`
(Zeile 997–998 in rest-types/src/lib.rs):
```rust
#[serde(default)]
pub sales_person_absences: Arc<[SalesPersonAbsenceTO]>,
```

---

## OpenAPI / utoipa Impact [VERIFIED: Codebase]

`WeeklySummaryTO` ist aktuell ohne `ToSchema` deriviert:
```rust
// rest-types/src/lib.rs:989 (aktuell)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeeklySummaryTO {
```

`WorkingHoursPerSalesPersonTO` hat `ToSchema` (Zeile 955). `WeeklySummaryTO` selbst nicht —
es wird nur als JSON serialisiert, nicht als OpenAPI-Schema-Objekt registriert.

**Konsequenz für Phase 53:** `SalesPersonAbsenceTO` braucht `ToSchema` (per D-53-01 bereits
vorgesehen). Da `WeeklySummaryTO` kein `ToSchema` hat, muss `SalesPersonAbsenceTO` nicht
explizit in `ApiDoc.components` registriert werden — es ist kein Dependency-Zyklus.

**ABER:** `WorkingHoursPerSalesPersonTO` hat `ToSchema` und ist in `WeeklySummaryTO` eingebettet.
Das Pattern beibehalten: `SalesPersonAbsenceTO` kriegt `ToSchema` per D-53-01-Spec.

**Kein `#[utoipa::path]`-Change nötig:** `booking_information.rs` REST-Handler hat keine
utoipa-Annotationen (keine `pub struct BookingInformationApiDoc` vorhanden). Kein Eintrag in
einer zentralen ApiDoc-Struct für WeeklySummary. Kein Handlungsbedarf.

---

## Docs-Freshness-Trigger-Check [VERIFIED: Codebase]

CLAUDE.md definiert folgende Trigger-Tabelle:

| Geänderte Datei | Trigger? | Docs-Datei |
|-----------------|----------|------------|
| `rest-types/src/lib.rs` | Kein expliziter Trigger in der Tabelle für reine DTO-Erweiterungen | — |
| `service/src/booking_information.rs` | Kein Trigger in Tabelle (nicht Trait-Signaturen-Änderung im engeren Sinne — Struct-Erweiterung ist additiv) | Prüfen ob F07 VAA erwähnen soll |
| `service_impl/src/booking_information.rs` | Kein `migrations/`-Touch, kein `permission.rs`-Touch, kein Schema-Bump | — |
| `shifty-dioxus/src/state/weekly_overview.rs` | Frontend — kein CLAUDE.md Backend-Trigger | — |

**Harte Trigger (CLAUDE.md) — KEINER ausgelöst:**
- `migrations/sqlite/*.sql` → nicht berührt (keine Migration)
- `service_impl/src/permission.rs` → nicht berührt
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` → bleibt 12, nicht berührt
- `service_impl/src/reporting.rs` (Balance-Formel) → nicht berührt

**Weiche Empfehlung (CONTEXT.md Kanonische Referenz):** `docs/features/F07-reporting-balance.md`
(+ `_de.md`) — dort wird `WeeklySummary` und die Jahresansicht erwähnt (Zeilen 560/558 in EN/DE).
Ein kurzer additiver Nebensatz: "Freiwillige mit aktiver Absence-Period erscheinen ab Phase 53
als `sales_person_absences` im `WeeklySummaryTO`" — reicht semantisch. Kein Redesign der Seite.
Beide Sprachversionen synchron updaten (CLAUDE.md-Regel: beide Sprachen müssen synchron sein).

**Planner-Entscheidung:** F07-Update als optionaler Wave-4-Task oder Post-Phase-Cleanup.
Streng genommen kein hartes Gate, aber GSD-Milestone-Close prüft Docs-Freshness.

---

## Common Pitfalls

### Pitfall 1: Doppeltes `sales_person_service.get_all()` in `get_weekly_summary`

**Was schiefgeht:** Ein zweites `get_all()`-Call für namen-Lookup einbauen, statt den
bestehenden Load zu nutzen.
**Warum:** `get_weekly_summary` lädt `volunteer_ids` bereits per `get_all()` (Zeile 292).
**Wie vermeiden:** Beim ersten Call `all_sales_persons` halten (Arc-Clone); `volunteer_ids`
davon ableiten. Kein zweiter Service-Call.

### Pitfall 2: `sales_person_id`-Filter fehlt in der VAA-02-Formel

**Was schiefgeht:** `find_working_hours_for_calendar_week(&all_work_details, year, week)`
gibt Rows ALLER Personen zurück — ohne `.filter(|wh| wh.sales_person_id == sp_id)` würde
die Summe über alle Freiwilligen statt nur eine Person gehen.
**Warum:** `find_working_hours_for_calendar_week` filtert nur nach Zeitraum, nicht nach Person.
**Wie vermeiden:** Immer `wh.sales_person_id == sp_id` als ersten Filter in der Kette.

### Pitfall 3: `#[serde(default)]` fehlt auf neuem `WeeklySummaryTO`-Feld

**Was schiefgeht:** Ein Backend-Client mit altem JSON (ohne `sales_person_absences`-Feld)
kann nicht deserialisieren → Panic oder Error.
**Warum:** Serde-Strict-Mode ohne Default = Deser-Fehler.
**Wie vermeiden:** `#[serde(default)]` auf `sales_person_absences` setzen (analog zu
`committed_voluntary_hours`, Zeile 997).

### Pitfall 4: `get_summery_for_week` ohne `absent_volunteer_ids` — DTO-Semantik-Inkonsistenz

**Was schiefgeht:** `get_summery_for_week` liefert leeres `sales_person_absences`-Feld, obwohl
Freiwillige abwesend sind — stille semantische Inkonsistenz zwischen Wochen-View und Jahres-View.
**Warum:** VFA-01 wurde für die Jahresansicht eingebaut; Wochen-Methode lief separat.
**Wie vermeiden:** D-53-06 explizit umsetzen — `all_absences` laden + `absent_volunteer_ids`
inline aufbauen (s. §get_summery_for_week-Sektion).

### Pitfall 5: FE-Init-Sites nicht updated (breaking change bei Struct-Erweiterung)

**Was schiefgeht:** `state::WeeklySummary`-Struct bekommt neues Feld → alle Literal-Konstruktoren
in FE brechen (Rust compile error).
**Warum:** Rust: struct literal muss alle Felder nennen.
**Wie vermeiden:** Alle FE-Init-Sites kartiert (Tabelle oben) — die FE-Sites sind schon OK
(verwenden `state::WeeklySummary`, das Feld existiert bereits als `Vec<SalesPersonAbsence>`).
Nur `WeeklySummaryTO` bekommt ein NEUES Feld — dort ist serde-Deser der Befüll-Weg, kein
Literal-Konstruktor im FE.

### Pitfall 6: `absent_volunteer_ids` in `get_summery_for_week` nicht nach `volunteer_ids` gefiltert

**Was schiefgeht:** `all_absences` enthält auch Absencen von bezahlten Mitarbeitern;
ohne `volunteer_ids.contains(&period.sales_person_id)` würden Bezahlte mit Absence auftauchen.
**Warum:** `find_all()` ist kategorie- und rollen-agnostisch.
**Wie vermeiden:** Exakt dieselbe Filterung wie in `get_weekly_summary:429–431`:
`volunteer_ids.contains(&period.sales_person_id) && period_overlaps_week(...)`.

---

## Don't Hand-Roll

| Problem | Nicht bauen | Stattdessen | Warum |
|---------|-------------|-------------|-------|
| Wochen-Überlappungsprüfung | Eigene Date-Range-Logik | `period_overlaps_week()` (bereits in `booking_information.rs:81`) | Bestehender, getesteter VFA-01-Helper |
| EmployeeWorkDetails-Zeitbereichs-Filter | Eigenes Datum-Fenster | `find_working_hours_for_calendar_week()` aus `crate::reporting` | Bereits importiert, korrekte Range-Semantik |
| Cap-Gate-Check | Inline-Bool-Wiederholung | `.filter(|wh| wh.cap_planned_hours_to_expected || wh.expected_hours == 0.0)` | Identisch zur bestehenden Zeile 514 — nicht neu erfinden |
| Sort case-insensitive | ICU/unicode Dep | `str::to_lowercase().cmp()` | D-53-04 explizit: kein neuer Dep nötig |

---

## Runtime State Inventory

> Phase 53 ist kein Rename/Refactor — nur Feld-Erweiterung. Inventar dennoch explizit beantwortet.

| Kategorie | Gefundenes | Aktion |
|-----------|-----------|--------|
| Stored data | Keine persistierten `WeeklySummary`-Rows (live-view-only) | Keine Migration |
| Live service config | Kein externer Service enthält `WeeklySummary`-Struktur | Keine Aktion |
| OS-registered state | Keine | Keine |
| Secrets/env vars | Keine berührt | Keine |
| Build artifacts | Kein `egg-info`/installiertes Binary betroffen | `cargo sqlx prepare` falls neue query! — VAA-Phase hat keine neue SQL-Query (keine DAO-Änderung) |

**`cargo sqlx prepare` nicht nötig:** Phase 53 berührt keine DAO-Traits, keine `query!`/`query_as!`-Makros. Keine `.sqlx`-Datei-Änderung.

---

## Environment Availability

> Reine Code-Änderung ohne externe Deps. Minimaler Check.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/Cargo | Backend-Build | ✓ | (nix develop) | — |
| `cargo clippy` | Clippy-Gate | ✓ | via nix develop | — |
| `cargo test` | Test-Gate | ✓ | via nix develop | — |
| wasm32 target | FE-WASM-Gate (optional) | ✓ | via nix develop | Skip FE-Gate wenn Tool fehlt |

**Keine blockenden fehlenden Dependencies.**

---

## Validation Architecture

### Test Framework

| Property | Wert |
|----------|------|
| Framework | Rust built-in + tokio-test (`#[tokio::test]`) |
| Config-Datei | kein separates Config-File — Cargo.toml workspace |
| Quick run command | `cargo test -p service_impl booking_information_vaa -- --nocapture` |
| Full suite command | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map

| REQ-ID | Verhalten | Test-Typ | Automatisierter Command | File vorhanden? |
|--------|-----------|----------|------------------------|-----------------|
| VAA-01 | Freiwilliger mit Vacation-Period in Woche N erscheint in `sales_person_absences` | unit (Backend-Service) | `cargo test -p service_impl vaa01_volunteer_with_period_appears` | ❌ Wave 3 |
| VAA-02 | Stunden-Wert = `committed_voluntary` cap-gated (D-53-02-Formel) | unit (Backend-Service) | `cargo test -p service_impl vaa02_hours_value_is_committed_voluntary` | ❌ Wave 3 |
| VAA-03 #1 | Freiwilliger mit Period → erscheint mit korrektem Wert | unit | `cargo test -p service_impl vaa03_volunteer_with_period` | ❌ Wave 3 |
| VAA-03 #2 | Freiwilliger ohne Period → nicht in Liste | unit | `cargo test -p service_impl vaa03_volunteer_without_period_absent` | ❌ Wave 3 |
| VAA-03 #3 | Bezahlter Mitarbeiter unverändert (Regression-Lock) | unit | `cargo test -p service_impl vaa03_paid_employee_unchanged` | ❌ Wave 3 |
| VAA-04 | FE-Rendering-Zeile bleibt buchstäblich unverändert (keine neue Farbe/Icon) | Static (code review) | `grep -n 'format.*h.*absence' shifty-dioxus/src/page/weekly_overview.rs` | ✓ Bestehendes File |
| FE-Mapper | Union-Merge + sort_by(name) korrekt | unit (FE cargo test) | `cargo test -p shifty-dioxus state::weekly_overview` | ❌ Wave 2 (neuer Test) |

### Sampling Rate

- **Per task commit:** `cargo test -p service_impl booking_information`
- **Per wave merge:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Phase gate:** Full suite grün + FE-WASM-Gate (`cargo build --target wasm32-unknown-unknown` in shifty-dioxus) vor `/gsd-verify-work`

### Wave 0 Gaps (vor der ersten Implementation)

- [ ] `service_impl/src/test/booking_information_vaa.rs` — VAA-01/02/03 Assertions
- [ ] `service_impl/src/test/mod.rs` — `pub mod booking_information_vaa` eintragen
- [ ] FE-Test in `state/weekly_overview.rs` erweitern: Union-Merge-Assertion (Freiwilliger + Bezahlter in sort-order)

---

### Test-Skelett für VAA-03 (Backend) [ASSUMED: Struktur von booking_information_vfa.rs abgeleitet]

```rust
// service_impl/src/test/booking_information_vaa.rs
//
// VAA-03: sales_person_absences in get_weekly_summary
// 3 Assertions: (1) Freiwilliger mit Period erscheint,
//               (2) Freiwilliger ohne Period → nicht in Liste,
//               (3) Bezahlter Mitarbeiter unverändert.

// TestDeps: identisch zu booking_information_vfa.rs (gleiche Mock-Typen)

// Fixtures:
// - volunteer_id_1(): Uuid (abwesend — hat Period für WEEK_UNDER_TEST)
// - volunteer_id_2(): Uuid (anwesend — keine Period für WEEK_UNDER_TEST)
// - paid_id(): Uuid (is_paid=true, hat report mit absence_hours)
// - WEEK_UNDER_TEST: u8 = 20  (2026-W20)
// - YEAR: u32 = 2026
//
// Mock-Setup:
// - sales_person_service.get_all() → [volunteer_1 (is_paid=false), volunteer_2 (is_paid=false), paid (is_paid=true)]
// - absence_service.find_all() → [AbsencePeriod { sales_person_id: volunteer_id_1(), W20 }]
// - employee_work_details_service.all() → [wd_volunteer_1 { cap=true, committed_voluntary=5.0, year range: 2026 }]
// - reporting_service.get_year() → paid employee hat report mit absence_hours in W20
// - toggle_service.get_toggle_value() → Ok(None) (legacy off)
// - alle anderen: analog zu vfa-Test (leer / passthrough)

#[tokio::test]
async fn vaa03_volunteer_with_period_appears_with_correct_hours() {
    // Setup ...
    let summaries = service.get_weekly_summary(YEAR, Authentication::Full, None).await.unwrap();
    let week = summaries.iter().find(|s| s.week == WEEK_UNDER_TEST).unwrap();
    assert!(
        week.sales_person_absences.iter().any(|a| a.sales_person_id == volunteer_id_1()),
        "VAA-03: Freiwilliger mit Period muss in sales_person_absences erscheinen"
    );
    let entry = week.sales_person_absences.iter().find(|a| a.sales_person_id == volunteer_id_1()).unwrap();
    assert!((entry.hours - 5.0).abs() < 0.001, "VAA-02: hours muss committed_voluntary sein");
}

#[tokio::test]
async fn vaa03_volunteer_without_period_absent_not_in_list() {
    // Setup (volunteer_id_2 hat keine Period)
    let week = /* ... */;
    assert!(
        !week.sales_person_absences.iter().any(|a| a.sales_person_id == volunteer_id_2()),
        "VAA-03: Freiwilliger ohne Period darf nicht in sales_person_absences erscheinen"
    );
}

#[tokio::test]
async fn vaa03_paid_employee_unchanged_regression_lock() {
    // Bezahlter Mitarbeiter hat report mit absence_hours > 0
    // working_hours_per_sales_person muss weiterhin korrekt befüllt sein
    // sales_person_absences darf den Bezahlten nicht enthalten
    let week = /* ... */;
    assert!(
        !week.sales_person_absences.iter().any(|a| a.sales_person_id == paid_id()),
        "VAA-03: Bezahlter Mitarbeiter darf nicht in sales_person_absences erscheinen"
    );
    assert!(
        week.working_hours_per_sales_person.iter().any(|wh| wh.sales_person_id == paid_id()),
        "VAA-03 Regression-Lock: working_hours_per_sales_person bleibt für Bezahlte erhalten"
    );
}
```

---

## Security Domain

> `security_enforcement` nicht explizit `false` in config.json — Abschnitt required.

### Applicable ASVS Categories

| ASVS Category | Applicable | Standard Control |
|---------------|-----------|-----------------|
| V2 Authentication | Nein — Endpoint erfordert SHIFTPLANNER_PRIVILEGE (bereits vorhanden, kein neuer Auth-Code) | bestehende `check_permission`-Calls |
| V3 Session Management | Nein | — |
| V4 Access Control | Nein — neues Feld ist nur für Shiftplanner sichtbar (durch `is_shiftplanner`-Gate unverändert; Freiwilligen-Namen sind HR-Infos) | bestehender `is_shiftplanner`-Check in get_weekly_summary |
| V5 Input Validation | Nein — keine neuen Inputs (nur Output-Feld) | — |
| V6 Cryptography | Nein | — |

### Hinweis: Freiwilligen-Namen im DTO

`SalesPersonAbsence.name` enthält Mitarbeiternamen. Diese sind bereits in
`working_hours_per_sales_person` für Bezahlte enthalten — dasselbe Permission-Gate
(`is_shiftplanner`) schützt sie. Das neue Feld geht durch denselben Code-Pfad und wird
ebenfalls nur bei `is_shiftplanner = true` befüllt.

**Kein neues Berechtigungsrisiko.**

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Daten-Leak: Mitarbeiternamen für Nicht-Shiftplanner | Information Disclosure | Bestehendes `is_shiftplanner`-Gate; `sales_person_absences` im Assembly-Loop hinter demselben Check |

---

## State of the Art

| Alter Ansatz | Aktueller Ansatz | Geändert | Impact |
|--------------|-----------------|----------|--------|
| FE-Merge: `working_hours_per_sales_person`-Loop baut alle Absencen | Phase 53: Backend liefert Freiwilligen-Absencen fertig im DTO; FE macht Union-Merge aus beiden Quellen | Phase 53 | Sauberere Separation; FE kennt keine Formel mehr |
| Jahresansicht zeigte nur bezahlte Mitarbeiter-Absencen | Jahresansicht zeigt Union: bezahlt + Freiwillige (mit Absence-Period) | Phase 53 | Vollständigere Verfügbarkeitsübersicht |

**Deprecated/obsolete:**
- FE-solo-Merge aus `working_hours_per_sales_person` für *alle* Absencen → ersetzt durch Backend-gelieferte Freiwilligen-Absencen + FE-Union. FE-Bezahlt-Pfad bleibt erhalten (Regression-Lock VAA-03 #3).

---

## Assumptions Log

| # | Behauptung | Abschnitt | Risiko wenn falsch |
|---|-----------|-----------|-------------------|
| A1 | `WeeklySummaryTO` hat kein `ToSchema` (kein ApiDoc-Eintrag nötig für `SalesPersonAbsenceTO`) | OpenAPI/utoipa Impact | Gering: wenn ToSchema hinzugefügt werden müsste, wäre das ein additiver Change |
| A2 | FE `state::WeeklySummary` bricht nicht durch neues DTO-Feld (nur DTO-Side neu, state-side Feld existiert schon) | FE-Init-Sites | Gering: verifiziert durch Code-Lesen der Init-Sites |
| A3 | `cargo sqlx prepare` nicht nötig (keine neuen DAO-Queries) | Runtime State Inventory | Mittel: wenn doch eine neue Query eingebaut wird, muss prepare laufen — Planner in Gate setzen |

**Alle Claims in diesem Research wurden per direkter Code-Verifikation (Read-Tool) bestätigt.**

---

## Open Questions

1. **Sortier-Stabilität bei Namensduplikaten**
   - Was bekannt ist: D-53-04 definiert sort by `name.to_lowercase()` (case-insensitive)
   - Unklar: Bei zwei Personen mit identischem Namen (unwahrscheinlich, aber möglich) ist die Reihenfolge instabil.
   - Empfehlung: Kein Handlungsbedarf — stable sort in Rust (`Vec::sort_by`) ist stabil bei gleichen Schlüsseln.

2. **F07-Docs-Update Scope**
   - Was bekannt ist: Kein hartes Gate ausgelöst, aber `WeeklySummary` wird in F07 erwähnt.
   - Unklar: Wünscht User einen expliziten VAA-Nebensatz in F07 oder reicht es als Phase-53-Ergebnis implizit zu verstehen?
   - Empfehlung: Planner fügt optionalen Wave-4-Task "F07 + F07_de Nebensatz VAA" hinzu; blockiert nicht das Phase-Gate.

---

## Sources

### Primary (HIGH confidence)
- `service_impl/src/booking_information.rs` — direkte Code-Verifikation der Fill-Sites, Formel, `absent_volunteer_ids`-Aufbau
- `service/src/booking_information.rs` — Trait + Struct-Definitionen (WeeklySummary, WorkingHoursPerSalesPerson)
- `rest-types/src/lib.rs` — WeeklySummaryTO, WorkingHoursPerSalesPersonTO, From-Impls
- `shifty-dioxus/src/state/weekly_overview.rs` — FE-Mapper, bestehender Merge-Loop
- `service_impl/src/test/booking_information_vfa.rs` — Referenz-Teststruktur für VAA-03

### Secondary (MEDIUM confidence)
- `shifty-dioxus/src/page/weekly_overview.rs` — Rendering-Zeile + Init-Sites
- `shifty-dioxus/src/loader.rs` — FE-Init-Site Fallback-Konstruktor
- `shifty-dioxus/src/component/weekly_overview_chart.rs` — FE-Init-Site Test-Helper
- `service_impl/src/reporting.rs` — `find_working_hours_for_calendar_week` Helper-Signatur

### Tertiary (LOW confidence)
- keine

---

## Metadata

**Konfidenz-Aufschlüsselung:**
- Standard Stack: HIGH — alle Crates direkt im Workspace, keine externen Deps
- Architektur: HIGH — direkte Code-Verifikation beider Fill-Sites + FE-Mapper
- Pitfalls: HIGH — aus konkretem Code-Reading abgeleitet (nicht aus Training)
- Test-Skelett: MEDIUM — Struktur von `booking_information_vfa.rs` abgeleitet (Pattern ist etabliert, Fixture-Details im Planner-Ermessen)

**Research date:** 2026-07-06
**Valid until:** 2026-08-06 (stabiler Code, kein fast-moving Ecosystem)
