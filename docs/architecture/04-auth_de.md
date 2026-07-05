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
