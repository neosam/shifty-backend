# Phase 63: Personen-Detailansicht + Ausprägungs-Aktionen - Pattern Map

**Mapped:** 2026-09-20
**Files analyzed:** 17 (backend 8, frontend 9)
**Analogs found:** 17 / 17

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `service/src/person_detail.rs` (NEW) | service (trait, business-logic) | request-response / CRUD (aggregate read + orchestrated write) | `service/src/person_directory.rs` | exact |
| `service_impl/src/person_detail.rs` (NEW) | service (business-logic impl) | request-response / CRUD | `service_impl/src/person_directory.rs` | exact |
| `dao/src/person.rs` (EDIT — add `delete`) | dao (trait) | CRUD | existing `PersonDao::create`/`find_by_id` in same file | exact (same file, same trait) |
| `dao_impl_sqlite/src/person.rs` (EDIT — implement `delete`) | dao (sqlite impl) | CRUD | existing `PersonDao` impl in same file | exact |
| `rest/src/person_detail.rs` (NEW) | route/controller | request-response | `rest/src/person_directory.rs` | exact |
| `rest/src/lib.rs` (EDIT — nest new router + ApiDoc) | route (wiring) | request-response | existing `.nest("/person-directory", …)` block | exact |
| `rest-types/src/lib.rs` (EDIT — new TOs) | model (DTO) | transform | `PersonEntryTO`/`PersonKindTO` (lines ~2854-2925) | exact |
| `shifty_bin/src/main.rs` (EDIT — DI wiring) | config (DI wiring) | — | `PersonDirectoryServiceImpl` construction block (~1371-1379) + its Deps struct (~354-376) | exact |
| `service_impl/src/permission.rs` / `dao_impl_sqlite/src/lib.rs` (EDIT — `create_user` tx refactor, Pattern 2 Option 1) | service/dao (auth) | CRUD | `SalesPersonService::create` (`service_impl/src/sales_person.rs:166-194`) — the shared-tx convention `create_user` must adopt | role-match |
| `shifty-dioxus/src/page/person_detail.rs` (NEW) | component (page) | request-response | `shifty-dioxus/src/page/user_details.rs` + `shifty-dioxus/src/page/sales_person_details.rs` | exact (literal port + retokenize) |
| `shifty-dioxus/src/page/user_details.rs` (DELETE) | component (page) | — | — | n/a (removed) |
| `shifty-dioxus/src/page/sales_person_details.rs` (DELETE) | component (page) | — | — | n/a (removed) |
| `shifty-dioxus/src/page/user_management.rs` (EDIT — toolbar, row actions) | component (page) | request-response | same file, existing toolbar/dialog code | exact |
| `shifty-dioxus/src/router.rs` (EDIT) | route (frontend) | — | existing `UserDetails`/`SalesPersonDetails` route entries in same file | exact |
| `shifty-dioxus/src/component/top_bar.rs` (EDIT — breadcrumbs) | component | — | existing breadcrumb branches for `UserDetails`/`SalesPersonDetails` (lines 94-95, 883-889) | exact |
| `shifty-dioxus/src/service/user_management.rs` (EDIT — extend coroutine, add pure helpers) | service (frontend action dispatch) | event-driven (coroutine) | same file's existing `UserManagementAction` enum + match arms | exact |
| `shifty-dioxus/src/i18n/{de,en,cs}.rs` (EDIT — new keys) | config (i18n) | — | existing `Key::UserManagement`/`Key::AddNewUser` entries in same files | exact |
| `shifty-dioxus/Dioxus.toml` (EDIT — new proxy entry) | config | — | existing `[[web.proxy]] backend = "http://localhost:3000/person-directory"` (line 117-118) | exact |

## Pattern Assignments

### `service/src/person_detail.rs` + `service_impl/src/person_detail.rs` (business-logic service, request-response/CRUD)

**Analog:** `service_impl/src/person_directory.rs` (full file — 152 lines, read in full)

**Module-header pattern to copy** (tier justification comment block, lines 1-27):
```rust
//! Phase 62 Plan 03 (PADM-01 / PADM-06 / PADM-08): Business-Logic-Tier
//! `PersonDirectoryService` — das Aggregat hinter der einen Personen-Liste.
//!
//! **Tier: Business-Logic.** Der Service fuehrt zwei Fach-Objekte zusammen
//! (Sales Person und Benutzerkonto) und pflegt mit `PersonKind` eine
//! Cross-Entity-Invariante — kein Entity-Manager im Sinne der
//! CLAUDE.md-Konvention.
//!
//! **Dependencies: genau drei — `PersonDirectoryDao`, `PermissionService`,
//! `TransactionDao`.** Kein weiterer Domain-Service. Waere einer dabei,
//! entstuende ueber `PermissionService` ein DI-Zyklus...
//!
//! **Gate: das Admin-Privileg, als erste Anweisung (D-62-08, PADM-08...)**
```
For `person_detail.rs`, the Deps list will be larger (SalesPersonDao/Service, PermissionDao/Service, UserInvitationDao, PersonDao, TransactionDao — per RESEARCH's Architectural Responsibility Map) — copy the *shape* of this justification comment, not the exact dependency list.

**`gen_service_impl!` DI macro pattern** (lines 39-45):
```rust
gen_service_impl! {
    struct PersonDirectoryServiceImpl: PersonDirectoryService = PersonDirectoryServiceDeps {
        PersonDirectoryDao: PersonDirectoryDao<Transaction = Self::Transaction> = person_directory_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}
```

**Gate-first + bulk-read pattern** (lines 112-152, the direct template for `get_detail(person_id)`):
```rust
async fn list(
    &self,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[PersonEntry]>, ServiceError> {
    // Gate as the FIRST instruction, before any DB access.
    self.permission_service.check_permission("admin", context).await?;

    let tx = self.transaction_dao.use_transaction(tx).await?;
    let rows = self.person_directory_dao.list_directory_entries(tx.clone()).await?;
    // ONE bulk query for roles, not one call per row.
    let role_rows = self.person_directory_dao.all_user_roles(tx.clone()).await?;
    self.transaction_dao.commit(tx).await?;
    // ... assemble in Rust, sort in Rust (never SQLite collation) ...
}
```
For the detail read: narrow to one query per section (sales-person row, roles-for-user via existing `PermissionDao::roles_for_user`, invitations via `UserInvitationDao`, shiftplan assignment) — bounded per-person reads, not N+1.

**Free-function assembly pattern (testable without DI/DB)** (lines 47-60):
```rust
/// Baut eine Verzeichniszeile aus der DAO-Rohzeile plus der Rollen-Map.
///
/// Bewusst eine **freie** Funktion und keine Methode: so bleiben Klassifikation
/// und Anzeigenamen-Ableitung ohne Datenbank und ohne DI-Gerüst testbar.
fn entry_from_row(
    row: &PersonDirectoryRow,
    roles_by_user: &HashMap<Arc<str>, Vec<Arc<str>>>,
) -> PersonEntry {
    let kind = match (row.sales_person_name.as_ref(), row.username.as_ref()) {
        (Some(_), Some(_)) => PersonKind::Linked,
        (Some(_), None) => PersonKind::SalesPersonOnly,
        (None, Some(_)) => PersonKind::UserOnly,
        ...
```
Use the same shape for a pure `orphan check`/kind-derivation helper feeding `remove_facet`.

---

### `service_impl/src/sales_person.rs::create` — combined-create transaction pattern (Pattern 2)

**Analog:** `service_impl/src/sales_person.rs:166-194` (already reads shared-transaction convention correctly)

```rust
async fn create(
    &self,
    sales_person: &SalesPerson,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<SalesPerson, service::ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    self.permission_service
        .check_permission(HR_PRIVILEGE, context)
        .await?;
    if sales_person.id != Uuid::nil() {
        return Err(ServiceError::IdSetOnCreate);
    }
    if sales_person.version != Uuid::nil() {
        return Err(ServiceError::VersionSetOnCreate);
    }
    let sales_person = SalesPerson {
        id: self.uuid_service.new_uuid("sales-person-id"),
        version: self.uuid_service.new_uuid("sales-person-version"),
        ..sales_person.clone()
    };
    self.sales_person_dao
        .create(&SalesPersonEntity::from(&sales_person), SALES_PERSON_SERVICE_PROCESS, tx.clone())
        .await?;
    // caller decides commit — participates in outer tx if tx.clone() was passed in
}
```
**Copy this shape** for `create_combined`: call `sales_person_service.create(&sp, ctx, Some(tx.clone()))` so it joins the outer transaction instead of committing early — this only works once `PermissionService::create_user`/`PermissionDao::create_user` also accept the shared `Transaction` type (see the outlier excerpt below).

**The incompatible outlier — `PermissionDao::create_user`'s own local transaction** (`dao_impl_sqlite/src/lib.rs:95-135`, DO NOT copy this shape — it is the bug class RESEARCH Pattern 2 documents):
```rust
async fn create_user(&self, user: &dao::UserEntity, process: &str) -> Result<(), DaoError> {
    let name = user.name.as_ref();
    // D-61-01: ... Die lokale Transaktion ist zwingend ...
    let mut tx = self.pool.begin().await.map_db_error()?;
    let person_id = uuid::Uuid::new_v4().as_bytes().to_vec();
    query!("INSERT INTO person (id, created, update_process, update_version) VALUES (?, ..., ?, randomblob(16))",
        person_id, process).execute(tx.as_mut()).await.map_db_error()?;
    query!(r"INSERT INTO user (name, person_id, update_process) VALUES (?, ?, ?)",
        name, person_id, process).execute(tx.as_mut()).await...
    // ... tx.commit() happens inside this method, invisible to any caller's outer tx
}
```
And the corresponding service call site (`service_impl/src/permission.rs:121-134`):
```rust
async fn create_user(
    &self,
    user: &str,
    context: Authentication<Self::Context>,
) -> Result<(), service::ServiceError> {
    self.check_permission("admin", context).await?;
    self.permission_dao
        .create_user(&dao::UserEntity { name: user.into() }, PERMISSION_SERVICE_PROCESS)
        .await?;
    Ok(())
}
```
Note: no `tx: Option<Self::Transaction>` parameter exists in this trait method at all — this is the gap Pattern 2 Option 1 (RESEARCH.md) recommends closing by threading the shared `Transaction` type through both the DAO and service signatures, matching `SalesPersonService::create`'s convention.

---

### `service_impl/src/sales_person.rs::set_user` — 409-conflict translation (link-to-existing write path, reuse unchanged)

**Analog:** `service_impl/src/sales_person.rs:316-379` (full method, already production code — do not rebuild, only call)

```rust
async fn set_user(
    &self,
    sales_person_id: Uuid,
    user_id: Option<Arc<str>>,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<(), ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    self.permission_service.check_permission(HR_PRIVILEGE, context).await?;
    match self.sales_person_dao.discard_assigned_user(sales_person_id, tx.clone()).await {
        Ok(()) => {}
        Err(dao::DaoError::EntityAlreadyExists) => {
            // Kein Commit: die Transaktion wird verworfen (P-61-B).
            return Err(ServiceError::EntityAlreadyExists(sales_person_id));
        }
        Err(other) => return Err(other.into()),
    }
    if let Some(user) = user_id {
        match self.sales_person_dao.assign_to_user(sales_person_id, user.as_ref(), SALES_PERSON_SERVICE_PROCESS, tx.clone()).await {
            Ok(()) => {}
            Err(dao::DaoError::EntityAlreadyExists) => {
                return Err(ServiceError::EntityAlreadyExists(sales_person_id));
            }
            Err(other) => return Err(other.into()),
        }
    }
    self.transaction_dao.commit(tx).await?;
    Ok(())
}
```
Already exposed at `POST /sales-person/{id}/user` / `DELETE /sales-person/{id}/user` (`rest/src/sales_person.rs:268-300`). The candidate-picker UI and "Verknüpfung lösen" action call these **unchanged** endpoints — no new backend write path needed here.

---

### `rest/src/person_detail.rs` (controller, request-response)

**Analog:** `rest/src/person_directory.rs` (full file, 60-ish lines, read in full)

**Module-header + route-contract comment pattern** (lines 1-18):
```rust
//! Phase 62 Plan 05 (PADM-01 / PADM-08): REST-Oberflaeche des
//! Personen-Verzeichnisses — `GET /person-directory`.
//!
//! **Das Autorisierungs-Gate liegt im Service, nicht hier (D-62-08, T-62-13).**
//! `PersonDirectoryServiceImpl::list` verlangt das Admin-Privileg als erste
//! Anweisung, vor jedem Datenbankzugriff. Der Handler prueft bewusst **nicht**
//! zusaetzlich...
//!
//! **Der Praefix `/person-directory` ist ein Vertrag ueber vier Stellen.** Er
//! steht im `.nest(...)`-Aufruf und im `ApiDoc`-Nest (beides `rest/src/lib.rs`),
//! in der Aufruf-URL des Frontends (`shifty-dioxus/src/api.rs`) und im
//! `[[web.proxy]]`-Eintrag (`shifty-dioxus/Dioxus.toml`). Alle vier muessen
//! zeichengleich bleiben...
```
**Route + handler + utoipa pattern** (lines 30-62):
```rust
pub fn generate_route<RestState: RestStateDef>() -> Router<RestState> {
    Router::new().route("/", get(get_person_directory::<RestState>))
}

#[instrument(skip(rest_state))]
#[utoipa::path(
    get,
    path = "/",
    tags = ["PersonDirectory"],
    responses(
        (status = 200, description = "...", body = [PersonEntryTO]),
        (status = 403, description = "Not an admin"),
        (status = 500, description = "Internal server error"),
    ),
)]
pub async fn get_person_directory<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let entries: Arc<[PersonEntryTO]> = rest_state
                .person_directory_service()
                .list(context.into(), None)
                .await?
                .iter()
                .map(PersonEntryTO::from)
                .collect();
            // ... build 200 Response ...
        }).await,
    )
}
```
Copy for `GET /person/{id}`, `POST /person`, `POST /person/{id}/facet`, `DELETE /person/{id}/facet/{kind}`, `GET /person/candidates/sales-persons`, `GET /person/candidates/users` — one handler per operation, `error_handler` wraps every one, utoipa annotation with explicit status list including the new 409 path for combined-create conflicts.

**409-conflict REST annotation to copy** (`rest/src/sales_person.rs:277-283`):
```rust
#[utoipa::path(
    post,
    path = "/{id}/user",
    ...
    responses(
        (status = 204, description = "Set sales person user"),
        (status = 404, description = "Sales person not found"),
        (status = 409, description = "Sales person or user is already linked to a different counterpart"),
        (status = 500, description = "Internal server error"),
    ),
)]
```

**Central 409/error-mapping — never handler-local** (`rest/src/lib.rs:150-195`, the `error_handler` every new handler must route through):
```rust
fn error_handler(result: Result<Response, RestError>) -> Response {
    if let Err(ref err) = result {
        error!(error = ?err, "REST error mapping");
    }
    match result {
        Ok(response) => response,
        Err(RestError::ServiceError(service::ServiceError::Forbidden)) => {
            Response::builder().status(403).body(Body::empty()).unwrap()
        }
        Err(RestError::ServiceError(service::ServiceError::EntityAlreadyExists(id))) => {
            Response::builder().status(409).body(Body::new(id.to_string())).unwrap()
        }
        Err(RestError::ServiceError(service::ServiceError::EntityNotFound(id))) => {
            Response::builder().status(404).body(Body::new(id.to_string())).unwrap()
        }
        // ... EntityNotFoundGeneric, DatabaseQueryError, etc.
    }
}
```
This is the D-61-04 409 link path — every new `ServiceError::EntityAlreadyExists` returned by the new business-logic service (candidate conflict, combined-create partial failure) is picked up here automatically; do not add a second, handler-local status-mapping branch.

---

### `dao/src/person.rs` + `dao_impl_sqlite/src/person.rs` — adding `PersonDao::delete` (D-63-01 orphan cleanup)

**Analog:** existing `PersonDao::create`/`find_by_id` in the same files (`dao/src/person.rs:33-52`, `dao_impl_sqlite/src/person.rs` `PersonDb`/`TryFrom` block)

```rust
// dao/src/person.rs — trait method shape to extend
async fn find_by_id(&self, id: Uuid, tx: Self::Transaction) -> Result<Option<PersonEntity>, DaoError>;
async fn create(&self, entity: &PersonEntity, process: &str, tx: Self::Transaction) -> Result<(), DaoError>;
// NEW: async fn delete(&self, id: Uuid, tx: Self::Transaction) -> Result<(), DaoError>;
```
Module-header note to preserve/extend (the trait was deliberately capped at 4 methods — the new 5th method's doc comment should say why, per RESEARCH's explicit guidance):
```rust
//! Das Trait bleibt bewusst auf genau vier Methoden beschraenkt. Vorratshaltung
//! fuer Phase 62/63 ist ausdruecklich unerwuenscht (Research Pitfall 5).
```
`PersonEntity` has no `deleted` column (D-60-04 — identity + audit only), so `delete` here is a real `DELETE FROM person WHERE id = ?`, not a soft-delete stamp — mirror the row-layout/`TryFrom` conventions already in `dao_impl_sqlite/src/person.rs` (`PersonDb` struct + manual UUID/time parsing), not the `sales_person` soft-delete pattern.

---

### `shifty-dioxus/src/page/person_detail.rs` (NEW page component, request-response)

**Analog:** `shifty-dioxus/src/page/user_details.rs` (333 lines) + `shifty-dioxus/src/page/sales_person_details.rs` (339 lines) — both read in full; literal section-by-section port target per UI-SPEC.

**Admin guard pattern to copy verbatim** (`shifty-dioxus/src/page/user_management.rs:60-92`):
```rust
pub(crate) fn is_admin(auth_info: Option<&AuthInfo>) -> bool {
    auth_info.map(|a| a.has_privilege("admin")).unwrap_or(false)
}

#[component]
pub fn NotAuthorizedView() -> Element {
    rsx! {
        TopBar {}
        div { class: "p-md text-ink-muted", "Not authorized." }
    }
}
// in the page component, BEFORE any hook or data access:
if !is_admin(auth.auth_info.as_ref()) {
    return rsx! { NotAuthorizedView {} };
}
```

**Section-header markup to port (retokenized — see UI-SPEC's Legacy-Class Retokenization table, do NOT copy the legacy classes below verbatim):**
```rust
// sales_person_details.rs:95-98 (legacy — retokenize text-gray-800 -> text-ink, border-gray-200 -> border-border)
h2 { class: "text-h2 text-gray-800 mb-4 pb-2 border-b border-gray-200",
    "{i18n.t(Key::BasicInformation)}"
}
```
```rust
// user_details.rs:76-90 (legacy — same retokenization, plus the role-count badge pill:
// bg-gray-100 text-gray-500 -> bg-surface-alt text-ink-muted)
div { class: "flex items-center justify-between mb-4",
    h2 { class: "text-h2 text-gray-800", "{i18n.t(Key::RoleAssignments)}" }
    span { class: "text-small font-normal text-gray-500 bg-gray-100 px-2 py-1 rounded",
        { /* "{assigned}/{total}" via Key::RolesCount */ }
    }
}
```

**Pure-helper contract to implement (named exactly per UI-SPEC, unit-tested, no Dioxus runtime):**
```rust
pub(crate) struct SectionVisibility { pub stammdaten: bool, pub shiftplan_zuordnung: bool, pub roles: bool, pub invitations: bool }
pub(crate) fn visible_sections(kind: PersonKindTO) -> SectionVisibility { /* ... */ }
pub(crate) enum FacetKind { SalesPerson, User }
pub(crate) fn is_last_facet(kind: PersonKindTO, removing: FacetKind) -> bool { /* ... */ }
pub(crate) struct DeleteConfirmationCopy { pub title: &'static str, pub body_key: Key, pub hint_key: Option<Key>, pub last_facet_key: Option<Key> }
pub(crate) fn delete_confirmation_copy(kind: PersonKindTO, facet: FacetKind) -> DeleteConfirmationCopy { /* ... */ }
```
Analog for "pure helper + unit test" shape: `matches_search` in `user_management.rs:32-35` — a free function, trivially testable, called from `rsx!` without any store/DOM dependency.

---

### SSR rendertest harness (reuse for section-visibility tests)

**Analog:** `shifty-dioxus/src/page/user_management.rs:474+` (existing, 18 green tests in Phase 62)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rest_types::PersonKindTO;
    use std::sync::Arc;

    fn render_directory(entries: Vec<PersonEntryTO>, search_initial: &str) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps { entries: Rc<[PersonEntryTO]>, initial: String }
        // ... VirtualDom::new_with_props(...).rebuild_in_place(); dioxus_ssr::render(&vdom)
    }
}
```
New detail-view tests should follow: `render_detail(entry: PersonEntryTO) -> String`, then `assert!(html.contains(...))` / `assert!(!html.contains(...))` per `PersonKindTO` variant (`Linked`, `SalesPersonOnly`, `UserOnly`) — mirrors the 18-test suite Phase 62 already built on this exact harness.

---

### `shifty-dioxus/src/router.rs` (route wiring)

**Analog:** the very entries being replaced, in the same file (lines 45-50):
```rust
#[route("/user_management/")]
UserManagementPage {},
#[route("/user_details/:user_id/")]
UserDetails { user_id: String },
#[route("/sales_person_details/:sales_person_id/")]
SalesPersonDetails { sales_person_id: String },
```
Replace the two `UserDetails`/`SalesPersonDetails` routes with:
```rust
#[route("/person/:person_id/")]
PersonDetail { person_id: String },
```
and update the `pub use crate::page::...` re-export block accordingly (`UserDetails`/`SalesPersonDetails` imports removed, `PersonDetail` added) — same file, lines 8-23.

---

### `shifty-dioxus/src/component/top_bar.rs` (breadcrumb wiring)

**Analog:** the existing breadcrumb branches for the two retired routes at lines 94-95 and 883-889 — read those two spots directly when implementing (not excerpted here to avoid a redundant read of a 1342-line file at plan time; grep `UserDetails\|SalesPersonDetails` in this file for the exact `match Route::...` arms to replace with a single `Route::PersonDetail { .. }` arm).

---

### `shifty-dioxus/src/service/user_management.rs` (coroutine action dispatch, event-driven)

**Analog:** existing `UserManagementAction` enum + match arms, same file:
```rust
pub enum UserManagementAction {
    LoadPersonDirectory,
    ...
    DeleteUser(ImStr),
    ...
}

pub async fn user_management_service(mut rx: UnboundedReceiver<UserManagementAction>) {
    while let Some(action) = rx.next().await {
        match action {
            UserManagementAction::LoadPersonDirectory => { /* ... */ }
            ...
            UserManagementAction::DeleteUser(user) => match delete_user(user).await { /* ... */ },
            ...
        }
    }
}
```
Extend with new variants for: `LoadPersonDetail(person_id)`, `CreateCombinedEntry{..}`, `AddFacet{..}`, `LinkCandidate{..}`, `UnlinkFacet{..}`, `DeleteSalesPerson(sales_person_id)` (D-63-06's net-new frontend call path — today `delete_sales_person` has zero frontend caller, verified by RESEARCH's grep), `DeleteLogin(user)` (renaming/reusing the existing `DeleteUser` shape). Follow the exact same `match action { Variant(args) => match api_call(args).await { Ok(_) => STORE.write()..., Err(e) => ERROR_STORE... } }` shape already used for `DeleteUser`.

---

### `shifty-dioxus/src/i18n/{de,en,cs}.rs` (i18n keys, config)

**Analog:** existing key-registration lines in the same three files:
```rust
i18n.add_text(Locale::De, Key::UserManagement, "Benutzerverwaltung");
i18n.add_text(Locale::De, Key::AddNewUser, "Neuen Benutzer hinzufügen");
```
Add every new `Key::` variant named in `63-UI-SPEC.md`'s Copywriting Contract (e.g. `CreatePersonEntry`, `CreatePersonEntryDisplayName`, `CreatePersonEntryWantsLogin`, `DeleteLoginConfirmBody`, `DeleteSalesPersonConfirmBody`, ...) to all three locale files in the same shape/order, using the exact DE/EN/CS strings the UI-SPEC already fixes verbatim (D-63-08's two sentences must not be reworded). A pre-existing key-set-parity test (grep for it near the `Key` enum / i18n test module) compares EN/CS key sets against the DE reference — run it after adding keys to all three files in the same commit.

---

### `shifty-dioxus/Dioxus.toml` (dev-proxy, config)

**Analog:** the immediately-preceding proxy block for the sibling aggregate endpoint (lines 116-118):
```toml
[[web.proxy]]
backend = "http://localhost:3000/person-directory"
```
Add a new, separate entry (NOT a sub-path of `/person-directory` — proxy matching is prefix-literal, not hierarchical):
```toml
[[web.proxy]]
backend = "http://localhost:3000/person"
```
**Non-negotiable verification per CONTEXT.md/RESEARCH.md:** this exact bug class (missing proxy entry → 404/405 in `dx serve`, invisible to any `cargo test`) has shipped three times in this repo (v1.6 `/toggle`, v1.8 `/vacation-entitlement-offset`, v1.10 `/special-days`). Verify with a `dx serve` smoke test against `/person/{id}`, not a unit test.

## Shared Patterns

### Admin gate as first instruction (Business-Logic tier)
**Source:** `service_impl/src/person_directory.rs:117-122`
**Apply to:** every new method on the Phase 63 business-logic service (`get_detail`, `create_combined`, `add_facet`, `remove_facet`, candidate-list reads)
```rust
async fn list(&self, context: Authentication<Self::Context>, tx: Option<Self::Transaction>)
    -> Result<Arc<[PersonEntry]>, ServiceError> {
    self.permission_service.check_permission("admin", context).await?;
    // ...
}
```

### 409-conflict translation (DaoError::EntityAlreadyExists → ServiceError::EntityAlreadyExists → HTTP 409)
**Source:** `service_impl/src/sales_person.rs:339-355` (service side), `rest/src/lib.rs:180-183` (REST mapping)
**Apply to:** any new write in the business-logic service that can race (combined-create partial conflict, candidate-picker link submitted after the target got linked elsewhere)
```rust
match self.sales_person_dao.assign_to_user(...).await {
    Ok(()) => {}
    Err(dao::DaoError::EntityAlreadyExists) => {
        return Err(ServiceError::EntityAlreadyExists(sales_person_id)); // no commit — tx discarded
    }
    Err(other) => return Err(other.into()),
}
```

### `gen_service_impl!` DI macro + `Option<Transaction>`/commit convention
**Source:** `service_impl/src/person_directory.rs:39-45`, `service_impl/src/sales_person.rs:166-172`
**Apply to:** `service_impl/src/person_detail.rs`'s new struct definition and every method body
```rust
gen_service_impl! {
    struct XServiceImpl: XService = XServiceDeps {
        SomeDao: SomeDao<Transaction = Self::Transaction> = some_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}
// every method:
async fn do_x(&self, ..., tx: Option<Self::Transaction>) -> Result<T, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // ...
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

### DI wiring order: Business-Logic strictly after Basic block
**Source:** `shifty_bin/src/main.rs:352-376` (Deps-struct declaration) + `:1371-1379` (construction)
**Apply to:** the new person-detail service's Deps struct and its `Arc::new(...)` construction call
```rust
pub struct PersonDirectoryServiceDependencies;
impl service_impl::person_directory::PersonDirectoryServiceDeps for PersonDirectoryServiceDependencies {
    type Context = Context;
    type Transaction = Transaction;
    type PersonDirectoryDao = PersonDirectoryDao;
    type PermissionService = PermissionService;
    type TransactionDao = TransactionDao;
}
type PersonDirectoryService = service_impl::person_directory::PersonDirectoryServiceImpl<PersonDirectoryServiceDependencies>;
// ...later, after the entire Basic block:
let person_directory_service = Arc::new(
    service_impl::person_directory::PersonDirectoryServiceImpl::<PersonDirectoryServiceDependencies> {
        person_directory_dao,
        permission_service: permission_service.clone(),
        transaction_dao: transaction_dao.clone(),
    },
);
```
Place the new person-detail service's construction in the same Business-Logic section (near `person_directory_service`/`voluntary_stats_service`), strictly after `person_service`/`sales_person_service`/`permission_service` are already built.

### REST route prefix as a four-point contract
**Source:** `rest/src/person_directory.rs:11-18`
**Apply to:** every new `/person/...` route — the same four places must all agree: `.nest(...)` + `ApiDoc` nest in `rest/src/lib.rs`, the frontend call URL in `shifty-dioxus/src/api.rs`/`loader.rs`, and the `[[web.proxy]]` entry in `Dioxus.toml`.

### Component-level frontend admin guard
**Source:** `shifty-dioxus/src/page/user_management.rs:60-92` (itself copied from `page/settings.rs:556-570`)
**Apply to:** `page/person_detail.rs` — same `is_admin`/`NotAuthorizedView`/early-return-before-any-hook shape.

### Fat-Backend / no-derivation discipline
**Source:** module-header prose in `service_impl/src/person_directory.rs` ("Fat Backend (PADM-06...)"), enforced by grep in `page/user_management.rs`'s own doc comment
**Apply to:** all new frontend code — `display_name`, `PersonKindTO`, and section visibility are consumed as-is from the backend response; no `.unwrap_or(...)`/fallback-derivation logic in Dioxus code, verified by grep as this repo's established proof form.

## No Analog Found

None — every file in scope has a close, recently-modified (Phase 60-62) analog in the same module family, since Phase 63 directly extends the Phase 61/62 person-model work.

## Metadata

**Analog search scope:** `service/src/`, `service_impl/src/`, `dao/src/`, `dao_impl_sqlite/src/`, `rest/src/`, `rest-types/src/`, `shifty_bin/src/main.rs`, `shifty-dioxus/src/page/`, `shifty-dioxus/src/service/`, `shifty-dioxus/src/router.rs`, `shifty-dioxus/src/component/top_bar.rs`, `shifty-dioxus/src/i18n/`, `shifty-dioxus/Dioxus.toml`
**Files scanned:** ~20 (full or targeted reads: `person_directory.rs` [service+service_impl+rest], `sales_person.rs` [service_impl+rest], `permission.rs` [service_impl], `dao_impl_sqlite/src/lib.rs` [create_user], `person.rs` [dao+dao_impl_sqlite], `rest/src/lib.rs` [error_handler], `shifty_bin/src/main.rs` [DI wiring], `user_details.rs`, `sales_person_details.rs`, `user_management.rs`, `router.rs`, `Dioxus.toml`, `i18n/de.rs`, `service/user_management.rs`)
**Pattern extraction date:** 2026-09-20
