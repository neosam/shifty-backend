# Phase 6: rest-types Unification & Frontend Compile-Through — Pattern Map

**Mapped:** 2026-05-07
**Files analyzed:** 12 neue/modifizierte Dateien (5 Backend, 7 Frontend)
**Analogs found:** 12 / 12

---

## File Classification

| Neue/Modifizierte Datei | Rolle | Data Flow | Closest Analog | Match Quality |
|-------------------------|-------|-----------|----------------|---------------|
| `rest-types/src/lib.rs` (Backend) | model/DTO | request-response | `rest-types/src/lib.rs` selbst (neue TO-Definitionen nach bestehendem Muster) | exact |
| `shifty-dioxus/Cargo.toml` | config | — | `shifty-dioxus/Cargo.toml` (Zeile 28-29 swap) | exact |
| `shifty-dioxus/rest-types/` (löschen) | — | — | — | n/a |
| `shifty-dioxus/src/state/shiftplan.rs` | state/model | transform | `shifty-dioxus/src/state/employee.rs` (From<&*TO>-Impls) | exact |
| `shifty-dioxus/src/state/employee.rs` | state/model | transform | `shifty-dioxus/src/state/employee.rs` selbst (panic-Branch) | exact |
| `shifty-dioxus/src/state/user_management.rs` | state/model | transform | `shifty-dioxus/src/state/shiftplan.rs` (SalesPerson-From-Impl) | exact |
| `shifty-dioxus/src/loader.rs` | utility | transform/CRUD | `shifty-dioxus/src/loader.rs` selbst (load_shift_plan) | exact |
| `shifty-dioxus/src/api.rs` | utility | request-response | `shifty-dioxus/src/api.rs` selbst (add_booking, copy_week) | exact |
| `shifty-dioxus/src/component/week_view.rs` | component | event-driven | `shifty-dioxus/src/component/dialog.rs` (rsx!{} Pattern) | role-match |
| `shifty-dioxus/src/page/shiftplan.rs` | component | event-driven | `shifty-dioxus/src/component/dialog.rs` (rsx!{} Pattern) | role-match |
| `shifty-dioxus/src/component/extra_hours_modal.rs` | component | event-driven | `shifty-dioxus/src/component/dialog.rs` (rsx!{} Pattern) | role-match |
| `shifty-dioxus/src/component/booking_log_table.rs` | component | event-driven | `shifty-dioxus/src/component/dialog.rs` (rsx!{} Pattern) | role-match |

---

## Pattern Assignments

---

### `rest-types/src/lib.rs` (Backend, Wave 0 — Blocker-Patch)

**Analog:** Bestehende TO-Definitionen in `rest-types/src/lib.rs` (Zeilen 1498–1560)

#### Blocker 1: `shifty_utils`-Import feature-gaten

**Ist-Zustand** (`rest-types/src/lib.rs`, Zeile 8):
```rust
use shifty_utils::{derive_from_reference, LazyLoad};
```

**Soll-Zustand** (nach dem Analog des Frontend-Forks `shifty-dioxus/rest-types/src/lib.rs`, Zeile 8-9):
```rust
#[cfg(feature = "service-impl")]
use shifty_utils::{derive_from_reference, LazyLoad};
```

**Warum:** Die Frontend-Fork hat dies bereits korrekt gated. Das Backend hat es ungegated. Nach Feature-Gating bleibt `cargo check --workspace` grün; `shifty_utils` ist trotzdem im Dep-Graph (Cargo.toml), aber der `use`-Import ist dann kompilationsbedingt auf `service-impl`-Builds beschränkt.

---

#### Blocker 2: `ShiftplanTO` braucht `PartialEq, Eq`

**Backend-Ist** (`rest-types/src/lib.rs`, Zeilen 13):
```rust
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct ShiftplanTO {
```

**Frontend-Fork-Ist** (`shifty-dioxus/rest-types/src/lib.rs`, Zeile 14) — das ist der Analog:
```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ShiftplanTO {
```

**Soll-Zustand** im Backend:
```rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, ToSchema)]
pub struct ShiftplanTO {
```

**Warum:** Dioxus-`GlobalSignal` erfordert `PartialEq` für Reaktivität. Das Frontend nutzt `ShiftplanTO` in `Rc<[ShiftplanTO]>` und in Vergleichsoperationen. Ohne `PartialEq` Compile-Fehler nach dem Swap.

**Derive-Diff-Strategie für andere TOs:** Analog zur `ShiftplanTO`-Diskrepanz muss beim Plan-Ausführer ein vollständiger derive-Diff zwischen beiden `lib.rs`-Dateien gemacht werden. Konventions-Regel aus der Frontend-Fork: Enums und Structs die in `GlobalSignal`-Containern oder in `==`-Vergleichen landen, brauchen `PartialEq, Eq`.

---

#### Blocker 3: `InvitationStatus`, `InvitationResponse`, `GenerateInvitationRequest` in `rest-types` einführen

**Analog-Quelle:** `rest/src/user_invitation.rs`, Zeilen 23–71 (vollständige Definitionen)

**Muster für `InvitationStatus`** (aus `rest/src/user_invitation.rs`, Zeilen 23–35):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InvitationStatus {
    Valid,
    Expired,
    Redeemed,
    #[serde(rename = "sessionrevoked")]
    SessionRevoked,
}
```

**Muster für `GenerateInvitationRequest`** (aus `rest/src/user_invitation.rs`, Zeilen 48–54):
```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateInvitationRequest {
    pub username: String,
    pub expiration_hours: Option<i64>,
}
```

**Muster für `InvitationResponse`** (aus `rest/src/user_invitation.rs`, Zeilen 56–71):
```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub username: String,
    pub token: Uuid,
    pub invitation_link: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub redeemed_at: Option<time::OffsetDateTime>,
    pub status: InvitationStatus,
}
```

**Wichtig:** `redeemed_at` nutzt `time::serde::rfc3339::option`. Das Backend-`rest-types/Cargo.toml` hat `time = { ..., features = ["serde-human-readable"] }`. `rfc3339`-Serde benötigt jedoch `time = { ..., features = ["serde-human-readable", "formatting", "parsing"] }`. Entweder `time`-Features in `rest-types/Cargo.toml` erweitern, ODER `redeemed_at` als `Option<String>` deklarieren (einfacher, WASM-kompatibel). Beide Varianten sind wire-kompatibel wenn das Backend RFC3339-Strings serialisiert.

**Nach der Migration:** In `rest/src/user_invitation.rs` die lokalen Definitionen durch Re-Exports ersetzen:
```rust
// In rest/src/user_invitation.rs — bestehende Definitionen durch ersetzen:
pub use rest_types::{InvitationStatus, InvitationResponse, GenerateInvitationRequest};
```

---

### `shifty-dioxus/Cargo.toml` (Wave 1 — Cargo-Swap)

**Analog:** `shifty-dioxus/Cargo.toml` Zeilen 28-29 (aktueller Zustand)

**Ist-Zustand** (Zeilen 28-29):
```toml
[dependencies.rest-types]
path = "rest-types"
```

**Soll-Zustand:**
```toml
[dependencies.rest-types]
path = "../rest-types"
default-features = false
```

**Kontext:** `shifty-backend/Cargo.toml` enthält `exclude = ["shifty-dioxus"]` — der Path-Dep cross-workspace ist ein Standard-Cargo-Pattern. `default-features = false` ist ein No-Op (Backend-`rest-types` hat `default = []`), aber ein explizites Zukunftssicherheits-Signal.

---

### `shifty-dioxus/src/state/shiftplan.rs` (Wave 2, Cluster C + E)

**Analog:** `shifty-dioxus/src/state/shiftplan.rs` — `Slot`-Struct und `From<&SlotTO>`-Impl (Zeilen 164–198)

#### Ist-Zustand `Slot`-Struct (Zeilen 164–172):
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: Uuid,
    pub day_of_week: Weekday,
    pub from: time::Time,
    pub to: time::Time,
    pub bookings: Rc<[Booking]>,
    pub min_resources: u8,
}
```

#### Soll-Zustand (neue Felder nach UI-SPEC Regel 2 — state-mirror, kein Render):
```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: Uuid,
    pub day_of_week: Weekday,
    pub from: time::Time,
    pub to: time::Time,
    pub bookings: Rc<[Booking]>,
    pub min_resources: u8,
    pub max_paid_employees: Option<u8>,  // NEU — v1.3 FUI-02 rendert dies
    pub current_paid_count: u8,          // NEU — v1.3 FUI-01 rendert dies
}
```

#### `From<&SlotTO>`-Impl Ist-Zustand (Zeilen 187–198):
```rust
impl From<&SlotTO> for Slot {
    fn from(slot: &SlotTO) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
            bookings: [].into(),
            min_resources: slot.min_resources,
        }
    }
}
```

#### Soll-Zustand `From<&SlotTO>` (neue Felder mappen):
```rust
impl From<&SlotTO> for Slot {
    fn from(slot: &SlotTO) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
            bookings: [].into(),
            min_resources: slot.min_resources,
            max_paid_employees: slot.max_paid_employees,  // NEU
            current_paid_count: 0,  // NEU — default; wird in loader.rs gesetzt
        }
    }
}
```

#### Weekday `from_num_from_monday` — Panic-Branch (Zeile 59):

**Ist-Zustand (Zeile 59):**
```rust
_ => panic!("Invalid weekday number: {}", num),
```

**Soll-Zustand (UI-SPEC Regel 3 — defensiver Fallback):**
```rust
_ => Weekday::Monday,  // Defensive fallback — v1.x; invalid input → Monday
```

**Alternativ** (semantisch reicher, aber erfordert neuen Variant):
```rust
// Wenn Weekday::Unknown(u8) hinzugefügt wird:
_ => Weekday::Monday,  // bevorzugt — kein neuer Variant nötig für Phase 6
```

---

### `shifty-dioxus/src/state/employee.rs` (Wave 2, Cluster — Panic-Branch)

**Analog:** `shifty-dioxus/src/state/employee.rs` selbst — `from_identifier`-Funktion (Zeilen 78–91)

#### `from_identifier` Panic-Branch — Ist-Zustand (Zeile 89):
```rust
pub fn from_identifier(identifier: &str) -> Self {
    match identifier {
        "shiftplan" => WorkingHoursCategory::Shiftplan,
        // ... alle bekannten Varianten ...
        _ => panic!("Unknown working hours category: {}", identifier),
    }
}
```

**Soll-Zustand (UI-SPEC Regel 3):** Erfordert neuen `Unknown(Rc<str>)`-Variant auf dem Enum, aber UI-SPEC schreibt nur vor, wenn ein Plan-Wave diese Stelle berührt. Falls kein Wave sie berührt, bleibt sie unverändert (Phase 6 berührt `from_identifier` nur indirekt). Falls doch berührt:
```rust
// Neuer Variant am Enum:
pub enum WorkingHoursCategory {
    // ... bestehende ...
    Unknown(Rc<str>),
}

// In from_identifier:
_ => WorkingHoursCategory::Unknown(identifier.into()),
```

**`From<&WorkingHoursCategory> for ExtraHoursCategoryTO` Panic-Branch — Ist-Zustand (Zeilen 140–157):**
```rust
impl From<&WorkingHoursCategory> for ExtraHoursCategoryTO {
    fn from(category: &WorkingHoursCategory) -> Self {
        match category {
            WorkingHoursCategory::ExtraWork(_) => ExtraHoursCategoryTO::ExtraWork,
            // ... weitere Varianten ...
            _ => panic!(
                "Cannot convert working hours category to extra hours category: {:?}",
                category
            ),
        }
    }
}
```

**Bewertung:** Diese Panic ist ein Program-Invariant-Check (nicht User-facing) — sie schlägt nur an, wenn der Aufrufer einen semantisch invaliden State übergibt (z.B. `Shiftplan` → `ExtraHoursCategoryTO`). Laut RESEARCH.md kann diese Panic bestehen bleiben, falls kein Plan-Wave sie direkt berührt. Falls ein `Unknown`-Variant eingeführt wird, muss ein Arm ergänzt werden:
```rust
WorkingHoursCategory::Unknown(_) => ExtraHoursCategoryTO::ExtraWork,  // fallback
```

**Wichtiger Befund:** Die `From<&ExtraHoursCategoryTO>` und `From<&ExtraHoursReportCategoryTO>` Impls (Zeilen 125–170) sind bereits vollständig exhaustive — alle Varianten inkl. `UnpaidLeave` und `VolunteerWork` sind gemappt. Kein Handlungsbedarf hier.

---

### `shifty-dioxus/src/loader.rs` (Wave 2, Cluster C + E)

**Analog:** `shifty-dioxus/src/loader.rs` — `load_shift_plan`-Funktion (Zeilen 151–192)

#### Ist-Zustand `load_shift_plan` Slot-Mapping (Zeilen 162–189):
```rust
let slots = shiftplan_week
    .days
    .iter()
    .flat_map(|day| day.slots.iter())
    .map(|slot| Slot {
        id: slot.slot.id,
        day_of_week: slot.slot.day_of_week.into(),
        from: slot.slot.from,
        to: slot.slot.to,
        min_resources: slot.slot.min_resources,
        bookings: slot.bookings.iter().map(|booking| Booking {
            // ... bestehende Felder ...
        }).collect(),
    })
    .collect();
```

#### Soll-Zustand (neue Felder durchreichen, UI-SPEC Regel 2):
```rust
.map(|slot| Slot {
    id: slot.slot.id,
    day_of_week: slot.slot.day_of_week.into(),
    from: slot.slot.from,
    to: slot.slot.to,
    min_resources: slot.slot.min_resources,
    max_paid_employees: slot.slot.max_paid_employees,  // NEU — aus SlotTO
    current_paid_count: slot.current_paid_count,        // NEU — aus ShiftplanSlotTO
    bookings: slot.bookings.iter().map(|booking| Booking {
        // ... unverändert ...
    }).collect(),
})
```

**Hinweis:** `slot` ist ein `&ShiftplanSlotTO` — `slot.slot` ist ein `SlotTO`, `slot.current_paid_count` ist direkt auf dem `ShiftplanSlotTO` (Backend `rest-types/src/lib.rs`, Zeilen 977–987 bestätigt).

#### `load_day_aggregate` braucht identisches Muster (Zeilen 194–240+)

Das gleiche Slot-Konstruktions-Muster erscheint auch in `load_day_aggregate`. Beide Stellen müssen konsistent die neuen Felder durchreichen.

#### `ShiftplanDayTO.unavailable` durchreichen (Cluster C)

Der `load_shift_plan`-Loop iteriert `day.slots` aber ignoriert `day.unavailable`. Nach dem Swap ist `day.unavailable: Option<UnavailabilityMarkerTO>` vorhanden. Falls ein `ShiftplanDay`-State-Typ existiert (oder eingeführt wird), muss das Feld gespiegelt werden.

**Aktueller Befund:** `load_shift_plan` baut kein `ShiftplanDay`-State-Objekt — es flacht `days.slots` direkt in `Rc<[Slot]>` ab. Das `unavailable`-Feld von `ShiftplanDayTO` landet nirgendwo. UI-SPEC Regel 2 schreibt vor: state-Mirror mit Default. Da kein `ShiftplanDay`-State-Typ existiert, ist der einfachste Weg, `unavailable` in der `load_shift_plan`-Funktion zu ignorieren (Feld existiert nach dem Swap, wird aber nicht gelesen — kein Compile-Fehler).

---

### `shifty-dioxus/src/api.rs` (Wave 2, Cluster D + F)

**Analog:** `shifty-dioxus/src/api.rs` — `add_booking` (Zeilen 195–223), `copy_week` (Zeilen 235–249), `generate_invitation` (Zeilen 1113–1125)

#### `add_booking` Rückgabetyp — Ist-Zustand (Zeilen 195–223):
```rust
pub async fn add_booking(
    config: Config,
    sales_person_id: Uuid,
    slot_id: Uuid,
    week: u8,
    year: u32,
) -> Result<(), reqwest::Error> {
    // ...
    let response = client.post(url).json(&booking_to).send().await?;
    response.error_for_status_ref()?;
    info!("Added");
    Ok(())
}
```

**Soll-Zustand (Cluster D — `BookingCreateResultTO` deserialisieren, Warnings ignorieren):**

Laut RESEARCH.md ist das Verhalten für v1.2 akzeptabel (`()` zurückgeben, Backend-Warnings ignorieren). Das Muster ist: Backend antwortet mit `BookingCreateResultTO`, aber `response.error_for_status_ref()?` prüft den Status-Code. Die Body-Deserialisierung kann übersprungen werden.

```rust
pub async fn add_booking(
    config: Config,
    sales_person_id: Uuid,
    slot_id: Uuid,
    week: u8,
    year: u32,
) -> Result<(), reqwest::Error> {
    // ... unverändert bis response ...
    let response = client.post(url).json(&booking_to).send().await?;
    response.error_for_status_ref()?;
    // v1.2: BookingCreateResultTO-Warnings werden ignoriert (no-op).
    // v1.3: Rückgabetyp auf BookingCreateResultTO ändern und warnings propagieren.
    info!("Added");
    Ok(())
}
```

**Kein Compile-Fehler** — `rest_types::BookingCreateResultTO` muss nur importierbar sein, nicht aktiv genutzt. Der Typ ist nach dem Swap in `rest_types` vorhanden.

#### `copy_week` — analog zu `add_booking`:
```rust
pub async fn copy_week( ... ) -> Result<(), reqwest::Error> {
    // ... unverändert — CopyWeekResultTO-Warnings werden in v1.2 ignoriert
}
```

#### `get_shiftplan_assignments` / `set_shiftplan_assignments` — Import-Pfad-Änderung (Zeilen 1203–1240):

**Ist-Zustand:** Importiert `state::ShiftplanAssignment` (lokale struct in `state/user_management.rs`).

**Soll-Zustand (Cluster F):** `ShiftplanAssignmentTO` ist nach dem Swap in `rest_types` verfügbar und wire-kompatibel mit der lokalen `ShiftplanAssignment`-struct (gleiche Felder, gleiche Serde-Attribute). Optionen:
1. Lokale `ShiftplanAssignment` behalten — sie ist wire-kompatibel, kein Compile-Fehler. Import-Pfad bleibt.
2. Lokale `ShiftplanAssignment` durch `rest_types::ShiftplanAssignmentTO` ersetzen — sauberer, aber mehr Änderung.

**Empfehlung:** Lokale struct in `state/user_management.rs` behalten (sie hat identische Serde-Shape). Kein Refactoring nötig für Phase 6.

#### `generate_invitation` / `list_user_invitations` — nach Swap:

```rust
// In src/api.rs — imports nach dem Swap:
use rest_types::{
    // ... bestehende ...
    GenerateInvitationRequest, InvitationResponse,
    // ... (diese waren vorher aus dem Fork)
};
```

Die Funktionen selbst bleiben unverändert — nur der Import-Pfad wechselt von `rest_types` (Fork) zu `rest_types` (Backend). Da beide denselben Crate-Namen haben, ändert sich im Code nichts.

---

### `shifty-dioxus/src/state/user_management.rs` (Wave 2, Cluster F)

**Analog:** `shifty-dioxus/src/state/user_management.rs` selbst (Zeilen 1–29)

**Ist-Zustand** (vollständige Datei, Zeilen 7–12):
```rust
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShiftplanAssignment {
    pub shiftplan_id: Uuid,
    #[serde(default = "default_permission_level")]
    pub permission_level: String,
}
```

**Backend-`ShiftplanAssignmentTO`** (`rest-types/src/lib.rs`, Zeilen 1487–1496):
```rust
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct ShiftplanAssignmentTO {
    pub shiftplan_id: Uuid,
    #[serde(default = "default_permission_level")]
    pub permission_level: String,
}
```

**Bewertung:** Die lokale `ShiftplanAssignment`-struct und `ShiftplanAssignmentTO` sind wire-kompatibel (identische Serde-Shape, identische `default_permission_level`-Funktion). Für Phase 6: lokale struct behalten. `ToSchema`-Derive fehlt auf der lokalen struct — aber das ist Backend-Only relevant, kein WASM-Build-Problem.

**Wenn doch ersetzt wird** (sauberer aber optional):
```rust
// state/user_management.rs — Zeile 1:
use rest_types::ShiftplanAssignmentTO as ShiftplanAssignment;
// ODER: Typ-Alias anlegen
pub type ShiftplanAssignment = rest_types::ShiftplanAssignmentTO;
```

---

### `shifty-dioxus/src/component/dialog.rs` (Regel-1-Analog-Quelle)

**Analog-Quelle für leeres RSX-Pattern** (`shifty-dioxus/src/component/dialog.rs`, ca. Zeile 139):

```rust
// Bestehendes Empty-Render-Pattern im Codebase (Regel 1 Anker):
if !props.open {
    return rsx! {};
}
```

**Dieses Pattern ist der Anker für Regel 1 (UI-SPEC).** Alle neuen Match-Arme für neue Enum-Varianten in `rsx!`-Kontext nutzen genau dieses Muster:

```rust
// Pattern für neue WarningTO-Varianten in jedem Warning-Renderer:
match warning {
    WarningTO::BookingOnAbsenceDay { .. } => rsx! { /* bestehender Code */ },
    WarningTO::BookingOnUnavailableDay { .. } => rsx! { /* bestehender Code */ },
    WarningTO::AbsenceOverlapsBooking { .. } => rsx! { /* bestehender Code */ },
    WarningTO::AbsenceOverlapsManualUnavailable { .. } => rsx! { /* bestehender Code */ },
    WarningTO::PaidEmployeeLimitExceeded { .. } => rsx! {},  // v1.3 FUI-01
}
```

---

## Shared Patterns

### Shared Pattern 1: GET-Endpoint-Wrapper (request-response)

**Quelle:** `shifty-dioxus/src/api.rs`, Zeilen 55–68 (`get_slots`)

```rust
pub async fn get_slots(
    config: Config,
    year: u32,
    week: u8,
    shiftplan_id: Uuid,
) -> Result<Rc<[SlotTO]>, reqwest::Error> {
    info!("Fetching ...");
    let url = format!("{}/slot/week/{year}/{week}/{shiftplan_id}", config.backend);
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    info!("Fetched");
    Ok(res)
}
```

Jede neue GET-Bindung in `api.rs` kopiert dieses Muster: `reqwest::get(url) → error_for_status_ref()? → json()`.

### Shared Pattern 2: POST-Endpoint-Wrapper (request-response mit Body)

**Quelle:** `shifty-dioxus/src/api.rs`, Zeilen 195–223 (`add_booking`)

```rust
let client = reqwest::Client::new();
let response = client.post(url).json(&payload).send().await?;
response.error_for_status_ref()?;
info!("Done");
Ok(())
```

### Shared Pattern 3: `From<&*TO> for state::*` (transform)

**Quelle:** `shifty-dioxus/src/state/shiftplan.rs`, Zeilen 103–118 (`From<&BookingTO>`) und Zeilen 129–153 (`From<&SalesPersonTO>`)

```rust
impl From<&BookingTO> for Booking {
    fn from(booking: &BookingTO) -> Self {
        Self {
            id: booking.id,
            // ... Felder 1:1 oder mit Konvertierung mappen
        }
    }
}
```

Alle neuen state-Mirror-Felder werden in der zugehörigen `From`-Impl gemappt. Default-Werte: `Option<u8>` → `None`, `u8` → `0`.

### Shared Pattern 4: TO-Definitionen in `rest-types/src/lib.rs` (Backend)

**Quelle:** `rest-types/src/lib.rs`, Zeilen 1498–1560 (ToggleTO, ImpersonateTO)

```rust
// Standard-TO-Derive-Set für WASM-kompatible Types:
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct FooTO {
    pub field: Type,
    #[serde(default)]
    pub optional_field: Option<Type>,
}

// Enum-TOs mit tag:
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum BarTO {
    VariantA { field: Type },
    VariantB,
}
```

`service-impl`-abhängige `From`-Impls werden mit `#[cfg(feature = "service-impl")]` umschlossen.

### Shared Pattern 5: Backend-TO-Namespace nach dem Swap

**Quelle:** `shifty-dioxus/src/api.rs`, Zeilen 3–11 (Import-Block)

```rust
use rest_types::{
    BillingPeriodTO, BlockTO, BookingConflictTO, BookingLogTO, BookingTO,
    // ... alle genutzten TOs
    GenerateInvitationRequest, InvitationResponse,  // nach Swap: aus Backend-rest-types
};
```

Nach dem Swap bleibt der `use rest_types::*`-Namespace identisch — nur der physische Pfad der Crate ändert sich von `shifty-dioxus/rest-types/` zu `shifty-backend/rest-types/`.

---

## No Analog Found

Alle Dateien haben konkrete Analogs. Es gibt keine Datei ohne Vorlage.

| Datei | Bemerkung |
|-------|-----------|
| `shifty-dioxus/rest-types/` (Löschung) | Keine Anpassung nötig — `rm -rf`-Operation |

---

## Kritische Reihenfolge-Constraints

Die Patterns haben eine strikte Wave-Abhängigkeit:

```
Wave 0 (Backend):
  1. rest-types/src/lib.rs: shifty_utils-Import feature-gaten
  2. rest-types/src/lib.rs: ShiftplanTO + andere TOs: PartialEq/Eq-Derives hinzufügen
  3. rest-types/src/lib.rs: InvitationStatus/InvitationResponse/GenerateInvitationRequest einfügen
  4. rest/src/user_invitation.rs: lokale Definitionen durch Re-Exports ersetzen
  → Verifikation: cargo check --workspace (Backend)

Wave 1 (Cargo-Swap):
  5. shifty-dioxus/Cargo.toml: path = "../rest-types", default-features = false
  6. shifty-dioxus/rest-types/ löschen
  → Verifikation: cargo check (in shifty-dioxus/, non-WASM)

Wave 2 (Frontend Compile-Errors beheben, parallel innerhalb der Wave):
  7a. state/shiftplan.rs: Slot-Felder + From-Impl + Weekday-Panic-Branch
  7b. state/employee.rs: Panic-Branches (nur wenn berührt)
  7c. state/user_management.rs: ShiftplanAssignment (behalten oder ersetzen)
  7d. loader.rs: Slot-Felder durchreichen in load_shift_plan + load_day_aggregate
  7e. api.rs: Import-Pfade verifizieren (sollten automatisch stimmen)
  7f. Render-Sites (week_view, shiftplan, extra_hours_modal, booking_log_table):
       neue Match-Arme via rsx!{} (nur wenn Compiler es verlangt)
  → Verifikation: nix develop --command cargo build --target wasm32-unknown-unknown
```

---

## Metadata

**Analog-Suchbereich:** `shifty-dioxus/src/`, `shifty-backend/rest-types/src/`, `shifty-backend/rest/src/`
**Dateien gescannt:** 15 Dateien direkt gelesen + Grep-Analyse
**Pattern-Extraktion:** 2026-05-07
