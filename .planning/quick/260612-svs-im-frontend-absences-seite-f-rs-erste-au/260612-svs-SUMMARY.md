---
quick_id: 260612-svs
slug: im-frontend-absences-seite-f-rs-erste-au
date: 2026-06-12
status: complete
---

# Quick Task 260612-svs — Summary

## Was geändert wurde

Auf der Absences-Seite werden Krankheitstage vorerst vollständig ausgeblendet —
nur Urlaubstage und unbezahlte Tage sind sichtbar. Reports bleiben unverändert.

### Datei: `shifty-dioxus/src/page/absences.rs` (einzige geänderte Datei)

1. **Zentrale Konstante** `pub const SICK_LEAVE_ENABLED: bool = false;` mit
   Doku-Kommentar — Reaktivierung ist ein One-Liner.
2. **Pure Helpers:** parametrischer Kern `category_visible_with(enabled, category)`
   + Wrapper `is_visible_category` und `is_visible_marker_category` (unmappbare
   Marker-Kategorien bleiben sichtbar, nur SickLeave wird ausgeblendet).
3. **Single-Choke-Point in `AbsencesPage`:** `visible_absences` / `visible_markers`
   werden VOR der User-Filter-Pipeline berechnet; Liste, `total_count` („X von Y")
   und `StatsGrid` konsumieren nur sichtbare Einträge (auch `active_count` zählt
   keine versteckten Krankheits-Absences mehr).
4. **StatsGrid:** Krankheits-Kachel nur bei `SICK_LEAVE_ENABLED`; Grid konditional
   2 statt 3 Desktop-Spalten (beide Tailwind-Klassenstrings literal — Pitfall 5).
5. **AbsenceModal + AbsenceFilterBar:** `sick_leave`-Option in beiden
   Kategorie-Dropdowns nur bei `SICK_LEAVE_ENABLED`.
6. **6 neue Tests:** 3 Pure-Function-Tests (beide Schalterstellungen + Marker-
   Variante inkl. unmappbarer Kategorie), 3 SSR-Snapshots (StatsGrid ohne
   Krankheits-Kachel, FilterBar + Modal ohne `sick_leave`-Option, Urlaub/Unbezahlt
   weiterhin vorhanden).

## Entscheidungen (aus CONTEXT.md)

- Komplett ausblenden (Liste + Marker), Anlegen entfernt, Stats-Kachel + Filter raus.
- Scope strikt auf die Absences-Seite begrenzt — Reports zeigen Krankheitstage weiter.
- Konstante statt hartem Entfernen, weil die Abschaltung explizit temporär ist.

## Verifikation

- `cargo test` → **589 passed, 0 failed** (47 davon in absences, 6 neu).
- `cargo check --target wasm32-unknown-unknown` → sauber (nur vorbestehende Warnings).

## Commit

Noch **nicht committet** — User committet manuell via jj (`jj-commit`-Skill).
