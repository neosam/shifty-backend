---
phase: 35-slot-einzelwoche-aenderung
verified: 2026-06-30T19:00:00Z
status: human_needed
score: 3/4 must-haves verified
behavior_unverified: 1
overrides_applied: 0
behavior_unverified_items:
  - truth: "SWO-01: Im Slot-Editor kann ein Shiftplanner explizit zwischen 'nur diese Woche' und 'ab dieser Woche' waehlen; 'nur diese Woche' wirkt ausschliesslich in der gewaaehlten KW."
    test: "Browser oeffnen, Slot-Editor in Edit-Modus oeffnen, Radio 'Nur diese Woche' waehlen, speichern"
    expected: "Nur die gewaeehlte KW wird geaendert; die KW davor und KW+1 zeigen Original-Werte; kein doppelter Slot oder verwaiste Buchung"
    why_human: "Dioxus WASM UI-Interaktion ist nicht zuverlaessig automatisierbar (D-25-06-Caveat, explizit in VALIDATION.md 'Manual-Only' eingetragen). SSR-Tests bestaetigen Rendering und Verdrahtung; den tatsaechlichen Klick + Backend-Persist + Datenbankresultat kann kein Cargo-Test abdecken."
human_verification:
  - test: "Browser-Smoke: Slot-Editor oeffnen (Edit-Modus), Radio 'Nur diese Woche' waehlen, speichern"
    expected: "Nur die gewaelhlte Kalenderwoche wird geaendert; KW-1 und KW+1 zeigen weiterhin Original-Werte; keine Doppelzaehlung in Reports/Balance."
    why_human: "Dioxus WASM UI-Interaktion nicht automatisierbar (D-25-06-Caveat). Bereits in 35-VALIDATION.md als 'Manual-Only' klassifiziert."
---

# Phase 35: Slot-Werte nur fuer eine Woche aendern — Verification Report

**Phasen-Ziel:** Ein Shiftplanner kann die Werte eines Slots (Kapazitaet/Zeiten) fuer genau eine Kalenderwoche als einmalige Ausnahme aendern, ohne die wiederkehrende Struktur ab dieser KW dauerhaft zu veraendern — atomar (alles in einer Transaktion, Rollback bei Fehler) und ohne Doppelzaehlung in Reports/Balance.

**Verifiziert:** 2026-06-30T19:00:00Z
**Status:** human_needed
**Re-Verification:** Nein — initiale Verifikation

---

## Ziel-Erreichung

### Beobachtbare Wahrheiten (Roadmap Success Criteria)

| # | Wahrheit | Status | Evidenz |
|---|----------|--------|---------|
| 1 | SWO-01: Im Slot-Editor explizite Wahl zwischen "nur diese Woche" / "ab dieser Woche"; "nur diese Woche" wirkt ausschliesslich in der gewaaehlten KW | PRESENT_BEHAVIOR_UNVERIFIED | Radiogruppe im Code + SSR-Tests bestaetigen Rendering und Verdrahtung; Runtime-Klick+Persist nicht automatisierbar (WASM) |
| 2 | SWO-02: Mechanik = Split+Re-Merge mit 3 Slot-Versionen (Seg1 bis KW-1 / Seg2 nur KW mit neuen Werten / Seg3 ab KW+1 mit Originalwerten); Buchungen KW→Seg2, KW+1→Seg3 | VERIFIED | `modify_slot_single_week` implementiert (service_impl/src/shiftplan_edit.rs:199-326); 7 D-35-05-Tests alle GRUEN; REST-Route PUT /shiftplan-edit/slot/{year}/{week}/single-week vorhanden |
| 3 | SWO-03: Gesamter Vorgang (Slot-Schnitte + Booking-Re-Points) laeuft in EINER Transaktion; bei Fehler vollstaendiger Rollback | VERIFIED | Genau 1x `use_transaction` (Zeile 208), genau 1x `commit` (Zeile 324) in `modify_slot_single_week`; `test_msw_rollback_no_commit_on_error` verifiziert dass kein Commit erfolgt bei Fehler (GRUEN) |
| 4 | SWO-04: Booking-Neuzuweisungen durch harte Tests abgesichert — nichts doppelt oder verwaist; Gate = shiftplan.edit | VERIFIED | 7 TDD-Tests (D-35-05) alle GRUEN: Partition KW26→Seg2 / KW27→Seg3, je-genau-einmal, Rollback (commit times 0), Erste-KW-Edge (delete statt update), unbegrenztes valid_to, keine Buchungen, Forbidden; `check_permission("shiftplan.edit")` als erster Aufruf nach use_transaction |

**Score:** 3/4 Wahrheiten VERIFIED (1 PRESENT_BEHAVIOR_UNVERIFIED — SWO-01 Browser-Runtime)

---

## Erforderliche Artefakte

| Artefakt | Erwartet | Status | Details |
|----------|----------|--------|---------|
| `service/src/shiftplan_edit.rs` | Trait-Methode `modify_slot_single_week` | VERIFIED | Zeile 62-69; vollstaendige Signatur inkl. `#[automock]`-Mock-Generierung |
| `service_impl/src/shiftplan_edit.rs` | Vollimplementierung 3-Segment-Split + Booking-Partition | VERIFIED | Zeilen 199-326; original_snapshot-Pattern vor Mutation; Seg2 geschlossen (Sonntag KW); Seg3 aus snapshot; Partition `calendar_week == change_week as i32` |
| `service_impl/src/test/shiftplan_edit.rs` | D-35-05 Test-Modul | VERIFIED | 7 Test-Funktionen (test_msw_*) ab Zeile 1574; alle 26 shiftplan_edit-Tests GRUEN |
| `rest/src/shiftplan_edit.rs` | Handler `edit_slot_single_week` + Route | VERIFIED | Handler Zeilen 74-95; Route in `generate_route` Zeilen 23-26; kein `#[utoipa::path]` (konsistent mit `edit_slot`) |
| `shifty-dioxus/src/state/slot_edit.rs` | `SlotEdit.single_week: bool` Feld | VERIFIED | Zeile 103 (`pub single_week: bool`); Default `false` in `new_edit()` Zeile 115 |
| `shifty-dioxus/src/i18n/mod.rs` | 4 i18n-Key-Varianten + Presence-Test | VERIFIED | Zeilen 228-231 (Enum-Varianten); `i18n_slot_edit_mode_keys_present_in_all_locales`-Test vorhanden und GRUEN |
| `shifty-dioxus/src/i18n/{de,en,cs}.rs` | 4 Uebersetzungen pro Locale | VERIFIED | Alle 12 Match-Arme geprueft; de: "Geltungsbereich"/"Ab dieser Woche (Standard)"/"Nur diese Woche" + Hint; en/cs: entsprechende Texte; {week}/{year}-Platzhalter im Hint |
| `shifty-dioxus/src/api.rs` | `update_slot_single_week` | VERIFIED | Zeile 167; URL `.../single-week` |
| `shifty-dioxus/src/loader.rs` | `save_slot_single_week` | VERIFIED | Zeile 715; delegiert an `api::update_slot_single_week` |
| `shifty-dioxus/src/service/slot_edit.rs` | `SetSingleWeek(bool)` + Routing + Resets | VERIFIED | Action Zeile 26; save_slot_edit-Routing Zeilen 58-65; Resets in new_slot_edit (Zeile 44) und load_slot_edit (Zeile 109) |
| `shifty-dioxus/src/component/slot_edit.rs` | Radiogruppe im Edit-Modus + Hinweis-Absatz | VERIFIED | Props Zeilen 35+40; Radiogruppe in RSX Zeilen 176-196; Wrapper-Verdrahtung Zeilen 333+337; SSR-Tests GRUEN |

---

## Schluessel-Verknuepfungen (Key Links)

| Von | Nach | Via | Status | Details |
|-----|------|-----|--------|---------|
| REST-Route PUT /shiftplan-edit/slot/{year}/{week}/single-week | ShiftplanEditService::modify_slot_single_week | `generate_route` in rest/src/shiftplan_edit.rs | VERIFIED | Handler `edit_slot_single_week` Zeilen 74-95 |
| `modify_slot_single_week` | SlotService + BookingService | `tx.clone()` in einer Transaktion | VERIFIED | Alle Service-Aufrufe in Zeilen 208-324 innerhalb derselben tx |
| Radio onchange | SLOT_EDIT_STORE.single_week | `on_set_single_week` → `SlotEditAction::SetSingleWeek` → `set_single_week` | VERIFIED | Kette in slot_edit.rs (component) + service/slot_edit.rs |
| save_slot_edit | loader::save_slot_single_week | `if single_week { ... }` | VERIFIED | Verzweigung in service/slot_edit.rs Zeilen 59-65 |
| api::update_slot_single_week | PUT .../single-week Backend-Route | `format!("{}/shiftplan-edit/slot/{}/{}/single-week", ...)` | VERIFIED | api.rs Zeile 174 |

---

## Behavioristische Stichproben (Spot-Checks)

| Verhalten | Kommando | Ergebnis | Status |
|-----------|----------|----------|--------|
| 7 D-35-05-Tests (modify_slot_single_week) | `SQLX_OFFLINE=true cargo test -p service_impl shiftplan_edit` | 26/26 GRUEN | PASS |
| 3 neue SSR-Tests + i18n-Presence-Test + Legacy-Guard | `cargo test -p shifty-dioxus slot_edit` | 17/17 GRUEN | PASS |
| test_msw_rollback_no_commit_on_error | Enthalten in cargo test oben | GRUEN (kein Panic = kein unerwarteter commit) | PASS |
| test_msw_booking_partition_and_each_exactly_once | Enthalten in cargo test oben | GRUEN (KW26→Seg2, KW27→Seg3, je genau einmal) | PASS |
| test_msw_forbidden | Enthalten in cargo test oben | GRUEN | PASS |
| Kein Snapshot-Schema-Bump | `grep CURRENT_SNAPSHOT_SCHEMA_VERSION service_impl/src/billing_period_report.rs` | Wert = 12 | PASS |
| Keine neuen SQL-Queries | `grep sqlx::query service_impl/src/shiftplan_edit.rs` | 0 Treffer | PASS |

---

## Requirements-Coverage

| Requirement | Plan | Beschreibung | Status | Evidenz |
|-------------|------|-------------|--------|---------|
| SWO-01 | 35-02, 35-03 | Explizite Modus-Wahl im Editor | PRESENT_BEHAVIOR_UNVERIFIED | Radiogruppe im Code + SSR-Tests; Browser-Runtime per D-25-06-Caveat manuell |
| SWO-02 | 35-01 | 3-Segment-Split-Mechanik | VERIFIED | Implementierung + 7 Tests GRUEN |
| SWO-03 | 35-01 | Atomaritaet (1 Transaktion) | VERIFIED | Genau 1 commit; Rollback-Test GRUEN |
| SWO-04 | 35-01, 35-02 | Harte Tests + Gate shiftplan.edit + i18n de/en/cs | VERIFIED | 7 D-35-05-Tests GRUEN; i18n-Presence-Test GRUEN |

---

## Anti-Pattern-Scan

| Datei | Befund | Schwere | Auswirkung |
|-------|--------|---------|------------|
| `shifty-dioxus/src/api.rs` Zeile 1504 | `// TODO: Find a better way to convert serde error to reqwest error` | INFO | Pre-existing, nicht von Phase 35 eingefuehrt; kein Bezug zu single-week-Funktionalitaet |

Kein TBD, FIXME, XXX oder `todo!()` in den von Phase 35 modifizierten Bereichen der Implementierung. Keine Stubs. Keine leeren Implementierungen.

---

## Menschliche Verifikation erforderlich

### 1. Browser-Smoke: Modus-Radiogruppe End-to-End

**Test:** Backend starten (Port 3000), Frontend starten (Port 8080), Schichtplan oeffnen, Slot in Edit-Modus oeffnen, Radio "Nur diese Woche" waehlen, speichern.

**Erwartet:** Nur die gewaaehlte Kalenderwoche wird geaendert; die Schichtplanstruktur in der KW davor und in KW+1 zeigt die Original-Werte (3 Segmente im DB); Buchungen der Ausnahme-KW verbleiben auf Segment 2, spaetere Buchungen auf Segment 3; Reports/Balance zeigen keine Doppelzaehlung.

**Warum manuell:** Dioxus-WASM-Interaktion (Klick auf Radio-Button triggert Dioxus-Signal, Submit-Button, Netzwerk-Request an PUT .../single-week, DB-Persistenz) ist per D-25-06-Caveat nicht automatisierbar. SSR-Tests bestaetigen das Rendering; Code-Inspektion bestaetigt die Verdrahtungskette; das tatsaechliche Laufzeit-Ergebnis (Klick → Netzwerk → DB → korrekte KW-Aenderung) erfordert einen manuellen Browser-Test.

---

## Lueckenzusammenfassung

Keine Luecken — alle Artefakte existieren, sind substanziell und verdrahtet. Einziger offener Punkt ist die Browser-E2E-Verifikation von SWO-01, die per Phase-Planungs-Entscheidung (VALIDATION.md "Manual-Only") explizit als manuell klassifiziert ist.

---

_Verifiziert: 2026-06-30T19:00:00Z_
_Verifikator: Claude (gsd-verifier)_

---

## Gap-Closure (WR-01)

**Gefunden:** Code-Review-Finding WR-01 — `modify_slot_single_week` erstellte Segment 3
bedingungslos, auch wenn `seg3_valid_from > original_valid_to` (bounded Slot, Ausnahme-KW =
letzte Gültigkeitswoche). Im echten `SlotService::create_slot` führte das zu `DateOrderWrong`
und einem vollständigen Rollback des Vorgangs (kein Datenverlust, aber ein legitimer
Schichtplan-Edit schlug fehl).

**Ursache:** Segment-3-Erstellung hatte keinen symmetrischen Guard zur Erste-KW-Edge-Prüfung
von Segment 1. Alle 7 D-35-05-Tests verwendeten `valid_to: None` (unbegrenzter Slot) und
deckten den bounded-last-week-Fall nicht ab.

**Fix (commit 21ad88d, plan 35-04):**
- `seg2.valid_to` wird auf `min(seg2_valid_to, original_valid_to)` geklemmt — Seg2 überschreitet
  nie das ursprüngliche Slot-Ende bei bounded Slots.
- Segment 3 wird nur erstellt wenn `original_valid_to.is_none_or(|vt| seg3_valid_from <= vt)`.
  `seg3_slot_id` ist `Option<Uuid>`; im Booking-Re-Point-Loop stellt eine Invariante
  (`expect("post-exception Buchung impliziert, dass Segment 3 existiert")`) sicher, dass
  post-exception Buchungen nur erreichbar sind, wenn Seg3 tatsächlich erstellt wurde.

**Test (commit f43f0b9, plan 35-04):**
`test_msw_last_week_of_bounded_slot_no_date_order_error` — bounded Slot (`valid_to=2026-06-28`,
Sonntag der Ausnahme-KW 26), `create_slot` `.times(1)` im Mock beweist, dass Seg3 übersprungen
wird. RED (vor Fix): mockall-Panic "called 2 times which is more than the expected 1".
GREEN (nach Fix): 27/27 shiftplan_edit-Tests grün, `cargo clippy --workspace -D warnings` sauber.
