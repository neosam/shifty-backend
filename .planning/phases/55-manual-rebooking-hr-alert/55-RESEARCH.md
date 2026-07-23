# Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5) — Research

**Researched:** 2026-07-23
**Domain:** Rust (Axum/SQLx-Backend, clean layered architecture) + Dioxus/WASM-Frontend; Cross-Entity-Transaktions-Orchestrierung, Concurrency-sicheres State-Transition, Read-Aggregat-Neutralität
**Confidence:** HIGH (fast alle Erkenntnisse aus der Codebase selbst verifiziert; keine externen Paket-Recherchen nötig)

## Summary

Phase 55 ist eine reine In-Codebase-Feature-Phase auf einem in Phase 54 vollständig
gelegten Datenmodell. Es werden **keine neuen externen Pakete**, **keine Migrationen**
und **kein neues Datenmodell** eingeführt. Der gesamte Research-Fokus liegt darauf, die
exakten bestehenden Signaturen und Muster zu identifizieren, die kopiert werden müssen —
und die **eine wirklich neue architektonische Entscheidung** (Rebooking-Neutralitäts-Filter
im `ReportingService`) sauber zu fassen.

Phase 54 hat geliefert: `RebookingBatchService`/`RebookingBatchDao` (Basic-Tier, HR-gated,
`create_batch_with_entries` in einer Tx, UNIQUE-Partial-Index als Autorität), das
`ExtraHoursSource::{Manual, Rebooking}`-Marker-Enum (mit `#[default] Manual`, bereits durch
die gesamte ExtraHours-Kette verdrahtet inkl. `From<&ExtraHoursEntity>`), `VoluntaryStatsService`
(BL-Tier, HR-only), `committed_voluntary_target_in_range`. **Was Phase 54 explizit NICHT
gemacht hat und Phase 55 nachziehen MUSS:** Der `source == Rebooking`-Neutralitäts-Filter ist
im `ReportingService` **noch nicht aktiv** — die Aggregations-Sites in `reporting.rs`
(Zeilen ~894, ~1401, ~2263 für `volunteer_hours`; die `ExtraWork`-Summierung für balance)
zählen aktuell ALLE ExtraHours-Rows unabhängig vom `source`-Marker. Genau diese Lücke ist
der Kern von VOL-ACCT-03 und muss chirurgisch geschlossen werden (siehe „Kritische
Architektur-Entscheidung" unten).

Der neue `RebookingReconciliationService` ist ein **Business-Logic-Tier**-Service (konsumiert
`ExtraHoursService` + `RebookingBatchService` + `ReportingService`/`VoluntaryStatsService` +
`PermissionService` + `TransactionDao`). Er orchestriert in genau einer Transaktion: zwei
`ExtraHours::create`-Calls (mit `source=Rebooking`) → ein `RebookingBatchService::create`
(kind=Manual|HrSuggestion, state=Approved) mit den beiden extra_hours-IDs als Entry-FKs.
Die Approve/Reject-State-Transition (F5) braucht ein **neues** state-conditional DAO-UPDATE
mit `rows_affected()==1`-Guard — dieses Muster existiert in der Codebase noch nicht als
Optimistic-Lock-Variante (nur `user_invitation::delete_expired` nutzt überhaupt `rows_affected()`),
wird also in Phase 55 erstmals eingeführt.

**Primary recommendation:** Baue `RebookingReconciliationService` als BL-Service mit **einem
einzigen internen Write-Path** (`rebook(direction, hours, week, kind)`), den sowohl F3 (Manual)
als auch F5-Approve (HrSuggestion) aufrufen (Pitfall 15). Löse die Read-Aggregat-Neutralität
(VOL-ACCT-03) durch einen `source`-bewussten Filter an den ExtraHours-Aggregations-Sites in
`reporting.rs`, NICHT durch nachträgliche Korrektur-Arithmetik. Sichere die Neutralität mit
einem Property-Test ab, bevor irgendein FE-Code entsteht.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Atomarer Doppel-Eintrag (2×ExtraHours + 1×Batch) | Service (BL) | DAO | Cross-Entity-Invariante in einer Tx → gehört in BL-Service, nicht REST |
| Approve/Reject-State-Transition (Race-sicher) | DAO (SQL) | Service (BL) | `WHERE state='pending'` + affected-rows ist atomar nur auf DB-Ebene garantierbar |
| Rebooking-Neutralität in Read-Aggregaten | Service (`ReportingService`) | — | Der `source`-Filter muss dort greifen, wo aggregiert wird (single source of truth) |
| Alert-Predicate (`balance<=-0.5 ∧ voluntary_ist>0 ∧ cap`) | Service (`reporting.rs` pure fn) | — | Backend-computed (Fat Backend); FE macht keine Arithmetik |
| `has_pending_rebooking` DTO-Feld | Service (`short_employee_report`) + DAO-Query | REST | Braucht neues DAO-Query `find_pending_for_sales_person` (Basic-Tier) |
| `proposed_hours = min(\|balance\|, voluntary_ist)` | Service (BL) | — | D-55-03, Backend-computed |
| IST/DANN-Modal-Werte | Service (BL, `RebookingSuggestionTO`) | — | Alle DANN-Werte backend-computed (HR-ALERT-02) |
| Alert-Banner (dauerhafte Warnzeile) | Frontend (Component) | REST-Poll | Reine Anzeige über HR-gated Read-Aggregat |
| Modal (F3/F5) | Frontend (Component) | State-Mapper | thin `From<&…TO>`-Mapper, keine Arithmetik |

## Standard Stack

**Keine neuen Pakete.** Phase 55 nutzt ausschließlich die bereits im Workspace vorhandenen
Crates und Dependencies. Alle relevanten Bausteine sind bereits `Cargo.toml`-verdrahtet.

### Core (bereits vorhanden, wird konsumiert)
| Baustein | Ort | Zweck | Warum Standard |
|----------|-----|-------|----------------|
| `RebookingBatchService` / `RebookingBatchDao` | `service{,_impl}/src/rebooking_batch.rs`, `dao{,_impl_sqlite}/src/rebooking_batch.rs` | Batch+Entry-Persistenz, UNIQUE-Slot-Autorität | Phase-54-Basis [VERIFIED: codebase] |
| `ExtraHoursService::create` | `service/src/extra_hours.rs:286` | Erzeugt einzelne ExtraHours-Row inkl. `source` | Standard-CRUD-Pfad [VERIFIED: codebase] |
| `ExtraHoursSource::{Manual, Rebooking}` | `service/src/extra_hours.rs:59` | Rebooking-Marker | Phase-54-Datenmodell, `#[default] Manual` [VERIFIED: codebase] |
| `ReportingService::get_report_for_employee_range` | `service_impl/src/reporting.rs` | balance + volunteer_hours-Aggregat | Single source of truth für F1-Ist + Balance [VERIFIED: codebase] |
| `VoluntaryStatsService::get_voluntary_stats` | `service_impl/src/voluntary_stats.rs:90` | F1/F2 Ist/Soll/Delta | HR-only Read-Aggregat für IST/DANN-Modal [VERIFIED: codebase] |
| `gen_service_impl!` Macro | `service_impl/src/lib.rs` | DI-Boilerplate für neuen BL-Service | Projekt-Standard [VERIFIED: codebase] |
| `TransactionDao::{use_transaction, commit}` | überall | Transaktions-Klammer | Projekt-Standard [CITED: CLAUDE.md] |

### Supporting (Frontend, bereits vorhanden)
| Baustein | Ort | Zweck | Wann nutzen |
|----------|-----|-------|-------------|
| `extra_hours_modal.rs` / `absence_convert_modal.rs` | `shifty-dioxus/src/component/` | Modal-Muster | Vorbild für `manual_rebooking_modal` + `rebooking_suggestion_modal` [VERIFIED: codebase] |
| `impersonation_banner.rs` / `warning_list.rs` | `shifty-dioxus/src/component/` | dauerhafte inline Warn-/Banner-Zeile | Vorbild für `rebooking_alert_banner` [VERIFIED: codebase] |
| `loader.rs` + `api`-Modul | `shifty-dioxus/src/` | HTTP-Loader-Konvention (`loader` → `api::get_*/post_*`) | 4 neue Loader delegieren an `api` [VERIFIED: codebase] |
| `state/*.rs` `From<&…TO>`-Mapper | `shifty-dioxus/src/state/` | thin DTO→State-Mapping | `state/rebooking.rs` (neu, keine Arithmetik) [VERIFIED: codebase] |
| `TopBar {}`-Component | `shifty-dioxus/src/page/employee_details.rs:141` | Action-Menu-Host | F3-Button-Placement (D-55-06) [VERIFIED: codebase] |

### Alternatives Considered
| Statt | Könnte man | Tradeoff |
|-------|-----------|----------|
| Neutralitäts-Filter im `ReportingService` | Nachträgliche Korrektur-Row / separater „net"-Aggregat-Pass | Verletzt „single source of truth"; verstreute Doppel-Zähl-Logik → Pitfall 1-Regression-Risiko |
| state-conditional UPDATE (`WHERE state='pending'`) | Optimistic-Lock mit `AND version=?` | Zusätzliche Komplexität ohne Gewinn — `state` ist der harte Diskriminator (siehe Open-Question 3) |
| DTO-additiv (`has_pending_rebooking`) | Separate Sammel-API `GET /rebooking-suggestions/summary` | Mehr Round-Trips; D-55-02 hat DTO-additiv gepinnt (Präzedenz VAA-04) |

**Installation:** — (keine)

**Version verification:** Nicht anwendbar — keine externen Pakete werden hinzugefügt.

## Package Legitimacy Audit

> Nicht anwendbar. Phase 55 installiert **keine** externen Pakete (npm/PyPI/crates). Alle
> genutzten Bausteine sind bereits im Workspace vorhanden und durch Phase 54 (bzw. frühere
> Phasen) verifiziert.

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```
F3 (Manuelle Umbuchung)                     F5 (HR-Alert + Modal)
──────────────────────                      ──────────────────────
Employee-Details TopBar                     Employee-Overview (/employees)
  │ (D-55-06 Action-Menu)                     │ dauerhafte Warnzeile pro SP
  ▼                                           │ (has_pending_rebooking ODER
manual_rebooking_modal                        │  Alert-Predicate true)
  (Woche + Menge + Richtung + Vorschau)       ▼
  │                                         Klick → rebooking_suggestion_modal
  │ POST /rebooking/manual                    (IST/DANN-Table, alle DANN backend-computed)
  │                                           │  approve → POST /rebooking-suggestions/{id}/approve
  │                                           │  reject  → POST /rebooking-suggestions/{id}/reject
  ▼                                           ▼
┌───────────────────────────────────────────────────────────────────────┐
│  REST rest/src/rebooking.rs  (HR-gated, error_handler-Wrapper)         │
│    POST /rebooking/manual                                              │
│    GET  /rebooking-suggestions                                        │
│    POST /rebooking-suggestions/{id}/approve                          │
│    POST /rebooking-suggestions/{id}/reject                          │
└──────────────────────────────┬────────────────────────────────────────┘
                               ▼
┌───────────────────────────────────────────────────────────────────────┐
│  RebookingReconciliationService  (BL-Tier, NEU)                        │
│  EINE Transaktion (use_transaction → … → commit):                     │
│   ① proposed = min(|balance|, voluntary_ist)   [F5; F3 = HR-Eingabe]  │
│   ② ExtraHoursService::create(-N src-Kat, source=Rebooking)          │
│   ③ ExtraHoursService::create(+N ziel-Kat, source=Rebooking)         │
│   ④ RebookingBatchService::create(batch{kind,state=Approved},        │
│         entry{out_id, in_id, audit-snapshots})                       │
│   Approve/Reject (F5): DAO state-conditional UPDATE                   │
│         WHERE id=? AND state='pending' → rows_affected()==1 else 409  │
└───────┬───────────────────────┬───────────────────────┬───────────────┘
        ▼                       ▼                       ▼
ExtraHoursService        RebookingBatchService     ReportingService /
(Basic)                  + Dao (Basic, P54)        VoluntaryStatsService
        │                       │                  (balance, voluntary_ist,
        ▼                       ▼                   IST/DANN, Alert-Predicate)
   extra_hours-Tabelle    rebooking_batch(_entry)         │
   (source-Spalte)        (UNIQUE-Slot)                   ▼
                                              source==Rebooking-FILTER
                                              (VOL-ACCT-03 NEUTRALITÄT — NEU)
```

**Datenfluss-Lesart:** F3 und F5-Approve münden in **denselben** internen Write-Path des
`RebookingReconciliationService` (Pitfall 15, single write-path). Der einzige Unterschied ist
`kind` (Manual vs. HrSuggestion) und dass F5 zusätzlich eine vorher persistierte
`pending`-Batch-Row via state-conditional UPDATE auf `approved` hebt.

### Recommended Project Structure (neu/geändert)
```
service/src/rebooking_reconciliation.rs      # NEU: Trait (BL) + RebookingDirection-Enum
service_impl/src/rebooking_reconciliation.rs # NEU: Impl (gen_service_impl!), Tx-Orchestrierung
service_impl/src/rebooking_batch.rs          # +update_state (approve/reject), +find_pending_for_sales_person
service_impl/src/reporting.rs                # source==Rebooking-Filter + alert_predicate pure fn + short-report-Feld
dao/src/rebooking_batch.rs                    # +update_state_conditional, +find_pending_for_sales_person
dao_impl_sqlite/src/rebooking_batch.rs        # SQL: state-conditional UPDATE + rows_affected-Guard
rest/src/rebooking.rs                          # NEU: 4 Routes, #[utoipa::path], error_handler
rest-types/src/rebooking.rs                    # NEU: ManualRebookingRequestTO, RebookingSuggestionTO, RebookingBatchTO
rest-types/src/lib.rs                          # ShortEmployeeReportTO +has_pending_rebooking +pending_rebooking_id
shifty_bin/src/main.rs                          # DI: RebookingReconciliationService NACH allen Basic-Services + Route-Registrierung
shifty-dioxus/Dioxus.toml                       # +proxy /rebooking, /rebooking-suggestions
shifty-dioxus/src/loader.rs + api-Modul         # 4 neue Loader/API-Funktionen
shifty-dioxus/src/state/rebooking.rs            # NEU: thin From<&…TO>-Mapper
shifty-dioxus/src/component/{rebooking_alert_banner,rebooking_suggestion_modal,manual_rebooking_modal}.rs
shifty-dioxus/src/page/{employees,employee_details}.rs  # Banner-Einbindung + F3-Trigger
shifty-dioxus/i18n/{de,en,cs}/*.ftl             # Labels, Approve/Reject, Banner-Text, RebookingErrorSlotTaken
docs/features/F14-rebooking.{md,_de.md}         # F3/F5-Sektionen (Trigger, State-Machine, IST/DANN)
```

### Pattern 1: BL-Service Tx-Orchestrierung (Vorbild: `voluntary_stats` / `absence`)
**What:** Ein BL-Service öffnet EINE Transaktion, ruft mehrere Basic-Services/DAOs mit
`Some(tx.clone())` auf und committet am Ende. Wichtig: **Basic-Services rufen NUR mit dem
durchgereichten `tx` auf** — sonst öffnet jeder Basic-Service seine eigene Tx und committet
zu früh (kein Rollback über die Grenze).
**When to use:** Cross-Entity-Invariante, die atomar sein muss.
**Example:**
```rust
// Muster aus service_impl/src/rebooking_batch.rs (Phase 54) — belegt use_transaction/commit-Klammer
async fn rebook(&self, /* … */ tx: Option<Self::Transaction>) -> Result<…, ServiceError> {
    self.permission_service.check_permission(HR_PRIVILEGE, context.clone()).await?;
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // ① ExtraHours -N (source=Rebooking)  — ExtraHoursService::create(&eh_out, ctx, Some(tx.clone()))
    // ② ExtraHours +N (source=Rebooking)  — ExtraHoursService::create(&eh_in,  ctx, Some(tx.clone()))
    // ③ RebookingBatchService::create(&batch, &[entry{out_id, in_id, audit}], ctx, Some(tx.clone()))
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

### Pattern 2: state-conditional UPDATE + affected-rows-Guard (NEU in Phase 55)
**What:** Race-sicheres Approve/Reject: `UPDATE … SET state=?, approved=?, approved_by=?,
update_version=? WHERE id=? AND state='pending' AND deleted IS NULL`, danach
`result.rows_affected()` prüfen. `==1` → gewonnen; `==0` → schon von jemand anderem
approved/rejected → `ServiceError::Conflict` → HTTP 409.
**When to use:** Zwei-HR-Double-Approve-Race (Pitfall 12).
**Example:**
```rust
// dao_impl_sqlite/src/rebooking_batch.rs (NEU) — rows_affected()-API belegt in user_invitation.rs:225
let result = sqlx::query!(
    r#"UPDATE rebooking_batch
       SET state = ?, approved = ?, approved_by = ?, update_version = ?, update_process = ?
       WHERE id = ? AND state = 'pending' AND deleted IS NULL"#,
    new_state_str, approved_ts, approved_by, version_vec, process, id_vec
).execute(&mut *conn).await.map_db_error()?;
if result.rows_affected() != 1 {
    return Err(DaoError::/* Conflict-Variante */);
}
```
**Warnung:** Innerhalb der Approve-Tx wird die state-Transition ZUERST versucht; nur bei
`rows_affected()==1` folgen die ExtraHours-Pair-Writes. So ist garantiert, dass die
Pair-Rows nur genau einmal entstehen.

### Pattern 3: DTO-additive Erweiterung mit `#[serde(default)]` (Präzedenz VAA-04, Phase 53)
**What:** Bestehendes `ShortEmployeeReportTO` um zwei Felder erweitern; `#[serde(default)]`
hält Wire-Kompat für alte Konsumenten.
**Example:**
```rust
// rest-types/src/lib.rs:372 — erweitern
pub struct ShortEmployeeReportTO {
    pub sales_person: SalesPersonTO,
    pub balance_hours: f32,
    /* … bestehende Felder … */
    #[serde(default)]
    pub volunteer_hours: f32,
    #[serde(default)]                       // NEU (D-55-02)
    pub has_pending_rebooking: bool,
    #[serde(default)]                       // NEU (D-55-02)
    pub pending_rebooking_id: Option<Uuid>,
}
```
Das `From<&ShortEmployeeReport>`-Impl (Zeile 384) muss die beiden neuen Felder mappen; die
Service-Struct `ShortEmployeeReport` (`service/src/reporting.rs:148`) bekommt die Felder
ebenfalls.

### Pattern 4: Alert-Predicate als pure fn + Truth-Table (Pitfall 11)
**What:** Das Predicate wird als testbare pure fn `alert_predicate(balance: f32,
voluntary_ist: f32, cap_active: bool) -> bool` implementiert — KEIN Dioxus-Browser-Test
(MEMORY `reference_dioxus_browser_test_date_inputs`). Truth-Table als Unit-Test-Matrix.
**Example (D-55-01 Grenzfälle):**
```rust
fn alert_predicate(balance: f32, voluntary_ist: f32, cap_active: bool) -> bool {
    cap_active && balance <= -0.5 && voluntary_ist > 0.0
}
// Truth-Table: balance=-0.49 → false ; -0.5 → true ; -0.51 → true ;
//              voluntary_ist=0.0 → false ; cap_active=false → false
```

### Anti-Patterns to Avoid
- **FE-Arithmetik für DANN-Werte:** Verstößt gegen Fat-Backend-Thin-Client. Alle IST/DANN +
  `proposed_hours` sind backend-computed (`RebookingSuggestionTO`).
- **Zwei getrennte Write-Paths für F3 und F5-Approve:** Verstößt gegen Pitfall 15. Ein
  gemeinsamer interner `rebook(...)` mit `kind`-Parameter.
- **Neutralität durch Korrektur-Row statt Filter:** Erzeugt eine dritte ExtraHours-Row zum
  „Ausgleich" → Doppel-Zähl-Risiko. Stattdessen `source`-Filter an der Aggregations-Site.
- **Basic-Service öffnet eigene Tx im BL-Kontext:** Immer `Some(tx.clone())` durchreichen.
- **Snapshot-Version-Bump ohne Not:** Phase 55 ändert keine `value_type`-Berechnung → **kein**
  Bump (siehe „Snapshot-Schema-Version" unten).

## Kritische Architektur-Entscheidung: VOL-ACCT-03 Neutralitäts-Filter

Dies ist die **einzige nicht-triviale neue Design-Frage** der Phase — der Planner MUSS sie
explizit fassen (empfohlen als eigener Sub-Plan mit vorangestelltem Property-Test).

**Befund [VERIFIED: codebase — reporting.rs].** Der `source`-Marker fließt bereits durch die
gesamte ExtraHours-Kette (`ExtraHours.source`, `From<&ExtraHoursEntity>` setzt ihn). ABER die
Aggregations-Sites in `service_impl/src/reporting.rs` filtern **noch nicht** nach `source`:
- `volunteer_hours`-Summierung: Zeilen ~894 (`manual_volunteer_hours`), ~1401, ~2263 — alle
  filtern nur `category == VolunteerWork`, NICHT `source`.
- Die `ExtraWork`-Summierung (die in balance via `ReportType::WorkingHours` einfließt) filtert
  ebenfalls nicht nach `source`.
- Die Kommentare in `voluntary_stats.rs` (Zeile 30–32) und `reporting.rs` (Zeile 184–186)
  sagen ausdrücklich: „Der source-Marker-Filter für Rebooking-Neutralität wird **ab Phase 55**
  zentral im ReportingService greifen."

**Semantik-Analyse (wichtig — der Planner darf das nicht falsch verdrahten):**

Ein Rebooking `VolunteerWork → ExtraWork` schreibt `-N VolunteerWork(Rebooking)` +
`+N ExtraWork(Rebooking)`. Betrachte die zwei betroffenen Aggregat-Familien getrennt:

1. **Voluntary-Read-Aggregate (F1-Ist / F2 via `VoluntaryStatsService` → `volunteer_hours`):**
   Hier MÜSSEN Rebooking-Rows **herausgefiltert** werden. Sonst zöge das `-N VolunteerWork`
   den `voluntary_ist` künstlich nach unten und verletzte „Ist bleibt unverändert"
   (VOL-ACCT-03). → **Filter: `volunteer_hours` zählt nur `source == Manual`.**

2. **Balance (`balance_hours`):** Hier ist die Frage subtiler. `VolunteerWork` fließt NICHT
   direkt in balance (`ReportType::Documented`), `ExtraWork` fließt als `WorkingHours` ein.
   - Wenn `ExtraWork(Rebooking)` NICHT gefiltert wird: balance steigt um +N — das ist der
     **beabsichtigte Real-Effekt** (Defizit schließen; das ist der ganze Zweck von F3/F5).
   - Der VOL-ACCT-03-Property-Test verlangt `balance_before == balance_after` für einen
     **Roundtrip** (`VolunteerWork→ExtraWork` UND wieder zurück), bzw. präziser: dass die
     **Voluntary-Read-Aggregate** neutral bleiben. Der Planner muss den Test so formulieren,
     dass er die tatsächlich geforderte Invariante prüft (siehe „Validation Architecture").

**Empfehlung (der Planner pinnt dies im Plan als explizite Entscheidung):** Filtere
`source == Rebooking` **nur** aus den **Voluntary-Read-Aggregaten** (`volunteer_hours` in allen
drei Sites + der `VoluntaryStatsService`-Kette, die darüber läuft). Lasse `ExtraWork(Rebooking)`
in `balance` einfließen (das ist die gewollte Wirkung). Der Property-Test „Roundtrip-Neutralität"
bucht dann hin **und** zurück und beweist, dass `voluntary_ist`, `voluntary_soll`, `voluntary_delta`
**und** `balance` nach dem vollständigen Roundtrip identisch zu vorher sind.

**Offene Detailfrage an den Planner (aus REQUIREMENTS.md wörtlich):** REQUIREMENTS VOL-ACCT-03
und ROADMAP Success-Criterion 5 formulieren den Property-Test als „Rebooking über eine Woche →
Ist/Soll/Delta unverändert (balance-neutral)". Das legt nahe, dass **schon eine einzelne
Umbuchung** (nicht erst der Roundtrip) balance-neutral sein soll. Falls das die Intention ist,
müssen **beide** Rebooking-Rows (`-N Volunteer` UND `+N ExtraWork`) aus balance gefiltert
werden — dann ist eine Einzel-Umbuchung neutral, und der „Effekt" entsteht nur dadurch, dass
die Rows im Report als bewusste Umbuchung sichtbar sind, aber die Kennzahlen nicht verschieben.
**Dies ist die zentrale zu klärende Semantik.** [ASSUMED — muss vom Planner/User bestätigt
werden; die CONTEXT.md-Constraint-Sektion sagt „Pair-ExtraHours sind negativ + positiv in
denselben Betrag; Marker-Filter im Reporting neutralisiert sie für die Read-Sicht" — das
spricht für **vollständige Filterung beider Rows aus allen Read-Aggregaten inkl. balance**.]

→ **Der Property-Test MUSS zuerst geschrieben werden** (TDD, `tdd_mode: true`) und die
gewünschte Invariante festnageln, BEVOR der Filter implementiert wird.

## Don't Hand-Roll

| Problem | Nicht selbst bauen | Stattdessen nutzen | Warum |
|---------|--------------------|--------------------|-------|
| Batch+Entry atomar schreiben | Eigenes Multi-INSERT | `RebookingBatchService::create` (P54) | Pre-Check + UNIQUE-Autorität + defensive id/version schon drin |
| UNIQUE-Slot-Kollision erkennen | Manueller SELECT-vor-INSERT im REST | `create` gibt `EntityAlreadyExists` → REST mappt 409 | Race-sicher via DB-Index |
| Einzelne ExtraHours-Row | Direkt-INSERT | `ExtraHoursService::create` | setzt id/version/created/source konsistent |
| balance/voluntary_ist berechnen | Eigene Neuberechnung | `ReportingService` / `VoluntaryStatsService` | single source of truth; sonst Drift |
| Transaktions-Klammer | Manuelles BEGIN/COMMIT | `TransactionDao::{use_transaction, commit}` | Projekt-Konvention, Rollback-Semantik |
| DTO→FE-State | Arithmetik im Mapper | thin `From<&…TO>` | Fat-Backend-Thin-Client |
| Race beim Approve | Read-then-write im Service | state-conditional UPDATE + `rows_affected()` | TOCTOU-frei nur auf DB-Ebene |

**Key insight:** Fast alles ist schon gebaut. Phase 55 ist zu ~70% Verdrahtung existierender
Bausteine; die einzigen genuin neuen Bausteine sind (a) der `source`-Neutralitäts-Filter,
(b) das state-conditional UPDATE, (c) der `RebookingReconciliationService` als Orchestrator,
(d) das `find_pending_for_sales_person`-DAO-Query, (e) die FE-Komponenten.

## Runtime State Inventory

> Phase 55 ist **kein** Rename/Refactor/Migration. Es ist eine additive Feature-Phase ohne
> Änderung an gespeicherten Keys, Service-Configs, OS-State, Secrets oder Build-Artefakten.

| Kategorie | Gefunden | Aktion |
|-----------|----------|--------|
| Stored data | Keine — neue `rebooking_batch`-Rows werden geschrieben, aber kein Key/keine Collection umbenannt. Bestehende `extra_hours` unangetastet. | none |
| Live service config | Keine — kein n8n/externe Config berührt. | none |
| OS-registered state | Keine — kein Cron/Task (F4 ist Phase 56). | none |
| Secrets/env vars | Keine. | none |
| Build artifacts | Keine — kein Package-Rename, keine neue Migration. `sqlx`-Query-Cache (`.sqlx/`) muss ggf. für neue Queries regeneriert werden (`cargo sqlx prepare`). | `cargo sqlx prepare` nach neuen Queries |

## Common Pitfalls

### Pitfall 1: Doppel-Zählung der Rebooking-Rows (der zentrale Fallstrick)
**What goes wrong:** Wird der `source`-Filter vergessen, zählen die `-N/+N`-Pair-Rows in
Reports/Balance/Voluntary-Ist doppelt bzw. verschieben die Kennzahlen entgegen VOL-ACCT-03.
**Why:** Die Aggregations-Sites in `reporting.rs` filtern aktuell nur nach `category`, nicht
nach `source` (verifiziert).
**How to avoid:** `source`-Filter an ALLEN relevanten Aggregations-Sites; Property-Test zuerst.
**Warning signs:** `voluntary_ist_before != voluntary_ist_after` nach einer Umbuchung.

### Pitfall 6/7: UNIQUE-Slot-TOCTOU (F3/F5-Kollision) & Claim-on-Suggest
**What goes wrong:** Zwei parallele Buchungen für `(sp, iso_year, iso_week)`; oder F5-Suggest
belegt Slot, F3 will dieselbe Woche.
**Why:** UNIQUE-Partial-Index ist global über alle `kind`s.
**How to avoid:** Auf `EntityAlreadyExists` → HTTP 409 + i18n `RebookingErrorSlotTaken`
(D-55-05). Nicht auto-overwrite. Der Index ist die Autorität, nicht der Pre-Check.

### Pitfall 11: FE-Predicate-Test statt pure fn
**What goes wrong:** Alert-Predicate wird in einem Dioxus-Browser-Test verifiziert → flaky,
langsam, schwer zu warten (MEMORY `reference_dioxus_browser_test_date_inputs`).
**How to avoid:** `alert_predicate` als pure fn im Backend + Truth-Table-Unit-Test (D-55-01).

### Pitfall 12: Double-Approve-Race
**What goes wrong:** Zwei HR approven denselben `pending`-Batch gleichzeitig → zwei
ExtraHours-Pairs.
**How to avoid:** state-conditional UPDATE `WHERE state='pending'` + `rows_affected()==1`; der
Zweite bekommt 409. Reihenfolge: state-UPDATE ZUERST, ExtraHours-Writes nur bei Erfolg.

### Pitfall 14: Fehlender Dioxus.toml-Proxy für neue Routes
**What goes wrong:** Neue Backend-Routes ohne Proxy-Eintrag → 404/CORS im Dev-Frontend
(MEMORY `feedback_dioxus_proxy_for_new_backend_endpoints`).
**How to avoid:** In `shifty-dioxus/Dioxus.toml` je einen `[[web.proxy]] backend =
"http://localhost:3000/rebooking"` und `/rebooking-suggestions` ergänzen (Muster: bestehende
Einträge Zeile 45+).

### Pitfall 15: Zwei Write-Paths für dieselbe Semantik
**What goes wrong:** F3 und F5-Approve implementieren die Doppel-Eintrag-Logik getrennt → Drift.
**How to avoid:** Ein interner `rebook(direction, hours, week, kind)` im
`RebookingReconciliationService`, den beide Pfade aufrufen.

### Pitfall (Clippy-Gate)
**What goes wrong:** `cargo test` grün, aber `nix build`/CI rot wegen Clippy-Warnung.
**How to avoid:** Jedes Test-Gate MUSS `cargo clippy --workspace -- -D warnings` ausführen
(CLAUDE.md, hard gate). Auch im Frontend-Crate.

## Code Examples

### `RebookingDirection`-Enum (neu, im Trait-Modul)
```rust
// service/src/rebooking_reconciliation.rs (NEU)
// D-55-06: Modal wählt Richtung explizit (kein Delta-Vorzeichen-Ableiten im TopBar-Trigger).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebookingDirection {
    VolunteerToExtra,  // -N VolunteerWork, +N ExtraWork (Standard-Ausgleich)
    ExtraToVolunteer,  // -N ExtraWork, +N VolunteerWork (Korrektur überzahlter Zusagen)
}
```

### DI-Wiring (Reihenfolge!)
```rust
// shifty_bin/src/main.rs — RebookingReconciliationService NACH ExtraHoursService,
// RebookingBatchService, ReportingService/VoluntaryStatsService (alle bereits konstruiert).
pub struct RebookingReconciliationServiceDependencies;
impl service_impl::rebooking_reconciliation::RebookingReconciliationServiceDeps
    for RebookingReconciliationServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type ExtraHoursService = ExtraHoursService;
    type RebookingBatchService = RebookingBatchService;
    type ReportingService = ReportingService;        // für balance/voluntary_ist + proposed_hours
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
```

## State of the Art

| Alt | Neu | Wann | Impact |
|-----|-----|------|--------|
| ExtraHours-Aggregate ohne source-Filter (Phase 54) | source==Rebooking-Filter in Read-Aggregaten | Phase 55 | VOL-ACCT-03-Neutralität aktiv |
| `rebooking_batch` nur Datenmodell (kein Konsument) | F3/F5-Konsumenten schreiben Batches | Phase 55 | Erste echte Batch-Erzeugung |
| Kein state-conditional UPDATE in der Codebase | Erstes Optimistic-State-Transition-Muster | Phase 55 | Wiederverwendbar für spätere Approval-Flows |

**Deprecated/outdated:** Nichts. Die entfernte Range-basierte Voluntary-Ist-pure-fn
(reporting.rs:177) ist bereits Phase-54-Historie und NICHT wiederzubeleben.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | VOL-ACCT-03-Property-Test verlangt Neutralität schon bei **Einzel-Umbuchung** (nicht erst Roundtrip) → beide Rebooking-Rows aus **allen** Read-Aggregaten inkl. balance filtern | Kritische Architektur-Entscheidung | Falscher Filter-Scope → Property-Test scheitert oder balance-Semantik falsch. **Muss vom User/Planner bestätigt werden.** CONTEXT-Constraint spricht klar für vollständige Filterung. |
| A2 | state-only UPDATE (`WHERE state='pending'`) genügt, kein `AND version=?` | Pattern 2 / Open Questions | Bei Fehleinschätzung theoretisch ABA-Problem — praktisch ausgeschlossen, da Batch one-shot (D-55-04, kein Undo) |
| A3 | F3-Modal minimalistisch (Menge+Richtung+Woche, ohne DANN-Duplikat) | Open Questions | Nur UX-Umfang; kein Datenrisiko |
| A4 | Kein Snapshot-Schema-Bump nötig (12 bleibt) | Snapshot-Sektion | Falls doch ein `value_type` berührt wird → Snapshot-Drift. Analyse unten begründet „Nein". |
| A5 | `security_enforcement`/`nyquist_validation` nicht in config.json → beide als **enabled** behandelt | Validation/Security | Falls anders gemeint, überflüssige Sektionen — kein Schaden |

## Open Questions (RESOLVED)

> Alle Open Questions sind durch die Discuss-Phase-Entscheidungen (CONTEXT.md D-55-01…07) und
> die Planner-Pins (55-01…55-03) aufgelöst. Auflösung pro Frage:

1. **Q1 — Neutralitäts-Filter-Scope (balance ja/nein).** → **RESOLVED via D-55-A1.** Beide
   Rebooking-Rows (`-N` UND `+N`) werden aus ALLEN Read-Aggregaten inkl. `balance` gefiltert
   (VolunteerWork- UND ExtraWork/WorkingHours-Sites). Eine EINZELNE manuelle Umbuchung ist
   damit balance/ist/soll/delta-neutral; die Umbuchung wird nur als Audit-Batch sichtbar. Der
   VOL-ACCT-03-Property-Test (55-02 Task 1, TDD-first/RED) beweist diese Invariante empirisch,
   bevor der Filter entsteht. (Gepinnt in 55-02 `<pinned_decisions>` D-55-A1.)

2. **Q2 — F3-Modal-Vorschau-Umfang.** → **RESOLVED: minimalistisch** (Menge + Richtung + Woche
   + Text-Vorschau, ohne DANN-Duplikat). HR kennt den Kontext; die DANN-Rechnung ist nur für
   das F5-Alert-/Suggestion-Modal nötig. (Umgesetzt in 55-04 `manual_rebooking_modal`.)

3. **Q3 — state-conditional UPDATE: `version`-Guard nötig?** → **RESOLVED: Nein** —
   `WHERE state='pending'` reicht, kein `AND version=?`. Der State ist der harte Diskriminator,
   der Batch ist one-shot (D-55-04, kein Undo); `rows_affected()==1` fängt den Race ab. (Gepinnt
   als Prohibition in 55-01, D-55-A2.)

**Zusätzlich RESOLVED — F5-Lifecycle:** Das F5-HR-Alert-Modell ist **Modell B (Claim-on-Suggest)**,
via User-Entscheidung gepinnt (55-01/02/03 `<pinned_decisions>` D-55-B). HR öffnet das F5-Modal →
`suggest` erzeugt via `create_pending_suggestion` einen `rebooking_batch(kind=HrSuggestion,
state=Pending)` und claimt den UNIQUE-Slot (der bisher fehlende pending-Batch-Producer) → HR
entscheidet via state-conditional UPDATE (approve → Pair-ExtraHours; reject → nur State). Der
Banner zeigt sich bei true-Prädikat auch OHNE offenen pending-Batch (der alte
"Predicate true ∧ kein Batch → has_pending_rebooking=false"-Case war der Bug). Der pending-Batch
ist zugleich das, was der Phase-56-Cron zum Skippen des Slots liest (HR-ALERT-04, ROADMAP §Phase 56).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust-Toolchain + `cargo` | gesamte Phase | ✓ (Projekt läuft) | — | — |
| `sqlx` CLI (`cargo sqlx prepare`) | neue compile-time-checked Queries | ✓ (Projekt nutzt SQLx überall) | — | lokale DB via `sqlx setup` |
| Dioxus-Toolchain (`dx`) | FE-Build/Dev-Proxy | ✓ (bestehendes FE-Crate) | — | — |
| SQLite (in-memory für Tests) | Integration-Tests | ✓ (bestehende Test-Infra) | — | — |

**Missing dependencies with no fallback:** keine.
**Missing dependencies with fallback:** keine.

## Snapshot-Schema-Version: KEIN Bump (12 bleibt)

**Analyse [VERIFIED: codebase + CLAUDE.md-Klausel].** `CURRENT_SNAPSHOT_SCHEMA_VERSION = 12`
(`service_impl/src/billing_period_report.rs:117`). Ein Bump ist laut CLAUDE.md nur nötig bei
Änderung an persistierten `billing_period_sales_person`-`value_type`s oder deren Input-Set.

Phase 55 (F3/F5):
- Fügt **keinen** neuen `BillingPeriodValueType` hinzu (ROADMAP „Out of Scope" bestätigt:
  Rebooking schreibt existierende Kategorien `VolunteerWork`/`ExtraWork`).
- Der Snapshot-Builder (`build_and_persist_billing_period_report`) liest nur `Manual`-Rows in
  seine value_types (der Rebooking-Filter neutralisiert die neuen Rows für die Read-Sicht —
  genau der Punkt von VOL-ACCT-03). Damit **ändert sich das Input-Set des Snapshots nicht**:
  eine Neu-Berechnung eines alten Snapshots ergäbe denselben Wert, weil Rebooking-Rows
  herausgefiltert werden.
- Kommentar in `billing_period_report.rs:369` bestätigt bereits: „KEIN value_type-Change ->
  KEIN CURRENT_SNAPSHOT_SCHEMA_VERSION-Bump".

**Konsequenz:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **12**. Die Bump-Frage für die
F4-Cron-schreibende Quelle wird in Phase 56 gepinnt (REB-AUTO-05, STATE.md Zeile 191). Damit
der „Nein"-Zweig belastbar ist, sollte der Property-Test implizit mitzeigen, dass eine
Rebooking-Umbuchung die Snapshot-relevanten value_types nicht verschiebt (folgt aus VOL-ACCT-03).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` (`#[tokio::test]` async) + `mockall` (Unit) + in-memory SQLite (Integration) |
| Config file | keine separate — Cargo-nativ |
| Quick run command | `cargo test -p service_impl rebooking` |
| Full suite command | `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REB-MANUAL-01 | Atomarer Doppel-Eintrag (2×EH + Batch) in einer Tx | Integration | `cargo test -p service_impl rebooking_reconciliation_creates_pair` | ❌ Wave 0 (`service_impl/src/test/rebooking_reconciliation.rs`) |
| REB-MANUAL-02 | Beide Richtungen (Volunteer↔Extra) | Unit/Integration | `cargo test -p service_impl rebooking_direction` | ❌ Wave 0 |
| REB-MANUAL-03 | Modal-Vorschau (FE) | manuell/UAT | — (Fat-Backend: `proposed_hours`-fn testbar) | ❌ Wave 0 |
| HR-ALERT-01 | Alert-Predicate `balance<=-0.5 ∧ vol_ist>0 ∧ cap` | Unit (pure fn) | `cargo test -p service_impl alert_predicate` | ❌ Wave 0 |
| HR-ALERT-02 | IST/DANN backend-computed (`RebookingSuggestionTO`) | Integration | `cargo test -p service_impl suggestion_dann_values` | ❌ Wave 0 |
| HR-ALERT-03 | Approve/Reject state-conditional (Race) | Integration | `cargo test -p service_impl approve_double_race_conflict` | ❌ Wave 0 |
| HR-ALERT-04 | UNIQUE-Slot-Claim (409 bei Kollision) | Integration | `cargo test -p service_impl rebooking_slot_taken_409` | ❌ Wave 0 |
| VOL-ACCT-03 | Roundtrip-Neutralität (balance/ist/soll/delta) | Property/Integration | `cargo test -p service_impl rebooking_neutrality` | ❌ Wave 0 (TDD zuerst!) |
| D-55-01 | Grenzfälle -0.49/-0.5/-0.51 | Unit | `cargo test -p service_impl alert_predicate_boundary` | ❌ Wave 0 |
| D-55-03 | `min(\|balance\|, voluntary_ist)`-Matrix | Unit | `cargo test -p service_impl proposed_hours` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p service_impl rebooking` + `cargo clippy --workspace -- -D warnings`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite grün + Clippy grün vor `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `service_impl/src/test/rebooking_reconciliation.rs` — REB-MANUAL-*, HR-ALERT-03/04, VOL-ACCT-03
- [ ] Pure-fn-Tests für `alert_predicate` + `proposed_hours` (in `reporting.rs`-Test-Modul oder eigenem)
- [ ] `service_impl/src/test/mod.rs` — neues Test-Modul registrieren
- [ ] Mock-Setup: `MockRebookingBatchService`, `MockExtraHoursService` sind via `automock` schon vorhanden
- [ ] **VOL-ACCT-03-Property-Test ZUERST** (tdd_mode) — nagelt die Neutralitäts-Invariante fest bevor der Filter entsteht

## Security Domain

> `security_enforcement` nicht explizit in config.json → als enabled behandelt. Phase 55 ist
> eine Feature-Phase auf bestehendem HR-Gate-Auth; keine neuen Auth-/Krypto-Flächen.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | nein | Bestehendes OIDC/Mock-Auth, unverändert |
| V3 Session Management | nein | Unverändert |
| V4 Access Control | **ja** | Alle neuen Routes/Service-Methoden HR-gated via `PermissionService::check_permission(HR_PRIVILEGE, …)` — an ERSTER Stelle jeder Methode (Präzedenz `rebooking_batch.rs:47`) |
| V5 Input Validation | **ja** | `ManualRebookingRequestTO`: `hours>0`, gültige `iso_year`/`iso_week`, gültige `RebookingDirection`; SQLx compile-time-checked Queries (kein SQL-Injection) |
| V6 Cryptography | nein | Keine Krypto |

### Known Threat Patterns for {Rust/Axum/SQLx}
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL-Injection | Tampering | SQLx `query!`/`query_as!` (parametrisiert, compile-time-checked) |
| Double-Approve-Race | Tampering | state-conditional UPDATE + `rows_affected()==1` (Pitfall 12) |
| UNIQUE-Slot-TOCTOU | Tampering | DB-UNIQUE-Partial-Index als Autorität (Pitfall 6) |
| Privilege-Escalation (Non-HR bucht) | Elevation of Privilege | HR-Gate an erster Stelle jeder Service-Methode + API-Level-Redaktion (Non-HR sieht `has_pending_rebooking=false`) |
| Audit-Umgehung (Reject spurlos) | Repudiation | Rejected-Batch bleibt persistent (D-55-04, Audit-Trail) |

## Project Constraints (from CLAUDE.md)

- **Tests Pflicht** für alle Änderungen (globale + Projekt-Regel). `tdd_mode: true` → Test zuerst.
- **Clippy hard gate:** `cargo clippy --workspace -- -D warnings` in JEDEM Test-Gate (nicht durch
  `cargo test` abgedeckt). CI + `nix build` erzwingen es.
- **Transaktions-Muster:** Jede Service-Methode akzeptiert `Option<Transaction>`;
  `use_transaction` → business logic → `commit`.
- **Service-Tier-Konvention:** `RebookingReconciliationService` = **Business-Logic** (konsumiert
  mehrere Domain-Services) → DI in `main.rs` NACH allen Basic-Services konstruieren, keine
  `OnceLock`-Tricks. `RebookingBatchService` bleibt Basic (nur DAO+Permission+Clock+Uuid+Tx).
- **OpenAPI:** neue Routes brauchen `#[utoipa::path]` + `ToSchema` auf den TOs + Eintrag in
  `ApiDoc`.
- **DAO-Konvention:** alle Queries `WHERE deleted IS NULL`; `TryFrom`-Row-Konversion.
- **Docs-Freshness-Gate (hart):** `service/**`, `dao/**`, `main.rs`, `migrations/**` berührt →
  passende `docs/`-Sektion im selben Commit/PR. Konkret hier: `service/**` (neue Traits/Enum),
  `service_impl/reporting.rs` (Balance-/Kategorie-Semantik → `docs/features/F07-*` +
  `docs/domain/time-accounting.md`), `main.rs` (DI → `docs/architecture/02-service-tiers.md` +
  `service-graph-runtime.mmd`), und **`docs/features/F14-rebooking.{md,_de.md}`** (existieren
  bereits als Phase-54-Stub, Status „partially shipped") bekommen die F3/F5-Sektionen. EN **und**
  DE synchron halten (sonst Drift).
- **Snapshot-Schema-Versioning:** KEIN Bump in Phase 55 (siehe Analyse oben) — begründet, nicht
  vergessen.

## Sources

### Primary (HIGH confidence)
- Codebase-Direktinspektion: `service/src/rebooking_batch.rs`, `service_impl/src/rebooking_batch.rs`,
  `dao/src/rebooking_batch.rs`, `service/src/voluntary_stats.rs`, `service/src/extra_hours.rs`,
  `service_impl/src/reporting.rs` (Aggregations-Sites 894/1401/2263), `rest-types/src/lib.rs:372`,
  `service/src/reporting.rs:148`, `shifty_bin/src/main.rs` (DI 288–351),
  `migrations/sqlite/20260707000000_create-rebooking-batch.sql`,
  `service_impl/src/billing_period_report.rs:117,369`, `dao_impl_sqlite/src/user_invitation.rs:225`,
  `shifty-dioxus/Dioxus.toml`, `shifty-dioxus/src/{loader,component,state,page}/…`.
- `.planning/phases/55-manual-rebooking-hr-alert/55-CONTEXT.md` (autoritativ, D-55-01…07).
- `.planning/ROADMAP.md` (Phase-55-Sektion), `.planning/REQUIREMENTS.md` (REB-MANUAL/HR-ALERT/VOL-ACCT-03),
  `.planning/STATE.md` (Snapshot-Version 12), `docs/features/F14-rebooking.md` (Stub).
- `CLAUDE.md` (Root, Projekt-Konventionen).

### Secondary (MEDIUM confidence)
- Phase-54-Artefakte (`54-RESEARCH.md`/`54-*-SUMMARY.md`) für Datenmodell-Kontext.

### Tertiary (LOW confidence)
- Keine — es war keine externe Web-/Paket-Recherche nötig.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — vollständig aus Codebase verifiziert, keine externen Pakete.
- Architecture: HIGH — Muster (BL-Tx-Orchestrierung, gen_service_impl!, HR-Gate) direkt belegt;
  einzige neue Design-Frage (Neutralitäts-Filter-Scope) explizit als A1/Open-Question markiert.
- Pitfalls: HIGH — alle aus ROADMAP-Pitfall-Match + Codebase-Befunden (fehlender source-Filter,
  fehlendes state-conditional UPDATE) bestätigt.

**Research date:** 2026-07-23
**Valid until:** 2026-08-22 (30 Tage — stabile In-Codebase-Domäne; Neubewertung nur bei
Änderung an `reporting.rs`-Aggregation oder `rebooking_batch`-Schema).
