---
quick_id: 260612-svs
slug: im-frontend-absences-seite-f-rs-erste-au
date: 2026-06-12
mode: quick-discuss
---

# Quick Task 260612-svs: Absences-Seite — Krankheitstage vorerst ausblenden

## Beschreibung

Auf der Absences-Seite (`shifty-dioxus/src/page/absences.rs`) sollen fürs Erste nur
Urlaubstage und unbezahlte Tage erscheinen — keine Krankheitstage (dort fehlt fachlich
noch etwas). Reports bleiben unverändert. Umsetzung über zentrale Konstante
(`SICK_LEAVE_ENABLED = false`), damit die Reaktivierung später ein One-Liner ist.

Entscheidungen aus CONTEXT.md (locked):
- Liste: Krankheits-Absences UND Krankheits-Marker komplett ausblenden
- Modal: Kategorie-Option „Krankheit" entfernen
- Stats-Kachel „Krankheitstage" + Filter-Option „Krankheit" ausblenden
- Nur Absences-Seite — Reports unangetastet

## Tasks

### Task 1: Konstante + Sichtbarkeits-Helper + Page-Pipeline
- **files:** `shifty-dioxus/src/page/absences.rs`
- **action:**
  - `pub const SICK_LEAVE_ENABLED: bool = false;` mit Kommentar (temporär).
  - Pure Helpers: parametrischer Kern `category_visible_with(enabled, category)` +
    Wrapper `is_visible_category(category)` und `is_visible_marker_category(&ExtraHoursCategoryTO)`.
  - In `AbsencesPage`: `visible_absences` / `visible_markers` als Single-Choke-Point
    vor der bestehenden Filter-Pipeline; `filtered`, `total_count` und `StatsGrid`
    konsumieren nur noch die sichtbaren Listen.
- **verify:** `cargo test` + `cargo check --target wasm32-unknown-unknown`
- **done:** Krankheits-Einträge und -Marker erscheinen nicht mehr in Liste/Zähler/Stats.

### Task 2: UI-Stellen — Stats-Kachel, Modal-Dropdown, Filter-Dropdown
- **files:** `shifty-dioxus/src/page/absences.rs`
- **action:**
  - `StatsGrid`: Krankheits-StatBox nur bei `SICK_LEAVE_ENABLED` rendern; Grid-Spalten
    konditional (2 statt 3 Spalten im Desktop-Layout).
  - `AbsenceModal`: `sick_leave`-Option im Kategorie-Dropdown nur bei `SICK_LEAVE_ENABLED`.
  - `AbsenceFilterBar`: `sick_leave`-Option im Kategorie-Filter nur bei `SICK_LEAVE_ENABLED`.
- **verify:** SSR-Snapshot-Tests (bestehende) + neue Assertions.
- **done:** Keine Krankheits-UI mehr auf der Seite sichtbar.

### Task 3: Tests
- **files:** `shifty-dioxus/src/page/absences.rs` (`#[cfg(test)] mod tests`)
- **action:** Pure-Function-Tests für `category_visible_with` (enabled/disabled ×
  Kategorien) und Marker-Variante; SSR-Test: StatsGrid rendert ohne Krankheits-Label,
  FilterBar/Modal ohne `sick_leave`-Option.
- **verify:** `cargo test` grün.
- **done:** Ausblendungslogik testabgedeckt.

## must_haves

- **truths:**
  - Bei `SICK_LEAVE_ENABLED == false` erscheinen auf der Absences-Seite keine
    Krankheits-Absences, -Marker, -Stats-Kachel, -Modal-Option, -Filter-Option.
  - Reports/andere Seiten unverändert (kein Code außerhalb `absences.rs`).
  - Reaktivierung = Konstante auf `true` setzen.
- **artifacts:** Konstante + Helpers + Tests in `absences.rs`.
- **key_links:** `shifty-dioxus/src/page/absences.rs`
