# Phase 46: Backend-Hygiene & i18n (BE + FE) - Context

**Gathered:** 2026-07-02
**Status:** Ready for planning
**Mode:** Autonomous discuss (Textform, 1 relevante Frage → IMP-05 Copy)

<domain>
## Phase Boundary

Drei kleine, unabhängige Hygiene-Themen:

- **HYG-04**: „Edit structure"-Texte in Schichtplan-UI vollständig in de/en/cs übersetzen.
- **HYG-05**: REST-Test-Layer verifiziert, dass **alle** REST-Endpoints (nicht nur JSON) den korrekten `Content-Type` in der Response tragen.
- **IMP-05**: Pre-existing `i18n_impersonation_keys_match_german_reference`-Test grün stellen — kanonische Copy-Entscheidung ist getroffen: **Test anpassen an die kompakte 🥸-Emoji-Form der shipped Copy**, nicht die Prosaform der aktuellen Test-Referenz.

Kein Snapshot-Bump, keine Migration, keine neuen Deps.

</domain>

<decisions>
## Implementation Decisions

### IMP-05 Copy-Entscheidung (Q1 — User: „schon gefixt" = Entscheidung getroffen)
- **Kanonische Form** = shipped Copy in `shifty-dioxus/src/i18n/de.rs`:
  - `Key::ImpersonateActAs` → `"🥸 Agieren"`
  - `Key::ImpersonateBanner` → aktuelle De-Form (z.B. `"🥸 Du agierst als {user}"` — beim Umsetzen exakt aus `de.rs` übernehmen, nicht raten)
  - `Key::ImpersonateStop` → aktuelle De-Form (z.B. `"🥸 Beenden"` — dito)
- **Umsetzung**: Test `i18n_impersonation_keys_match_german_reference` in `shifty-dioxus/src/i18n/mod.rs:1575` anpassen, Copy in `de.rs` unverändert lassen.
- **Begründung**: Die 🥸-Form ist die sichtbare UI, hat bewussten Charakter, ist button-tauglich kurz. Die Test-Referenz war eine idealisierte Prosaform, die nie in der UI angekommen ist.

### HYG-04 Test-Stil (Q2 — User: „brauchst du echt nicht testen")
- **Kein Presence-Test** — nur die fehlenden i18n-Keys für „Edit structure" in de/en/cs ergänzen.
- **Umsetzung**: `grep -rn "Edit structure\|edit_structure"` in `shifty-dioxus/src/` → jede fehlende Übersetzung in `i18n/{en,de,cs}.rs` ergänzen.

### HYG-05 Content-Type-Test Scope (Q3 — User: „alle Endpoints")
- **Alle registrierten REST-Routen** werden geprüft, nicht nur JSON.
- Test iteriert über die utoipa/Axum-Route-Registrierung (oder eine explizit gepflegte Test-Matrix aller Endpoints).
- **Erwartete Content-Types pro Route** kommen aus der Handler-Definition:
  - JSON-Handler (default) → `application/json`
  - Falls es Handler mit anderen Typen gibt (z.B. `text/html`, `text/plain`, `application/pdf`, `text/csv`) — der Test kennt den erwarteten Typ pro Route und assertet ihn.
- **Umsetzung** (Claude's Discretion): Test-Layer in `rest/tests/` oder `service_impl/src/test/rest_content_type.rs` — smoke-test mit Test-Server (mock DB), pro Route ein GET/POST + Content-Type-Assertion. Route-Metadaten idealerweise aus der OpenAPI/utoipa-Doc extrahieren.

### Claude's Discretion
- Exakte Test-Location für HYG-05 (Backend-Root-Test-Crate vs. `rest/`-Modul).
- Exakte Iteration-Strategie über Routen (utoipa-Reflection vs. explizite Matrix).

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `shifty-dioxus/src/i18n/mod.rs:1535 i18n_impersonation_keys_present_in_all_locales` — Presence-Test-Vorbild (Phase-32-Muster).
- `shifty-dioxus/src/i18n/mod.rs:1575 i18n_impersonation_keys_match_german_reference` — der zu fixende Test (IMP-05).
- `shifty-dioxus/src/i18n/{de,en,cs}.rs` — Locale-Files, `add_text(Locale::X, Key::Y, "…")`-Muster.
- OpenAPI/`utoipa`-Doc für Route-Metadaten (siehe `rest/src/lib.rs` ApiDoc struct).

### Established Patterns
- Backend-Integration-Tests: `service_impl/src/test/` mit `#[tokio::test]` und in-memory SQLite.
- REST-Tests: bisher primär via Service-Layer; ein neuer Layer für Content-Type kann das erste Route-Level-Test-Vehikel im Repo werden.

### Integration Points
- **FE HYG-04:** `shifty-dioxus/src/page/shiftplan*.rs` (Wochenraster) und/oder `src/component/shiftplan_*.rs` — dort „Edit structure"-Texte lokalisieren + zu i18n-Keys machen.
- **FE IMP-05:** `shifty-dioxus/src/i18n/mod.rs` (Test) + evtl. de.rs (bestätigen dass Copy dort steht, nicht ändern).
- **BE HYG-05:** neuer Test-Layer (`rest/tests/content_type.rs` oder `service_impl/src/test/rest_content_type.rs`), nutzt `axum::Router` + `tower::ServiceExt`-Testing.

</code_context>

<specifics>
## Specific Ideas

- IMP-05: **Test aktualisieren**, damit `assert_eq!(i18n.t(Key::ImpersonateActAs).as_ref(), "🥸 Agieren")` (und analog Banner+Stop mit ihren shipped De-Strings) grün wird. Beim Umsetzen exakte Strings aus `de.rs` übernehmen (nicht aus dem Gedächtnis rekonstruieren).
- HYG-04: Nur Ergänzung (add_text-Aufrufe), keine Test-Erweiterung.
- HYG-05: Vollständigkeit ist das Ziel — jede Route hat einen erwarteten Content-Type, jede Route wird geprüft.

</specifics>

<deferred>
## Deferred Ideas

Nichts — Scope-treu innerhalb Phase 46.

</deferred>
