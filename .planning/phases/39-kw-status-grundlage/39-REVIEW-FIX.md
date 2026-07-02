---
phase: 39-kw-status-grundlage
fixed_at: 2026-07-02T02:30:00Z
review_path: .planning/phases/39-kw-status-grundlage/39-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 4
skipped: 3
status: partial
---

# Phase 39: Code Review Fix Report

**Fixed at:** 2026-07-02T02:30:00Z
**Source review:** .planning/phases/39-kw-status-grundlage/39-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 7 (WR-01, WR-02, WR-03, LO-01, LO-02, LO-03, LO-04)
- Fixed: 4 (WR-01, WR-02, LO-01, LO-02)
- Skipped intentionally: 3 (WR-03, LO-03, LO-04 — see rationale below)

## Fixed Issues

### WR-01: WeekStatusBadge rendert leer statt Panic bei Unset

**Files modified:** `shifty-dioxus/src/component/atoms/week_status_badge.rs`
**Commit:** 45206812
**Applied fix:**
- `week_status_badge_class(&WeekStatus::Unset)` gibt jetzt `""` zurueck statt `unreachable!()` zu triggern.
- `WeekStatusBadge` haelt fruehetigehn Guard: `if props.status == WeekStatus::Unset { return rsx!{}; }`.
- Neuer Test `badge_class_for_unset_does_not_panic` verifiziert das Verhalten.
- Modul-Docstring aktualisiert (beschreibt nun den internen Guard statt unreachable!).

### WR-02: Tote Felder loaded_year/loaded_week aus WeekStatusStore entfernt

**Files modified:** `shifty-dioxus/src/service/week_status.rs`
**Commit:** e1f797a5
**Applied fix:**
- `pub loaded_year: Option<u32>` und `pub loaded_week: Option<u8>` aus `WeekStatusStore` entfernt.
- Zugehoerige Schreib-Stellen `store.loaded_year = Some(year)` und `store.loaded_week = Some(week)` entfernt.
- Grep-Verifizierung vor Entfernung: keine Leser in der gesamten Codebase.

### LO-01: Veraltetes #[allow(dead_code)] auf WeekStatusAction entfernt

**Files modified:** `shifty-dioxus/src/service/week_status.rs`
**Commit:** 645fa98a
**Applied fix:**
- `#[allow(dead_code)]` und den zugehoerigen Platzhalter-Kommentar entfernt.
- `WeekStatusAction` wird aktiv in `page/shiftplan.rs` (Load + Set) verwendet.

### LO-02: Toter 404-Pfad in api::get_week_status entfernt

**Files modified:** `shifty-dioxus/src/api.rs`, `shifty-dioxus/src/service/week_status.rs`
**Commit:** 3b658e68
**Applied fix:**
- 404-Branch aus `get_week_status` entfernt; Backend gibt immer HTTP 200 zurueck.
- Rueckgabetyp von `Result<Option<WeekStatusTO>, _>` auf `Result<WeekStatusTO, _>` vereinfacht.
- Aufrufer `load_week_status` in `service/week_status.rs` angepasst (kein `.map(...).unwrap_or(...)` mehr).
- Erklaerenden Kommentar (`D-39-06`) in der API-Funktion eingefuegt.

## Skipped Issues

### WR-03: Soft-Delete bumpt update_version nicht

**File:** `dao_impl_sqlite/src/week_status.rs:163-175`
**Reason:** Absichtlich uebersprungen — `week_message` nutzt exakt dasselbe Muster (kein Version-Bump beim Soft-Delete). Eine Aenderung nur an `week_status` wuerde vom etablierten Template-Muster abweichen. Beide Entitaeten sollten konsistent angepasst werden, sobald das Projekt eine globale Konsistenz-Initiative fuer delete-version-bumps startet.

### LO-03: PUT-Handler ignoriert body.year / body.calendar_week

**File:** `rest/src/week_status.rs:84-111`
**Reason:** Absichtlich uebersprungen — die Pfad-Parameter sind kanonisch. Das ist eine akzeptable Design-Entscheidung (Pfad gewinnt ueber Body). Keine Sicherheitsimplikation. Das gleiche Muster existiert im Projekt. Kein Fix ohne Scope-Aenderung (neues Body-Struct wuerde REST-Types-Crate beeinflussen).

### LO-04: update() prueft rows_affected nicht

**File:** `dao_impl_sqlite/src/week_status.rs:131-154`
**Reason:** Absichtlich uebersprungen — entspricht dem etablierten Muster in `week_message` und anderen einfachen DAOs. SQLite ist Single-Writer; bei aktueller Nutzung in einer Transaktion ist ein stilles No-Op praktisch unmoeglich. Konsistenz mit dem Projektmuster hat Vorrang.

---

## Gate-Ergebnisse

| Gate | Ergebnis |
|---|---|
| `cargo clippy --workspace -- -D warnings` (backend root) | GRUEN — keine Warnungen |
| `cargo build --target wasm32-unknown-unknown` (shifty-dioxus) | GRUEN — kompiliert sauber |
| `cargo test -p shifty-dioxus week_status` | GRUEN — 14/14 Tests bestanden |

---

_Fixed: 2026-07-02T02:30:00Z_
_Fixer: Claude Sonnet 4.6 (gsd-code-fixer)_
_Iteration: 1_
