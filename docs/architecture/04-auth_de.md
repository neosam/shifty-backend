# Authentifizierung & Autorisierung

## Zwei Auth-Modi

Shifty unterstützt zwei Auth-Modi, gesteuert per Feature-Flag:

- **`mock_auth`** (Dev) — Ein fest verdrahteter Admin-User wird bei
  jedem Request injiziert. Kein Login, keine Rollen-Prüfung im
  Session-Layer.
- **`oidc`** (Prod) — OpenID Connect Login gegen einen externen IdP.
  Session-Token wird geprüft, User + Rollen aus dem IdP übernommen.

Der Feature-Flag entscheidet zur Compile-Zeit, welcher Session-Layer
in `shifty_bin` verdrahtet wird.

## `Authentication<Context>`

Der Auth-Kontext, den Services entgegennehmen, ist generisch:

```rust
pub enum Authentication<C> {
    Authenticated(C),   // Konkreter User-Context (User-ID, Rollen)
    Full,               // All-Rights-Bypass (nur intern)
}
```

Der Context `C` ist ein Trait, den Adapter (Session-Layer für Prod,
Mock-Session für Dev, Test-Harness) implementieren.

## Der `Full`-Bypass

`Authentication::Full` ist der **all-rights bypass** für interne
Aggregat-Aufrufe. Verhalten in `service_impl/src/permission.rs:28,41,63,80,90`:

```rust
match auth {
    Authentication::Full => Ok(()),   // Alle Checks sofort erfolgreich
    Authentication::Authenticated(ctx) => { /* echte Prüfung */ }
}
```

### Wann `Full` verwenden

**Nur** für interne Aggregat-Reads durch Business-Logic-Services:

- `ReportingService` liest `SalesPersonService::get`, `BookingService::list`,
  `ExtraHoursService::list`, `AbsenceService::list`, `CarryoverService::get`
  mit `Full`, weil der User-Context bereits im REST-Handler geprüft wurde
  und die internen Reads nicht jeder für sich nochmal alles wissen sollen.
- Beispiel-Verweise:
  - `service_impl/src/scheduler.rs:60,68` —
    `update_carryover_all_employees(year-1, Full)` (Batch-Job hat kein
    User-Context).
  - `service_impl/src/extra_hours.rs:51-54` —
    `custom_extra_hours.get_by_id(key, Full)` (interne Definitions-Auflösung).
  - `service_impl/src/sales_person_shiftplan.rs:65,92` — interne
    Sales-Person-Reads.

### Wann `Full` NIEMALS verwenden

- **In REST-Handlern.** Der User-Context aus der Session muss durchgereicht
  werden. `Full` in einem REST-Handler wäre ein katastrophaler
  Auth-Bypass.
- **Beispiel Gegenmuster:** `service_impl/src/pdf_shiftplan.rs:21`
  dokumentiert explizit: "(D-49-07); niemals wird intern auf
  `Authentication::Full` hochgehebelt." Der Export-Chain reicht den
  User-Auth durch bis zur Datenschicht.

### Der Full-Bypass für Toggle-Reads (Phase 51)

**[Verifiziert per Memory + Test-Datei]** Der `ToggleService` hatte einen
Guard, der Full-Reads mit einem User-ID-Guard verhinderte
(`service_impl/src/test/toggle.rs:547-556`). Das war ein Bug, weil
Reporting und BookingInformation den Toggle mit `Full` lesen. Die
Gap-Closure in Phase 51 stellt sicher, dass Full-Reads für Toggles den
Guard umgehen.

Konsequenz für neue Services: **Read-Ops müssen `Full` akzeptieren**,
sonst brechen interne Aggregate.

## Rollen & Privilegien

Die Rollen-Definition entwickelt sich über mehrere Migrations:

- `20240426150045_user-roles.sql` — Basis-Rollen.
- `20240614075633_shiftplanner-role.sql` — Shiftplanner-Rolle
  hinzugefügt.
- `20241118165756_add-role-shiftplan-edit.sql` — Feinere Trennung
  Read/Edit für Shiftplan.

**[Zu prüfen]** — die exakte Enumeration aller Rollen und welche
Privilegien sie tragen. Siehe
[`../features/F12-auth-session.md`](../features/F12-auth-session.md).

## Selbst-Zugriffs-Gate auf Sales-Person-Daten

Die meisten mitarbeiterbezogenen Lese- und Schreibpfade sind als **HR ∨ self**
abgesichert: entweder der Aufrufer trägt `HR_PRIVILEGE`, oder der Aufrufer
*ist* die Sales Person, um die es geht. Die „self"-Hälfte ist
`SalesPersonService::verify_user_is_sales_person(sales_person_id, ctx, tx)`
(`service_impl/src/sales_person.rs`). Die Aufrufer verbinden beide Hälften mit
`tokio::join!` und `hr.or(sp)?` — ein HR-Benutzer umgeht das Gate also über
sein Privileg, und nur ein Nicht-HR-Aufrufer hängt tatsächlich daran
(z. B. `service_impl/src/absence.rs:109-119`).

`verify_user_is_sales_person` löst den Login der Sales Person über
`SalesPersonDao::get_assigned_user` auf und vergleicht ihn mit
`PermissionService::current_user_id`. Ist eine der beiden Seiten `None`,
liefert der Aufruf `ServiceError::Forbidden` — fail-closed.

**Der Abgleich selbst fällt im SQL, über `person_id`** (v2.7, Phase 61
[D-61-08]). Seit die 1:1-Verknüpfungstabelle von
`migrations/sqlite/20260917000000_drop-sales-person-user.sql` aufgelöst wurde,
lösen alle vier lesenden Verknüpfungs-Pfade (`find_by_user`,
`find_sales_person_by_user_id`, `get_assigned_user`,
`get_all_user_assignments` in `dao_impl_sqlite/src/sales_person.rs`) über
einen Join auf:

```sql
SELECT u.name AS user_id
FROM sales_person sp
JOIN user u ON u.person_id = sp.person_id
WHERE sp.id = ?
```

**Konvention — der Identitätsvergleich gehört ins SQL, nicht nach Rust.** Das
ist eine dauerhafte Regel, keine Phasen-Notiz. Sowohl `sales_person.person_id`
als auch `user.person_id` sind schema-seitig nullable, ein Rust-seitiger
Vergleich wäre also ein `Option<Uuid>`-Vergleich — und `None == None` ergibt
`true`. Zwei *unverknüpfte* Zeilen würden damit aufeinander matchen und das
Gate ginge **fail-open**: ein Fremder läse fremde Buchungs- und Lohndaten.
SQLs Drei-Werte-Logik lässt `NULL = NULL` nie wahr werden, der Join liefert
dann schlicht keine Zeile und das Gate schließt. Jede künftige Überarbeitung
dieser Pfade muss die Entscheidung in der Query belassen.

Regressionstests: `service_impl/src/test/person_self_gate.rs` fährt gegen eine
echte In-Memory-SQLite-Datenbank (bewusst **kein** DAO-Mock — ein Mock führt
kein SQL aus und könnte diese Eigenschaft gar nicht prüfen). Die drei
Kernfälle sind `self_may_access_own_sales_person` (self darf),
`foreign_sales_person_rejected` (fremd darf nicht) und
`unlinked_user_rejected` (unverknüpft darf nicht); den oben beschriebenen
fail-open-Fall sichert `null_person_id_does_not_match_null_person_id`.

Zu beachten: **keiner** der vier Pfade filtert `deleted IS NULL`, und keiner
darf damit anfangen — eine soft-gelöschte Sales Person behält den
Selbstzugriff auf ihre eigene Historie
(`soft_geloeschte_sales_person_behaelt_selbstzugriff`).

## Benutzer-Schreibpfade auf der geteilten Transaktion (v2.7, Phase 63)

Das Anlegen und Löschen eines Login-Benutzers ist mit Phase 63
**komponierbar** geworden. Beide Schreibpfade sind vom Pool auf die vom
Aufrufer übergebene Transaktion gewandert:

```rust
// dao/src/permission.rs
type Transaction: crate::Transaction;

async fn create_user(&self, user: &UserEntity, person_id: Option<Uuid>,
                     process: &str, tx: Self::Transaction) -> Result<(), DaoError>;
async fn delete_user(&self, username: &str, tx: Self::Transaction) -> Result<(), DaoError>;
```

Zwei Dinge haben sich geändert, und eines bewusst nicht.

**1. Die Transaktion ist jetzt Parameter, kein internes Detail.** Vorher
öffnete `create_user` eine methoden-interne `pool.begin()`-Transaktion, und
`delete_user` schrieb direkt auf den Pool — beides für den aufrufenden Service
unsichtbar und von außen nicht rückrollbar. Die Personenverwaltung braucht den
Benutzer-Schreibvorgang in *derselben* Transaktion wie den
Sales-Person-Schreibvorgang (damit „danach existieren genau die gewählten
Objekte" gelten kann) und in derselben Transaktion wie das Aufräumen der
Person, die gerade ihren letzten Träger verloren hat. Über Commit und Rollback
entscheidet jetzt der Aufrufer. Siehe
[`05-transactions_de.md`](./05-transactions_de.md).

**2. `create_user` kann eine bestehende Person-Identität übernehmen.**
`person_id: None` behält das alte Verhalten — es wird eine frische
`person`-Zeile miterzeugt [D-61-01]. `person_id: Some(id)` hängt den Benutzer
an die **bestehende** Person und legt keine zweite Zeile an. Genau das
ermöglicht es, dass ein Dialog Benutzer *und* Sales Person auf derselben
Identität anlegt.

**3. Was sich nicht geändert hat: die Oberfläche von `PermissionService`.**
Das Service-Trait führt **keinen** `Transaction`-Assoziationstyp und
**keinen** `tx`-Parameter — und darf auch keinen bekommen. Rund zwanzig
Services führen `PermissionService` als Dependency; ein Assoziationstyp dort
wäre ein workspace-weiter Ripple ohne Gegenwert, und jeder dieser Services
müsste einen Transaktionstyp benennen, den er nicht benutzt. Der
Assoziationstyp sitzt deshalb eine Ebene tiefer, auf `dao::PermissionDao`.
`PermissionServiceImpl` hat eine dritte Dependency bekommen
(`TransactionDao`), um sich selbst eine Transaktion zu holen und sie am Ende
von `create_user` / `delete_user` zu committen; von außen verhält sich der
Service exakt wie vorher, und das Admin-Gate ist weiterhin seine erste
Anweisung.

**Dokumentierte Einschränkung — einen Benutzer mit offener Sitzung löschen.**
`session.user_id` ist die einzige auf `user(name)` zeigende
Fremdschlüssel-Beziehung **ohne** Kaskade (Migration
`20241116224840_add-session.sql`). Rollen und offene Einladungen nimmt ihre
jeweilige Kaskade mit; eine offene Sitzung nicht. Einen solchen Benutzer zu
löschen läuft deshalb in einen Fremdschlüssel-Fehler und kommt am Endpoint als
generischer **500** heraus statt als sprechende Ablehnung. Bekannt,
vorbestehend und innerhalb von Phase 63 bewusst nicht gefixt — ein Fix ist
eine Schema-Änderung und gehört in einen eigenen Vorgang.

## Session-Management

- **Session-Tabelle:** Migration `20241116224840_add-session.sql`,
  Constraint-Verschärfung `20241118180147_make-session-id-not-null.sql`.
- **Session pro Login:** Beim Login wird eine Session-Zeile angelegt,
  Session-ID im Cookie. Beim Logout / Ablauf wird sie invalidiert.
- **[Zu prüfen]** Genaues Refresh-Verhalten bei Token-Expiry.

## User Invitation

Für neue Nutzer gibt es einen Invitation-Flow:

- Migration `20251016154210_add-user-invitation-table.sql`.
- Erweiterungen: `20251017044013_add-session-tracking-to-user-invitation.sql`,
  `20251020000000_add-session-revoked-at-to-user-invitation.sql`.

Details in [`../features/F10-templates-communication.md`](../features/F10-templates-communication.md).

## Verwandte Randfälle

- Full-Bypass-Missbrauch → [`../domain/edge-cases.md#6-authentifizierung--autorisierung`](../domain/edge-cases.md#6-authentifizierung--autorisierung)
- Token-Expiry, Rollenänderung mid-session → dieselbe Sektion.
- Invitation-Link mehrfach eingelöst → dieselbe Sektion.
