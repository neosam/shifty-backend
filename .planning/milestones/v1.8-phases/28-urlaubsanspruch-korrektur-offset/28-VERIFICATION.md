---
phase: 28-urlaubsanspruch-korrektur-offset
verified: 2026-06-29T11:20:00Z
status: passed
score: 6/6 success criteria verified (automation + live HR browser smoke); 1 dev-proxy bug found & fixed
behavior_unverified: 0
overrides_applied: 0
---

# Phase 28: Urlaubsanspruch-Korrektur via Offset (HR, BE+FE) — Verification Report

**Phase Goal:** HR kann den berechneten Jahres-Urlaubsanspruch per signed Offset (Delta) korrigieren; HR-gekennzeichnet+editierbar, für User unsichtbar (nur Effektivwert). Plus Off-by-one-Proration-Fix + Snapshot-Bump 11→12.
**Verified:** 2026-06-29 (automated + live HR browser smoke by orchestrator)
**Status:** PASSED — all automated gates + live HR browser smoke confirmed; one dev-proxy gap found and fixed during the smoke.

## Live HR browser smoke (2026-06-29) — PASSED (+ bug found & fixed)
Backend (:3000) + frontend (:8080) started; DEVUSER temporarily granted HR (reverted after); test offset cleaned up.
- **Read-path / HR API-hiding:** for Max Schmidt, `GET /vacation-balance` returned `entitled_days: 16` (= round(15) + offset 1), `offset_days: 1`, `computed_entitled_days: 15`, `remaining_days: 31` — offset flows through; HR sees the breakdown.
- **HR inline editor:** the "Vertragsanspruch" StatBox rendered **"calculated 15 + Offset [n]"** with a signed number input; the big box showed the effective value.
- **Write-path:** setting the offset to 3 via the UI saved (no error) → effective **18**, remaining **33/33**, backend persisted `offset_days: 3` — full round-trip confirmed.
- **BUG FOUND & FIXED (fix(28) commit):** the FE offset-save first returned **HTTP 405** because `/vacation-entitlement-offset` was missing from `shifty-dioxus/Dioxus.toml`'s dev proxy (the 28-04 work added the api call + backend route but not the dev-proxy registration). Added the proxy rule; after a dx-serve restart the write-path works. (Dev-proxy only; production serves all routes via the backend.)

---

## Goal Achievement — Observable Truths (6 ROADMAP success criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC1 + D-28-02: `entitled_effective = round(Σ vacation_days_for_year) + offset_days`, flowing to `remaining_days` | VERIFIED | `vacation_balance.rs:195` computed, `:209-214` offset read+add (after round), `:280` remaining uses effective, `:287` `entitled_days: entitled_effective`. Tests `offset_calc` (17+1=18 / −2=15) pass. |
| 2 | SC2 + D-28-01: signed offset persisted per person+year; survives reload | VERIFIED | migration `20260629000000_create-vacation-entitlement-offset.sql` (id PK, signed `offset_days`, soft-delete, partial unique `(sales_person_id, year) WHERE deleted IS NULL`); DAO + Basic `VacationEntitlementOffsetService` (7 CRUD/HR tests pass). |
| 3 | SC3 + D-28-03: delta survives contract changes (not frozen) | VERIFIED | `offset_delta` test: base 17→20 with offset +1 ⇒ effective 21. Offset added to the live-computed base, not stored as absolute. |
| 4 | SC4 + D-28-07: HR detail StatBox shows effective + always-visible signed inline "berechnet {n} + Offset [x]", saves HR-gated on blur/Enter | VERIFIED (code) / smoke pending | `absences.rs:542` `VacationContractCell`; `service/vacation_balance.rs:66` `SaveOffset` → `loader/api::save_vacation_entitlement_offset` → `POST /vacation-entitlement-offset`. Rendered visual + persist roundtrip = browser smoke (pending). |
| 5 | SC5 + D-28-03: user self-view shows ONLY the effective value (no offset/computed, no field) | VERIFIED | API-level hiding: service sets `offset_days`/`computed_entitled_days` = `Some` only when `is_hr = hr.is_ok()`, else `None` (`vacation_balance.rs`); `offset_api_hiding` test (HR→Some, self→None). FE editor branch fires only on `is_hr && computed.is_some()` — no client-side re-derivation. |
| 6 | SC6 + D-28-06b: setting/changing the offset is HR-gated (`HR_PRIVILEGE`); new texts de/en/cs | VERIFIED | HR gate inside Basic offset service (`get_non_hr_forbidden` test); `POST/DELETE /vacation-entitlement-offset` REST CRUD (utoipa + ToSchema + ApiDoc); i18n `VacationOffset*` keys in en/de(`Locale::De`)/cs + coverage test. |

**Bonus (D-28-04 + D-28-05):** off-by-one proration fix (`employee_work_details.rs:178`, year-start `ordinal()-1`; year-end branch left correct per RESEARCH) + **`CURRENT_SNAPSHOT_SCHEMA_VERSION` bumped 11→12** (`billing_period_report.rs:117`, v12 history naming `VacationEntitlement`), 3 snapshot-version guard tests re-pinned to 12, full-year/partial-year regression tests added.

**Score:** 6/6 success criteria automation-verified; truth #4's rendered-visual + persist roundtrip is the single pending human item.

---

## Automated Gates (all PASS — integrated full-workspace run by orchestrator)

- **Backend `cargo test --workspace`:** EXIT 0 — 504 service_impl lib + 61 integration + 13 service + 11 dao + rest/openapi/rest-types/utils, **0 failures**. Includes the new offset (7) + balance-integration (offset_calc/delta/api_hiding) + off-by-one regression (5) + snapshot-guard (==12) tests.
- **Backend `cargo clippy --workspace -- -D warnings`:** EXIT 0 — clean (hard CI gate).
- **Frontend WASM build** (frontend devShell, provides rust-lld): `cargo build --target wasm32-unknown-unknown` → Finished, 0 errors.
- **Frontend `cargo test`:** 678 passed, 0 failed (677 baseline + 1 new); no new clippy warnings (207 == pre-existing baseline).
- **sqlx:** migration applied additively (`sqlx migrate run`, never reset); `.sqlx/` offline cache regenerated.
- **Snapshot discipline:** bump 11→12 done + documented + guard-tested (CLAUDE.md rule honored).

## Pending Human-UAT (browser smoke)

On the vacation overview, as HR open a person's detail (`forced_self`): the "Vertragsanspruch" StatBox shows the effective number with an inline "berechnet {computed} + Offset [x]" field; set +1 → box shows base+1; blur/Enter persists; after reload the offset remains. As a normal user, the same StatBox shows ONLY the effective number — no field, no "berechnet/Offset" line, and the raw API response carries no `offset_days` (API-level hiding). **Resume:** `/gsd-verify-work 28`.

---

*Verification by orchestrator (autonomous run), 2026-06-29. Automated gates authoritative; HR offset roundtrip smoke deferred to human.*
