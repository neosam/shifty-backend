---
quick_id: 260612-nlv
type: execute
autonomous: true
files_modified:
  - shifty-dioxus/src/page/absences.rs
requirements: [QUICK-260612-nlv]
must_haves:
  truths:
    - "Stundenbasierte Marker werden in der Absence-Page durch dieselbe Kategorie-/Personen-/Status-/show_past-Filter-Pipeline geführt wie Range-Absences"
    - "Wenn nur stundenbasierte Einträge existieren und ein Filter gesetzt ist, blendet die Liste nicht-passende Marker aus statt alle anzuzeigen"
    - "filtered_count zählt gefilterte Range-Absences + gefilterte Marker; total_count zählt alle Absences + alle Marker"
    - "Ein Marker mit nicht-abbildbarer ExtraHoursCategoryTO (ExtraWork/Holiday/Unavailable/VolunteerWork/Custom) fällt bei aktivem konkretem Kategorie-Filter aus der Liste"
    - "HR-Gating des Personen-Filters (D-09) bleibt unverändert — kein Lockern des is_hr-Gates"
  artifacts:
    - path: "shifty-dioxus/src/page/absences.rs"
      provides: "marker_matches_filters pure function + gefilterte Marker-Durchreichung + korrigierte Counter"
      contains: "fn marker_matches_filters"
  key_links:
    - from: "AbsencesPage filtered-block"
      to: "marker_matches_filters"
      via: "iter().filter() über hourly_markers"
      pattern: "marker_matches_filters"
    - from: "AbsenceList props.hourly_markers"
      to: "filtered_markers_rc"
      via: "durchgereichte gefilterte Marker statt roher hourly_markers"
      pattern: "hourly_markers: filtered_markers_rc"
---

<objective>
Fix the Absence-Page filter bug: stundenbasierte Marker (`ExtraHoursMarker` aus
`ABSENCE_HOURLY_STORE`) werden aktuell UNGEFILTERT an `AbsenceList` durchgereicht
und gar nicht in `filtered_count`/`total_count` gezählt. Folge: wenn nur
stundenbasierte Einträge existieren, scheint die Filterung komplett kaputt (alle
Personen/Kategorien bleiben sichtbar, Zähler stimmt nicht).

Lösung (kein Scope-Creep): Eine pure, testbare Filter-Funktion
`marker_matches_filters` einführen — analog zur bestehenden Inline-Filter-Closure
für `absences` — die einen Marker gegen die vier Filterkriterien prüft
(Kategorie, Person, Status, show_past). Die gefilterten Marker an `AbsenceList`
durchreichen und die Counter um die Marker erweitern.

Purpose: Filter-Pipeline-Konsistenz zwischen Range-Absences und stundenbasierten
Markern; Counter passt zum Sichtbaren.
Output: 1 geänderte Datei (`shifty-dioxus/src/page/absences.rs`) — neue pure
function + verdrahtete Filterung + Counter-Korrektur + Unit-Tests.

Root cause ist bereits bestätigt; NICHT neu diagnostizieren.
</objective>

<execution_context>
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/workflows/execute-plan.md
</execution_context>

<context>
@.planning/STATE.md
@./CLAUDE.md
@./shifty-dioxus/CLAUDE.md

<interfaces>
<!-- Aus shifty-dioxus/src/state/absence_period.rs — vom Executor direkt nutzbar, keine Codebase-Exploration nötig. -->

ExtraHoursMarker (src/state/absence_period.rs:113-123):
```rust
pub struct ExtraHoursMarker {
    pub extra_hours_id: Uuid,
    pub sales_person_id: Uuid,
    pub when: time::Date,
    pub amount: f32,
    pub category: ExtraHoursCategoryTO,   // direkt vom Backend
    pub description: Arc<str>,
    pub person_name: Arc<str>,
}
```

AbsenceCategory + From-Impls (src/state/absence_period.rs:15-40):
```rust
pub enum AbsenceCategory { Vacation, SickLeave, UnpaidLeave }
// From<&AbsenceCategoryTO> for AbsenceCategory existiert.
// KEINE From-Impl ExtraHoursCategoryTO → AbsenceCategory — die muss hier
// als Option-liefernde Map-Funktion gebaut werden (nicht-abbildbar = None).
```

ExtraHoursCategoryTO (rest-types/src/lib.rs:708-717) — 8 Varianten:
```rust
pub enum ExtraHoursCategoryTO {
    ExtraWork, Vacation, SickLeave, Holiday,
    Unavailable, UnpaidLeave, VolunteerWork, Custom(Uuid),
}
```
NUR Vacation/SickLeave/UnpaidLeave bilden auf AbsenceCategory ab.
ExtraWork/Holiday/Unavailable/VolunteerWork/Custom sind NICHT abbildbar.

compute_status (src/page/absences.rs:91) — bereits vorhanden, pure:
```rust
pub fn compute_status(from: time::Date, to: time::Date, today: time::Date) -> AbsenceStatus
// AbsenceStatus { Active, Planned, Finished }
```
Für Marker als Einzeltag aufrufen: compute_status(marker.when, marker.when, today).
</interfaces>

<filter_reference>
<!-- Bestehende Range-Absence-Filter-Closure (src/page/absences.rs:1657-1682) — Vorbild: -->
- Kategorie: `if let Some(cat) = category_filter { if a.category != cat { return false } }`
- Person:    `if let Some(person) = person_filter { if a.sales_person_id != person { return false } }`
- Status:    `let status = compute_status(...); if let Some(s) = status_filter { if status != s { return false } }`
- show_past: `if !show_past && status == AbsenceStatus::Finished { return false }`
- `filter_active` (Z. 1685-1688): basiert NUR auf den Filterwerten, NICHT auf den Quellen → unverändert lassen.
</filter_reference>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: marker_matches_filters pure function + Unit-Tests</name>
  <files>shifty-dioxus/src/page/absences.rs</files>
  <behavior>
    Neue pure function (oberhalb des `#[cfg(test)]`-Moduls, nahe `compute_status`):

    `fn marker_matches_filters(marker: &ExtraHoursMarker, category_filter: Option<AbsenceCategory>, person_filter: Option<Uuid>, status_filter: Option<AbsenceStatus>, show_past: bool, today: time::Date) -> bool`

    Logik (analog zur Range-Closure, aber Einzeltag):
    - Kategorie: `ExtraHoursCategoryTO` → `Option<AbsenceCategory>` mappen
      (Vacation/SickLeave/UnpaidLeave → Some(...), alle anderen → None).
      Helper z.B. `fn map_marker_category(c: &ExtraHoursCategoryTO) -> Option<AbsenceCategory>`.
      Wenn `category_filter == Some(cat)`: Marker matcht nur, wenn
      `map_marker_category(&marker.category) == Some(cat)`. Insbesondere fällt
      ein Marker mit nicht-abbildbarer Kategorie bei aktivem konkretem
      Kategorie-Filter heraus (→ false).
    - Person: `if let Some(p) = person_filter { if marker.sales_person_id != p { return false } }`
    - Status: `let status = compute_status(marker.when, marker.when, today);`
      `if let Some(s) = status_filter { if status != s { return false } }`
    - show_past: `if !show_past && status == AbsenceStatus::Finished { return false }`
    - sonst `true`.

    Tests (im bestehenden `#[cfg(test)] mod tests`, neben den compute_status-Tests;
    injiziertes `today` analog — KEIN `js::current_datetime()` im Testpfad):
    - Helper `fn marker(category: ExtraHoursCategoryTO, sp: Uuid, when: time::Date) -> ExtraHoursMarker`
      (amount/description/person_name/extra_hours_id mit Dummy-Werten füllen).
    - marker_category_match: Filter=Some(Vacation) + Vacation-Marker → true.
    - marker_category_mismatch: Filter=Some(Vacation) + SickLeave-Marker → false.
    - marker_unmappable_category_with_active_filter: Filter=Some(Vacation) +
      ExtraWork-Marker → false (nicht-abbildbar fällt raus).
    - marker_no_category_filter_passes: Filter=None + ExtraWork-Marker → true
      (ohne aktiven Kategorie-Filter bleibt auch nicht-abbildbarer Marker sichtbar).
    - marker_person_match / marker_person_mismatch: Filter=Some(uuid) vs anderer uuid.
    - marker_status_filter_single_day: when=today, Filter=Some(Active) → true;
      Filter=Some(Planned) → false (Einzeltag-Status via compute_status).
    - marker_show_past_hides_finished: when in der Vergangenheit (Finished),
      show_past=false → false; show_past=true → true.
  </behavior>
  <action>
    Füge `marker_matches_filters` (+ privaten `map_marker_category`-Helper) im
    Produktions-Teil der Datei ein, sinnvoll platziert direkt nach `compute_status`
    (~Z. 99). Importiere `ExtraHoursCategoryTO` lokal in der Funktion oder am
    Dateikopf — `ExtraHoursMarker` ist bereits importiert (Z. 47), `compute_status`
    + `AbsenceStatus` + `AbsenceCategory` sind im selben Modul/importiert.
    Pitfall 5 ist hier nicht relevant (keine Tailwind-Klassen). Schreibe die Tests
    ZUERST (RED), dann die Funktion (GREEN) gemäß TDD.
  </action>
  <verify>
    <automated>cd shifty-dioxus && cargo test --lib marker_ 2>&1 | tail -20</automated>
  </verify>
  <done>Neue pure function existiert; alle neuen marker_*-Tests grün; bestehende Tests unverändert grün.</done>
</task>

<task type="auto">
  <name>Task 2: Marker durch Filter-Pipeline führen + Counter korrigieren</name>
  <files>shifty-dioxus/src/page/absences.rs</files>
  <action>
    In `AbsencesPage` (Filter-Block ~Z. 1654-1688):

    1. Nach dem `filtered`-Block für Range-Absences einen analogen Block für
       Marker einfügen:
       ```rust
       let filtered_markers: Vec<ExtraHoursMarker> = hourly_markers
           .iter()
           .filter(|m| marker_matches_filters(
               m, category_filter_val, person_filter_val,
               status_filter_val, show_past_val, today,
           ))
           .cloned()
           .collect();
       let filtered_markers_rc: Rc<[ExtraHoursMarker]> = Rc::from(filtered_markers);
       ```
    2. Counter korrigieren:
       - `total_count` (Z. 1656): `absences.len() + hourly_markers.len()`.
       - `filtered_count` (Z. 1683): `filtered.len() + filtered_markers_rc.len()`.
       (Reihenfolge so wählen, dass `filtered_markers_rc` vor der
       `filtered_count`-Berechnung existiert — ggf. `total_count`/`filtered_count`
       nach den beiden Filter-Blöcken verschieben.)
    3. `AbsenceList`-Aufruf (Z. 1832): `hourly_markers: hourly_markers.clone()`
       ersetzen durch `hourly_markers: filtered_markers_rc.clone()`.
    4. `filter_active` (Z. 1685-1688) UNVERÄNDERT lassen (basiert auf
       Filterwerten, nicht Quellen). Prüfen, dass das Empty-State-Verhalten
       von `AbsenceList` korrekt bleibt: wenn beide gefilterten Quellen leer
       und `filter_active` true → "keine Treffer für Filter"; wenn beide leer
       und `filter_active` false → "noch keine Einträge".
    5. HR-Gating des Personen-Filters (D-09) NICHT anfassen — kein `is_hr`-Gate
       ändern.

    KEINE Backend-Änderung, kein neuer API-Pfad, keine i18n-Änderung.
  </action>
  <verify>
    <automated>cd shifty-dioxus && cargo build --target wasm32-unknown-unknown 2>&1 | tail -5 && cargo test 2>&1 | tail -15 && cargo clippy 2>&1 | grep -A3 "absences.rs" | head -20; echo "clippy-done"</automated>
  </verify>
  <done>
    `filtered_markers_rc` wird an AbsenceList durchgereicht (kein rohes
    `hourly_markers` mehr im AbsenceList-Aufruf); total_count = absences + alle
    Marker; filtered_count = gefilterte absences + gefilterte Marker; WASM-Gate
    exit 0; cargo test grün; clippy sauber für die geänderte Datei.
  </done>
</task>

</tasks>

<verification>
- WASM-Gate: `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` exit 0.
- Tests: `cd shifty-dioxus && cargo test` — neue marker_*-Tests + alle bestehenden grün.
- Clippy: `cd shifty-dioxus && cargo clippy` sauber für `src/page/absences.rs`.
- Grep-Gates:
  - `grep -c "fn marker_matches_filters" src/page/absences.rs` ≥ 1
  - `grep "hourly_markers: filtered_markers_rc" src/page/absences.rs` matcht (Durchreichung gefixt)
  - `grep -c "is_hr" src/page/absences.rs` unverändert gegenüber HEAD (D-09 nicht angefasst)
</verification>

<success_criteria>
- Stundenbasierte Marker laufen durch dieselbe Kategorie-/Personen-/Status-/show_past-Pipeline wie Range-Absences.
- Counter (`filtered_count`/`total_count`) schließen die Marker ein und passen zum Sichtbaren.
- Nicht-abbildbare Marker-Kategorie fällt bei aktivem konkretem Kategorie-Filter heraus.
- Nur `shifty-dioxus/src/page/absences.rs` geändert; keine Backend-/API-/i18n-Änderung.
- HR-Gating (D-09) unverändert.
</success_criteria>

<output>
After completion, create `.planning/quick/260612-nlv-absence-page-stundenbasierte-marker-durc/260612-nlv-SUMMARY.md`.
VCS: jj-Repo — Commits macht der User manuell (jj-nativ). KEINE git-Operationen im Task.
</output>
