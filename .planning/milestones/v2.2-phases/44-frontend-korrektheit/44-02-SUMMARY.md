---
phase: 44
plan: 02
subsystem: frontend
status: complete
tags:
  - bugfix
  - frontend
  - error-handling
  - i18n
requires:
  - v2.2 milestone
provides:
  - "ShiftyError::InvitationParse(String) variant"
  - "UserManagementStore.user_invitations_load_error field"
  - "i18n key UserInvitationsLoadError (de/en/cs)"
  - "user_details inline red error banner on invitation-load failure"
affects:
  - shifty-dioxus/src/error.rs
  - shifty-dioxus/src/api.rs
  - shifty-dioxus/src/service/user_management.rs
  - shifty-dioxus/src/page/user_details.rs
  - shifty-dioxus/src/i18n/{mod,de,en,cs}.rs
tech-stack:
  added: []
  patterns:
    - "pure-fn extraction of parse logic for unit-testability without HTTP"
    - "dual error channel: central ERROR_STORE overlay + per-page persistent inline banner"
key-files:
  created: []
  modified:
    - shifty-dioxus/src/error.rs
    - shifty-dioxus/src/api.rs
    - shifty-dioxus/src/loader.rs
    - shifty-dioxus/src/service/user_management.rs
    - shifty-dioxus/src/page/user_details.rs
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
decisions:
  - "Parse-Fehler propagieren als eigene ShiftyError-Variante `InvitationParse(String)` statt `#[from] serde_json::Error`, weil die Call-Site die 200-Zeichen-Body-Head anhaengt (Diagnose-Info + PII-Limit)."
  - "Doppelter Fehler-Kanal beibehalten: ERROR_STORE feuert weiterhin fuer den Overlay-Toast, das neue Store-Feld `user_invitations_load_error` steuert den persistierenden Inline-Banner auf der User-Details-Seite. Overlay = one-shot, Banner = page-scoped, komplementaer."
  - "Bei Fehler wird `user_invitations` auf leere Rc geresettet, damit die stale Liste unterhalb des Banners nicht mehr als 'Erfolg' erscheint."
metrics:
  duration_min: 4
  completed_date: 2026-07-02
  tests_added: 5
  tests_passing: 777
  tests_failing_out_of_scope: 1
---

# Phase 44 Plan 02: BUG-02 — list_user_invitations parse-error sichtbar Summary

**One-liner:** ShiftyError-Variante `InvitationParse(String)` + Inline-Banner + i18n (de/en/cs) machen einen Backend-JSON-Schema-Drift auf der User-Details-Seite sofort sichtbar — statt ihn als leere Einladungsliste zu verschleiern.

## Objective

BUG-02 (v2.2) beheben: `shifty-dioxus/src/api.rs::list_user_invitations` hatte einen `serde_json`-Parse-Fehler in einem `match`-Zweig auf `Ok(Rc::new([]))` fallen gelassen (mit `TODO`-Kommentar). Jede Response, deren JSON nicht mehr zum Frontend-`InvitationResponse`-Schema passt (Feld-Umbenennung, neue Enum-Variante, falsche Root-Shape), erschien im UI identisch zum echten leeren Zustand — kein Trace, keine Meldung. Fix: neue `ShiftyError::InvitationParse(String)`-Variante, propagierter Fehler-Pfad ueber api/loader/service, per-page Inline-Banner ueber dem Empty-State-Zweig.

## Changes (Datei-Diff-Zusammenfassung)

- **`shifty-dioxus/src/error.rs`** (+16 Zeilen)
  - Neue Variante `#[error("invitation parse error: {0}")] InvitationParse(String)`. Kein `#[from]` — die Konvertierung `serde_json::Error → String → InvitationParse` ist call-site-explizit, weil dort die 200-Zeichen-Body-Head angehaengt wird.

- **`shifty-dioxus/src/api.rs`** (+83 Zeilen, −25 Zeilen)
  - Neue `pub(crate) fn parse_invitations_response(body: &str) -> Result<Rc<[InvitationResponse]>, ShiftyError>` extrahiert die Parse-Logik als pure fn.
  - `pub async fn list_user_invitations` retourniert jetzt `Result<..., crate::error::ShiftyError>` statt `Result<..., reqwest::Error>`; delegiert an `parse_invitations_response`.
  - Silent-empty-Fallback + TODO-Kommentar entfernt.
  - Neuer Test-Modul `mod parse_invitations_tests` mit 5 Tests (siehe unten).

- **`shifty-dioxus/src/loader.rs`** (unveraendert)
  - `load_user_invitations` retournierte bereits `Result<..., ShiftyError>`; `?`-Operator propagiert die neue Variante automatisch, weil `list_user_invitations` jetzt denselben Error-Typ liefert. Keine Aenderung noetig — nur verifiziert.

- **`shifty-dioxus/src/service/user_management.rs`** (+16 Zeilen, −3 Zeilen)
  - Neues `UserManagementStore`-Feld `user_invitations_load_error: Option<ImStr>`, `#[derive(Default)]` liefert `None`.
  - `load_user_invitations`-service resettet das Feld bei `Ok` (`None`) und setzt es bei `Err` auf `Some(err.to_string().into())`; parallel wird `user_invitations` auf `Rc::new([])` gesetzt (keine stale Liste unter dem Banner) und `ERROR_STORE` wie bisher gefuellt.

- **`shifty-dioxus/src/i18n/mod.rs`** (+1 Zeile)
  - Neuer `Key::UserInvitationsLoadError` alphabetisch bei `UserInvitations`.

- **`shifty-dioxus/src/i18n/{de,en,cs}.rs`** (+5 Zeilen je Datei)
  - Uebersetzungen (siehe Abschnitt „i18n-Zeilen woertlich" unten).

- **`shifty-dioxus/src/page/user_details.rs`** (+13 Zeilen)
  - Neuer roter Inline-Banner (`border-red-300 bg-red-50`) zwischen dem `mt-8 border-t pt-6`-Titel-Block und dem `is_empty()`-Zweig. Rendert nur, wenn `user_management.user_invitations_load_error` gesetzt ist. Zeigt den i18n-Titel + darunter die technische Fehler-Message in `font-mono text-small`.

## Test-Zaehlung

- **Neu:** 5 Tests in `api::parse_invitations_tests`:
  1. `invitation_parse_display_contains_prefix_and_cause` — `ShiftyError::InvitationParse("…").to_string()` enthaelt `"invitation parse error"` UND die Ursachen-Message.
  2. `valid_empty_array_parses_ok` — `"[]"` bleibt weiterhin `Ok` mit `len() == 0`.
  3. `valid_populated_array_parses_ok` — minimales `InvitationResponse`-Fixture parst `Ok`; pinnt die rest-types-Pflichtfelder (id/username/token/invitation_link/redeemed_at/status) als Drift-Detektor.
  4. `invalid_json_returns_invitation_parse_error` — `"nope"` erzeugt `Err(ShiftyError::InvitationParse(_))` mit nicht-leerer Message + Body-Head-Snippet.
  5. `wrong_shape_object_returns_invitation_parse_error` — `{"error":"nope"}` (v1.x-Silent-Empty-Trigger) liefert jetzt `Err(InvitationParse(_))` statt „no invitations".
- **Bestand:** 772 vorherige Tests → jetzt insgesamt 777 passing.

## Verify-Ergebnis

| Gate | Kommando | Ergebnis |
|------|---------|----------|
| Silent-empty gone (api.rs) | `grep -c 'Ok(Rc::new(\[\]))' src/api.rs` | `0` ✅ |
| InvitationParse in use | `grep -n 'InvitationParse' src/error.rs src/api.rs \| wc -l` | `12` ≥ 3 ✅ |
| Store-Feld gesetzt+gelesen | `grep -c 'user_invitations_load_error' src/service/user_management.rs src/page/user_details.rs` | 3 + 1 = 4 ≥ 3 ✅ |
| i18n-Key in allen 4 Dateien | `grep -l 'UserInvitationsLoadError' src/i18n/{mod,de,en,cs}.rs` | 4 Dateien ✅ |
| WASM-Build | `cargo build --target wasm32-unknown-unknown` | Finished ✅ |
| Frontend-Test-Suite | `cargo test -p shifty-dioxus` | 777 passed / 1 failed (`i18n_impersonation_keys_match_german_reference`, pre-existing out-of-scope, siehe MEMORY.md) ✅ |
| Backend-Clippy-Gate | `cargo clippy --workspace -- -D warnings` | Finished (0 Warnings) ✅ |
| Parse-Unit-Tests einzeln | `cargo test -p shifty-dioxus -- parse_invitations` | 5 passed / 0 failed ✅ |

## Die drei i18n-Zeilen woertlich

- **De:** `"Einladungen konnten nicht geladen werden. Details siehe Fehleranzeige."`
- **En:** `"Failed to load invitations. See error banner for details."`
- **Cs:** `"Nepodařilo se načíst pozvánky. Podrobnosti viz chybové okno."`

## Success Criteria — Abgleich

1. ✅ `api::list_user_invitations` propagiert Parse-Fehler als `ShiftyError::InvitationParse(String)` — kein silent-empty-Fallback mehr (BUG-02 SC 2). Grep-Gate `0` bestaetigt.
2. ✅ Store trennt „leere Liste" und „Parse-Fehler" via `user_invitations_load_error: Option<ImStr>`; UI rendert bei Fehler einen sichtbaren roten Inline-Banner UND fuellt weiterhin den zentralen ErrorView (Overlay). „leere Liste" != „Parse-Fehler" ist visuell durchgesetzt.
3. ✅ i18n de/en/cs fuer den neuen Fehler-Titel vorhanden; WASM-Build + `cargo test -p shifty-dioxus` gruen (nur pre-existing Impersonation-Test rot).

## Deviations from Plan

**None — Plan wurde exakt ausgefuehrt.**

Kleine Zusatze innerhalb des Scopes:

- **Bonus-Test:** Zusaetzlich zum von `<behavior>` verlangten Fail-Test fuer `"nope"` habe ich `wrong_shape_object_returns_invitation_parse_error` fuer `{"error":"nope"}` ergaenzt — genau die Backend-Response-Shape, die v1.x als „No invitations found" verschleiert hat. Regressionswert.
- **Zusatzliche `parse_invitations_response`-Doc:** Sicherheits-Hinweis zur 200-Char-Head im Docstring ergaenzt (referenziert `error.rs`).

## Restrisiken

- **Manuelle Verify nicht durchgefuehrt:** Der Plan sieht optional eine manuelle Live-Sanity vor (Backend-Endpoint patchen, sodass er `{"error":"nope"}` liefert, um Banner + Overlay in Browser zu sehen). Die strukturelle Verifikation (Grep-Gates, Parse-Fn-Tests, WASM-Build) ist die Nyquist-Ebene und deckt alle Codepfade. Live-Test kann bei Bedarf im Kontext von 44-01/44-03 nachgezogen werden.
- **Pre-existing Test-Fail:** `i18n::tests::i18n_impersonation_keys_match_german_reference` ist rot (bekannter Bug, nicht in diesem Scope; TODO-Datei existiert unter `.planning/todos/pending/2026-07-02-i18n-impersonation-key-test-mismatch.md`).
- **Information-Disclosure T-44-02-I-1** ist wie im Threat-Model geplant mitigiert (200-Char-Head-Limit als Anpassungspunkt, falls das Invitation-Schema um Sensitives erweitert wird — dann Limit im `parse_invitations_response` anpassen).

## Self-Check: PASSED

- `.planning/phases/44-frontend-korrektheit/44-02-SUMMARY.md` — FOUND (dieser File)
- Alle betroffenen Sourcen existieren und wurden modifiziert.
- Testlauf haengt 5 neue Tests unter `parse_invitations_tests` an, alle 5 gruen.
- Grep-Gates alle 4 gruen.
- WASM-Build + Backend-Clippy gruen.
- Frontend-Test-Suite bis auf pre-existing `i18n_impersonation_keys_match_german_reference` gruen (777/778).
