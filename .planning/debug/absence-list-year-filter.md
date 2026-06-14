---
slug: absence-list-year-filter
status: resolved
trigger: "In der Abwesenheitsliste sehe ich nach wie vor stundenbasierte Einträge von Jahren, die nicht ausgewählt sind."
created: 2026-06-14
updated: 2026-06-14
---

# Debug: absence-list-year-filter

## Symptoms

- expected: In der globalen Abwesenheits-Übersicht sollen nur Einträge des aktuell ausgewählten Jahres angezeigt werden.
- actual: Es werden stundenbasierte Legacy-Einträge (alte `extra_hours`) von Jahren angezeigt, die NICHT ausgewählt sind.
- view: Globale Abwesenheits-Übersicht (übergreifende Liste über alle Mitarbeiter)
- entry_type: Alte `extra_hours`-Einträge (Einzeltage, stundenbasiert, vor dem Absence-Cutover)
- error_messages: keine (falsche Daten, kein Fehler/Crash)
- timeline: War schon immer falsch — keine Regression. Jahres-Filterung dieser Legacy-Einträge funktionierte in dieser Ansicht nie korrekt.
- reproduction: Globale Abwesenheits-Übersicht öffnen, ein Jahr auswählen → stundenbasierte Einträge aus anderen Jahren bleiben sichtbar.

## Current Focus

reasoning_checkpoint:
  hypothesis: "Die Absences-Seite (shifty-dioxus/src/page/absences.rs) wendet auf KEINE der beiden Quellen (Range-Absences und stundenbasierte Marker) einen Jahres-Filter an. `selected_year` steuert nur Vacation-Balance/Stats-Label-Reload, nicht die gerenderte Liste. Deshalb erscheinen Marker (und Ranges) aus allen Jahren unabhängig vom gewählten Jahr."
  confirming_evidence:
    - "Filter-Pipeline (absences.rs:1901-1942): `filtered` und `filtered_markers` filtern nur category/person/status/show_past — `year` kommt nirgends vor."
    - "Loader (loader.rs:891-951) und API (list_absence_periods / list_absence_periods_by_sales_person) sind nicht jahres-scoped: sie liefern ALLE Ranges + ALLE hourly_markers."
    - "`year` (absences.rs:1808) fließt nur in StatsGrid (Label/in_year-Zählung) und VacationEntitlementCard — nicht in AbsenceList."
    - "ExtraHoursMarker hat `when: time::Date` (state/absence_period.rs:124); AbsencePeriod hat from_date/to_date — beide tragen die Jahres-Info clientseitig."
  falsification_test: "Wenn nach Hinzufügen eines Jahres-Predicate Marker/Ranges aus anderen Jahren weiterhin sichtbar wären, wäre die Hypothese falsch. Unit-Tests über `marker_matches_filters` + neue Range-Year-Funktion mit Einträgen aus zwei Jahren beweisen/widerlegen das."
  fix_rationale: "Jahres-Predicate clientseitig in beide Filter-Closures einziehen (Marker: when.year()==year; Range: from_date.year()==year || to_date.year()==year, konsistent mit stats_for_person). Adressiert die Wurzel: fehlender Year-Filter in der Render-Pipeline — kein Backend-Symptom-Patch."
  next_action: "marker_matches_filters um year-Param erweitern, Range-Filter-Closure um year-Predicate erweitern, pure Helper `range_in_year` + Tests hinzufügen."

## Evidence

- timestamp: 2026-06-14
  checked: "shifty-dioxus/src/page/absences.rs Filter-Pipeline (Zeilen 1881-1948) und marker_matches_filters (181-214)"
  found: "Weder die Range-Filter-Closure (1901-1926) noch marker_matches_filters wenden ein Jahres-Predicate an. Gefiltert wird nur nach category/person/status/show_past."
  implication: "Der gewählte Jahres-Filter (`selected_year`) wirkt sich nicht auf die gerenderte Absence-/Marker-Liste aus."

- timestamp: 2026-06-14
  checked: "shifty-dioxus/src/loader.rs (891-951), service/absence.rs LoadAll/LoadForSalesPerson"
  found: "Loader/API laden ALLE Absence-Periods und ALLE hourly_markers ohne Jahres-Parameter. Es gibt keinen Backend-seitigen Jahres-Filter für diese Liste."
  implication: "Die Jahres-Filterung MUSS clientseitig in der Render-Pipeline erfolgen (oder via neuer year-scoped Loader). Minimaler Fix = clientseitiges Predicate."

- timestamp: 2026-06-14
  checked: "stats_for_person (absences.rs:1414-1443)"
  found: "Stats nutzen bereits `from_date.year()==year || to_date.year()==year` als in_year-Kriterium — etabliertes Jahres-Semantik-Vorbild im selben File."
  implication: "Range-Year-Predicate konsistent zu stats_for_person halten (überlappt das Jahr)."

## Eliminated

- hypothesis: "Backend-Service/DAO-Jahres-Filter für Absence-Aggregation fehlt/ist falsch."
  evidence: "Die globale Übersicht ist eine reine Frontend-Liste (shifty-dioxus); Backend liefert bewusst alle Einträge, Jahr ist eine clientseitige Sicht. Kein Backend-Year-Param in diesem Pfad."
  timestamp: 2026-06-14

## Resolution

root_cause: "shifty-dioxus/src/page/absences.rs filtert die gerenderte Liste (Range-Absences + stundenbasierte Legacy-Marker) nur nach category/person/status/show_past, aber NIE nach dem gewählten `selected_year`. Backend/Loader liefern absichtlich alle Jahre. Folge: stundenbasierte extra_hours-Marker (und Ranges) anderer Jahre bleiben sichtbar, egal welches Jahr gewählt ist."
fix: |
  shifty-dioxus/src/page/absences.rs — clientseitiges Jahres-Predicate in die Render-Pipeline eingezogen:
  - Neue pure Helper `range_in_year(from, to, year)` (touches-year, konsistent zu stats_for_person) und `marker_in_year(when, year)`.
  - `marker_matches_filters(..)` um `year: u32` erweitert; schließt Marker aus anderen Jahren als ERSTES Kriterium aus.
  - Range-Filter-Closure in `AbsencesPage` um `range_in_year(...)`-Guard erweitert.
  - `total_count` (Counter-Nenner "X von Y") ebenfalls jahres-scoped, da `selected_year` eine Navigations-Sicht ist (wie ◀ Jahr ▶), kein User-Filter.
  Backend/Loader unverändert (liefern bewusst alle Jahre).
verification: |
  - `cargo test` (shifty-dioxus): 600 passed, 0 failed. Neue Tests grün:
    marker_in_year_matches_only_its_year, marker_from_other_year_is_filtered_out,
    marker_year_filter_wins_over_other_passing_filters, range_in_year_touches_either_boundary.
    `marker_from_other_year_is_filtered_out` reproduziert den Bug direkt (2024er Marker bei year=2026 → ausgeschlossen, bei year=2024 → sichtbar).
  - `cargo check` (shifty-dioxus): clean.
  - WASM-Build-Gate konnte hier nicht laufen (Linker `lld` fehlt im Environment, prä-existent, auch ohne diese Änderung) — Native-Build kompiliert dieselbe absences.rs und ist grün.
  - Human-Verify in der echten UI (Chrome, 2026-06-14): BESTÄTIGT. Methode: zwei
    stundenbasierte Vacation-extra_hours-Testmarker (2025 + 2026, beide im
    two_year_window) in localdb.sqlite3 angelegt.
    - Jahr 2026 gewählt → Liste "4 of 4": 2026er Stundenmarker (Badge "hours-based")
      sichtbar, 2025er Marker NICHT sichtbar.
    - Jahr 2025 gewählt → Liste "1 of 1": nur 2025er Stundenmarker sichtbar, 2026er
      Marker NICHT sichtbar.
    → Symmetrisch korrekt: stundenbasierte Einträge erscheinen ausschließlich im
      eigenen Jahr. Bug behoben. Testdaten nach Verifikation wieder gelöscht
      (DB zurück im Ausgangszustand: nur 1 ExtraWork-Zeile).
    - Nebenbefund: Backend lädt Marker nur im Fenster [aktuelles Jahr-1, +1]
      (two_year_window(), rest/src/absence.rs:68) — Jahre außerhalb werden gar
      nicht erst geladen; der clientseitige Jahresfilter ist davon unabhängig korrekt.
files_changed:
  - shifty-dioxus/src/page/absences.rs

## Specialist Review

- specialist: rust/dioxus (general code review)
- verdict: LOOKS_GOOD
- summary: |
    Fix is idiomatic and correct. range_in_year uses touches-year semantics
    (from.year()==year || to.year()==year) consistent with stats_for_person;
    markers use exact when.year()==year (correct for single-day entries).
    Year-boundary/multi-year ranges handled correctly (Dec-Jan span shows under
    both years, intentional). u32->i32 cast sound, no off-by-one. Signal
    reactivity verified: year read in component body, so filter closures and
    total_count recompute on year navigation. Counter "X of Y" internally
    consistent (denominator year-scoped, numerator year-scoped + user-filtered).
    Regression tests directly encode the reported scenario.
  minor_note: |
    No component-level test for total_count year-scoping (only pure helpers
    tested) — pragmatic given Dioxus component-testing constraints in this repo.
