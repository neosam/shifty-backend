# Phase 16: Jahresansicht display — Research

**Recherchiert:** 2026-06-24
**Domain:** Transport-Layer-Erweiterung (WeeklySummaryTO, frontend WeeklySummary, From-Mapping) + Frontend-Rendering (dritter Token, drittes Chart-Segment) + i18n De/En/Cs
**Confidence:** HIGH (alle kritischen Code-Pfade direkt am HEAD verifiziert)

---

<user_constraints>
## User Constraints (aus CONTEXT.md)

### Locked Decisions

- **D-01 (committed in overall_available_hours — Backend):** `overall_available_hours = paid_hours + committed_voluntary_hours + volunteer_hours` im Backend in `get_weekly_summary` (`booking_information.rs`). NICHT erst im Frontend. Kein Snapshot-Bump (WeeklySummary nicht persistiert; bleibt Version 7).
- **D-02 (drei getrennte Tokens):** Tabelle zeigt `💰paid | 🎯committed | 🤝surplus`. `volunteer_hours` (🤝) zeigt nur den Surplus über der Zusage (Band 2). Neuer „zugesagt"-Token zeigt `committed_voluntary_hours` (Band 1). KEIN kombinierter Inline-„5+2"-Token.
- **D-03 (0 zeigen — KEINE blank/Strich-Sonderlogik):** `committed_voluntary_hours == 0` → normal `0.00` anzeigen, konsistent mit paid/volunteer. Revidiert CVC-07 SC#2 und REQUIREMENTS-Formulierung „blank/Strich, nicht 0" — diese gilt für Phase 16 NICHT. Blank/Strich nur falls überhaupt in Phase 17 (Mitarbeiteransicht).
- **D-04 (drei gestapelte Farb-Segmente — CVC-F-02 in Phase 16 vorgezogen):** Chart-Balken: `paid + committed + surplus`. Tooltip nennt alle drei. Required-Linie bleibt. CVC-F-02 gilt als in Phase 16 erledigt.

### Claude's Discretion

- Exaktes Token-Emoji/Icon für „zugesagt" (`🎯` Vorschlag) und Spalten-/Header-Anordnung.
- Konkrete Farbwahl für das committed-Chart-Segment (Token-basiert, KEINE Hardcoded-Hex).
- Exakte i18n-Label-Texte (z. B. „Zugesagt"/„Committed"/„Přislíbeno") und ob `PaidVolunteer`-Key umbenannt/erweitert wird oder ein neuer Key hinzukommt.
- Test-Platzierung (SSR-Render-Test, Chart-Segment-Test, From-Mapping-Test, Per-Locale-Matcher).

### Deferred Ideas (AUSSERHALB SCOPE)

- Editor-Input (`contract_modal.rs`) für `committed_voluntary` + „alle"-Filter Mitarbeiteransicht + unpaid-volunteer-Record + `is_paid`-Gating → **Phase 17**.
- Blank/Strich-Darstellung statt „0" → falls überhaupt, dann **Phase 17** (Mitarbeiteransicht).
- Inline-Banner „Zusage nicht erfüllt" → **v1.5 (CVC-F-01)**.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Beschreibung | Research-Deckung |
|----|-------------|------------------|
| CVC-07 | `weekly_overview` zeigt committed-Kapazität separat (eigener Token, Überschuss sichtbar); ⚠ D-03 revidiert „blank/Strich" zu „0.00" | D-01 (Backend overall), D-02 (drei Token), D-03 (0-Regel); Code-Tracing belegt korrekter Pfad via erste `get_weekly_summary`-Variante |
| CVC-08 | Alle neuen benutzersichtbaren Strings in De/En/Cs vollständig; kein `Locale::En`-statt-`Locale::De`-Bug; Per-Locale-Reference-Matcher | i18n-Lücken-Audit verifiziert (s. Abschnitt i18n-Lücken); Pattern aus Phase 8.4 wiederverwendbar |
</phase_requirements>

---

## Zusammenfassung

Phase 16 führt den in Phase 15 berechneten dritten Term `committed_voluntary_hours` (Band 1 „zugesagt") durch drei Schichten: (1) `WeeklySummaryTO` + `From<&WeeklySummary>` im Backend-`rest-types`-Crate, (2) Frontend-`WeeklySummary`-Struct + `From<&WeeklySummaryTO>` in `state/weekly_overview.rs`, (3) Tabelle + Chart + i18n in `page/weekly_overview.rs` und `component/weekly_overview_chart.rs`.

Der kritische Research-Flag aus CONTEXT.md ist eindeutig geklärt: Die Jahresansicht nutzt die **erste** `get_weekly_summary`-Implementierung (ab Zeile 136 der `impl`, Trait-Methode `get_weekly_summary(year, ...)`) — diese ist seit Phase 15 vollständig gewired (Band 1 + Band 2). Die `overall_available_hours`-Zeile in dieser ersten Variante (Zeile ~273: `= volunteer_hours + paid_hours`) muss für D-01 um `committed_voluntary_hours` erweitert werden. Die zweite Variante `get_summery_for_week` bedient den Einzel-Wochen-Pfad (`weekly-resource-report/year/{week}`) und ist für Phase 16 beim committed-Feld absichtlich als Placeholder gelassen (Kommentar Zeile 544 bestätigt: year-view-only).

Snapshot-Versioning: VERIFIZIERT kein Bump nötig — `WeeklySummary` ist year-view-only und wird von `billing_period_report.rs` nie konsumiert. `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7` bleibt.

**Hauptempfehlung:** Drei sequenzielle Waves: (A) Backend: `overall_available_hours` + `WeeklySummaryTO`-Erweiterung; (B) Frontend-State: `From<&WeeklySummaryTO>` + `WeeklySummary`-Struct; (C) Frontend-Render: dritter Token + drittes Chart-Segment + i18n. WASM-Build-Gate nach jeder Wave.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| committed in overall_available_hours | API/Backend (`booking_information.rs`) | — | D-01: Berechnung im Backend, damit Diff-Spalte + Chart konsistent dieselbe Zahl sehen |
| committed via Transport-Layer | REST-Types (`WeeklySummaryTO`) | — | Single Source of Truth; Frontend-WASM-Build bricht, wenn Feld fehlt |
| committed im Frontend-State | Frontend State (`state/weekly_overview.rs`) | — | From-Mapping-Boundary; rustc erzwingt Vollständigkeit |
| dritter Token (Tabelle) | Frontend Page (`page/weekly_overview.rs`) | — | UI-Rendering; SSR-Tests pinnen |
| drittes Chart-Segment | Frontend Component (`component/weekly_overview_chart.rs`) | — | CVC-F-02 (per D-04 in Phase 16 vorgezogen) |
| i18n De/En/Cs | Frontend i18n (`i18n/{de,en,cs}.rs`) | `i18n/mod.rs` (Key-Enum) | Konvention: alle drei Locales gleichzeitig pflegen |

---

## Standard Stack

### Core (alles bestehend — keine neuen Dependencies)

| Crate / Datei | Zweck | Warum relevant |
|-------------|-------|----------------|
| `service/src/booking_information.rs` | `WeeklySummary`-Service-Struct | Trägt bereits `committed_voluntary_hours: f32` (Phase 15) |
| `rest-types/src/lib.rs` | `WeeklySummaryTO` + `From<&WeeklySummary>` | Fehlt committed-Feld + Mapping — Phase 16 ergänzt |
| `shifty-dioxus/src/state/weekly_overview.rs` | Frontend-`WeeklySummary` + `From<&WeeklySummaryTO>` | Fehlt committed-Feld + Mapping — Phase 16 ergänzt |
| `shifty-dioxus/src/page/weekly_overview.rs` | Token-Rendering, Diff-Berechnung, SSR-Tests | Zwei Stellen: Desktop (Z. 103) + Mobile (Z. 108) anpassen |
| `shifty-dioxus/src/component/weekly_overview_chart.rs` | Chart-Segmente, bar_total, Tooltip, Legend | `bar_total = paid + volunteer` → um committed erweitern |
| `shifty-dioxus/src/i18n/mod.rs` | Key-Enum | Neuer Key für „zugesagt"-Label + Chart-Label |
| `shifty-dioxus/src/i18n/{de,en,cs}.rs` | Übersetzungen | Alle drei Locales gleichzeitig — kein `Locale::En`-statt-`Locale::De`-Bug |
| `base_types::format_hours` | Stunden-Formatierung | Für committed-Token wiederverwenden (keine neue Funktion) |

---

## Kritischer Research-Flag: Zwei `get_weekly_summary`-Varianten — EINDEUTIG GEKLÄRT

### Call-Pfad für die Jahresansicht (verifiziert)

```
Frontend page/weekly_overview.rs
  → WeeklySummaryAction::LoadYear(year)
  → service/weekly_summary.rs::load_weekly_summary_year()
  → loader::load_weekly_summary_for_year()
  → api::get_weekly_overview(config, year)
  → GET /booking-information/weekly-resource-report/{year}
  → rest/src/booking_information.rs::get_weekly_summary()  [REST-Handler]
  → booking_information_service().get_weekly_summary(year, ...)  [Trait-Aufruf]
  → service_impl/src/booking_information.rs  ERSTE VARIANTE (Zeile 136-295)
```

### Erste Variante (`get_weekly_summary`, Zeile 136–295 in service_impl) — speist die Jahresansicht

- **Trait-Methode:** `async fn get_weekly_summary(&self, year: u32, ...) -> Result<Arc<[WeeklySummary]>, ServiceError>`
- **Status:** Seit Phase 15 vollständig gewired — `committed_voluntary_hours` (Band 1) korrekt berechnet (Zeilen 219-226), `volunteer_hours` als per-person-surplus Band 2 (Zeilen 207-216).
- **D-01 TODO:** `overall_available_hours = volunteer_hours + paid_hours` (Zeile 273) — muss auf `= committed_voluntary_hours + volunteer_hours + paid_hours` erweitert werden.

### Zweite Variante (`get_summery_for_week`, Zeile 297–562 in service_impl) — speist den Einzel-Wochen-Pfad

- **Trait-Methode:** `async fn get_summery_for_week(&self, year: u32, week: u8, ...) -> Result<WeeklySummary, ServiceError>`
- **Endpoint:** `GET /booking-information/weekly-resource-report/year/{week}`
- **Status:** `committed_voluntary_hours: 0.0` als Placeholder (Zeile 547), Kommentar explizit: „year-view-only". Diese Variante bedient NICHT den Jahresansicht-Pfad.
- **Phase-16-Scope:** `overall_available_hours` (Zeile 386) ebenfalls `volunteer + paid` — gemäß D-01-Kommentar in CONTEXT `booking_information.rs:104-106` ist diese Zeile für Phase 16 nicht im Scope für committed (der Einzel-Wochen-View ist nicht die Jahresansicht). Keine Änderung nötig.

**Konklusion:** Phase 16 ändert in der Backend-Impl NUR Zeile 273 (erste Variante, `overall_available_hours`). Zweite Variante bleibt unberührt.

---

## Architecture Patterns

### System-Datenfluss (Phase 16 — neue Elemente fett)

```
booking_information.rs::get_weekly_summary (Zeile 136–295)
  Band 1: committed_voluntary_hours (Zeile 219–226) ✓ Phase 15 gewired
  Band 2: volunteer_hours (surplus) (Zeile 207–216) ✓ Phase 15 gewired
  **overall_available_hours = committed + volunteer + paid** ← D-01 (Zeile 273)
        ↓ WeeklySummary (service-Struct)
  **WeeklySummaryTO.committed_voluntary_hours** ← neu + From<&WeeklySummary>
        ↓ JSON via GET /booking-information/weekly-resource-report/{year}
  **state::weekly_overview::WeeklySummary.committed_voluntary_hours** ← neu + From<&WeeklySummaryTO>
        ↓
  page/weekly_overview.rs
    WeeklyOverviewTable:
      Desktop (Z.103): 💰paid | **🎯committed** | 🤝surplus
      Mobile (Z.108):  💰paid | **🎯committed** | 🤝surplus
    Diff-Spalte (Z.87): available_hours − required_hours  (no change, available_hours kommt from state)
        ↓
  component/weekly_overview_chart.rs
    **bar_total = paid + committed + surplus** ← D-04
    Drei gestapelte Segmente: paid / **committed** / surplus
    Tooltip: alle drei Werte
    **i18n-Key::Committed** (neuer Key) ← Chart-Legend
        ↓
  i18n/mod.rs Key-Enum: **Key::Committed** + ggf. Key::PaidVolunteerCommitted
  i18n/{de,en,cs}.rs: alle drei Locales + Per-Locale-Matcher-Tests
```

### Empfohlene Wave-Reihenfolge

```
Wave 1 (Backend+Transport):
  service_impl/src/booking_information.rs  Zeile 273 (overall_available_hours)
  rest-types/src/lib.rs                   WeeklySummaryTO + From<&WeeklySummary>

Wave 2 (Frontend-State):
  shifty-dioxus/src/state/weekly_overview.rs  WeeklySummary-Struct + From<&WeeklySummaryTO>
  + WASM-Build-Gate (kompiliert erst wenn TO + State synchron)

Wave 3 (Frontend-Render + i18n + Tests):
  i18n/mod.rs Key-Enum  (zuerst — Compiler-Gate)
  i18n/{de,en,cs}.rs
  page/weekly_overview.rs  (Token-Rendering, sample_week-Signatur, SSR-Tests)
  component/weekly_overview_chart.rs  (bar_total, Segmente, Tooltip, Tests)
  + WASM-Build-Gate
```

---

## Don't Hand-Roll

| Problem | Nicht selbst bauen | Stattdessen | Warum |
|---------|-------------------|-------------|-------|
| Stunden-Formatierung | Eigene Format-Funktion | `base_types::format_hours(value, decimals)` | Bestehende Funktion, konsistentes Format |
| Token-Farben im Chart | Hardcoded Hex (`#3B82F6` o. ä.) | CSS-Custom-Properties (`var(--accent)`, `var(--ink-muted)`, `var(--bad)`) | Test `chart_uses_token_styles_not_legacy_hex` pinnt das Verbot von Roh-Hex im Prod-Source |
| Per-Locale-Test-Pattern | Eigene Test-Logik | Pattern aus Phase 8.4: `i18n_*_match_{german,english,czech}_reference` | Fängt `Locale::En`-statt-`Locale::De`-Bug; etabliertes Muster |
| SSR-Render-Tests | Eigenes Test-Harness | Bestehendes `WeeklyOverviewTable`-Pattern mit `render_table()` + `VirtualDom` + `dioxus_ssr::render` | Einfach erweiterbar; `sample_week`-Signatur anpassen |

---

## Spezifische Integrations-Stellen (Code-Level verifiziert)

### Backend: `service_impl/src/booking_information.rs`

| Zeile | Was | Phase-16-Aktion |
|-------|-----|----------------|
| 273 | `let overall_available_hours = volunteer_hours + paid_hours;` | `= committed_voluntary_hours + volunteer_hours + paid_hours` (D-01) |
| 274–281 | `WeeklySummary { ..., overall_available_hours, ..., committed_voluntary_hours, ... }` | Bereits korrekt befüllt (Phase 15) — keine Änderung |
| 386 | `let overall_available_hours = volunteer_hours + paid_hours;` (zweite Variante) | KEIN Change (year-view-only, Placeholder korrekt laut Kommentar) |
| 547 | `committed_voluntary_hours: 0.0,` (zweite Variante) | KEIN Change (Einzel-Wochen-Variante, nicht Jahresansicht) |

### Transport: `rest-types/src/lib.rs`

| Zeile | Was | Phase-16-Aktion |
|-------|-----|----------------|
| 904–920 | `WeeklySummaryTO` struct | Feld hinzufügen: `pub committed_voluntary_hours: f32` + `#[serde(default)]` für Wire-Backward-Compat |
| 922–948 | `impl From<&WeeklySummary> for WeeklySummaryTO` | Mapping-Arm: `committed_voluntary_hours: weekly_summary.committed_voluntary_hours` |

**Hinweis:** `WeeklySummaryTO` hat derzeit kein `#[derive(ToSchema)]` und keinen `#[utoipa::path]`-Handler — keine OpenAPI-Änderung nötig (analog zu `EmployeeWorkDetailsTO` out-of-scope, per REQUIREMENTS). [VERIFIED: `rest/src/booking_information.rs` hat kein `utoipa`-Attribut]

### Frontend State: `shifty-dioxus/src/state/weekly_overview.rs`

| Zeile | Was | Phase-16-Aktion |
|-------|-----|----------------|
| 12–27 | `WeeklySummary`-Struct | Feld hinzufügen: `pub committed_voluntary_hours: f32` |
| 29–63 | `impl From<&WeeklySummaryTO> for WeeklySummary` | Mapping-Arm: `committed_voluntary_hours: summary.committed_voluntary_hours` |
| 65–72 | Hilfsmethoden `monday_date()`, `sunday_date()` | Keine Änderung |

**Pitfall:** Wenn `committed_voluntary_hours` im `From`-Mapping vergessen wird, zeigt die UI 0, obwohl der Backend-Wert korrekt ist. Rustc fängt nur Missing-Fields im Struct-Literal, nicht im `From`-Pattern.

### Frontend Page: `shifty-dioxus/src/page/weekly_overview.rs`

| Zeile | Was | Phase-16-Aktion |
|-------|-----|----------------|
| 87 | `let diff = week.available_hours - week.required_hours;` | Keine Änderung (available_hours kommt aus State = `overall_available_hours` vom Backend, D-01 berechnet es korrekt) |
| 103 | Desktop-Token: `format!("💰{} | 🤝{}", paid, volunteer)` | → `format!("💰{} | 🎯{} | 🤝{}", paid, committed, surplus)` |
| 108 | Mobile-Token: identischer format!-String | → identisch anpassen |
| 218–235 | `sample_week(year, week, paid, volunteer, required) -> WeeklySummary` | Signatur um `committed: f32` erweitern; `available_hours: paid + committed + volunteer` |
| SSR-Tests | 11 bestehende SSR-Tests | `sample_week`-Aufrufe anpassen; neuen Test für dritten Token hinzufügen |

### Frontend Chart: `shifty-dioxus/src/component/weekly_overview_chart.rs`

| Zeile | Was | Phase-16-Aktion |
|-------|-----|----------------|
| 12–29 | `compute_max_hours`: `bar_total = paid + volunteer` | → `bar_total = paid + committed + volunteer` (oder `available_hours` direkt nutzen) |
| 47–52 | `WeeklyOverviewChart`-Wrapper: Legende mit `paid_label`, `volunteer_label` | `committed_label` Parameter hinzufügen |
| 56–63 | `WeeklyOverviewChartView`-Props | `committed_label: String` hinzufügen |
| 99–101 | `paid_pct`, `vol_pct` | `committed_pct` hinzufügen |
| 107 | Tooltip: zwei Werte | Drei Werte inkl. committed |
| 122–128 | Zwei gestapelte Divs (volunteer, paid) | Drittes Div für committed einfügen (zwischen paid und volunteer) |
| 78–90 | HTML-Legende: zwei Einträge | Dritten Legendeneintrag für committed hinzufügen |
| Test (Z. 247) | `chart_uses_token_styles_not_legacy_hex` | Erweiterung: committed-Segment muss ebenfalls Token-Farbe verwenden (kein Roh-Hex) |

**Chart-Farb-Empfehlung (Claude's Discretion):** Für committed-Segment eine neue CSS-Custom-Property wie `var(--accent-warm)` oder `var(--warn)` verwenden, falls im Design-System vorhanden. Alternativ `var(--accent)` mit reduzierter Opazität (0.6) als Mittelschicht zwischen paid (volle Opazität) und volunteer (0.35 Opazität) — konsistent mit bestehenden Segment-Opazitäten.

### i18n: Neue Keys + bestehende Lücken

#### i18n-Lücken-Audit (verifiziert am HEAD)

| Key | de.rs | en.rs | cs.rs | Status |
|-----|-------|-------|-------|--------|
| `Key::Volunteer` | Z.572 ✓ | Z.517 ✓ | **fehlt** | Bestehende Lücke — Phase 16 schließt |
| `Key::PaidVolunteer` | Z.133 ✓ | Z.105 ✓ | **fehlt** | Bestehende Lücke — Phase 16 schließt oder Key umbenennen/erweitern |

**Phase-16-neue Keys (alle drei Locales):**

| Key (Vorschlag) | De | En | Cs |
|----------------|----|----|-----|
| `Key::Committed` | „Zugesagt" | „Committed" | „Přislíbeno" |
| `Key::PaidVolunteerCommitted` (Header-Spalte — wenn PaidVolunteer ersetzt wird) | „Bezahlt / Zugesagt / Freiwillig" | „Paid / Committed / Volunteer" | „Placené / Přislíbeno / Dobrovolné" |

**Entscheidung Header-Key (Claude's Discretion):** Entweder `Key::PaidVolunteer` → Wert ändern auf „Bezahlt / Zugesagt / Freiwillig" (einfacher, potenziell irreführend wenn der Key-Name bleibt), oder neuer `Key::PaidCommittedVolunteer`. Empfehlung: neuer Key, damit der Name selbstbeschreibend bleibt; `PaidVolunteer`-Key weiterhin aus anderen Stellen behalten falls dort referenziert.

---

## Snapshot-Versioning (verifiziert)

**`CURRENT_SNAPSHOT_SCHEMA_VERSION = 7` bleibt unverändert.**

Begründung: `WeeklySummary` ist year-view-only. Verifiziert in `service_impl/src/billing_period_report.rs`: `build_new_billing_period` und der `BillingPeriodValueType`-Enum konsumieren `ReportingService`-Outputs (Achse A), NICHT `BookingInformationService::WeeklySummary` (Achse B). Die `committed_voluntary_hours`-Erweiterung in `get_weekly_summary` berührt keine persistierten `BillingPeriodValueType`-Werte. Die CLAUDE.md-Bump-Regel ist nicht ausgelöst. [VERIFIED: `billing_period_report.rs:75` = `CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 7`; kein `WeeklySummary`-Konsum in diesem File] [ASSUMED: kein anderer Persistenz-Pfad konsumiert WeeklySummary — plausibel durch Architektur, aber nicht exhaustiv gegrept]

---

## Common Pitfalls

### Pitfall 1: `From<&WeeklySummaryTO>` im Frontend vergessen (Omission-Lücke)

**Was schiefläuft:** `committed_voluntary_hours` in `WeeklySummaryTO` vorhanden, aber in `impl From<&WeeklySummaryTO> for WeeklySummary` nicht gemappt → Frontend zeigt immer `0.00` für committed, obwohl Backend den korrekten Wert liefert.
**Warum:** Rustc erzwingt nur, dass alle Felder des Ziel-Struct im Literal vorkommen — nicht, dass sie korrekt befüllt sind. Ein `committed_voluntary_hours: Default::default()` würde kompilieren.
**Vermeidung:** `From`-Impl als erstes Prüfziel im Review; From-Mapping-Roundtrip-Test (s. Validation Architecture).
**Warnsignal:** Token `🎯0.00` für alle Wochen, obwohl Personen `committed_voluntary > 0` haben.

### Pitfall 2: `overall_available_hours` in der falschen Variante anpassen

**Was schiefläuft:** Entwickler findet die zweite Variante (`get_summery_for_week`, Zeile 386) und ändert dort, vergisst die erste (Zeile 273) — oder umgekehrt.
**Warum:** Zwei ähnliche Implementierungen desselben Traits, unterschiedliche Methoden-Namen, gleiche Variablen-Namen.
**Vermeidung:** Research-Flag ist eindeutig geklärt — NUR Zeile 273 (erste Variante) für D-01. Plan-Executor muss dies explizit dokumentieren.
**Warnsignal:** Jahresansicht zeigt committed nicht in Diff-Spalte; Chart-Höhe weicht von Diff-Spalte ab.

### Pitfall 3: `bar_total` im Chart nicht erweitern

**Was schiefläuft:** Token-Rendering in der Tabelle zeigt drei Werte korrekt, aber Chart-Balken zeigt nur `paid + volunteer` → visueller Widerspruch zu Diff-Spalte (D-04).
**Warum:** `compute_max_hours` liest `w.paid_hours + w.volunteer_hours` — zwei unabhängige Stellen.
**Vermeidung:** `compute_max_hours` und Segment-Berechnung gemeinsam anpassen; alternativ `available_hours`-Feld aus State verwenden (enthält nach D-01 alle drei).
**Warnsignal:** Chart-Balken niedriger als Diff-Spalte für Wochen mit committed > 0.

### Pitfall 4: Hardcoded Hex im Chart-Prod-Source

**Was schiefläuft:** Committed-Segment mit Hex-Farbe (`#RRGGBB`) — Test `page_source_does_not_use_legacy_classes` schlägt fehl.
**Warum:** Test `include_str!("../component/weekly_overview_chart.rs")` und sucht nach `#3B82F6`, `#10B981` etc. im Prod-Source (nicht im Test-Code).
**Vermeidung:** Ausschließlich CSS-Custom-Properties verwenden. Die Liste der verbotenen Hexes steht in `weekly_overview.rs:584`.
**Warnsignal:** `page_source_does_not_use_legacy_classes` schlägt fehl.

### Pitfall 5: `Locale::De`-statt-`Locale::En`-Bug (oder umgekehrt) in i18n-Dateien

**Was schiefläuft:** `add_text(Locale::En, Key::Committed, "Zugesagt")` in `en.rs` — englischer Locale zeigt deutschen Text.
**Warum:** Bekannte Falle im Projekt (aus STATE.md Accumulated Context, Plan 08-04 Pattern).
**Vermeidung:** Per-Locale-Reference-Matcher-Tests (`i18n_committed_match_german_reference`, `_english_reference`, `_czech_reference`) — stichprobenartig die erwarteten Strings gegen `generate(Locale::De)` etc. gegenchecken.
**Warnsignal:** Per-Locale-Reference-Matcher-Test schlägt fehl.

### Pitfall 6: `sample_week`-Signatur-Inkonsistenz bricht SSR-Tests

**Was schiefläuft:** `sample_week` in `page/weekly_overview.rs:218` und in `component/weekly_overview_chart.rs:153` hat beide eine separate, lokale Definition. Wenn nur eine angepasst wird, kompilieren Tests mit falschen Werten.
**Warum:** Zwei lokale `fn sample_week`-Definitionen, nicht DRY.
**Vermeidung:** Beide `sample_week`-Definitionen gleichzeitig um `committed: f32` erweitern; `available_hours = paid + committed + surplus` in beiden.

### Pitfall 7: `WeeklySummaryTO` in `rest-types` ohne `#[serde(default)]` auf dem neuen Feld

**Was schiefläuft:** Ältere API-Responses (ohne das Feld) deserialisieren mit JSON-Fehler statt Default-Wert.
**Warum:** `rest-types` ist shared crate; Wire-Backward-Compat ist Pflicht.
**Vermeidung:** `#[serde(default)]` auf `committed_voluntary_hours: f32` in `WeeklySummaryTO` setzen. Pattern aus Phase 14 für `EmployeeWorkDetailsTO`.

---

## Validation Architecture

Nyquist-Validation ist aktiviert (kein explizites `false` in `.planning/config.json`).

### Test-Framework

| Eigenschaft | Wert |
|-------------|------|
| Framework | `cargo test` (Rust built-in, Backend + Frontend getrennt) |
| Frontend-Test-Crate | `shifty-dioxus/` — `cargo test` + `dioxus-ssr` für SSR-Render-Tests |
| Backend-Test-Crate | Workspace-Root — `cargo test --workspace` |
| Schneller Run (Frontend) | `cargo test -p shifty-dioxus` |
| Schneller Run (Backend) | `cargo test -p service_impl` |
| Vollständige Suite | `cargo test --workspace` (Backend) + `cargo test` in `shifty-dioxus/` |
| WASM-Build-Gate | `cargo build --target wasm32-unknown-unknown` (in `shifty-dioxus/`, unter `nix develop`) |

### Phase-Requirements → Test-Map

| Req-ID | Verhalten | Test-Typ | Automatischer Befehl | Datei vorhanden? |
|--------|-----------|----------|--------------------|-----------------|
| CVC-07 (a) | `overall_available_hours = paid + committed + surplus` nach D-01 | unit | `cargo test -p service_impl booking_information` | ✅ (bestehende Tests; neue Tests nötig) |
| CVC-07 (b) | `WeeklySummaryTO` trägt `committed_voluntary_hours` | unit (From-Roundtrip) | `cargo test -p rest-types` (oder `service_impl`) | ❌ Wave 0 |
| CVC-07 (c) | Frontend `From<&WeeklySummaryTO>` mappt committed korrekt | unit | `cargo test -p shifty-dioxus` | ❌ Wave 0 |
| CVC-07 (d) | Dritter Token `🎯committed` in Tabelle (Desktop + Mobile) | SSR-Render | `cargo test -p shifty-dioxus` | ❌ Wave 0 |
| CVC-07 (e) | Surplus-Anzeige: committed=5, actual=7 → `🎯5.00 | 🤝2.00` | SSR-Render | `cargo test -p shifty-dioxus` | ❌ Wave 0 |
| CVC-07 (f) | committed=0 zeigt `0.00` (D-03: keine blank-Logik) | SSR-Render | `cargo test -p shifty-dioxus` | ❌ Wave 0 |
| CVC-07 (g) | Chart: drei Segmente, kein Roh-Hex in Prod-Source | SSR + Source-Audit | `cargo test -p shifty-dioxus` | ✅ (bestehend erweitert) |
| CVC-07 (h) | `bar_total` im Chart = paid + committed + surplus | unit | `cargo test -p shifty-dioxus` | ❌ Wave 0 |
| CVC-08 (a) | Neuer Key in allen 3 Locales vorhanden | unit | `cargo test -p shifty-dioxus` | ❌ Wave 0 |
| CVC-08 (b) | Per-Locale-Reference-Matcher für committed-Key (De/En/Cs) | unit | `cargo test -p shifty-dioxus` | ❌ Wave 0 |
| CVC-08 (c) | `Key::Volunteer` in cs.rs vorhanden (bestehende Lücke) | unit | `cargo test -p shifty-dioxus` | ❌ Wave 0 |
| CVC-08 (d) | `Key::PaidVolunteer` (oder Nachfolger) in cs.rs vorhanden | unit | `cargo test -p shifty-dioxus` | ❌ Wave 0 |

### Sampling-Rate

- **Pro Task-Commit:** `cargo test -p shifty-dioxus` (Frontend-Suite) oder `cargo test -p service_impl` (Backend-Suite)
- **Pro Wave-Merge:** `cargo test --workspace` + `cargo test` in `shifty-dioxus/` + WASM-Build-Gate
- **Phase-Gate:** Vollständige Suite grün vor `/gsd-verify-work`

### Wave-0-Lücken

- [ ] `shifty-dioxus/src/state/weekly_overview.rs` — From-Mapping-Roundtrip-Test (`committed_voluntary_hours` wird korrekt gemappt)
- [ ] `rest-types/src/lib.rs` — From-Mapping-Test (`WeeklySummaryTO::from(&weekly_summary).committed_voluntary_hours == weekly_summary.committed_voluntary_hours`)
- [ ] `shifty-dioxus/src/page/weekly_overview.rs` — SSR-Test: dritter Token sichtbar; committed=5,actual=7 → `🎯5.00 | 🤝2.00`; committed=0 → `🎯0.00`
- [ ] `shifty-dioxus/src/component/weekly_overview_chart.rs` — Test: `compute_max_hours` nutzt `paid + committed + volunteer`; drittes Segment kein Roh-Hex
- [ ] `shifty-dioxus/src/i18n/{de,en,cs}.rs` — Per-Locale-Reference-Matcher-Tests für neue Keys; `Key::Volunteer` und `Key::PaidVolunteer`-Äquivalent in cs.rs

---

## Project Constraints (aus CLAUDE.md)

- **VCS:** Repository wird mit `jj` (co-located mit git) verwaltet — Commits IMMER manuell durch User (niemals aus Agent heraus). GSD-Auto-Commit deaktiviert.
- **NixOS:** `nix develop` (NICHT `nix-shell`) für WASM-Toolchain. WASM-Build-Gate: `cargo build --target wasm32-unknown-unknown` in `shifty-dioxus/`.
- **i18n-Pflicht:** Alle benutzersichtbaren Texte in allen drei Locales (De/En/Cs) gleichzeitig. Kein Locale::En-statt-Locale::De-Bug.
- **Layered Architecture:** REST → Service (trait) → DAO — keine Schichtverletzungen.
- **Service-Tier-Konvention:** `BookingInformationServiceImpl` ist ein Business-Logic-Service (konsumiert `ReportingService`). Keine neuen DI-Dependencies nötig (Phase 16 ist rein display-seitig).
- **Snapshot-Versioning:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 7 (verifiziert — WeeklySummary nicht persistiert).
- **Token-Farben:** Keine Hardcoded-Hex-Werte im Prod-Source des Chart. Verbotene Hexes: `#3B82F6`, `#10B981`, `#EF4444`, `#e5e7eb`, `#6b7280`, `#374151` (aus bestehendem Test `page_source_does_not_use_legacy_classes`).
- **`cargo test` grün:** Backend + Frontend vor jedem Commit.
- **Kein OpenAPI-Task:** `WeeklySummaryTO` hat keine `#[utoipa::path]`/`ToSchema`-Anbindung; keine OpenAPI-Änderung nötig.

---

## Assumptions Log

| # | Claim | Abschnitt | Risiko wenn falsch |
|---|-------|-----------|-------------------|
| A1 | Kein anderer Persistenz-Pfad konsumiert `WeeklySummary` — Snapshot-Versioning nicht ausgelöst | Snapshot-Versioning | Sehr gering: Architektur-Dokumentation + `billing_period_report.rs`-Read bestätigen Achse-B-only; wäre wenn falsch ein falsches Snapshot ohne Bump |
| A2 | `WeeklySummaryTO` hat kein `#[derive(ToSchema)]` / OpenAPI-Anbindung | Integrations-Stellen (Transport) | Sehr gering: direkt in `rest/src/booking_information.rs` verifiziert — kein `utoipa`-Attribut |
| A3 | Suggested i18n-Übersetzungen für Cs (Přislíbeno, Dobrovolné) sind korrekt | i18n-Lücken | Mittel: Czech-Übersetzungen werden vom Planner/Executor aus Wortschatz-Kontext abgeleitet; User sollte prüfen |
| A4 | Die zweite Variante (`get_summery_for_week`) braucht keinen D-01-Einzel-Wochen-Change | Kritischer Research-Flag | Sehr gering: Kommentar in Zeile 544 explizit; CONTEXT.md D-01 bezieht sich auf Jahresansicht |

---

## Open Questions

1. **i18n Header-Key: Umbenennen oder neuer Key?**
   - Was wir wissen: `Key::PaidVolunteer` wird in `page/weekly_overview.rs:58` für den Tabellen-Header verwendet; der Header heißt aktuell „Bezahlt / Freiwillig".
   - Unklar: Ob ein neuer Key (`Key::PaidCommittedVolunteer`) oder der bestehende Key mit neuem Inhalt bevorzugt wird (Auswirkung auf andere potenzielle Konsumenten des Keys).
   - Empfehlung: Neuer Key `Key::PaidCommittedVolunteer` — klarer Name, kein Bruch bestehender Konsumenten.

2. **Chart: `available_hours` als `bar_total` oder Felder summieren?**
   - Was wir wissen: `week.available_hours` in State enthält nach D-01 bereits `paid + committed + surplus`. Alternativ explizit `week.paid_hours + week.committed_voluntary_hours + week.volunteer_hours` summieren.
   - Unklar: Welche Variante für zukünftige Wartbarkeit klarer ist.
   - Empfehlung: Explizite Felder summieren — macht die Abhängigkeit im Chart-Code transparent und vermeidet, dass `available_hours` sich durch zukünftige Änderungen anders verhält.

---

## Environment Availability

Step 2.6: SKIPPED (Phase 16 ist rein code-seitig — keine neuen externen Dependencies. Bestehende Tools: `cargo`, `nix develop`, `dx` sind durch NixOS flake.nix gepinnt und bekannt verfügbar.)

---

## Security Domain

> Phase 16 berührt keine Authentifizierungs-, Session-, Access-Control- oder Kryptografie-Logik. Alle REST-Handler sind bereits durch bestehende Permission-Checks abgesichert (`shiftplanner.or(sales)?` in beiden `get_weekly_summary`-Varianten). Keine neuen Endpoints, keine neuen Datenbank-Schreibpfade, keine neuen Secrets. ASVS-Kategorien V2/V3/V4/V6 — nicht anwendbar für diese Phase. V5 (Input Validation) — nicht relevant (read-only Display-Erweiterung). Sicherheitsrelevante Bedenken: keine.

---

## Quellen

### Primär (HIGH confidence — direkte Code-Reads am HEAD)

- `service_impl/src/booking_information.rs:136-295` — erste `get_weekly_summary`-Variante (vollständig gewired Phase 15); `overrall_available_hours`-Zeile 273
- `service_impl/src/booking_information.rs:297-562` — zweite `get_summery_for_week`-Variante; Placeholder-Zeile 547
- `service/src/booking_information.rs:37-56` — `WeeklySummary`-Struct mit `committed_voluntary_hours: f32` (Band 1) + `volunteer_hours` (Band 2)
- `rest/src/booking_information.rs:14-28,58-79` — Route-Map + REST-Handler `get_weekly_summary` → Trait-Aufruf erster Variante
- `rest-types/src/lib.rs:904-948` — `WeeklySummaryTO` (fehlt committed-Feld) + `From<&WeeklySummary>` (fehlt Mapping)
- `shifty-dioxus/src/api.rs:915-927` — `get_weekly_overview` → `GET /booking-information/weekly-resource-report/{year}`
- `shifty-dioxus/src/loader.rs:611-621` — `load_weekly_summary_for_year` → `api::get_weekly_overview` → `WeeklySummary::from`
- `shifty-dioxus/src/service/weekly_summary.rs:29-35` — `WeeklySummaryAction::LoadYear` → `load_weekly_summary_year`
- `shifty-dioxus/src/state/weekly_overview.rs:1-72` — Frontend-`WeeklySummary`-Struct (fehlt committed) + `From<&WeeklySummaryTO>` (fehlt Mapping)
- `shifty-dioxus/src/page/weekly_overview.rs:87-132,218-235` — Token-Rendering (Z.103/108), Diff (Z.87), `sample_week`-Helper (Z.218)
- `shifty-dioxus/src/component/weekly_overview_chart.rs:12-29,96-128` — `compute_max_hours` + `bar_total`, Segmente
- `shifty-dioxus/src/i18n/mod.rs:118-372` — Key-Enum inkl. `Key::Paid`, `Key::Volunteer`, `Key::PaidVolunteer`, `Key::ChartRequiredHours`
- `shifty-dioxus/src/i18n/de.rs:133,571,572,625,628` — de-Locale Chart-Keys
- `shifty-dioxus/src/i18n/en.rs:105,517` — en-Locale
- `shifty-dioxus/src/i18n/cs.rs:127-135,539,597-600` — cs-Locale; `Key::Volunteer` und `Key::PaidVolunteer` **fehlen verifiziert**
- `service_impl/src/billing_period_report.rs:75` — `CURRENT_SNAPSHOT_SCHEMA_VERSION = 7`; kein WeeklySummary-Konsum
- `.planning/phases/16-jahresansicht-display/16-CONTEXT.md` — D-01..D-04, Research-Flag, Deferred
- `.planning/REQUIREMENTS.md` — CVC-07, CVC-08

### Sekundär (HIGH confidence — Projekt-Konventionen)

- `.planning/research/SUMMARY.md` — Achse A vs. Achse B; Achse B ist Year-View-Pfad (HIGH, direktverifiziert 2026-06-22)
- `.planning/research/PITFALLS.md` — P1 (Doppelzählung), P2 (Snapshot-Drift); relevant für Kontext
- `.planning/STATE.md` — Accumulated Context: Per-Locale-Reference-Matcher-Pattern (Phase 8.4), token-based chart colors, jj-only commits
- `shifty-backend/CLAUDE.md` — Snapshot-Versioning-Regeln, Service-Tier-Konvention
- `shifty-dioxus/CLAUDE.md` — i18n-Konventionen, Locale::De-Bug-Falle

---

## Metadata

**Confidence Breakdown:**

| Bereich | Level | Grund |
|---------|-------|-------|
| Call-Pfad-Tracing | HIGH | Direkte Code-Reads: REST-Handler → Trait-Methode → Impl-Variante vollständig verfolgt |
| Integrations-Stellen | HIGH | Zeilennummern direkt verifiziert am HEAD |
| i18n-Lücken | HIGH | Alle drei Locale-Dateien direkt gegrept |
| Snapshot-Versioning | HIGH | `billing_period_report.rs` direkt gelesen; Kommentar Zeile 544 verifiziert |
| Pitfalls | HIGH | Aus direktem Code-Read abgeleitet + bestehende Pitfall-Docs |
| i18n-Czech-Übersetzungen | MEDIUM | Übersetzungs-Vorschläge aus Kontext-Wissen; User-Review empfohlen |

**Research-Datum:** 2026-06-24
**Gültig bis:** ~2026-07-24 (stabiler Code, niedrige Drift-Rate für diese Layer)
