# Phase 28 — Seed: Urlaubsanspruch-Korrektur via Offset (HR)

**Milestone**: v1.8 · **Requirement**: VAC-OFFSET-01 · **Typ**: Backend + Frontend
**Erstellt**: 2026-06-29

## Problem / Motivation

Shifty berechnet den Jahres-Urlaubsanspruch aliquot (Proration) und rundet das
Ergebnis. Dabei kommt es im Einzelfall auf einen fachlich „falschen" Wert
(z.B. **17 statt 18**). HR braucht einen Mechanismus, um den **Gesamturlaub**
auf den korrekten Wert zu bringen.

**Entscheidung (User, 2026-06-29)**: Es soll ein **Offset (Korrektur-Delta)**
sein, KEIN absoluter Override. Begründung: Bei späteren Vertragsänderungen soll
sich der Anspruch weiter neu berechnen und der Offset „mitwandern" (berechnet + Offset),
statt eingefroren auf einem Absolutwert zu stehen.

**Sichtbarkeit (User, 2026-06-29)**: In der **HR-Ansicht gekennzeichnet** und
editierbar; für **normale User unsichtbar** (nur die finale Zahl).

## Wie der Anspruch heute berechnet wird (verifiziert 2026-06-29)

`service_impl/src/vacation_balance.rs:186`:

```rust
let entitled_days: f32 = work_details
    .iter()
    .map(|wd| wd.vacation_days_for_year(year))  // aliquot pro Vertragszeile
    .sum::<f32>()
    .round();                                     // ← finale Rundung
```

`EmployeeWorkDetails::vacation_days_for_year` (`service/src/employee_work_details.rs:158`)
zieht bei unterjährigem Vertragsstart/-ende den anteiligen Teil von
`vacation_days` (u8) ab. Mehrere Vertragszeilen pro Person/Jahr werden summiert.

`remaining_days = entitled_days + carryover_days − (used_days + planned_days)`
(`vacation_balance.rs:257`).

**Off-by-one entdeckt** (`employee_work_details.rs:173`): Für Start am 1.1. ist
`from_date.ordinal() == 1` statt 0 → es wird `vacation_days * (1/365) ≈ 0,05`
zu viel abgezogen. Allein kippt das die Rundung selten; das „17 statt 18"
entsteht eher durch reguläre unterjährige Proration. → Begleit-Fix optional.

## Zielmechanik

```
entitled_effective = round(sum(vacation_days_for_year)) + offset
```

- `offset` ist **signed** (±), pro **Person + Jahr**.
- Delta-Verhalten: überlebt Vertragsänderungen.
- Wirkt automatisch auf `remaining_days` durch.

## Skizze der Umsetzung

**Datenmodell** (neu): Tabelle `vacation_entitlement_offset`
(`id`, `sales_person_id`, `year`, `offset_days` INTEGER signed, `version`,
`created`, `deleted`). Soft-delete-Konvention wie übrige Tabellen.
→ DAO-Trait + `dao_impl_sqlite` + Migration (`migrations/sqlite/`).

**Backend**:
- Service zum Lesen/Setzen des Offsets (HR-gated, `HR_PRIVILEGE`).
- `vacation_balance`-Berechnung um den Offset erweitern (nach `.round()`).
- REST-Endpoint(s) HR-gated (analog `rest/src/vacation_balance.rs` /
  `employee_work_details.rs`).
- `#[utoipa::path]` + ToSchema für neue DTOs.

**Frontend** (`shifty-dioxus`) — UX entschieden 2026-06-29:
- **Platzierung**: Personen-Detailansicht `VacationEntitlementSelfBody`
  (`page/absences.rs:415–481`), konkret an der **„Vertragsanspruch"-StatBox**
  (`VacationStatContract`, Wert = `entitled_days`). HR gelangt dorthin per Klick
  auf eine Person in `VacationPerPersonList` (`forced_self`-Pfad, `absences.rs:358–363`).
- **Edit-Control = Inline-Zahlenfeld** (kein Modal, kein Popover): in der HR-Ansicht
  immer sichtbar, **signed** Offset, Beschriftung „berechnet {n} + Offset [x]";
  die Box-/Hero-Zahl zeigt den **Effektivwert**. Speichern on-blur/Enter, HR-gated.
- `VacationEntitlementSelfBody` braucht dafür ein durchgereichtes **`is_hr`-Flag**
  (aktuell nur in `VacationEntitlementCard` via `props.is_hr` vorhanden) — bestimmt,
  ob das Offset-Feld + „berechnet/Offset"-Zeile rendern.
- **User-Ansicht**: dieselbe StatBox zeigt nur den Effektivwert (kein Feld, keine
  „berechnet/Offset"-Zeile).
- **Optional**: kleiner Indikator an Personen mit Offset in der kompakten
  `VacationPerPersonList` (`PersonVacationCard`, `absences.rs:672–727`); editiert wird nur im Detail.
- **Datenfluss**: Offset (und idealerweise der berechnete Basiswert vor Offset) muss
  in die HR-Ansicht geladen werden — entweder `VacationBalanceTO` um `offset_days`
  (+ ggf. `computed_entitled`) erweitern oder separater Load. Self-Pfad: siehe offener Punkt 1.
- i18n de/en/cs für neue Labels („berechnet", „Offset", …).

## Offene Punkte für die Planung

1. **„Für User unsichtbar" — UI-only oder API-level?** Self-Endpoint ist
   „HR ∨ self"; ein User könnte den Offset sonst in der rohen API-Antwort sehen.
   Empfehlung: im Self-Pfad serverseitig nicht ausliefern (sauberer als nur UI).
2. **Off-by-one-Begleit-Fix** (`employee_work_details.rs:173`) mitnehmen? Wenn ja,
   Regressionstests für Voll- und Teiljahr-Verträge.
3. **Snapshot-Bump**: Urlaub ist vermutlich kein billing `value_type`
   (`billing_period_sales_person`) → kein `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump.
   Bei Planung verifizieren.
4. **Mehrjahres-UX**: Offset ist jahresbezogen — Edit immer im Kontext des in der
   Übersicht gewählten Jahres.

## Gates (Definition of Done)

- Backend: `cargo build` + `cargo test --workspace` + `cargo clippy --workspace -- -D warnings`
- Frontend: `cargo build --target wasm32-unknown-unknown` (Backend-Shell wg. lld) + `cargo test`
- Migration additiv anwenden (`sqlx migrate run`, NICHT `reset`).
- Browser-Roundtrip: HR setzt Offset → finale Zahl in HR- UND User-Ansicht korrekt; Marker nur in HR-Ansicht.
