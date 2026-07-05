# Feature: Text-Templates & User-Invitation (Kommunikation)

> **Kurzform:** Wiederverwendbare Text-/HTML-Templates (Tera/MiniJinja) für
> Reports plus ein Einladungs-Flow, der Nutzer per einmaligem Link/Token in
> das System bringt und die daraus entstandene Session serverseitig
> widerrufbar macht.

**Cluster-ID:** F10
**Status:** produktiv
**Erstmalig eingeführt:** Text-Templates 2025-08 (v1.2), User-Invitation
2025-10 (v1.2 Phase 6)
**Zuständige Crates:**
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
  `shifty-dioxus/src/page/user_details.rs` (Invitation-Panel)

---

## 1. Was ist das? (Fachlich)

F10 bündelt zwei Bereiche, die beide "Kommunikation nach außen" abdecken —
einmal in Richtung *Report-Empfänger* (Text-Templates), einmal in Richtung
*neue Nutzer* (User-Invitation).

### 1.1 Text-Templates

Text-Templates sind vom HR verwaltete, wiederverwendbare Vorlagen für
Berichte. Sie werden derzeit vom Custom-Billing-Period-Report konsumiert
(`POST /billing-period/{billing_period_id}/custom-report/{template_id}`, siehe
`docs/template-examples/README.md`), sind aber generisch: Jeder Template-Typ
ist ein freier String (`template_type`), sodass später weitere Konsumenten
(z. B. Shiftplan-Reports, E-Mail-Vorlagen) angebunden werden können, ohne das
Schema zu ändern. Aktuell im Frontend fest verdrahtete Typen sind
`billing-period` und `shiftplan-report`
(`text_template_management.rs:169-177`).

Ein Template besteht aus:
- optionalem, sprechenden **Namen** (`name`, seit Migration 2025-08-17),
- **`template_type`** — Fach-Kategorie / Konsumkette (Filter im FE),
- **`template_text`** — der eigentliche Template-Body (HTML, Text, …),
- **`template_engine`** — `Tera` (default) oder `MiniJinja` (seit
  Migration 2026-03-12).

**Beispiel-Workflow aus User-Sicht (Text-Templates):**

1. HR öffnet "Text Template Management".
2. HR klickt "Add New", vergibt Name, wählt Typ (`billing-period`), wählt
   Engine (`Tera`), fügt Template-Body ein.
3. Speichern → Template steht im REST bereit.
4. HR wählt in "Billing Period Details" ein Template aus und lässt den
   Custom-Report rendern.
5. Fehlerhafter Body oder fehlende Variable → Fehler kommt vom Konsumenten
   (Billing-Period-Custom-Report), nicht vom Template-Service selbst.

### 1.2 User-Invitation

Der Einladungs-Flow ist der einzige Weg, wie ein Admin einen neuen Nutzer
außerhalb von OIDC/Mock-Auth ins System bringt. Ergebnis der Einladung ist
ein **einmaliger Link** mit einem eingebetteten UUID-Token, den der
Eingeladene im Browser aufruft. Auf Backend-Seite werden dabei ggf. der
User in `permission_dao` angelegt und eine Session ausgestellt; die
Invitation wird an dieser Session verankert und behält damit den Bezug
"Wer wurde wann per welchem Link eingeladen und sitzt jetzt in welcher
Session?".

**Beispiel-Workflow aus User-Sicht (Invitation):**

1. Admin öffnet "User Details" für einen Zielnutzer und drückt "Invitation
   erzeugen" (Default: 7 Tage Gültigkeit).
2. Backend erzeugt Invitation-Record + Token, FE zeigt den Link zum
   Kopieren.
3. Admin gibt den Link an den Eingeladenen (Chat, E-Mail, Zettel, …).
4. Eingeladener öffnet `/auth/invitation/{token}` → Backend validiert
   Token, legt User an falls nötig, erstellt Session, setzt Cookie,
   redirected auf `/`, markiert Invitation als `Redeemed` samt
   `session_id`.
5. Admin kann jederzeit im FE:
   - eine **unverbrauchte** Invitation *revoken* (löschen) → Link
     wird ungültig;
   - eine **eingelöste** Invitation "Session revoken" → aktive Session
     des Eingeladenen wird invalidiert; Invitation trägt danach den
     Status `SessionRevoked`.

## 2. Fachliche Regeln

### 2.1 Text-Templates

- Alle Schreib-Operationen (`create`, `update`, `delete`) verlangen
  `HR_PRIVILEGE`
  (`service_impl/src/text_template.rs:78,112,149`).
- Lese-Operationen (`get_all`, `get_by_id`, `get_by_template_type`) haben
  **kein** eigenes Permission-Gate im Service
  (`service_impl/src/text_template.rs:26-69`). **[Zu prüfen]** ob das
  gewollt ist oder ob die Zugriffsbegrenzung ausschließlich am REST-
  Auth-Middleware-Layer erwartet wird — für einen HR-only Nutzen wäre ein
  konsistentes Gate sauberer.
- `id` wird beim `create` serverseitig neu gewürfelt und ist immutabel; ein
  vom Client mitgeschickter `id`-Wert wird überschrieben
  (`service_impl/src/text_template.rs:90`).
- `version` wird bei jedem `create`/`update` neu gewürfelt
  (`service_impl/src/text_template.rs:91,131`).
- `created_at`/`created_by` werden nur beim `create` gesetzt und in `update`
  aus der Bestandsversion übernommen
  (`rest/src/text_template.rs:233-238`).
- Vor `update` und `delete` prüft der Service explizit die Existenz und
  liefert sonst `EntityNotFoundGeneric`
  (`service_impl/src/text_template.rs:124-128,161-165`).
- `template_engine` fällt per DB-DEFAULT auf `tera` zurück, wenn das Feld
  fehlt (Migration 2026-03-12). Erlaubte Werte am DAO-Rand: `tera`,
  `minijinja` — sonst `DaoError::EnumValueNotFound`
  (`dao/src/text_template.rs:16-28`).
- Soft-Delete: DAO-Queries filtern `deleted IS NULL` (Konvention laut
  Projekt-CLAUDE.md; im DAO-Impl auf gleiche Weise). **[Zu prüfen]** ob das
  Text-Template-DAO die Soft-Delete-Konvention konsistent umsetzt oder tat-
  sächlich löscht — die Trait-Signatur `delete(id, process, tx)` sagt es
  nicht.

### 2.2 User-Invitation

- Alle Admin-Aktionen (`generate_invitation`, `list_invitations_for_user`,
  `find_invitation_by_session`, `revoke_invitation`,
  `revoke_session_for_invitation`) hängen am Permission-String `"admin"`
  (`service_impl/src/user_invitation.rs:53,150,180,214,254`).
- **`validate_and_consume_token` und `mark_token_redeemed` sind
  bewusst ungate-t**, weil sie im nicht-authentifizierten Bootstrap-Path
  laufen (`rest/src/user_invitation.rs:36-128`) — sonst könnte niemand
  seinen Einladungs-Link je einlösen.
- Default-Expiration: `expiration_hours = 7*24` = 7 Tage
  (`rest/src/user_invitation.rs:150`).
- Token wird nur einmal einlösbar konsumiert: `validate_and_consume_token`
  wirft `EntityNotFoundGeneric("Invitation token has already been used")`
  sobald `session_id.is_some()`
  (`service_impl/src/user_invitation.rs:106-110`).
- Abgelaufener Token: identischer Fehlertyp mit Message
  `"Invitation token has expired"`
  (`service_impl/src/user_invitation.rs:112-117`).
- User-Autocreate: Wenn `permission_dao.find_user(username)` beim Einlösen
  `None` liefert, wird der User via `create_user(...,
  USER_INVITATION_SERVICE_PROCESS)` angelegt
  (`service_impl/src/user_invitation.rs:119-133`).
- Session-Anker: Der REST-Layer erzeugt die Session
  (`SessionService::new_session_for_user`) *nach* erfolgreicher Token-
  Validierung und ruft dann `mark_token_redeemed(token, session_id)` auf
  (`rest/src/user_invitation.rs:53-70`). Damit steht `session_id` NUR
  auf der eingelösten Invitation, nicht auf allen offenen.
- Status-Ableitung (`InvitationStatus`, `service_impl/src/user_invitation.rs
  :28-38`): Reihenfolge der Checks: `session_revoked_at` >
  `redeemed_at` > `expiration_date < now` > sonst `Valid`. **Wichtig:**
  ein per Admin widerrufener Link (`revoke_invitation` = hartes
  `delete_by_id`) taucht danach gar nicht mehr auf; nur die
  Session-Revocation ist als eigener Status sichtbar.
- Cookie-Semantik im OIDC-Pfad: `path="/"`, `expires=now+365d`,
  `http_only`, `SameSite=Strict`, `secure=true`
  (`rest/src/user_invitation.rs:73-81`).
- Cleanup: `cleanup_expired_invitations` löscht **alle** abgelaufenen
  Rows harten Wegs (`dao_impl_sqlite/src/user_invitation.rs:211-226`),
  auch bereits eingelöste — d. h. wer nach Ablauf noch die Audit-Info
  "wer war wann eingeladen" sucht, findet sie nach Cleanup nicht mehr.
  **[Zu prüfen]** ob das gewollt ist oder ob "nur unbenutzte
  abgelaufene" das Ziel wäre.

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `text_template` | Vorlagen-Repository für Reports | `id BLOB PK`, `name TEXT?`, `template_type TEXT`, `template_text TEXT`, `template_engine TEXT NOT NULL DEFAULT 'tera'`, `created_at`, `created_by`, `deleted`, `deleted_by`, `update_version BLOB`, `update_process TEXT` |
| `user_invitation` | Einladungs-Records inkl. Session-Verankerung | `id TEXT PK`, `username TEXT FK→user.name ON DELETE CASCADE`, `token TEXT UNIQUE`, `expiration_date TEXT`, `created_date TEXT DEFAULT datetime('now')`, `update_process TEXT NOT NULL`, `redeemed_at TEXT?`, `session_id TEXT? FK→session.id ON DELETE SET NULL`, `session_revoked_at TEXT?` |

Indizes:
- `idx_text_template_type` auf `template_type`
- `idx_text_template_deleted` auf `deleted`
- `idx_text_template_name` auf `name`
- `idx_user_invitation_token` auf `token`
- `idx_user_invitation_expiration` auf `expiration_date`
- `idx_user_invitation_session` auf `session_id`
- `idx_user_invitation_redeemed` auf `redeemed_at`
- `idx_user_invitation_session_revoked` auf `session_revoked_at`

### Migrations

Chronologisch:

- `20250816133730_add-message-template-table.sql` — Basistabelle
  `text_template` inkl. `template_type`, `template_text`, Audit-Spalten
  und Indizes. Trotz Datei-Name "message-template" heißt die Tabelle in
  der DB `text_template`.
- `20250817000000_add-name-to-text-template.sql` — `name TEXT` (nullable)
  + `idx_text_template_name`.
- `20251016154210_add-user-invitation-table.sql` — Basistabelle
  `user_invitation` mit `token UNIQUE`, FK auf `user(name)` und
  Ablauf-Index.
- `20251017044013_add-session-tracking-to-user-invitation.sql` —
  `redeemed_at`, `session_id` (FK auf `session(id) ON DELETE SET NULL`),
  passende Indizes; hebt die Semantik von "einmalig" auf "einmalig +
  weiß welche Session daraus wurde".
- `20251020000000_add-session-revoked-at-to-user-invitation.sql` —
  `session_revoked_at`, damit "Admin hat Session widerrufen" persistent
  vom bloßen "abgelaufen" unterscheidbar bleibt.
- `20260312000000_add-template-engine-to-text-template.sql` —
  `template_engine TEXT NOT NULL DEFAULT 'tera'`; Alt-Rows werden implizit
  auf `tera` gepinnt, neue Rows dürfen `minijinja` wählen.

### Beziehungen

```
user (name) ──┐
              │  ON DELETE CASCADE
              ▼
        user_invitation ──── session_id ──► session (id)
                                            ON DELETE SET NULL
```

Beim Löschen eines Users kippen dessen Invitations weg. Beim harten
Löschen einer Session wird `session_id` auf `NULL` gesetzt — der
Invitation-Record bleibt für die Historie erhalten und liefert dann
Status `Redeemed` (weil `redeemed_at` gesetzt bleibt), obwohl die
Session nicht mehr existiert. **[Zu prüfen]** ob dieser Zustand in der
UI absichtlich als "Redeemed" (statt "SessionRevoked" oder eigener
"Orphaned"-Status) gezeigt wird.

## 4. Service-API

### 4.1 `TextTemplateService`

Datei: `service/src/text_template.rs:83-129`.

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

**Auth-Gates:**
- `create`, `update`, `delete` → `HR_PRIVILEGE`
  (`service_impl/src/text_template.rs:78,112,149`).
- `get_all`, `get_by_id`, `get_by_template_type` → **kein** Gate im
  Service (siehe Regel-Fußnote in 2.1).

**TX-Verhalten:**
- Jede Methode öffnet die TX via
  `transaction_dao.use_transaction(tx).await?` und committet am Ende
  selbst. Kein Composite über mehrere Services — kein Rollback-Fan-out.

**Dependencies (Basic-Tier gem. `CLAUDE.md` "Service-Tier-Konventionen"):**
- `TextTemplateDao`
- `PermissionService`
- `TransactionDao`

### 4.2 `UserInvitationService`

Datei: `service/src/user_invitation.rs:36-93`.

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

**Auth-Gates:** siehe 2.2. `validate_and_consume_token` und
`mark_token_redeemed` sind ohne `auth`-Parameter definiert — sie werden
nur aus dem REST-Handler `authenticate_with_invitation` gerufen, der
selbst öffentlich (pre-auth) hängt.

**TX-Verhalten:**
- Alle schreibenden Methoden öffnen TX und committen.
- `list_invitations_for_user` und `find_invitation_by_session` öffnen
  die TX zwar (Discard-Binding `let _tx = …`), committen sie aber nicht
  — das ist ein bewusster Read-Only-Pfad, hinterlässt aber theoretisch
  eine offene TX bis zum Drop.  **[Zu prüfen]** ob der `TransactionDao`
  im Drop-Path auto-rollback macht (Konvention im Rest des Codebase:
  ja).

**Dependencies:** `UserInvitationDao`, `PermissionDao` (User-Autocreate),
`PermissionService`, `SessionService` (Session-Invalidierung),
`UuidService`, `TransactionDao` — damit **Business-Logic-Tier**
(orchestriert `SessionService` + `PermissionDao` als Cross-Entity-Op).

## 5. REST-Endpoints

### 5.1 Text-Templates — gemountet unter `/text-templates` (`rest/src/lib.rs:673`)

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/text-templates` | Alle Templates listen | — | `Vec<TextTemplateTO>` | 401, 500 |
| `GET` | `/text-templates/{id}` | Ein Template abrufen | — | `TextTemplateTO` | 401, 404, 500 |
| `GET` | `/text-templates/by-type/{template_type}` | Filter nach Typ | — | `Vec<TextTemplateTO>` | 401, 500 |
| `POST` | `/text-templates` | Anlegen | `CreateTextTemplateRequestTO` | `TextTemplateTO` (201) | 400, 401, 403 (HR), 500 |
| `PUT` | `/text-templates/{id}` | Aktualisieren | `UpdateTextTemplateRequestTO` | `TextTemplateTO` | 400, 401, 403 (HR), 404, 500 |
| `DELETE` | `/text-templates/{id}` | Löschen | — | 204 | 401, 403 (HR), 404, 500 |

DTOs siehe `rest-types::{TextTemplateTO, CreateTextTemplateRequestTO,
UpdateTextTemplateRequestTO, TemplateEngineTO}`
(`rest-types/src/lib.rs:1526-1601`).

### 5.2 User-Invitation — gemountet unter `/user-invitation` (`rest/src/lib.rs:676`) plus öffentliche Bootstrap-Route

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `POST` | `/user-invitation/invitation` | Invitation erzeugen | `GenerateInvitationRequest` | `InvitationResponse` | 400, 403 (admin), 500 |
| `GET` | `/user-invitation/invitation/user/{username}` | Invitations eines Users listen | — | `Vec<InvitationResponse>` | 403, 404, 500 |
| `DELETE` | `/user-invitation/invitation/{id}` | Invitation revoken (Row löschen) | — | 204 | 403, 404, 500 |
| `POST` | `/user-invitation/invitation/{id}/revoke-session` | Session zur Invitation invalidieren + `session_revoked_at` setzen | — | 204 | 403, 404 (keine Session assoziiert), 500 |
| `GET` | `/auth/invitation/{token}` | **Öffentlich, pre-auth** — Token einlösen, User ggf. anlegen, Session/Cookie setzen, Redirect `/` | — | 302 Redirect | 400 "Invalid or expired invitation token", 500 |

Die letzte Route ist **außerhalb** des Auth-Middleware-Stacks montiert
(`rest/src/lib.rs:752-756`) und existiert in zwei Feature-Gate-Varianten:
- `feature = "oidc"`: erzeugt echte Session via `SessionService::
  new_session_for_user`, setzt `app_session`-Cookie
  (`rest/src/user_invitation.rs:36-96`).
- `feature = "mock_auth"` (ohne `oidc`): Mock-Session-ID
  `mock-session-<uuid>` wird nur für den Redeem-Marker verwendet,
  Auth-Bypass läuft global (`rest/src/user_invitation.rs:99-128`).

DTOs siehe `rest-types::{GenerateInvitationRequest, InvitationResponse,
InvitationStatus}` (`rest-types/src/lib.rs:2286-2322`); Serde-
Repräsentation von `InvitationStatus` ist lowercase, `SessionRevoked`
wird als `"sessionrevoked"` serialisiert
(`rest-types/src/lib.rs:2296-2297`).

## 6. Frontend-Integration

### 6.1 Text-Templates

- **Pages:** `shifty-dioxus/src/page/text_template_management.rs`
  (HR-Verwaltungsansicht: Liste, Anlegen, Editieren, Löschen).
- **Services:** `shifty-dioxus/src/service/text_template.rs`
  (`TEXT_TEMPLATE_STORE` `GlobalSignal`, Coroutine-Actions
  `LoadTemplates`, `LoadTemplatesByType`, `SaveTemplate`,
  `UpdateTemplate`, `DeleteTemplate`).
- **State:** `shifty-dioxus/src/state/text_template.rs` — `TextTemplate`
  DTO + `TemplateEngine` Enum (Frontend-Spiegelbild).
- **Loader:** `shifty-dioxus/src/loader.rs` (`load_text_templates`,
  `load_text_templates_by_type`, `save_text_template`,
  `update_text_template`).
- **i18n-Keys:** `TextTemplateManagement`, `TemplateType`, `TemplateText`,
  `TemplateName`, `AddNew`, `AddNewTemplate`, `EditTemplate`, `Save`,
  `Cancel`, `Edit`, `Delete`, `Actions`, `TemplateEngine`,
  `TemplateEngineTera`, `TemplateEngineMiniJinja`.
- **Fest verdrahtete Template-Typen im FE-Dropdown:** `billing-period`,
  `shiftplan-report` (`page/text_template_management.rs:169-177`) — für
  neue Typen wäre eine Erweiterung dort nötig.
- **Proxy:** `[[web.proxy]] backend = "http://localhost:3000/text-templates"`
  in `shifty-dioxus/Dioxus.toml`.

### 6.2 User-Invitation

- **Pages:** `shifty-dioxus/src/page/user_details.rs` — Panel mit
  Liste der Invitations pro User, "Link kopieren"-Button, Revoke- und
  Revoke-Session-Buttons je nach Status.
- **Services:** `UserManagementAction::RevokeInvitation`,
  `UserManagementAction::RevokeInvitationSession` (siehe
  `page/user_details.rs:258-274`).
- **DTO-Reuse:** Frontend nutzt `rest-types::InvitationResponse` und
  `InvitationStatus` direkt (v1.2-Phase-6-Migration in `rest-types`,
  `rest-types/src/lib.rs:2282-2334`).
- **Proxy:** `[[web.proxy]] backend =
  "http://localhost:3000/user-invitation"` in `Dioxus.toml`.
- Der Redeem-Pfad `/auth/invitation/{token}` läuft **nicht** über den
  Frontend-Router — er wird direkt vom Backend gehändelt und redirected
  danach auf `/`.

## 7. Randfälle

Für die zentrale Randfall-Referenz siehe
[`../domain/edge-cases.md`](../domain/edge-cases.md), Sektion
[6. Authentifizierung / Autorisierung](../domain/edge-cases.md#6-authentifizierung--autorisierung).

- **Doppelt einlöster Invitation-Link:** `validate_and_consume_token`
  wirft `EntityNotFoundGeneric("Invitation token has already been used")`
  sobald `session_id.is_some()`; im REST-Layer landet das als HTTP 400
  "Invalid or expired invitation token"
  (`rest/src/user_invitation.rs:90-95`). Die dahinter liegende Session
  bleibt gültig, bis sie separat via `revoke-session` invalidiert wird.
- **Abgelaufener Link:** identisch verpackt als HTTP 400. Nach Ablauf
  ist derselbe Token nicht mehr reaktivierbar; Admin muss neuen erzeugen.
- **Race "einlösen genau bei Ablauf":** `validate_and_consume_token`
  vergleicht `expiration_date < now` **vor** dem Session-Commit; da die
  Session-Erzeugung im REST-Handler und *nicht* in derselben Service-TX
  läuft (`rest/src/user_invitation.rs:46-70`), gibt es ein kleines
  Zeitfenster, in dem ein gerade-noch-gültiger Token nach Session-
  Erzeugung als "abgelaufen" wirken könnte — die Session ist dennoch
  gültig, nur der `mark_token_redeemed`-Aufruf scheitert nicht daran,
  weil er keinen Zeit-Check macht (`dao_impl_sqlite/src/user_invitation.rs
  :173-192`). Praktisch: der User ist eingeloggt, die Invitation zeigt
  aber `Redeemed`. **[Zu prüfen]** ob das dokumentiert ist.
- **Session-Revoke ohne assoziierte Session:** `revoke_session_for_invitation`
  gibt `EntityNotFoundGeneric("No session associated with this
  invitation")` (`service_impl/src/user_invitation.rs:275-279`) → REST
  404. UI zeigt den Button nur bei `Redeemed`
  (`page/user_details.rs:267-274`).
- **Revoke einer eingelösten Invitation via `DELETE
  /user-invitation/invitation/{id}`:** löscht die Row **inklusive**
  Redeem-Historie (`delete_by_id`); die daraus entstandene Session bleibt
  aktiv, verliert nur ihren Invitation-Anker. **[Zu prüfen]** ob das
  gewollt ist — konsistenter wäre ein Soft-Delete oder erzwungenes
  Session-Revoke davor.
- **Template mit fehlender Variable:** Der Fehler entsteht erst beim
  Rendern durch den Konsumenten (Billing-Period-Custom-Report). Der
  `TextTemplateService` selbst validiert Template-Syntax nicht — kaputte
  Templates lassen sich anlegen und speichern.
- **Template-Typ frei wählbar:** REST akzeptiert jeden String; das FE
  filtert nur nach den bekannten zwei Typen. Ein Template mit unbekanntem
  Typ ist vom `GET /text-templates/by-type/{type}`-Filter erreichbar,
  aber vom Standard-FE-Dropdown nicht anwählbar.
- **Cutover Legacy → MiniJinja:** Alt-Templates ohne `template_engine`-
  Spalte werden per DB-DEFAULT `tera` gesetzt (Migration 2026-03-12), was
  dem historischen Verhalten entspricht — keine Rückwärts-Kompatibilitäts-
  Bruch für vorhandene Custom-Reports.
- **Fehlender User beim Redeem:** Autocreate legt einen leeren User an
  (`UserEntity { name }`) ohne Rollen; alle Berechtigungen müssen danach
  vom Admin gesetzt werden.

## 8. Tests

- **Unit-Tests:** Weder für `text_template` noch für `user_invitation`
  existiert aktuell eine Test-Datei unter
  `service_impl/src/test/` (Verzeichnisstand siehe `mod.rs`-Liste). Beide
  Services werden ausschließlich manuell und via FE-Integration
  abgedeckt. **[Zu prüfen]** ob das eine bewusste Deprioritisierung ist —
  Auth-relevanter Code wie `validate_and_consume_token` würde mindestens
  von Property-Style-Tests (Race, Expiry, Double-Redeem) profitieren.
- **REST-Compile-Time-Coverage:** `rest/src/text_template.rs` und
  `rest/src/user_invitation.rs` sind über `TextTemplateApiDoc` bzw.
  `UserInvitationApiDoc` in den OpenAPI-Merge eingebunden
  (`rest/src/lib.rs:67,71,586`), d. h. Schema-Drift zwischen DTO und
  Handler wird zumindest vom `utoipa`-Macro erwischt.
- **Frontend-Test:** `text_template_management.rs:284-311` hat einen
  Guard-Test gegen Legacy-Tailwind-Klassen im Quelltext (kein Behavior-
  Test).
- **Bekannte Lücken:**
  - Kein Roundtrip-Test für Token-Einlösung inkl. `mark_token_redeemed`.
  - Kein Test, der `cleanup_expired_invitations` gegen bereits eingelöste
    Rows fährt (Datenverlust-Risiko, siehe 2.2).
  - Kein Test für die Interaktion `session ON DELETE SET NULL` →
    Status-Ableitung.
  - Keine Coverage für `MiniJinja`-Pfad (Engine-Enum am DAO-Rand).

## 9. Historie & Kontext

- **2025-08 (v1.2)** — Text-Template-Grundgerüst (`text_template`-Tabelle,
  `template_type`, `template_text`). Motivation: konfigurierbare
  Billing-Period-Custom-Reports ohne Deploy
  (siehe `docs/template-examples/README.md`).
- **2025-08-17 (v1.2)** — `name`-Spalte nachgezogen, damit HR die
  Templates in der Liste nach Namen und nicht nach UUID unterscheiden
  kann.
- **2025-10 (v1.2 Phase 6)** — User-Invitation-MVP: Basistabelle,
  einmaliger Token, öffentliche Redeem-Route mit dual-featured OIDC/
  Mock-Auth-Pfad. Kommentar in `rest/src/user_invitation.rs:19-22`
  verweist auf die DTO-Migration nach `rest-types`.
- **2025-10-17** — Session-Tracking (`redeemed_at`, `session_id`) — Ziel:
  von "einmalig einlösbar" nach "einmalig einlösbar und wir wissen,
  welche Session dabei rauskam".
- **2025-10-20** — `session_revoked_at` als eigener Status-Marker, damit
  Admin-getriebenes Session-Revoken sichtbar vom natürlichen Ablauf
  unterscheidbar bleibt.
- **2026-03-12 (aktueller Milestone)** — `template_engine` als zweiter
  Engine-Slot (`Tera` bleibt Default, `MiniJinja` neu). Kein Rückwärts-
  Kompat-Break, Alt-Rows fallen auf `tera` zurück.
- Für weiteren Kontext-Read: `docs/template-examples/` (Konsum-Kette),
  `.planning/phases/…` — **[Zu prüfen]** welche konkrete Phase die
  Invitation-Migration enthalten hat (Verzeichnis-Sichtprüfung fand
  keinen offensichtlichen `invitation`-Slug; vermutlich unter dem v1.2-
  Phase-6-Slug archiviert).

---

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.

---

**Fazit:** F10 kapselt zwei orthogonale, aber semantisch verwandte
Kommunikations-Stubs — Text-Templates als reines HR-CRUD auf einer
schlanken `text_template`-Tabelle (Engine-Auswahl seit 2026-03-12) und
User-Invitation als admin-gate-teter Einladungs-Flow mit einmaligem
Token, Session-Verankerung und getrenntem "hart löschen" vs. "Session
widerrufen"-Weg. Größte offene Baustellen sind die fehlenden Service-
Level-Unit-Tests für den Auth-nahen Redeem-Pfad und die aggressive
Cleanup-Semantik, die auch eingelöste Invitations mitlöscht.
