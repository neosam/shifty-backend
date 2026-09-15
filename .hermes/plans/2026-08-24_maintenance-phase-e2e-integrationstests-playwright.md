# Plan: Phase 59 (Maintenance) — E2E-Integrationstests mit ferngesteuertem Browser (Playwright)

Repo: `~/programming/projects/shifty-backend` | Branch: `jj`-Branch `e2e-browser-tests`
Kanban-Task: `t_763a257c` | Läuft parallel zur Feature-Phase 58, kein gegenseitiges Abhängen
Off-theme Maintenance-Phase (eigenes Thema, kein Feature-Milestone) — kein Milestone-Goal,
kein REQUIREMENTS.md-Eintrag; Abnahmemessstab = Scope 1 aus dem Todo.

## Kontext & Ist-Zustand

- CI (`.github/workflows/rust.yml`) ist rein Backend: fmt, Clippy `-D warnings`, Build
  (default + OIDC), `cargo test --workspace`. Kein Browser, kein FE-Job.
- `flake.nix`: `packages.default/backend-mock/backend-oidc` (via `default.nix`),
  `packages.frontend` (WASM-Build), `devShells.default` mit rust/sqlx/nodejs/lld —
  **kein chromium/playwright**.
- FE läuft via `dx serve` (Dioxus-TOML-Dev-Proxy, Präzedenz: fehlende Proxy-Einträge
  → 405, v1.6/v1.8), Backend `cargo run` (mock_auth, Auto-Admin) Port 3000.
- Bekannte Caveats (aus dem Todo): (a) `<input type=date>` programmatisch setzen
  triggert Dioxus-Signale nicht zuverlässig → in Playwright per **Tastatureingabe**
  (locator.click + fill-Tastatursequenz) statt value-Set; (b) CDP-Screenshots können
  einfrieren → Playwright-native `page.screenshot()` (kein html2canvas).
- PO-Entscheidung: **Playwright akzeptiert** (Node-Stack, Ökosystem-Reife); CI separat
  getriggert; Flake-Browser-Packages nur als diff-Vorschlag mit @user-Freigabe.

## Architektur

`e2e/`-Verzeichnis auf Repo-Root (eigenes `package.json` + `playwright.config.ts`,
TypeScript, `@playwright/test`):

```
e2e/
  package.json          # @playwright/test, typescript
  playwright.config.ts  # 2 workers=0 (Serial!), base URL FE :8080, backend :3000
  helpers/
    bootstrap.ts        # Seed-Setup: frische SQLite-DB, mock_auth-Admin, Rollen-User
    auth.ts             # mock_auth-Login, Session-Cookie, Rollen-Switch (Impersonation-Endpoint)
    seed.ts             # deterministische Seeds (Verträge, Absenzen, Urlaubsanspruch,
                        # Paid-Capacity, Shiftplan, Slots) — feste UUIDs/Festwerte
  specs/
    login.spec.ts       # Flow 1
    absence-modal.spec.ts  # Flow 2
    hr-offset-roundtrip.spec.ts  # Flow 3
    paid-limit-409.spec.ts     # Flow 4
  README.md             # lokal + CI-Anleitung
```

Umgebungs-Orchestrierung: `e2e/helpers/bootstrap.ts` startet Backend
(`cargo build` einmalig, Binary mit `SHIFTY_DB_PATH=<tmp>` + `mock_auth`) und
`dx serve` als Child-Processes, wartet auf Health-Endpoints (Backend-`/health` oder
bekannter Root; FE-HTTP-200), killt beide im `globalTeardown`. DB = frische Temp-DB,
`sqlx migrate run`-Semantik via `sqlx database reset`-Script (destructive-Präzedenz:
nur gegen Temp-Datei, nie die Dev-DB).

## Schritte

1. **Playwright-Scaffold + Flake-diff-Vorschlag.** `e2e/`-Struktur wie oben,
   `playwright.config.ts` mit `workers: 0` (Serial — Shared-Backend-DB), `retries: 0`,
   `screenshot: 'only-on-failure'`, Timeout 30s/Step. **Flake-Änderung als diff-Vorschlag
   an @user (nicht still):** `packages.browsers = pkgs.playwright` bzw.
   `devShells.default += [ pkgs.playwright ]` + `PLAYWRIGHT_BROWSERS_PATH`-Hinweis.
   Plan-Regel: Flake-Commit erst nach @user-Freigabe, bis dahin läuft E2E lokal.
   - Verifikation: `cd e2e && npx playwright test --list` listet 0 Specs ohne Crash;
     Flake-diff als Board-Comment mit @user-Freigabe-Anfrage hinterlegt.
2. **Bootstrap-Helper (Env-Orchestrierung).** `bootstrap.ts`: Backend-Start
   (Temp-DB, mock_auth), FE-`dx serve`-Start, Readiness-Polling (max 120s),
   Teardown. `seed.ts`: deterministische Seeds für die 4 Flows (feste Namen/UUIDs,
   `2026` als Test-Jahr, Vertrag 40h/5 Tage).
   - Verifikation: manueller Dry-Run — `npx playwright test --list` + separater
     `npm run bootstrap:check`-Skript zeigt „backend ready / fe ready" und killt sauber.
3. **Flow 1: Login → Navigation.** `login.spec.ts`: mock_auth-Login, Hauptnavigation
   sichtbar, Session persistiert über Reload (Cookie-Präzedenz v1.9 Impersonation-Banner).
   - Verifikation: `npx playwright test login` grün.
4. **Flow 2: Abwesenheit anlegen inkl. Modal-Close.** `absence-modal.spec.ts`:
   Absence-Modal öffnen → Person/Zeitraum → Create → **Modal schließt**
   (der historische Bug!) → Eintrag sichtbar in der Konflikt-/Abwesenheitsliste.
   Date-Fields per Tastatureingabe (Caveat a).
   - Verifikation: `npx playwright test absence-modal` grün; Absence-Create-API-Status 201.
5. **Flow 3: HR-Urlaubs-Offset-Roundtrip.** `hr-offset-roundtrip.spec.ts`: HR-Rolle,
   StatBox „calculated N + Offset", Offset setzen → effektiv N+Offset, persistiert
   (Reload → Wert bleibt, Backend `offset_days` lesbar).
   - Verifikation: `npx playwright test hr-offset` grün.
6. **Flow 4: Paid-Limit-409.** `paid-limit-409.spec.ts`: Admin-Toggle
   `paid_limit_hard_enforcement` an, Shiftplanner bucht über das Limit → HTTP 409 +
   lokalisierte Inline-Meldung sichtbar; Shiftplanner-Bypass bleibt bestehen
   (Zweitfall: als Shiftplanner → 201).
   - Verifikation: `npx playwright test paid-limit` grün.
7. **Rollen-Coverage-Note + Caveats-Doku.** In `README.md`: welche Rolle pro Flow,
   warum mock_auth-Admin-Seed, die beiden Caveats + Workarounds, Laufzeit-Benchmark
   (erwartet < 3–5 min für alle 4 Flows lokal).
   - Verifikation: README vorhanden, `npm run e2e` (alias für `playwright test`)
     läuft komplett grün lokal.
8. **CI-Job (separat getriggert, PO-Freigabe).** `.github/workflows/e2e.yml`:
   eigener Workflow, `workflow_dispatch` + optional `pull_request` auf
   `e2e/**`-Pfade (Path-Trigger hält die Backend-Pipeline unangetastet); Steps:
   checkout, nix-setup bzw. rust+node-Setup, Backend-Build, `npx playwright install
   --with-deps chromium`, `npm run e2e`. **Erst nach Flake-/CI-Freigabe committen**
   (deployments-nah, PO-Regel: diff-Vorschlag zuerst).
   - Verifikation: Workflow einmal per `workflow_dispatch` grün (oder als letzter
     Schritt der Phase nach @user-Freigabe).

## Abhängigkeiten

- 1 → 2 (Struktur + Env) → 3–6 (jeder Flow unabhängig voneinander, parallelisierbar
  nach 2; aber Serial-Execution im Runner). 7 nach 3–6. 8 nach 7 und @user-Freigabe.
- **Keine Abhängigkeit von Phase 58** — die 4 Flows berühren keine
  Werktage-Verteilung; bewusst parallel lauffähig.

## Risiken

- **Flake/CI-Freigabe hängt** → Phase kann lokal komplett fertig werden und verifiziert
  werden; nur Schritt 8 wartet. Kein Blocker für die übrigen Schritte.
- **Dioxus-WASM-Interaktions-Flakiness** (Caveat a, D-25-06-Klasse) →
  `workers: 0` + feste Waits via `waitFor` auf selektierbare Zustände (Text/DOM),
  keine Sleeps; bei Flakiness: Flow als `@flaky`-Tag markieren + Bug-Task, nicht
  schlafen-lassen.
- **`dx serve`-Startzeit im CI** (WASM-Dev-Server) → Readiness-Poll mit 120s-Budget;
  CI-Timeout pro Job 15 min.
- **Determinismus der Seeds** → feste UUIDs + `2026`-Referenzjahr; „heute"-abhängige
  Logik (ISO-Woche) in Tests gegen Injektion/Clock-Mock des Backends prüfen
  (mock_auth-Backend kennt keinen Clock-Override → Seeds relativ zur
  Backend-aktuelen Woche bauen, `bootstrap.ts` ermittelt ISO-Woche per API).
- **Temp-DB-Leaks** → `globalTeardown` killt + löscht; CI-Artefakt-Screenshots
  nur on-failure (Volume-Schutz).

## Nicht enthalten (Scope)

- Kein Playwright für WASM-Unit-Tests (FE-`cargo test` bleibt der FE-Gate).
- Keine visuellen Regressionstests (Screenshot-Pixel-Vergleich) — Screenshot nur
  als Failure-Artefakt.
- Kein OIDC-Lauf (mock_auth reicht für Rollen-Coverage; OIDC-e2e wäre eigene Phase).
- Kein FE-Performance-/Load-Testing.
