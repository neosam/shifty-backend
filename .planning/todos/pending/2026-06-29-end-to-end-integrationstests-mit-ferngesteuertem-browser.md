---
created: 2026-06-29T12:48:02.144Z
title: End-to-End Integrationstests mit ferngesteuertem Browser
area: testing
files:
  - shifty-dioxus/
  - rest/
  - flake.nix
---

## Problem

Es fehlen echte End-to-End-Integrationstests: ein Setup, in dem **Backend und Frontend
zusammen laufen** und über einen **ferngesteuerten Browser** automatisiert geprüft wird,
ob die Features tatsächlich funktionieren.

Heute decken wir ab:
- Backend: `cargo test --workspace` (Unit + In-Memory-SQLite-Integration auf Service-/REST-Ebene).
- Frontend: `cargo test` + WASM-Build-Gate.
- e2e/Browser nur **manuell** (Live-HR-Browser-Smokes via claude-in-chrome, ad hoc pro Phase).

Die Lücke ist genau der Pfad, der wiederholt erst im manuellen Smoke aufschlug und nicht von
grünen Unit-Tests gefangen wurde — z.B. fehlende `Dioxus.toml`-Dev-Proxy-Einträge
(`/toggle` in v1.6, `/vacation-entitlement-offset` in v1.8 → FE-Save lief auf 405) oder
das AbsenceModal, das nach sauberem Create/Update nicht schloss. Ein automatisierter
Full-Stack-e2e-Lauf würde solche Regressionen vor dem manuellen UAT abfangen.

## Solution

TBD — vor Planung zu klären:

- **Test-Runner / Browser-Treiber:** Playwright (eigener Node-Stack) vs. ein Rust-natives
  CDP/WebDriver-Harness vs. Wiederverwendung der bestehenden claude-in-chrome-Mechanik
  in skriptbarer Form. Trade-off: Ökosystem-Reife (Playwright) vs. „kein zweiter
  Toolchain"-Konsistenz (Rust).
- **Orchestrierung der Umgebung:** Backend (port 3000) + Frontend (`dx serve`, port 8080)
  + frische/seedbare Test-DB deterministisch hochfahren und nach dem Lauf abräumen.
  Reproduzierbar via `flake.nix`/Nix-Shell; bestehender Helfer:
  Skill `run-rust-backend-and-frontend` + `sqlx database reset`.
- **Auth im Test:** `mock_auth`-Feature nutzen (Auto-Admin) und gezielt Rollen
  (HR/Sales/Admin/Shiftplanner) durchspielen — viele Bugs waren rollen-/permission-spezifisch.
- **Daten-Setup:** deterministische Seeds für die getesteten Flows (Abwesenheiten,
  Urlaubsanspruch, Paid-Capacity, Shiftplan-Buchung).
- **Scope erster Wurf:** die kritischen, wiederholt manuell gesmokten Flows zuerst
  (Login → Navigation; Abwesenheit anlegen inkl. Modal-Close; HR-Urlaubs-Offset-Roundtrip;
  Paid-Limit-409). Nicht alles auf einmal.
- **CI-Integration:** als separates, optional getriggertes Gate (WASM-/Browser-Lauf ist
  teurer als `cargo test`); klären, ob/wie es in `.github/workflows/rust.yml` bzw. den
  `nix build` passt, ohne die schnelle Backend-Pipeline zu verlangsamen.
- **Bekannte Caveats (Memory):** programmatisches Setzen von `<input type=date>` triggert
  Dioxus-Signale nicht zuverlässig; CDP-Screenshots können einfrieren (html2canvas-Workaround) —
  beim Tooling-Entscheid berücksichtigen.

Eigenes Infrastruktur-/Tooling-Thema (off-theme zu Feature-Milestones) — Kandidat für eine
eigene Maintenance-Phase oder `/gsd-new-milestone`.
