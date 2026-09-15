# Plan: Phase 59 (Maintenance) — E2E-Browser-Test-Infrastruktur (Playwright)

Repo: `~/programming/projects/shifty-backend` | VCS: `jj` (co-located git) | Branch: `e2e-browser-tests`
Kanban-Task: `t_f6daeb7a` | Parent: `t_763a257c` (PO-Analyse, Comment 18)
Off-theme **Maintenance-Phase** — kein Milestone-Goal, kein `REQUIREMENTS.md`-Eintrag.
Abnahmemessstab (PO-Festlegung 24.08./25.08.): P0 + P1 + die vier Scope-1-Specs (P2) + P3 (optional). **Mehr nicht.**

> Alle Fakten unten wurden am 25.08.2026 im Code verifiziert (Datei:Zeile), nicht aus Memory.
> Abweichungen von der PO-Analyse (Comment 18) sind mit **⚠️ KORREKTUR** markiert.

## Kontext & Ist-Zustand

- **CI** (`.github/workflows/rust.yml`): rein Backend — `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, Build (default + OIDC `--no-default-features --features oidc,json_logging`), `cargo test --workspace`. Trigger: `push` + `pull_request` auf `main`/`feature/**`. **Kein Browser, kein FE-Job.**
- **`flake.nix`**: `packages.frontend` (dioxus-cli, WASM-Build via `sourceRoot=shifty-dioxus`), `devShells.default` mit `rustc/cargo/sqlx-cli/sqlite/nodejs/lld`. **Kein chromium/playwright.** `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER=lld`.
- **Testpfad MUSS `dx serve` sein** (Port 8080, Dioxus-Dev-Proxy). Die wiederkehrende Bug-Klasse = fehlender `[[web.proxy]]`-Eintrag → FE-Save 405 (v1.6/v1.8 Präzedenz). `dx build` + statischer Server würde genau diesen Proxy-Pfad *wegtesten*.
- **Backend-Boot**: `shifty_bin` Default-Features (`mock_auth`, `local_logging`), Port 3000. DB-Pfad `sqlite:./localdb.sqlite3` ist **cwd-relativ** → frisches Temp-Dir pro Lauf = deterministisch; Migrations laufen beim Start automatisch. Readiness: `GET :3000/version` (`rest/src/lib.rs:678`).
- **mock_auth** (`rest/src/session.rs:314-317`): jede Anfrage ohne Cookie → auto-DEVUSER-Session. `main.rs:1657` legt `DEVUSER` an (admin), `main.rs:1658` zusätzlich `admin`. **Keine Login-UI** — `NotAuthenticated` (`shifty-dioxus/src/page/not_authenticated.rs`) rendert nur, wenn `/auth-info` ≠ 200.
- **`shifty-dioxus/assets/config.json`** ist committed: `{"backend":"http://localhost:8080","show_my_time":true,"is_prod":false,"env_short_description":"LOCAL","show_absences":true}`. FE spricht also Port 8080 (dx-serve), der via Proxy auf 3000 weiterleitet. `show_absences=true` → Absences-Menüpunkt ohne Konfig-Änderung sichtbar. **E2E darf diese Datei NICHT modifizieren.**
- **Ports 3000/8080 hart verdrahtet** (Proxy-Einträge + config.json) → kein paralleles E2E auf derselben Maschine. CI-Runner sind isoliert.
- **`POST /dev/seed`** (`rest/src/dev.rs`, nur unter `mock_auth`): seedet `sales_persons`, `work_details`, `extra_hours`, `bookings`, `special_days` — **additiv** (2× Seed = doppelte Personen, verifiziert per `test_seed_twice_is_additive`). **Seedet KEINE User.** → immer frische DB.
- **VCS/Hygiene**: Repo via `jj` (co-located git), GSD-Auto-Commit aktiv (`commit_docs:true`, `tdd_mode:true`, `code_review:true`, `verifier:true`). Root-`Cargo.toml`: `exclude=["shifty-dioxus"]` — **`e2e` muss ergänzt werden.**
- **Snapshot-Schema-Version**: aktuell **13** (D-57-10, Phase 57). Phase 59 ist eine Test-Infrastruktur-Phase — **kein Snapshot-Bump, keine Migration, keine Cargo-Deps im Backend-Workspace** (e2e/ ist Node/TS, excluded).
- **Existiert bereits**: `.planning/phases/59-e2e-browser-testinfra/` (leer), Origin-Todo `.planning/todos/pending/2026-06-29-end-to-end-integrationstests-mit-ferngesteuertem-browser.md`, Entwurf `.hermes/plans/2026-08-24_maintenance-phase-e2e-integrationstests-playwright.md`.

### ⚠️ KORREKTUR 1 — P0-OpenAPI-Quelle ist stale
Die PO-Analyse (Comment 18 §2.1) verweist auf `shifty-dioxus/openapi.json` als "reale Backend-Routen". **Das ist ein Stale-Snapshot:** committed, 41950 B, mtime 2026-07-22, ~45 Pfade — es fehlen u. a. `settings`, `admin/cutover`, `dev`, `vacation-entitlement-offset`, `week-status`, `pdf-export-config`, `feature-flag`. Gegen diese Datei getestet, würde der P0-Test für existierende Routen "fehlend" melden (False-Positives).

**Authoritative Quelle = die LIVE utoipa-Definition** `ApiDoc` (`rest/src/lib.rs:600`, `#[derive(OpenApi)]`, ~29 Nests, `pub struct ApiDoc`, Methode `openapi()` ist **synchron + in-memory** — kein Server, kein Browser nötig). Der P0-Test ruft `ApiDoc::openapi()` direkt auf. (Die stale committed-Datei wird als separates Hygiene-Follow-up notiert, nicht P0-Scope.)

### ⚠️ KORREKTUR 2 — Paid-Limit-409: Admin/Shiftplanner bypassen
`service_impl/src/shiftplan_edit.rs:641`: `if hard && !is_shiftplanner { … block … }`, mit `is_shiftplanner = sp_perm.is_ok()` (`:595`) und `sp_perm = check_permission("shiftplanner")`. **Admin erhält alle Privilegien via admin-auto-grant-Trigger** (`:587`) → `is_shiftplanner=true` → Bypass. Ebenso jeder Shiftplanner. **Der 409-Test läuft daher NUR mit einem User ohne `shiftplanner`-Privileg** — praktisch ein reiner `sales`-User, der sich **selbst** bucht (Permission `Shiftplanner ∨ self`, D-24-04, `:585-595`; Self-Grant via `verify_user_is_sales_person`).

## Architektur (e2e/-Workspace, Node/TS, aus Cargo-Workspace excluded)

```
e2e/
  package.json            # @playwright/test, typescript
  playwright.config.ts    # workers:1 (SERIAL — shared Backend+DB), retries:0,
                          #   base_url http://localhost:8080, backend :3000,
                          #   screenshot:'only-on-failure', trace:'on-first-retry',
                          #   timeout 30s, globalSetup/Teardown
  tsconfig.json
  fixtures/
    roles.sql             # deterministische User/Rolle-Fixture (s. P2-Spec 3/4)
    paid_limit.sql        # Slot.max_paid_employees + paid-bookings (s. P2-Spec 4)
    apply.sh              # sqlite3-Fixture-Apply + /dev/seed-Trigger
  helpers/
    bootstrap.mjs         # Env-Orchestrierung: Temp-Dir → Fixture → Backend →
                          #   Seed → dx serve → Warmup → (return handles)
    teardown.mjs          # kill Backend + dx serve, Temp-Dir rekursiv leeren/entfernen
    api.mjs               # Playwright request-Client: /auth-info, /admin/impersonate/*,
                          #   /dev/seed, /toggle/*, /absence-period, /vacation-entitlement-offset
  specs/
    app-boot.spec.ts          # P2-Spec 1
    absence-modal.spec.ts     # P2-Spec 2
    hr-offset-roundtrip.spec.ts  # P2-Spec 3
    paid-limit-409.spec.ts    # P2-Spec 4
  run.sh                # Einstieg: bootstrap → npx playwright test → teardown (trap)
  README.md             # lokal + CI-Anleitung, Rolle pro Spec, Caveats, Laufzeit
```

**Umgebungs-Orchestrierung (`run.sh` + `helpers/bootstrap.mjs`):**
1. `TMPDIR=$(mktemp -d)` (frische, deterministische DB-Location).
2. Fixture-Apply: `sqlite3 $TMPDIR/localdb.sqlite3 < fixtures/roles.sql` (+ `paid_limit.sql` wo nötig) **vor** Backend-Start (Migrations laufen beim Start und erzeugen die Tabellen; Fixture fügt User/Rollen/Slots hinzu).
3. Backend-Start: `cd $TMPDIR && cargo run -p shifty_bin` (Default-Features `mock_auth,local_logging`), `cwd=$TMPDIR` → `localdb.sqlite3` dort. Readiness-Poll `GET :3000/version` (max 120s; Cargo-First-Run-Budget).
4. Seed: `POST :3000/dev/seed` (additiv → Basisdaten).
5. FE-Start: `cd shifty-dioxus && dx serve` (Port 8080). **Warmup:** erster Request triggert WASM-Compile (minutenlang) → zweites `GET :8080` bis 200 (max 300s).
6. `npx playwright test` (serial).
7. **`trap`-Teardown** (auch bei Failure/Ctrl-C): `DELETE /admin/impersonate/` (Impersonation-Stop vor Teardown — P10-Kante), kill `shifty_bin` + `dx serve`, Temp-Dir rekursiv entfernen, Playwright-Artifacts (trace/screenshot) auf Failure nach `e2e/artifacts/` sichern.

**Test-Disziplinen (verbindlich):**
- **P10-Impersonation-Kante** (D-32-02a): während Impersonation eines Nicht-Admins ist der effektive Context Nicht-Admin → admin-only-Endpoints 403. **Kein Admin-Call während Impersonation; `DELETE /admin/impersonate/` (Stop) VOR Teardown.**
- **Date-Inputs:** Playwright *tastet* in `<input type=date>` (echte Keyboard-Events → Dioxus-Signale triggern). **Niemals** `.fill()`/`value`-Set auf Date-Feldern (Caveat a, D-25-06-Klasse).
- **Serial** (`workers:1`): shared Backend + eine DB. Keine Parallelisierung.
- **Waits auf selektierbare Zustände** (Text/DOM/Response), keine blauen `sleep`. Bei Flakiness: Spec `@flaky`-taggen + Bug-Task, nicht schlafen-lassen.
- **Screenshots** via Playwright `page.screenshot()` (CDP) — kein html2canvas-Pfad (Freeze-Caveat entfällt).

## P0 — Proxy-Coverage-Test (kein Browser, reines Rust, **eigene Task**)

**Ziel:** Die 405-Bug-Klasse (fehlender `[[web.proxy]]`-Eintrag) *statisch* fangen — größte Value-per-Effort, sofort baubar, unabhängig vom Browser-E2E. Lädt in `cargo test --workspace`.

**Crate:** `rest` (hat bereits `rest/tests/`). Datei: `rest/tests/proxy_coverage.rs`.

**Quellen im Test:**
- **Proxy-Präfixe**: parse `shifty-dioxus/Dioxus.toml` → die 36 `[[web.proxy]]`-Blöcke (`backend = "http://localhost:3000/<prefix>"`) → Präfix `<prefix>` extrahieren. (TOML-Crate ist bereits im Workspace; falls nicht: `toml` als dev-dep von `rest`.)
- **Live-OpenAPI**: `rest::ApiDoc::openapi()` (in-memory utoipa, synchron) → `openapi.paths` Keys.
- **FE-Aufrufe**: parse `shifty-dioxus/src/api.rs` → aufgerufene Präfixe. api.rs baut URLs als `format!("{}/<path>", backend_url)` und `format!("{}<path>", config.backend)` → Regex auf `{}/([a-zA-Z0-9_/-]+)` und `{}\(?[a-zA-Z0-9_/-]`.

**Assertions:**
1. **(ii, primär, robust, kein False-Positive) FE⊆Proxy:** Jedes in `api.rs` aufgerufene Präfix hat einen `[[web.proxy]]`-Eintrag. Fängt exakt die 405-Klasse (neuer FE-Endpunkt ohne Proxy-Eintrag).
2. **(i, secondary, Value) Proxy⊆(OpenAPI ∪ Allowlist):** Jedes Proxy-Präfix taucht in der Live-utoipa-OpenAPI als Präfix eines Pfads auf **oder** steht in einer kuratierten `PROVEN_DIRECT_ROUTES`-Allowlist (Pfade, die in `rest/src/lib.rs` per `app.route()`/separatem Nest geroutet sind aber nicht im utoipa-ApiDoc-Nest). **Initiale Allowlist (verifiziert):** `authenticate`, `auth-info`, `version` (direkt `app.route()`, `lib.rs:644/677/678`); dazu prüfen+kurieren bei First-Run: `working-hours`, `booking-information`, `slot`, `rebooking-suggestions`, `admin/cutover`, `booking`, `shiftplan`, `blocks`, `toggle-group` (alle Proxy-Präfixe die kein exaktes ApiDoc-Nest tragen). Die Allowlist ist *bewusst* kuratiert — jeder neue Eintrag braucht einen Kommentar (warum nicht in ApiDoc). **Neue Lücken ohne Allowlist-Eintrag = Test-Fail.**

**⚠️ NICHT gegen die stale `shifty-dioxus/openapi.json` testen** (Korrektur 1). `ApiDoc::openapi()` ist die Quelle.

**Akzeptanzkriterien P0:**
- `cargo test -p rest --test proxy_coverage` grün.
- Test 1 (FE⊆Proxy) + Test 2 (Proxy⊆OpenAPI∪Allowlist) beide aktiv.
- Bewusst-Fail-Proof: einen Proxy-Eintrag temporär entfernen → Test 1 rot (405-Fang belegt). (Im Plan als Verifikationsschritt, nicht committeter Test.)
- `cargo clippy --workspace -- -D warnings` grün (Test-Code einbezogen).
- Stale `shifty-dioxus/openapi.json` als **Follow-up-Todo** notiert (`.planning/todos/pending/`), nicht in P0 gefixt.

## P1 — E2E-Infrastruktur (e2e/-Workspace + Runner)

### Schritte

**P1-1: Scaffold.** `e2e/` anlegen (Struktur s. oben). `package.json` (`@playwright/test`, `typescript`), `playwright.config.ts` (`workers:1`, `retries:0`, `baseURL:'http://localhost:8080'`, `timeout:30000`, `screenshot:'only-on-failure'`, `trace:'on-first-retry'`, `outputDir:'artifacts'`), `tsconfig.json`. Root-`Cargo.toml`: `exclude=["shifty-dioxus","e2e"]`. Root-`.gitignore`: `e2e/node_modules/`, `e2e/artifacts/`, `e2e/test-results/`.
- **Verifikation:** `cargo metadata --no-deps` zeigt `e2e` im `exclude`-Array (Cargo ignoriert e2e, kein Workspace-Fehler, kein neuer Crate-Mitglied); `grep -n '"e2e"' Cargo.toml` → 1 Treffer; `cd e2e && npx playwright test --list` → 0 Specs, kein Crash; `cargo clippy --workspace -- -D warnings` bleibt grün (e2e nicht im Workspace).

**P1-2: Bootstrap-Helper (Env-Orchestrierung).** `helpers/bootstrap.mjs` + `helpers/teardown.mjs` + `run.sh` (Orchestrierung s. oben, 7 Schritte). `helpers/api.mjs`: Playwright `APIRequestContext`-Wrapper um Backend-Endpoints (`authInfo()`, `impersonateStart(name)`, `impersonateStop()`, `impersonateStatus()`, `seed()`, `toggleEnable(name)`/`toggleDisable(name)`, `absencePeriods()`, `entitlementOffsetGet(year, spId)`). Ports 3000/8080 hart. **Readiness-Polling** statt fester Waits.
- **Verifikation:** manuelles Dry-Run `./e2e/run.sh --bootstrap-only` (Flag nur für Dev) zeigt „backend ready (version=…) / fe ready (8080=200)“ und beendet beide Prozesse + leert das Temp-Dir sauber. Orphan-Check: danach muss keine shifty_bin- und keine dx-serve-Prozess-Instanz mehr laufen (per Prozessnamens-Liste prüfen).

**P1-3: Flake-/Toolchain-Änderung — NUR als diff-Vorschlag mit User-Freigabe (PO-Festlegung, NICHT still committen).** Vorschlag für `flake.nix` `devShells.default`: `playwright` (oder `+ pkgs.playwright-browsers`) + `PLAYWRIGHT_BROWSERS_PATH`-Hinweis; plus Hinweis `npx playwright install chromium --with-deps` (lokale Alternative ohne Flake). Vorschlag als **Board-Comment/PR-Diff** an User; System-Ebene (Browser-Installation auf dem Host) = @sysadmin.
- **Verifikation:** diff-Vorschlag liegt als dokumentierter Block in `e2e/README.md` + Board-Comment mit @user-Freigabe-Anfrage; **kein Flake-Commit** ohne Freigabe. Bis dahin: E2E läuft lokal mit bereits installiertem chromium/Playwright (devShell hat `nodejs`).

**Akzeptanzkriterien P1:**
- `./e2e/run.sh` bootet kompletten Stack (Backend :3000 + dx serve :8080) aus einem frischen Temp-Dir, läuft einen Dummy-Playwright-Spec (z. B. `test('boot', async ({request}) => { const r = await request.get('/version'); expect(r.status()).toBe(200); })`) grün, reißt alles sauber wieder ab.
- Determinismus: zwei aufeinanderfolgende Läufe liefern identische Ergebnisse (frische DB, keine Leaks).
- `rust.yml` unverändert; Root-Workspace + .gitignore aktualisiert; Flake nur als Vorschlag.

## P2 — Die vier Scope-1-Specs (Abnahmemessstab, mehr NICHT)

Jede Spec: **eigene Fixture** (`fixtures/*.sql`, deterministisch, feste Namen/Festwerte, relativ zur aktuellen ISO-Woche des Backends gebaut — mock_auth-Backend hat keinen Clock-Override) + **eigene spec.ts** + eigener Testlauf (serial).

### P2-Spec 1 — App-Boot (ex-taktisch „Login“)
**Ist-Zustand:** keine Login-UI unter mock_auth; `NotAuthenticated` rendert nur bei `/auth-info` ≠ 200.
**Fixtures:** keine (DEVUSER wird beim Backend-Start angelegt, `main.rs:1657`; auto-Session pro Cookie-less-Request, `session.rs:314`).
**Flow:** `page.goto('http://localhost:8080/')` → assert: (a) `NotAuthenticated`-Marker ABSENT (kein `h1` mit `Key::WelcomeTitle`-Text, kein `a[href=/authenticate]`), (b) Hauptnavigation + Home-Content PRESENT, (c) `GET :3000/auth-info` (via `api.mjs`) → 200 + `username` gesetzt.
**Verifikation:** `npx playwright test app-boot` grün.

### P2-Spec 2 — Absence-Modal-Close nach Create (historischer Bug)
**Ist-Zustand:** `AbsenceModal` (`src/page/absences.rs:1043+`, props `sales_persons`, `mode: Create|Edit`), `AbsenceModalEvent::Created(result)` → `:1148` setzt Modal-Result → Close. Person-Dropdown + From/To-Datepicker (`<input type=date>`, D-05).
**Rolle:** HR (Impersonation, s. Disziplin) — Absence-Create ist `HR_PRIVILEGE`-gated (`service_impl/src/absence.rs:94-95`, `111-112`). DEVUSER(Admin) ginge auch via admin-auto-grant; **Rolle für die Spec: HR-User** (realistischer, übt die eigentliche Gate-Logik) — Impersonation auf `hr-test-user` (s. Fixture).
**Fixtures:** `fixtures/roles.sql` — User `hr-test-user` (Rolle `hr`) + SalesPerson `hr-target-sp` (via `sales_person` + `sales_person_user`, `is_paid=1`), verknüpft; deterministische UUIDs/Festwerte.
**Flow:** Impersonation `hr-test-user` (api.mjs) → `page.goto('/absences/')` → reload (Rolle wirksam) → „Neue Abwesenheit“-Button → Person-Dropdown `hr-target-sp` wählen → From/To per **Tastatureingabe** (Caveat a) → Create → assert: (a) **Modal ist geschlossen** (Modal-Root aus DOM weg), (b) neuer Eintrag in der Abwesenheitsliste sichtbar (Name + Datum), (c) Backend-Verifikation `GET :3000/absence-period` (api.mjs) enthält den Eintrag (Create→201).
**Verifikation:** `npx playwright test absence-modal` grün; Absence-Create-Status 201 (via request-Client-Interception oder api.mjs-Readback).

### P2-Spec 3 — HR-Offset-Roundtrip (Impersonation HR → Offset setzen → Reload → persistiert)
**Ist-Zustand:** HR-gated CRUD `/vacation-entitlement-offset` (`rest/src/lib.rs:700-701`, `rest/src/vacation_entitlement_offset.rs:3`), FE-Editor in `src/page/absences.rs` (HR-StatBox „calculated N + Offset [n]“ — v1.8/v2.1 Präzedenz; `loader.rs`/`service/vacation_balance.rs` laden). **Diese Spec fängt genau den v1.8-Proxy-405 (`/vacation-entitlement-offset` fehlte in Dioxus.toml — heute vorhanden, Präfix `vacation-entitlement-offset` in der verifizierten 36er-Liste).**
**Rolle:** HR (Impersonation `hr-test-user`).
**Fixtures:** wie Spec 2 (`hr-test-user` + `hr-target-sp` mit Work-Details, damit Urlaubsanspruch calculated > 0).
**Flow:** Impersonation HR → `/absences/` (bzw. Employee-Detail-Route `#/employees/<sp_id>/` wo die StatBox rendert — Coder verortet die Offset-UI per grep `entitlement-offset` in `src/page/`) → Offset-Feld per Tastatur/Number-Input setzen → Save → `page.reload()` → assert: (a) Offset-Wert **nach Reload** im Feld/StatBox persistiert, (b) Backend `GET :3000/vacation-entitlement-offset?year=…&sales_person_id=…` (api.mjs) → `offset_days` = gesetzter Wert.
**Verifikation:** `npx playwright test hr-offset` grün.

### P2-Spec 4 — Paid-Limit-409 (Booking über Paid-Capacity)
**⚠️ KORREKTUR 2:** 409-Trigger **nur für Nicht-Shiftplanner** (`shiftplan_edit.rs:641`). DEVUSER(Admin) + jeder Shiftplanner **bypassen** (`:587,595`).
**Mechanik:** `POST /shiftplan-edit/booking` (`rest/src/shiftplan_edit.rs:32-35`) → Service `book_slot_with_conflict_check` (`service_impl/src/shiftplan_edit.rs:575+`): Permission `Shiftplanner ∨ self` (`:585-595`, D-24-04) → Wochen-Sperre-Gate (`:608`) → Paid-Limit-Check (`:625-657`): `slot.max_paid_employees = Some(max)` + Toggle `paid_limit_hard_enforcement` ON + `!is_shiftplanner` + `prospective > max` → `ServiceError::PaidLimitExceeded` → **HTTP 409** (`rest/src/lib.rs:265-269`). Toggle-Default ist **OFF/soft** (Migration `20260627000000_seed-paid-limit-toggle.sql`: `0`) → Test **muss** Toggle explizit aktivieren: `PUT :3000/toggle/paid_limit_hard_enforcement/enable` (api.mjs, admin — DEVUSER **vor** Impersonation!).
**Rolle:** reiner `sales`-User **ohne** `shiftplanner`-Privileg, bucht **sich selbst** (Self-Grant via `sales_person_user`).
**Fixtures:** `fixtures/paid_limit.sql` — (1) User `paid-test-user` (Rolle `sales` NUR), (2) SalesPerson `paid-target-sp` (`is_paid=1`) + `sales_person_user(paid-target-sp, paid-test-user)`, (3) Slot mit `max_paid_employees=1` (via `PUT /shiftplan-edit/slot/{year}/{week}` als Admin **vor** Impersonation, oder direkt in der Fixture-SQL auf die `slot`-Tabelle — Coder entscheidet nach Slot-Schema; **empfohlen: via API**, hält Fixture klein), (4) optional: eine bestehende paid-Booking in (slot, year, week) so dass `existing_paid=1` → `prospective=2 > 1` → 409.
**Flow:** (1) als DEVUSER: Toggle `paid_limit_hard_enforcement` enable (api.mjs) + Slot mit `max_paid_employees=1` setzen. (2) Impersonation `paid-test-user` → `/shiftplan/` → reload. (3) Slot auswählen, `paid-target-sp` (Selbst) buchen → FE-Response 409 → assert: (a) **Inline-Block-Banner an der Slot-Zelle** (`block_error.set(Some(slot_id))`, `src/page/shiftplan.rs:590-592`, D-24-05) sichtbar — **kein** globaler Error-Dialog, (b) `api.mjs`: `POST :3000/shiftplan-edit/booking` (direkt, gleicher Body) → Status 409 + Body enthält PaidLimitExceeded, (c) **Negativ-Proof:** nach `PUT /toggle/paid_limit_hard_enforcement/disable` (als Admin — erst Impersonation stoppen!) dieselbe Buchung → 201 (soft: nur Warning, keine Blockade). **P10:** Impersonation-Stop VOR jedem Admin-Call + vor Teardown.
**Verifikation:** `npx playwright test paid-limit` grün (409 + Inline-Banner + 201-nach-Deaktivierung).

### Akzeptanzkriterien P2 (Gesamt)
- Alle vier Specs einzeln grün: `npx playwright test <spec>`.
- **Kompletter Scope-1-Lauf** `./e2e/run.sh` (alle 4 Specs serial) grün, Laufzeit-Benchmark in README (erwartet < 3–5 min inkl. Warmup).
- Jede Spec hat eigene Fixture; deterministisch (feste Namen, aktuelle ISO-Woche via API ermittelt).
- P10-Disziplin in Spec 3/4 eingehalten (Impersonation-Stop vor Admin-Call/Teardown) — per Code-Review + Teardown-Log.
- Caveats (Date-Input-Tastatur, CDP-Screenshot) in README + als Kommentare an den Use-Stellen dokumentiert.

## P3 — optionales CI-Gate (nach User-Freigabe)

**Eigene Workflow-Datei** `.github/workflows/e2e.yml` — **`rust.yml` bleibt UNANGEGRIFFEN** (PO-Festlegung).
- Trigger: `workflow_dispatch` (manuell) + optional `schedule` (z. B. wöchentlich). **Kein `pull_request`-Trigger** (CI-Kosten bewusst: WASM-Build; PO: „für PRs nicht“).
- Steps: checkout → rust+node-Setup → Backend-Build (`cargo build -p shifty_bin`) → `cd e2e && npx playwright install chromium --with-deps` → `./run.sh`.
- **Offen (in P3 messen, NICHT jetzt festnageln — PO-Offener Punkt 1):** FE-Build-Strategie in CI — `dx serve` (dioxus-cli via `cargo install`) vs. Nix-Action. CI-Runner ist ubuntu-latest (kein Nix by default) → dioxus-cli via `cargo install dioxus-cli` ist der wahrscheinliche Pfad; Nix-Setup (`nix build`/`setup-nix`-Action) als Alternative. Entscheidung nach First-Run-Messung (Build-Zeit, Zuverlässigkeit).
- **Erst nach Flake-/CI-Freigabe committen** (PO-Regel: diff-Vorschlag zuerst).
- **Verifikation:** Workflow einmal per `workflow_dispatch` grün.

## Abhängigkeiten

```
P1-1 (Scaffold) → P1-2 (Bootstrap) → P1-3 (Flake-Vorschlag, parallel möglich, nicht blockierend)
P1-2 → P2-1..P2-4 (alle Specs brauchen den Runner; untereinander unabhängig, Serial-Execution)
P2-1..P2-4 → P3 (CI braucht stabile lokale Specs)
P0: VOLLSTÄNDIG UNABHÄNGIG (kein Browser, reines Rust) — parallel zu P1 startbar
```
- **Keine Abhängigkeit von Phase 58** (Feature) — die 4 Scope-1-Flows berühren keine Werktage-Verteilung.
- **P2-3/P2-4 teilen sich `fixtures/roles.sql`** (hr-test-user, paid-test-user) → in P1-2 als gemeinsames Fixture anlegen, nicht pro Spec duplizieren.

## Risiken

| Risiko | Auswirkung | Gegenmaßnahme im Plan |
|---|---|---|
| **Dioxus-WASM-Interaktions-Flakiness** (Caveat a, D-25-06-Klasse: Date-Input, Modal-Timing) | Specs rot, aber Feature korrekt | `workers:1` + Waits auf selektierbare Zustände (keine Sleeps); Date-Input per Tastatur; bei Flakiness: `@flaky`-Tag + Bug-Task, nicht schlafen-lassen |
| **`dx serve`-WASM-Compile-First-Run** (minutenlang) | Readiness-Timeout | Warmup-Schritt (2. Request), 300s-Budget; CI-Timeout 15 min |
| **Determinismus der Seeds** (ISO-Woche-Abhängigkeit, „heute“-Logik) | Flaky auf Wochen-/Jahresgrenzen | Seeds relativ zur **aktuellen** ISO-Woche des Backends (per API ermittelt), feste Namen/UUIDs; `2026` als Referenzjahr wo möglich |
| **Temp-DB-Prozess-Leaks** (orphan shifty_bin/dx serve) | Port-Konflikte bei Folge-Läufen | `trap`-Teardown (kill + Temp-Dir-Leerung) auch bei Failure/Ctrl-C; Orphan-Check per Prozessnamens-Liste im Dry-Run |
| **P10-Impersonation-Kante** (admin-Endpoint während Impersonation → 403) | Spec 3/4 rot in Teardown | Test-Disziplin: kein Admin-Call während Impersonation; Stop VOR Teardown (explizit in Spec 3/4 + README) |
| **Flake-/CI-Freigabe hängt** | P3 wartet | Phase lokal komplett fertig + verifiziert; nur P3 wartet. Kein Blocker für P0/P1/P2 |
| **P0-False-Positives** (stale openapi.json / nicht-ApiDoc-Routen) | P0-Test meldet „fehlende“ Routen, die existieren | **Korrektur 1:** gegen LIVE `ApiDoc::openapi()` testen, nicht gegen die stale committed-Datei; kuratierte Allowlist mit Begründung pro Eintrag |
| **Paid-Limit-Test mit falscher Rolle** (Admin/Shiftplanner bypassen) | 409 tritt nicht auf → Spec green-wrong | **Korrektur 2:** reiner sales-User ohne shiftplanner, Selbst-Buchung; Negativ-Proof (201 nach Toggle-Deaktivierung) belegt den Mechanismus |
| **e2e/ verschlechtert Root-Workspace-Build** (Cargo erkennt e2e als Mitglied) | `cargo build --workspace` bricht / zieht Node-Abhängigkeiten | `exclude=["shifty-dioxus","e2e"]` in P1-1 als erstes; Verifikation via `cargo metadata` |

## Nicht enthalten (Scope) — PO-Festlegung „mehr nicht“

- **Keine Shiftplan-Buchungs-Flows** jenseits von Paid-Limit-409 (Copy-Week, Slot-Edit, Vacation-Add).
- **Kein i18n-Wechsel** (de/en/cs-Wechsel-Flow) — App läuft mit Default-Locale.
- **Keine Reporting-Views** (Weekly-Overview, Employee-Report, PDF-Export).
- **Keine OIDC-E2E** (mock_auth reicht für Rollen-Coverage; OIDC wäre eigene Phase).
- **Kein Playwright für WASM-Unit-Tests** (FE-`cargo test` bleibt der FE-Gate, 787+ Tests).
- **Keine visuellen Regressionstests** (Screenshot-Pixel-Vergleich) — Screenshot nur als Failure-Artefakt.
- **Kein FE-Performance-/Load-Testing.**
- **Keine Migration, kein Snapshot-Bump, keine Backend-Cargo-Deps** (außer `toml` als dev-dep von `rest` für P0, falls nötig).
- Scope-Ausdehnung erst nach **1–2 Wochen stabiler Scope-1-Tests** (PO-Offener Punkt 6).

## Operator Next Steps (nach Plan-Freigabe)

1. **P0-Task anlegen** (separat, parallel startbar): Kanban-Task für `rest/tests/proxy_coverage.rs` (s. P0-Sektion) — klein, Stunden, kein Browser.
2. **Phase 59 P1 starten** (`/gsd-execute-phase 59` oder manuell): P1-1 → P1-2 → (P1-3 parallel) → P2-1..P2-4 → P3 (nach Freigabe).
3. **Flake-/CI-Freigabe einholen** (User, via Board-Comment/PR-Diff) — blockiert nur P1-3-Commit + P3.
4. **@sysadmin** für Browser-Installation auf dem Host (System-Ebene, PO-Festlegung).
