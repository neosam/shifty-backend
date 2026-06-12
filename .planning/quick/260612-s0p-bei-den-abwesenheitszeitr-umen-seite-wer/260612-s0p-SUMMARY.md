---
quick_id: 260612-s0p
slug: bei-den-abwesenheitszeitr-umen-seite-wer
date: 2026-06-12
status: complete
---

# Quick Task 260612-s0p — Summary

## Was geändert wurde

Auf der Abwesenheitszeiträume-Seite zeigen die Personen-Dropdowns jetzt nur noch
Sales Persons, die **bezahlt** (`is_paid == true`) **und aktiv** (`inactive == false`)
sind.

### Datei: `shifty-dioxus/src/page/absences.rs`

1. **Neue Pure-Function** `is_selectable_employee(sp: &SalesPerson) -> bool`
   (`sp.is_paid && !sp.inactive`), platziert neben `compute_status`.
2. **`AbsenceModal`-Dropdown** (Mitarbeiterauswahl): `.filter(|sp| is_selectable_employee(sp))`.
3. **`AbsenceFilterBar`-Dropdown** (HR-Personenfilter): `.filter(|sp| is_selectable_employee(sp))`.
4. **4 Unit-Tests** im bestehenden `#[cfg(test)] mod tests` für alle paid×active-Kombinationen.

### Bewusste Designentscheidung

Gefiltert wird **nur am Dropdown** (Auswahlpunkt), nicht an der Datenquelle
(`loader::load_sales_persons`). Die geladene Liste bleibt vollständig, damit bestehende
Absences von inzwischen inaktiven/unbezahlten Personen weiterhin korrekt mit Namen
aufgelöst werden. Das entspricht dem etablierten Muster in `shiftplan.rs`.

## Verifikation

- `cargo test selectable` → 4 passed, 0 failed.
- `cargo check --target wasm32-unknown-unknown` → kompiliert sauber (nur vorbestehende Warnings).
- Voller `cargo build --target wasm32-unknown-unknown` scheiterte nur am Link-Schritt
  (`linker lld not found`) — umgebungsbedingt (NixOS), unabhängig von dieser Änderung.

## Commit

Noch **nicht committet** — in diesem jj-Repo committet der User manuell via jj
(`jj-commit`-Skill).
