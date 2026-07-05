# Feature: Text Templates & User Invitation (Communication)

> **In short:** Reusable text / HTML templates (Tera / MiniJinja) for
> reports, plus an invitation flow that onboards users via one-time link
> / token and makes the resulting session server-side revocable.

**Cluster ID:** F10
**Status:** in production
**First introduced:** Text Templates 2025-08 (v1.2), User Invitation
2025-10 (v1.2 Phase 6)
**Responsible crates:**
- `service::text_template`, `service_impl::text_template`, `dao::text_template`,
  `dao_impl_sqlite::text_template`
- `service::user_invitation`, `service_impl::user_invitation`,
  `dao::user_invitation`, `dao_impl_sqlite::user_invitation`
- `rest::text_template`, `rest::user_invitation`
- `rest-types::{TextTemplateTO, CreateTextTemplateRequestTO,
  UpdateTextTemplateRequestTO, TemplateEngineTO, GenerateInvitationRequest,
  InvitationResponse, InvitationStatus}`
- Frontend: `shifty-dioxus/src/page/text_template_management.rs`,
  `shifty-dioxus/src/service/text_template.rs`,
  `shifty-dioxus/src/page/user_details.rs` (invitation panel)

---

## 1. What is it? (Business perspective)

F10 bundles two areas that both cover "outward communication" — once
toward *report recipients* (text templates) and once toward *new users*
(user invitation).

### 1.1 Text Templates

Text templates are HR-managed reusable templates for reports. Currently
they are consumed by the Custom Billing Period Report
(`POST /billing-period/{billing_period_id}/custom-report/{template_id}`,
see `docs/template-examples/README.md`), but they are generic: each
template type is a free string (`template_type`), so further consumers
(e.g. shift plan reports, email templates) can be added later without
changing the schema. Currently hardcoded types in the frontend are
`billing-period` and `shiftplan-report`
(`text_template_management.rs:169-177`).

A template consists of:
- an optional descriptive **name** (`name`, since migration 2025-08-17),
- **`template_type`** — business category / consumer chain (filter in the
  frontend),
- **`template_text`** — the actual template body (HTML, text, …),
- **`template_engine`** — `Tera` (default) or `MiniJinja` (since
  migration 2026-03-12).

**Example workflow from a user's perspective (text templates):**

1. HR opens "Text Template Management".
2. HR clicks "Add New", enters a name, selects a type (`billing-period`),
   selects an engine (`Tera`), pastes the template body.
3. Save → the template is available via REST.
4. HR selects a template in "Billing Period Details" and renders the
   custom report.
5. A faulty body or missing variable → the error comes from the consumer
   (Billing Period Custom Report), not from the template service itself.

### 1.2 User Invitation

The invitation flow is the only way an admin can onboard a new user
outside OIDC / mock auth. The result of an invitation is a **one-time
link** with an embedded UUID token that the invitee opens in the
browser. On the backend, the user is created in `permission_dao` if
needed and a Session is issued; the Invitation is anchored to that
session and thereby preserves the association "who was invited when via
which link, and now sits in which Session?".

**Example workflow from a user's perspective (invitation):**

1. Admin opens "User Details" for a target user and clicks "Generate
   invitation" (default: 7 days validity).
2. Backend creates an Invitation record + token; the frontend displays
   the link to copy.
3. Admin sends the link to the invitee (chat, email, note, …).
4. The invitee opens `/auth/invitation/{token}` → backend validates the
   token, creates the user if needed, creates a Session, sets the cookie,
   redirects to `/`, marks the Invitation as `Redeemed` including
   `session_id`.
5. At any time, the admin can in the frontend:
   - *revoke* an **unconsumed** Invitation (delete) → link becomes
     invalid;
   - "revoke Session" on a **redeemed** Invitation → the invitee's
     active Session is invalidated; the Invitation then carries the
     status `SessionRevoked`.

## 2. Business Rules

### 2.1 Text Templates

- All write operations (`create`, `update`, `delete`) require
  `HR_PRIVILEGE`
  (`service_impl/src/text_template.rs:78,112,149`).
- Read operations (`get_all`, `get_by_id`, `get_by_template_type`) have
  **no** dedicated permission gate in the service
  (`service_impl/src/text_template.rs:26-69`). **[To verify]** whether
  that is intentional or whether access restriction is only expected at
  the REST auth-middleware layer — a consistent gate would be cleaner for
  an HR-only use case.
- `id` is regenerated server-side on `create` and is immutable; an `id`
  value sent by the client is overwritten
  (`service_impl/src/text_template.rs:90`).
- `version` is regenerated on every `create`/`update`
  (`service_impl/src/text_template.rs:91,131`).
- `created_at`/`created_by` are set only on `create` and carried over
  from the existing version on `update`
  (`rest/src/text_template.rs:233-238`).
- Before `update` and `delete`, the service explicitly checks existence
  and otherwise returns `EntityNotFoundGeneric`
  (`service_impl/src/text_template.rs:124-128,161-165`).
- `template_engine` falls back to `tera` via DB DEFAULT if the field is
  missing (migration 2026-03-12). Allowed values at the DAO boundary:
  `tera`, `minijinja` — otherwise `DaoError::EnumValueNotFound`
  (`dao/src/text_template.rs:16-28`).
- Soft-delete: DAO queries filter `deleted IS NULL` (convention per
  project CLAUDE.md; same in the DAO impl). **[To verify]** whether the
  text-template DAO applies the soft-delete convention consistently or
  actually deletes — the trait signature `delete(id, process, tx)` does
  not say.

### 2.2 User Invitation

- All admin actions (`generate_invitation`, `list_invitations_for_user`,
  `find_invitation_by_session`, `revoke_invitation`,
  `revoke_session_for_invitation`) are gated by the permission string
  `"admin"`
  (`service_impl/src/user_invitation.rs:53,150,180,214,254`).
- **`validate_and_consume_token` and `mark_token_redeemed` are
  intentionally ungated** because they run in the unauthenticated
  bootstrap path (`rest/src/user_invitation.rs:36-128`) — otherwise
  nobody could ever redeem their invitation link.
- Default expiration: `expiration_hours = 7*24` = 7 days
  (`rest/src/user_invitation.rs:150`).
- Token is single-use: `validate_and_consume_token` throws
  `EntityNotFoundGeneric("Invitation token has already been used")` as
  soon as `session_id.is_some()`
  (`service_impl/src/user_invitation.rs:106-110`).
- Expired token: identical error type with message
  `"Invitation token has expired"`
  (`service_impl/src/user_invitation.rs:112-117`).
- User auto-create: if `permission_dao.find_user(username)` returns
  `None` on redeem, the user is created via `create_user(...,
  USER_INVITATION_SERVICE_PROCESS)`
  (`service_impl/src/user_invitation.rs:119-133`).
- Session anchor: the REST layer creates the Session
  (`SessionService::new_session_for_user`) *after* successful token
  validation and then calls `mark_token_redeemed(token, session_id)`
  (`rest/src/user_invitation.rs:53-70`). Thus `session_id` sits ONLY on
  the redeemed Invitation, not on all open ones.
- Status derivation (`InvitationStatus`, `service_impl/src/user_invitation.rs
  :28-38`): order of checks: `session_revoked_at` >
  `redeemed_at` > `expiration_date < now` > otherwise `Valid`.
  **Important:** a link revoked by admin (`revoke_invitation` = hard
  `delete_by_id`) then no longer appears at all; only Session revocation
  is visible as a dedicated status.
- Cookie semantics in the OIDC path: `path="/"`, `expires=now+365d`,
  `http_only`, `SameSite=Strict`, `secure=true`
  (`rest/src/user_invitation.rs:73-81`).
- Cleanup: `cleanup_expired_invitations` hard-deletes **all** expired
  rows (`dao_impl_sqlite/src/user_invitation.rs:211-226`), including
  already-redeemed ones — i.e. anyone looking for the audit info "who was
  invited when" after expiry will no longer find it after cleanup.
  **[To verify]** whether that is intentional or whether "only unused
  expired" would be the goal.

## 3. Data Model

### Tables

| Table | Purpose | Important columns |
| --- | --- | --- |
| `text_template` | Template repository for reports | `id BLOB PK`, `name TEXT?`, `template_type TEXT`, `template_text TEXT`, `template_engine TEXT NOT NULL DEFAULT 'tera'`, `created_at`, `created_by`, `deleted`, `deleted_by`, `update_version BLOB`, `update_process TEXT` |
| `user_invitation` | Invitation records incl. session anchor | `id TEXT PK`, `username TEXT FK→user.name ON DELETE CASCADE`, `token TEXT UNIQUE`, `expiration_date TEXT`, `created_date TEXT DEFAULT datetime('now')`, `update_process TEXT NOT NULL`, `redeemed_at TEXT?`, `session_id TEXT? FK→session.id ON DELETE SET NULL`, `session_revoked_at TEXT?` |

Indexes:
- `idx_text_template_type` on `template_type`
- `idx_text_template_deleted` on `deleted`
- `idx_text_template_name` on `name`
- `idx_user_invitation_token` on `token`
- `idx_user_invitation_expiration` on `expiration_date`
- `idx_user_invitation_session` on `session_id`
- `idx_user_invitation_redeemed` on `redeemed_at`
- `idx_user_invitation_session_revoked` on `session_revoked_at`

### Migrations

Chronological:

- `20250816133730_add-message-template-table.sql` — base table
  `text_template` incl. `template_type`, `template_text`, audit columns,
  and indexes. Despite the file name "message-template" the table is
  called `text_template` in the DB.
- `20250817000000_add-name-to-text-template.sql` — `name TEXT` (nullable)
  + `idx_text_template_name`.
- `20251016154210_add-user-invitation-table.sql` — base table
  `user_invitation` with `token UNIQUE`, FK on `user(name)` and
  expiration index.
- `20251017044013_add-session-tracking-to-user-invitation.sql` —
  `redeemed_at`, `session_id` (FK on `session(id) ON DELETE SET NULL`),
  matching indexes; upgrades the semantics from "single-use" to
  "single-use + we know which Session came out of it".
- `20251020000000_add-session-revoked-at-to-user-invitation.sql` —
  `session_revoked_at`, so that "admin has revoked Session" remains
  persistently distinguishable from mere "expired".
- `20260312000000_add-template-engine-to-text-template.sql` —
  `template_engine TEXT NOT NULL DEFAULT 'tera'`; old rows are implicitly
  pinned to `tera`, new rows may choose `minijinja`.

### Relationships

```
user (name) ──┐
              │  ON DELETE CASCADE
              ▼
        user_invitation ──── session_id ──► session (id)
                                            ON DELETE SET NULL
```

Deleting a user drops the user's invitations. Hard-deleting a Session
sets `session_id` to `NULL` — the Invitation record is retained for
History and then reports status `Redeemed` (because `redeemed_at` stays
set) even though the Session no longer exists. **[To verify]** whether
this state is intentionally shown as "Redeemed" in the UI (rather than
"SessionRevoked" or a dedicated "Orphaned" status).

## 4. Service API

### 4.1 `TextTemplateService`

File: `service/src/text_template.rs:83-129`.

```rust
#[async_trait]
pub trait TextTemplateService {
    type Context;
    type Transaction: dao::Transaction;

    async fn get_all(&self, ctx, tx) -> Result<Arc<[TextTemplate]>, ServiceError>;
    async fn get_by_id(&self, id: Uuid, ctx, tx) -> Result<TextTemplate, ServiceError>;
    async fn get_by_template_type(&self, template_type: &str, ctx, tx) -> Result<Arc<[TextTemplate]>, ServiceError>;
    async fn create(&self, item: &TextTemplate, ctx, tx) -> Result<TextTemplate, ServiceError>;
    async fn update(&self, item: &TextTemplate, ctx, tx) -> Result<TextTemplate, ServiceError>;
    async fn delete(&self, id: Uuid, ctx, tx) -> Result<(), ServiceError>;
}
```

**Auth gates:**
- `create`, `update`, `delete` → `HR_PRIVILEGE`
  (`service_impl/src/text_template.rs:78,112,149`).
- `get_all`, `get_by_id`, `get_by_template_type` → **no** gate in the
  service (see rule footnote in 2.1).

**TX behavior:**
- Every method opens the TX via
  `transaction_dao.use_transaction(tx).await?` and commits at the end
  itself. No composite over multiple services — no rollback fan-out.

**Dependencies (Basic tier per `CLAUDE.md` "Service Tier Conventions"):**
- `TextTemplateDao`
- `PermissionService`
- `TransactionDao`

### 4.2 `UserInvitationService`

File: `service/src/user_invitation.rs:36-93`.

```rust
#[async_trait]
pub trait UserInvitationService {
    type Transaction;
    type Context;

    async fn generate_invitation(&self, username: &str, expiration_hours: i64, tx, auth) -> Result<UserInvitation, ServiceError>;
    async fn validate_and_consume_token(&self, token: &Uuid, tx) -> Result<Arc<str>, ServiceError>;
    async fn mark_token_redeemed(&self, token: &Uuid, session_id: &str, tx) -> Result<(), ServiceError>;
    async fn find_invitation_by_session(&self, session_id: &str, tx, auth) -> Result<Option<UserInvitation>, ServiceError>;
    async fn list_invitations_for_user(&self, username: &str, tx, auth) -> Result<Vec<UserInvitation>, ServiceError>;
    async fn revoke_invitation(&self, id: &Uuid, tx, auth) -> Result<(), ServiceError>;
    async fn cleanup_expired_invitations(&self, tx) -> Result<u64, ServiceError>;
    async fn revoke_session_for_invitation(&self, invitation_id: &Uuid, tx, auth) -> Result<(), ServiceError>;
}
```

**Auth gates:** see 2.2. `validate_and_consume_token` and
`mark_token_redeemed` are defined without an `auth` parameter — they are
only called from the REST handler `authenticate_with_invitation`, which
itself hangs publicly (pre-auth).

**TX behavior:**
- All writing methods open a TX and commit.
- `list_invitations_for_user` and `find_invitation_by_session` open the
  TX (discard binding `let _tx = …`), but do not commit — this is an
  intentional read-only path, but it theoretically leaves an open TX
  until drop. **[To verify]** whether the `TransactionDao` auto-rolls-back
  in the drop path (convention in the rest of the codebase: yes).

**Dependencies:** `UserInvitationDao`, `PermissionDao` (user auto-create),
`PermissionService`, `SessionService` (Session invalidation),
`UuidService`, `TransactionDao` — thus **Business-Logic tier**
(orchestrates `SessionService` + `PermissionDao` as a cross-entity op).

## 5. REST Endpoints

### 5.1 Text Templates — mounted under `/text-templates` (`rest/src/lib.rs:673`)

| Method | Path | Description | DTO In | DTO Out | Important errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/text-templates` | List all templates | — | `Vec<TextTemplateTO>` | 401, 500 |
| `GET` | `/text-templates/{id}` | Get one template | — | `TextTemplateTO` | 401, 404, 500 |
| `GET` | `/text-templates/by-type/{template_type}` | Filter by type | — | `Vec<TextTemplateTO>` | 401, 500 |
| `POST` | `/text-templates` | Create | `CreateTextTemplateRequestTO` | `TextTemplateTO` (201) | 400, 401, 403 (HR), 500 |
| `PUT` | `/text-templates/{id}` | Update | `UpdateTextTemplateRequestTO` | `TextTemplateTO` | 400, 401, 403 (HR), 404, 500 |
| `DELETE` | `/text-templates/{id}` | Delete | — | 204 | 401, 403 (HR), 404, 500 |

DTOs see `rest-types::{TextTemplateTO, CreateTextTemplateRequestTO,
UpdateTextTemplateRequestTO, TemplateEngineTO}`
(`rest-types/src/lib.rs:1526-1601`).

### 5.2 User Invitation — mounted under `/user-invitation` (`rest/src/lib.rs:676`) plus public bootstrap route

| Method | Path | Description | DTO In | DTO Out | Important errors |
| --- | --- | --- | --- | --- | --- |
| `POST` | `/user-invitation/invitation` | Create invitation | `GenerateInvitationRequest` | `InvitationResponse` | 400, 403 (admin), 500 |
| `GET` | `/user-invitation/invitation/user/{username}` | List invitations of a user | — | `Vec<InvitationResponse>` | 403, 404, 500 |
| `DELETE` | `/user-invitation/invitation/{id}` | Revoke invitation (delete row) | — | 204 | 403, 404, 500 |
| `POST` | `/user-invitation/invitation/{id}/revoke-session` | Invalidate Session for invitation + set `session_revoked_at` | — | 204 | 403, 404 (no session associated), 500 |
| `GET` | `/auth/invitation/{token}` | **Public, pre-auth** — redeem token, create user if needed, set Session/cookie, redirect `/` | — | 302 Redirect | 400 "Invalid or expired invitation token", 500 |

The last route is mounted **outside** the auth middleware stack
(`rest/src/lib.rs:752-756`) and exists in two feature-gate variants:
- `feature = "oidc"`: creates a real Session via `SessionService::
  new_session_for_user`, sets the `app_session` cookie
  (`rest/src/user_invitation.rs:36-96`).
- `feature = "mock_auth"` (without `oidc`): mock Session ID
  `mock-session-<uuid>` is used only for the redeem marker; auth bypass
  runs globally (`rest/src/user_invitation.rs:99-128`).

DTOs see `rest-types::{GenerateInvitationRequest, InvitationResponse,
InvitationStatus}` (`rest-types/src/lib.rs:2286-2322`); the serde
representation of `InvitationStatus` is lowercase, `SessionRevoked` is
serialized as `"sessionrevoked"`
(`rest-types/src/lib.rs:2296-2297`).

## 6. Frontend Integration

### 6.1 Text Templates

- **Pages:** `shifty-dioxus/src/page/text_template_management.rs`
  (HR management view: list, create, edit, delete).
- **Services:** `shifty-dioxus/src/service/text_template.rs`
  (`TEXT_TEMPLATE_STORE` `GlobalSignal`, coroutine actions
  `LoadTemplates`, `LoadTemplatesByType`, `SaveTemplate`,
  `UpdateTemplate`, `DeleteTemplate`).
- **State:** `shifty-dioxus/src/state/text_template.rs` — `TextTemplate`
  DTO + `TemplateEngine` enum (frontend mirror).
- **Loader:** `shifty-dioxus/src/loader.rs` (`load_text_templates`,
  `load_text_templates_by_type`, `save_text_template`,
  `update_text_template`).
- **i18n keys:** `TextTemplateManagement`, `TemplateType`, `TemplateText`,
  `TemplateName`, `AddNew`, `AddNewTemplate`, `EditTemplate`, `Save`,
  `Cancel`, `Edit`, `Delete`, `Actions`, `TemplateEngine`,
  `TemplateEngineTera`, `TemplateEngineMiniJinja`.
- **Hardcoded template types in the frontend dropdown:** `billing-period`,
  `shiftplan-report` (`page/text_template_management.rs:169-177`) — new
  types would require an extension there.
- **Proxy:** `[[web.proxy]] backend = "http://localhost:3000/text-templates"`
  in `shifty-dioxus/Dioxus.toml`.

### 6.2 User Invitation

- **Pages:** `shifty-dioxus/src/page/user_details.rs` — panel with the
  list of invitations per user, "Copy link" button, revoke- and
  revoke-Session buttons depending on status.
- **Services:** `UserManagementAction::RevokeInvitation`,
  `UserManagementAction::RevokeInvitationSession` (see
  `page/user_details.rs:258-274`).
- **DTO reuse:** frontend uses `rest-types::InvitationResponse` and
  `InvitationStatus` directly (v1.2 Phase 6 migration in `rest-types`,
  `rest-types/src/lib.rs:2282-2334`).
- **Proxy:** `[[web.proxy]] backend =
  "http://localhost:3000/user-invitation"` in `Dioxus.toml`.
- The redeem path `/auth/invitation/{token}` does **not** go through the
  frontend router — it is handled directly by the backend and then
  redirects to `/`.

## 7. Edge cases

For the central edge case reference see
[`../domain/edge-cases.md`](../domain/edge-cases.md), section
[6. Authentication / Authorization](../domain/edge-cases.md#6-authentifizierung--autorisierung).

- **Double-redeemed invitation link:** `validate_and_consume_token`
  throws `EntityNotFoundGeneric("Invitation token has already been used")`
  as soon as `session_id.is_some()`; at the REST layer this becomes
  HTTP 400 "Invalid or expired invitation token"
  (`rest/src/user_invitation.rs:90-95`). The underlying Session remains
  valid until separately invalidated via `revoke-session`.
- **Expired link:** identically packaged as HTTP 400. After expiry the
  same token cannot be reactivated; the admin must generate a new one.
- **Race "redeem right at expiry":** `validate_and_consume_token`
  compares `expiration_date < now` **before** the session commit; since
  Session creation runs in the REST handler and *not* in the same
  service TX (`rest/src/user_invitation.rs:46-70`), there is a small
  time window in which a just-still-valid token could appear "expired"
  after Session creation — the Session is nonetheless valid; only the
  `mark_token_redeemed` call does not fail because of it (it does no
  time check, `dao_impl_sqlite/src/user_invitation.rs:173-192`). In
  practice: the user is logged in, but the Invitation shows `Redeemed`.
  **[To verify]** whether this is documented.
- **Session revoke without associated Session:**
  `revoke_session_for_invitation` returns
  `EntityNotFoundGeneric("No session associated with this
  invitation")` (`service_impl/src/user_invitation.rs:275-279`) → REST
  404. The UI only shows the button on `Redeemed`
  (`page/user_details.rs:267-274`).
- **Revoking a redeemed invitation via `DELETE
  /user-invitation/invitation/{id}`:** deletes the row **including**
  redeem history (`delete_by_id`); the resulting Session stays active,
  it just loses its Invitation anchor. **[To verify]** whether this is
  intentional — a soft-delete or a forced Session revoke beforehand
  would be more consistent.
- **Template with missing variable:** the error arises only at render
  time in the consumer (Billing Period Custom Report). The
  `TextTemplateService` itself does not validate template syntax —
  broken templates can be created and stored.
- **Template type freely choosable:** REST accepts any string; the
  frontend filters only on the two known types. A template with an
  unknown type is reachable via the `GET /text-templates/by-type/{type}`
  filter but not selectable from the standard frontend dropdown.
- **Cutover legacy → MiniJinja:** old templates without a
  `template_engine` column are set to `tera` via DB DEFAULT (migration
  2026-03-12), which matches historical behavior — no backward
  compatibility break for existing custom reports.
- **Missing user on redeem:** auto-create creates an empty user
  (`UserEntity { name }`) without roles; all permissions must then be
  set by the admin.

## 8. Tests

- **Unit tests:** neither for `text_template` nor for `user_invitation`
  does a test file currently exist under `service_impl/src/test/`
  (directory state see `mod.rs` listing). Both services are covered
  only manually and via frontend integration. **[To verify]** whether
  this is an intentional deprioritization — auth-relevant code such as
  `validate_and_consume_token` would at least benefit from property-style
  tests (race, expiry, double-redeem).
- **REST compile-time coverage:** `rest/src/text_template.rs` and
  `rest/src/user_invitation.rs` are included in the OpenAPI merge via
  `TextTemplateApiDoc` and `UserInvitationApiDoc` respectively
  (`rest/src/lib.rs:67,71,586`), i.e. schema drift between DTO and
  handler is at least caught by the `utoipa` macro.
- **Frontend test:** `text_template_management.rs:284-311` has a guard
  test against legacy Tailwind classes in the source code (not a
  behavior test).
- **Known gaps:**
  - No roundtrip test for token redemption incl. `mark_token_redeemed`.
  - No test that runs `cleanup_expired_invitations` against
    already-redeemed rows (data-loss risk, see 2.2).
  - No test for the interaction `session ON DELETE SET NULL` → status
    derivation.
  - No coverage for the `MiniJinja` path (engine enum at the DAO
    boundary).

## 9. History & Context

- **2025-08 (v1.2)** — text-template scaffold (`text_template` table,
  `template_type`, `template_text`). Motivation: configurable Billing
  Period Custom Reports without deploy
  (see `docs/template-examples/README.md`).
- **2025-08-17 (v1.2)** — `name` column added so HR can distinguish
  templates in the list by name rather than by UUID.
- **2025-10 (v1.2 Phase 6)** — User Invitation MVP: base table,
  single-use token, public redeem route with dual-featured OIDC / mock
  auth path. The comment in `rest/src/user_invitation.rs:19-22` refers
  to the DTO migration to `rest-types`.
- **2025-10-17** — session tracking (`redeemed_at`, `session_id`) —
  goal: from "single-use" to "single-use and we know which Session came
  of it".
- **2025-10-20** — `session_revoked_at` as a dedicated status marker so
  admin-driven Session revocation stays visibly distinguishable from
  natural expiry.
- **2026-03-12 (current milestone)** — `template_engine` as a second
  engine slot (`Tera` remains default, `MiniJinja` new). No backward
  compatibility break; old rows fall back to `tera`.
- For further context reads: `docs/template-examples/` (consumer chain),
  `.planning/phases/…` — **[To verify]** which concrete phase contained
  the invitation migration (directory scan found no obvious `invitation`
  slug; presumably archived under the v1.2 Phase 6 slug).

---

*Last verification against code:* see git blame of this file.

---

**Conclusion:** F10 encapsulates two orthogonal but semantically related
communication stubs — text templates as pure HR CRUD on a lean
`text_template` table (engine selection since 2026-03-12), and user
invitation as an admin-gated invitation flow with single-use token,
Session anchor, and separate "hard delete" vs. "revoke Session" paths.
The largest open construction sites are the missing service-level unit
tests for the auth-adjacent redeem path and the aggressive cleanup
semantics that also delete redeemed invitations.
