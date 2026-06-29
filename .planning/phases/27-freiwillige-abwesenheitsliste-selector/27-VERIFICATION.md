---
phase: 27-freiwillige-abwesenheitsliste-selector
verified: 2026-06-29T07:35:00Z
status: human_needed
score: 5/6 must-haves automation-verified; 1 pending human browser-smoke
behavior_unverified: 1
overrides_applied: 0
---

# Phase 27: Freiwillige in Abwesenheitsliste auswählbar (FE) — Verification Report

**Phase Goal:** Auf der Abwesenheitsseite sind aktive Freiwillige (`is_paid==false`) in beiden Personen-Selektoren (AbsenceModal + HR AbsenceFilterBar) auswählbar, sichtbar getrennt von Angestellten über native `optgroup`-Gruppierung (VOL-SEL-01).
**Verified:** 2026-06-29 (automated by orchestrator)
**Status:** HUMAN_NEEDED — all automated gates pass; one interactive browser-smoke deferred to human-UAT (user chose to continue to Phase 28; phases are independent).

---

## Goal Achievement — Observable Truths

| # | Truth (must_have) | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC1 + D-27-01: active volunteers appear in a labeled "Freiwillige" optgroup BELOW "Angestellte" in the AbsenceModal dropdown | VERIFIED (code) / smoke pending | `absences.rs:142` `grouped_selectable` (employees-first), `:170` `grouped_person_options`, modal call-site `:1292`. Rendered-visual + create-path = browser smoke (pending). |
| 2 | SC2 + D-27-06: AbsenceFilterBar applies the same grouping; "Alle"-option preserved BEFORE the groups | VERIFIED | filter call-site `:1497` (`Uuid::parse_str(&person_value).ok()`); standalone "Alle"-option retained at `:1425`. |
| 3 | SC3 + D-27-02: inactive in NEITHER group; `is_selectable_employee` stays `is_paid && !inactive`; `selectable_balances` unchanged → HR `VacationPerPersonList` stays paid-only | VERIFIED | grep guard matches exactly once (`:116`); 4 `is_selectable_*` + 5 `selectable_balances_*` tests green & unchanged (incl. `selectable_unpaid_but_active_returns_false`). |
| 4 | SC4 + D-27-04: both group labels resolve in en/de/cs via the two new keys | VERIFIED | `Key::AbsenceGroupEmployees`/`AbsenceGroupVolunteers`; en "Employees/Volunteers", de (Locale::De) "Angestellte/Freiwillige", cs "Zaměstnanci/Dobrovolníci"; absence i18n coverage test extended + green. |
| 5 | SC5 + D-27-03: a group with zero active members renders NO optgroup | VERIFIED | tests `grouped_selectable_omits_empty_volunteers_group` + `..._omits_empty_employees_group` pass; `grouped_selectable` only pushes non-empty groups. |
| 6 | D-27-05: category dropdown unchanged — volunteers get same categories as employees | VERIFIED | `:1229` category dropdown untouched; `SICK_LEAVE_ENABLED=false` unchanged. |

**Score:** 5/6 fully automation-verified; truth #1's rendered-visual + create-path roundtrip is the single pending human item.

---

## Automated Gates (all PASS)

- **WASM build** (frontend devShell, provides rust-lld): `cargo build --target wasm32-unknown-unknown` → Finished, 46 pre-existing warnings, 0 errors.
- **Frontend test suite:** `cargo test` → **677 passed, 0 failed** (incl. 5 new `grouped_selectable_*` tests + extended i18n coverage + unchanged guard tests).
- **Clippy (soft, FE workspace excluded from CI clippy):** 207 warnings vs ~198 pre-existing baseline; **zero** warnings on the changed identifiers/files → no new lints introduced.
- **D-27-02 guard grep:** `is_selectable_employee` body intact (matches once).
- **TDD:** confirmed RED (missing symbols) → GREEN (5/5).

## Pending Human-UAT (deferred 2026-06-29 — user chose "continue to Phase 28")

Browser smoke (backend roundtrip; create-path ≠ edit-path) on `/absences` with at least one ACTIVE `is_paid==false` sales person in the dev DB:
1. AbsenceModal "Neu" → person dropdown shows a "Freiwillige" group (below "Angestellte") with the volunteer; select + pick category/range + SAVE → absence created (no error banner, appears in list).
2. HR AbsenceFilterBar → "Alle" first, then grouped "Angestellte"/"Freiwillige"; selecting the volunteer filters the list.
3. Inactive persons appear in neither group; an empty group renders no empty optgroup.
4. Locale de/cs → labels "Angestellte"/"Freiwillige" and "Zaměstnanci"/"Dobrovolníci".

**Resume:** `/gsd-verify-work 27` (or manual browser check).

---

*Verification by orchestrator (autonomous run), 2026-06-29. Automated gates authoritative; visual/roundtrip smoke deferred to human.*
