# Phase 63: Personen-Detailansicht + Ausprägungs-Aktionen - Research

**Researched:** 2026-09-20
**Domain:** Rust/Axum backend service composition + Dioxus/WASM frontend detail view, within an established multi-crate workspace that already carries 90% of the domain model from Phases 60–62.
**Confidence:** HIGH — every claim below is grounded in direct code reads of this repository (file:line), not external library research. There is no new external dependency in this phase.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

Alle acht Entscheidungen sind am 2026-09-20 vom Nutzer als Block bestätigt worden. D-63-01 bis D-63-03 beantworten die drei in der ROADMAP gepinnten Discuss-Points unter derselben Nummer.

- **D-63-01:** Die `person`-Zeile wird mitgelöscht, sobald sie ihre letzte Ausprägung verliert. Keine Waisen, keine Admin-Sicht auf trägerlose Personen. Abgrenzung zu D-61-05: Beim **Lösen** entsteht weiterhin keine trägerlose Person. D-63-01 greift ausschließlich beim **Löschen** des letzten Trägers. Reversibility: reversible (Service-Logik, keine Schema-Wirkung).
- **D-63-05:** „Verknüpfung lösen" und „löschen" sind zwei getrennte, verschieden beschriftete Aktionen — bewusst KEIN Sammelknopf „Eintrag löschen":
  | Aktion | Wirkung | Charakter |
  |---|---|---|
  | „Verknüpfung lösen" | Aus einem Eintrag werden zwei; beide Objekte bleiben vollständig erhalten (D-61-05) | harmlos, umkehrbar über den Kandidaten-Picker |
  | „Login löschen" | `delete_user` — **Hard Delete** inkl. Rollen und Einladungen | destruktiv, endgültig |
  | „Verkäufer löschen" | `SalesPersonService::delete` — **Soft Delete** (`deleted`-Stempel) | destruktiv, Daten bleiben in der DB |
- **D-63-06:** Das Löschen der Verkäufer-Seite wird im Frontend **erstmals freigeschaltet**. Heute existiert `SalesPersonService::delete` nur im Backend — kein Frontend-Aufrufpfad. Auflage: Die Bestätigung nennt das bestehende `inactive`-Flag ausdrücklich als schonende Alternative.
- **D-63-08:** Die Lösch-Bestätigung trägt festen, ausprägungs-spezifischen Text ohne gezählte Betroffenheit — zwei Textbausteine (Verkäufer-Seite: Schichten/Stundenkonto/Abrechnungs-Historie bleiben erhalten, Soft Delete, Hinweis auf „inaktiv setzen"; Benutzer-Seite: Login/Rollen/Einladungen werden endgültig gelöscht). Ein Zähl-Endpoint für echte Betroffenheit wurde **verworfen** (zu viel Umfang für diese Phase).
- **D-63-04:** Eine neue Route `/person/:person_id/`; die beiden alten Detail-Routen (`Route::UserDetails`, `Route::SalesPersonDetails`) werden **ersatzlos entfernt**, keine Weiterleitungs-Hüllen. Nachweis-Auflage: abschließender Grep auf `UserDetails`/`SalesPersonDetails` muss leer sein (außer Archiv/Planungsdateien).
- **D-63-02:** „Neu anlegen" und „Bestehendes verknüpfen" bleiben zwei getrennt beschriftete Aktionen, kein Kombi-Feld. Der Kandidaten-Picker (PADM-04) hängt ausschließlich an „Bestehendes verknüpfen" und zeigt nur **unverknüpfte** Kandidaten. Serverseitig bleibt D-61-04: 409 bei bereits verknüpften Zielen, auch wenn sie im Picker nicht erscheinen.
- **D-63-03:** Der neue Anlege-Dialog **ERSETZT** die beiden heutigen Knöpfe (`Key::CreateNewSalesPerson`, `Key::AddNewUser`), ergänzt sie nicht. Nachweis-Auflage: genau ein Anlege-Einstieg in der Toolbar nach dem Umbau.
- **D-63-07:** Der Anlege-Dialog bleibt schlank: (1) Anzeigename, (2) zwei Umschalter „bekommt ein Login" / „bekommt einen Verkäufer-Datensatz", (3) Benutzername-Feld nur sichtbar wenn „Login"-Umschalter an ist. Farbe/bezahlt/freiwillig/inaktiv/Schichtplan-Zuordnung kommen danach in der Detailansicht, wohin der Dialog nach dem Speichern weiterleitet. Mindestens ein Umschalter muss an sein.

### Gesperrt — in dieser Phase nicht neu aufmachen

- Sektions-Sichtbarkeit richtet sich nach der **Ausprägung**, nicht nach der Rolle. Abschnittsweises Rollen-Gating ist PADM-F-02 (Future). Seite als Ganzes ist admin-gated (PADM-08/D-62-08).
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**. Keine der vier Bump-Klauseln greift — nicht "vorsichtshalber" bumpen.
- Keine Migration in dieser Phase. Schema steht seit Phase 60/61.
- `sales_person.id` und `user.name` werden nicht umgeschlüsselt.
- Verknüpfen ist ein Merge zweier Personen (D-61-03), 409 nur bei echter Mehrdeutigkeit (D-61-04), Lösen hinterlässt nie eine trägerlose Person (D-61-05).
- Fat Backend, Thin Client — Ausprägung und Anzeigename kommen fertig aus dem Backend, keine Ableitungsregel im Frontend.
- Tier-Regel: `PersonService` bleibt Basic-Tier. Detail-Aggregat und transaktionaler Anlege-Pfad sind Business-Logic-Tier, konstruiert **nach** dem Basic-Block in `shifty_bin/src/main.rs`.
- `Dioxus.toml`-Proxy-Eintrag für jeden neuen Routen-Präfix im selben Commit, verifiziert per `dx serve`-Smoke — dieser Bug wurde dreimal geshipt.
- `find_sales_person_by_user_id` ist eine Autorisierungsprüfung, kein CRUD — nicht anfassen.

### Claude's Discretion

- Zuschnitt des Detail-Aggregats: ein `GET /person/{id}`-Endpoint mit allem, oder Kern-Aggregat plus gezielte Nachladepfade. Auflage: kein N+1 pro Zeile/Abschnitt, keine Ausprägungs-Ableitung im Frontend.
- Name, Modul-Ablage und Trait-Zuschnitt des Business-Logic-Services (Arbeitstitel `PersonDetailService` oder Erweiterung von `PersonDirectoryService`).
- Form des Anlege-Endpoints: neuer transaktionaler Endpoint (bevorzugt, da SC-2 "genau die gewählten Objekte" verlangt) vs. Frontend-Orchestrierung aus bestehenden Calls.
- Routen-Präfixe und Endpoint-Zuschnitt (`/person/{id}`, Kandidaten-Pfade), mit Dev-Proxy-Auflage.
- Darstellung der Abschnitte (Reihenfolge, Karten vs. Sektionen). Wiederverwendbar: `h2`-Sektionsköpfe aus `sales_person_details.rs:95,147,227` und `user_details.rs:76,148`.
- Form des Kandidaten-Pickers (Dropdown, durchsuchbare Liste, Dialog), clientseitige Filterung ja/nein.
- Durchsetzung "mindestens ein Umschalter muss an sein" (deaktivierter Button vs. Validierungsmeldung).
- i18n-Key-Namen (bestehende wiederverwenden, wo passend). Alle drei Sprachen synchron, abgesichert durch bestehenden Key-Mengen-Test.
- Wellen-Schnitt, Plan-Anzahl, Testablage.

### Deferred Ideas (OUT OF SCOPE)

- Gezählte Betroffenheit in der Lösch-Bestätigung ("47 Buchungen, 3 Abrechnungsperioden") — eigener Zähl-Endpoint, kostet zu viel Umfang.
- Weiterleitungs-Hüllen für `/user_details/:id/` und `/sales_person_details/:id/` — bewusst nicht gebaut.
- Abschnittsweises Rollen-Gating (PADM-F-02, Future).
- HR-Report `/employees/:id` in die Personen-Ansicht einfalten (PADM-F-01, Future).
- Echte Personenstammdaten (E-Mail, Telefon, Geburtsdatum) an `person` (PERS-F-01, Future).
- Zusammenführen zweier Personen im UI — Mechanik existiert (D-61-03), Bedienoberfläche ist kein v2.7-Scope.
- Planungs-Soll für dynamische Verträge — Phase 64.
- Backlog: `WR-01`, `WR-03`, `WR-07`, `IN-01..IN-07` aus `61-REVIEW.md`.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PADM-02 | Beim Anlegen erfasst der Admin **einen** Eintrag und schaltet zu, ob er ein Benutzerkonto und/oder einen Verkäufer-Datensatz bekommt | Combined-create endpoint gap identified (see "Missing Backend Pieces" #2); transaction-boundary mismatch between `PermissionDao::create_user` (own local `pool.begin()`) and the shared `Transaction`/`TransactionDao` pattern documented with exact fix options |
| PADM-03 | Fehlende Ausprägung nachträglich zuschaltbar — Objekt wird angelegt und verknüpft | Reuses the same combined-create machinery on a single-object slice: "add facet" = create SalesPerson (existing `SalesPersonService::create`) or User (existing `PermissionService::create_user`, already person-aware) then `SalesPersonService::set_user` to link — see Pattern 2 |
| PADM-04 | Zwei getrennt bestehende Objekte verknüpfbar via Kandidaten-Auswahl, nicht Eintippen | Write path **already exists and is fully implemented**: `POST /sales-person/{id}/user` with 409 (D-61-04). Only the candidate **read** paths need REST exposure — `PersonService::unlinked_sales_persons`/`unlinked_users` exist since Phase 61 but have zero REST/frontend wiring today (verified via grep) |
| PADM-05 | Detailansicht zeigt alles an einem Ort, blendet fehlende Ausprägung aus | Detail-aggregate endpoint is a clean new build; old pages (`user_details.rs`, `sales_person_details.rs`) supply the exact sections to port, itemized in Pattern 1 |
| PADM-07 | Bestätigung nennt konkret, was verschwindet vs. bleibt, je Ausprägung | D-63-08 already fixes exact copy split; backend cascade behavior for both delete paths verified via migration FK reads (see "Deletion/Removal Semantics") |

</phase_requirements>

## Summary

Phases 60–62 did almost all of the hard backend groundwork for this phase already: the `person` table, the `PersonService`/`PersonDao` basic-tier pair with ready-to-use `unlinked_sales_persons`/`unlinked_users` candidate queries, the `PersonDirectoryService` business-logic aggregate, and — critically — the **entire link/unlink write path** (`SalesPersonService::set_user`, exposed at `POST /sales-person/{id}/user` and `DELETE /sales-person/{id}/user`, complete with the D-61-04 409-on-conflict translation). This means PADM-04's hardest part (the merge semantics, the race handling, the 409 contract) is **done**; what's missing for PADM-04 is only a thin REST read-endpoint for the two candidate lists and a picker UI.

The real backend work in this phase is threefold: (1) a new business-logic-tier detail aggregate (`GET /person/{id}` or similar) that assembles sales-person + shiftplan-assignment + roles + invitations for one person — a straightforward mirror of `PersonDirectoryServiceImpl::list` but for a single id, with the same N+1 discipline; (2) a transactional combined-create path for PADM-02/PADM-03, which surfaces a genuine architectural snag: `PermissionDao::create_user` already does the "create person + create user, atomically" dance internally — but via its **own local `pool.begin()` transaction**, not the workspace's shared `Transaction`/`TransactionDao` convention that `SalesPersonService::create` uses. A combined create that must create *both* a user and a sales person in one atomic unit (SC-2's "genau die gewählten Objekte") cannot simply call both existing service methods back-to-back without refactoring `PermissionDao::create_user`/`PermissionService::create_user` to accept the shared transaction type; (3) closing the D-63-01 orphan-cleanup gap — neither `PersonService` nor `PersonDao` currently has a `delete` method at all, and the exact trigger condition for "person lost its last Träger" needs a precise, code-checkable definition (documented under Common Pitfalls — this is a real design ambiguity, not a triviality).

On the frontend, the two pages being retired (`user_details.rs`, 333 lines; `sales_person_details.rs`, 339 lines) are the literal blueprint: every section the new component needs already exists as working RSX, using the established `USER_MANAGEMENT_STORE` global-signal + `UserManagementAction` coroutine pattern. The frontend has **zero** call path to `delete_sales_person` today (verified by grep) — D-63-06 requires wiring this up for the first time. The `/person` route prefix has **no** `Dioxus.toml` proxy entry yet (only `/person-directory` exists) — this is the single most expensive place to slip, given this exact bug class has shipped three times already in this repo.

**Primary recommendation:** Build one new business-logic-tier service (working title `PersonDetailService`, sibling to `PersonDirectoryServiceImpl`, constructed after the Basic block in `shifty_bin/src/main.rs`) that owns three new REST-exposed operations — `get_detail(person_id)`, `create_combined(display_name, user_name?, wants_sales_person)`, `remove_facet(...)` (orchestrating hard-delete-user / soft-delete-sales-person plus the D-63-01 orphan check) — while reusing the *existing* `SalesPersonService::set_user`/`unlinked_*` machinery unchanged for the link/unlink and candidate-picker flows.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Detail aggregate read (sales-person + roles + invitations for one person) | API / Backend (Business-Logic Service) | — | Cross-entity read combining `SalesPersonDao`/`PermissionDao`/`UserInvitationDao`; exactly the `PersonDirectoryServiceImpl` pattern, one row instead of all rows |
| Combined create (user + sales person in one transaction) | API / Backend (Business-Logic Service) | Database (transaction boundary) | Needs to span two Basic-tier services atomically — a Business-Logic-tier concern per this repo's own tier convention |
| Add-missing-facet ("Login hinzufügen" / "Person hinzufügen") | API / Backend (Business-Logic Service) | — | Same combined-create machinery, single-object slice, then link via existing `set_user` |
| Link-to-existing (candidate picker write) | API / Backend (Basic Service, already exists) | — | `SalesPersonService::set_user` already implements this correctly (D-61-04 merge semantics); no new backend code needed for the write |
| Candidate list read (unlinked sales persons / users) | API / Backend (Basic Service, already exists) | REST (new thin wrapper needed) | `PersonService::unlinked_sales_persons`/`unlinked_users` exist since Phase 61; only the REST route is missing |
| Remove-facet / delete with orphan cleanup | API / Backend (Business-Logic Service) | Database (soft/hard delete + person cleanup) | Orchestrates two Basic services (`SalesPersonService::delete`, `PermissionService::delete_user`) plus the new person-orphan check — cross-entity by definition |
| Section visibility by `PersonKindTO` | Browser / Client (pure helper function) | — | Must be a pure, unit-testable function per the Verifikations-Hinweis (Pitfall 10) — DOM assembly is Dioxus/WASM's job, the *decision* of what to show is not |
| Impact-summary text selection (delete confirmation copy) | Browser / Client (pure helper function) | — | D-63-08 fixes the text per side; picking which fixed text to show is a pure match on `PersonKindTO`/facet type |
| Route + breadcrumb wiring | Browser / Client / Frontend Server (SSR-rendered via Dioxus) | — | `router.rs`, `component/top_bar.rs` — pure routing config, no business logic |

## Project Constraints (from CLAUDE.md)

- **Layered architecture is non-negotiable:** REST (Axum) → Service (business logic) → DAO (data access) → SQLite. Every new service method takes `Option<Self::Transaction>`, calls `self.transaction_dao.use_transaction(tx)`, and commits itself.
- **Service-Tier Convention (verbatim from CLAUDE.md, and re-affirmed as a locked decision in this phase's CONTEXT.md):** Basic Services consume only DAOs + `PermissionService` + `TransactionDao` — never another domain service. Business-Logic Services may consume Basic and other Business-Logic services. `PersonService` **stays Basic-tier** in this phase; the new detail/create/remove-orchestration service **must** be Business-Logic-tier and constructed in `shifty_bin/src/main.rs` strictly after the Basic block — no `OnceLock`/forward-declaration tricks.
- **`#[utoipa::path(...)]` is mandatory** on every new REST handler; every new TO needs `ToSchema`; every new route needs an `ApiDoc` struct entry (see `PersonDirectoryApiDoc` for the pattern).
- **Billing Period Snapshot Schema Versioning:** `CURRENT_SNAPSHOT_SCHEMA_VERSION` stays at 13 — nothing in this phase touches a persisted `value_type`, computation, or input set. Locked in CONTEXT.md too; do not bump "just in case."
- **Clippy hard gate:** `cargo clippy --workspace -- -D warnings` must pass before every commit (not covered by `cargo test`).
- **Docs-Freshness-Gate table** — this phase's changes hit these trigger rows:
  - `shifty_bin/src/main.rs` DI wiring change → `docs/architecture/02-service-tiers.md` + `docs/architecture/diagrams/service-graph-runtime.mmd` (both EN and DE, i.e. `_de.md`/`.mmd` variants where applicable) — **only if** the DI graph visibly grows by a service (it will: the new detail/create/remove service).
  - `service/**` trait changes (new `PersonService::delete`? new `PersonDetailService` trait) → the doc that already carries the person-management narrative, which — verified by grep — is **`docs/features/F01-employee-management.md` + `F01-employee-management_de.md`** (Phase 62 extended this file, not a new `F*-person*.md` — the phrase in 63-CONTEXT.md's canonical_refs pointing at "die neue/erweiterte docs/features/F*.md zur Personenverwaltung" resolves to this exact file pair).
  - `dao/**` changes (new `PersonDao::delete`?) → same `docs/features/F01-employee-management.md` pair, data-model subsection.
  - `service_impl/src/permission.rs` untouched in this phase's core plan (no auth-gate change is planned) — **but** if the planner routes delete-user orchestration through a new call site here, re-check `docs/architecture/04-auth.md` + `docs/features/F12-auth-session.md`.
  - No `migrations/sqlite/*.sql` in this phase (locked: "Keine Migration in dieser Phase") → `docs/architecture/03-data-model.md` does **not** need a schema-shape update, but its `PersonDirectory`-related prose may need a one-line mention of the new delete/orphan-cleanup behavior for completeness (discretionary, not gate-triggering).
- **User's global CLAUDE.md:** "Always make sure you have tests for the changes" — reinforces the Verifikations-Hinweis's requirement for pure, unit-tested visibility/impact-text helpers plus the SSR rendertest harness pattern already established in `user_management.rs:506+`.

## Standard Stack

No new external dependency is introduced by this phase — it is 100% internal architecture (new service, new REST routes, new TOs, new Dioxus components) built on crates already in the workspace `Cargo.toml` lockfiles.

### Core (already in use, no version change)
| Library | Version (locked in repo) | Purpose | Why Standard (for this repo) |
|---------|---------|---------|--------------|
| axum | workspace-pinned | REST routing/handlers | Established pattern throughout `rest/` |
| utoipa | workspace-pinned | OpenAPI schema generation | Mandatory per CLAUDE.md §REST API |
| sqlx | workspace-pinned | Compile-time-checked SQL | Established DAO pattern; `SQLX_OFFLINE=true` in Nix shell |
| mockall | workspace-pinned | `#[automock]` trait mocks for service/DAO tests | Used on every trait in `service/`/`dao/` |
| dioxus / dioxus_ssr | workspace-pinned | Frontend components + SSR rendertest harness | Established pattern in `page/user_management.rs` tests |

### Alternatives Considered

None — no library-selection decision exists in this phase. All "alternatives" are internal design choices (endpoint shape, service split), captured as Claude's Discretion in CONTEXT.md, not library alternatives.

**Installation:** No new packages. `cargo build`/`cargo test` in the existing workspace via `nix develop --command bash -c '<cmd>'` (toolchain confirmed present: `cargo 1.95.0`, `rustc 1.95.0` inside the Nix devshell — see Environment Availability below).

## Package Legitimacy Audit

**Not applicable — this phase introduces zero new external packages** (verified: no new crate names appear anywhere in the CONTEXT.md decisions, and the Standard Stack section above consists entirely of crates already present in the workspace). The Package Legitimacy Gate protocol is a no-op here; nothing to check via `gsd-tools query package-legitimacy check`.

## Architecture Patterns

### System Architecture Diagram

```
Admin browser (Dioxus/WASM)
│
├─ Route "/person/:person_id/"  ─────────────► PersonDetail component (NEW)
│                                                 │
│                                                 ├─ on load: GET /person/{id}  ───────┐
│                                                 ├─ section visibility: pure helper   │
│                                                 │   fn section_visibility(kind) -> …  │
│                                                 └─ actions dispatch coroutine          │
│                                                     (UserManagementAction, extended)   │
│                                                                                        │
├─ Route "/user_management/"  ──► PersonDirectoryTable (existing, Phase 62)             │
│     │                                                                                  │
│     ├─ "Eintrag anlegen" (NEW, replaces 2 old buttons)                                │
│     │     └─ CreateEntryDialog ──► POST /person  (NEW, combined create)               │
│     │                                                                                  │
│     └─ row → edit_href = "/person/{person_id}/"  (was: 2 different routes)            │
│                                                                                        │
▼                                                                                        ▼
Axum REST layer (rest/)                                                    Axum REST layer (rest/)
│                                                                                        │
├─ GET  /person/{id}          (NEW) ───► PersonDetailService::get_detail                │
├─ POST /person               (NEW) ───► PersonDetailService::create_combined ──────────┘
├─ POST /person/{id}/facet    (NEW, PADM-03 "add missing facet")
├─ DELETE /person/{id}/facet/{kind} (NEW, PADM-07 remove-facet incl. orphan cleanup)
├─ GET  /person/candidates/sales-persons  (NEW, thin wrapper)  ──► PersonService::unlinked_sales_persons (EXISTS, Phase 61)
├─ GET  /person/candidates/users          (NEW, thin wrapper)  ──► PersonService::unlinked_users (EXISTS, Phase 61)
├─ POST /sales-person/{id}/user           (EXISTS, D-61-04)    ──► SalesPersonService::set_user  (link-to-existing write path)
├─ DELETE /sales-person/{id}/user         (EXISTS)             ──► SalesPersonService::set_user(None) ("Verknüpfung lösen")
├─ DELETE /sales-person/{id}              (EXISTS, HR-gated)   ──► SalesPersonService::delete (soft delete — frontend wiring is NEW, D-63-06)
├─ DELETE /permission/user/               (EXISTS, admin-gated)──► PermissionService::delete_user (hard delete, FK CASCADE → user_role, user_invitation)
└─ GET  /person-directory                 (EXISTS, Phase 62)   ──► PersonDirectoryService::list (unchanged, still backs the list page)
        │
        ▼
Business-Logic tier (service_impl/): PersonDetailServiceImpl (NEW)
   consumes: SalesPersonDao (or SalesPersonService), PermissionDao (or PersonService+PermissionService),
             UserInvitationDao, PersonDao, PermissionService, TransactionDao
   constructed in shifty_bin/src/main.rs AFTER the Basic block (same slot class as PersonDirectoryServiceImpl)
        │
        ▼
Basic tier (service_impl/): PersonService, SalesPersonService, PermissionService — UNCHANGED in this phase
        │
        ▼
DAO tier (dao_impl_sqlite/): PersonDao (+ new delete method), SalesPersonDao, PermissionDao — UNCHANGED except the new PersonDao::delete
        │
        ▼
SQLite: person, sales_person, user, user_role, user_invitation (schema UNCHANGED — no migration this phase)
```

### Recommended Project Structure

Backend (new files only; existing files get additive edits):
```
service/src/
└── person_detail.rs          # NEW — PersonDetailService trait (Business-Logic)
service_impl/src/
└── person_detail.rs          # NEW — PersonDetailServiceImpl
dao/src/
└── person.rs                 # EDIT — add PersonDao::delete (Basic, 5th method — see note below)
dao_impl_sqlite/src/
└── person.rs                 # EDIT — implement the new delete
rest/src/
├── person_detail.rs          # NEW — GET/POST/DELETE handlers + PersonDetailApiDoc
└── lib.rs                    # EDIT — .nest("/person", person_detail::generate_route())
rest-types/src/lib.rs         # EDIT — new TOs: PersonDetailTO, CreatePersonEntryTO, etc.
shifty_bin/src/main.rs        # EDIT — wire PersonDetailServiceImpl after Basic block
```

Frontend (mirrors the existing `page/`/`service/` split):
```
shifty-dioxus/src/
├── page/
│   ├── person_detail.rs      # NEW — replaces user_details.rs + sales_person_details.rs
│   ├── user_details.rs       # DELETE (D-63-04)
│   └── sales_person_details.rs  # DELETE (D-63-04)
├── service/
│   └── user_management.rs    # EDIT — extend UserManagementAction, add pure visibility helpers
├── router.rs                 # EDIT — remove UserDetails/SalesPersonDetails, add PersonDetail{person_id}
├── component/top_bar.rs      # EDIT — breadcrumb branches
└── Dioxus.toml                # EDIT — add [[web.proxy]] backend = "http://localhost:3000/person"
```

**Note on `PersonDao`'s "exactly four methods" lock:** `dao/src/person.rs`'s module header states the trait is *deliberately* capped at four methods "Vorratshaltung für Phase 62/63 ist ausdrücklich unerwünscht" (stockpiling for Phase 62/63 is explicitly unwanted) — but that lock was written *before* Phase 62/63 existed, as a guard against speculative methods nobody yet needed. Phase 63 now has a concrete, justified need (`delete`, for D-63-01 orphan cleanup) — this is exactly the kind of need the lock's own reasoning anticipates satisfying later. Adding a fifth method here is consistent with the lock's intent, not a violation of it; the planner should still note in the new method's doc comment why this is the first deliberate exception.

### Pattern 1: Detail aggregate = "PersonDirectoryServiceImpl, but for one id"

**What:** `PersonDirectoryServiceImpl::list` (`service_impl/src/person_directory.rs:112-152`) is the exact template for the new detail read: gate first, single-statement-per-concern reads, then assemble in Rust — never SQL-side `COALESCE`/derivation.

**When to use:** Building `PersonDetailServiceImpl::get_detail(person_id)`.

**Example (existing code to mirror, not to modify):**
```rust
// Source: service_impl/src/person_directory.rs:112-152
async fn list(
    &self,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[PersonEntry]>, ServiceError> {
    // Gate as the FIRST instruction, before any DB access.
    self.permission_service.check_permission("admin", context).await?;

    let tx = self.transaction_dao.use_transaction(tx).await?;
    let rows = self.person_directory_dao.list_directory_entries(tx.clone()).await?;
    // ONE bulk query for roles, not one call per row — apply the same
    // discipline to invitations/shiftplan-assignments in the detail read.
    let role_rows = self.person_directory_dao.all_user_roles(tx.clone()).await?;
    self.transaction_dao.commit(tx).await?;
    // ... assemble in Rust, sort in Rust (never SQLite collation) ...
}
```

For the detail aggregate, this pattern narrows to: one query for the person's sales-person row (if any), one for roles-for-this-user (existing `PermissionDao::roles_for_user`, already scoped to one user — no N+1 risk there since it's already a single lookup, not per-row), one for invitations (`UserInvitationDao`, existing `list_user_invitations`-style query keyed by username), and one for shiftplan assignments (existing sales-person-shiftplan DAO). Four bounded reads for one person is not an N+1 pattern — N+1 would be calling any of these once *per row* of a list, which does not apply here.

### Pattern 2: Combined create — the transaction-boundary snag

**What:** `SalesPersonService::create` (`service_impl/src/sales_person.rs:166-194`) already follows the shared-transaction convention correctly: it accepts `Option<Self::Transaction>`, uses `self.transaction_dao.use_transaction(tx)`, and — critically — **does not commit until the caller's transaction commits**, if the caller passes its own `tx.clone()` through. This is exactly the mechanism a combined create needs: call `sales_person_service.create(&sp, ctx, Some(tx.clone()))` and it will participate in the outer transaction instead of committing early.

`PermissionService::create_user` (`service_impl/src/permission.rs:121-133`) does **not** follow this convention — it has no `tx` parameter at all in its trait signature (`service/src/user_service.rs` doesn't even define one for user creation; check `service/src/lib.rs` `PermissionService::create_user` signature directly). Worse, `PermissionDaoImpl::create_user` (`dao_impl_sqlite/src/lib.rs:95-135`) opens its **own local** `self.pool.begin()` transaction internally to atomically write both the `person` row and the `user` row — a transaction that is invisible to, and cannot be joined with, the outer `TransactionImpl`/`TransactionDao` machinery used everywhere else in this codebase.

**Why this matters for PADM-02:** SC-2 requires "nach dem Speichern existieren genau die gewählten Objekte" (after saving, exactly the chosen objects exist) — i.e. if the combined create is asked for *both* a user and a sales person, and the sales-person insert fails after the user insert already committed (because `create_user`'s internal transaction already committed independently), the invariant breaks: a stray user+person exists with no sales person, silently.

**Options for the planner (Claude's Discretion in CONTEXT.md explicitly defers "Form des Anlege-Endpoints" to the planner):**
1. **Refactor `PermissionDao::create_user`/`PermissionService::create_user` to accept the shared `Transaction` type**, matching every other Basic service in the codebase. This is the architecturally consistent fix and removes the special-cased local `pool.begin()`. Blast radius: `dao/src/permission.rs` trait signature change ripples to `dao_impl_sqlite/src/lib.rs`'s impl and every existing caller of `PermissionService::create_user`/`PermissionDao::create_user` (currently: `rest/src/permission.rs::add_user` only, per the grep of `create_user` callers — low blast radius).
2. **Have the new combined-create endpoint accept partial atomicity as a documented, tested edge case**: create the user first (already internally atomic person+user), then the sales person in the *same* new outer transaction seeded from the same `person_id`; on sales-person-creation failure, explicitly roll back by deleting the just-created user+person as a compensating action. More code, weaker guarantee, avoids touching `PermissionDao`'s signature.
3. **Route combined-create through direct DAO calls** in the new Business-Logic service (bypass `PermissionService`/`SalesPersonService` create methods entirely, write `person`+`sales_person`+`user` rows directly via `PersonDao`/`SalesPersonDao`/`PermissionDao` under one shared transaction) — most control, but duplicates validation/permission logic that the Basic services already carry (e.g. `SalesPersonService::create`'s HR-gate check, id/version-must-be-nil checks).

Option 1 is the cleanest fit for this codebase's own stated conventions and should be the default recommendation; the planner should pick explicitly and document the choice, since CONTEXT.md leaves it open on purpose.

### Pattern 3: Link-to-existing — nothing new to build for the write path

**What:** The entire PADM-04 write path already exists and is production-hardened:
```rust
// Source: service_impl/src/sales_person.rs:316-379 (excerpted)
async fn set_user(&self, sales_person_id: Uuid, user_id: Option<Arc<str>>, context, tx)
    -> Result<(), ServiceError> {
    // ... HR-gate ...
    match self.sales_person_dao.discard_assigned_user(sales_person_id, tx.clone()).await {
        Ok(()) => {}
        Err(dao::DaoError::EntityAlreadyExists) => {
            return Err(ServiceError::EntityAlreadyExists(sales_person_id)); // -> 409
        }
        Err(other) => return Err(other.into()),
    }
    if let Some(user) = user_id {
        match self.sales_person_dao.assign_to_user(sales_person_id, user.as_ref(), ..).await {
            Ok(()) => {}
            Err(dao::DaoError::EntityAlreadyExists) => {
                return Err(ServiceError::EntityAlreadyExists(sales_person_id)); // -> 409, tx rolled back
            }
            Err(other) => return Err(other.into()),
        }
    }
    self.transaction_dao.commit(tx).await?;
    Ok(())
}
```
Exposed at `POST /sales-person/{id}/user` (link, body = username string) and `DELETE /sales-person/{id}/user` (unlink — "Verknüpfung lösen", D-63-05). Both routes exist in `rest/src/sales_person.rs:26-28` today, already utoipa-documented with the 409 response.

**When to use:** The new candidate-picker UI (PADM-04) should call these **existing** endpoints unchanged. Do not build a new link endpoint.

**What's actually missing:** REST exposure for the two candidate-list reads. `PersonService::unlinked_sales_persons`/`unlinked_users` (`service/src/person.rs:83-94`) exist and are tested since Phase 61, but grep confirms **zero** REST route references them (`grep -rn "unlinked_sales_persons\|unlinked_users" rest/` returns nothing). A thin wrapper analogous to `rest/src/person_directory.rs` is the only new backend code PADM-04 needs.

### Anti-Patterns to Avoid

- **Deriving `PersonKindTO`/display logic in the frontend.** The repo has a hard, repeatedly-enforced rule (PADM-06, grep-verified pattern across v2.4/v2.5/Phase 62): the backend always sends the ready-to-render kind/name; the frontend interpolates, never infers. Applies equally to the new detail view's section visibility and to the delete-confirmation copy selection.
- **A collective "delete entry" button.** Explicitly forbidden by D-63-05 — never combine the hard-delete-user and soft-delete-sales-person actions behind one label/handler.
- **Building a new merge/link endpoint.** PADM-04's write semantics are already correct and tested (D-61-04); reinventing them risks diverging from the established 409 contract.
- **SQL-side sorting or name-derivation for the detail read.** Sort/derive in the Business-Logic service, exactly as `PersonDirectoryServiceImpl::list` does — SQLite's default TEXT collation is case-sensitive and would silently misorder.
- **Forgetting the `Dioxus.toml` proxy entry for `/person`.** This exact bug (missing `[[web.proxy]]` → dev-server 404/405, invisible to any unit test) has shipped three times in this repo (v1.6 `/toggle`, v1.8 `/vacation-entitlement-offset`, v1.10 `/special-days`). The only real detection method is a `dx serve` smoke test against the new route, not a green `cargo test`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Link/merge conflict detection (two objects already linked) | A new "check if already linked" validation layer | Existing `SalesPersonService::set_user` + `assign_to_user`/`discard_assigned_user` DAO methods, which already translate `DaoError::EntityAlreadyExists` → `ServiceError::EntityAlreadyExists` → HTTP 409 | This exact race-condition-safe logic was built and reviewed in Phase 61 (D-61-04); duplicating it risks a second, divergent 409 contract |
| Unlinked-candidate filtering | A new anti-join query | Existing `PersonDao::sales_persons_without_user`/`users_without_sales_person` (`dao/src/person.rs:59-79`), already correctly handling the soft-delete-in-ON-clause gotcha (D-62-05/WR-02 fix) | Getting the soft-delete filter placement wrong (WHERE vs. ON) silently degenerates a LEFT JOIN into an INNER JOIN — this was a real bug found and fixed in Phase 62; the fix is already applied to these exact methods |
| Cascading delete of roles/invitations on hard user-delete | Explicit DELETE statements for `user_role`/`user_invitation` | Existing FK `ON DELETE CASCADE` (verified: `migrations/sqlite/20240426150045_user-roles.sql:55` for `user_role`, `migrations/sqlite/20251016154210_add-user-invitation-table.sql:9` for `user_invitation`) | The cascade is already schema-enforced; adding redundant application-level deletes would just be dead code that could drift from the schema |
| Person-orphan detection | Ad-hoc "does this person have anything left" query scattered per call site | A single new `PersonDao`/`PersonDetailService` helper, queried once from the remove-facet orchestration path | Centralizing avoids two different, possibly-inconsistent definitions of "orphaned" appearing in create vs. delete flows |

**Key insight:** This phase's backend risk is not "what library do I need" — it's "which of the three already-built layers (Basic PersonService, Basic SalesPersonService, Basic PermissionService) does each new capability route through, and where exactly does a shared transaction need to span two of them." Every piece of *domain* logic (merge semantics, candidate anti-joins, cascade deletes) is already implemented correctly; the new work is almost entirely orchestration and REST/frontend plumbing.

## Common Pitfalls

### Pitfall 1: D-63-01's "last Träger" is genuinely ambiguous at the soft-delete boundary — pin the exact rule before implementing

**What goes wrong:** D-63-01 says the `person` row is deleted "sobald sie ihre letzte Ausprägung verliert" (as soon as it loses its last facet), reasoning that afterward the row "von nichts mehr referenziert wird" (is referenced by nothing). But "Verkäufer löschen" is a **soft** delete (`service_impl/src/sales_person.rs:253-280` — sets `deleted` timestamp, the row and its `person_id` FK-less pointer physically remain). So after soft-deleting the only sales person of a `SalesPersonOnly` entry, the `sales_person` row **still exists** and still carries `person_id`. Two readings are both defensible from the CONTEXT text alone:
  - **(a) Schema-literal reading:** a person is only "trägerlos" when zero rows (of either liveness) reference `person_id` — under this reading, D-63-01 essentially never fires for the sales-person side in this phase, since soft-delete never removes the row. It would only fire for `UserOnly` + hard-delete-login.
  - **(b) Directory-visible reading:** a person is "trägerlos" when zero **live** (non-soft-deleted) rows reference it — under this reading, soft-deleting the last live sales person of a `SalesPersonOnly` entry *does* orphan the person row (even though a dead `sales_person` row with a now-dangling `person_id` remains), and the person row should be deleted.

**Why it happens:** D-60-01's own reasoning for keeping soft-deleted sales persons' `person_id` intact ("historische billing_period_sales_person-Snapshots einer anderen Person zuordnen") is about *not reassigning* a person identity, not about the `person` row's continued literal existence — `billing_period_sales_person` snapshots reference `sales_person.id`, never `person.id` directly (verified: nothing in `service_impl/src/billing_period_report.rs`'s snapshot schema touches `person`). So deleting the `person` row while a soft-deleted `sales_person` row still dangles a `person_id` at it does **not** break snapshot continuity — but it does leave a `sales_person.person_id` value that resolves to nothing, which is a bit of a "living dead" state, an artifact this repo has otherwise never had.

**How to avoid:** The planner should pick reading (b) — it matches "keine Admin-Sicht auf trägerlose Personen" more literally (a soft-deleted-only entry is already invisible in the directory today per D-62-04's join filter, so nothing regresses either way, but only (b) actually reduces the number of dead `person` rows over time, which is the stated intent) — and implement the orphan-check as: *after this facet-removal action, does the person have (i) a user row (any — `user` has no soft-delete concept) OR (ii) a **live** `sales_person` row (`deleted IS NULL`)? If neither, delete the `person` row.* This check must run **after** the soft-delete/hard-delete write, in the same transaction, in the new Business-Logic-tier orchestration service — never in the Basic-tier `SalesPersonService::delete`/`PermissionService::delete_user` themselves (tier rule: Basic services don't know about `person`-level cross-entity concerns).

**Warning signs:** If the planner's remove-facet endpoint doesn't call any new "is this person now orphaned" query, D-63-01 is not actually implemented — the person row will accumulate as dead weight for every future sales-person-only deletion, forever.

### Pitfall 2: `PermissionDao::create_user`'s private local transaction breaks the shared-transaction assumption for combined create

Covered in full under Pattern 2 above. Restated as a pitfall: if a plan naively writes `sales_person_service.create(...).await?; permission_service.create_user(...).await?;` back-to-back inside the new combined-create handler *without* addressing the transaction-boundary mismatch, SC-2's "genau die gewählten Objekte" atomicity guarantee silently does not hold on partial failure. This is easy to miss because both calls individually "work" in isolation and in the happy path — the gap only shows under failure-injection testing (e.g. a validation error on the sales-person side after the user was already committed).

### Pitfall 3: Deleting a user with a live session will hit an unhandled FK violation

**What goes wrong:** `session.user_id` has a plain `FOREIGN KEY (user_id) REFERENCES user (name)` with **no** `ON DELETE CASCADE` (`migrations/sqlite/20241116224840_add-session.sql:8`), unlike `user_role`/`user_invitation` which do cascade. Since sqlx sets `PRAGMA foreign_keys = ON` on every connection, calling `DELETE FROM user WHERE name = ?` (`dao_impl_sqlite/src/lib.rs:148-153`, unchanged by this phase) while that user still has an active `session` row will raise a SQLite foreign-key-constraint error, which `map_db_error` will surface as a generic `DatabaseQueryError` → HTTP 500, not a clean, explainable failure.

**Why it happens:** This is pre-existing behavior (not introduced by this phase), but D-63-08's confirmed claim ("Login, Rollen und offene Einladungen werden endgültig gelöscht") implicitly assumes the delete always succeeds cleanly. It has likely gone unnoticed because `delete_user` today is invoked rarely and mostly against test/seed users without active sessions.

**How to avoid:** Not required to be fixed in this phase (out of the locked decisions and not listed as a success criterion), but the planner should at minimum note this as a known edge case in the UAT/verification pass for "Login löschen" — deleting a currently-logged-in admin's own login, or another admin's login while they have an open session, is the exact scenario that will surface a raw 500 instead of the D-63-08 confirmation flow completing. Flagging it now avoids it being discovered as a surprise defect during UAT.

**Warning signs:** A UAT/manual test that deletes a user who is *currently logged in during the test* (a very plausible admin-testing-on-themselves scenario) will hit this immediately.

### Pitfall 4 (carried from CONTEXT.md's Verifikations-Hinweis / Pitfall 10): DOM-rendering logic is hard to verify headless

Section visibility by `PersonKindTO` and delete-confirmation text selection must be pulled into pure, non-Dioxus functions (e.g. `fn visible_sections(kind: PersonKindTO) -> SectionSet`, `fn delete_confirmation_copy(facet: FacetKind) -> (&'static str, &'static str)`) that get direct unit tests, with the SSR rendertest harness (`user_management.rs:506+` pattern, `dioxus_ssr::render`) as the second layer verifying the helper's output actually reaches the DOM. A **Human-UAT checkpoint must be planned from the start** (not discovered at close) per the Verifikations-Hinweis — this mirrors the two `human_needed` overrides from v2.6 that were retrofitted late; this phase should not repeat that pattern.

### Pitfall 5: Missing `Dioxus.toml` proxy entry for the new `/person` prefix

`grep` confirms `Dioxus.toml` today has a proxy entry for `/person-directory` but **not** for `/person`. Since `/person` and `/person-directory` are different path prefixes (not a sub-path relationship from the proxy matcher's perspective), the new detail/create/candidate/remove-facet routes need their **own** `[[web.proxy]] backend = "http://localhost:3000/person"` entry. This exact bug class (missing proxy entry → 404/405 in `dx serve`, invisible to `cargo test`) has shipped three times in this repository already. The only reliable detection is running `dx serve` and exercising the new route in a browser or via `curl` against the dev-server port, not a unit test.

## Code Examples

### Existing 409-conflict translation pattern (reuse verbatim for any new conflict-prone write)
```rust
// Source: service_impl/src/sales_person.rs:339-355 (already in production)
match self.sales_person_dao.assign_to_user(sales_person_id, user.as_ref(), PROCESS, tx.clone()).await {
    Ok(()) => {}
    Err(dao::DaoError::EntityAlreadyExists) => {
        // No commit: the transaction is discarded.
        return Err(ServiceError::EntityAlreadyExists(sales_person_id));
    }
    Err(other) => return Err(other.into()),
}
```

### Existing admin-gate-first pattern for any new Business-Logic-tier read/write
```rust
// Source: service_impl/src/person_directory.rs:117-122 (already in production)
async fn list(&self, context: Authentication<Self::Context>, tx: Option<Self::Transaction>)
    -> Result<Arc<[PersonEntry]>, ServiceError> {
    // Gate as the FIRST instruction, before any DB access — no partial response,
    // no field-level redaction on missing privilege.
    self.permission_service.check_permission("admin", context).await?;
    // ...
}
```

### Existing component-level frontend admin guard (reuse for the new detail page)
```rust
// Source: shifty-dioxus/src/page/user_management.rs:60-92 (already in production)
pub(crate) fn is_admin(auth_info: Option<&AuthInfo>) -> bool {
    auth_info.map(|a| a.has_privilege("admin")).unwrap_or(false)
}

#[component]
pub fn NotAuthorizedView() -> Element {
    rsx! { TopBar {} div { class: "p-md text-ink-muted", "Not authorized." } }
}
// in the page component, BEFORE any hook or data access:
if !is_admin(auth.auth_info.as_ref()) {
    return rsx! { NotAuthorizedView {} };
}
```

### Existing SSR rendertest harness (reuse for section-visibility tests)
```rust
// Source: shifty-dioxus/src/page/user_management.rs:474-497 (already in production, pattern to copy)
fn render_directory(entries: Vec<PersonEntryTO>, search_initial: &str) -> String {
    let mut vdom = VirtualDom::new_with_props(DirectoryWrapper, WrapperProps { /* ... */ });
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}
// New detail-view tests should follow: render_detail(entry: PersonEntryTO) -> String,
// then assert!(html.contains(...)) / assert!(!html.contains(...)) per PersonKindTO.
```

## State of the Art

Not applicable in the conventional sense (no external library evolution to track) — but one internal "state of the art" shift is directly relevant: **Phase 62's UAT already flagged the exact regression this phase must fix.** Verbatim from `62-07-SUMMARY.md`: *"Es gibt jetzt trotzdem neuen benutzer und neuen verkaeufer separat. Das checkt niemand."* — the two-button create UI became *worse* after the Phase-62 list-merge (it lost the implicit tab-context that used to explain which kind of object each button created). D-63-03 is the fix, already locked.

**Deprecated/outdated (as of this phase):**
- `Route::UserDetails` / `Route::SalesPersonDetails` — replaced by `Route::PersonDetail { person_id }` (or equivalent name), no redirect shims (D-63-04).
- `Key::CreateNewSalesPerson` / `Key::AddNewUser` as toolbar entry points — replaced by the single combined create dialog (D-63-03); the i18n keys themselves may be reused inside the new dialog's copy rather than deleted outright (Claude's Discretion).
- The primitive "type a username to connect" UI in `sales_person_details.rs:189-225` (`Key::ConnectUserAccount`) — replaced by the candidate picker (PADM-04), which reuses its underlying `UpdateSalesPersonUser`/`RemoveSalesPersonUser` actions or their successors but changes the input mechanism from free-text to a bounded candidate list.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | D-63-01's "last Träger" should be read as "last **live** facet" (soft-deleted sales persons count as gone for orphan-detection purposes), not "last **row**, live or dead" | Common Pitfalls #1 | If the alternate (schema-literal) reading is actually intended, implementing A1's reading deletes `person` rows the product owner expected to keep referenced by a dangling `sales_person.person_id` — low blast radius since nothing else in the schema depends on `person.id` continuing to exist (no FK), but worth a one-line confirmation during planning/discuss rather than silently picking a side |
| A2 | Refactoring `PermissionDao::create_user`/`PermissionService::create_user` to accept the shared `Transaction` type (Pattern 2, Option 1) is the preferred fix for the combined-create atomicity gap, over the compensating-action or direct-DAO alternatives | Architecture Patterns, Pattern 2 | If the planner instead chooses Option 2 or 3, the atomicity guarantee is weaker or validation logic gets duplicated — not incorrect, just a different (documented) tradeoff; flagged as Claude's Discretion in CONTEXT.md so this is explicitly not a locked decision |
| A3 | The Docs-Freshness-Gate target for this phase's `service/**`/`dao/**` changes is `docs/features/F01-employee-management.md` (+ `_de.md`), not a new/different `F*.md` file, based on grep evidence that Phase 62 already extended exactly this file for the same subject matter | Project Constraints (from CLAUDE.md) | If wrong, the planner documents the changes in the wrong file and the Docs-Freshness-Gate PR-review check would flag it — low risk since it's a grep-verifiable claim, re-checkable in seconds at plan time |

## Open Questions

1. **Does "Verkäufer löschen" on a `SalesPersonOnly` entry trigger person-row deletion under D-63-01, given the soft-delete row survives?**
   - What we know: The directory query already excludes soft-deleted sales persons from being visible (D-62-04), so the *user-facing* outcome ("entry disappears") is identical whether or not the underlying `person` row is also deleted.
   - What's unclear: Whether D-63-01's "keine Waisen" intent extends to cleaning up the now-referenced-by-nothing-live `person` row, or whether it was scoped by the user only to the hard-delete-user case.
   - Recommendation: Resolve via Common Pitfalls #1 / Assumption A1 (delete it) unless the planner or a discuss-phase follow-up surfaces a reason not to; the risk of over-deleting here is low (no downstream reference depends on `person.id` surviving a soft-deleted sales person).

2. **Should the combined-create transaction-boundary fix (Pattern 2) land as a `PermissionDao` signature refactor in this same phase, or should the phase accept a documented weaker-atomicity fallback to keep blast radius smaller?**
   - What we know: The refactor's blast radius is small (one existing caller: `rest/src/permission.rs::add_user`).
   - What's unclear: Whether the phase's existing 5-requirement scope has room for a cross-cutting DAO trait signature change, or whether the planner should treat this as a follow-up hardening task.
   - Recommendation: Given the low blast radius and that SC-2 explicitly demands the atomicity guarantee ("genau die gewählten Objekte"), do the refactor in this phase rather than deferring — deferring would mean PADM-02 ships without its literal success criterion met under failure conditions.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo/rustc (via `nix develop`) | All backend build/test/clippy work | ✓ | cargo 1.95.0, rustc 1.95.0 | — |
| Nix | Devshell entry point | ✓ | 2.34.8 | — |
| node (for Dioxus/WASM frontend tooling) | Frontend `dx serve` smoke test | Not checked directly on bare PATH (repo's own LEARNINGS note node/cargo are absent from default PATH) | — | `nix develop --command <cmd>` per repo convention (confirmed working for cargo) |
| SQLite dev DB | `sqlx` compile-time query checking | ✓ (`localdb.sqlite3` present at repo root, plus a pre-60-04 backup) | — | `SQLX_OFFLINE=true` per CLAUDE.md if DB drifts |

**Missing dependencies with no fallback:** None identified — this phase needs no new external tool or service.

**Missing dependencies with fallback:** `node`/`dx` CLI access is routed through `nix develop --command bash -c '<cmd>'` per this repo's established toolchain convention (documented in CONTEXT.md's "Toolchain" note and prior phase LEARNINGS) — not a gap, just a required invocation pattern.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Backend framework | `cargo test` (workspace), mock-based unit tests via `mockall` + `#[automock]`, integration tests with in-memory SQLite (`service_impl/src/test/`) |
| Frontend framework | `cargo test` inside `shifty-dioxus` crate; SSR rendertests via `dioxus_ssr::render` against a `VirtualDom` built from injected props (no store, no network) — established in `page/user_management.rs:474+` |
| Config file | Workspace `Cargo.toml` at repo root; no separate test-framework config file |
| Quick run command | `nix develop --command bash -c "cargo test -p service_impl person_detail"` (backend), `nix develop --command bash -c "cargo test --manifest-path shifty-dioxus/Cargo.toml person_detail"` (frontend) |
| Full suite command | `nix develop --command bash -c "cargo test --workspace"` then `nix develop --command bash -c "cargo clippy --workspace -- -D warnings"` (clippy is a separate hard gate, not covered by `cargo test`) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PADM-02 | Combined create produces exactly the chosen objects, atomically on partial failure | integration (service_impl) | `cargo test -p service_impl person_detail::create_combined` | ❌ Wave 0 — new test module |
| PADM-03 | Add-missing-facet creates and links the missing object without touching the existing one | integration (service_impl) | `cargo test -p service_impl person_detail::add_facet` | ❌ Wave 0 |
| PADM-04 | Candidate list excludes already-linked objects; server still rejects (409) an already-linked target if requested directly | integration (rest + service_impl, reusing existing `set_user` 409 tests as the baseline) | `cargo test -p service_impl sales_person::set_user` (existing, verify still green) + new candidate-list test | Existing 409 tests: ✅ (Phase 61); candidate-list REST test: ❌ Wave 0 |
| PADM-05 | Detail aggregate returns exactly the sections the entry's `PersonKindTO` implies, no N+1 | unit (pure section-visibility helper) + integration (aggregate read) | `cargo test -p service_impl person_detail::get_detail` + `cargo test --manifest-path shifty-dioxus/Cargo.toml visible_sections` | ❌ Wave 0 (both) |
| PADM-07 | Remove-facet orchestration deletes the right things (hard vs soft) and cleans up an orphaned `person` row | integration (service_impl) | `cargo test -p service_impl person_detail::remove_facet` | ❌ Wave 0 |
| PADM-07 (frontend copy) | Delete-confirmation text matches D-63-08's fixed per-side copy | unit (pure helper) | `cargo test --manifest-path shifty-dioxus/Cargo.toml delete_confirmation_copy` | ❌ Wave 0 |
| (cross-cutting) | Section visibility never derived from role, only from `PersonKindTO` | unit (pure helper) + SSR rendertest | `cargo test --manifest-path shifty-dioxus/Cargo.toml person_detail_visibility` | ❌ Wave 0 |
| (cross-cutting) | `Dioxus.toml` proxy entry for `/person` actually works | manual/human — `dx serve` smoke | N/A (not automatable per this repo's own established pattern for this bug class) | N/A |

### Sampling Rate
- **Per task commit:** targeted `cargo test -p <crate> <module>` for the module just touched, plus `cargo clippy --workspace -- -D warnings` (hard gate, cheap enough to run every commit).
- **Per wave merge:** `cargo test --workspace` (full suite).
- **Phase gate:** Full suite green + `dx serve` smoke against `/person/{id}` (both an existing linked entry and a freshly created user-only entry) before `/gsd-verify-work`, plus the Human-UAT pass planned from the start (Pitfall 10 / Verifikations-Hinweis — do not defer this to phase close).

### Wave 0 Gaps
- [ ] `service_impl/src/test/person_detail.rs` (or equivalent module) — covers PADM-02, PADM-03, PADM-05, PADM-07 service-level behavior
- [ ] `shifty-dioxus/src/page/person_detail.rs` `#[cfg(test)] mod tests` — covers section-visibility and delete-confirmation-copy pure helpers, plus SSR rendertests per `PersonKindTO` variant (mirrors `user_management.rs:506+`)
- [ ] REST-level tests for the new `/person/*` routes (candidate lists, detail GET, combined-create POST, remove-facet DELETE) — likely colocated with existing `rest/src/person_directory.rs`-style tests if any exist, or added fresh
- [ ] No new test framework install needed — everything reuses `cargo test` + `mockall` + `dioxus_ssr`, all already present

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No change in this phase | Existing OIDC/mock-auth unaffected |
| V3 Session Management | Indirectly (Pitfall 3 — delete-user vs. live session FK) | No new control needed to satisfy locked scope; flagged as a known edge case, not a phase requirement |
| V4 Access Control | Yes | All new endpoints must gate on `"admin"` privilege as the **first** instruction, matching `PersonDirectoryServiceImpl::list`'s established pattern (D-62-08) — not `HR_PRIVILEGE`, which is what the pre-existing `SalesPersonService`/`PermissionService` Basic-tier methods use internally; the new Business-Logic-tier orchestration adds the stricter admin gate on top, consistent with PADM-08 |
| V5 Input Validation | Yes | Combined-create dialog: "at least one toggle must be on" is a business-rule validation, not a schema one — enforce server-side too (D-63-07 says "Mindestens ein Umschalter muss an sein" — this must not be a frontend-only guard) |
| V6 Cryptography | No | Not touched |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Direct URL access to `/person/:id/` bypassing the admin-only nav | Elevation of Privilege | Component-level guard (`is_admin` early-return, already the established pattern) is defense-in-depth; the **authoritative** gate is the backend admin-check on every new endpoint — do not rely on frontend gating alone |
| Client-supplied `kind`/derived display data trusted instead of recomputed server-side | Tampering | N/A here — this repo's established Fat-Backend convention means the client never sends derived state back for interpretation; the new create/facet endpoints should take only raw inputs (name, toggles, username) and let the server derive everything else |
| Race on "link to existing candidate" (two admins linking the same target simultaneously) | Tampering / Repudiation-adjacent race | Already handled — `DaoError::EntityAlreadyExists` → 409 (D-61-04), reused unchanged by this phase |
| Orphan `person` rows accumulating as a low-grade resource/data-hygiene issue if D-63-01 is not implemented correctly | Information Disclosure (indirectly — stale identities lingering) | Correct implementation of the orphan-check per Common Pitfalls #1 |

## Sources

### Primary (HIGH confidence — direct codebase reads this session)
- `service/src/person.rs`, `service/src/person_directory.rs` (via `service_impl/src/person_directory.rs` usage), `service_impl/src/person_directory.rs`, `service_impl/src/sales_person.rs`, `service_impl/src/permission.rs`, `dao/src/person.rs`, `dao/src/person_directory.rs`, `dao/src/permission.rs`, `dao_impl_sqlite/src/person.rs`, `dao_impl_sqlite/src/person_directory.rs`, `dao_impl_sqlite/src/lib.rs`, `rest/src/person_directory.rs`, `rest/src/sales_person.rs`, `rest/src/permission.rs`, `rest/src/user_invitation.rs`, `rest-types/src/lib.rs` (`PersonEntryTO`/`PersonKindTO` region), `shifty_bin/src/main.rs` (DI wiring), `migrations/sqlite/20260916000000_create-person-backfill.sql`, `migrations/sqlite/20260917000000_drop-sales-person-user.sql`, `migrations/sqlite/20240426150045_user-roles.sql`, `migrations/sqlite/20251016154210_add-user-invitation-table.sql`, `migrations/sqlite/20241116224840_add-session.sql`
- `shifty-dioxus/src/page/user_details.rs`, `shifty-dioxus/src/page/sales_person_details.rs`, `shifty-dioxus/src/page/user_management.rs`, `shifty-dioxus/src/router.rs`, `shifty-dioxus/src/component/top_bar.rs`, `shifty-dioxus/src/page/settings.rs`, `shifty-dioxus/src/service/user_management.rs`, `shifty-dioxus/Dioxus.toml`, `shifty-dioxus/src/i18n/{de,en,cs}.rs`
- `.planning/phases/63-personen-detailansicht-auspr-gungs-aktionen/63-CONTEXT.md` (locked product decisions, exhaustive canonical refs)
- `.planning/phases/62-personen-verzeichnis-eine-liste-statt-tabs/62-07-SUMMARY.md` (UAT findings that directly shaped D-63-03)
- `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, `.planning/config.json`, `/home/neosam/programming/projects/shifty-backend/CLAUDE.md`

### Secondary (MEDIUM confidence)
- None — no external documentation was needed; this phase's domain is entirely internal to the codebase already read directly.

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependency, everything verified as already present and in active use.
- Architecture: HIGH for what exists (verified by direct file reads); MEDIUM for the two open design questions (D-63-01 exact orphan semantics, combined-create transaction-boundary fix approach) — both are explicitly flagged as Claude's Discretion in CONTEXT.md or as genuine ambiguities in Assumptions Log/Open Questions, not guessed silently.
- Pitfalls: HIGH — all five pitfalls are grounded in direct code/migration reads (FK cascade behavior, missing proxy entry precedent, trait signature mismatch), not speculation.

**Research date:** 2026-09-20
**Valid until:** No external time-decay risk (no library-version dependency); valid until the codebase itself changes underneath this phase — effectively until this phase starts execution, given the plan will be created immediately after.
