---
phase: 51
milestone: v2.4
type: plan_index
created: 2026-07-04
plans: 8
---

# Phase 51 — Kurzer-Tag-Slot-Kürzung — Plan Index

## Requirement → Plan Matrix

| Requirement | P01 | P02 | P03 | P04 | P05 | P06 | P07 | P08 |
|---|---|---|---|---|---|---|---|---|
| **SHC-01** — kanonische Clip-Fn + 4 Tests | ✅ primary | | | | | | | |
| **SHC-02** — Reporting/Ist-Stunden clippen | | | ✅ (Chain B) | ✅ (Chain A') | ✅ (Chain C) | ✅ primary (Chain D) | | |
| **SHC-03** — FE WeekView zeigt geclippt | | | | | | | ✅ primary | |
| **SHC-04** — PDF konsistent zu WeekView | | | ✅ (auto via Chain B) | | | | ✅ verifikation | |
| **SHC-05** — future ShortDay auf existing bookings, no rewrite | | | ✅ | ✅ | ✅ | ✅ | | |
| **SHC-06** — admin-stichtag `shortday_slot_clipping_active_from` | | ✅ primary (BE) | ✅ | ✅ | ✅ | ✅ | | ✅ primary (FE) |

Jede der sechs Requirements ist von mindestens einem Plan abgedeckt. SHC-04 fällt aus SHC-03 automatisch — PDF-Renderer konsumiert dasselbe geclippte Aggregat (Chain B); P07 verifiziert das per Test, ohne Renderer-Code zu ändern.

## Decision → Plan Matrix (D-51-01..09)

| Decision | Plan | Notes |
|---|---|---|
| **D-51-01** — `Slot::clip_to` in `service/src/slot.rs` | P01 | Method + 4 unit tests |
| **D-51-02** — Fat Backend: DTO liefert geclippt | P07 | `effective_to`-Feld + Mapper |
| **D-51-03** — Booking-Create nicht abgelehnt | P05 (implicit) | Nichts hinzugefügt an ShiftplanEditService; Verify-Test in P05 |
| **D-51-04** — keine visuelle Zusatz-Markierung | P07 | Verify-Test: FE rendert nur Länge, kein Extra-Attribut |
| **D-51-05** — iCal via BlockService, nicht separat | P04 | Automatisch mit Chain A' |
| **D-51-06** — vier BE-Aggregat-Ketten | P03/P04/P05/P06 | Chain B / A' / C / D |
| **D-51-07** — Toggle `shortday_slot_clipping_active_from` | P02 | Toggle seed + Rust-Helper `shortday_clip_gate` |
| **D-51-08** — Chain D: Rust-Layer Clipping (kein SQL-JOIN) | P06 | DAO returnt raw slots, Rust-Layer aggregiert |
| **D-51-09** — `effective_to` am `ShiftplanSlotTO`-Wrapper | P07 | Feld am Wrapper, nicht am `SlotTO` |

Jede Decision ist per Konvention als Tag in `must_haves.truths` des zuständigen Plans (Decision-Coverage-Gate `check.decision-coverage-plan`).

## Wave Order

- **Wave 1** (parallel): P01, P02 — Foundation
- **Wave 2** (parallel): P03, P04, P05, P06 — BE-Aggregat-Ketten (kein `files_modified`-Overlap zwischen den vier Ketten)
- **Wave 3**: P07 — DTO/FE/PDF-Konsum (hängt an P03: `From<&ShiftplanSlot> for ShiftplanSlotTO` konsumiert `effective_to`)
- **Wave 4**: P08 — Admin Settings UI (hängt an P02: Toggle muss seed sein)

## Non-Goals (explicit)

- **Kein Snapshot-Bump.** `CURRENT_SNAPSHOT_SCHEMA_VERSION` in `service_impl/src/billing_period_report.rs:117` bleibt **12**. Chain-D-Refactor (P06) ändert nur Live-Berechnung, kein persistiertes `BillingPeriodValueType`. Snapshot-Immunität ist zusätzlich durch das Stichtag-Gate abgesichert (D-51-07 + D-03).
- **Kein Booking-Rewrite / Cascade-Warning** (D-51-03).
- **Kein Soll-Stunden-Impact** (D-05).
- **Kein neuer Cargo-Dep.**
- **Keine i18n-Keys außer 3 neuen für P08** (Label/Description/UnsetHint — analog HCFG-02).
- **Kein FE-Clipping** (D-51-02).

## Files touched (union across plans, for `files_modified`-Overlap sanity)

- **P01:** `service/src/slot.rs`
- **P02:** `migrations/sqlite/20260704000000_seed-shortday-slot-clipping-toggle.sql` (neu), `service_impl/src/lib.rs` (Helper-Modul-Registrierung), `service_impl/src/shortday_gate.rs` (neu)
- **P03:** `service_impl/src/shiftplan.rs`, `service_impl/src/test/shiftplan.rs`
- **P04:** `service_impl/src/block.rs`, `service_impl/src/test/block.rs`
- **P05:** `service_impl/src/booking_information.rs`, `service_impl/src/test/booking_information.rs`
- **P06:** `dao/src/shiftplan_report.rs`, `dao_impl_sqlite/src/shiftplan_report.rs`, `service_impl/src/shiftplan_report.rs`, `service_impl/src/test/shiftplan_report.rs` (neu falls nicht vorhanden), ggf. `.sqlx/`
- **P07:** `rest-types/src/lib.rs`, `service_impl/src/shiftplan.rs` (Chain-B-Konsumenten setzen `effective_to`; conflict mit P03 → P07 hängt sequentiell an P03), `shifty-dioxus/src/loader.rs`, `service_impl/src/test/pdf_render.rs` (neue Assertion), `service_impl/src/test/shiftplan.rs` (Assertion)
- **P08:** `shifty-dioxus/src/page/settings.rs`, `shifty-dioxus/src/i18n/mod.rs`, `shifty-dioxus/src/i18n/de.rs`, `shifty-dioxus/src/i18n/en.rs`, `shifty-dioxus/src/i18n/cs.rs`

**File-overlap check:**
- Wave 2 (P03/P04/P05/P06): kein Overlap. P03 → `service_impl/src/shiftplan.rs`, P04 → `service_impl/src/block.rs`, P05 → `service_impl/src/booking_information.rs`, P06 → `dao_impl_sqlite/src/shiftplan_report.rs` + `service_impl/src/shiftplan_report.rs`. Alle disjoint.
- P07 überlappt mit P03 auf `service_impl/src/shiftplan.rs` → P07 depends_on `51-03` (Wave 3).
- P08 überlappt mit P02 auf keiner Datei, aber depends_on `51-02` semantisch (Toggle muss existieren) → Wave 4.

## Per-Plan Gates (jedes Plan-Verify)

- `cargo test --workspace` grün
- `cargo clippy --workspace -- -D warnings` grün (Clippy ist Pflicht-Gate, siehe `feedback_clippy_gate` in Auto-Memory)
- Bei neuen `query!`/`query_as!` (nur P06): `cargo sqlx prepare --workspace` + `.sqlx/` committen
- Bei FE-Änderung (P07, P08): `cargo build --target wasm32-unknown-unknown` + `cargo clippy -p shifty-dioxus -- -D warnings` + `cargo test -p shifty-dioxus` — alle grün

## Executor-Notiz

Executor arbeitet auf Opus (voller Kontext). Plans zitieren Datei:Zeile — bitte nicht per Grep neu suchen, sondern die referenzierten Zeilen direkt lesen. Bug-Fix-Sites in P03/P05 sind *keine* Feature-Debatte, sondern D-04-Compliance.

## PLAN-CHECK Warnings (aus `51-PLAN-CHECK.md`, non-blocking, Executor-relevant)

1. **`shifty_bin/src/main.rs` ist eine implizit geteilte Datei über P03/P04/P05/P06.** Jeder der vier Wave-2-Pläne fügt einen neuen Dep-Parameter (`ToggleService`, teils zusätzlich `SpecialDayService`) an einer bestehenden `ServiceImpl::new(...)`-Konstruktion in `shifty_bin/src/main.rs`. Die Zeilen sind pro Service unterschiedlich (kein True-Konflikt), aber die Datei taucht bewusst NICHT in den `files_modified` der Pläne auf. **Empfehlung:** Wave 2 sequentiell (nicht parallel) ausführen, um jj/git-Merge-Konflikte zu vermeiden — oder falls parallel, den DI-Wire-Fix als kleinen Extra-Commit nach jedem Plan machen. Reihenfolge intern egal (P03→P04→P05→P06 oder beliebig andere).

2. **P06 Task 5: Delete-Branch bevorzugen.** Die pre-existing SQL-Bugs bei `dao_impl_sqlite/src/shiftplan_report.rs:114, 147` (`fehlt /60.0` bei Minute-Teil) verschwinden nur, wenn die alten SUM-Queries gelöscht werden. Task 5 hat zwei Branches (delete vs. deprecate) — der Executor soll **delete** wählen, sofern keine externen Konsumenten außerhalb der ServiceImpl-Methoden nachweisbar sind.
