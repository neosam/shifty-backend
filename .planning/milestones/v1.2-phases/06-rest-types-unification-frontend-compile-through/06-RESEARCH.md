# Phase 6: rest-types Unification & Frontend Compile-Through — Research

**Researched:** 2026-05-07
**Domain:** Cargo-Workspace-Topology, Rust/WASM-Dependency-Management, Dioxus-Frontend-TO-Migration
**Confidence:** HIGH (alle kritischen Fakten direkt aus Sourcecode und Cargo-Manifesten verifiziert)

---

<phase_requirements>
## Phase Requirements

| ID | Beschreibung | Research Support |
|----|--------------|-----------------|
| RT-01 | `shifty-dioxus/Cargo.toml` deklariert `rest-types = { path = "../rest-types", default-features = false }` | Cargo-Topology-Analyse bestätigt: Backend-`rest-types/Cargo.toml` hat feature `service-impl`; `default = []` ist bereits gesetzt. Der Swap ist mechanisch einfach — ABER es gibt eine Blocking-Landmine (siehe §6). |
| RT-02 | `shifty-dioxus/rest-types/`-Verzeichnis existiert nicht mehr | Einfache `rm -rf`-Operation nach dem Cargo-Swap; Git-Track entfernen. |
| RT-03 | Alle 17 fehlenden Structs/Enums + 4 fehlenden Felder sind aus Frontend-Code referenzierbar | Jeder der 17 Types ist im Backend-`rest-types/src/lib.rs` vorhanden (verifiziert per Grep). AUSNAHME: `InvitationStatus`/`InvitationResponse`/`GenerateInvitationRequest` — diese leben in `rest/src/user_invitation.rs`, NICHT in `rest-types`. Muss vor dem Swap in `rest-types` wandern. |
| FC-01 | Match-Arme erschöpfend für alle Backend-Enums | Die kritischen Panics in `state/employee.rs:89/151` und `state/shiftplan.rs:59` müssen adressiert werden. `ExtraHoursCategoryTO` und `ExtraHoursReportCategoryTO` im Frontend sind tatsächlich bereits vollständig — nur die `From<>` Impls im alten Fork hatten Lücken. |
| FC-02 | `cargo build --target wasm32-unknown-unknown` grün | Requires nix develop-Shell; WASM-Toolchain nur via `flake.nix` verfügbar. |
</phase_requirements>

---

## Summary

Phase 6 ist eine mechanische Konsolidierung: Backend-`rest-types` ersetzt den Frontend-Fork `shifty-dioxus/rest-types/`. Die Grundstruktur ist klar — swap Cargo-Dep, delete Fork, fix Compile-Errors. Die Komplexität liegt in drei spezifischen Problemen.

**Problem 1: Blocking-Landmine — `shifty_utils` unconditional import.** Das Backend-`rest-types/src/lib.rs` hat auf Zeile 8 `use shifty_utils::{derive_from_reference, LazyLoad};` ohne Feature-Gate. `shifty_utils` ist eine eigene Crate im Backend-Workspace. Ohne Anpassung würde `default-features = false` NICHT ausreichen — `shifty_utils` wäre trotzdem required. Verifizierung: `LazyLoad::new()` wird auf Zeile 745 innerhalb eines `#[cfg(feature = "service-impl")]`-Blocks verwendet und `derive_from_reference!()` nur in `service-impl`-gated Blöcken. Der `use shifty_utils::...`-Import selbst ist NICHT gated. Lösung: Den Import mit `#[cfg(feature = "service-impl")]` gaten, ODER `shifty_utils` als optionale Dependency in `rest-types/Cargo.toml` deklarieren.

**Problem 2: `InvitationStatus`/`InvitationResponse`/`GenerateInvitationRequest` fehlen im Backend-`rest-types`.** Diese Types leben in `rest/src/user_invitation.rs` (Backend-REST-Layer), NICHT in `rest-types/src/lib.rs`. Das Frontend-Fork hat sie lokal definiert. Nach dem Swap kompiliert der Frontend-Code `use rest_types::InvitationResponse` nicht mehr. Lösung: Diese Types in `rest-types/src/lib.rs` verschieben (Wave 0 oder Wave 1 des Cargo-Swap-Plans).

**Problem 3: `ShiftplanTO` derive-Diskrepanz.** Backend: `#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]` (kein `PartialEq`/`Eq`). Frontend-Fork: `#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]`. `PartialEq`+`Eq` auf `ShiftplanTO` wird vom Frontend benötigt (Dioxus-Reaktivität). Lösung: `PartialEq, Eq` zum Backend-`ShiftplanTO` hinzufügen; prüfen welche anderen TOs im Frontend-Fork zusätzliche Derives haben.

**Primary recommendation:** Wave 1 umfasst zwei Plans: (1a) Backend-`rest-types` patchen (`shifty_utils` gaten, `InvitationStatus`-Familie migrieren, fehlende Derives hinzufügen), (1b) Cargo-Swap + Fork-Delete im Frontend. Wave 1 muss sequenziell sein: 1a vor 1b.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| DTO-Definitionen (Serde-Shape) | Backend `rest-types` Crate | — | Single Source of Truth nach RT-01/02 |
| State-Mapping (TO → state::*) | Frontend `src/state/*.rs` | `src/loader.rs` | `From<&*TO>`-Impls in state, Orchestrierung in loader |
| API-Wrapper (HTTP-Calls) | Frontend `src/api.rs` | — | Einzige Stelle für `reqwest`-Calls |
| WASM-Compile-Gate | Frontend Build (`nix develop`) | — | `cargo build --target wasm32-unknown-unknown` |
| Feature-Flag-Gating für Backend-Services | `rest-types` `[features]`-Table | — | `service-impl` Feature isoliert WASM-inkompatible Deps |

---

## 1. Drift-Inventur Mapping

Vollständige Tabelle aller fehlenden TOs/Felder mit Fundort im Backend und Konsumenten im Frontend.

### 1a. Fehlende TO-Structs/-Enums (17 gesamt — alle in Backend-`rest-types/src/lib.rs`)

| TO-Name | Backend-Pfad (Zeile) | Frontend-Konsumenten | Risiko/Komplexität |
|---------|----------------------|----------------------|-------------------|
| `ShiftplanAssignmentTO` | `rest-types/src/lib.rs:1487-1496` | `src/api.rs` (NICHT importiert — `src/state/user_management.rs:8-11` hat eigene lokale `ShiftplanAssignment`-struct die SERDE-kompatibel ist) | NIEDRIG — frontend-Struct ist wire-kompatibel mit Backend-TO (gleiche Felder, gleiche defaults). Nur Import-Path wechselt. |
| `ToggleTO` | `rest-types/src/lib.rs:1498-1526` | `src/api.rs` (kein Import gefunden — Toggle-Endpoints könnten noch nicht klientenseitig gebunden sein) | NIEDRIG — kein aktiver Konsument im Frontend-Code erkennbar |
| `ToggleGroupTO` | `rest-types/src/lib.rs:1528-1553` | Wie ToggleTO | NIEDRIG |
| `ImpersonateTO` | `rest-types/src/lib.rs:1555-1560` | `src/api.rs` (kein Import gefunden) | NIEDRIG |
| `AbsenceCategoryTO` | `rest-types/src/lib.rs:1566-1592` | Wird von `WarningTO` und `UnavailabilityMarkerTO` referenziert (transitiv) | MITTEL — wird als Feld in anderen TOs benutzt |
| `AbsencePeriodTO` | `rest-types/src/lib.rs:1594-1646` | `src/api.rs` (keine direkte Nutzung — Absence-Endpoints nicht im Frontend gebunden) | MITTEL — AbsencePeriodCreateResultTO benötigt es |
| `WarningTO` (5 Varianten) | `rest-types/src/lib.rs:1670-1781` | `src/api.rs` (via `BookingCreateResultTO`, `CopyWeekResultTO`, `AbsencePeriodCreateResultTO`) | HOCH — erschöpfende Match-Arme in 3 Result-Wrapper-Consumern; UI-SPEC Regel 1 anwenden |
| `UnavailabilityMarkerTO` (3 Varianten) | `rest-types/src/lib.rs:1785-1823` | `src/loader.rs` (`load_shift_plan` ignoriert `unavailable`-Feld derzeit; nach RT-01 müssen `ShiftplanDayTO.unavailable`-Felder durchgereicht werden) | MITTEL — Feld spiegeln, kein RSX nötig |
| `BookingCreateResultTO` | `rest-types/src/lib.rs:1825-1840` | `src/api.rs:add_booking()` gibt derzeit `Result<(), reqwest::Error>` zurück — Backend antwortet aber mit `BookingCreateResultTO`. Wrapper-Shape-Anpassung nötig. | HOCH — Signaturen von `api::add_booking` müssen angepasst werden; `warnings`-Feld ignorieren (no-op, UI-SPEC) |
| `CopyWeekResultTO` | `rest-types/src/lib.rs:1842-1857` | `src/api.rs:copy_week()` gibt `Result<(), reqwest::Error>` zurück — analog zu BookingCreateResultTO | MITTEL |
| `AbsencePeriodCreateResultTO` | `rest-types/src/lib.rs:1859-1875` | `src/api.rs` (Absence-Endpoints nicht gebunden) | NIEDRIG |
| `CutoverGateDriftRowTO` | `rest-types/src/lib.rs:1885-1896` | `src/api.rs` (kein Konsument gefunden) | NIEDRIG — Cutover-UI fehlt im Frontend |
| `CutoverGateDriftReportTO` | `rest-types/src/lib.rs:1898-1908` | Wie oben | NIEDRIG |
| `CutoverRunResultTO` | `rest-types/src/lib.rs:1910-1922` | Wie oben | NIEDRIG |
| `CutoverProfileBucketTO` | `rest-types/src/lib.rs:1937-1949` | Wie oben | NIEDRIG |
| `CutoverProfileTO` | `rest-types/src/lib.rs:1954-1963` | Wie oben | NIEDRIG |
| `ExtraHoursCategoryDeprecatedErrorTO` | `rest-types/src/lib.rs:1926-1934` | `src/api.rs` (kein Konsument gefunden) | NIEDRIG |

**Wichtige Beobachtung: InvitationStatus-Familie fehlt im Backend-rest-types**

`InvitationStatus`, `InvitationResponse`, `GenerateInvitationRequest` sind **NICHT** in `rest-types/src/lib.rs` definiert. Sie leben in `rest/src/user_invitation.rs` (Backend-REST-Layer). [VERIFIED: direkter Grep auf beide Dateien]

Das Frontend-Fork hat diese Types selbst definiert. Beim Swap werden sie unter `rest_types::InvitationResponse` erwartet (importiert in `src/api.rs:7` und `src/service/user_management.rs:7`). Diese Types MÜSSEN in `rest-types/src/lib.rs` wandern, BEVOR der Swap stattfindet.

### 1b. Fehlende Felder auf existierenden TOs (4 Felder)

| Backend-TO | Fehlendes Feld | Backend-Zeile | Frontend-Konsument | Aktion |
|-----------|---------------|--------------|-------------------|--------|
| `SlotTO` | `max_paid_employees: Option<u8>` | `rest-types/src/lib.rs:321` | `src/loader.rs:load_shift_plan()` konstruiert `Slot` manuell ohne dieses Feld | Feld zu `state::shiftplan::Slot` hinzufügen (`Option<u8>`, default `None`); in `From<&SlotTO>` mappen; NICHT rendern (UI-SPEC Regel 2) |
| `ShiftplanSlotTO` | `current_paid_count: u8` | `rest-types/src/lib.rs:986` | `src/loader.rs:load_shift_plan()` iteriert `shiftplan_week.days[].slots[]` — Feld wird derzeit nicht durchgereicht | Feld zu `state::shiftplan::Slot` (oder neuen state-Typ) hinzufügen; in loader mappen; NICHT rendern |
| `ShiftplanDayTO` | `unavailable: Option<UnavailabilityMarkerTO>` | `rest-types/src/lib.rs:997` | `src/loader.rs:load_shift_plan()` iteriert days — Feld wird ignoriert | State-Mirror-Feld hinzufügen; `UnavailabilityMarkerTO` muss importierbar sein |
| `BillingPeriodTO` | `snapshot_schema_version: u32` | Backend `rest-types/src/lib.rs:1312` | `src/service/billing_period.rs` / `src/loader.rs` — Billing-State-Struct `BillingPeriodTO` in Frontend-Fork fehlt das Feld | Feld zum Frontend-State-Mirror hinzufügen; pure diagnostic; NICHT rendern |

**ACHTUNG:** `BillingPeriodTO` in der Frontend-Fork (`rest-types/src/lib.rs:1226-1237`) hat kein `snapshot_schema_version`-Feld. Das Backend hat es auf Zeile 1312. [VERIFIED]

### 1c. ExtraHoursCategoryTO Match-Situation

Positiv-Befund: `ExtraHoursCategoryTO` und `ExtraHoursReportCategoryTO` im **aktuellen Frontend-Code** (`src/state/employee.rs`) sind VOLLSTÄNDIG exhaustive — beide `UnpaidLeave` und `VolunteerWork` sind in allen `From`-Impls gemappt (Zeilen 125-170). [VERIFIED: direkter Coderead]

Die CONCERNS.md §1.C-Warnung bezog sich auf den alten FORK (`rest-types/src/lib.rs:391-403`), nicht auf den Frontend-Hauptcode. Nach dem Swap ist diese Lücke irrelevant (Fork wird gelöscht).

### 1d. Vorhandene Typen, die NICHT fehlen (Gegenbeweis)

`PlanDayViewTO` und `ShiftplanDayAggregateTO` existieren sowohl im Backend (Zeilen 1052+) als auch im Frontend-Fork (Zeilen 950+). Diese brauchen keine Spezial-Behandlung.

---

## 2. Feature-Cluster-Identifikation

### Cluster A: Cargo-Swap-Blocker (Wave 0 / Plan 1a)

Muss ZUERST erledigt werden, bevor der Swap stattfinden kann. Betrifft Backend-Code.

| Problem | Betroffene Dateien | Aktion |
|---------|-------------------|--------|
| `shifty_utils` unconditional import | `rest-types/src/lib.rs:8` | Import mit `#[cfg(feature = "service-impl")]` gaten |
| `InvitationStatus`, `InvitationResponse`, `GenerateInvitationRequest` fehlen in `rest-types` | `rest/src/user_invitation.rs` | Types in `rest-types/src/lib.rs` verschieben; in `rest/` via `rest_types::` importieren |
| `ShiftplanTO` fehlen `PartialEq, Eq` | `rest-types/src/lib.rs:13` | Derives hinzufügen; prüfen welche weiteren TOs FE-Fork-extra-Derives haben |

### Cluster B: Cargo-Swap + Fork-Delete (Wave 1 / Plan 1b)

Setzt Cluster A voraus. Rein mechanisch.

| Aktion | Dateien |
|--------|---------|
| `shifty-dioxus/Cargo.toml` Zeile 28-29: `path = "rest-types"` → `path = "../rest-types", default-features = false` | `shifty-dioxus/Cargo.toml` |
| `shifty-dioxus/rest-types/` löschen | `rm -rf shifty-dioxus/rest-types/` |
| Backend-Workspace: `cargo check --workspace` sicherstellen | Backend-Workspace |

### Cluster C: Absence-Stack (Wave 2, parallel möglich)

Betrifft: `AbsencePeriodTO`, `AbsenceCategoryTO`, `UnavailabilityMarkerTO`, `AbsencePeriodCreateResultTO`

Frontend-Konsumenten: `src/loader.rs` (ShiftplanDay.unavailable-Feld), `src/state/shiftplan.rs` (neues Feld State-Mirror)

Disjunkte Module: `src/state/shiftplan.rs`, `src/loader.rs` (absence-Teil)

### Cluster D: Booking-Result-Wrapper (Wave 2, parallel möglich)

Betrifft: `BookingCreateResultTO`, `CopyWeekResultTO`, `WarningTO`

Frontend-Konsumenten: `src/api.rs` (`add_booking`, `copy_week`), ggf. Render-Sites für WarningTO

Wichtig: `add_booking()` gibt derzeit `Result<(), reqwest::Error>` zurück, obwohl der Backend-Endpoint `BookingCreateResultTO` zurückgibt. Nach dem Swap muss der Rückgabetyp angepasst werden (Backend-Antwort deserialisieren), aber das `warnings`-Feld wird ignoriert (no-op).

Disjunkte Module: `src/api.rs` (booking-Sektion), ggf. `src/service/` (der Aufrufer von `add_booking`)

### Cluster E: Slot-Capacity (Wave 2, parallel möglich)

Betrifft: `SlotTO.max_paid_employees`, `ShiftplanSlotTO.current_paid_count`

Frontend-Konsumenten: `src/loader.rs` (load_shift_plan), `src/state/shiftplan.rs` (Slot-struct)

Disjunkte Module: `src/state/shiftplan.rs` (Slot-struct), `src/loader.rs` (slot-Mapping-Teil)

### Cluster F: ShiftplanAssignment + User-Invitations (Wave 2, parallel möglich)

Betrifft: `ShiftplanAssignmentTO`, `InvitationStatus`/`InvitationResponse`/`GenerateInvitationRequest`

Frontend-Konsumenten:
- `src/state/user_management.rs:8-11` (lokale `ShiftplanAssignment` struct — muss durch Backend-`ShiftplanAssignmentTO` ersetzt oder behalten und `Serialize`/`Deserialize`-Compat geprüft werden)
- `src/api.rs:19` (`state::ShiftplanAssignment` importiert, nicht `rest_types::ShiftplanAssignmentTO`)
- `src/api.rs:7` (`InvitationResponse` importiert)
- `src/service/user_management.rs:7` (`InvitationResponse` importiert)

Disjunkte Module: `src/state/user_management.rs`, `src/api.rs` (user-invitation-Sektion)

### Cluster G: Billing + Cutover-Surface (Wave 2, parallel möglich)

Betrifft: `BillingPeriodTO.snapshot_schema_version`, Cutover-DTOs (`CutoverGateDriftRowTO`, etc.), `ExtraHoursCategoryDeprecatedErrorTO`

Frontend-Konsumenten:
- `src/service/billing_period.rs` / `src/loader.rs` für `BillingPeriodTO`-Feld
- Cutover-DTOs: kein aktiver Frontend-Konsument identifiziert (werden vom Frontend nicht aufgerufen)

Disjunkte Module: `src/service/billing_period.rs`, `src/loader.rs` (billing-Teil)

### Cluster H: Toggle/Impersonate (Wave 2, parallel möglich)

Betrifft: `ToggleTO`, `ToggleGroupTO`, `ImpersonateTO`

Frontend-Konsumenten: Kein aktiver Konsument in `src/api.rs` identifiziert (Endpoints noch nicht gebunden)

Disjunkte Module: Keine — diese Types brauchen nur importierbar zu sein. Kompiliert after swap automatisch da keine Nutzung.

### Cluster-Disjunktheit-Analyse

Die Cluster C, D, E, F, G, H sind weitgehend disjunkt bezüglich der betroffenen Modul-Mengen. Ausnahme: `src/loader.rs` wird von Cluster C (absence) und Cluster E (slot-capacity) beide berührt. Der Planner muss entscheiden, ob beide in einen Plan oder in zwei Plans mit expliziter Zeilen-Partitionierung gehen.

---

## 3. Match-Arm-Pattern und Panic-Situationen

### Aktive Panics (müssen adressiert werden)

| Datei | Zeile | Panic-Kontext | Erforderliche Aktion |
|-------|-------|--------------|---------------------|
| `src/state/employee.rs` | 89 | `WorkingHoursCategory::from_identifier()`: `_ => panic!("Unknown working hours category: {}", identifier)` | UI-SPEC Regel 3: `_ => WorkingHoursCategory::Unknown(identifier.into())` — braucht neuen `Unknown(Rc<str>)`-Variant |
| `src/state/employee.rs` | 151 | `From<&WorkingHoursCategory> for ExtraHoursCategoryTO`: `_ => panic!(...)` bei `Shiftplan`/`VacationDays` | Diese Panic ist ein PROGRAM-INVARIANT-Check (nicht user-facing), nicht ein enum-exhaustiveness-Problem. Kann bleiben FALLS kein Plan-Wave diese Funktion verändert. |
| `src/state/shiftplan.rs` | 59 | `Weekday::from_num_from_monday()`: `_ => panic!("Invalid weekday number: {}", num)` | Defensiver Fallback: `_ => Weekday::Monday` (oder neuer `Unknown(u8)`) — ist ein interner Util, kein Wire-Type |

Die `ExtraHoursCategoryTO`-Match-Arme in `state/employee.rs:140-170` sind erschöpfend (alle Varianten gemappt). [VERIFIED]

### Render-Sites mit potenziellem Match-Erweiterungsbedarf

Nach dem Cargo-Swap und der Einführung von `WarningTO` in `api.rs` muss geprüft werden, ob irgendwo im Render-Code bereits `match warning { ... }` existiert. Falls ja: UI-SPEC Regel 1 (`rsx! {}`). Aktuelle Suche ergab keinen direkten WarningTO-Render-Code (Warnings werden in `api.rs` derzeit ignoriert — `add_booking` gibt `()` zurück).

---

## 4. Cargo-Workspace-Topology

### Verifikation: Workspace-Exclusion von `shifty-dioxus`

`shifty-backend/Cargo.toml` enthält `exclude = ["shifty-dioxus"]`. [VERIFIED: direkter Read]

Das bedeutet: `shifty-dioxus` ist ein eigenständiger Cargo-Workspace, der NICHT Teil des Backend-Workspaces ist. Ein `path = "../rest-types"` in `shifty-dioxus/Cargo.toml` referenziert eine Crate AUSSERHALB seines eigenen Workspaces — das ist ein Standard-Cargo-Pattern und funktioniert problemlos.

### Feature-Analyse: `default-features = false`

Backend `rest-types/Cargo.toml`:
```toml
[features]
default = []
service-impl = ["dep:service", "dep:shifty-utils"]
```

`default = []` ist bereits gesetzt. `default-features = false` im Frontend ist also effektiv ein No-Op bezüglich Features — aber ein explizites Signal für Zukunftssicherheit.

**BLOCKING-ISSUE:** Das `use shifty_utils::{derive_from_reference, LazyLoad};` auf Zeile 8 von `rest-types/src/lib.rs` ist NICHT feature-gated. `shifty_utils` ist eine mandatory dep in `rest-types/Cargo.toml` (`path = "../shifty-utils"` ohne `optional = true`). Dies bedeutet: selbst mit `default-features = false` zieht die Backend-`rest-types` `shifty_utils` als transitive Dependency — was beim WASM-Build nur dann ein Problem ist, wenn `shifty_utils` nicht WASM-kompatibel ist.

**Untersuchungsergebnis `shifty_utils` WASM-Compat:** `shifty-utils` hängt nur von `thiserror` und `time` ab (keine `std::net`, `tokio`, `std::fs`, `std::io`). Die Crate ist wahrscheinlich WASM-kompatibel. [VERIFIED: direkter Cargo.toml-Read + Source-Scan]

**Implikation:** Der `use shifty_utils::...`-Import ist kein WASM-Blocker, aber er macht `shifty_utils` zu einer obligatorischen Compile-Dep für das Frontend. Das ist akzeptabel solange `shifty_utils` WASM-kompatibel bleibt. Der eigentliche Compile-Fehler nach dem Swap kommt vom `use shifty_utils::{...}` in Verbindung mit `LazyLoad` — denn `LazyLoad` ist eine generische Struct, nicht eine Macro, und `derive_from_reference!()` ist ein Makro. Beide werden im Frontend-Code NICHT aufgerufen (da `service-impl`-gated). Der `use`-Import ist in Rust zulässig auch wenn die Items nicht benutzt werden — der Compiler warnt aber nicht weg wenn die dep im Cargo.toml präsent ist.

**Vorläufige Conclusion:** Der `shifty_utils`-Import in `rest-types/src/lib.rs:8` ist wahrscheinlich KEIN Build-Blocker, da `shifty_utils` selbst WASM-kompatibel ist. Der Plan sollte aber explizit `cargo build --target wasm32-unknown-unknown` als Verifikation nutzen.

### time-Versionierung

| Crate | time-Version | Features |
|-------|-------------|---------|
| Backend `rest-types/Cargo.toml` | `0.3.36` | `serde-human-readable` |
| Frontend `shifty-dioxus/Cargo.toml` (main crate) | `0.3.41` | viele |
| Frontend `rest-types/Cargo.toml` (fork) | `0.3.41` | `serde-human-readable, parsing, formatting, serde, std` |

Nach dem Swap verwendet das Frontend die Backend-`rest-types` mit `time = "0.3.36"`. Das Frontend selbst deklariert `time = "0.3.41"`. Cargo verwendet `semver`-kompatible Versionen aus dem Resolver — bei Minor-Versions (beide `0.3.x`) wird der höchste kompatible genommen. Kein Konflikt erwartet.

**JEDOCH:** Backend `rest-types` deklariert nur `features = ["serde-human-readable"]`. Frontend-`loader.rs` und `state/*.rs` nutzen `time::Date`, `time::PrimitiveDateTime` usw. die nur im Frontend-Crate (`Cargo.toml`) mit den nötigen Features aktiviert sind — NICHT in `rest-types`. Die `rest-types` selbst brauchen für `time::Date`-Felder nur `serde-human-readable`; die Parsing/Formatting-Features braucht nur der Frontend-Code selbst. Kein Konflikt.

### PartialEq-Diskrepanz in TOs

Die Frontend-Fork hat auf einigen TOs `PartialEq, Eq` hinzugefügt (Dioxus-Reaktivität). Das Backend hat diese Derives nicht überall.

Identifiziert: `ShiftplanTO` — Backend hat `#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]`, Frontend-Fork hat `#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]`. [VERIFIED]

`SlotTO` hat `PartialEq, Eq` in BEIDEN (Backend Zeile 306, Frontend-Fork). [VERIFIED]

Wave-0-Plan muss eine vollständige Diff-Liste dieser Derives erstellen und fehlende Derives zum Backend-`rest-types` hinzufügen, wo das Frontend sie benötigt.

---

## 5. Validation Architecture

> `.planning/config.json` hat keinen `nyquist_validation`-Key — behandelt als enabled.

### Test-Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Standard Rust, no pytest/jest) |
| Config | Kein separates Test-Config-File — Tests inline + `src/tests/` |
| Quick run (Frontend) | `cd shifty-dioxus && cargo test` (non-WASM targets) |
| WASM-Compile-Check | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` (requires nix develop) |
| Backend check | `cd shifty-backend && cargo check --workspace` |

### Phase Requirements → Test Map

| Req ID | Verhalten | Test-Typ | Automatisierter Befehl | Test existiert? |
|--------|----------|---------|----------------------|----------------|
| RT-01 | `Cargo.toml` hat `path = "../rest-types"` | Strukturcheck | `grep -r 'path = "../rest-types"' shifty-dioxus/Cargo.toml` | Wave 0 |
| RT-02 | Verzeichnis `rest-types/` unter `shifty-dioxus/` nicht vorhanden | Strukturcheck | `find shifty-dioxus -type d -name rest-types \| wc -l \| grep -q "^0"` | Wave 0 |
| RT-03 | Alle 17 TOs + 4 Felder kompilieren | Compile-Check | `cargo build --target wasm32-unknown-unknown` in `shifty-dioxus/` | FC-02 |
| FC-01 | Match-Arme exhaustive (kein `panic!` auf bekannte Varianten) | Compile-Check + Review | `cargo build --target wasm32-unknown-unknown` (rustc enforces exhaustiveness) | FC-02 |
| FC-02 | WASM-Build grün | Compile-Check | `cargo build --target wasm32-unknown-unknown` | Phase-Gate |

### Sampling Rate

- **Nach jedem Plan-Wave:** `cargo check` oder `cargo build --target wasm32-unknown-unknown` in `shifty-dioxus/`
- **Phase-Gate (vor Phase 7):** Vollständiger WASM-Build grün; `find . -type d -name rest-types | wc -l == 1` im Repo-Root

### Wave 0 Gaps

Es fehlt kein Test-Framework — `cargo test` läuft schon. Was fehlt sind Plan-spezifische Verifikationsschritte:

- [ ] Shell-Skript oder inline-Verifikation: `find . -type d -name rest-types | wc -l` muss `1` ergeben (RT-02)
- [ ] Backend `cargo check --workspace` nach Wave-0-Backend-Patch bleibt grün

---

## 6. Risiken und Landmines

### Landmine 1 — `shifty_utils` unconditional import (KRITISCH)

**Was:** `use shifty_utils::{derive_from_reference, LazyLoad};` auf Zeile 8 von `rest-types/src/lib.rs` ist nicht `#[cfg(feature = "service-impl")]`-gated.

**Risiko:** Wenn `shifty_utils` irgendwann WASM-inkompatible Dependencies bekommt (z. B. `sqlx`, `tokio`), bricht der Frontend-Build still.

**Aktueller Status:** `shifty_utils` ist aktuell WASM-kompatibel (nur `thiserror` + `time`). Compile-Fehler unwahrscheinlich in v1.2.

**Empfehlung:** Als Teil von Wave-0-Backend-Patch den `use shifty_utils::...`-Import mit `#[cfg(feature = "service-impl")]` gaten, um langfristige Hygiene sicherzustellen.

### Landmine 2 — `InvitationStatus`-Familie nicht in `rest-types` (KRITISCH)

**Was:** Frontend-Code importiert `rest_types::InvitationResponse` und `rest_types::GenerateInvitationRequest`. Diese existieren NICHT im Backend-`rest-types`.

**Risiko:** Nach Cargo-Swap unmittelbarer Compile-Fehler: `error[E0432]: unresolved import 'rest_types::InvitationResponse'`.

**Fix:** In Wave-0-Backend-Patch: `InvitationStatus`, `InvitationResponse`, `GenerateInvitationRequest` aus `rest/src/user_invitation.rs` nach `rest-types/src/lib.rs` verschieben (oder duplizieren mit Re-Export).

### Landmine 3 — `ShiftplanTO` und weitere TO derive-Diskrepanzen (MITTEL)

**Was:** Frontend-Fork hat `PartialEq, Eq` auf `ShiftplanTO`, Backend-`rest-types` nicht.

**Risiko:** Compile-Fehler wenn Frontend-Code `ShiftplanTO` in Kontexten verwendet, die `PartialEq` erfordern (z. B. `GlobalSignal`-Reaktivität oder struct-Equality-Checks in Tests).

**Fix:** Wave-0-Backend-Patch: `PartialEq, Eq` zu `ShiftplanTO` und anderen betroffenen TOs hinzufügen. Vollständige Diff-Liste muss im Plan ermittelt werden.

### Landmine 4 — `add_booking` Rückgabetyp-Mismatch (MITTEL)

**Was:** `api::add_booking()` gibt `Result<(), reqwest::Error>` zurück, ignoriert die Backend-Antwort (`BookingCreateResultTO`). Nach dem Swap ist das zulässig — `response.error_for_status_ref()?` prüft den Status-Code, die Body-Deserialisierung wird übersprungen. Kein Compile-Fehler, aber potenzielle Laufzeit-Diskrepanz falls der Backend-Code einen 422 wirft.

**Risiko:** Kein Compile-Fehler. Warnings in `BookingCreateResultTO` werden nach dem Swap weiterhin ignoriert. Dieses Verhalten ist für v1.2 akzeptabel (FC-01: no-op ist ok, UI-Closure ist v1.3).

### Landmine 5 — `BillingPeriodTO.snapshot_schema_version` Feld fehlt im Frontend-Fork

**Was:** Backend `BillingPeriodTO` (Zeile 1312) hat `snapshot_schema_version: u32`. Frontend-Fork `BillingPeriodTO` (Zeile 1226-1237) hat dieses Feld NICHT.

**Risiko nach Swap:** Serde-Deserialisierung schlägt NICHT fehl, da Backend `snapshot_schema_version` immer serialisiert (kein `skip_serializing_if`). Frontend würde Feld einfach ignorieren, aber NUR wenn das Backend-`rest-types`-`BillingPeriodTO` kein `#[serde(default)]` auf dem Feld hat. Prüfen!

Zeile 1312 im Backend: `pub snapshot_schema_version: u32,` — kein `#[serde(default)]`. Das bedeutet: wenn die Backend-Antwort dieses Feld enthält und der Frontend-State-Typ es nicht hat, ist es serde-seitig ok (unknown fields werden ignoriert). Aber wenn Frontend-Code `BillingPeriodTO.snapshot_schema_version` referenzieren möchte, gibt es einen Compile-Fehler. Momentan tut das kein Frontend-Code — aber nach dem Swap existiert das Feld auf dem Backend-`BillingPeriodTO`, also wird der Compiler es kennen.

**Fix:** Kein Code-Change im Frontend nötig für Compile-Grün — das Feld ist nach dem Swap VORHANDEN (aus dem Backend). Der Frontend-State-Mirror `BillingPeriod` (falls existiert) braucht nur das Feld ignorieren oder hinzufügen.

### Landmine 6 — WASM-Build-Toolchain nur via `nix develop`

**Was:** `dx` und `wasm-bindgen-cli` sind nicht im `PATH`, nur in der Nix-Dev-Shell (`flake.nix`).

**Risiko:** Ein Plan der `cargo build --target wasm32-unknown-unknown` außerhalb von `nix develop` ausführt, schlägt fehl.

**Fix:** Alle WASM-Compile-Checks in Plans mit `nix develop --command cargo build --target wasm32-unknown-unknown` oder innerhalb der Nix-Shell schreiben.

### Landmine 7 — `ShiftplanSlotTO` im Frontend-Fork fehlt `current_paid_count`

**Was:** Frontend-Fork `ShiftplanSlotTO` (im Frontend-Fork) hat kein `current_paid_count`-Feld. Nach dem Swap ist das Feld vorhanden (Backend-`ShiftplanSlotTO` hat es). `loader.rs:load_shift_plan()` konstruiert `Slot` manuell ohne dieses Feld — `slot.slot.current_paid_count` wird nach dem Swap verfügbar sein, aber der Frontend-Loader-Code referenziert es noch nicht.

**Risiko:** Kein Compile-Fehler (Feld existiert, wird nur nicht gelesen). Aber `state::shiftplan::Slot` hat kein `max_paid_employees`/`current_paid_count`-Feld — wenn der Loader das neue Feld setzen soll, braucht `Slot` das Feld.

**Fix:** Wave-2 Cluster E Plan erweitert `state::shiftplan::Slot` um `max_paid_employees: Option<u8>` und `current_paid_count: u8` (default `None`/`0`); loader mappt aus `ShiftplanSlotTO.current_paid_count`.

---

## 7. Code-Beispiele

### Pattern: Cargo-Dep-Swap (RT-01)

```toml
# shifty-dioxus/Cargo.toml — VORHER:
[dependencies.rest-types]
path = "rest-types"

# NACHHER:
[dependencies.rest-types]
path = "../rest-types"
default-features = false
```
[VERIFIED: aktueller Stand in `shifty-dioxus/Cargo.toml:28-29`]

### Pattern: Neues TO-Feld in State-Mirror spiegeln (UI-SPEC Regel 2)

```rust
// In src/state/shiftplan.rs — Slot-Struct erweitern:
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    // ... bestehende Felder ...
    pub max_paid_employees: Option<u8>,  // NEU — v1.3 FUI-02 rendert dies
    pub current_paid_count: u8,          // NEU — v1.3 FUI-01 rendert dies
}

// In loader.rs::load_shift_plan — Mapping erweitern:
Slot {
    // ... bestehende Felder ...
    max_paid_employees: slot.slot.max_paid_employees,      // aus SlotTO via ShiftplanSlotTO
    current_paid_count: slot.current_paid_count,            // aus ShiftplanSlotTO
    // v1.3 rendert diese Felder — v1.2 übergibt sie nur durch
}
```
[ASSUMED — Kein Test beweist die exakte Slot-Mapping-Syntax; Patterns aus Coderead extrapoliert]

### Pattern: No-Op Match-Arm für neue WarningTO-Varianten (UI-SPEC Regel 1)

```rust
// In einem hypothetischen Warning-Renderer:
match warning {
    WarningTO::BookingOnAbsenceDay { .. } => rsx! { /* bestehender Renderer */ },
    WarningTO::BookingOnUnavailableDay { .. } => rsx! { /* bestehender Renderer */ },
    WarningTO::AbsenceOverlapsBooking { .. } => rsx! { /* bestehender Renderer */ },
    WarningTO::AbsenceOverlapsManualUnavailable { .. } => rsx! { /* bestehender Renderer */ },
    WarningTO::PaidEmployeeLimitExceeded { .. } => rsx! {},  // v1.3 FUI-01
}
```
[CITED: UI-SPEC §"Regel 1 — Match-Arme: invisible-skip via rsx! {}"]

### Pattern: `InvitationStatus`-Migration in `rest-types/src/lib.rs`

```rust
// In rest-types/src/lib.rs hinzufügen:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InvitationStatus {
    Valid,
    Expired,
    Redeemed,
    #[serde(rename = "sessionrevoked")]
    SessionRevoked,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub username: String,
    pub token: Uuid,
    pub invitation_link: String,
    pub status: InvitationStatus,
    // OffsetDateTime benötigt time-Feature "serde" + "formatting"  
    // Alternative: String für RFC3339 (einfacher, WASM-kompatibel)
    pub redeemed_at: Option<String>,  // RFC3339 string
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GenerateInvitationRequest {
    pub username: String,
    pub expiration_hours: Option<i64>,
}
```
[ASSUMED — `redeemed_at` als String statt `Option<OffsetDateTime>` ist eine Design-Entscheidung, die geprüft werden muss; Backend REST-Layer verwendet `#[serde(with = "time::serde::rfc3339::option")]`]

---

## 8. State of the Art

| Altes Muster | Aktuelles Muster | Seit wann | Impact |
|-------------|-----------------|-----------|--------|
| Frontend-Fork von `rest-types` | Backend-`rest-types` via `path = ".."` mit `default-features = false` | v1.2 Phase 6 (jetzt) | Eliminiert Drift; rustc erzwingt Aktualität |
| `add_booking()` gibt `()` zurück (ignoriert Warnings) | Idealerweise: `BookingCreateResultTO` deserialisieren, Warnings propagieren | v1.3 FUI-* | v1.2 behält das alte Muster (no-op ok) |

---

## Assumptions Log

| # | Behauptung | Abschnitt | Risiko wenn falsch |
|---|-----------|---------|-------------------|
| A1 | `shifty_utils` ist WASM-kompatibel (keine std-only net/io/fs deps) | §4 / §6 Landmine 1 | Mittleres Risiko — `cargo build --target wasm32-unknown-unknown` würde mit `shifty_utils`-Linking-Fehler scheitern |
| A2 | `redeemed_at: Option<String>` als RFC3339-String in `InvitationResponse` ist serde-kompatibel mit dem Backend | §7 Code-Beispiele | Niedrig-Mittel — wenn Backend `#[serde(with = "time::serde::rfc3339::option")]` serialisiert, kann Frontend es als `Option<String>` deserialisieren |
| A3 | Keine weiteren TOs ausser `ShiftplanTO` haben `PartialEq`-Diskrepanzen zwischen Fork und Backend | §4 Landmine 3 | Mittel — ein vollständiger derive-Diff zwischen beiden Files muss im Plan gemacht werden |
| A4 | `ToggleTO`, `ToggleGroupTO`, `ImpersonateTO` haben keinen aktiven Frontend-Konsumenten | §2 Cluster H | Niedrig — nach Swap: Import wird verfügbar, aber kein Compile-Fehler da keine Nutzung |

---

## Open Questions (RESOLVED)

> **Status (Revision iter 1, 2026-05-07):** Alle drei Open Questions sind durch Plan-Entscheidungen beantwortet. Plan-Referenzen am Ende jeder Frage.

1. **`redeemed_at` in `InvitationResponse` (RESOLVED)**
   - Was wir wissen: Backend-REST-Layer serialisiert es als RFC3339-String via `#[serde(with = "time::serde::rfc3339::option")]`
   - Was unklar war: Ob die Frontend-Fork `optional_timestamp`-Modul (Zeilen 1413+) im Frontend-`rest-types` nach dem Swap entfallen soll oder ob wir `time::serde::rfc3339::option` in das Backend-`rest-types` importieren müssen
   - **RESOLVED — Entscheidung D-Phase6-01:** `InvitationResponse.redeemed_at` ist `Option<String>` (RFC3339-Format) in Backend-`rest-types`. Frontend deserialisiert RFC3339-String als String, was für die aktuelle Display-Logik ausreicht. Vermeidet `time`-Feature-Erweiterung in `rest-types/Cargo.toml` (potenzieller WASM-Impact).
   - **Plan-Referenz:** Plan 06-00 Task 1 Änderung C (Invitation-Familie hinzufügen), siehe `06-00-PLAN.md` `<action>`-Block "Änderung C — Invitation-Familie hinzufügen".

2. **Vollständige derive-Diff zwischen Backend-`rest-types` und Frontend-Fork (RESOLVED)**
   - Was wir wissen: `ShiftplanTO` ist betroffen
   - Was unklar war: Welche weiteren TOs im Frontend-Fork `PartialEq, Eq, Hash` haben, die das Backend nicht hat
   - **RESOLVED:** Empirisch verifiziert per Grep — nur `ShiftplanTO` weist eine relevante Diskrepanz auf (`PartialEq, Eq` fehlt im Backend). `SlotTO` hat `PartialEq, Eq` in beiden. Andere TOs im Frontend-Fork haben keine zusätzlichen Derives, die das Backend nicht hätte. Sollte Plan 06-04 weitere `trait bound .* PartialEq`-Compile-Errors aufdecken, ziehen wir den Fix in Wave 0 nach (siehe `06-04-PLAN.md` Task 0 Patch-Strategie-Tabelle, Zeile "trait bound .* PartialEq").
   - **Plan-Referenz:** Plan 06-00 Task 0 Änderung A (`ShiftplanTO` derive-Erweiterung), siehe `06-00-PLAN.md` `<action>`-Block "Änderung A — ShiftplanTO Derives ergänzen".

3. **`ShiftplanTO`-Nutzung für Dioxus-Reaktivität im Frontend (RESOLVED)**
   - Was wir wissen: `src/service/user_management.rs` nutzt `ShiftplanAssignment` (state-type, nicht TO), `GlobalSignal<UserManagementStore>` enthält `shiftplan_assignments: Vec<ShiftplanAssignment>`
   - Was unklar war: Ob `ShiftplanTO` direkt in einem `GlobalSignal` sitzt (was `PartialEq` erfordern würde)
   - **RESOLVED:** `PartialEq, Eq` werden vorsorglich auf `ShiftplanTO` ergänzt — egal ob das Frontend den Type direkt in einem `GlobalSignal` nutzt oder nur transitiv. Die Frontend-Fork hat diese Derives, also ist Parity die sichere Wahl. Falls Plan 06-04 weitere `PartialEq`-Bedarfe aufdeckt, ergänzen wir sie ad-hoc analog zu D-Phase6-01.
   - **Plan-Referenz:** Plan 06-00 Task 0 Änderung A (`ShiftplanTO` derive-Erweiterung mit `PartialEq, Eq`), siehe `06-00-PLAN.md`.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Backend-Compile, Frontend-non-WASM-Tests | ✓ | 1.93.0 | — |
| `nix develop` | WASM-Toolchain (`wasm32-unknown-unknown` target, `dx`, `wasm-bindgen-cli`) | ✓ | 2.31.2 | — |
| `dx` (dioxus-cli) | `dx serve` für FC-03 (Phase 7, nicht Phase 6) | ✗ (ausserhalb Nix-Shell) | — | `nix develop --command dx` |
| wasm32-unknown-unknown target | `cargo build --target wasm32-unknown-unknown` | ✗ (ausserhalb Nix-Shell) | — | `nix develop` (flake.nix Zeile 22) |
| `wasm-bindgen-cli` | Post-Build für `dist/` | ✗ (ausserhalb Nix-Shell) | — | `nix develop` |

**Alle WASM-Compile-Checks in Plans müssen innerhalb `nix develop` ausgeführt werden.**

---

## Project Constraints (aus CLAUDE.md)

Directives aus `shifty-backend/CLAUDE.md`, `CLAUDE.md` (root), `CLAUDE.local.md`:

1. **VCS: jj (Jujutsu)** — KEINE `git commit` / `git add`. Auto-Commit deaktiviert. User committe manuell.
2. **NixOS**: `nix develop` (NICHT `nix-shell`). `sqlx migrate run` für additive Migrationen; `sqlx database reset` ist DESTRUCTIVE (User-Confirmation nötig).
3. **Cargo Tests**: `cargo test` muss grün bleiben. Für Frontend: `cargo test` in `shifty-dioxus/`; für Backend: `cargo test --workspace` in `shifty-backend/`.
4. **i18n**: Phase 6 fügt KEINE neuen i18n-Keys hinzu (kein neues UI — UI-SPEC bestätigt).
5. **OpenAPI**: Backend-`rest-types`-Änderungen brauchen `ToSchema`-Derives auf neuen Types.
6. **`CURRENT_SNAPSHOT_SCHEMA_VERSION`**: NICHT relevant für v1.2 (keine Reporting-Änderungen).
7. **Service-Tier-Konvention**: Nicht relevant (Phase 6 berührt keine Service-Schicht).
8. **No Tailwind/RSX/Token/Layout-Change**: UI-SPEC Phase 6 ist Compile-Gate, visuelles Delta = 0.

---

## Validation Architecture

> `nyquist_validation`-Key fehlt in `.planning/config.json` — behandelt als enabled.

### Test-Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (Rust standard) |
| Config file | Kein separates Config-File |
| Quick run (Frontend) | `cd /home/neosam/programming/rust/projects/shifty/shifty-dioxus && cargo test` |
| WASM compile check | `nix develop --command cargo build --target wasm32-unknown-unknown` (in `shifty-dioxus/`) |
| Backend quick check | `cargo check --workspace` (in Backend-Workspace-Root) |

### Phase Requirements → Test Map

| Req ID | Verhalten | Test-Typ | Automated Command | Test existiert? |
|--------|----------|---------|-------------------|----------------|
| RT-01 | `Cargo.toml` Zeile enthält `path = "../rest-types"` | Strukturcheck | `grep -q 'path = "../rest-types"' shifty-dioxus/Cargo.toml && echo OK` | ❌ Wave 0 |
| RT-02 | Kein `shifty-dioxus/rest-types/`-Verzeichnis | Strukturcheck | `test $(find . -type d -name rest-types \| wc -l) -eq 1 && echo OK` | ❌ Wave 0 |
| RT-03 | Alle fehlenden TOs kompilieren | Compile-Check | `cargo build --target wasm32-unknown-unknown` | ❌ FC-02 |
| FC-01 | Exhaustive Match-Arme | Compile-Check | `cargo build --target wasm32-unknown-unknown` (rustc enforced) | ❌ FC-02 |
| FC-02 | WASM-Build grün | Compile-Check | `cargo build --target wasm32-unknown-unknown` | ❌ Phase-Gate |

### Wave 0 Gaps

- [ ] Wave-0-Backend-Patch: `InvitationStatus`-Familie in `rest-types/src/lib.rs` einführen
- [ ] Wave-0-Backend-Patch: derive-Diff zwischen Fork und Backend ermitteln; fehlende `PartialEq`/`Eq`-Derives hinzufügen
- [ ] Wave-0-Backend-Patch: `shifty_utils`-Import mit `#[cfg(feature = "service-impl")]` gaten (Hygiene)
- [ ] Verifizierung: `cargo check --workspace` in Backend bleibt grün nach Wave-0-Patch
- [ ] Verifizierung: `cargo build --target wasm32-unknown-unknown` in `shifty-dioxus/` grün nach Wave-1-Swap

---

## Architektur-Diagramm (Datenfluss nach Phase 6)

```
Backend-`rest-types` (SINGLE SOURCE OF TRUTH)
      │
      │ path = "../rest-types", default-features = false
      ▼
shifty-dioxus/Cargo.toml
      │
      ├── src/api.rs       ─── rest_types::*TO (importiert direkt)
      │                          │
      │                          ▼ HTTP/JSON Deserialisierung
      │                       (Backend-Antwort)
      │
      ├── src/loader.rs    ─── rest_types::*TO → state::* (From<&*TO>)
      │
      ├── src/state/*.rs   ─── state::* Structs (From<&*TO> impls)
      │                          │
      │                          ▼
      └── src/component/   ─── state::* in Props/Render (KEINE *TO)
          src/page/
```

---

## TL;DR — Was jeder Plan wissen muss

1. **Wave 0 ist Backend-seitig** (zwei Blocker müssen VOR dem Frontend-Swap gepatcht werden):
   - `InvitationStatus`, `InvitationResponse`, `GenerateInvitationRequest` in `rest-types/src/lib.rs` einführen
   - Missing `PartialEq`/`Eq`-Derives auf Backend-TOs identifizieren und hinzufügen (mindestens `ShiftplanTO`)
   - Optional: `use shifty_utils::...`-Import feature-gaten (Hygiene)

2. **Wave 1 = Cargo-Swap + Fork-Delete**: `shifty-dioxus/Cargo.toml:28-29` ändern, dann `rm -rf shifty-dioxus/rest-types/`. Danach öffnet sich die Compile-Error-Welle.

3. **`shifty-dioxus` ist aus dem Backend-Workspace excluded** (`exclude = ["shifty-dioxus"]` in `shifty-backend/Cargo.toml`). Path-Dep auf `../rest-types` funktioniert Cross-Workspace normal.

4. **`default-features = false` reicht technisch** — `rest-types`-default ist bereits `[]`. Aber `shifty_utils` ist eine unconditional dep; aktuell WASM-kompatibel aber sollte feature-gated werden.

5. **`InvitationStatus`-Familie existiert NUR im Frontend-Fork und in `rest/src/user_invitation.rs`** (nicht in `rest-types`). Dies ist der kritischste Missing-Type-Blocker.

6. **Alle WASM-Compile-Checks** müssen innerhalb `nix develop` in `shifty-dioxus/` laufen — WASM-Target ist ausserhalb der Nix-Shell nicht verfügbar.

7. **UI-SPEC-Regel 1 + 2 sind bindend**: Neue Match-Arme → `rsx! {}`; neue Felder → State-Mirror mit Default, kein Render. Kein `unimplemented!()`, kein `todo!()`, kein `panic!()` in Match-Armen.

8. **`ExtraHoursCategoryTO`-Match-Arme im Frontend sind bereits vollständig** — kein Handlungsbedarf hier.

9. **`add_booking()`/`copy_week()`-Rückgabetypen** können als `()` bleiben — die Backend-Warnings werden ignoriert (no-op, v1.3). Kein Compile-Fehler; die TOs müssen aber importierbar sein.

10. **`BillingPeriodTO.snapshot_schema_version`**: Nach Swap ist das Feld im Type vorhanden (aus Backend). Frontend braucht nur sicherzustellen, dass sein State-Mirror `BillingPeriod` (falls vorhanden) das Feld trägt oder ignoriert.

---

## Sources

### Primary (HIGH confidence)

- [VERIFIED: Direkter Read] `shifty-backend/rest-types/src/lib.rs` — alle 17 fehlenden TO-Definitionen; `ShiftplanSlotTO.current_paid_count`; `ShiftplanDayTO.unavailable`; `BillingPeriodTO.snapshot_schema_version`; `ShiftplanAssignmentTO`; `WarningTO`-Varianten; `UnavailabilityMarkerTO`-Varianten
- [VERIFIED: Direkter Read] `shifty-dioxus/rest-types/src/lib.rs` — Frontend-Fork; fehlende Felder in `BillingPeriodTO` und `SlotTO`; `InvitationStatus`-Definition
- [VERIFIED: Direkter Read] `shifty-dioxus/src/api.rs` — alle Endpoint-Bindings; Rückgabetypen; Import-Liste
- [VERIFIED: Direkter Read] `shifty-dioxus/src/state/employee.rs` — Panic-Sites; `ExtraHoursCategoryTO` Match-Exhaustiveness
- [VERIFIED: Direkter Read] `shifty-dioxus/src/state/shiftplan.rs` — Panic-Site `from_num_from_monday`; `Slot`-Struct ohne neue Felder
- [VERIFIED: Direkter Read] `shifty-backend/rest-types/Cargo.toml` — Feature-Tabelle; `service-impl`-Feature; `default = []`; `time = "0.3.36"`
- [VERIFIED: Direkter Read] `shifty-dioxus/Cargo.toml` — aktueller `path = "rest-types"`; `time = "0.3.41"`
- [VERIFIED: Direkter Read] `shifty-backend/Cargo.toml` — `exclude = ["shifty-dioxus"]`-Workspace-Config
- [VERIFIED: Direkter Read] `shifty-backend/shifty-utils/Cargo.toml` + `src/lib.rs` — WASM-Kompatibilität; nur `thiserror` + `time`-Deps
- [VERIFIED: Direkter Read] `rest/src/user_invitation.rs` — `InvitationStatus`/`InvitationResponse`/`GenerateInvitationRequest` leben NICHT in `rest-types`
- [VERIFIED: Direkter Bash-Grep] `use shifty_utils::{derive_from_reference, LazyLoad};` auf Zeile 8 ohne Feature-Gate

### Secondary (MEDIUM confidence)

- [CITED: `.planning/codebase/frontend/CONCERNS.md`] — §1 Drift-Inventur; §9 Panic-Sites
- [CITED: `.planning/phases/06-rest-types-unification-frontend-compile-through/06-UI-SPEC.md`] — No-Op-Rendering-Pattern, Touch-Point-Inventar
- [CITED: `.planning/ROADMAP.md`] — Wave-Topologie-Vorschlag; Success-Criteria

### Tertiary (LOW confidence)

- Keine

---

## Metadata

**Confidence breakdown:**
- Drift-Inventur (§1): HIGH — alle Types direkt in Sourcen verifiziert
- Cargo-Topology (§4): HIGH — Cargo.toml-Files direkt gelesen
- Cluster-Disjunktheit (§2): MEDIUM — basiert auf Modul-Analyse, aber echte Abhängigkeiten erst beim Compile-Error-Lauf sichtbar
- `shifty_utils` WASM-Kompatibilität (§6): MEDIUM — Source-Scan ohne WASM-Build-Test

**Research date:** 2026-05-07
**Valid until:** 2026-06-07 (stabile Codebase; abhängig von Backend-Änderungen)
**Last revision:** 2026-05-07 — Open Questions resolved (Revision iter 1)
</content>
</invoke>