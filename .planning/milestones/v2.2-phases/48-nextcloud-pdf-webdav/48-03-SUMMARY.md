---
phase: 48-nextcloud-pdf-webdav
plan: 03
subsystem: service_impl
status: complete
tags:
  - webdav
  - nextcloud
  - retry
  - tdd
  - security
requirements:
  - EXP-01
  - EXP-03 (retry portion)
dependency_graph:
  requires: []
  provides:
    - "service_impl::webdav_client::WebDavClient (pub API for 48-04 scheduler)"
    - "service_impl::webdav_client::WebDavError (Transient/Permanent/Io variants)"
  affects:
    - service_impl (new prod deps: reqwest_dav, reqwest, base64, thiserror; new dev-dep: wiremock)
tech-stack:
  added:
    - "reqwest_dav 0.3 (WebDAV verbs; default-features=false, features=[rustls-tls])"
    - "reqwest 0.12 (used directly for MKCOL/PUT; default-features=false, features=[rustls-tls, http2])"
    - "base64 0.22 (Basic-Auth header encoding)"
    - "thiserror 1 (WebDavError enum derivation)"
    - "wiremock 0.6 (dev-dependency, HTTP mock server for tests)"
  patterns:
    - "In-run exponential-backoff retry (2s/4s/8s, capped at 3 attempts) around one MKCOL+PUT pass"
    - "Test-friendly delay injection via new_with_delays constructor (production uses DEFAULT_RETRY_DELAYS constant, tests inject Duration::from_millis(10))"
    - "Custom Debug impl to skip sensitive fields (Nextcloud app_token) — T-48-08 mitigation"
    - "Pure classification fn (classify: StatusCode + on_mkcol -> Classification) — unit-testable without network"
key-files:
  created:
    - service_impl/src/webdav_client.rs
  modified:
    - service_impl/Cargo.toml
    - service_impl/src/lib.rs
    - Cargo.lock
decisions:
  - "D-48-DAV-VERSION: reqwest_dav 0.3 statt der im Plan zitierten 0.1 (letzte stabile Version; deviation Rule 3 — die Plan-Version existiert nicht mehr auf crates.io als aktuelle Version)"
  - "D-48-DAV-REQWEST-EXPLICIT: reqwest 0.12 explizit als eigene Dependency (statt nur transitiv via reqwest_dav) mit default-features=false — verhindert dass native-tls in den Nix-Build gezogen wird, gibt uns Kontrolle über TLS-Backend (rustls-tls) unabhängig vom reqwest_dav-Feature-Gate"
  - "D-48-DAV-RAW-HTTP: MKCOL + PUT werden direkt über reqwest::Client gemacht (nicht über reqwest_dav's High-Level-API). Grund: die Tests messen HTTP-Verben gegen wiremock, und MKCOL braucht ohnehin Method::from_bytes(b\"MKCOL\") — der reqwest_dav-Wrapper würde die Klassifikations-Logik (405-on-MKCOL vs 405-elsewhere) verstecken"
  - "D-48-DAV-RETRY-SCOPE: Retry umfasst genau eine ganze MKCOL+PUT-Passage (nicht MKCOL und PUT separat) — ein Transient-Fehler auf MKCOL retried die gesamte Sequenz, ein Permanent-Fehler auf MKCOL bricht sofort ab. Konsistent mit der Plan-Vorgabe 'Retry umfasst eine ganze upload_file-Passage'"
  - "D-48-DAV-HTTPS-ONLY-OFF: https_only(false) im reqwest-Builder gesetzt, damit wiremock-Tests gegen http://localhost:PORT funktionieren. Produktion nutzt trotzdem https:// URLs aus der Config — der Toggle steht nur, damit ein http:// URL nicht direkt vom Client-Builder abgelehnt wird"
metrics:
  duration_seconds: 893
  completed_date: 2026-07-02
  tasks_completed: 2
  files_created: 1
  files_modified: 3
  tests_added:
    - "6 wiremock integration tests (Test A-F)"
    - "5 classify() unit tests"
    - "1 Debug-impl-does-not-leak-token test (T-48-08 verification)"
  total_new_tests: 12
---

# Phase 48 Plan 03: WebDAV-Client mit MKCOL + PUT + In-Run-Retry Summary

**One-liner:** Pure WebDAV-Client-Wrapper mit MKCOL-idempotent + PUT-overwrite und 3× Exponential-Backoff-Retry (2s/4s/8s) — inklusive Token-Leak-Guard im Debug-Impl und rustls-only TLS.

## Objective — Erfüllt

Erfüllt den Upload-Teil von EXP-01 (WebDAV-Push nach Nextcloud) und den Retry-Teil von EXP-03 (in-run 3× Exponential-Backoff). Der Scheduler in 48-04 kann jetzt `use service_impl::webdav_client::{WebDavClient, WebDavError}` importieren und pro gerenderter PDF-Woche `client.upload_file(folder, filename, bytes).await` aufrufen.

## What Got Built

### `service_impl/src/webdav_client.rs` (neuer Client)
- **`pub struct WebDavClient`** — cheap-to-clone (Arc'd), thin wrapper um `reqwest::Client`.
- **`WebDavClient::new(base_url, user, app_token)`** — Prod-Konstruktor, delays = `DEFAULT_RETRY_DELAYS` (`[2s, 4s, 8s]`).
- **`WebDavClient::new_with_delays(...)`** — Test-Konstruktor mit injectable delays (Tests nutzen `[10ms; 3]`).
- **`upload_file(folder, filename, bytes) -> Result<(), WebDavError>`** — MKCOL-idempotent + PUT (overwrite), umschlossen von Retry-Loop.
- **`WebDavError`** (thiserror) mit 3 Varianten: `Transient { attempts, reason }`, `Permanent { status, body }`, `Io(reqwest::Error)`.
- **`Classification`** private Enum (Success / MkcolExisting / Transient / Permanent) + `classify(status, on_mkcol)` pure fn — unit-testbar.
- **Custom `Debug` impl** schließt Client und Credentials aus, gibt nur `base_url` + `retry_delays` aus (T-48-08 Mitigation).
- **Konstanten:** `DEFAULT_RETRY_DELAYS: [Duration; 3]`, `REQUEST_TIMEOUT: Duration = 30s` (T-48-10 Mitigation).

### `service_impl/Cargo.toml`
- Neue prod-Dependencies:
  - `reqwest_dav = { version = "0.3", default-features = false, features = ["rustls-tls"] }`
  - `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "http2"] }`
  - `base64 = "0.22"`
  - `thiserror = "1"`
- Neue dev-Dependency:
  - `wiremock = "0.6"`
- Ausführlicher Kommentar an der reqwest_dav-Zeile erklärt: warum rustls-tls (Nix-Build ohne openssl-sys, T-48-09 Threat-Mitigation).

### `service_impl/src/lib.rs`
- `pub mod webdav_client;` hinzugefügt (Scheduler in 48-04 nutzt).

## Tests (12 neue Tests, alle grün, <100ms Suite)

### 6 wiremock-Integrations-Tests (spec-Tests A–F aus dem Plan)
- **A** `put_success_returns_ok` — 1× MKCOL 201 + 1× PUT 201 → Ok, kein Retry.
- **B** `mkcol_folder_exists_treated_as_success` — MKCOL 405 (Ordner existiert) + PUT 201 → Ok, 405 wird geschluckt.
- **C** `mkcol_created_then_put_success` — MKCOL 201 + PUT 201 → Ok.
- **D** `transient_5xx_retries_and_succeeds` — 2× PUT 503, dann PUT 201 → Ok nach 3 Versuchen.
- **E** `permanent_4xx_no_retry` — PUT 401 → `Err(Permanent { status: 401, .. })`, exakt 1 PUT (kein Retry).
- **F** `transient_exhausted_returns_error` — 3× PUT 503 → `Err(Transient { attempts: 3, .. })`.

### 5 classify() Unit-Tests
- `classify_2xx_is_success` (200/201/204)
- `classify_405_on_mkcol_is_mkcol_existing`
- `classify_405_off_mkcol_is_permanent`
- `classify_5xx_is_transient` (500/503/504)
- `classify_401_403_404_are_permanent`

### 1 Token-Leak-Guard-Test (T-48-08 Verifikation)
- `debug_impl_does_not_leak_app_token` — baut Client mit `"super-secret-token-abc123"` und asserted dass `format!("{client:?}")` weder den Token-String noch das Wort "token" enthält.

## Verification — All Gates Green

| Gate | Command | Result |
|------|---------|--------|
| Modul-Tests | `cargo test -p service_impl webdav_client` | 12/12 passed in 0.04s |
| Full workspace tests | `cargo test --workspace` | 731+ passed, 0 failed |
| Clippy hard gate | `cargo clippy --workspace -- -D warnings` | grün |
| Full workspace build | `cargo build --workspace` | grün |

Test-Suite läuft in ~40ms — weit unter dem 5-Sekunden-Limit aus dem Plan.

## Threat-Model Coverage

| Threat | Mitigation delivered |
|--------|---------------------|
| T-48-08 (Token-Leak) | Custom `Debug` impl skipt `client` + Credentials; verifiziert durch `debug_impl_does_not_leak_app_token`-Test. Zusätzlich `header_value.set_sensitive(true)` auf dem Authorization-Header, so dass reqwest/hyper ihn auch aus Wire-Level-Logs auslässt. |
| T-48-09 (TLS-Spoofing) | reqwest mit `rustls-tls` (default-features=false); KEIN `.danger_accept_invalid_certs(true)` (expliziter Kommentar im Code verbietet die Änderung ohne Threat-Review). |
| T-48-10 (DoS) | 30s Per-Request-Timeout; Retry hart auf `DEFAULT_RETRY_DELAYS.len()` = 3 Versuche gedeckelt; worst-case pro `upload_file` ≈ 30s × 3 + (2+4)s Delays ≈ 96s. |
| T-48-SC (Supply Chain) | `reqwest_dav` 0.3.3 + `wiremock` 0.6.5 via CONTEXT-Decision Q2 (LOCKED, discuss-phase mit User) — der pre-flight-Human-Verify-Checkpoint aus Plan Task 0 gilt damit als abgehakt. |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Version-Pin] reqwest_dav 0.3 statt 0.1**
- **Found during:** Task 1 (vor `cargo add`)
- **Issue:** Plan spec zitiert `reqwest_dav = "0.1"`, aber `cargo search reqwest_dav` liefert `reqwest_dav = "0.3.3"` als aktuellen stable release. `0.1` existiert nicht mehr als "letzte stabile Version".
- **Fix:** `reqwest_dav = { version = "0.3", default-features = false, features = ["rustls-tls"] }`. Plan-Kommentar "letzte stabile Version verifizieren" explizit erwähnt.
- **Files modified:** service_impl/Cargo.toml
- **Impact:** Keiner auf die Test-Behavior — die Tests messen HTTP-Verben, nicht die reqwest_dav-API.

**2. [Rule 2 — Kritische Missing-Functionality] reqwest explizit als Dep (nicht nur transitiv)**
- **Found during:** Task 1
- **Issue:** Der Plan sagt "reqwest ist als transitive Dep bereits verfügbar (via reqwest_dav), aber sicherstellen dass rustls-tls als default-Feature genutzt wird". Aber `reqwest_dav = { default-features = false, features = ["rustls-tls"] }` propagiert nicht automatisch auf ein transitiv-genutztes `reqwest` in unserem Code — wir müssen `reqwest` explizit als eigene Dep listen, damit wir vom Feature-Gate profitieren.
- **Fix:** `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "http2"] }` als eigene prod-Dep in `service_impl/Cargo.toml`. Verhindert dass `openssl-sys`/`native-tls` in den Nix-Build gezogen wird.
- **Files modified:** service_impl/Cargo.toml
- **Impact:** Nix-Build-Sicherheit. Ohne diesen Fix hätte der Nix-Build fehlgeschlagen (openssl-sys nicht in shell.nix).

**3. [Rule 3 — Additive-Feature] base64 und thiserror explizit als Deps**
- **Found during:** Task 1
- **Issue:** Der Plan nutzt `base64` (Basic-Auth-Encoding) und `thiserror` (WebDavError-Enum), aber diese sind nicht in `service_impl/Cargo.toml` (Workspace hat sie bislang nur transitiv).
- **Fix:** `base64 = "0.22"` + `thiserror = "1"` als prod-Deps hinzugefügt.
- **Files modified:** service_impl/Cargo.toml
- **Impact:** Keiner — beide sind stabile, weit verbreitete Crates.

**4. [Rule 3 — Test-Environment] `https_only(false)` im reqwest-Builder**
- **Found during:** Test-Ausführung
- **Issue:** wiremock startet auf `http://127.0.0.1:PORT`. Ohne `https_only(false)` würde reqwest die Requests direkt beim Client-Builder ablehnen.
- **Fix:** `https_only(false)` im reqwest-Builder, mit Kommentar dass Produktion trotzdem `https://` URLs aus der Config nutzt.
- **Files modified:** service_impl/src/webdav_client.rs
- **Impact:** Keine — reqwest's `https_only(true)` ist eh ein zusätzlicher Schutz, nicht die Haupt-TLS-Sicherheit; die kommt von rustls + Cert-Validation. Die Nextcloud-URL aus der DB-Config ist immer `https://`.

### Skipped by Design

- **Task 0 (Human-Verify Package-Legitimacy Checkpoint)** — Auto-approved per Executor-Prompt-Instruktion: die deps sind bereits durch CONTEXT.md Q2 (LOCKED discuss-phase Entscheidung) autorisiert. Manuell während Task 1 gegengecheckt: `reqwest_dav 0.3.3` (Owner niuhuan, MIT/Apache), `wiremock 0.6.5` (Owner LukeMathWalker, MIT/Apache) — beide crates.io Standard-Owner mit hohen Download-Zahlen.

### Auth Gates

- Keine (kein interaktiver Auth-Schritt in dieser Phase — Basic-Auth-Credentials kommen später vom Scheduler-Config in 48-04).

## Known Stubs

- Keine. Der Client ist selbstständig funktionsfähig und wird in 48-04 durch den Scheduler mit echter Config gefüttert.

## Follow-ups (für 48-04 Scheduler)

Der Scheduler in 48-04 muss:
1. `use service_impl::webdav_client::{WebDavClient, WebDavError, DEFAULT_RETRY_DELAYS};`
2. Aus dem `PdfExportConfigService` lesen: `nextcloud_url`, `webdav_user`, `webdav_app_token`, `target_folder`.
3. Pro gerenderter PDF-Woche `client.upload_file(&config.target_folder, &filename, bytes).await` aufrufen.
4. Auf `Err(WebDavError::Transient { .. })` (nach 3 fails) UND `Err(WebDavError::Permanent { .. })` → `pdf_export_config.last_error_at` + `.last_error_message` schreiben. Auf `Ok(())` → `.last_success_at` setzen.
5. **NICHT** nochmal retryen — die Retry-Loop lebt IM Client (per Plan spec "der Scheduler-Task selbst retried NICHT nochmal").

## Success Criteria — Erfüllt

- ✅ EXP-01 Upload-Anteil (MKCOL + PUT) implementiert.
- ✅ EXP-03 Retry-Anteil (3× Exponential-Backoff 2s/4s/8s Prod, `[10ms; 3]` Test) implementiert und getestet.
- ✅ Transient vs Permanent Klassifikation korrekt (5xx / IO-Error → Transient; 4xx außer MKCOL-405 → Permanent; 2xx → Success; 405-on-MKCOL → Success).
- ✅ Kein Token-Leak in `Debug`-Output (test-verifiziert).
- ✅ Alle Client-Tests grün, alle Workspace-Tests grün, Clippy hard gate grün.
- ✅ Test-Suite <5s (tatsächlich ~40ms für den webdav_client-Filter).

## Self-Check: PASSED

- ✅ `service_impl/src/webdav_client.rs` existiert (verifiziert per Read).
- ✅ `service_impl/Cargo.toml` enthält reqwest_dav 0.3, reqwest 0.12, base64 0.22, thiserror 1, wiremock 0.6.
- ✅ `service_impl/src/lib.rs` enthält `pub mod webdav_client;`.
- ✅ Alle 4 Gates grün: cargo test -p service_impl webdav_client, cargo test --workspace, cargo clippy --workspace -D warnings, cargo build --workspace.
- ✅ 12 neue Tests, keine bestehenden Tests broken.
- Commits: GSD auto-commit übernimmt (per config.json `commit_docs: true`, jj-co-located).
