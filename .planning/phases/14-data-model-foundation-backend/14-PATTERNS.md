# Phase 14: Data-model foundation (backend) - Pattern Map

**Mapped:** 2026-06-23
**Files analyzed:** 8 touch sites across 7 files
**Analogs found:** 8 / 8 (jeder Touch-Site hat einen exakten line-für-line-Präzedenzfall)

> **Leitprinzip dieser Phase:** `committed_voluntary: f32` wird an JEDER Stelle dort eingefügt, wo heute `cap_planned_hours_to_expected` steht — SAME position, SAME order. Einzige Divergenz: `cap_planned…` ist ein Bool (`i64` in der DB, `!= 0`-Coercion), `committed_voluntary` ist ein numerisches `REAL`/`f32` ohne Bool-Coercion. Sekundärer Präzedenzfall: `is_dynamic` (ebenfalls Bool — beim Numerik-Aspekt nicht verlassen).

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog (im selben File) | Match Quality |
|---------------|------|-----------|----------------------------------|---------------|
| `migrations/sqlite/<ts>_add-committed-voluntary-to-employee-work-details.sql` (NEU) | migration | DDL | `migrations/sqlite/20260426120000_add-cap-flag-to-employee-work-details.sql` | exact (Typ-Divergenz INTEGER→REAL) |
| `dao_impl_sqlite/src/employee_work_details.rs` | model (DAO row + queries) | CRUD | `cap_planned_hours_to_expected` (12× im File) | exact |
| `dao/src/employee_work_details.rs` | model (DAO trait entity) | CRUD | `cap_planned_hours_to_expected: bool` (Z.22) | exact (Typ bool→f32) |
| `service/src/employee_work_details.rs` | model (Service struct + 2 Konversionen) | transform | `cap_planned_hours_to_expected` (Z.26/56/207) | exact (Typ bool→f32) |
| `service_impl/src/employee_work_details.rs` | service (Update/Rotate carry-forward) | CRUD | `entity.cap_planned_hours_to_expected = …` (Z.248) | exact |
| `rest-types/src/lib.rs` | model (TO + 2 Konversionen) | request-response | `cap_planned_hours_to_expected` (Z.611/650/688) | exact (Typ bool→f32) |
| `service_impl/src/reporting.rs` | service (SUM-Aggregations-Semantik, nur Test) | transform | `expected_hours`-`.fold` (Z.240-254) | role+flow-match (Bool-`.any()` Z.264 NICHT) |
| `service_impl/src/test/employee_work_details.rs` | test | — | `update_propagates_cap_planned_hours_flag_to_dao` (Z.86-143) | exact |

---

## Pattern Assignments

### 1. Migration (NEU) — `migrations/sqlite/<ts>_add-committed-voluntary-to-employee-work-details.sql`

**Analog:** `migrations/sqlite/20260426120000_add-cap-flag-to-employee-work-details.sql`

Vollständiger Analog-Inhalt:
```sql
ALTER TABLE employee_work_details
ADD COLUMN cap_planned_hours_to_expected INTEGER NOT NULL DEFAULT 0;
```

**Zu erzeugender Inhalt (Typ-Divergenz: REAL statt INTEGER, Default-Literal `0`):**
```sql
ALTER TABLE employee_work_details
ADD COLUMN committed_voluntary REAL NOT NULL DEFAULT 0;
```

**Insertion point:** Neue Datei via `sqlx migrate add committed-voluntary-to-employee-work-details` (erzeugt frischen Timestamp-Prefix). Additiv anwenden mit `sqlx migrate run` — **NIEMALS** `sqlx database reset` (destruktiv, braucht User-Confirmation). Auf NixOS: `nix develop --command`. Danach `cargo sqlx prepare` für den `.sqlx`-Offline-Cache (erster harter Compile-Gate, da `query_as!`/`query!` compile-time-checked).

---

### 2. DAO Row + Queries — `dao_impl_sqlite/src/employee_work_details.rs`

**Analog:** `cap_planned_hours_to_expected` (12 Vorkommen). Das neue Feld MUSS in der SAME Position erscheinen.

**KRITISCHE DIVERGENZ:** `cap_planned_hours_to_expected` ist `pub cap_planned_hours_to_expected: i64` (Bool als DB-Integer). `committed_voluntary` ist `REAL`/`f32` — analog zum bestehenden `pub expected_hours: f64` (Z.17). Verwende **`f64` im `*Db`-Struct** und `as f32`-Cast in `TryFrom` (wie `expected_hours`), NICHT `i64` + `!= 0`.

**(a) Row-Struct-Feld (`EmployeeWorkDetailsDb`, nach Z.26):**
```rust
// Z.25-26 (Analog cap):
    pub is_dynamic: i64,
    pub cap_planned_hours_to_expected: i64,
    // EINFÜGEN (Typ wie expected_hours Z.17 `f64`):
    pub committed_voluntary: f64,
```

**(b) `TryFrom<&EmployeeWorkDetailsDb>` (Z.62-63):** Bool-Analog nutzt `!= 0`. NICHT kopieren — `committed_voluntary` folgt dem `expected_hours`-Muster (Z.50 `working_hours.expected_hours as f32`):
```rust
// Z.62-63 (Analog cap, MIT !=0 — NICHT übernehmen für numeric):
            is_dynamic: working_hours.is_dynamic != 0,
            cap_planned_hours_to_expected: working_hours.cap_planned_hours_to_expected != 0,
            // EINFÜGEN (Typ-Cast wie expected_hours Z.50):
            committed_voluntary: working_hours.committed_voluntary as f32,
```

**(c) 4× SELECT-Spaltenliste** — `committed_voluntary` direkt NACH `cap_planned_hours_to_expected` in jeder Liste:
- `all()` — Z.119
- `find_by_id()` — Z.169
- `find_by_sales_person_id()` — Z.221
- `find_for_week()` — Z.275

```sql
                is_dynamic,
                cap_planned_hours_to_expected,
                committed_voluntary,   -- EINFÜGEN in allen 4 SELECTs
```

**(d) INSERT** — drei synchron zu pflegende Stellen:
- Lokale Variable (nach Z.327): `let committed_voluntary = entity.committed_voluntary as f64;` (Analog Z.319 `let expected_hours = entity.expected_hours as f64;`, NICHT die `as i64`-Bool-Zeile Z.327)
- Spaltenliste (nach Z.353 `cap_planned_hours_to_expected,`): `committed_voluntary,`
- VALUES-Tupel (Z.368): EIN weiteres `?` ergänzen (von 23 auf 24 Platzhalter)
- Binding-Liste (nach Z.381 `cap_planned_hours_to_expected,`): `committed_voluntary,`

**(e) UPDATE (Z.421-447):**
- Lokale Variable (nach Z.420): `let committed_voluntary = entity.committed_voluntary as f64;`
- SET-Klausel (nach Z.433 `cap_planned_hours_to_expected = ?`): Komma + `committed_voluntary = ?`
- Binding-Liste (nach Z.446 `cap_planned_hours_to_expected,`): `committed_voluntary,` (VOR `id`)

---

### 3. DAO Trait Entity — `dao/src/employee_work_details.rs`

**Analog:** `pub cap_planned_hours_to_expected: bool,` (Z.22 im `EmployeeWorkDetailsEntity`-Struct).

```rust
// Z.21-22:
    pub is_dynamic: bool,
    pub cap_planned_hours_to_expected: bool,
    // EINFÜGEN (numeric, KEIN bool):
    pub committed_voluntary: f32,
```

**Insertion point:** Direkt nach Z.22, vor der Leerzeile + `pub monday: bool` (Z.24). Typ ist `f32` wie `pub expected_hours: f32` (Z.13).

---

### 4. Service Struct + beide Konversionen — `service/src/employee_work_details.rs`

**Analog:** `cap_planned_hours_to_expected` (Z.26 / Z.56 / Z.207). Typ-Vorbild für `f32`: `pub expected_hours: f32` (Z.17).

**(a) Struct-Feld (`EmployeeWorkDetails`, Z.26):**
```rust
// Z.25-26:
    pub is_dynamic: bool,
    pub cap_planned_hours_to_expected: bool,
    // EINFÜGEN:
    pub committed_voluntary: f32,
```

**(b) `From<&Entity> for EmployeeWorkDetails` (Z.56):**
```rust
// Z.55-56:
            is_dynamic: working_hours.is_dynamic,
            cap_planned_hours_to_expected: working_hours.cap_planned_hours_to_expected,
            // EINFÜGEN:
            committed_voluntary: working_hours.committed_voluntary,
```

**(c) `TryFrom<&EmployeeWorkDetails> for Entity` (Z.207):**
```rust
// Z.206-207:
            is_dynamic: working_hours.is_dynamic,
            cap_planned_hours_to_expected: working_hours.cap_planned_hours_to_expected,
            // EINFÜGEN:
            committed_voluntary: working_hours.committed_voluntary,
```

> Beide Konversionen sind reine 1:1-Feld-Kopien (kein Cast) — `f32` auf beiden Seiten. Keine Omission-Lücke: fehlt eine Richtung → Compile-Error (Struct-Init unvollständig).

---

### 5. Update/Rotate Carry-Forward (CVC-02) — `service_impl/src/employee_work_details.rs`

**Analog:** Z.248 — der selektive Feld-für-Feld-Copy im `update()`-Pfad:
```rust
// Z.241-248 (bestehender selektiver Spread):
        entity.to_calendar_week = employee_work_details.to_calendar_week;
        entity.to_day_of_week = employee_work_details.to_day_of_week;
        entity.to_year = employee_work_details.to_year;
        entity.expected_hours = employee_work_details.expected_hours;
        entity.vacation_days = employee_work_details.vacation_days;
        entity.workdays_per_week = employee_work_details.workdays_per_week;
        entity.is_dynamic = employee_work_details.is_dynamic;
        entity.cap_planned_hours_to_expected = employee_work_details.cap_planned_hours_to_expected;
        // EINFÜGEN (nach Z.248):
        entity.committed_voluntary = employee_work_details.committed_voluntary;
```

**KRITISCH (CVC-02):** Dieser Pfad lädt das alte Entity per `find_by_id` (Z.228-232) und kopiert NUR die explizit gelisteten Felder. Wird `committed_voluntary` hier ausgelassen, schreibt die DAO den alten/Default-Wert zurück (silent reset) — exakt der Bug, gegen den der `cap_planned`-Test (Z.86 im Test-File) eine Regression absichert. Diese Zeile ist Pflicht, NICHT optional.

---

### 6. rest-types TO + beide Konversionen — `rest-types/src/lib.rs`

**Analog:** `cap_planned_hours_to_expected` (Z.610-611 / Z.650 / Z.688). Wire-Backward-Compat über `#[serde(default)]`.

**WICHTIG:** `EmployeeWorkDetailsTO` hat bewusst **kein** `ToSchema`/`#[utoipa::path]` (serde-transparent). KEINE OpenAPI-Annotation hinzufügen.

**(a) TO-Struct-Feld (Z.609-611):**
```rust
    pub is_dynamic: bool,
    #[serde(default)]
    pub cap_planned_hours_to_expected: bool,
    // EINFÜGEN (mit #[serde(default)] für Backward-Compat — alte Clients senden das Feld nicht):
    #[serde(default)]
    pub committed_voluntary: f32,
```

**(b) `From<&EmployeeWorkDetails> for EmployeeWorkDetailsTO` (Z.650):**
```rust
            is_dynamic: working_hours.is_dynamic,
            cap_planned_hours_to_expected: working_hours.cap_planned_hours_to_expected,
            // EINFÜGEN:
            committed_voluntary: working_hours.committed_voluntary,
```

**(c) `From<&EmployeeWorkDetailsTO> for EmployeeWorkDetails` (Z.688):**
```rust
            is_dynamic: working_hours.is_dynamic,
            cap_planned_hours_to_expected: working_hours.cap_planned_hours_to_expected,
            // EINFÜGEN:
            committed_voluntary: working_hours.committed_voluntary,
```

> Beide `From`-Impls sind `#[cfg(feature = "service-impl")]`-gated (Z.635 / Z.673) — unverändert lassen. Die abgeleiteten Display-Felder (`days_per_week`/`hours_per_day`/`hours_per_holiday`, Z.662-664) sind für `committed_voluntary` NICHT relevant (Feld inert, kein Display in Phase 14).

---

### 7. Overlap-SUM-Aggregations-Semantik (CVC-03, NUR Test in Phase 14) — `service_impl/src/reporting.rs`

**Analog (KOPIEREN):** `expected_hours`-SUM via `.fold` (Z.240-254) — die `acc + a`-Reduktion über die `find_working_hours_for_calendar_week`-Selektion.

**Selektions-Basis (Z.77-86, unverändert wiederverwenden):**
```rust
pub fn find_working_hours_for_calendar_week(
    working_hours: &[EmployeeWorkDetails],
    year: u32,
    week: u8,
) -> impl Iterator<Item = &EmployeeWorkDetails> {
    working_hours.iter().filter(move |wh| {
        (year, week) >= (wh.from_year, wh.from_calendar_week)
            && (year, week) <= (wh.to_year, wh.to_calendar_week)
    })
}
```

**SUM-Reduktions-Präzedenz (Z.254, `expected_hours` fold acc+a) — generalisiert auf numeric:**
```rust
// Vorlage für committed_voluntary (D-OVERLAP-AGG = SUM):
find_working_hours_for_calendar_week(&working_hours, year, week)
    .map(|wh| wh.committed_voluntary)
    .sum()
```

**NICHT KOPIEREN — Bool-`.any()`-Anti-Pattern (Z.264-265):**
```rust
// cap_planned ist bool → .any() — generalisiert NICHT auf einen numerischen Wert:
let cap_active = find_working_hours_for_calendar_week(&working_hours, year, week)
    .any(|wh| wh.cap_planned_hours_to_expected);
```

**Insertion point / Scope:** In Phase 14 entsteht KEIN Produktions-Read-Site (Feld inert). Per Claude's Discretion (CONTEXT Z.33): entweder ein wiederverwendbarer Accessor (`committed_voluntary_for_calendar_week`) oder nur ein getesteter, dokumentierter Helper. SC#4 verlangt nur, dass die SUM-Semantik definiert + per Test gepinnt ist. Der eigentliche Produktions-Read-/Aggregations-Pfad gehört zu Phase 15. Der Test pinnt: zwei überlappende Rows in derselben ISO-Woche (z.B. 5h + 5h → 10h).

---

### 8. Integration/Unit-Test-Anker — `service_impl/src/test/employee_work_details.rs`

**Analog:** Das gesamte File ist der exakte Carry-Forward-Test-Präzedenzfall. `update_propagates_cap_planned_hours_flag_to_dao` (Z.86-143) ist die line-für-line-Vorlage für den CVC-02-Carry-Forward-Test.

**(a) Test-Fixture-Helper (Z.54-80 `entity_with_cap`):** Muss um `committed_voluntary` ergänzt werden, sobald `EmployeeWorkDetailsEntity` das Feld trägt (sonst Compile-Error — Struct-Init unvollständig):
```rust
// Z.66-67 im Fixture:
        is_dynamic: false,
        cap_planned_hours_to_expected: cap,
        // EINFÜGEN (z.B. fester Wert für den Round-Trip):
        committed_voluntary: 0.0,
```
> Empfehlung: Helper-Signatur um einen Parameter erweitern (z.B. `entity_with_cap_and_committed(id, version, cap, committed)`) oder ein zweites Fixture, damit der fraktionale Wert (z.B. `2.5`) gesetzt werden kann.

**(b) Carry-Forward-Test (CVC-02) — Vorlage Z.86-143:** Spiegelt den `cap`-Test exakt, nur mit numerischem Feld. Kern-Assertion (Z.98-104) wird:
```rust
// Analog Z.98-104 (cap-Bool .with(function(|e| e.cap_planned...))):
dao.expect_update()
    .with(
        function(|e: &EmployeeWorkDetailsEntity| (e.committed_voluntary - 2.5).abs() < f32::EPSILON),
        always(),
        always(),
    )
    .returning(|_, _, _| Ok(()));
```
Setup: `find_by_id` liefert alten Wert (z.B. `0.0`), Input flippt auf `2.5` (analog Z.93-95 + Z.133-134), Assertion prüft, dass der NEUE Wert (nicht der stale geladene) an die DAO durchgereicht wird (CVC-02). Float-Vergleich über Epsilon, NICHT `==`.

**(c) Round-Trip-Test (CVC-01, Open→Save→Reload):** Per Claude's Discretion (CONTEXT Z.34) entweder hier erweitern oder neues Modul. Verifiziert, dass ein fraktionaler `committed_voluntary` (z.B. `2.5`) durch `From<Entity>` → Service → `TryFrom`/`From` → TO → zurück unverändert bleibt.

**(d) SUM-Aggregations-Test (CVC-03 / D-OVERLAP-AGG-TEST):** Zwei überlappende Rows in derselben ISO-Woche → SUM (5h + 5h → 10h). Pinnt die Semantik aus Site 7.

**Modul-Registrierung:** `employee_work_details` ist bereits in `service_impl/src/test/mod.rs` registriert — kein neuer `mod`-Eintrag nötig, wenn im selben File erweitert wird.

---

## Shared Patterns

### Numeric-Field-Threading (vs. Bool-Field)
**Source:** `expected_hours` (DAO `f64`-Row + `as f32`-Cast, Service/Entity/TO `f32`)
**Apply to:** Alle `committed_voluntary`-Touch-Sites
Der primäre Präzedenzfall `cap_planned_hours_to_expected` liefert die **Position + Reihenfolge** an jedem Touch-Site. Für den **Typ** (numeric statt bool) ist `expected_hours` das Vorbild:
- DAO-Row: `f64` (nicht `i64`)
- DAO `TryFrom`: `as f32`-Cast (nicht `!= 0`)
- DAO INSERT/UPDATE-Variable: `as f64`-Cast (nicht `as i64`)
- Entity/Service/TO: `f32` 1:1 (kein Cast)
- Reporting: `.sum()` (nicht `.any()`)

### Konversions-Vollständigkeit (keine Omission)
**Source:** Struct-Init-Pattern an jeder Boundary (Rust erzwingt vollständige Init)
**Apply to:** DAO `TryFrom`, Service `From`+`TryFrom`, TO beide `From`
Jede der 6 Konversionen ist eine vollständige Struct-Init → fehlt das Feld, schlägt der Compile fehl. Das ist das Sicherheitsnetz gegen silent `0.0`. EINZIGE Stelle ohne Compile-Schutz: der selektive Spread in `service_impl/.../employee_work_details.rs:248` (CVC-02) — dort kompiliert es auch bei Auslassung, daher per Test abgesichert.

### Wire-Backward-Compat
**Source:** `#[serde(default)]` auf `cap_planned_hours_to_expected` (rest-types Z.610)
**Apply to:** `EmployeeWorkDetailsTO.committed_voluntary`
Alte Clients senden das Feld nicht → `#[serde(default)]` liefert `0.0`. KEIN `ToSchema`/utoipa (Struct ist serde-transparent).

---

## No Analog Found

Keine. Alle 8 Touch-Sites haben einen exakten line-für-line-Präzedenzfall (`cap_planned_hours_to_expected` für Position/Threading, `expected_hours` für den numerischen Typ, `find_working_hours_for_calendar_week` + `expected_hours`-`.fold` für die SUM-Aggregation, das bestehende Test-File für Carry-Forward).

---

## Metadata

**Analog search scope:** `migrations/sqlite/`, `dao/src/`, `dao_impl_sqlite/src/`, `service/src/`, `service_impl/src/`, `service_impl/src/test/`, `rest-types/src/`
**Files scanned:** 8 (alle gelesen)
**Pattern extraction date:** 2026-06-23
**Build-Hinweis (NixOS):** `nix develop --command cargo sqlx prepare` + `cargo check --workspace` ist der erste harte Compile-Gate nach Migration. `sqlx migrate run` (additiv) — NIEMALS `sqlx database reset` (destruktiv).
