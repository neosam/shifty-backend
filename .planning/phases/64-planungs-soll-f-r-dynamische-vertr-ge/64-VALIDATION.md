---
phase: 64
slug: planungs-soll-f-r-dynamische-vertr-ge
status: draft
nyquist_compliant: false
wave_0_complete: false
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
| {tbd} | {tbd} | 1 | PSOLL-01 | — | N/A (reine Anzeige-Berechnung) | unit | `cargo test -p service_impl test_planning_target_dynamic_get_week` | ❌ W0 | ⬜ pending |
| {tbd} | {tbd} | 1 | PSOLL-03 | — | N/A | unit | `cargo test -p service_impl test_planning_target_parity_both_sites` | ❌ W0 | ⬜ pending |
| {tbd} | {tbd} | 1 | PSOLL-04 | — | N/A | unit | `cargo test -p service_impl test_available_hours_unchanged_dynamic` | ❌ W0 | ⬜ pending |
| {tbd} | {tbd} | 1 | PSOLL-04 | — | N/A | unit | `cargo test -p service_impl test_planning_target_matches_expected_for_fixed_contract` | ❌ W0 | ⬜ pending |
| {tbd} | {tbd} | 2 | PSOLL-02 | — | N/A | statisch | siehe § Manual-Only — Grep-Assertion, kein sinnvoller Unit-Test | n/a | ⬜ pending |

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
- [ ] Falls D-64-01 umgesetzt wird: die bestehenden
      `working_hours_mini_overview.rs`-Tests um das neue Feld erweitern
      (`make_row`-Helfer, `WorkingHoursMini`-Literale), sonst Compile-Fehler durch
      fehlendes Struct-Feld

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
- [ ] Sampling-Kontinuität: keine drei aufeinanderfolgenden Tasks ohne automatisierten Verify
- [ ] Wave 0 deckt alle MISSING-Referenzen
- [ ] Keine Watch-Mode-Flags
- [ ] Feedback-Latenz < 15 s
- [ ] `nyquist_compliant: true` im Frontmatter gesetzt

**Approval:** pending
