---
phase: 6
slug: rest-types-unification-frontend-compile-through
status: approved
shadcn_initialized: false
preset: not-applicable
created: 2026-05-07
reviewed_at: 2026-05-07
phase_character: compile-gate
visual_change: none
---

# Phase 6 — UI Design Contract

> **Phase 6 ist ein Compile-Gate, kein UI-Design.** Diese Spec definiert das
> *minimale visuelle Verhalten*, das die rest-types-Konsolidierung einhalten
> muss. Sichtbare UI-Closure (`current_paid_count`-Anzeige, Capacity-Editor,
> `VolunteerWork`/`UnpaidLeave`-Rendering, `cap_planned_hours_to_expected`-
> Settings-UI) ist explizit auf v1.3 verschoben — Requirements FUI-01..04 in
> `.planning/REQUIREMENTS.md`.
>
> **Visuelles Delta gegenüber heute: Null.** Alle bestehenden Komponenten,
> Tokens, Layouts, Strings bleiben byte-identisch. Phase 6 darf Tailwind-
> Klassen, RSX-Strukturen, Dialog-Layouts, Spacing oder Typografie *nicht*
> verändern.

---

## Phase-Charakter & Scope

| Aspekt | Status |
|--------|--------|
| Phase-Typ | Compile-Gate (Backend-Frontend-Wire-Schema-Alignment) |
| Visuelles Delta | None — Phase 6 ist visuell ein No-Op |
| Neue Komponenten | None |
| Neue Pages | None |
| Neue Dialoge / Forms | None |
| Neue Tokens (Spacing/Color/Type) | None |
| Neue i18n-Keys | None (no-op-Rendering rendert keinen sichtbaren Text) |
| Geänderte i18n-Keys | None |
| Neue Animations / A11y-Erweiterungen | None |
| Backlog-Owner für sichtbare UI-Closure | v1.3 (FUI-01..04) |

**Scope-Aussage:** Phase 6 macht den Compile-Pfad frei. Phase 7 (FC-03 / RC-01)
verifiziert, dass `dx serve` ohne Runtime-Panic startet und die bestehende
Shiftplan-Navigation visuell unverändert rendert. v1.3 baut die *sichtbare*
UI-Closure auf dieser Compile-Basis auf.

---

## Design System (Status quo, unverändert)

| Property | Value |
|----------|-------|
| Tool | none — Dioxus 0.6 + Tailwind, kein shadcn (React-only) |
| Preset | not applicable |
| Component library | Dioxus 0.6.1 RSX (kein Radix / Base UI) |
| Icon library | inline SVG / asset!()-bundled (siehe `assets/`) |
| Font | system stack (Tailwind default) — siehe `tailwind.config.js` |
| Tokens-Quelle | `shifty-dioxus/input.css` (CSS-Vars) + `tailwind.config.js` |
| Self-policing Lint | `no_legacy_classes_in_source` Tests in jeder Page |

**Phase 6 ändert keinen dieser Werte.** Die Detection ist hier nur dokumentiert,
damit der Checker und v1.3-Researcher den Status quo kennen.

---

## Spacing Scale

> N/A für neuen UI-Code — Phase 6 fügt keine UI hinzu. Bestehende Tokens in
> `tailwind.config.js` und `input.css` bleiben unverändert.

Falls in Phase 6 *unerwartet* doch ein RSX-Block geändert werden muss (z.B.
weil ein Match-Arm in einer bestehenden Komponente erweitert wird und der
Compiler einen sichtbaren Branch erzwingt): KEINE neuen `class:`-Strings, KEINE
inline-Styles. Ausschließlich `return rsx! {};` (siehe Pattern unten).

| Token | Value | Usage |
|-------|-------|-------|
| (n/a) | — | Phase 6 fügt keine UI-Elemente hinzu; bestehende Spacing-Tokens bleiben |

Exceptions: none — keine Phase-6-spezifischen Spacing-Entscheidungen.

---

## Typography

> N/A für neuen UI-Code — siehe v1.3 (FUI-01..04) für sichtbare UI-Erweiterungen.

| Role | Size | Weight | Line Height |
|------|------|--------|-------------|
| (n/a) | — | — | — |

Bestehende Typo-Tokens (siehe CONVENTIONS.md):
- `text-micro` (11px / 600) — unverändert
- `text-small` (12px / 500) — unverändert
- `text-body` (14px / 400) — unverändert
- `text-lg` (16px / 600) — unverändert (siehe `tailwind.config.js:51`)

Phase 6 fügt KEINE neuen Typo-Klassen hinzu.

---

## Color

> N/A für neuen UI-Code. Bestehende CSS-Vars (`--bg`, `--surface`, `--ink`,
> `--accent`, `--good`, `--warn`, `--bad`, …) bleiben unverändert.

| Role | Value | Usage |
|------|-------|-------|
| (n/a) | — | Phase 6 verändert keine Farb-Token-Belegung |

Accent reserved for: status quo — Phase 6 ändert keine Farb-Reservierungen.

**Wichtig:** Die `WarningTO::PaidEmployeeLimitExceeded`-Variante (aus v1.1) wird
in Phase 6 NICHT visuell hervorgehoben (kein `--warn`-Badge, kein gelber
Banner). Das ist FUI-01 Backlog. In Phase 6 rendert sie nichts (siehe
"No-Op-Rendering-Pattern" unten).

---

## Copywriting Contract

> Kein neues sichtbares UI heißt: kein neuer Copy.

| Element | Copy |
|---------|------|
| Primary CTA | (n/a — keine neuen CTAs) |
| Empty state heading | (n/a) |
| Empty state body | (n/a) |
| Error state | (n/a — bestehender `ERROR_STORE`-Banner unverändert) |
| Destructive confirmation | (n/a — keine neuen destruktiven Aktionen) |

**i18n-Pflicht:** Keine. Phase 6 fügt KEINEN `Key`-Variant zu
`shifty-dioxus/src/i18n/mod.rs` hinzu. Die i18n-Parity-Tests
(`shifty-dioxus/src/i18n/mod.rs:422+`) bleiben grün ohne Edits.

Falls in einem Plan-Wave doch ein versehentlich sichtbarer String entsteht
(z.B. Debug-Text in einem Match-Arm): SOFORT zurückrollen, das ist keine
Compile-Gate-Phase-Aktivität, sondern v1.3-Scope.

---

## No-Op-Rendering-Pattern (CORE DESIGN CONTRACT für Phase 6)

Dies ist der **eigentliche Design-Kontrakt** der Phase. Er beschreibt, *wie*
neue Backend-Enum-Varianten und TO-Felder im Frontend behandelt werden, ohne
sichtbare UI-Erweiterung.

### Regel 1 — Match-Arme: invisible-skip via `rsx! {}`

Wenn ein bestehender `match`-Ausdruck in `rsx!`-Kontext einen neuen Arm braucht
(z.B. `WarningTO::PaidEmployeeLimitExceeded` in einem Warning-Renderer), nutze
**ausschließlich** das in der Codebase bereits etablierte Empty-RSX-Pattern:

```rust
match warning {
    WarningTO::AbsenceConflict { .. } => rsx! { /* bestehender Renderer */ },
    WarningTO::CrossSourceConflict { .. } => rsx! { /* bestehender Renderer */ },
    WarningTO::PaidEmployeeLimitExceeded { .. } => rsx! {},  // v1.3 FUI-01
    WarningTO::FutureVariant1 { .. } => rsx! {},             // v1.3 FUI-* (placeholder)
    WarningTO::FutureVariant2 { .. } => rsx! {},             // v1.3 FUI-* (placeholder)
}
```

**Begründung:** Dioxus' `rsx! {}` (leerer Block) ist im Codebase als
Empty-Render-Pattern etabliert (siehe `src/component/dialog.rs:139` —
`if !props.open { return rsx! {}; }`). Es produziert keinen DOM-Output, keine
neuen Wrapper-Knoten, keinen Whitespace.

**Verbot:** Kein `unimplemented!()`, kein `todo!()`, kein `panic!()`, kein
unsichtbarer Marker-Span (`span { class: "hidden", ... }`). Diese würden
entweder die WASM-Runtime crashen (panic-Familie) oder die DOM-Tree-Topologie
ändern (Marker-Span).

### Regel 2 — TO-Felder: ignore-on-state-mapping

Wenn ein neues Feld auf einem bestehenden TO landet (`SlotTO.max_paid_employees`,
`ShiftplanSlotTO.current_paid_count`, `ShiftplanDayTO.unavailable`,
`BillingPeriodTO.snapshot_schema_version`):

- **Wenn das Feld eine bestehende state-Domain-Type-Spiegelung hat** (z.B.
  `Slot` in `src/state/shiftplan.rs`): Feld zur state-struct hinzufügen (mit
  passendem Default — `Option<u8>` → `None`, `u8` → `0`, etc.), in beiden
  `From`-Richtungen mappen, aber im Render-Code NICHT lesen. Bestehende
  Render-Pfade ignorieren das Feld einfach. v1.3 schaltet die Anzeige ein.
- **Wenn das Feld nur in einem nicht-gerenderten Diagnostic-Pfad lebt** (z.B.
  `BillingPeriodTO.snapshot_schema_version`): Feld in den state-Mirror
  übernehmen, aber komplett render-frei lassen.

### Regel 3 — `panic!`-Branches: defensive Fallback-Variant statt Panic

CONCERNS.md §9 listet drei `panic!`-Sites in `src/state/employee.rs:89/151` und
`src/state/shiftplan.rs:59`. Phase 6 KANN diese defensiv konvertieren, falls
ein Plan-Wave einen Match-Arm dort erweitern muss. Pattern:

```rust
// VORHER (CONCERNS §9):
panic!("Unknown working hours category: {}", identifier)

// NACHHER (defensiv, kein neues sichtbares UI):
WorkingHoursCategory::Unknown(identifier.into())
```

Die `Unknown`-Variant rendert in keinem bestehenden Render-Pfad sichtbar (kein
neuer Match-Arm in `rsx!`-Kontext nötig — wenn doch, dann Regel 1: `rsx! {}`).

**Wenn ein Plan keine `panic!`-Site berührt:** unverändert lassen. Phase 6
verlangt nicht, dass *alle* Panics sofort entfernt werden — nur die, die im
Match-Arm-Erweiterungspfad eines Plan-Wave liegen.

### Regel 4 — Visuelles Delta = 0

`cargo build --target wasm32-unknown-unknown` muss grün werden, OHNE dass
`dx serve` ein einziges sichtbares Pixel anders rendert als heute. Das ist
das stärkste Akzeptanzkriterium.

Wenn ein Plan-Wave-PR irgendetwas am gerenderten DOM ändert, ist das ein
Phase-6-Verstoß (Visual-Regression im Compile-Gate). Solche Änderungen
gehören in v1.3-FUI-Phasen.

---

## Touch-Point-Inventar (für Planner-Decomposition)

Dies ist die **definitive Liste der Stellen**, an denen Match-Arme erweitert
oder Felder gespiegelt werden müssen. Quelle: CONCERNS.md §1 (Drift-Inventur).

### A. Modul-Mengen mit erschöpfungs-pflichtigen Match-Arms

Diese Module enthalten `match`-Statements gegen Backend-Enums, die nach
dem Cargo-Swap (RT-01) nicht mehr exhaustive sein werden:

| Modul | Enum | Erweiterung |
|-------|------|-------------|
| `shifty-dioxus/src/state/employee.rs` | `ExtraHoursCategory` (lokal) ↔ `ExtraHoursCategoryTO` | `UnpaidLeave`, `VolunteerWork` (auf wire bereits, im From-Impl fehlend) — siehe CONCERNS §1.C |
| `shifty-dioxus/src/state/shiftplan.rs` | `Weekday`-Konversionen, neue `unavailable`-Lese-Branches | weekday-num panic → defensive Fallback (Regel 3) |
| `shifty-dioxus/src/loader.rs` | TO → state mapping aller drift-betroffenen TOs | `SlotTO.max_paid_employees`, `ShiftplanSlotTO.current_paid_count`, `ShiftplanDayTO.unavailable`, `BillingPeriodTO.snapshot_schema_version` durchreichen |
| `shifty-dioxus/src/api.rs` | Endpoint-Wrapper für Cutover-DTOs, `BookingCreateResultTO`, `CopyWeekResultTO`, `AbsencePeriodCreateResultTO`, `ImpersonateTO`, `ToggleTO`/`ToggleGroupTO`, `ShiftplanAssignmentTO`, `AbsencePeriodTO`, `AbsenceCategoryTO`, `WarningTO`, `UnavailabilityMarkerTO`, `ExtraHoursCategoryDeprecatedErrorTO` | Wrapper-Shape-Anpassung; KEIN neues UI |
| `shifty-dioxus/src/state/user_management.rs` | Fork-eigene `ShiftplanAssignment`-struct → Backend-`ShiftplanAssignmentTO` | Lokale struct durch Backend-TO ersetzen oder als state-Mirror passend mappen |

### B. Render-Sites mit potentiellen Match-Erweiterungen

Diese Render-Komponenten *könnten* (nicht: müssen) Match-Arm-Erweiterungen
erfordern, je nach Wave-Decomposition. Falls ja: **Regel 1 (rsx! {}) anwenden,
nichts visuell ändern.**

| Komponente | Aktueller Match-Kontext | No-Op-Branch |
|------------|------------------------|--------------|
| `src/component/week_view.rs` | Slot-Cell-Rendering, Booking-Liste | Falls Warning-Renderer dort lebt: `WarningTO::PaidEmployeeLimitExceeded => rsx! {}` |
| `src/page/shiftplan.rs` | Day/Slot-Iteration, Booking-Aktionen | Falls Match auf `WarningTO` oder `UnavailabilityMarkerTO` neue Arme braucht: `rsx! {}` |
| `src/component/employee_view.rs` | ExtraHoursCategory-Rendering | Falls neuer `UnpaidLeave`/`VolunteerWork`-Arm sichtbar werden muss: aktuell sind beide bereits visuell vorhanden (CONCERNS §1.C — wire-side OK, From-Impl-Side stale); nur `From`-Impl-Korrektur, kein RSX-Edit |
| `src/component/extra_hours_modal.rs` | Kategorie-Auswahl | Falls Dropdown-Render einen Match braucht: `rsx! {}` für UnpaidLeave/VolunteerWork in v1.2; v1.3 FUI-03 schaltet Anzeige ein |
| `src/component/booking_log_table.rs` | Warning-Rendering | `WarningTO::PaidEmployeeLimitExceeded => rsx! {}` |

### C. State-Mirror-Erweiterungen (Felder zu spiegeln)

| state struct | Neues Feld | Quelle | Render-Verhalten |
|--------------|-----------|--------|------------------|
| `state::shiftplan::Slot` | `max_paid_employees: Option<u8>` | `SlotTO.max_paid_employees` (CONCERNS §1.B) | NOT rendered in v1.2; FUI-02 in v1.3 |
| `state::shiftplan::ShiftplanSlot` (oder Aequivalent) | `current_paid_count: u8` | `ShiftplanSlotTO.current_paid_count` (CONCERNS §1.B) | NOT rendered in v1.2; FUI-01 in v1.3 |
| `state::shiftplan::ShiftplanDay` (oder Aequivalent) | `unavailable: Option<UnavailabilityMarker>` | `ShiftplanDayTO.unavailable` (CONCERNS §1.B) | NOT rendered in v1.2; v1.3 entscheidet visuelle Auflösung |
| `state::*::BillingPeriod` (falls existent) | `snapshot_schema_version: u32` | `BillingPeriodTO.snapshot_schema_version` (CONCERNS §1.B) | NOT rendered (diagnostic field); state-Mirror nur für serde-Kompatibilität |

---

## Registry Safety

> N/A — kein shadcn, kein Third-Party-Block-Import. Phase 6 zieht keinen
> externen UI-Code in den Build.

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| (none) | — | not applicable |

---

## Anti-Goals (was Phase 6 explizit NICHT tut)

| Anti-Goal | Owner |
|-----------|-------|
| `current_paid_count`-Anzeige im Week-View hinzufügen | v1.3 FUI-01 |
| Capacity-Editor in Slot-Settings rendern | v1.3 FUI-02 |
| `VolunteerWork` / `UnpaidLeave` als sichtbare ExtraHours-Kategorien anzeigen | v1.3 FUI-03 |
| `cap_planned_hours_to_expected` im Settings-UI exponieren | v1.3 FUI-04 |
| `WarningTO::PaidEmployeeLimitExceeded` als Banner/Badge rendern | v1.3 (FUI-01-Followup) |
| `UnavailabilityMarkerTO` als Day-Marker visuell zeigen | v1.3 (separat zu FUI-*) |
| Spacing/Color/Typography-Tokens ändern | v1.3 oder later (kein Bedarf erkannt) |
| Neue i18n-Keys hinzufügen | v1.3 (gemeinsam mit FUI-*) |
| `panic!`-Sites in `state/employee.rs` / `state/shiftplan.rs` *flächendeckend* defensiv machen | optional in Phase 6 (nur falls Wave-PR die Site berührt; siehe Regel 3); sonst Backlog |
| `api.rs`-Refaktorierung (CONCERNS §3 — Domain-Split, fetch_json-Helper) | Backlog (eigene Phase / Milestone) |
| `unwrap()`/`expect()`-Audit (CONCERNS §2) | Backlog |

---

## Akzeptanzkriterien (Verzahnung mit ROADMAP-Success-Criteria)

Diese UI-Spec ist gehalten, wenn alle ROADMAP-Phase-6-Success-Criteria
(`.planning/ROADMAP.md` Phase 6, Punkte 1–5) erfüllt sind UND zusätzlich:

1. **Visuelles Delta = 0:** Ein Side-by-Side-Vergleich `dx serve` (vor Phase 6
   committed state) vs. `dx serve` (nach Phase 6) zeigt für die Shiftplan-
   Hauptseite, das Booking-Modal, die Employee-Übersicht und Billing-Period-
   Detail keinen sichtbaren Pixel-Unterschied. (Phase 7 FC-03 verifiziert das
   indirekt durch Runtime-Smoke.)
2. **No-Op-Rendering-Pattern (Regel 1) konsistent angewandt:** Alle in Phase 6
   neu hinzugefügten Match-Arme in `rsx!`-Kontext rendern entweder `rsx! {}`
   oder reichen einen bestehenden Renderer durch. Keine `unimplemented!()`/
   `todo!()`/`panic!()`-Branches.
3. **Keine neuen Tokens / Klassen:** `git diff shifty-dioxus/tailwind.config.js
   shifty-dioxus/input.css` zeigt nur formatierungs-neutrale Edits oder ist
   leer. Keine neuen `text-*`, `bg-*`, `border-*`-Tokens.
4. **Keine neuen i18n-Keys:** `git diff shifty-dioxus/src/i18n/mod.rs
   shifty-dioxus/src/i18n/en.rs shifty-dioxus/src/i18n/de.rs
   shifty-dioxus/src/i18n/cs.rs` zeigt keine `Key::`-Erweiterungen oder neue
   `add_text`-Aufrufe.
5. **State-Mirror-Felder mit konsistenten Defaults:** Alle in §C neu
   gespiegelten Felder haben einen sinnvollen Default (`Option<u8>::None`,
   `u8: 0`, `u32: 0`, `Option<UnavailabilityMarker>::None`), sodass
   bestehende Test-Fixtures ohne Edits weiter laufen.

---

## Checker Sign-Off

> Phase 6 ist Compile-Gate, kein UI-Design-Phase. Die 6 Standard-UI-Dimensionen
> sind weitgehend N/A — der Checker sollte gegen den **No-Op-Kontrakt** prüfen,
> nicht gegen Visuelles.

- [ ] Dimension 1 Copywriting: PASS — keine neuen Strings hinzugefügt
- [ ] Dimension 2 Visuals: PASS — visuelles Delta = 0
- [ ] Dimension 3 Color: PASS — Token-Belegung unverändert
- [ ] Dimension 4 Typography: PASS — keine neuen Typo-Klassen
- [ ] Dimension 5 Spacing: PASS — kein neues Spacing
- [ ] Dimension 6 Registry Safety: PASS — kein Third-Party-Pull
- [ ] Phase-spezifisch: No-Op-Rendering-Pattern (Regel 1) auf alle neuen Match-Arme angewandt
- [ ] Phase-spezifisch: State-Mirror-Erweiterungen (§C) mit korrekten Defaults

**Approval:** pending
