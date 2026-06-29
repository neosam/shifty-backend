# Phase 32: Admin-Impersonation Frontend + Audit-Schicht (FE+BE) - Context

**Gathered:** 2026-06-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Admins können temporär als anderer User agieren (**lesen UND schreiben**), mit
persistentem nicht-schließbarem Banner auf jeder Seite, **strukturiertem Audit der echten
Admin-Identität** bei jeder schreibenden Aktion, und sauberem Store-Teardown beim Beenden.

**Wichtige Ausgangslage (Code-verifiziert):** Das **Impersonation-Backend existiert
bereits** und ist sauberer als das ROADMAP/der Todo annahm:
- Endpoints `POST /admin/impersonate/{user_id}`, `DELETE /admin/impersonate`,
  `GET /admin/impersonate` (`rest/src/impersonate.rs`).
- `Session.impersonate_user_id`, `start_impersonate`/`stop_impersonate` Session-Service,
  `resolve_session_user_id` + `context_extractor` (`rest/src/session.rs`).
- **Alle 3 Handler admin-gaten bereits gegen die ECHTE `session.user_id`** (nicht den
  effektiven Context — `impersonate.rs:66-72/128-134/178-184`) → **D-32-02 (Admin-Gate-
  Two-Path) ist bereits implementiert**, nur noch zu dokumentieren.
- `ImpersonateTO { impersonating: bool, user_id: Option<Arc<str>> }` (`rest-types`).

**Verbleibend:** (1) die Audit-Schicht (`Extension<RealUser>` + zentrale Logging-
Middleware), (2) das gesamte Impersonation-**Frontend**, (3) Doku des Two-Path-Vertrags.

</domain>

<decisions>
## Implementation Decisions

### Audit-Mechanik (IMP-03, D-32-01 — User-Entscheidung: zentrale Middleware)
- **D-32-01:** Audit über **eine zentrale Tower-Middleware**, NICHT pro Write-Handler
  (kein Blast-Radius) und nicht nur Start/Stop.
  1. Neuer Newtype `RealUser(Arc<str>)` (Modul-Discretion, z.B. `rest/src/session.rs` o.
     eigenes Modul).
  2. In **BEIDEN** `context_extractor`-Varianten (`#[cfg(feature="oidc")]` UND
     `#[cfg(all(mock_auth, not(oidc)))]`, `session.rs:62-98` + `:100-166`): wenn
     `session.impersonate_user_id.is_some()`, zusätzlich `Extension(RealUser(
     session.user_id.clone()))` in die Request-Extensions einfügen (der effektive
     `Context` bleibt `resolve_session_user_id` — **unverändert**).
  3. Neue Middleware-Layer (nach `context_extractor` in der Layer-Kette,
     `rest/src/lib.rs`): wenn `Extension<RealUser>` vorhanden **und** Methode ∈
     {POST, PUT, PATCH, DELETE} → `tracing::info!(real_user = …, acting_as = …,
     method = …, path = …, "impersonated write")`. Garantiert: **keine Schreib-Aktion
     bleibt unattribuiert** (ROADMAP SC3), ohne einen einzigen Write-Handler anzufassen.
  4. Zusätzlich **explizite Start/Stop-Audit-Zeilen** `tracing::info!` in
     `start_impersonate`/`stop_impersonate` (`impersonate.rs`): echter Admin + Ziel-User.
- **D-32-01a (Out-of-Scope-Lock):** **KEINE** Änderung an `Authentication<Context>`-
  Signaturen (Blast-Radius, explizit Out of Scope). **KEINE** DB-Persistierung des Audits
  (log-only via `tracing` für v1.9; DB-Audit-UI ist Out of Scope).

### Admin-Gate-Two-Path-Vertrag (IMP-01, D-32-02 — bereits implementiert, dokumentieren)
- **D-32-02:** Der Vertrag „Impersonation-Handler prüfen die ECHTE Caller-Identität (rohe
  `session.user_id`), nicht den effektiven (impersonierten) Context" ist **bereits im Code**
  (`impersonate.rs` baut `Authentication::Context(Some(session.user_id.clone()))` für den
  admin-Check). Aufgabe: diesen Invariant **dokumentieren** (Kommentar/Doc in
  `rest/src/lib.rs` an der Route-Nest-Stelle und/oder bei den Handlern), damit künftige
  Änderungen ihn nicht versehentlich brechen. Kein Code-Change am Gate selbst nötig.
- **D-32-02a (P10 Known Limitation):** Während der Impersonation eines **Nicht-Admins** ist
  der effektive Context nicht-admin → der Admin ist von **admin-only Endpoints**
  ausgesperrt (korrekte Sicherheitseigenschaft). **Stop funktioniert trotzdem** (nutzt rohe
  `session.user_id`). Diese Einschränkung wird (a) im Banner/known-limitation-Text und
  (b) im DISCUSS/CONTEXT dokumentiert — kein Workaround, by design.

### Frontend (IMP-01/02/04 — der Großteil, komplett neu)
- **D-32-03 (Banner-Identität — User-Entscheidung: Username direkt):** Der Banner zeigt die
  **`user_id`/den Username** direkt aus `ImpersonateTO` („Du agierst als {user_id} —
  Impersonation beenden"). **Keine** Backend-Änderung an `ImpersonateTO`, **kein**
  Anzeige-Namen-Lookup (Friendly-Name ist deferred).
- **D-32-04 (Banner):** Persistenter, **nicht-schließbarer** Amber-Banner auf **jeder
  Seite** (Mount in `app.rs`, oberhalb des Routers/Content), mit One-Click-Stop-Button.
  Nicht-blockierend (Banner-statt-Dialog-Konvention, Projekt-Memory). Neue Komponente
  `component/impersonation_banner.rs`.
- **D-32-05 (Reload-Persistenz, IMP-02):** Der Impersonation-Status wird **beim App-Mount**
  via `GET /admin/impersonate` als **erster Init-Call** geladen (nicht nur aus dem
  Start-Callback) → Banner erscheint nach hartem Reload automatisch wieder. Neuer
  Service `service/impersonate.rs` (Store + Coroutine/Logik), 3 API-Calls in `api.rs`.
- **D-32-06 (Store-Teardown beim Stop, IMP-04):** Nach `DELETE /admin/impersonate` werden
  user-bezogene FE-Stores für den echten User **neu geladen** — mindestens
  `current_sales_person`; der Planner prüft beim Lesen, welche weiteren user-spezifischen
  Stores (z.B. die im Shiftplan/Report) stale werden könnten und re-initialisiert sie
  (kein stale Impersonations-State). Symmetrisch sollte auch der **Start** die relevanten
  Stores neu laden, damit die impersonierte Sicht sofort korrekt ist (Planner-Discretion,
  aber konsistent zu IMP-04).
- **D-32-07 (Einstieg, IMP-01):** Start der Impersonation aus der **HR-/Admin-Personen-
  liste** (Planner findet die Komponente; eine admin-sichtbare „Als diese Person agieren"-
  Aktion pro Person, die `POST /admin/impersonate/{user_id}` aufruft). Der `user_id` für
  den Endpoint ist die **Auth-Identität** der Zielperson — der Planner klärt beim Lesen, wie
  aus einem Personen-Listen-Eintrag die `user_id` gewonnen wird (SalesPerson vs.
  Auth-Username-Mapping); falls nicht trivial verfügbar, ist das ein Sub-Punkt für die
  Planung (ggf. kleinster nötiger Zugang).

### i18n (querschnittlich)
- **D-32-08:** Alle neuen benutzersichtbaren Strings (Banner-Text, Stop-Button,
  „Als Person agieren"-Aktion, ggf. P10-Hinweis) in **de/en/cs** (`i18n/mod.rs` Key-Enum +
  `en.rs`/`de.rs`/`cs.rs`).

### Claude's Discretion
- Modul-Ort von `RealUser`; genaue Layer-Reihenfolge/Name der Audit-Middleware; Store-
  Struktur des FE-`impersonate`-Service (GlobalSignal-Muster wie die anderen Services);
  exakter Mount-Punkt des Banners in `app.rs`; welche weiteren Stores bei Start/Stop
  reloaded werden (D-32-06); wie der `user_id` aus der Personenliste gewonnen wird (D-32-07).
- Test-Strategie: Backend-Audit-Middleware + `RealUser`-Inject per `cargo test --workspace`
  (Integration-Test-Stil wie `shifty_bin/src/integration_test/`); FE reine Logik (Status-
  Mapping) per `cargo test` wo testbar. Banner-Visual + Live-Roundtrip ist optionaler
  manueller Browser-Smoke (nicht pixel-automatisierbar).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirement & Roadmap
- `.planning/REQUIREMENTS.md` §IMP-01..04 + §"Out of Scope" (kein Context-Signatur-Change,
  kein DB-Audit, kein anderer Admit, kein Auto-Timeout).
- `.planning/ROADMAP.md` §"Phase 32" — Goal + Success Criteria 1–5 + Known limitation (P10).

### Backend — bestehender Mechanismus (Single Source of Truth, MEIST nur lesen)
- `rest/src/impersonate.rs` — 3 Handler; **admin-Gate gegen echte `session.user_id`
  (Z.66-72/128-134/178-184)** = D-32-02; hier kommen die Start/Stop-`tracing`-Zeilen rein.
- `rest/src/session.rs` — `Context = Option<Arc<str>>` (Z.12), `resolve_session_user_id`
  (Z.54-60), **beide** `context_extractor`-Varianten (Z.62-98 oidc, Z.100-166 mock_auth) =
  Inject-Punkt für `Extension<RealUser>`.
- `rest/src/lib.rs` — Route-Nest `/admin/impersonate` (Z.635) + Layer-Kette = Einbau-Punkt
  der Audit-Middleware + Doku-Stelle für den Two-Path-Vertrag.
- `rest-types` `ImpersonateTO` — bleibt unverändert (D-32-03).

### Frontend — neu zu bauen
- `shifty-dioxus/src/api.rs` — 3 neue Calls (Start POST/{user_id}, Stop DELETE, Status GET).
- `shifty-dioxus/src/service/impersonate.rs` (neu) — Store + Logik (Muster: andere Services
  in `service/`, GlobalSignal + Coroutine).
- `shifty-dioxus/src/component/impersonation_banner.rs` (neu) — der Banner.
- `shifty-dioxus/src/app.rs` — Banner-Mount + Init-Call beim App-Mount.
- HR-/Admin-Personenliste-Komponente (Planner findet sie) — „Als Person agieren"-Einstieg.
- `shifty-dioxus/src/i18n/{mod.rs,en.rs,de.rs,cs.rs}` — neue Keys.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `context_extractor` setzt schon den effektiven `Context` per `resolve_session_user_id` →
  der `RealUser`-Inject ist ein additiver Zweig daneben (wenn `impersonate_user_id.is_some()`).
- Banner-statt-Dialog-Konvention (Projekt-Memory): Banner ist nicht-blockierend, kein Modal.
- FE-Service-Muster (`GlobalSignal<Store>` + `async fn service(rx)`-Coroutine) wie
  `weekly_summary.rs`/`booking_conflict.rs` → Vorlage für `impersonate.rs`.
- `current_sales_person`-Signal im Shiftplan (Phase 30/31) ist DER user-bezogene Store, der
  beim Stop reloaded werden muss (D-32-06).

### Established Patterns / Guardrails
- **BE-Phase!** Backend-Änderungen MÜSSEN `cargo clippy --workspace -- -D warnings` +
  `cargo test --workspace` bestehen (Pflicht-Gate). FE-Änderungen: WASM-Build + FE-`cargo test`.
- **Service-Tier-Konvention:** der Audit/Impersonation-Pfad ist REST-Layer-Middleware +
  bestehender Session-Service — kein neuer Domain-Service nötig; keine zyklische DI.
- **Kein Snapshot-Bump** (keine `BillingPeriodValueType`-Computation berührt).
- Static-Tailwind/Pitfall-5: der Amber-Banner nutzt statische Klassen (`bg-warn`/`warn-soft`
  o.ä., kein interpoliertes Class-String).

### Integration Points
- Layer-Kette in `rest/src/lib.rs`: Audit-Middleware NACH `context_extractor` (braucht die
  `RealUser`-Extension). Reihenfolge beachten.
- `app.rs`: Banner über dem Router-Outlet, damit er auf jeder Route sichtbar ist; Init-Call
  beim Mount.

</code_context>

<specifics>
## Specific Ideas

Akzeptanz (ROADMAP SC1-5):
1. Admin startet Impersonation aus der Personenliste → sofort Amber-Banner „Du agierst als
   {user_id}" + Stop-Button auf jeder Seite.
2. Harter Reload während Impersonation → Banner wieder da (GET-Status-Init beim Mount).
3. Jede Schreib-Aktion unter Impersonation → strukturierte Log-Zeile mit real_user + acting_as.
4. Nach Stop → `current_sales_person` (+ weitere) für den echten Admin neu geladen, kein Stale.
5. Nicht-Admin kann keine Impersonation starten/ausnutzen (Gate gegen rohe session.user_id).

P10-Hinweis im Banner/Doku: „Während der Impersonation eines Nicht-Admins sind admin-only
Funktionen deaktiviert; Beenden ist jederzeit möglich."

</specifics>

<deferred>
## Deferred Ideas

- Aufgelöster Anzeige-Name im Banner (statt user_id) — verworfen für v1.9 (D-32-03).
- DB-persistiertes Audit + Audit-Log-UI für andere Admins — Out of Scope (log-only v1.9).
- Impersonation-Session-Auto-Timeout (`impersonate_expires_at`) — Out of Scope (v2+).
- Impersonation eines anderen Admins — Out of Scope (P10).

### Reviewed Todos (not folded)
- Off-theme Treffer zu anderen Phasen/Backlog — nicht gefolded.

</deferred>

---

*Phase: 32-Admin-Impersonation Frontend + Audit-Schicht (FE+BE)*
*Context gathered: 2026-06-29*
