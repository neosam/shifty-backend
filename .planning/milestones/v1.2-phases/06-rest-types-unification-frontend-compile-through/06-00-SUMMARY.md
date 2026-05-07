---
phase: 06-rest-types-unification-frontend-compile-through
plan: 00
subsystem: api
tags: [rest-types, dto, cargo-features, wasm-prep, invitation, dioxus-parity]

# Dependency graph
requires: []
provides:
  - "rest-types exportiert Invitation-Familie (`InvitationStatus`, `GenerateInvitationRequest`, `InvitationResponse`)"
  - "rest-types ist mit `default-features = false` WASM-tauglich (kein unconditional `shifty_utils`-Pull)"
  - "ShiftplanTO trägt `PartialEq, Eq` (Frontend-Fork-Parity für Dioxus-Reaktivität)"
  - "rest/src/user_invitation.rs nutzt rest_types-Re-Export, From-Impl ist orphan-rule-konform in rest-types umgezogen"
affects:
  - "06-01-PLAN (Wave 1: shifty-dioxus Cargo-Swap auf path = '../rest-types')"
  - "06-02 / 06-03 / 06-04 (Frontend-Compile-Through-Wellen, die rest_types::Invitation* importieren)"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Feature-gating: `#[cfg(feature = \"service-impl\")]` auf use-Imports, die ausschließlich in service-impl-gated Code referenziert werden"
    - "Wire-Format-Stabilität via `Option<String>` (RFC3339) statt `Option<OffsetDateTime>` für WASM-kompatible Fork-Konsumenten — vermeidet `time`-Feature-Erweiterung"

key-files:
  created: []
  modified:
    - "rest-types/src/lib.rs"
    - "rest/src/user_invitation.rs"

key-decisions:
  - "D-Phase6-01 (umgesetzt): `InvitationResponse.redeemed_at: Option<String>` (RFC3339) statt `Option<OffsetDateTime>` — wire-kompatibel mit dem bisherigen Backend-Output, vermeidet `time`-Feature-Bumps mit potenziellem WASM-Impact"
  - "`From<service::user_invitation::InvitationStatus> for InvitationStatus` lebt in rest-types unter `service-impl`-cfg — orphan-rule-konform und konsistent mit den anderen `From<&service::*>`-Impls in rest-types"
  - "Lokaler Helper `redeemed_at_to_rfc3339` in rest/src/user_invitation.rs konvertiert die service-internen `Option<OffsetDateTime>` zur Wire-Form `Option<String>`; bei Format-Fehler stabiler Fallback auf `\"\"` (in der Praxis nicht erreichbar für gültige `OffsetDateTime`)"

patterns-established:
  - "rest-types als Single Source of Truth: REST-Layer re-exportiert via `pub use rest_types::{...}` statt eigener Definitionen — Vorbild für künftige Migrationen aus dem rest-Crate ins rest-types"

requirements-completed: [RT-03]

# Metrics
duration: ~10min
completed: 2026-05-07
---

# Phase 6 Plan 0: Backend-rest-types-Vorbereitung Summary

**Invitation-DTO-Familie nach `rest-types` migriert, `shifty_utils`-Import feature-gated, `ShiftplanTO` mit `PartialEq, Eq` ergänzt — Backend ist swap-ready für Wave 1.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-05-07T13:34Z
- **Completed:** 2026-05-07T13:44Z
- **Tasks:** 2 (beide auto, kein Checkpoint)
- **Files modified:** 2

## Accomplishments

- `InvitationStatus`, `GenerateInvitationRequest`, `InvitationResponse` leben jetzt in `rest-types/src/lib.rs` — der Frontend-Code (Wave 1+) kann sie via `use rest_types::*` importieren, ohne dass im Cargo-Swap ein `unresolved import` aufschlägt.
- `use shifty_utils::{derive_from_reference, LazyLoad}` ist nun `#[cfg(feature = "service-impl")]`-gated; `cargo check --no-default-features` für `rest-types` läuft grün — die WASM-Voraussetzung für Wave 1 ist erfüllt.
- `ShiftplanTO` trägt `PartialEq, Eq` (Fork-Parity für Dioxus-Signal-Diffing).
- Backend-Workspace bleibt voll funktionsfähig: `cargo check --workspace` grün, `cargo test --workspace` grün mit 466/466 Tests (Baseline 461 + 5 zusätzlich gefundene Tests, alle pass) — keine Regressionen.

## Task Commits

Each task was committed atomically via jj:

1. **Task 0: ShiftplanTO PartialEq/Eq + shifty_utils-Import feature-gaten** — `5feed466` (feat)
2. **Task 1: Invitation-Familie nach rest-types migrieren + From-Impl umziehen** — `305d287d` (feat)

_(SUMMARY-Commit folgt als separater jj-Change.)_

## Files Created/Modified

- `rest-types/src/lib.rs` (modified) — `shifty_utils`-Import feature-gated; `ShiftplanTO` derive-Set um `PartialEq, Eq` erweitert; Invitation-Familie (Enum + 2 Structs) am Dateiende eingefügt; `From<service::user_invitation::InvitationStatus> for InvitationStatus`-Impl unter `service-impl`-cfg ergänzt.
- `rest/src/user_invitation.rs` (modified) — Lokale Definitionen entfernt, ersetzt durch `pub use rest_types::{GenerateInvitationRequest, InvitationResponse, InvitationStatus}`; lokale `From<ServiceInvitationStatus> for InvitationStatus`-Impl entfernt (umgezogen in `rest-types`); kleiner Helper `redeemed_at_to_rfc3339` ergänzt; ungenutzte Imports (`serde::{Deserialize, Serialize}`, `utoipa::ToSchema`, `InvitationStatus as ServiceInvitationStatus`) entfernt; zwei Konstruktionsstellen für `redeemed_at` auf den neuen `Option<String>`-Wireshape umgestellt.

## Decisions Made

- **D-Phase6-01 umgesetzt:** `InvitationResponse.redeemed_at` ist im migrierten Type `Option<String>` (RFC3339). Ein lokaler Helper in `rest/src/user_invitation.rs` formatiert das `Option<OffsetDateTime>` aus dem Service-Layer in genau das Format, das der bisherige `#[serde(with = "time::serde::rfc3339::option")]` produziert hat — Wire-Output bleibt byte-identisch.
- **From-Impl in rest-types statt rest:** Die Konvertierung `service::user_invitation::InvitationStatus → rest_types::InvitationStatus` ist in `rest-types/src/lib.rs` unter `#[cfg(feature = "service-impl")]` umgezogen, weil nach dem Re-Export der lokale `InvitationStatus` in `rest/` ein Foreign-Type ist (orphan rule). Pattern ist konsistent mit den vorhandenen `From<&service::*>`-Impls in `rest-types`.
- **Imports schlanker gehalten:** Beim Entfernen der lokalen Definitionen wurden auch die nun toten Imports (`serde::{Deserialize, Serialize}`, `utoipa::ToSchema`, der lokale Alias `ServiceInvitationStatus`) bereinigt — nötig wegen `lints.rust.warnings = "deny"` in `rest/Cargo.toml`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Acceptance-Kriterium für unconditional `use shifty_utils` falsch spezifiziert**
- **Found during:** Task 0 (Strukturcheck nach den Edits)
- **Issue:** Das Plan-Acceptance-Kriterium fordert `grep -n '^use shifty_utils' lib.rs | wc -l` ist `0`. Tatsächlich bleibt die `use`-Zeile aber bestehen (sie wird nur durch ein darüberliegendes `#[cfg(...)]`-Attribut feature-gegated, exakt wie die anderen `service::*`-Imports unmittelbar daneben). Die Zeile selbst beginnt weiterhin mit `use shifty_utils::...`.
- **Fix:** Acceptance-Intent wird über das ergänzende Kriterium (`gegated Import vorhanden` per `grep -nB1 ... | grep -c 'cfg(feature = "service-impl")'` ist `1`) abgedeckt — dieser Check passt. Funktional ist die Anforderung erfüllt: `cargo check --no-default-features` für `rest-types` läuft grün, was die eigentliche Absicht (kein unconditional Pull von `shifty_utils`) verifiziert. Hier wurde nichts am Code repariert; nur der Acceptance-Plan wurde de-facto mit `cargo check --no-default-features` als ergänzenden Beweis erfüllt.
- **Files modified:** Keine zusätzlichen Code-Änderungen nötig.
- **Verification:** `cd rest-types && cargo check --no-default-features` Exit 0 — beweist, dass `shifty_utils` nicht mehr unconditional gezogen wird.
- **Committed in:** Teil von `5feed466` (Task 0 commit).

**2. [Rule 2 - Missing Critical] Ungenutzte Imports nach DTO-Re-Export entfernt**
- **Found during:** Task 1 (cargo check nach erstem Edit)
- **Issue:** Nach dem Ersatz der drei lokalen Definitionen durch ein `pub use rest_types::{...}` wurden `serde::{Deserialize, Serialize}`, `utoipa::ToSchema` und `InvitationStatus as ServiceInvitationStatus` ungenutzt. Da `rest/Cargo.toml` `lints.rust.warnings = "deny"` setzt, hätte das den Build mit deny-Errors gebrochen.
- **Fix:** Use-Liste entsprechend bereinigt; `utoipa::OpenApi` wird weiterhin im `#[derive(OpenApi)]` der `UserInvitationApiDoc` benötigt und bleibt erhalten. `time::OffsetDateTime` bleibt für den `redeemed_at_to_rfc3339`-Helper und für die OIDC-Cookie-Logik.
- **Files modified:** `rest/src/user_invitation.rs`
- **Verification:** `cargo check --workspace` grün ohne Warnings; `cargo build --workspace` ebenfalls grün.
- **Committed in:** Teil von `305d287d` (Task 1 commit).

**3. [Rule 2 - Missing Critical] `redeemed_at`-Wireformat-Helper hinzugefügt**
- **Found during:** Task 1 (Plan-Action erwähnt eine Format-Erfordernis, aber nur konditional)
- **Issue:** Plan sagt: "Falls Backend-Code `redeemed_at: Some(now)` setzt — umstellen auf `now.format(&...Rfc3339).unwrap_or_default()`". Tatsächlich werden in `rest/src/user_invitation.rs` an zwei Stellen (Zeilen 210 und 258 vor dem Edit) `redeemed_at: invitation.redeemed_at` gesetzt, wobei `invitation.redeemed_at` vom Typ `Option<OffsetDateTime>` aus dem Service-Layer kommt. Da der migrierte Wire-Type `Option<String>` ist, würde inline-Format an zwei Stellen Code-Duplikat erzeugen.
- **Fix:** Privater Helper `redeemed_at_to_rfc3339(value: Option<OffsetDateTime>) -> Option<String>` definiert, der `Option::map`+`OffsetDateTime::format(&Rfc3339)` kapselt; beide Konstruktionsstellen rufen den Helper. So bleibt das Wire-Format byte-identisch zum bisherigen `#[serde(with = "time::serde::rfc3339::option")]`-Output, und die Logik ist DRY.
- **Files modified:** `rest/src/user_invitation.rs`
- **Verification:** `cargo test --workspace` grün (alle 466 Tests, kein Wire-Format-Drift).
- **Committed in:** Teil von `305d287d` (Task 1 commit).

---

**Total deviations:** 3 auto-fixed (1 Bug-im-Acceptance-Plan, 2 Missing-Critical für Code-Hygiene + Wireformat)
**Impact on plan:** Alle drei Abweichungen sind notwendig für Korrektheit (deny-warnings-Hygiene, Wire-Format-Stabilität) bzw. die Plan-Intent zu erfüllen. Keine Scope-Creep — Task-Reichweite ist exakt die im Plan definierte (zwei Files, zwei mechanische Migrationen).

## Issues Encountered

- **Backend-Test-Baseline +5:** Plan dokumentiert 461 Tests als Baseline, tatsächlich liefen 466 Tests — alle pass, keine Regressionen. Das ist ein additiver Drift seit der Plan-Schreibung (vermutlich neuere shiftplan-edit-Tests). Kein Handlungsbedarf — die Anforderung (`baseline bleibt erreichbar`) ist eingehalten.

## User Setup Required

None — keine externen Service-Konfigurationen nötig. Wave-0-Backend-Patch ist rein interner Code-Refactor.

## Next Phase Readiness

**Bereit für Plan 06-01 (Wave 1: Cargo-Swap im shifty-dioxus):**
- `rest-types` exportiert die Invitation-Familie unter den exakten Namen, die das Frontend importiert (`rest_types::{InvitationStatus, InvitationResponse, GenerateInvitationRequest}`).
- `rest-types` kompiliert mit `--no-default-features` grün — der Frontend-Cargo-Swap (`default-features = false`) wird `shifty_utils` nicht mehr unconditional pullen.
- `ShiftplanTO` hat die für Dioxus benötigten `PartialEq, Eq`-Derives.

**Keine Blocker.** Wave 1 (`shifty-dioxus/Cargo.toml`-Swap + `rm -rf shifty-dioxus/rest-types/`) kann unmittelbar starten.

## Self-Check: PASSED

**Files verified:**
- `rest-types/src/lib.rs` — modified ✓ (commit `5feed466` und `305d287d`)
- `rest/src/user_invitation.rs` — modified ✓ (commit `305d287d`)
- `.planning/phases/06-rest-types-unification-frontend-compile-through/06-00-SUMMARY.md` — created ✓

**Commits verified (jj log):**
- `5feed466` (Task 0) — gefunden ✓
- `305d287d` (Task 1) — gefunden ✓

**Compile / Test gates:**
- `cargo check --workspace` Exit 0 ✓
- `cargo check --no-default-features` (rest-types) Exit 0 ✓ — wichtigster Wave-1-Pre-Req-Beweis
- `cargo test --workspace` 466/466 Tests pass ✓
- `cargo build --workspace` Exit 0, keine Warnings ✓

**Structural acceptance criteria (Plan):**
- Task 0: 1/1 ShiftplanTO mit PartialEq/Eq ✓; 1/1 cfg-gated shifty_utils-Import vorhanden ✓
- Task 1: 1× InvitationStatus-Enum, 1× InvitationResponse-Struct, 1× GenerateInvitationRequest-Struct in rest-types ✓; 0× lokale Definitionen mehr in rest/src/user_invitation.rs ✓; 1× `pub use rest_types::{...Invitation...}` re-export ✓

---
*Phase: 06-rest-types-unification-frontend-compile-through*
*Completed: 2026-05-07*
