---
phase: 62
slug: personen-verzeichnis-eine-liste-statt-tabs
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-09-19
---

# Phase 62 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from `62-RESEARCH.md` § Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust `#[test]` / `#[tokio::test]`); Frontend über `dioxus_ssr`-Rendertests im selben Runner |
| **Config file** | none — Workspace-`Cargo.toml` genügt |
| **Quick run command** | `nix develop --command bash -c 'SQLX_OFFLINE=true cargo test -p <crate> <modulpfad>'` |
| **Full suite command** | `nix develop --command bash -c 'SQLX_OFFLINE=true cargo test --workspace'` **und** `nix develop --command bash -c 'SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings'` |
| **Estimated runtime** | ~180 s (Workspace-Test) + ~120 s (Clippy, kalt deutlich länger) |

> **Clippy ist ein eigenständiges Hard-Gate.** `cargo test` führt Clippy NICHT
> mit aus. Jede Wave-Merge- und Phase-Gate-Prüfung muss Clippy separat fahren.
>
> **Toolchain:** weder `cargo` noch `node` liegen im Default-PATH.
> `cargo`/`clippy`/`sqlx` nur über `nix develop --command bash -c '<cmd>'` mit
> `SQLX_OFFLINE=true`. `rustfmt` fehlt in der DevShell — **nicht** repo-weit
> reformatieren.

---

## Sampling Rate

- **After every task commit:** Quick-run des betroffenen Crates
  (`service_impl`, `shifty_bin` oder `shifty-dioxus`) **plus**
  `cargo clippy --workspace -- -D warnings`.
- **After every plan wave:** `cargo test --workspace` + `cargo clippy --workspace -- -D warnings`.
- **Before `/gsd-verify-work`:** volle Suite grün **und** `dx serve`-Smoke gegen
  den neuen Routen-Präfix (Success Criterion 5 — nicht durch Unit-Test ersetzbar).
- **Max feedback latency:** ~300 s (Test + Clippy)

---

## Per-Task Verification Map

Task-IDs werden beim Planen vergeben; die Zeilen unten sind der
Requirement-seitige Vertrag, den die Pläne abdecken müssen.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | PADM-01 | — | N/A | unit | `cargo test -p service_impl --lib test::person_directory::liste_enthaelt_alle_drei_auspraegungen -- --exact` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-01 | — | N/A | unit (SSR) | `cargo test -p shifty-dioxus --lib page::user_management::tests::directory_search_matches_both_names -- --exact` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-06 | — | N/A | unit | `cargo test -p service_impl --lib test::person_directory::display_name_reines_benutzerkonto_ist_username -- --exact` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-06 | T-62-03 | Kein Namens-Fallback im Frontend (Fat Backend, Thin Client) | grep-Artefakt | `grep -n "unwrap_or\|\.or(" shifty-dioxus/src/page/user_management.rs` — kein Treffer im Namenskontext | n/a | ⬜ pending |
| TBD | TBD | TBD | PADM-08 | T-62-01 | HR ohne Admin-Rolle bekommt 403 vom Aggregat-Endpoint | integration (`tower::oneshot`) | `cargo test -p shifty_bin --bin shifty_bin person_directory::hr_ohne_admin_bekommt_403 -- --exact` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-08 | T-62-01 | Admin bekommt 200 mit Daten | integration (`tower::oneshot`) | `cargo test -p shifty_bin --bin shifty_bin person_directory::admin_bekommt_200 -- --exact` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | PADM-08 | T-62-02 | Component-Guard blockiert Nicht-Admin bei direktem Routenaufruf (Defense-in-Depth) | unit (SSR) | `cargo test -p shifty-dioxus --lib page::user_management::tests::non_admin_sees_not_authorized -- --exact` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SC-5 | — | N/A | unit (Store) | `cargo test -p shifty-dioxus --lib service::user_management::tests::load_directory_ist_ein_request -- --exact` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | D-62-05 / WR-02 | T-62-04 | Soft-Delete-Filter in der `ON`-Klausel, nicht in `WHERE` — Benutzer mit nur soft-gelöschter Sales Person bleibt Kandidat | unit | `cargo test -p service_impl --lib test::person_candidates::user_mit_nur_soft_geloeschter_sales_person_ist_kandidat -- --exact` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `service_impl/src/test/person_directory.rs` (neu) — Stubs für PADM-01, PADM-06
- [ ] `service_impl/src/test/person_candidates.rs` — Ergänzung `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat` (D-62-05 / WR-02)
- [ ] `shifty_bin/src/integration_test/person_directory.rs` (neu) — PADM-08 auf REST-Ebene (`tower::oneshot`, Muster `voluntary_stats.rs`) **plus** `mod person_directory;` in `shifty_bin/src/integration_test.rs`
- [ ] `shifty-dioxus/src/page/user_management.rs` — bestehendes `#[cfg(test)] mod tests` erweitern; `render_with`/`render_users_with` zu einem gemeinsamen Renderer für die Einzelliste zusammenführen
- [ ] Framework-Install: **keiner nötig** — `cargo test` und `dioxus_ssr` sind bereits Workspace-Abhängigkeiten

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Neuer Routen-Präfix in `shifty-dioxus/Dioxus.toml` wird vom Dev-Proxy korrekt weitergereicht | Success Criterion 5 | Der Proxy-Präfix greift nur zur Laufzeit von `dx serve`; ein Unit-Test kann die Proxy-Konfiguration nicht ausüben (dokumentierter 3× wiederholter Incident) | `dx serve` starten, `/user_management/` als Admin öffnen, im Netzwerk-Tab prüfen: genau **ein** Aggregat-Request, Status 200, Liste gefüllt |
| Sichtbare Ausprägung (Badge/Spalte) in der gerenderten Liste | PADM-01 | Visuelle Bestätigung, dass verknüpfte, reine Benutzer und reine Verkäufer unterscheidbar sind | `dx serve`, `/user_management/` öffnen, je einen Eintrag pro Ausprägung im DOM sichten |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags (`dx serve --hot-reload` ist Smoke, kein Test-Gate)
- [ ] Clippy-Hard-Gate separat gefahren (nicht Teil von `cargo test`)
- [ ] Feedback latency < 300 s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
