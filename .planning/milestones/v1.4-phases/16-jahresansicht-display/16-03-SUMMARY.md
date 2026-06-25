---
phase: 16-jahresansicht-display
plan: 03
subsystem: ui
tags: [committed-voluntary, weekly-overview, i18n, dioxus-ssr, chart, token-render, two-band, var-good]

# Dependency graph
requires:
  - phase: 16-jahresansicht-display
    provides: "16-01: WeeklySummaryTO.committed_voluntary_hours + From-Mapping; overall_available_hours summiert paid + committed (Band 1) + volunteer (Band 2)."
  - phase: 16-jahresansicht-display
    provides: "16-02: Frontend-State WeeklySummary.committed_voluntary_hours + From<&WeeklySummaryTO>-Mapping; sample_week-Helper tragen das Pflichtfeld."
provides:
  - "D-02/D-03: Jahresansicht-Tabelle rendert drei getrennte Tokens 💰paid | 🎯committed | 🤝surplus (Desktop + Mobile); committed=0 → 🎯0.00 (keine blank/Strich-Sonderlogik)."
  - "D-04: WeeklyOverviewChart bekommt drittes gestapeltes committed-Segment (var(--good)) zwischen paid (bottom) und surplus (top); bar_total = paid + committed + surplus; Tooltip + Legend nennen alle drei Bänder."
  - "CVC-08: Key::Committed + Key::PaidCommittedVolunteer in De/En/Cs vollständig; zwei bestehende cs.rs-Lücken (Key::Volunteer, Key::PaidVolunteer) geschlossen; Per-Locale-Reference-Matcher-Tests."
  - "committed-Term durch alle drei Render-Boundaries (Token, Chart, Diff via available_hours) — Phase-16-sichtbare-Wirkung komplett."
affects: [17-mitarbeiteransicht-editor (committed_voluntary Editor-Wiring + ggf. blank/Strich-Sonderlogik)]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Drei-Token-String in einer Tabellen-Zelle via format!(\"💰{} | 🎯{} | 🤝{}\", ...); kein CSS-Gap, kein kombinierter Token — D-02-konform."
    - "Chart-Segment-Farben token-basiert (var(--good) für committed); Hex-Audit-Test pinnt Abwesenheit von Roh-Hex im Prod-Source."
    - "Per-Locale-Reference-Matcher-Test gegen den Locale::De/Locale::En-Swap-Bug (generate(Locale::*).t(Key) == gepinnter String)."
    - "Volunteer-Guard-Test von globaler !contains(var(--good)) auf segment-spezifische Assertion verengt, damit ein legitimes committed-var(--good)-Segment koexistieren kann."

key-files:
  created: []
  modified:
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
    - shifty-dioxus/src/page/weekly_overview.rs
    - shifty-dioxus/src/component/weekly_overview_chart.rs

key-decisions:
  - "ROADMAP-SC#2-Korrektur (Task 3, Schritt 10) war bereits in der uncommitteten Working-Copy auf die D-03-Form gebracht (🎯0.00, keine blank/Strich-Sonderlogik); keine erneute Bearbeitung nötig — Gates (🎯0.00 present, kein 'blank/Strich, nicht 0'-Wortlaut) verifiziert."
  - "16-VALIDATION.md trägt bereits die Manual-Only-Verifications (Drei-Farben-Stapelung + Czech-Strings) und abgehakte Wave-0-Requirements — keine Ergänzung nötig."
  - "compute_max_hours_uses_larger_of_bar_or_required gezielt mit committed > 0 (bar_total 30+5+12=47) umgebaut, damit der Test die committed-Summierung pinnt statt nur 0.0 durchzureichen."

patterns-established:
  - "sample_week-Helper-Signatur in BEIDEN Test-Modulen (page + chart) identisch um committed: f32 zwischen paid und volunteer erweitert; available_hours = paid + committed + volunteer (Pitfall-6-Guard gegen divergierende Test-Helper)."
  - "render_view-/ViewProps-Test-Harness reicht committed_label analog zu volunteer_label durch, damit Chart-View-Tests mit der erweiterten Props-Signatur kompilieren."

requirements-completed: [CVC-07, CVC-08]

# Metrics
duration: ~18min
completed: 2026-06-24
---

# Phase 16 Plan 03: Jahresansicht display (Render/i18n) Summary

**committed_voluntary fließt jetzt durch die dritte Boundary (Render): drittes Tabellen-Token 🎯 (Desktop + Mobile), drittes gestapeltes Chart-Segment in var(--good), drei neue i18n-Strings in allen drei Locales — plus Schließen zweier cs.rs-Lücken und Verengung des Volunteer-var(--good)-Guard-Tests (D-04).**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-06-24
- **Completed:** 2026-06-24
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- **D-02/D-03 (Task 2):** Die Jahresansicht-Tabelle zeigt drei getrennte Tokens `💰{paid} | 🎯{committed} | 🤝{surplus}` an Desktop- und Mobile-Zelle; der Header nutzt den neuen `Key::PaidCommittedVolunteer`. committed=0 rendert `🎯0.00` (plain zero, zwei Dezimalstellen) — keine blank/Strich-Sonderlogik. Drei neue SSR-Tests pinnen das (🎯5.00/🤝2.00-Trennung, 🎯0.00, Drei-Band-Header).
- **D-04 (Task 3):** Der `WeeklyOverviewChart` bekommt ein drittes gestapeltes committed-Segment in `var(--good)` zwischen paid (bottom) und surplus (top); `bar_total = paid + committed_voluntary_hours + volunteer`; Tooltip + Legend nennen alle drei Bänder. Der versteckte Checker-Blocker — die globale `!html.contains("background: var(--good)")`-Assertion in `chart_volunteer_uses_ink_muted_not_good` — wurde auf eine segment-spezifische Volunteer-Assertion verengt (committed > 0, damit das committed-var(--good)-Segment koexistiert).
- **CVC-08 (Task 1):** `Key::Committed` (Zugesagt/Committed/Přislíbeno) + `Key::PaidCommittedVolunteer` (Bezahlt / Zugesagt / Freiwillig | Paid / Committed / Volunteer | Placené / Přislíbeno / Dobrovolné) in allen drei Locales; zwei bestehende cs.rs-Lücken (`Key::Volunteer` = Dobrovolné, `Key::PaidVolunteer` = Placené / Dobrovolné) geschlossen; vier neue Per-Locale-Matcher-Tests gegen den Locale-Swap-Bug.
- **Hex-Audit + WASM-Gate:** Kein Roh-Hex im Chart-Prod-Source (verbotene Hexes von Test gepinnt); `cargo build --target wasm32-unknown-unknown` exit 0; volle Frontend-Suite 614 passed / 0 failed (vorher 606 + 8 neue Tests).

## Task Commits

**KEINE Commits durch den Executor** — dieses Repo ist jj-managed, GSD-Auto-Commit ist deaktiviert. Alle Änderungen liegen uncommitted im Working Copy; der User committet manuell via jj. (Per `<vcs_jj_only>` in Plan + Prompt.)

Tasks logisch abgeschlossen:

1. **Task 1: i18n — Key::Committed + Key::PaidCommittedVolunteer in 3 Locales + cs.rs-Lücken + Per-Locale-Matcher** (TDD) — `src/i18n/mod.rs`, `src/i18n/de.rs`, `src/i18n/en.rs`, `src/i18n/cs.rs`.
2. **Task 2: page — dritter Token 🎯committed (Desktop + Mobile) + Header-Key + sample_week + SSR-Tests** (TDD) — `src/page/weekly_overview.rs`.
3. **Task 3: chart — drittes committed-Segment var(--good) + bar_total + Tooltip/Legend/Props + Volunteer-Guard verengt + ROADMAP-SC#2 + Hex-Audit + WASM-Gate** (TDD) — `src/component/weekly_overview_chart.rs`, `.planning/ROADMAP.md` (bereits D-03-konform, siehe Decisions).

## Files Created/Modified
- `shifty-dioxus/src/i18n/mod.rs` — Key-Enum: zwei neue Varianten `PaidCommittedVolunteer`, `Committed` nahe `Key::PaidVolunteer`; vier neue Test-Funktionen (`i18n_committed_keys_match_{german,english,czech}_reference`, `i18n_czech_closes_volunteer_and_paid_volunteer_gaps`).
- `shifty-dioxus/src/i18n/de.rs` — `Key::Committed` = "Zugesagt", `Key::PaidCommittedVolunteer` = "Bezahlt / Zugesagt / Freiwillig".
- `shifty-dioxus/src/i18n/en.rs` — `Key::Committed` = "Committed", `Key::PaidCommittedVolunteer` = "Paid / Committed / Volunteer".
- `shifty-dioxus/src/i18n/cs.rs` — `Key::Committed` = "Přislíbeno", `Key::PaidCommittedVolunteer` = "Placené / Přislíbeno / Dobrovolné", plus geschlossene Lücken `Key::PaidVolunteer` = "Placené / Dobrovolné" und `Key::Volunteer` = "Dobrovolné".
- `shifty-dioxus/src/page/weekly_overview.rs` — Token-String an Desktop- + Mobile-Zelle auf `💰{} | 🎯{} | 🤝{}` umgestellt; Header von `Key::PaidVolunteer` auf `Key::PaidCommittedVolunteer`; `sample_week`-Signatur um `committed: f32` erweitert (alle Aufrufer nachgezogen); drei neue SSR-Tests (`page_renders_three_separate_tokens_committed_and_surplus`, `page_committed_zero_renders_plain_zero_no_dash`, `page_header_uses_three_band_key`).
- `shifty-dioxus/src/component/weekly_overview_chart.rs` — `bar_total` + `compute_max_hours` summieren committed; `committed_pct`; drittes RSX-Segment `background: var(--good)`; `committed_label` durch Wrapper/Props/Legend/Tooltip/Test-ViewProps gethreadet; `sample_week`-Signatur erweitert (alle Aufrufer nachgezogen); `chart_uses_token_styles_not_legacy_hex` um `var(--good)`-Assertion erweitert; `chart_volunteer_uses_ink_muted_not_good` verengt (committed > 0, segment-spezifisch); neuer `chart_tooltip_names_all_three_bands`-Test; `compute_max_hours_uses_larger_of_bar_or_required` pinnt committed im bar_total (47.0).
- `.planning/ROADMAP.md` — Phase-16-SC#2 + Goal-Absatz: bereits in der uncommitteten Working-Copy auf die D-03-Form ("committed=0 wird als 🎯0.00 gezeigt, keine blank/Strich-Sonderlogik") gebracht; Gates verifiziert, keine erneute Bearbeitung nötig.

## Decisions Made
- **ROADMAP-SC#2 bereits D-03-konform:** Der Plan-Task-3-Schritt-10 verlangt, den "blank/Strich, nicht 0"-Wortlaut auf die D-03-Form umzuschreiben. Die Working-Copy-`ROADMAP.md` (im git-Status als `M` markiert) trug diesen Wortlaut bereits korrigiert: `grep "🎯0.00"` liefert 3 Treffer im Phase-16-Block, `grep "blank/Strich.*nicht .0."` liefert 0 Treffer. Der Überschuss-Teil von SC#2 (committed=5/actual=7 etc.) ist unverändert. Keine erneute Edit nötig — Akzeptanzkriterium erfüllt.
- **compute_max_hours-Test mit committed > 0:** Statt nur 0.0 durchzureichen wurde `compute_max_hours_uses_larger_of_bar_or_required` so umgebaut, dass eine Woche committed=5 trägt (bar_total = 30+5+12 = 47); das pinnt die CVC-07h-Summierung aktiv, nicht nur passiv.
- **Volunteer-Guard-Verengung (Schritt 8a) exakt nach Plan-SOLL:** committed=8 in der Test-Woche, negative Assertion segment-spezifisch (`!html.contains("background: var(--good); opacity: 0.35")`) plus positive Sanity-Assertion, dass das committed-Segment `var(--good)` (ohne opacity-Suffix) tatsächlich rendert.

## Deviations from Plan

None - plan executed exactly as written. (Die ROADMAP-Edit war bereits in der Working-Copy vorgenommen; das ist kein Abweichen vom Plan-Intent, sondern ein bereits erfüllter Plan-Schritt — siehe Decisions.)

## Issues Encountered
- `cargo`/`dx` nicht direkt auf PATH (NixOS) — alle Test-/Build-Läufe via `nix develop --command bash -c '...'` aus `shifty-dioxus/` (eigene Cargo-Workspace; `cargo test -p shifty-dioxus` aus dem Backend-Root schlägt mit "did not match any packages" fehl, daher aus `shifty-dioxus/` ohne `-p`).

## User Setup Required

None - no external service configuration required.

## Manual Verification (an User markiert)
Per Plan-`<output>` + 16-VALIDATION.md § Manual-Only-Verifications:
1. **Visuelle Drei-Farben-Stapelung des Charts im Browser:** Jahresansicht öffnen, prüfen, dass die Balken drei unterscheidbare Segmente (paid var(--accent) bottom · committed var(--good) middle · surplus var(--ink-muted) top) zeigen und der Tooltip alle drei Werte nennt. (SSR pinnt Klassen/Token, nicht gerenderte Pixel/Farb-Lesbarkeit.)
2. **Czech-Strings sprachlich prüfen (MEDIUM-confidence A3):** `Přislíbeno` (committed), `Dobrovolné` (volunteer), `Placené` (paid) in `cs.rs` review.

## Next Phase Readiness
- Phase-16-sichtbare-Wirkung komplett: committed ist durch alle drei Render-Boundaries (Token, Chart, Diff via available_hours). Keine Omission-Lücke (ROADMAP SC1).
- Offener Phase-17-Follow-up (aus 16-02, deferred-items.md): `committed_voluntary` ins Frontend-`EmployeeWorkDetails`-State + Editor (`contract_modal.rs`), ersetzt die `0.0`-Wire-Default-Platzhalter; ggf. blank/Strich-Sonderlogik nur für die Mitarbeiteransicht (D-03 verweist sie dorthin).

## Self-Check: PASSED

- FOUND: shifty-dioxus/src/i18n/mod.rs (Enum-Varianten + 4 Matcher-Tests; PaidCommittedVolunteer-grep = 4)
- FOUND: shifty-dioxus/src/i18n/{de,en,cs}.rs (je 1 Key::Committed add_text; cs.rs Volunteer + PaidVolunteer geschlossen)
- FOUND: shifty-dioxus/src/page/weekly_overview.rs (🎯-Token Desktop+Mobile, Header umgestellt, sample_week erweitert, 3 SSR-Tests)
- FOUND: shifty-dioxus/src/component/weekly_overview_chart.rs (bar_total + committed-Segment var(--good) + verengter Volunteer-Guard + Tooltip-Test)
- FOUND: .planning/phases/16-jahresansicht-display/16-03-SUMMARY.md
- VERIFIED: `cargo test i18n` (aus shifty-dioxus/) grün — 36 passed, alle 5 neuen i18n-Tests grün
- VERIFIED: `cargo test weekly_overview` grün — 30 passed inkl. 3 neue SSR-Tests + page_source_does_not_use_legacy_classes
- VERIFIED: `cargo test weekly_overview_chart` grün — 13 passed inkl. verengter Volunteer-Guard + chart_uses_token_styles_not_legacy_hex (mit var(--good)) + chart_tooltip_names_all_three_bands
- VERIFIED: `cargo build --target wasm32-unknown-unknown` exit 0
- VERIFIED: `cargo test` (volle Frontend-Suite) 614 passed / 0 failed
- VERIFIED: Hex-Audit — kein Roh-Hex (#3B82F6/#10B981/#EF4444/#e5e7eb/#6b7280/#374151) im Chart-Prod-Source
- VERIFIED: Locale-Swap-Guard — 0 Locale::En in de.rs, 0 Locale::De in en.rs
- VERIFIED: ROADMAP — 🎯0.00 (3 Treffer) im Phase-16-Block, kein "blank/Strich, nicht 0"-Wortlaut
- N/A: Commits — bewusst keine (jj-only, User committet manuell)

---
*Phase: 16-jahresansicht-display*
*Completed: 2026-06-24*
