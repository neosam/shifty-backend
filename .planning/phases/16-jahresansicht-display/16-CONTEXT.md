# Phase 16: Jahresansicht display - Context

**Gathered:** 2026-06-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Den in Phase 15 berechneten dritten Term `committed_voluntary_hours` (Band 1 „zugesagt") durch die Transport-/Frontend-Layer führen und in der **Jahresansicht** (`weekly_overview`) sichtbar machen:
- `WeeklySummaryTO` (rest-types) + `From<&WeeklySummary>`-Mapping tragen `committed_voluntary_hours`.
- Frontend `state/weekly_overview.rs` (`WeeklySummary` + `From<&WeeklySummaryTO>`) trägt das Feld.
- Tabelle (`page/weekly_overview.rs`) rendert committed als **eigenen Token** („zugesagt"), getrennt von paid/volunteer; Überschuss (Surplus/Band 2) sichtbar.
- Chart (`component/weekly_overview_chart.rs`) bekommt ein **drittes gestapeltes Farb-Segment** (paid / committed / surplus).
- `overall_available_hours` wird im Backend um den committed-Term erweitert (Diff-Spalte + Chart konsistent).
- i18n De / En / Cs für alle neuen Strings, mit Per-Locale-Reference-Matcher-Tests.

**NICHT in dieser Phase (Phase 17):** Editor-Input für `committed_voluntary` (`contract_modal.rs`), „alle"-Filter in der Mitarbeiteransicht, unpaid-volunteer-`EmployeeWorkDetails`-Record + `is_paid`-Gating der paid-only-Sites.

**NICHT in dieser Phase (v1.5):** Inline-Banner „Zusage nicht erfüllt" (CVC-F-01). — CVC-F-02 (committed-Band im Chart) wird **bewusst in Phase 16 vorgezogen**, siehe D-04.

</domain>

<decisions>
## Implementation Decisions

### Verfügbar-Semantik (höchste Hebelwirkung)
- **D-01 (committed zählt zur verfügbaren Kapazität, im Backend):** `overall_available_hours = paid_hours + committed_voluntary_hours + volunteer_hours` (= `paid + max(committed, actual)` pro Person, durch das Zwei-Band-Modell). Die Summe entsteht **im Backend** in `get_weekly_summary` (`booking_information.rs`) — NICHT erst im Frontend — damit Diff-Spalte (verfügbar − benötigt), Chart und alle Konsumenten konsistent dieselbe „verfügbar"-Zahl sehen. Sicher, weil `WeeklySummary` year-view-only ist und **nicht** persistiert wird (kein Billing-Snapshot-Konsum → kein Schema-Bump; bleibt Version 7). Entspricht dem Milestone-Scope „verfügbar = expected + committed_voluntary".
  - Phase 15 ließ `overall_available_hours = volunteer + paid` bewusst stehen (`booking_information.rs:270-273`, Kommentar „Phase 16 will sum both bands for display"). Phase 16 löst diesen TODO ein.

### Token-Darstellung & Überschuss-Notation
- **D-02 (drei getrennte Tokens):** Die Tabelle zeigt drei getrennte Tokens — heute `💰paid | 🤝volunteer`, künftig `💰paid | 🎯zugesagt (committed) | 🤝surplus`. Der **bestehende `volunteer_hours`-Token (🤝)** zeigt jetzt nur noch den **Surplus über der Zusage** (Band 2, nach Phase 15) und bleibt unverändert weiterverwendet. Der neue „zugesagt"-Token zeigt `committed_voluntary_hours` (Band 1).
  - Der ROADMAP-Wortlaut „committed=5, actual=7 → 5 + 2" liest sich damit als `🎯5` + `🤝2`; gedeckter Fall „committed=5, actual=3 → 5" als `🎯5` + `🤝0`. KEIN kombinierter Inline-„5 + 2"-Token (Variante verworfen).
  - Token-Icon (`🎯` ist Vorschlag) und exakte Spalten-/Header-Anordnung sind Claude's Discretion, solange die drei Werte klar getrennt und nicht vermischt sind (ROADMAP SC1).

### Blank/0-Regel
- **D-03 (einfach „0" zeigen — KEINE blank/Strich-Sonderlogik):** Der committed-Token zeigt bei `committed_voluntary_hours == 0` ganz normal `0.00`, konsistent mit paid/volunteer (die heute ebenfalls `0.00` zeigen). In der **aggregierten** Jahresansicht bedeutet committed=0 schlicht „keine Zusage diese Woche" — eine „blank/Strich"-Sonderregel würde nur Komplexität ohne Mehrwert bringen.
  - **Konsequenz für REQUIREMENTS:** Die „blank/Strich, nicht 0"-Formulierung in **CVC-07** ist revidiert. Planner zieht CVC-07 + ROADMAP-Phase-16-SC#2 nach (committed wird als „0" gezeigt, nicht blank). Die blank/Strich-Idee gehört, falls überhaupt, in die **Mitarbeiteransicht** (Phase 17), wo „0" mit „hat 0 zugesagt" verwechselt werden könnte — dort separat zu entscheiden, NICHT in Phase 16.

### Chart-Behandlung
- **D-04 (drei gestapelte Farb-Segmente — CVC-F-02 in Phase 16 vorgezogen):** Der Balken in `weekly_overview_chart.rs` bekommt ein **drittes farbiges Segment** für committed. Stapel-Reihenfolge/Komposition: `paid + committed + surplus` (drei Farben). Die Required-Linie/Marker bleibt unverändert. Tooltip nennt alle drei Werte.
  - **Scope-Hinweis:** „committed-Band im Chart" war in Phase 15 als **v1.5 / CVC-F-02** deferred. Der User holt es bewusst in Phase 16 (gleiche Display-Fläche, kein neuer Capability-Typ). Planner darf CVC-F-02 als in Phase 16 erledigt markieren.
  - **No-double-count im Chart (vom User bestätigt):** Band 2 (`volunteer_hours`/surplus) hat committed **bereits per-Person abgezogen** (`Σ_Person max(actual_p − committed_p, 0)`, Phase 15). Daher stapeln sich die drei Segmente korrekt ohne Doppelzählung: `paid + committed + surplus = paid + max(committed, actual)`. Das Chart darf die drei Felder direkt addieren.
  - Heute plottet der Chart `bar_total = paid_hours + volunteer_hours` (`weekly_overview_chart.rs:16`) — dieser muss um committed erweitert werden, sonst weicht der Balken von der Diff-Spalte (D-01) ab.

### Claude's Discretion
- Exaktes Token-Emoji/Icon für „zugesagt" (`🎯` Vorschlag) und genaue Spalten-/Header-Anordnung der drei Tokens.
- Konkrete Farbwahl für das committed-Chart-Segment (Token-basiert, KEINE Hardcoded-Hex — die Tests in `weekly_overview.rs` verbieten Roh-Hex im Chart-Prod-Source).
- Exakte i18n-Label-Texte (z.B. „Zugesagt"/„Committed"/„Přislíbeno") und ob der bestehende Header-Key `PaidVolunteer` umbenannt/erweitert wird oder ein neuer Key dazukommt.
- Test-Platzierung (SSR-Render-Test in `page/weekly_overview.rs`, Chart-Segment-Test, `From`-Mapping-Test, Per-Locale-Matcher).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap / Requirements
- `.planning/ROADMAP.md` § „Phase 16: Jahresansicht display" — Goal + Success Criteria (⚠ SC#2 „blank/Strich, nicht 0" revidiert durch D-03 → committed wird als „0" gezeigt).
- `.planning/REQUIREMENTS.md` — CVC-07 (⚠ „blank/Strich"-Teil revidiert per D-03), CVC-08 (i18n De/En/Cs vollständig, Per-Locale-Matcher). CVC-F-02 (Chart-Band) wird per D-04 in Phase 16 erledigt.

### Vorgänger-Phasen (Pflichtlektüre)
- `.planning/phases/15-reporting-no-double-count-snapshot-bump-same-commit/15-CONTEXT.md` — Zwei-Band-Modell (D-05), Cap-Gating (D-01/D-05), no-double-count-Invariante, Phase-15/16-Schnitt (D-04: TO + Frontend in Phase 16).
- `.planning/phases/15-.../15-01-SUMMARY.md` + `15-02-SUMMARY.md` — was Phase 15 tatsächlich lieferte (inkl. der zweiten `get_weekly_summary`-Variante mit `committed_voluntary_hours: 0.0`-Placeholder — siehe Research-Flag unten).
- `.planning/phases/14-data-model-foundation-backend/14-CONTEXT.md` — `committed_voluntary` end-to-end gefädelt; SUM-Helper.

### Milestone-Research (v1.4)
- `.planning/research/SUMMARY.md` — Zwei-Achsen (Achse A `reporting.rs` vs. Achse B `booking_information.rs`); Achse B ist der Year-View-Pfad.
- `.planning/research/PITFALLS.md` — P1 (Doppelzählung), P2 (Snapshot-Drift — hier irrelevant, kein Bump).

### Integrations-Code (verifiziert beim Scout)
- `service/src/booking_information.rs:37-58` — `WeeklySummary`-Service-Struct, hat seit Phase 15 `committed_voluntary_hours: f32` (Band 1) + `volunteer_hours` (Band 2).
- `service_impl/src/booking_information.rs:207-280` — erste `get_weekly_summary`-Variante: vollständig gewired (Band 1 + Band 2). `:270-273` der TODO-Kommentar für D-01. `:273` `overall_available_hours = volunteer + paid` (anzupassen).
- `service_impl/src/booking_information.rs:337-547` — **zweite** `get_weekly_summary`-Variante: `committed_voluntary_hours: 0.0`-Placeholder (`:547`), `overall_available_hours = volunteer + paid` (`:386`). **Research-Flag — siehe `<deferred>`.**
- `rest-types/src/lib.rs:904-948` — `WeeklySummaryTO` + `From<&WeeklySummary>` (committed-Feld + Mapping fehlen noch).
- `shifty-dioxus/src/state/weekly_overview.rs:11-63` — Frontend-`WeeklySummary` + `From<&WeeklySummaryTO>` (committed-Feld + Mapping fehlen noch).
- `shifty-dioxus/src/page/weekly_overview.rs:102-108` — Token-Rendering `💰{paid} | 🤝{volunteer}` (Desktop + Mobile); `:87` Diff-Berechnung `available_hours - required_hours`; SSR-Test-Suite + `sample_week`-Helper (`:218`) braucht das neue Feld.
- `shifty-dioxus/src/component/weekly_overview_chart.rs:16-111` — `bar_total = paid + volunteer` (`:16`), gestapelte Segmente, Legenden-Labels (`Key::Paid`/`Key::Volunteer`/`Key::ChartRequiredHours`), Tooltip; Test-Suite verbietet Hardcoded-Hex.
- `shifty-dioxus/src/i18n/mod.rs:120` (`Key`-Enum) + `en.rs`/`de.rs`/`cs.rs` — neue Keys hier in **allen drei** Locales pflegen; `Key::PaidVolunteer` (`de.rs:133`, `en.rs:105`) ist der Tabellen-Header.

### Frontend-Codebase-Maps
- `.planning/codebase/frontend/CONVENTIONS.md`, `STRUCTURE.md`, `TESTING.md` — Komponenten/State/Service-Pattern, SSR-Test-Konventionen, Per-Locale-Matcher (v1.3-Vorbild).

### Projekt-Regeln
- `shifty-backend/CLAUDE.md` § „Billing Period Snapshot Schema Versioning" — kein Bump (WeeklySummary nicht persistiert; bleibt Version 7).
- `shifty-dioxus/CLAUDE.md` — i18n in allen drei Locales; `Locale::De`-statt-`Locale::En`-Bug-Falle.
- `CLAUDE.local.md` — jj-only Commits (kein git/jj-Commit aus Agents heraus), NixOS `nix develop`; WASM-Build-Gate `cargo build --target wasm32-unknown-unknown`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `format_hours(value, decimals)` (`base_types`) — bestehende Stunden-Formatierung, für den committed-Token wiederverwenden.
- `WeeklyOverviewTable`-SSR-Test-Harness + `sample_week`-Helper (`page/weekly_overview.rs:218,237`) — neue committed-Tests andocken; `sample_week`-Signatur um committed erweitern.
- Per-Locale-Reference-Matcher-Pattern aus v1.3 (i18n-Tests) — gegen den `Locale::En`-statt-`Locale::De`-Bug.

### Established Patterns
- Dreischicht Frontend: Service (`weekly_summary.rs`) lädt TO → `state`-`From`-Mapping → `page`/`component` rendern. committed muss durch **alle drei** Boundaries (TO → state → render), sonst Omission-Lücke (ROADMAP SC1).
- Chart-Segmente sind token-basiert gefärbt (keine Roh-Hex); Tests pinnen das.
- `overall_available_hours` ist year-view-only (nicht persistiert) → Änderung an D-01 ist snapshot-neutral.

### Integration Points
- Backend: committed in `overall_available_hours` einrechnen (beide `get_weekly_summary`-Varianten) + `WeeklySummaryTO`-`From`.
- Frontend: `From<&WeeklySummaryTO>` (state) → dritter Token (Tabelle) + drittes Segment (Chart) + i18n.

</code_context>

<specifics>
## Specific Ideas

- Token-Layout-Idee: `💰{paid} | 🎯{committed} | 🤝{surplus}` (Desktop-Header + Mobile-Inline-Zeile, beide Stellen in `page/weekly_overview.rs:102-108` anpassen).
- Überschuss-Lesart: `committed=5, actual=7` → `🎯5 + 🤝2`; `committed=5, actual=3` → `🎯5 + 🤝0`.
- Chart: drei gestapelte Farben paid/committed/surplus; Required-Linie bleibt; Balken-Total = paid+committed+surplus.

</specifics>

<deferred>
## Deferred Ideas

- **Editor-Input (`contract_modal.rs`) für `committed_voluntary` + „alle"-Filter Mitarbeiteransicht + unpaid-volunteer-Record + `is_paid`-Gating** → **Phase 17**.
- **Blank/Strich-Darstellung statt „0"** → falls überhaupt, dann in der **Mitarbeiteransicht (Phase 17)**, NICHT in der aggregierten Jahresansicht (D-03).
- **Inline-Banner „Zusage nicht erfüllt"** → **v1.5 (CVC-F-01)**.

### Research Flag (für gsd-phase-researcher / gsd-planner) — HOCH-LEVERAGE
- **Zwei `get_weekly_summary`-Varianten:** `service_impl/src/booking_information.rs` enthält zwei Implementierungen. Die erste (`:207-280`) ist seit Phase 15 voll gewired (Band 1 + Band 2). Die zweite (`:337-547`) setzt `committed_voluntary_hours: 0.0` als **Placeholder** (`:547`) und nutzt `overall_available_hours = volunteer + paid` (`:386`). **Researcher MUSS klären:** Welche Variante speist tatsächlich den `weekly_overview`-Year-View (Achse B)? Wenn die zweite Variante (oder beide) den Frontend-Pfad bedient, muss committed **dort gewired** werden (sonst zeigt die UI 0 trotz korrekter Berechnung in der ersten Variante). Beide `overall_available_hours`-Zeilen (`:273` + `:386`) für D-01 prüfen. Siehe `15-01-SUMMARY.md` / `15-02-SUMMARY.md` für den Placeholder-Kontext.

</deferred>

---

*Phase: 16-jahresansicht-display*
*Context gathered: 2026-06-24*
