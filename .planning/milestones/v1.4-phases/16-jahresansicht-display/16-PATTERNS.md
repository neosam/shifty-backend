# Phase 16: Jahresansicht display - Pattern Map

**Mapped:** 2026-06-24
**Files analyzed:** 7 (modified) + 0 (created)
**Analogs found:** 7 / 7 (every target file is its own analog — `volunteer_hours`/Band-2 handling is the exact template for `committed_voluntary_hours`/Band-1)

> **Leitprinzip dieser Phase:** Es entstehen KEINE neuen Dateien. Jede Ziel-Datei
> enthält bereits die exakte Vorlage in Form der bestehenden `volunteer_hours`-
> (Band 2) bzw. `paid_hours`-Behandlung. Der Planner-Auftrag lautet durchgängig:
> „Mach genau das, was für `volunteer_hours` schon dasteht — nochmal, für
> `committed_voluntary_hours`." Alle Zeilennummern sind am HEAD verifiziert.

---

## File Classification

| Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---------------|------|-----------|----------------|---------------|
| `service_impl/src/booking_information.rs` | service (business-logic) | transform / request-response | dieselbe Datei, `volunteer_hours` + `overall_available_hours` (Z.273) | self / exact |
| `rest-types/src/lib.rs` (`WeeklySummaryTO`) | model / DTO | transform (`From<&WeeklySummary>`) | dieselbe Struct, `volunteer_hours`-Feld + Mapping-Arm | self / exact |
| `shifty-dioxus/src/state/weekly_overview.rs` | store / state | transform (`From<&WeeklySummaryTO>`) | dieselbe Struct, `volunteer_hours`-Feld + Mapping-Arm | self / exact |
| `shifty-dioxus/src/page/weekly_overview.rs` | component (page) | request-response (render) | `🤝{volunteer}`-Token (Z.103/108) + `sample_week` (Z.218) | self / exact |
| `shifty-dioxus/src/component/weekly_overview_chart.rs` | component | request-response (render) | volunteer-Segment (Z.122-124) + `bar_total` (Z.16) + Legende (Z.82-85) | self / exact |
| `shifty-dioxus/src/i18n/mod.rs` (`Key`-Enum) | config | transform | `Key::PaidVolunteer` (Z.120) / `Key::Volunteer` (Z.344) | self / exact |
| `shifty-dioxus/src/i18n/{de,en,cs}.rs` | config | transform | `Key::Volunteer`/`Key::PaidVolunteer` add_text-Zeilen | self / exact |

**Genau ZWEI Stellen sind KEIN reines Klonen, sondern Erweiterung der Summen-Formel:**
- Backend `overall_available_hours` (Z.273) — additiver Term.
- Chart `bar_total` (Z.16) — additiver Term.
Beides ist trotzdem eine 1-Token-Änderung an einer Additionszeile.

---

## Pattern Assignments

### `service_impl/src/booking_information.rs` (service, transform) — NUR erste Variante

**Analog:** dieselbe Datei, die bereits-gewirte `volunteer_hours`-/`committed_voluntary_hours`-Berechnung (Z.207-226) und die `overall_available_hours`-Summe (Z.273).

**Verifizierter Befund (Research-Flag aufgelöst):** Es gibt zwei `WeeklySummary`-
Producer im File. Phase 16 ändert **genau eine Zeile in der ERSTEN Variante**.

| Variante | Methode | Zeilen | committed-Status | Phase-16-Aktion |
|----------|---------|--------|------------------|-----------------|
| 1 (Jahresansicht / Achse B) | `get_weekly_summary(year, …)` | 136–295 | voll gewired (Band 1 Z.219-226, Band 2 Z.207-216) | **Z.273 ändern** |
| 2 (Einzel-Woche `…/year/{week}`) | `get_summery_for_week(year, week, …)` | 297–562 | `committed_voluntary_hours: 0.0` Placeholder (Z.547), Kommentar Z.544 „year-view-only" | **KEINE Änderung** (Z.386 + Z.547 bleiben) |

**Was Phase 15 schon liefert (NICHT anfassen — Z.219-226):**
```rust
// Band 1 (D-04 / CVC-04): cap-gated Σ_person committed per week (flat, no weight D-03).
let committed_voluntary_hours: f32 = find_working_hours_for_calendar_week(
    &all_work_details, year, week,
)
.filter(|wh| wh.cap_planned_hours_to_expected) // CVC-06 gate, per row
.map(|wh| wh.committed_voluntary)
.sum();
```

**Die EINZIGE Backend-Änderung (Z.270-273) — D-01:**
```rust
// IST (Z.270-273) — Phase-15-TODO:
// overall_available_hours stays volunteer_hours + paid_hours (Pitfall 2 — Phase 16 wires display).
// NOTE: volunteer_hours is now the per-person surplus (Band 2); the pledge (Band 1) lives
// in committed_voluntary_hours separately. Phase 16 will sum both bands for display.
let overall_available_hours = volunteer_hours + paid_hours;

// SOLL — committed-Term ergänzen (D-01):
let overall_available_hours = committed_voluntary_hours + volunteer_hours + paid_hours;
```
> Der `WeeklySummary { … }`-Literal-Arm (Z.274-290) befüllt `committed_voluntary_hours`
> bereits korrekt (Z.280) — **keine Änderung dort**. rustc erzwingt das Feld; der Fix
> ist ausschließlich die Summenzeile.

**No-double-count-Invariante (vom User bestätigt, D-04):** Band 2 (`volunteer_hours`)
hat committed bereits per-Person abgezogen (`Σ max(actual−committed,0)`). Daher ist
`paid + committed + volunteer = paid + max(committed, actual)` korrekt — keine
Doppelzählung.

**Test-Analog:** bestehende `service_impl`-Tests zu `get_weekly_summary` erweitern um
einen Fall, der `overall_available_hours == paid + committed + volunteer` pinnt
(CVC-07a). Befehl: `cargo test -p service_impl booking_information`.

---

### `rest-types/src/lib.rs` — `WeeklySummaryTO` (model, transform)

**Analog:** das `volunteer_hours`-Feld (Z.911) + sein Mapping-Arm (Z.930) in derselben Struct.

**Struct-Feld hinzufügen (nach Z.911 `volunteer_hours`):**
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeeklySummaryTO {
    // … year, week, overall_available_hours, required_hours, paid_hours …
    pub volunteer_hours: f32,
    #[serde(default)]               // ← Pitfall 7: Wire-Backward-Compat (Pattern wie EmployeeWorkDetailsTO, Phase 14)
    pub committed_voluntary_hours: f32,
    // … monday_available_hours … working_hours_per_sales_person
}
```

**Mapping-Arm hinzufügen (nach Z.930 `volunteer_hours: weekly_summary.volunteer_hours,`):**
```rust
#[cfg(feature = "service-impl")]
impl From<&WeeklySummary> for WeeklySummaryTO {
    fn from(weekly_summary: &WeeklySummary) -> Self {
        Self {
            // …
            volunteer_hours: weekly_summary.volunteer_hours,
            committed_voluntary_hours: weekly_summary.committed_voluntary_hours, // ← neu, exakt analog
            // …
        }
    }
}
```
> **Kein `ToSchema`/`#[utoipa::path]`:** `WeeklySummaryTO` hat keine OpenAPI-Anbindung
> (verifiziert). Kein OpenAPI-Task, kein `ToSchema`-Derive ergänzen.
> **Kein Snapshot-Bump:** `WeeklySummary` ist year-view-only, nicht persistiert.
> `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 7.

**Test-Analog (Wave-0-Lücke):** From-Mapping-Roundtrip-Test
`WeeklySummaryTO::from(&ws).committed_voluntary_hours == ws.committed_voluntary_hours`
(`cargo test -p rest-types` oder `-p service_impl`).

---

### `shifty-dioxus/src/state/weekly_overview.rs` (store, transform)

**Analog:** das `volunteer_hours`-Feld (Z.18) + sein Mapping-Arm (Z.37) in derselben Datei.

**Struct-Feld hinzufügen (nach Z.18 `pub volunteer_hours: f32,`):**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct WeeklySummary {
    // … available_hours, required_hours, paid_hours …
    pub volunteer_hours: f32,
    pub committed_voluntary_hours: f32,   // ← neu
    // … monday_available_hours … sales_person_absences
}
```

**Mapping-Arm hinzufügen (nach Z.37 `volunteer_hours: summary.volunteer_hours,`):**
```rust
impl From<&WeeklySummaryTO> for WeeklySummary {
    fn from(summary: &WeeklySummaryTO) -> Self {
        Self {
            // …
            volunteer_hours: summary.volunteer_hours,
            committed_voluntary_hours: summary.committed_voluntary_hours, // ← neu, exakt analog
            // …
        }
    }
}
```
> **Pitfall 1 (Omission-Lücke):** rustc erzwingt nur, dass das Feld im Literal *vorkommt*,
> nicht dass es korrekt befüllt ist. `Default::default()` würde kompilieren und die UI
> zeigte stumm `0.00`. **From-Mapping ist erstes Review-Ziel.** Mapping-Roundtrip-Test
> als Wave-0-Gate.
> Hilfsmethoden `monday_date()`/`sunday_date()` (Z.65-72) bleiben unverändert.

---

### `shifty-dioxus/src/page/weekly_overview.rs` (component/page, render)

**Analog:** der bestehende `🤝{volunteer}`-Token (Z.103 Desktop, Z.108 Mobile) und der
lokale `sample_week`-Helper (Z.218).

**Token-Rendering — ZWEI identische Stellen (Z.103 + Z.108):**
```rust
// IST (Z.103, Desktop-Zelle; Z.108 ist byte-identisch in der Mobile-Zeile):
{format!("💰{} | 🤝{}", format_hours(week.paid_hours, 2), format_hours(week.volunteer_hours, 2))}

// SOLL (D-02 — drei getrennte Tokens, NIE kombiniert):
{format!("💰{} | 🎯{} | 🤝{}",
    format_hours(week.paid_hours, 2),
    format_hours(week.committed_voluntary_hours, 2),   // 🎯 = Band 1 (Zusage)
    format_hours(week.volunteer_hours, 2))}            // 🤝 = Band 2 (Surplus)
```
> **D-03 (0-Regel):** `committed == 0` rendert ganz normal `🎯0.00` — keine blank/Strich-
> Sonderlogik. `format_hours(value, 2)` wiederverwenden (bereits importiert Z.7
> `base_types::{format_hours, ImStr}`). KEINE neue Format-Funktion.
> **Beide Stellen gleichzeitig ändern** — Desktop UND Mobile.

**Tabellen-Header (Z.58 + Z.72):**
```rust
// IST (Z.58):
let paid_volunteer = i18n.t(Key::PaidVolunteer);
// SOLL — neuer Key (UI-SPEC Copywriting-Entscheidung: NICHT PaidVolunteer mutieren):
let paid_committed_volunteer = i18n.t(Key::PaidCommittedVolunteer);
// und im th (Z.72): "{paid_committed_volunteer}"
```

**`sample_week`-Helper erweitern (Z.218-235) — Pitfall 6:**
```rust
// IST-Signatur (Z.218):
fn sample_week(year: u32, week: u8, paid: f32, volunteer: f32, required: f32) -> WeeklySummary {
    WeeklySummary {
        // …
        available_hours: paid + volunteer,           // ← Z.222
        volunteer_hours: volunteer,                  // ← Z.225
        // …
    }
}
// SOLL — committed-Parameter ergänzen:
fn sample_week(year: u32, week: u8, paid: f32, committed: f32, volunteer: f32, required: f32) -> WeeklySummary {
    WeeklySummary {
        // …
        available_hours: paid + committed + volunteer,   // D-01-konsistent
        committed_voluntary_hours: committed,            // ← neu
        volunteer_hours: volunteer,
        // …
    }
}
```
> ⚠ Es gibt ZWEI lokale `sample_week`-Definitionen (hier Z.218 + Chart Z.153). **Beide
> identisch erweitern**, sonst kompilieren die Tests mit falschen Werten (Pitfall 6).
> Alle bestehenden Aufrufer (z.B. `build_full_year` Z.283, `sample_week(year, w, 20.0, 5.0, 30.0)`)
> um das committed-Argument nachziehen.

**SSR-Test-Harness (Analog Z.237-244 `render_table` + `VirtualDom` + `dioxus_ssr::render`):**
neuen SSR-Test andocken (CVC-07 d/e/f):
- Desktop + Mobile rendern `🎯`-Token.
- `committed=5, actual=7` → `🎯5.00 | 🤝2.00`.
- `committed=0` → `🎯0.00` (D-03, keine blank-Sonderlogik).

**Source-Audit-Test (Z.553-592) — NICHT brechen:** `page_source_does_not_use_legacy_classes`
prüft den Prod-Source (vor `#[cfg(test)]`) auf Legacy-Klassen (Page) **und** verbotene
Hex-Literale im Chart-Prod-Source. Der committed-Token darf keine neuen Legacy-Klassen
einführen.

---

### `shifty-dioxus/src/component/weekly_overview_chart.rs` (component, render)

**Analog:** das volunteer-Segment (Z.122-124), `bar_total` (Z.16), Legenden-Eintrag
(Z.82-85), Tooltip (Z.106-112), Wrapper-Props (Z.47-51 / Z.60-63), `vol_pct` (Z.100).

**`bar_total` erweitern (Z.16) — D-04 / Pitfall 3:**
```rust
// IST (Z.16):
let bar_total = w.paid_hours + w.volunteer_hours;
// SOLL:
let bar_total = w.paid_hours + w.committed_voluntary_hours + w.volunteer_hours;
```
> Sonst weicht die Balkenhöhe stumm von der Diff-Spalte (D-01) ab (Pitfall 3).

**Drittes Segment einfügen (Analog volunteer-Segment Z.122-124) — Stapel bottom→top:
paid · committed · surplus (UI-SPEC §Color):**
```rust
// Vorhandenes Muster (Z.99-100 Prozente, Z.122-128 Segmente):
let paid_pct = (week.paid_hours / max_hours) * 100.0;
let vol_pct  = (week.volunteer_hours / max_hours) * 100.0;
// → committed_pct analog ergänzen:
let committed_pct = (week.committed_voluntary_hours / max_hours) * 100.0;

// Render-Reihenfolge (oben→unten im Flex-column-reverse-Stapel): surplus, committed, paid
// Surplus (Z.122-124, unverändert):
if week.volunteer_hours > 0.0 {
    div { style: "height: {vol_pct}%; background: var(--ink-muted); opacity: 0.35;" }
}
// Committed — NEU, zwischen surplus und paid, HARD CONSTRAINT var(--good):
if week.committed_voluntary_hours > 0.0 {
    div { style: "height: {committed_pct}%; background: var(--good);" }
}
// Paid (Z.126-128, unverändert):
if week.paid_hours > 0.0 {
    div { style: "height: {paid_pct}%; background: {paid_bg};" }
}
```
> **Farb-HARD-CONSTRAINT (UI-SPEC §Color):** committed = `var(--good)` (grün, „gesichert/
> zugesagt"). Ausschließlich CSS-Custom-Property, **niemals Roh-Hex**. Verbotene Hexes
> (vom Test gepinnt): `#3B82F6`, `#10B981`, `#EF4444`, `#e5e7eb`, `#6b7280`, `#374151`
> (Pitfall 4).

**Legenden-Eintrag (Analog volunteer Z.82-85), 3. Item:**
```rust
span { class: "inline-flex items-center gap-1.5",
    span { style: "background: var(--good); width: 12px; height: 12px; border-radius: 2px; display: inline-block;" }
    "{committed_label}"
}
```

**Tooltip (Z.106-112) — alle drei Werte (D-04):** `committed_label {committed}h` zwischen
paid und volunteer einfügen; `format_hours(week.committed_voluntary_hours, 1)` (1 Dezimal
im Chart, exakt wie paid/volunteer dort).

**Props/Wrapper-Threading (Analog `volunteer_label`):**
- `WeeklyOverviewChart` (Z.47-51): `committed_label: i18n.t(Key::Committed).to_string(),`
- `WeeklyOverviewChartView`-Props (Z.60-63): `committed_label: String,`
- Test-`ViewProps` (Z.200-209) + `render_view` (Z.211-244): `committed_label` ebenfalls
  durchreichen (sonst kompilieren die Chart-Tests nicht).

**Test-Erweiterung:** `chart_uses_token_styles_not_legacy_hex` (Z.247-267) um eine
Assertion `html.contains("var(--good)")` für das committed-Segment ergänzen; verbotene
Hexes bleiben absent. `compute_max_hours`-Test (Z.173-180) um committed im `bar_total`
nachziehen.

---

## Shared Patterns

### i18n: neue Keys in allen drei Locales (CVC-08)
**Source-Analog:** `Key::PaidVolunteer` (mod.rs:120) + `Key::Volunteer` (mod.rs:344) und
ihre `add_text`-Zeilen.
**Apply to:** alle benutzersichtbaren neuen Strings.

**Key-Enum (`i18n/mod.rs`) — neue Varianten (nahe Z.120 / Z.344):**
```rust
PaidCommittedVolunteer,   // Tabellen-Header, ersetzt PaidVolunteer-Nutzung in weekly_overview.rs:58
Committed,                // Token-/Chart-Legend-Label
```
> `Key::PaidVolunteer` (Z.120) bleibt definiert (UI-SPEC: nicht mutieren, Name bleibt
> selbstbeschreibend; andere Konsumenten unberührt).

**Drei Locales (exakt gespiegeltes `add_text`-Muster, je Datei eigener `Locale::*`):**

| Key | `de.rs` (`Locale::De`) | `en.rs` (`Locale::En`) | `cs.rs` (`Locale::Cs`) |
|-----|------------------------|------------------------|------------------------|
| `Key::Committed` | `"Zugesagt"` | `"Committed"` | `"Přislíbeno"` |
| `Key::PaidCommittedVolunteer` | `"Bezahlt / Zugesagt / Freiwillig"` | `"Paid / Committed / Volunteer"` | `"Placené / Přislíbeno / Dobrovolné"` |

Add-text-Muster (analog de.rs:572 `Key::Volunteer`):
```rust
i18n.add_text(Locale::De, Key::Committed, "Zugesagt");
i18n.add_text(Locale::En, Key::Committed, "Committed");
i18n.add_text(Locale::Cs, Key::Committed, "Přislíbeno");
```
> **Czech MEDIUM-confidence** (RESEARCH A3): `Přislíbeno`/`Dobrovolné`/`Placené` aus
> bestehendem Wortschatz (`CategoryVolunteerWork="Dobrovolnictví"`, `ShowPaid="Placené"`).
> User-Review empfohlen.

**Bestehende cs.rs-Lücken, die diese Phase MIT schließt (verifiziert: in cs.rs ABwesend):**
```rust
i18n.add_text(Locale::Cs, Key::Volunteer, "Dobrovolné");
i18n.add_text(Locale::Cs, Key::PaidVolunteer, "Placené / Dobrovolné");
```
> `Key::Volunteer` (de.rs:572 / en.rs:517 vorhanden) und `Key::PaidVolunteer`
> (de.rs:133 / en.rs:105 vorhanden) fehlen in cs.rs.

### Per-Locale-Reference-Matcher-Tests (Pitfall 5 — `Locale::De`-statt-`Locale::En`)
**Source-Analog:** `i18n_absence_keys_match_{german,english,czech}_reference`
(`i18n/mod.rs:778-809`).
**Apply to:** jede neue Key/Locale-Kombination.
```rust
#[test]
fn i18n_committed_keys_match_german_reference() {
    let i18n = generate(Locale::De);   // ← guard: De, nicht En
    assert_eq!(i18n.t(Key::Committed).as_ref(), "Zugesagt");
    assert_eq!(i18n.t(Key::PaidCommittedVolunteer).as_ref(), "Bezahlt / Zugesagt / Freiwillig");
}
// + _english_reference (generate(Locale::En)) + _czech_reference (generate(Locale::Cs))
// + Assertion, dass Key::Volunteer/Key::PaidVolunteer in cs.rs nicht leer sind.
```

### Stunden-Formatierung
**Source:** `base_types::format_hours(value, decimals)` (bereits importiert in beiden
Render-Dateien). **Apply to:** committed-Token (2 Dezimal in Tabelle, 1 Dezimal im
Chart-Tooltip). **Keine neue Format-Funktion bauen.**

### Token-basierte Chart-Farben (kein Roh-Hex)
**Source:** bestehende Segment-Styles (chart Z.79/83/87 — `var(--accent)`,
`var(--ink-muted)`, `var(--bad)`). **Apply to:** committed-Segment = `var(--good)`.
**Gepinnt durch** `chart_uses_token_styles_not_legacy_hex` + `page_source_does_not_use_legacy_classes`.

---

## No Analog Found

Keine. Jede Ziel-Datei trägt ihren eigenen, exakten Analog (`volunteer_hours`/Band-2-
bzw. `paid_hours`-Behandlung). RESEARCH.md-Code-Beispiele werden nicht benötigt — alle
Muster existieren live im Codebase.

---

## Metadata

**Analog search scope:** `service/src/`, `service_impl/src/`, `rest-types/src/`,
`shifty-dioxus/src/{state,page,component,i18n}/`
**Files scanned:** 7 (alle Ziel-Dateien direkt gelesen; i18n via Grep-Audit der drei Locales)
**Pattern extraction date:** 2026-06-24
**Verifikations-Status:** alle Zeilennummern am HEAD direkt verifiziert (Read/Grep),
deckungsgleich mit RESEARCH.md-Integrations-Tabellen.
