# Phase 40: Wochen-Sperre durchsetzen (BE+FE) — Research

**Recherchiert:** 2026-07-02
**Domäne:** Rust Backend — Service-Layer Lock-Gate; Axum REST; Dioxus 0.6 Frontend
**Konfidenz:** HIGH (alle Befunde aus Code-Verifikation im aktuellen Tree)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-40-01:** `ServiceError::WeekLocked { year, week }` mappt auf **HTTP 423 Locked** (nicht 409). Erster 423-Fall im Codebase; Compiler erzwingt den neuen Arm durch exhaustives Match.
- **D-40-02:** Harte Sperre inkl. Entfernen/Selbst-Ausbuchen. Keine Self-Service-Ausnahme. Bypass nur für `shiftplan.edit`-Holder (Schichtplaner, transitiv Admin).
- **D-40-03:** Proaktives Ausblenden der +/− Buttons für Nicht-Schichtplaner bei `week_status == Locked`. Server-Gate bleibt die eigentliche Durchsetzung.
- **D-40-04:** Kein Banner. Das rote „Gesperrt"-Badge aus Phase 39 + fehlende Buttons reichen als UI-Signal. Der 423-Pfad ist reines Sicherheitsnetz ohne eigene FE-Reaktion.
- **D-40-05:** i18n-Meldung in de/en/cs für die 423-Antwort.

### Claude's Discretion

- Exakter Name/Signatur des `assert_week_not_locked`-Helpers (freie Funktion vs. Methode auf Impl), solange die tx-Invariante gilt.
- Signatur/Verhalten der neuen `ShiftplanEditService::delete_booking`-Methode.
- Wie WeekStatusService als neue Dep in ShiftplanEditServiceDeps verdrahtet wird.
- Wortlaut der de/en/cs-Sperr-Meldung (Vorschlag in diesem Dokument).

### Deferred Ideas (OUT OF SCOPE)

- Bulk-KW-Sperre (WST-06)
- Publish-Notification (WST-07)
- Sperre weiterer Nicht-Shiftplan-Schreibpfade (Absence/Unavailable)
</user_constraints>

---

## Summary

Phase 40 ist eine **chirurgische Erweiterung** bestehender Service-Methoden: kein neues Datenmodell, kein neues REST-Endpoint-Routing für Slots, nur ein neuer Lock-Gate-Aufruf an sechs definierten Einfügestellen im Business-Logic-Tier. Die technische Basis (WeekStatus-Service, DAO, REST, Frontend-Signal) ist durch Phase 39 vollständig geliefert.

Der zentrale Baustein ist eine private `async fn assert_week_not_locked`-Methode auf `ShiftplanEditServiceImpl`, die den Shiftplanner-Bypass intern klärt, dann per `WeekStatusService::get_week_status` (innerhalb der bereits offenen Transaktion) den Status liest und bei `Locked` + Nicht-Schichtplaner `ServiceError::WeekLocked { year, week }` zurückgibt. Das erschöpfende `match` in `error_handler` erzwingt zur Compile-Zeit den neuen 423-Arm.

Das Frontend ergänzt nur einen `else if`-Zweig in der `button_mode`-Berechnung in `src/page/shiftplan.rs`. Alle dafür nötigen Signale (`week_status`, `is_shiftplanner`) sind bereits vorhanden.

**Primäre Empfehlung:** Alle sechs Einfügestellen aus einem einzigen Helper, WeekStatusService als neue Dep in ShiftplanEditServiceDeps (laut Enum-Bindung `<Context = Self::Context, Transaction = Self::Transaction>`), neuer `ServiceError::WeekLocked`-Arm mit HTTP 423 in `error_handler`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| Lock-Gate bei Schreibpfaden | Business-Logic (ShiftplanEditService) | — | Liegt nach Permission-Check; kennt is_shiftplanner für Bypass |
| Lock-Status lesen | Basic-Tier (WeekStatusService) | — | Bereits in Phase 39 implementiert; kein Privileg-Gate auf Read |
| HTTP 423 Mapping | REST (error_handler) | — | Exhaustives Match erzwingt Arm; einziger Ort für Error→HTTP |
| FE: Buttons ausblenden | Frontend (shiftplan.rs, button_mode) | — | Proaktiv, UX-only; Server-Gate ist die eigentliche Durchsetzung |
| delete_booking Bypass schließen | Business-Logic (ShiftplanEditService) | — | Einziger Nicht-Schichtplaner-Bypass (WST-04) |

---

## Verifizierte Code-Befunde (alle aus aktuellem Tree gelesen)

### 1. assert_week_not_locked — Signatur und Ort

**Empfehlung:** Private `async fn` Methode auf `ShiftplanEditServiceImpl<Deps>`, im bestehenden `impl`-Block mit `count_paid_bookings_in_slot_week` (ca. `service_impl/src/shiftplan_edit.rs:826`). [VERIFIED: codebase]

```rust
// service_impl/src/shiftplan_edit.rs — helper impl block
async fn assert_week_not_locked(
    &self,
    year: u32,
    calendar_week: u8,
    context: Authentication<Deps::Context>,
    tx: Deps::Transaction,
) -> Result<(), ServiceError> {
    // Shiftplanner bypass: check_permission gibt Ok → sofort return (D-40-02).
    if self.permission_service
        .check_permission(SHIFTPLANNER_PRIVILEGE, context)
        .await
        .is_ok()
    {
        return Ok(());
    }
    // Woche in DERSELBEN Transaktion lesen wie der Write → kein TOCTOU (SC4).
    let status = self
        .week_status_service
        .get_week_status(year, calendar_week, Authentication::Full, Some(tx))
        .await?;
    if status == service::week_status::WeekStatus::Locked {
        return Err(ServiceError::WeekLocked { year, week: calendar_week });
    }
    Ok(())
}
```

**Warum Methode statt freie Funktion:** Braucht `self.permission_service` und `self.week_status_service` — beide bereits via Deps zugänglich. Freie Funktion würde beide Arc-Refs als Parameter erfordern und wäre schwerer zu mocken.

**TOCTOU-Garantie:** `tx` wird via `self.transaction_dao.use_transaction(tx)` in der aufrufenden Methode geöffnet. Der Helper erhält `tx.clone()` (Arc-geteilt) — derselbe DB-Connection-Scope. `WeekStatusService::get_week_status` öffnet keine neue Transaktion wenn `Some(tx)` übergeben wird; der interne `commit`-Aufruf in `get_week_status` ist auf der Arc-geclonten tx ein No-Op (Pattern identisch mit `booking_service.get_for_slot_id_since(..., Some(tx.clone()))` in `modify_slot`). [VERIFIED: codebase — service_impl/src/shiftplan_edit.rs:84-89, service_impl/src/week_status.rs:52-57]

**WeekStatusService-Methode:** `get_week_status(year: u32, calendar_week: u8, context: Authentication<Self::Context>, tx: Option<Self::Transaction>) -> Result<WeekStatus, ServiceError>`. Kein Privileg-Gate auf Read (T-39-03). [VERIFIED: codebase — service/src/week_status.rs:38-44]

---

### 2. Einfügestellen — alle 5 bestehenden Schreibmethoden

#### 2.1 modify_slot (change_year, change_week explizit)

```
service_impl/src/shiftplan_edit.rs:51-143
```

- `use_transaction(tx)` bei Zeile 59
- `check_permission("shiftplan.edit", context)` bei Zeilen 60-62
- **Lock-Gate einfügen nach Zeile 62**, vor `get_slot` (Zeile 64):

```rust
self.assert_week_not_locked(change_year, change_week, context.clone(), tx.clone()).await?;
```

(year, week) aus Methodenparametern `change_year: u32, change_week: u8` — in-hand. [VERIFIED: codebase]

#### 2.2 remove_slot (change_year, change_week explizit)

```
service_impl/src/shiftplan_edit.rs:145-197
```

- `use_transaction` bei Zeile 153, `check_permission` bei Zeilen 154-156
- **Lock-Gate nach Zeile 156**, vor `get_slot` (Zeile 158):

```rust
self.assert_week_not_locked(change_year, change_week, context.clone(), tx.clone()).await?;
```

[VERIFIED: codebase]

#### 2.3 modify_slot_single_week (change_year, change_week explizit)

```
service_impl/src/shiftplan_edit.rs:199-346
```

- `use_transaction` bei Zeile 208, `check_permission` bei Zeilen 210-213
- **Lock-Gate nach Zeile 213**, vor `get_slot` (Zeile 215):

```rust
self.assert_week_not_locked(change_year, change_week, context.clone(), tx.clone()).await?;
```

[VERIFIED: codebase]

#### 2.4 book_slot_with_conflict_check (year/week aus booking-Entity)

```
service_impl/src/shiftplan_edit.rs:551-757
```

- `use_transaction` bei Zeile 557
- Permission-Pattern (Shiftplanner ∨ Self) bei Zeilen 562-575:
  ```rust
  let is_shiftplanner = sp_perm.is_ok();  // Zeile 574
  sp_perm.or(self_perm)?;                  // Zeile 575
  ```
- **Lock-Gate nach Zeile 575**, da `is_shiftplanner` bereits bekannt (Effizienz — vermeidet redundanten `check_permission`-Aufruf im Helper):

```rust
if !is_shiftplanner {
    self.assert_week_not_locked(
        booking.year,
        booking.calendar_week as u8,  // booking.calendar_week: i32, cast zu u8 safe (1–53)
        context.clone(),
        tx.clone(),
    ).await?;
}
```

(year, week) aus `booking.year: u32` und `booking.calendar_week: i32`. [VERIFIED: codebase — service/src/booking.rs:17-18]

**Hinweis für Schichtplaner:** Da `is_shiftplanner = true` den Gate-Aufruf überspringt, ist die Bypass-Semantik hier ohne redundanten `check_permission` sichergestellt.

#### 2.5 copy_week_with_conflict_check (Ziel-Woche, nicht Quell-Woche)

```
service_impl/src/shiftplan_edit.rs:759-818
```

- `use_transaction` bei Zeile 768, `check_permission("shiftplan.edit", context)` bei Zeilen 773-775
- **Lock-Gate nach Zeile 775**, vor `get_for_week` (Zeile 778):

```rust
self.assert_week_not_locked(to_year, to_calendar_week, context.clone(), tx.clone()).await?;
```

**Quelle vs. Ziel:** `from_calendar_week/from_year` = Quell-Woche (kein Schreiben). `to_year/to_calendar_week` = Ziel-Woche (wird beschrieben) → **ausschließlich Ziel-Woche** prüfen.

**Bypass ist hier immer aktiv:** `copy_week_with_conflict_check` erfordert `shiftplan.edit` → der Helper kehrt für den Shiftplanner-Kontext sofort mit Ok zurück. Der Gate-Aufruf läuft, schreibt aber nie eine WeekLocked-Error. Trotzdem gesetzt: Konsistenz, Schutz gegen zukünftige Permission-Änderungen.

**Delegation zu book_slot_with_conflict_check:** `copy_week` ruft `book_slot_with_conflict_check` bei Zeile 807 mit dem Ziel-Booking (`calendar_week: to_calendar_week, year: to_year`). Da `is_shiftplanner = true` für alle `copy_week`-Aufrufer, überspringt der Gate-Check in `book_slot_with_conflict_check` den Lock-Check ebenfalls. [VERIFIED: codebase]

---

### 3. delete_booking — neue Methode (WST-04 Bypass-Schließung)

**Problem:** `DELETE /booking/{id}` ruft heute `booking_service().delete(booking_id, context.into(), None)` direkt auf. `BookingService` ist Basic-Tier (kein Lock-Gate). Ein Nicht-Schichtplaner kann so in einer gesperrten Woche seine eigene Buchung entfernen. [VERIFIED: codebase — rest/src/booking.rs:156-172]

#### Trait-Erweiterung

```rust
// service/src/shiftplan_edit.rs — dem bestehenden ShiftplanEditService-Trait hinzufügen
async fn delete_booking(
    &self,
    booking_id: Uuid,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<(), ServiceError>;
```

Da der Trait `#[automock]` trägt, generiert mockall automatisch `MockShiftplanEditService::delete_booking` — kein manueller Mock nötig.

#### Implementierung (Semantik-Erhalt von BookingService::delete)

```rust
// service_impl/src/shiftplan_edit.rs
async fn delete_booking(
    &self,
    booking_id: Uuid,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<(), ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;

    // is_shiftplanner bestimmen (für Bypass + Permission-Check)
    let is_shiftplanner = self
        .permission_service
        .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
        .await
        .is_ok();

    // Booking laden um (year, calendar_week) zu lesen (vor Delete!)
    let booking = self
        .booking_service
        .get(booking_id, Authentication::Full, Some(tx.clone()))
        .await?;

    // Lock-Gate: nur für Nicht-Schichtplaner (D-40-02)
    if !is_shiftplanner {
        self.assert_week_not_locked(
            booking.year,
            booking.calendar_week as u8,
            context.clone(),
            tx.clone(),
        ).await?;
    }

    // Delegation an BookingService::delete erhält Semantik (Permission Shiftplanner ∨ Self)
    self.booking_service
        .delete(booking_id, context, Some(tx.clone()))
        .await?;

    self.transaction_dao.commit(tx).await?;
    Ok(())
}
```

**Schlüsselreihenfolge:** `get` → `assert_week_not_locked` → `delete`. Der `get`-Aufruf vor dem Delete ist zwingend: ohne Entity gibt es kein `year`/`calendar_week` für den Lock-Check. Falls die booking_id nicht existiert, schlägt `get` mit `EntityNotFound` fehl, bevor der Lock-Gate greift.

**Permission-Erhalt:** `booking_service.delete(booking_id, context, Some(tx.clone()))` delegiert mit dem Original-Context (nicht `Authentication::Full`) — das Basic-Tier `BookingService::delete` führt seinen eigenen Permission-Check (Shiftplanner ∨ Self) aus. Die ShiftplanEditService-Schicht addiert nur den Lock-Gate, ändert keine Zugriffsregel.

#### Handler-Umrouting

```rust
// rest/src/booking.rs:156-172 — handler umstellen
#[instrument(skip(rest_state))]
pub async fn delete_booking<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
    Path(booking_id): Path<Uuid>,
) -> Response {
    error_handler(
        (async {
            rest_state
                .shiftplan_edit_service()  // war: booking_service()
                .delete_booking(booking_id, context.into(), None)  // war: .delete(...)
                .await?;
            Ok(Response::builder().status(200).body(Body::empty()).unwrap())
        })
        .await,
    )
}
```

`shiftplan_edit_service()` ist in `RestStateDef` bereits vorhanden (Zeile 427 in `rest/src/lib.rs`). [VERIFIED: codebase] Kein Change an Route oder RestStateDef nötig.

---

### 4. ServiceError::WeekLocked → HTTP 423

#### Neue Variante in service/src/lib.rs

```rust
// service/src/lib.rs — ServiceError-Enum, nach PaidLimitExceeded (~Zeile 130)
#[error("Week {year}/{week} is locked — changes are not possible")]
WeekLocked { year: u32, week: u8 },
```

#### Neuer Arm in rest/src/lib.rs error_handler

Da `error_handler` (Zeilen 134-286) exhaustiv matcht (kein `_`-Wildcard), erzwingt der Compiler den neuen Arm zur Compile-Zeit. Template: `PaidLimitExceeded`-Arm (Zeilen 254-258, 409) — nur Status 423.

```rust
// rest/src/lib.rs — error_handler, nach PaidLimitExceeded-Arm
Err(RestError::ServiceError(err @ ServiceError::WeekLocked { .. })) => {
    Response::builder()
        .status(423)
        .body(Body::new(err.to_string()))
        .unwrap()
}
```

Der `err.to_string()` liefert die i18n-neutrale `#[error(...)]`-Message im Response-Body. Für vollständige i18n-Lokalisierung (D-40-05): Die drei Texte sind primär für die Backend-Response gedacht (FE zeigt keinen Fehler-Banner, D-40-04).

**Erstes 423 im Codebase:** Alle bisherigen Konflikte (PaidLimitExceeded, OverlappingTimeRange, EntityConflicts, NotLatestBillingPeriod, EntityAlreadyExists) sind 409. Der 423-Fall ist semantisch präziser und vom Compiler erzwungen — kein Risiko vergessenen Mappings. [VERIFIED: codebase — rest/src/lib.rs:254-285]

**OpenAPI-Annotation:** Endpoints in `rest/src/shiftplan_edit.rs` und `rest/src/booking.rs` (delete_booking) müssen `responses(status = 423, description = "Week is locked")` in `#[utoipa::path]` erhalten (CLAUDE.md Gate).

---

### 5. DI — WeekStatusService in ShiftplanEditServiceDeps

#### Macro-Erweiterung

```rust
// service_impl/src/shiftplan_edit.rs:26-44 — gen_service_impl!-Block
gen_service_impl! {
    struct ShiftplanEditServiceImpl: ShiftplanEditService = ShiftplanEditServiceDeps {
        // ... bestehende Deps ...
        ToggleService: service::toggle::ToggleService<...> = toggle_service,
        // NEU für Phase 40 (D-40-01 Lock-Gate):
        WeekStatusService: service::week_status::WeekStatusService<Context = Self::Context, Transaction = Self::Transaction> = week_status_service,
    }
}
```

**Tier-Konformität:** `WeekStatusService` ist Basic-Tier (keine Domain-Service-Deps, nur DAO + Permission + Clock + Uuid + Transaction). `ShiftplanEditService` ist Business-Logic-Tier. Basic-Dep in Business-Logic-Service ist explizit zulässig (CLAUDE.md §Service-Tier-Konventionen). [VERIFIED: CLAUDE.md]

#### main.rs Wiring

`WeekStatusService` ist in `main.rs` bereits als Basic-Tier-Service konstruiert (Phase 39, Plan 03, Commit `2c6c140`). Für die `ShiftplanEditServiceDependencies`-Struct in `main.rs` muss das Feld `week_status_service: Arc<WeekStatusService>` ergänzt und beim Konstruktor der Wert `week_status_service.clone()` übergeben werden.

`RestStateDef` benötigt **keine** Änderung: `WeekStatusService` ist dort bereits als eigener AssocType definiert (rest/src/lib.rs:355-358). [VERIFIED: codebase]

---

### 6. Frontend — +/- Buttons ausblenden (D-40-03)

#### Verfügbare Signale in shiftplan.rs (verifiziert)

```rust
// src/page/shiftplan.rs:107 — bereits vorhanden (Phase 39)
let week_status = WEEK_STATUS_STORE.read().status.clone();
// Typ: crate::state::week_status::WeekStatus (Unset/InPlanning/Planned/Locked)

// src/page/shiftplan.rs:108-111 — bereits vorhanden
let is_shiftplanner = auth_info
    .as_ref()
    .map(|auth_info| auth_info.has_privilege("shiftplanner"))
    .unwrap_or(false);
```

[VERIFIED: codebase — shifty-dioxus/src/page/shiftplan.rs:107-111]

#### button_mode Erweiterung (Zeilen 260-266)

Aktuelle Evaluierungsreihenfolge (Zeilen 260-266):

```rust
let button_mode = if *change_structure_mode.read() {
    WeekViewButtonTypes::Dropdown
} else if js::current_datetime().date() - date > time::Duration::weeks(2) && !is_hr {
    WeekViewButtonTypes::None
} else {
    WeekViewButtonTypes::AddRemove
};
```

**Neue Reihenfolge (Priorität 2 eingefügt, aus UI-SPEC.md):**

```rust
use crate::state::week_status::WeekStatus;

let button_mode = if *change_structure_mode.read() {
    WeekViewButtonTypes::Dropdown                                          // Priorität 1
} else if *week_status == WeekStatus::Locked && !is_shiftplanner {
    WeekViewButtonTypes::None                                              // Priorität 2 (NEU)
} else if js::current_datetime().date() - date > time::Duration::weeks(2) && !is_hr {
    WeekViewButtonTypes::None                                              // Priorität 3
} else {
    WeekViewButtonTypes::AddRemove
};
```

`week_status` muss ggf. dereferenziert werden (Signal vs. Clone — bei `WEEK_STATUS_STORE.read().status.clone()` ist es kein Signal sondern ein geclonter Wert, kein `*` nötig). [VERIFIED: codebase — state/week_status.rs: `#[derive(Clone, PartialEq, Debug, Default)]`]

**DOM-Verhalten:** `WeekViewButtonTypes::None` → `show_add: false`, `show_remove: false` auf jedem `ColumnViewItem` → bestehende `if props.item_data.show_add { … }`-Zweige in `ColumnViewSlot` erzeugen keinen DOM-Knoten. Kein neues Tailwind-Class. [VERIFIED: UI-SPEC.md]

**DayAggregateView:** Erhält `button_types` aus demselben `button_mode` — keine separate Änderung nötig. [VERIFIED: UI-SPEC.md]

#### i18n (D-40-05)

Drei Dateien zu ändern:

| Datei | Änderung |
|-------|---------|
| `src/i18n/mod.rs` | `WeekLockedError` zu `pub enum Key` hinzufügen |
| `src/i18n/de.rs` | `Key::WeekLockedError => "Diese Woche ist gesperrt — Änderungen sind nicht möglich."` |
| `src/i18n/en.rs` | `Key::WeekLockedError => "This week is locked — changes are not possible."` |
| `src/i18n/cs.rs` | `Key::WeekLockedError => "Tento týden je uzamčen — změny nejsou možné."` |

Presence-Test in `src/i18n/mod.rs::tests` nach Muster bestehender Presence-Tests ergänzen. [VERIFIED: UI-SPEC.md Copywriting-Vertrag]

**Note:** Da kein FE-Banner gebaut wird (D-40-04), wird `Key::WeekLockedError` im Frontend vorerst nicht direkt gerendert. Der Schlüssel ist für den 423-Response-Body und potenzielle spätere Verwendung.

---

## Standard Stack

Phase 40 installiert **keine neuen Pakete**. Alle verwendeten Crates sind bereits im Workspace.

| Crate | Verwendung | Status |
|-------|-----------|--------|
| `service::week_status` | Lock-Status Read (Phase 39) | vorhanden |
| `service::shiftplan_edit` | Schreib-Gate (Phase 40) | erweitern |
| `mockall` | Mock für WeekStatusService in Tests | vorhanden |
| `async-trait` | Trait-Def | vorhanden |
| `utoipa` | OpenAPI 423-Annotation | vorhanden |

---

## Package Legitimacy Audit

**Kein neues Paket installiert.** Audit entfällt.

---

## Don't Hand-Roll

| Problem | Nicht bauen | Verwenden | Warum |
|---------|-------------|-----------|-------|
| Shiftplanner-Bypass | eigene Role-Check-Logik | `permission_service.check_permission(SHIFTPLANNER_PRIVILEGE, ...)` | Konsistent mit bestehenden Gates; transitiver Admin-Bypass kostenlos |
| Exhaustives Error-Mapping | eigene Dispatch-Map | `match`-Arm im bestehenden `error_handler` | Compiler erzwingt bei neuer `ServiceError`-Variante sofortigen Build-Fehler |
| Transaction-Propagation | neue Tx öffnen | `Some(tx.clone())` wie bestehende inner calls | Kein TOCTOU; Arc-geteilt; bewährt im Codebase (modify_slot:84, 121, ...) |

---

## Common Pitfalls

### Pitfall 1: Sechs Pfade — einen vergessen

**Was schiefläuft:** Nur 5 der 6 Methoden erhalten den Gate. `delete_booking` fehlt → WST-04-Bypass bleibt offen.

**Warum:** `delete_booking` ist eine NEU zu schaffende Methode; die anderen 5 existieren und werden nur erweitert. Bei der Planung werden die 5 leicht als "alle" gezählt.

**Erkennen:** Test-Matrix 6×{locked, open} — fehlt `delete_booking`-Test, schlägt die Matrix fehl.

### Pitfall 2: copy_week — Quell- statt Ziel-Woche prüfen

**Was schiefläuft:** `assert_week_not_locked(from_year, from_calendar_week, ...)` statt `(to_year, to_calendar_week, ...)`. Gesperrte Ziel-Woche wird nicht blockiert; gesperrte Quell-Woche fälschlicherweise abgelehnt (Lesen ist nie gelockt).

**Warum:** `from_*` steht in der Methodensignatur vor `to_*`; Verwechslung beim Copy-Paste.

**Erkennen:** Integration-Test "copy_week target locked → 423" schlägt fehl (keine 423).

### Pitfall 3: delete_booking — booking.year/week nach delete lesen

**Was schiefläuft:** `booking_service.delete(id, ...)` vor `booking_service.get(id, ...)` aufrufen — dann ist die Entity soft-deleted und `year`/`calendar_week` ggf. nicht mehr lesbar.

**Warum:** Reihenfolge `delete → get` ist "effizienter" aber falsch.

**Erkennen:** Lock-Gate greift nie, da Entity nicht mehr erreichbar; Test "non-shiftplanner deletes from locked week" gibt 200 statt 423.

### Pitfall 4: Fehlender OpenAPI 423-Response auf allen betroffenen Endpoints

**Was schiefläuft:** Utoipa-Annotationen fehlen oder listen nur 200/403. OpenAPI-Spec ist inkonsistent mit tatsächlichem Verhalten.

**Gate:** CLAUDE.md verlangt `#[utoipa::path]` auf allen Handlern. Clippy-CI schlägt bei `cargo clippy -- -D warnings` nicht fehl, aber das OpenAPI-Spec-Gate der Codebase-Policy ist verletzt.

### Pitfall 5: booking.calendar_week Cast i32 → u8

**Was schiefläuft:** `booking.calendar_week: i32` ohne Cast an `assert_week_not_locked(... calendar_week: u8, ...)` übergeben → Compiler-Fehler.

**Fix:** `booking.calendar_week as u8` — ISO-Wochen 1–53 passen immer in u8. [VERIFIED: codebase — service/src/booking.rs:17]

### Pitfall 6: i18n — Meldung in weniger als allen drei Locales

**Was schiefläuft:** `cs.rs` fehlt oder hat `??`-Fallback. Kein Compile-Error, nur schlechte UX.

**Fix:** `src/i18n/mod.rs::tests` Presence-Test für alle drei Locales; Compiler schlägt nur bei `match`-Fehler fehl, nicht bei fehlendem Key-Matching.

---

## Validation Architecture

`nyquist_validation` nicht explizit `false` in `.planning/config.json` → Validation Architecture wird eingeschlossen.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in + mockall (Backend); cargo test (FE) |
| Config file | `Cargo.toml` workspace (kein separates Config-File) |
| Quick run command | `cargo test -p service_impl shiftplan_edit -- --nocapture` |
| Full suite command | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |

### Phase-40-Test-Matrix (6 Pfade × 2 Wochen-Zustände)

Alle Tests: **Integration mit in-memory SQLite** im Stil von `service_impl/src/test/week_status.rs`. Mock-Deps via `mockall` für WeekStatusService, PermissionService etc.

| Test-ID | Methode | Woche | Rolle | Erwartetes Ergebnis |
|---------|---------|-------|-------|---------------------|
| T-40-01 | modify_slot | Locked | Nicht-Schichtplaner | `Err(WeekLocked { year, week })` |
| T-40-02 | modify_slot | Locked | Schichtplaner | `Ok(Slot)` |
| T-40-03 | modify_slot | Open | Nicht-Schichtplaner | — (Permission→Forbidden; nur Schichtplaner darf modify_slot) |
| T-40-04 | modify_slot_single_week | Locked | Schichtplaner | `Ok(Slot)` |
| T-40-05 | remove_slot | Locked | Schichtplaner | `Ok(())` |
| T-40-06 | remove_slot | Open | Schichtplaner | `Ok(())` |
| T-40-07 | book_slot | Locked | Nicht-Schichtplaner | `Err(WeekLocked { year, week })` |
| T-40-08 | book_slot | Locked | Schichtplaner | `Ok(BookingCreateResult)` |
| T-40-09 | book_slot | Open | Nicht-Schichtplaner | `Ok(BookingCreateResult)` |
| T-40-10 | copy_week | Locked Ziel | Schichtplaner | `Ok(CopyWeekResult)` (Bypass) |
| T-40-11 | copy_week | Locked Quelle, Open Ziel | Schichtplaner | `Ok(CopyWeekResult)` |
| T-40-12 | delete_booking | Locked | Nicht-Schichtplaner | `Err(WeekLocked { year, week })` |
| T-40-13 | delete_booking | Locked | Schichtplaner | `Ok(())` |
| T-40-14 | delete_booking | Open | Nicht-Schichtplaner | `Ok(())` |
| T-40-15 | delete_booking | non-existent id | beliebig | `Err(EntityNotFound)` (vor Lock-Gate) |

**T-40-03 Hinweis:** `modify_slot`, `remove_slot`, `modify_slot_single_week`, `copy_week` erfordern `shiftplan.edit` → Nicht-Schichtplaner kriegen `Forbidden` bereits vor Lock-Gate. Für Phase 40 relevant sind nur die Tests, wo der Gate auch tatsächlich greifen kann.

**In-Transaktion-Check (T-40-16):** Mock `WeekStatusService` gibt `Locked` zurück; mock `BookingService::create` darf **nicht** aufgerufen worden sein (Mockall expect-Zähler 0). Verifiziert, dass kein Schreibeffekt vor dem Lock-Gate eintritt.

**delete_booking-Reihenfolge (T-40-17):** Mock-Setup — `booking_service.get` gibt Booking mit `year=2026, calendar_week=27`; `week_status_service.get_week_status(2026, 27, ...)` gibt `Locked`; `booking_service.delete` darf NICHT aufgerufen worden sein. Bestätigt: kein Delete bei gesperrter Woche.

### Wave 0 Gaps (noch zu erstellen)

- [ ] `service_impl/src/test/shiftplan_edit_lock.rs` — neues Testmodul für die 6-Pfad-Matrix
- [ ] `service_impl/src/test/mod.rs` — `mod shiftplan_edit_lock;` eintragen

*(Bestehende `service_impl/src/test/week_status.rs` und `shiftplan_edit`-Tests bleiben unverändert)*

### Sampling Rate

- **Je Task-Commit:** `cargo test -p service_impl shiftplan_edit_lock && cargo clippy --workspace -- -D warnings`
- **Je Wave-Merge:** `cargo test --workspace`
- **Phase-Gate:** Full suite grün + `cargo sqlx prepare --workspace` (nur wenn neue queries — hier nicht erwartet)

---

## Architecture Patterns

### Empfohlene Änderungsstruktur (minimal, chirurgisch)

```
service/src/
├── lib.rs             # +1 WeekLocked { year, week } Variante
├── shiftplan_edit.rs  # +1 delete_booking Methode im Trait
└── week_status.rs     # unverändert

service_impl/src/
└── shiftplan_edit.rs  # +assert_week_not_locked helper; +5 Gate-Aufrufe; +delete_booking impl
                       # +WeekStatusService in gen_service_impl!-Block

rest/src/
├── lib.rs             # +1 WeekLocked-Arm im error_handler (423)
└── booking.rs         # delete_booking handler: booking_service → shiftplan_edit_service

shifty_bin/src/
└── main.rs            # +week_status_service Feld in ShiftplanEditServiceDependencies

shifty-dioxus/src/
├── page/shiftplan.rs  # +else-if Zweig in button_mode
└── i18n/
    ├── mod.rs         # +WeekLockedError key
    ├── de.rs          # +WeekLockedError translation
    ├── en.rs          # +WeekLockedError translation
    └── cs.rs          # +WeekLockedError translation
```

### Pattern: Gate nach Permission, in-tx

```rust
// Muster das ALLE sechs Methoden verwenden sollen:
let tx = self.transaction_dao.use_transaction(tx).await?;
// 1. Permission-Check (bestehendes Gate)
self.permission_service.check_permission("shiftplan.edit", context.clone()).await?;
// 2. Lock-Gate (NEU, Phase 40) — in derselben tx
self.assert_week_not_locked(year, week, context.clone(), tx.clone()).await?;
// 3. Business-Logik
```

### Pattern: Shiftplanner-Bypass für gemischte Permission-Methoden

```rust
// Für book_slot_with_conflict_check und delete_booking:
let is_shiftplanner = self.permission_service
    .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
    .await
    .is_ok();
// ... (für delete_booking: verify_user_is_sales_person oder booking_service.delete prüft)
if !is_shiftplanner {
    self.assert_week_not_locked(year, week, context.clone(), tx.clone()).await?;
}
```

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | nein | — |
| V3 Session Management | nein | — |
| V4 Access Control | **ja** | `check_permission(SHIFTPLANNER_PRIVILEGE, context)` — existierender Mechanismus |
| V5 Input Validation | nein | year/week kommen von bestehenden API-Parametern |
| V6 Cryptography | nein | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Bypass via direkten API-Call wenn FE-Buttons ausgeblendet | Elevation of Privilege | Server-Gate (assert_week_not_locked) — FE-Control ist UX-only (D-40-03) |
| TOCTOU: Status zwischen Lock-Check und Write geändert | Tampering | Check und Write in derselben Transaktion; SQLite serialisierte Writes |
| delete_booking Bypass (WST-04): direkter DELETE ohne ShiftplanEditService | Elevation of Privilege | Handler auf shiftplan_edit_service().delete_booking() umgeroutet |
| Vergessener 423-Arm → unhandled match → panic | Denial of Service | Compiler-erzwungen durch exhaustives match ohne `_` |

---

## Environment Availability

Phase 40 ist reine Code-Änderung, keine neuen externen Abhängigkeiten. Step 2.6 wird übersprungen.

---

## Open Questions

1. **WeekStatusService Transaction-Typ-Kompatibilität in gen_service_impl!**
   - Was wir wissen: Phase 39 DI-Wiring in `RestStateDef` verwendet `<Context = Context>` ohne `Transaction`-Bindung (Auto-Fix-Deviation in 39-03).
   - Was unklar: Ob `WeekStatusService<Context = Self::Context, Transaction = Self::Transaction>` in `ShiftplanEditServiceDeps` korrekt kompiliert (Typ-Ausrichtung).
   - Empfehlung: Beim Executor: zuerst `cargo build -p service_impl` nach Macro-Erweiterung ausführen; bei E0220/E0277 auf `<Context = Self::Context>` reduzieren und `tx`-Parameter entsprechend anpassen.

2. **week_status Signal vs. Clone in shiftplan.rs**
   - Was wir wissen: `let week_status = WEEK_STATUS_STORE.read().status.clone()` (Zeile 107) — kein Signal, sondern geclonter Wert.
   - Was unklar: Ob `button_mode`-Berechnung reaktiv auf Wochen-Wechsel neu ausgeführt wird (ja, da Komponente bei WEEK_STATUS_STORE-Änderung neu rendert).
   - Empfehlung: Kein Problem — Dioxus re-rendert bei Store-Änderung, `week_status` wird dann neu geclont.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A-40-01 | `Arc<Transaction>`-Clone in inner service calls ist No-Op-Commit (bestehender Pattern funktioniert) | Helper-Impl | TOCTOU-Schutz schlägt fehl; innere Commits rollen äußere Tx rollback |
| A-40-02 | `booking_service.delete` in Basic-Tier führt eigenen Permission-Check (Shiftplanner ∨ Self) aus — Semantik bleibt bei Delegation erhalten | delete_booking | Entweder doppelter Check (harmlos) oder kein Check (Security-Lücke) |
| A-40-03 | `has_privilege("shiftplanner")` == `has_privilege("shiftplan.edit")` aus FE-Sicht (für button_mode Bypass) | FE | HR-Rolle ohne shiftplanner-Tag sieht fälschlich Buttons (minimal) |

---

## Sources

### Primary (HIGH confidence — codebase-verifiziert)

- `service_impl/src/shiftplan_edit.rs` — Alle 5 Schreibmethoden mit exakten Zeilennummern für Permission-Checks und Einfügestellen
- `service/src/shiftplan_edit.rs` — Trait-Definitionen aller 5 Schreibmethoden
- `service/src/lib.rs` — ServiceError-Enum, bestehende Varianten
- `rest/src/lib.rs` — error_handler (exhaustiv), PaidLimitExceeded-Arm als Template, RestStateDef mit bestehenden ShiftplanEditService- und WeekStatusService-AssocTypes
- `rest/src/booking.rs` — delete_booking-Handler (aktuell), Route-Definition
- `service/src/booking.rs` — Booking-Struct mit calendar_week:i32, year:u32; BookingService::delete-Signatur
- `service/src/week_status.rs` — WeekStatusService-Trait, get_week_status-Signatur
- `service_impl/src/week_status.rs` — get_week_status-Impl (No-Gate, tx-propagation)
- `shifty-dioxus/src/page/shiftplan.rs` — week_status (Zeile 107), is_shiftplanner (108-111), button_mode (260-266)
- `shifty-dioxus/src/state/week_status.rs` — FE WeekStatus-Enum (Locked-Variante verifiziert)
- `.planning/phases/39-kw-status-grundlage/39-02-SUMMARY.md` — WeekStatusService-Impl-Details
- `.planning/phases/39-kw-status-grundlage/39-03-SUMMARY.md` — REST-Wiring, DI-Pattern

### Secondary (HIGH confidence — offizielle Entscheidungsquelle)

- `.planning/phases/40-wochen-sperre-durchsetzen/40-CONTEXT.md` — D-40-01..05, Einfügestellen-Refs
- `.planning/phases/40-wochen-sperre-durchsetzen/40-UI-SPEC.md` — button_mode Prioritätentabelle, DOM-Verhalten

---

## Metadata

**Confidence breakdown:**
- Einfügestellen (line-level): HIGH — direkt aus aktuellem Tree gelesen
- assert_week_not_locked Signatur: HIGH — abgeleitet aus verfügbaren Deps, TX-Pattern aus bestehendem Code
- DI-Wiring main.rs: MEDIUM — Schema aus Phase 39 bekannt, exakte Feldnamen in ShiftplanEditServiceDependencies nicht separat verifiziert
- FE button_mode Änderung: HIGH — Signal und Enum direkt verifiziert

**Research date:** 2026-07-02
**Valid until:** 2026-08-02 (stabiles Codebase; keine Moving-Target-Deps)
