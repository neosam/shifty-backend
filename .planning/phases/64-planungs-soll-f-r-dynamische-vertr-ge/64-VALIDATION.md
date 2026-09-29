---
phase: 64
slug: planungs-soll-f-r-dynamische-vertr-ge
status: approved
nyquist_compliant: true
wave_0_complete: false  # Testdatei wird in Plan 64-01 Task 2 angelegt
created: 2026-09-29
---

# Phase 64 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Abgeleitet aus `64-RESEARCH.md` § Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust `#[tokio::test]` / `#[test]`) + `mockall` |
| **Config file** | keine separate Config — Workspace-`Cargo.toml` |
| **Quick run command** | `cargo test -p service_impl reporting` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| **Estimated runtime** | ~10 s quick, ~90 s full (95 Integrationstests dominieren) |

**Toolchain-Hinweis:** `cargo` liegt nicht im Standard-PATH. Alle Kommandos über
`nix develop --command bash -c '...'` aus dem Projekt-Root, oder PATH vorher
erweitern. `cargo fmt` ist in der Nix-Shell nicht verfügbar.

**`shifty-dioxus` ist kein Workspace-Member** (`exclude`d) — `cargo clippy --workspace`
deckt das Frontend NICHT ab. Sobald D-64-01 umgesetzt wird, kommen dazu:

```
cargo test   --manifest-path shifty-dioxus/Cargo.toml            (Baseline 858 passed)
cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings
cargo build  --manifest-path shifty-dioxus/Cargo.toml --target wasm32-unknown-unknown
```

---

## Sampling Rate

- **After every task commit:** `cargo test -p service_impl reporting` (+ das
  `shifty-dioxus`-Trio, falls der Task Frontend-Dateien anfasst)
- **After every plan wave:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Before `/gsd-verify-work`:** Volle Suite grün, inkl. WASM-Build
- **Max feedback latency:** ~10 s (Quick-Lauf)

---

## Per-Task Verification Map

Die Task-IDs stehen erst nach der Planung fest; diese Tabelle bindet die
Requirements an ihre Nachweise, der Planner trägt die IDs ein.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 64-01 / Task 2 (RED), Task 3 (GREEN) | 64-01 | 1 | PSOLL-01 | — | N/A (reine Anzeige-Berechnung) | unit | `cargo test -p service_impl test_planning_target_dynamic_get_week` | ❌ W0 | ⬜ pending |
| 64-01 / Task 2 (RED), Task 3 (GREEN) | 64-01 | 1 | PSOLL-01 | — | N/A | unit | `cargo test -p service_impl test_planning_target_dynamic_all_employees` | ❌ W0 | ⬜ pending |
| 64-01 / Task 2 (RED), Task 3 (GREEN) | 64-01 | 1 | PSOLL-03 | — | N/A | unit | `cargo test -p service_impl test_planning_target_parity_both_sites` | ❌ W0 | ⬜ pending |
| 64-01 / Task 2 | 64-01 | 1 | PSOLL-04 | T-64-02 | Verfuegbarkeitsrechnung liest weiterhin das bilanzrelevante Soll | unit | `cargo test -p service_impl test_available_hours_unchanged_dynamic` | ❌ W0 | ⬜ pending |
| 64-01 / Task 2 (RED), Task 3 (GREEN) | 64-01 | 1 | PSOLL-04 | — | N/A | unit | `cargo test -p service_impl test_planning_target_matches_expected_for_fixed_contract` | ❌ W0 | ⬜ pending |
| 64-01 / Task 3 | 64-01 | 1 | PSOLL-04 | T-64-02, T-64-03 | kein Feldname in Verfuegbarkeitsrechnung und Snapshot-Schreiber, Snapshot-Version 13 | statisch | Quellcode-Gates in `64-01-PLAN.md` Task 3 `<verify>` | n/a | ⬜ pending |
| 64-02 / Task 2 | 64-02 | 2 | PSOLL-01 | — | N/A | component (SSR) | `cargo test --manifest-path shifty-dioxus/Cargo.toml working_hours_mini_overview_dynamic_contract_shows_planning_target` | ❌ W0 | ⬜ pending |
| 64-02 / Task 2 | 64-02 | 2 | PSOLL-02 | — | N/A | statisch (Quellcode-Invariante im Testmodul) | `cargo test --manifest-path shifty-dioxus/Cargo.toml working_hours_mini_overview_reads_planning_target_only` | ❌ W0 | ⬜ pending |
| 64-04 / Task 2 | 64-04 | 4 | PSOLL-01, PSOLL-02 | — | N/A | manuell | siehe § Manual-Only — blockierender Checkpoint | n/a | ⬜ pending |

**Aenderung gegenueber dem Entwurf:** PSOLL-02 galt hier als „kein sinnvoller Unit-Test moeglich".
Das gilt nur fuer die Vollstaendigkeitsaussage („keine Fallback-Regel welcher Art auch immer").
Ein dauerhafter Quellcode-Invariantentest in derselben Datei ist sehr wohl moeglich und im Repo
praezedenzgestuetzt (`working_hours_mini_overview_no_legacy_classes_in_source` liest die eigene
Quelle und isoliert die Produktionsregion). Plan 64-02 baut ihn; die Sichtpruefung pro
Trefferstelle bleibt zusaetzlich im UAT.

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Neue Testdatei `service_impl/src/test/reporting_planning_target.rs` — deckt
      PSOLL-01, PSOLL-03 und PSOLL-04 ab; muss in `service_impl/src/test/mod.rs`
      registriert werden
- [ ] **Kein neuer Fixture-Bedarf** — `reporting_phase2_fixtures.rs` liefert bereits
      ein Spiegel-Paar (dynamischer und fester Vertrag)
- [ ] Das Cross-Site-Paritätsmuster für PSOLL-03 ist in
      `reporting_additive_merge.rs` (Zeilen 962–1241) vorhanden und 1:1 kopierbar —
      nicht neu erfinden
- [ ] D-64-01 IST entschieden (alle drei Anzeigestellen, Plan 64-02): die bestehenden
      `working_hours_mini_overview.rs`-Tests um das neue Feld erweitern
      (`make_row`-Helfer plus elf `WorkingHoursMini`-Literale), sonst Compile-Fehler durch
      fehlendes Struct-Feld. Plan 64-02 Task 1.
- [ ] Zusaetzlicher Wave-0-Befund des Planners: das neue DTO-Feld bricht drei
      `ShortEmployeeReportTO`-Literale im Frontend (`loader.rs` Testhelfer `make_report`,
      `tests/volunteer_work_tests.rs` zweimal) und drei `ShortEmployeeReport`-Literale im
      Backend-Testbaum (`booking_information_weekly_summary_year_boundary.rs` zweimal,
      `booking_information_vaa.rs` einmal). Weil `shifty-dioxus` kein Workspace-Member ist,
      MELDET `cargo test --workspace` den Frontend-Bruch nicht. Plan 64-01 Task 1 zieht deshalb
      alle sechs Literale mit und fuehrt die Frontend-Gates in seiner Verifikation.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Kein Fallback oder keine Ableitungsregel im Frontend | PSOLL-02 | Die Abwesenheit einer Berechnung ist keine Programmeigenschaft, die ein Unit-Test beobachten kann. Ein Test könnte nur eine konkrete falsche Formel ausschließen, nicht alle. | `grep -nE "planning_target_hours\|expected_hours\|dynamic_hours" shifty-dioxus/src/component/working_hours_mini_overview.rs` — jeder Treffer darf nur ein Feldzugriff sein, kein arithmetischer Ausdruck. Im Review pro Treffer bestätigen. |
| Sollstunden-Zahl > 0 erscheint im Browser unter dem Schichtplan | PSOLL-01 | Die Zahl entsteht im Backend und wird per WASM gerendert; die Render-Kette ist headless nur eingeschränkt prüfbar. Backend-Wert ist automatisiert belegt, die Anzeige nicht. | Dev-Server starten, einen Mitarbeiter mit dynamischem Vertrag öffnen, Übersicht unter dem Schichtplan prüfen: Zahl > 0. Zum Vergleich einen Mitarbeiter mit festem Vertrag: unveränderte Zahl. |

---

## Bekannte Fallen (aus RESEARCH.md, hier als Prüfpunkte)

Diese vier sind der Grund, warum diese Phase eine eigene Test-Matrix braucht:

1. **Reduktionsterme naiv wiederverwenden** — die Feiertags-/Abwesenheits-Reduktion
   hängt an einem Gate (`planned_hours <= 0.0` bzw. `expected_hours <= 0.0`), das
   für dynamische Verträge **immer** wahr ist. Wer die Terme unverändert
   übernimmt, überspringt die Reduktion genau dort, wo sie neu gebraucht wird.
2. **`get_reports_for_all_employees`-Dynamikzweig** fehlt die Holiday-Berechnung
   komplett — die zweite Konstruktionsstelle ist nicht symmetrisch zur ersten.
3. **`expected_hours` versehentlich ersetzen** statt ein Feld daneben zu stellen —
   bricht PSOLL-04 und die Balance.
4. **`has_pending_rebooking`-Präzedenz** als Vorbild missverstehen.

---

## Validation Sign-Off

- [ ] Alle Tasks haben `<automated>` verify oder eine Wave-0-Abhängigkeit
- [x] Sampling-Kontinuität: keine drei aufeinanderfolgenden Tasks ohne automatisierten Verify
- [x] Wave 0 deckt alle MISSING-Referenzen
- [x] Keine Watch-Mode-Flags
- [x] Feedback-Latenz < 15 s
- [x] `nyquist_compliant: true` im Frontmatter gesetzt

**Approval:** approved (Planner, 2026-09-29) — Task-IDs eingetragen, PSOLL-02 auf einen automatisierten Quellcode-Invariantentest gehoben, zusaetzlicher Wave-0-Befund (sechs Struct-Literale) ergaenzt.
