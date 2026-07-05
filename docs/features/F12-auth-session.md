# Feature: Auth & Session — OIDC/Mock, Permissions, Impersonation

> **Kurzform:** Sichert alle Shifty-Endpoints ab — in Produktion via OIDC, in
> Dev via Mock-Session; jeder Request bekommt einen Nutzer-`Context`, gegen den
> Rollen und Privilegien geprüft werden. Admins können zu Support-Zwecken die
> Identität anderer Nutzer temporär übernehmen (Impersonation).

**Cluster-ID:** F12
**Status:** produktiv
**Erstmalig eingeführt:** Rollen-Basissatz 04/2024 (`20240426150045_user-roles.sql`),
Sessions 11/2024 (`20241116224840_add-session.sql`), Impersonation 04/2026
(`20260401000000_add-impersonate-to-session.sql`)
**Zuständige Crates:** `service::permission`, `service::session`,
`service::user_service`, `service_impl::permission`, `service_impl::session`,
`service_impl::lib::UserServiceImpl`, `dao::permission`, `dao::session`,
`rest::session`, `rest::permission`, `rest::impersonate`, `rest::dev`,
`shifty-dioxus::service::auth`, `shifty-dioxus::page::not_authenticated`

---

## 1. Was ist das? (Fachlich)

Auth & Session ist der zentrale Zugangs-Layer von Shifty. Er entscheidet für
jeden HTTP-Request: *Wer ist der Aufrufer, existiert eine gültige Session, und
darf er das gefragte Aggregat sehen oder ändern?*

- In **Produktion** läuft Shifty gegen einen externen OIDC-Provider
  (Keycloak/Authelia). Nach erfolgreichem Login schreibt das Backend eine
  Session-ID als HTTP-only Cookie und persistiert die Session in SQLite.
- In **Dev/Test** existiert eine Mock-Variante: Sobald ein Request ohne
  Session-Cookie ankommt, wird automatisch ein `DEVUSER` angelegt und ein
  Session-Cookie gesetzt — Login-freies Arbeiten im Browser (`rest/src/session.rs:180-260`).
- **Rollen und Privilegien** bestimmen, welche Funktionen einem Nutzer offen
  stehen: `admin`, `hr`, `sales`, `shiftplanner`, `shiftplan.edit`.
- **Impersonation** ist der Support-Modus: Ein Admin darf temporär "als" ein
  anderer Nutzer arbeiten (typischer Use-Case: Sicht auf fremde Reports oder
  Reproduzieren von Bugs). Der Original-Admin bleibt für Audit-Logs
  identifizierbar (`RealUser`-Extension).

**Beispiel-Workflow aus Admin-Sicht:**

1. Admin öffnet `/user-management`, sucht Nutzer "anna" in der Users-Tab.
2. Klickt "Als anna arbeiten" — Backend setzt `session.impersonate_user_id = "anna"`.
3. Ab jetzt sieht Admin Shifty wie anna (eigene Reports, eigener Balance).
   Alle write-Requests werden mit `real_user = ADMIN, acting_as = anna` geloggt.
4. Admin klickt "Zurück zu meiner Identität" — `impersonate_user_id = NULL`,
   normale Admin-Sicht restauriert.

---

## 2. Fachliche Regeln

- **Regel A (Session-Cookie):** Der Cookie `app_session` (HTTP-only, SameSite=Strict,
  Secure) trägt die UUID der Session; Server-side wird sie gegen die `session`-Tabelle
  aufgelöst (`rest/src/session.rs:137-176`).
- **Regel B (OIDC-Gate):** Im OIDC-Build blockt `forbid_unauthenticated`
  (`rest/src/session.rs:270-293`) alle Requests ohne aufgelösten `Context`; nur
  `/authenticate` und `/*/ical` sind offen.
- **Regel C (Mock-Bypass):** Im Mock-Build lässt `forbid_unauthenticated`
  jeden Request durch (`rest/src/session.rs:262-269`) — Session wird bei Bedarf
  automatisch für `DEVUSER` erzeugt.
- **Regel D (Session-Lebensdauer):** `expires = created + 3600·24·365` (365 Tage,
  hart kodiert in `service_impl/src/session.rs:29`). Cookie-Expiry deckt sich
  (`+ time::Duration::days(365)` in `rest/src/session.rs:116, 217, 241`).
- **Regel E (Rollen-Enumeration):** Die Basisrollen `admin`, `sales`, `hr` sind
  im Bootstrap gesetzt (`20240426150045_user-roles.sql:100-112`), `shiftplanner`
  wurde 06/2024 nachgezogen (`20240614075633_shiftplanner-role.sql`),
  `shiftplan.edit` 11/2024 (`20241118165756_add-role-shiftplan-edit.sql`).
- **Regel F (Privileg-Ableitung):** Ein Nutzer hat ein Privileg, wenn eine
  seiner zugewiesenen Rollen dieses Privileg trägt (Join in `permission_dao.has_privilege`,
  konsumiert von `check_permission`, siehe `service_impl/src/permission.rs:35-55`).
- **Regel G (Admin darf alles):** Alle Admin-Operationen (Rollen anlegen, Users
  anlegen, User-Roles zuweisen) sind mit `check_permission("admin", …)` gegated
  (`service_impl/src/permission.rs:126, 140, 162, 177, 191, 199, 213, 231, 239, 255, 267, 279, 291`).
- **Regel H (`hr`-Privileg für User-Existence):** `user_exists` erfordert `hr`,
  nicht `admin` (`service_impl/src/permission.rs:150`) — HR-Rollen dürfen prüfen,
  ob ein User bekannt ist, ohne Admin zu sein.
- **Regel I (`Authentication::Full`-Bypass):** Interne Aufrufer, die keinen
  echten User-Context haben (Scheduler, Aggregations-Services, Dev-Seed), dürfen
  `Authentication::Full` übergeben; alle `PermissionService`-Methoden mit
  Full-Zweig geben sofort `Ok(())` bzw. `Ok(None)` zurück
  (`service_impl/src/permission.rs:28, 41, 63, 80, 90`).
- **Regel J (Impersonate ist admin-only):** Alle drei Impersonate-Endpoints
  prüfen `admin` gegen die *reale* `session.user_id`, nicht gegen den
  effektiven Context (`rest/src/impersonate.rs:67-72, 136-141, 192-198`).
- **Regel K (Audit für Impersonated Writes):** Solange eine Session
  impersoniert, wird jeder mutierende HTTP-Verb (POST/PUT/PATCH/DELETE) mit
  `real_user`, `acting_as`, `method`, `path` per `tracing::info!` protokolliert
  (`rest/src/session.rs:40-87`).

---

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `user` | Nutzer-Stamm (Username = PK) | `name`, `update_timestamp`, `update_process` |
| `role` | Rollen (Name = PK) | `name`, `update_process` |
| `privilege` | Privilegien (Name = PK) | `name`, `update_process` |
| `user_role` | N:M Nutzer↔Rolle | `user_name`, `role_name` (UNIQUE) |
| `role_privilege` | N:M Rolle↔Privileg | `role_name`, `privilege_name` (UNIQUE) |
| `session` | Aktive Sessions | `id` (PK, UUID), `user_id` (FK→user), `expires`, `created`, `impersonate_user_id` |

### Migrations (chronologisch)

- `20240426150045_user-roles.sql` — Basistabellen `user`, `role`, `privilege`,
  `user_role`, `role_privilege` mit Update-Triggers; Bootstrap der Rollen
  `admin/sales/hr` und der gleichnamigen Privilegien. Zusätzlich der Helper-View
  `V_UUID_V4`.
- `20240614075633_shiftplanner-role.sql` — Ergänzt Rolle+Privileg
  `shiftplanner`; `admin` erbt es automatisch via `role_privilege`.
- `20241116224840_add-session.sql` — `session`-Tabelle (initial mit `id`
  nullable — siehe Folge-Migration).
- `20241118165756_add-role-shiftplan-edit.sql` — Ergänzt Rolle+Privileg
  `shiftplan.edit`; `admin` erbt es.
- `20241118180147_make-session-id-not-null.sql` — Neuaufbau `session`-Tabelle
  mit `id TEXT NOT NULL PRIMARY KEY`. Daten werden per SELECT/INSERT übernommen.
- `20260401000000_add-impersonate-to-session.sql` — `ALTER TABLE session ADD
  COLUMN impersonate_user_id TEXT NULL`. Ermöglicht Support-Impersonation ohne
  Session-Neuaufbau.

Weitere Migrations zu User-Invitation liegen im Cluster **F13 (User Invitation)**
und sind hier bewusst nicht aufgeführt.

### Beziehungen

```
user  ─┬─< user_role >─┐
       │               │
       └──< session    └─> role ─< role_privilege >─ privilege
```

Alle FKs sind `ON DELETE CASCADE` in den Join-Tables, sodass Löschen eines
Users seine Rollenzuweisungen mit entfernt. Die `session`-FK auf `user` ist
nicht CASCADE — ein User-Delete bei aktiver Session würde fehlschlagen.
**[Zu prüfen]** ob das im Prod-Betrieb je auftritt.

## 4. Service-API

### 4.1 `Authentication<Context>`-Enum

`service/src/permission.rs:49-60`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Authentication<Context: Clone + PartialEq + Eq + Send + Sync + Debug + 'static> {
    Full,
    Context(Context),
}
```

Zwei Varianten:

- **`Authentication::Full`** — "Vertrauter interner Aufrufer". Kein User,
  keine Rollenprüfung, alle Permission-Gates werden übersprungen. Wird von
  Schedulern (`service_impl/src/scheduler.rs:60`, `pdf_export_scheduler.rs`
  Zeilen 220/251/279/293/302/317/335/344/373/381/391/418/436), Aggregations-
  Services (`service_impl/src/extra_hours.rs:51,54`) und dem Dev-Seeder
  (`rest/src/dev.rs:128,149,191`) genutzt.
- **`Authentication::Context(ctx)`** — Trägt den echten User-Kontext. Für die
  REST-Layer ist `Context = Option<Arc<str>>` (`rest/src/session.rs:12`) —
  der aufgelöste Username aus dem Session-Cookie oder `None` bei fehlender
  Auth. Über `From<Context>` (`service/src/permission.rs:54-60`) wird jedes
  `Context` automatisch in `Context(ctx)` gewrappt, was den `context.into()`-
  Pattern in allen REST-Handlern erklärt.

### 4.2 `PermissionService`

`service/src/permission.rs:64-169`. `Context: Clone + PartialEq + Eq + Debug + Send + Sync + 'static`.

```rust
async fn check_permission(&self, privilege: &str, ctx: Authentication<Self::Context>)
    -> Result<(), ServiceError>;
async fn check_only_full_authentication(&self, ctx: Authentication<Self::Context>)
    -> Result<(), ServiceError>;
async fn check_user(&self, user: &str, ctx: Authentication<Self::Context>)
    -> Result<(), ServiceError>;
async fn current_user_id(&self, ctx: Authentication<Self::Context>)
    -> Result<Option<Arc<str>>, ServiceError>;
async fn get_privileges_for_current_user(&self, ctx: Authentication<Self::Context>)
    -> Result<Arc<[Privilege]>, ServiceError>;
async fn get_roles_for_user(&self, user: &str, ctx: Authentication<Self::Context>)
    -> Result<Arc<[Role]>, ServiceError>;
// + CRUD für user/role/privilege/user_role/role_privilege
```

Wichtige Semantik-Details:

- **`check_permission("X", Full) → Ok(())`** (Zeile 41). Bypass.
- **`check_permission("X", Context(ctx))`** löst den User via
  `UserService::current_user(ctx)` auf, lädt Privilegien per DAO und returned
  `Forbidden` wenn nicht gefunden (Zeilen 43-53).
- **`check_only_full_authentication`** ist der *inverse* Gate: erlaubt
  **nur** Full und weist jeden User-Context zurück (Zeile 75-83). Genutzt für
  Endpoints, die niemals von außen erreichbar sein dürfen. **[Zu prüfen]** —
  aktuell scheint diese Methode keinen produktiven Aufrufer zu haben, ein
  Grep über `service_impl/` und `rest/` liefert keine Treffer außer der
  Trait-Definition selbst.
- **`get_privileges_for_current_user(Full)` → `[Privilege { name: "god-mode" }]`**
  (Zeile 90-92). Symbolischer Marker, damit der Dev-Seed-Pfad im Frontend
  keine leere Privilegien-Liste sieht.

### 4.3 `SessionService`

`service/src/session.rs:44-56`. Kein `Context`-parametrierter Auth-Check —
alle Methoden nehmen rohe IDs und werden **vor** der Auth-Layer aufgerufen,
denn Sessions sind die *Grundlage* der Auth, nicht deren Konsument.

```rust
async fn new_session_for_user(&self, user_id: &str) -> Result<Session, ServiceError>;
async fn invalidate_user_session(&self, id: &str) -> Result<(), ServiceError>;
async fn verify_user_session(&self, id: &str) -> Result<Option<Session>, ServiceError>;
async fn start_impersonate(&self, session_id: Arc<str>, target_user_id: Arc<str>)
    -> Result<(), ServiceError>;
async fn stop_impersonate(&self, session_id: Arc<str>) -> Result<(), ServiceError>;
```

Neue Sessions bekommen eine UUID (`service_impl/src/session.rs:32-36`) und
laufen 365 Tage.

### 4.4 `UserService`

`service/src/user_service.rs:11-15`. Minimales Trait:

```rust
async fn current_user(&self, context: Self::Context) -> Result<Arc<str>, ServiceError>;
```

Die produktive Implementierung ist trivial (`service_impl/src/lib.rs:54-66`):
`Context = Option<Arc<str>>`; ist `Some(name)` gesetzt, wird es
zurückgegeben, sonst `Unauthorized`. Der eigentliche Login passiert also
schon vorher in der Session-Middleware, `UserService` ist nur der
Übersetzungspunkt vom REST-Context zum konkreten Username.

### 4.5 Auth-Gates — Übersicht

| Methode | Gate |
| --- | --- |
| `check_permission` | `Full` → Ok; sonst DAO `has_privilege` |
| `check_user` | `Full` → Ok; sonst Vergleich `current_user == user` |
| `get_roles_for_user` | `admin` |
| `create_user`/`delete_user`/`get_all_users` | `admin` |
| `user_exists` | `hr` |
| `create_role`/`delete_role`/`get_all_roles` | `admin` |
| `create_privilege`/`delete_privilege`/`get_all_privileges` | `admin` |
| `add_user_role`/`delete_user_role` | `admin` |
| `add_role_privilege`/`delete_role_privilege` | `admin` |

### 4.6 TX-Verhalten

Weder `PermissionService` noch `SessionService` verwenden `Transaction`-Parameter.
Alle Ops sind Single-Statement-DAO-Calls; die transaktionale Konsistenz von
Rollen/Privilegien ist nicht kritisch, weil Änderungen selten und atomar
per SQL-Statement erfolgen. Impersonate ist ein `UPDATE`, nicht atomar mit
weiteren Ops.

### 4.7 Dependencies

- **`PermissionServiceImpl`** (`service_impl/src/permission.rs:10-15`):
  `PermissionDao`, `UserService`. Basic-Service — keine Domain-Services.
- **`SessionServiceImpl`** (`service_impl/src/session.rs:14-20`):
  `SessionDao`, `UuidService`, `ClockService`. Basic-Service.
- **`UserServiceImpl`** (`service_impl/src/lib.rs:54`): keine Deps.

## 5. REST-Endpoints

### 5.1 Auth-Info / Login

| Methode | Pfad | Beschreibung | DTO Out | Anmerkung |
| --- | --- | --- | --- | --- |
| `GET` | `/authenticate` | Login-Einstieg — 302 auf `/` (OIDC handled Redirect vorher) | — | `rest/src/lib.rs:507` |
| `GET` | `/logout` | OIDC-Logout (Redirect zum IdP) | — | Nur `oidc`-Feature, `rest/src/lib.rs:523` |
| `GET` | `/auth-info` | Aktueller User + Privilegien | `AuthInfoTO { user, privileges }` | `rest/src/lib.rs:537-564` |

### 5.2 Permission-CRUD (`/permission`)

Route-Aufbau: `rest/src/permission.rs:18-35`. Alle Endpoints delegieren an
`PermissionService` und rufen `check_permission("admin", …)` implizit.

| Methode | Pfad | DTO | Wichtige Fehler |
| --- | --- | --- | --- |
| `GET` | `/user` | `[UserTO]` | 401, 403 |
| `POST` | `/user` | `UserTO` | 400, 403 |
| `DELETE` | `/user/` | body `String` | 403, 404 |
| `GET` | `/role` | `[RoleTO]` | 403 |
| `POST` | `/role` | `RoleTO` | 403 |
| `DELETE` | `/role` | body `String` | 403, 404 |
| `GET` | `/user/{user}/roles` | `[RoleTO]` | 403, 404 |
| `GET` | `/privilege/` | `[PrivilegeTO]` | 403 |
| `POST` | `/user-role` | `UserRole` | 403 |
| `DELETE` | `/user-role` | `UserRole` | 403 |
| `POST` | `/role-privilege/` | `RolePrivilege` | 403 |
| `DELETE` | `/role-privilege/` | `RolePrivilege` | 403 |

### 5.3 Impersonate (`/admin/impersonate`)

Route-Aufbau: `rest/src/impersonate.rs:27-32`. Wichtige Eigenheit: Der
Admin-Check läuft immer gegen die *reale* `session.user_id` (D-32-02, Kommentar
`rest/src/lib.rs:688-699`).

| Methode | Pfad | Body/Path | Beschreibung |
| --- | --- | --- | --- |
| `GET` | `/` | — | Status: `ImpersonateTO { impersonating, user_id }` |
| `POST` | `/{user_id}` | Path | Impersonation starten, `session.impersonate_user_id = user_id` |
| `DELETE` | `/` | — | Impersonation beenden, Spalte auf NULL |

DTO: `ImpersonateTO { impersonating: bool, user_id: Option<Arc<str>> }`
(`rest-types/src/lib.rs:1707-1712`).

Audit-Log-Zeilen werden **nach** erfolgreichem Service-Call emittiert
(`rest/src/impersonate.rs:92-96, 149-153`), damit fehlgeschlagene Aufrufe kein
False-Positive im Audit erzeugen (D-32-01/WR-01, WR-02).

### 5.4 Dev-Endpoints (`/dev`, nur `mock_auth`-Feature)

Nur im Dev-Build gemountet (`rest/src/lib.rs:685-686`). Aggregiert nutzt
`Authentication::Full` als Bypass (`rest/src/dev.rs:128`).

| Methode | Pfad | Beschreibung |
| --- | --- | --- |
| `POST` | `/dev/seed` | Legt Anna/Max/Lisa/Tom/Sarah + WorkDetails + Bookings + SpecialDays an |
| `POST` | `/dev/clear` | `basic_dao.clear_all()` — **destruktiv** |

### 5.5 Middleware-Stack

Aufbau `rest/src/lib.rs:700-720` (Tower wickelt in *umgekehrter* Reihenfolge —
letzter `.layer()` läuft als äußerster):

```text
[Cookies]
  └─ [context_extractor] (setzt Context + optional RealUser)
        └─ [audit_impersonated_writes] (loggt POST/PUT/PATCH/DELETE bei Impersonation)
              └─ [forbid_unauthenticated] (nur OIDC: 401 wenn Context leer)
                    └─ Handler
```

Der Mounting-Kommentar in `rest/src/session.rs:52-62` erklärt die Layer-Reihenfolge
explizit und ist relevant, wenn man den Stack refaktoriert.

## 6. Frontend-Integration

- **Pages:** `shifty-dioxus/src/page/not_authenticated.rs` (24 Zeilen —
  Willkommens-Screen mit Link auf `/authenticate`, wird nur im OIDC-Build
  erreicht, weil im Mock-Modus `context_extractor` sofort einen `DEVUSER`
  anlegt); `user_management.rs` (878 Zeilen — Tab-Layout Users/SalesPersons,
  Add/Delete-User Dialoge, "Als … arbeiten"-Button für Impersonation ab
  Zeile 42); `user_details.rs` (333 Zeilen — Rollen-Zuweisung + Invitation-Liste).
- **Auth-Guard:** `shifty-dioxus/src/auth.rs` (24 Zeilen) — `<Auth>`-Komponente
  mit `authenticated`/`unauthenticated`-Slots, ausgewählt anhand
  `AUTH.auth_info` und `loading_done`. Solange geladen wird, zeigt sie
  "Fetching auth information…".
- **Auth-Service:** `shifty-dioxus/src/service/auth.rs` — `AUTH: GlobalSignal<AuthStore>`
  hält `AuthInfo { user, privileges }`. `load_auth_info()` ruft
  `api::fetch_auth_info` → `GET /auth-info` und befüllt den Store.
- **Impersonate-Service:** `shifty-dioxus/src/service/impersonate.rs` — eigener
  Coroutine mit `ImpersonateAction`, konsumiert von `user_management.rs:42`.
- **State:** `shifty-dioxus/src/state/*` — `AuthInfo` als Domain-Type.
- **i18n-Keys:** `WelcomeTitle`, `PleaseLogin` (Not-Authenticated),
  `UserManagement`, `BackToUserManagement`.
- **Proxy:** `Dioxus.toml` proxied `/permission`, `/auth-info`, `/authenticate`,
  `/admin/impersonate`. **[Zu prüfen]** — im Feature-Kontext nicht direkt
  aufgelistet, aber Konvention (siehe MEMORY.md Eintrag "Dioxus.toml Proxy").

## 7. Randfälle

Für die zentrale Randfall-Referenz siehe
[`../domain/edge-cases.md#6-authentifizierung--autorisierung`](../domain/edge-cases.md#6-authentifizierung--autorisierung).

- **Full-Bypass-Missbrauch:** Jeder Konsument, der `Authentication::Full` an
  einen Service reicht, umgeht **alle** Rollen-Checks. Das ist *by design*
  für interne Aggregate (siehe Toggle-Bypass-Regel im MEMORY-Eintrag
  "ToggleService Full-Context-Bypass"), aber ein einziger falsch platzierter
  `Full`-Aufruf aus einem HTTP-Handler wäre eine Privilege-Escalation. Regel:
  REST-Handler geben *immer* `context.into()` weiter — nur Scheduler, Cron
  und Startup-Migrationen dürfen `Full` konstruieren.
- **Token-Expiry:** Sessions leben 365 Tage. Läuft eine Session ab, liefert
  `verify_user_session` `None` — im OIDC-Modus resultiert das in 401,
  im Mock-Modus wird eine neue `DEVUSER`-Session frisch erzeugt
  (`rest/src/session.rs:210-233`).
- **Rollenwechsel mid-session:** Wenn ein Admin einem User während dessen
  aktiver Session eine Rolle entzieht, wirkt das sofort — jede
  `check_permission`-Anfrage prüft *live* gegen die DB. Es gibt kein
  Privileg-Caching im Backend. Der Frontend-Store `AUTH` cached aber
  bis zum nächsten Reload; UI kann also stale-privilegienbasiert Dinge
  anzeigen, die Backend beim tatsächlichen Aufruf ablehnt.
- **Impersonate + Admin-Only Endpoints:** Impersoniert ein Admin einen
  Non-Admin und ruft eine `check_permission("admin")`-gegatete Route auf,
  scheitert der Aufruf mit 403 — der effektive Context ist der Non-Admin
  (`rest/src/lib.rs:694-696`, D-32-02a). Nur `/admin/impersonate/*`
  bleibt zugänglich, weil dort explizit gegen die reale `session.user_id`
  geprüft wird.
- **Session ohne User:** `session.user_id` verweist per FK auf `user(name)`.
  Wird ein User gelöscht, während seine Session läuft, führt das zu einem
  Konsistenz-Bruch (kein `ON DELETE CASCADE`). **[Zu prüfen]** ob
  `permission_service.delete_user` die Sessions dieses Users räumt — im
  Trait und in der SQLite-Impl scheint das nicht der Fall zu sein.
- **`impersonate_user_id` verweist auf gelöschten User:** `start_impersonate`
  prüft Existenz per `user_exists` mit `Authentication::Full`
  (`rest/src/impersonate.rs:75-84`), aber späteres Löschen des Ziel-Users
  wird nicht kaskadiert.
- **Non-authenticated in OIDC:** `forbid_unauthenticated` lässt
  `/authenticate` und `/*/ical` durch. Alles andere → 401. Im Mock-Build
  passiert das nie, weil `context_extractor` immer einen Context erzeugt.

## 8. Tests

- **Unit `service_impl/src/test/permission_test.rs`** (472 Zeilen) —
  vollständige Abdeckung von `check_permission`, `create_user`,
  `delete_user`, `create_role`, `add_role_privilege`, `delete_role_privilege`,
  Roles/Privileges-Listen. Mock-Setup nutzt `NoneTypeExt::auth()`
  (`error_test.rs:130-137`), das Context `()` in `Authentication::Context(())`
  wickelt — damit werden die *nicht*-Full-Pfade getestet. Full-Pfad wird
  implizit über die Dev-Seed-Integration im Live-Backend abgedeckt.
- **Unit `service_impl/src/test/session.rs`** (120 Zeilen) —
  `test_start_impersonate`, `test_stop_impersonate`, plus Session-CRUD.
  Mockt `SessionDao` mit `expect_update_impersonate`.
- **Unit `rest/src/session.rs` (Modul `tests`)** — Tests für
  `real_user_extension` (present/absent) und `should_audit_impersonated_write`
  über alle HTTP-Verben (POST/PUT/PATCH/DELETE audit; GET/HEAD/OPTIONS nicht;
  nicht-impersonating nie).
- **Integration `service_impl/src/test/permission_test.rs::test_user_service_impl_*`**
  — testet `UserServiceImpl` direkt für Some/None-Context.
- **Bekannte Lücken:**
  - Kein automatisierter End-to-End-Test für den Impersonate-Audit-Log-Layer
    (nur die Pure-Function `should_audit_impersonated_write` ist gedeckt).
  - Kein Test dafür, dass `check_only_full_authentication` bei einem echten
    User-Context wirklich 403 liefert. **[Zu prüfen]** ob überhaupt ein
    Konsument existiert, sonst kann die Methode gestrichen werden.
  - Kein Test dafür, dass gleichzeitige Rollen-Änderung während laufender
    Session korrekt wirkt (Live-DB-Semantik).

## 9. Historie & Kontext

- **04/2024** — Basis-Auth (`20240426150045_user-roles.sql`): drei Rollen
  `admin/sales/hr`, drei gleichnamige Privilegien. Damals noch ohne Sessions —
  Auth lief rein über OIDC-Cookies (`SessionManagerLayer` vom `axum-oidc`-Crate).
- **06/2024** — `shiftplanner`-Rolle nachgezogen; erste Ausdifferenzierung
  zwischen HR (Zeiterfassung) und Planung.
- **11/2024** — `session`-Tabelle eingeführt (Hintergrund: iCal-Feeds und
  Custom-Cookies brauchten einen eigenen Session-Store getrennt vom
  OIDC-Middleware-Session). Zwei Migrations weil erste `id`-Definition
  fälschlich nullable war.
- **11/2024** — `shiftplan.edit`-Rolle: Splittet "Planer sehen" (`shiftplanner`)
  von "Planer schreiben" (`shiftplan.edit`). Motivation: Filialleitung sieht,
  ändert aber nicht selbst.
- **04/2026 (Phase 32 — Impersonation)** — `impersonate_user_id` in
  `session`-Tabelle. Kontext siehe D-32-01 (Audit-Log für Writes) und
  D-32-02 (Two-Path Admin-Gate). Frontend-Anteil D-32-07/IMP-01
  (`user_management.rs:42`).
- **Verweise:** `.planning/phases/32-impersonation/` für Kontext-Reads;
  Cluster **F13 (User Invitation)** für den Token-basierten Invite-Login-
  Fluss, der parallel zur regulären OIDC-Auth existiert und im Route-Mount
  bewusst *hinter* dem `forbid_unauthenticated`-Layer platziert ist
  (`rest/src/lib.rs:752-759`), damit er ohne bestehende Session
  konsumierbar ist.

---

**Fazit:** F12 sichert Shifty mit einem klaren Zwei-Modi-Muster (OIDC prod,
Mock dev) und einer bewusst schmalen `Authentication<Context>`-API, deren
`Full`-Variante der einzige Bypass für interne Aggregate ist. Impersonation
erweitert das ohne Bruch, indem die reale Admin-Identität als `RealUser`-
Extension erhalten und für Writes hart auditiert wird.

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
