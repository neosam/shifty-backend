# Konfiguration

Shifty wird zur Laufzeit über Environment-Variablen und beim Build
über Feature-Flags konfiguriert.

## Environment-Variablen

Basis-Template: `env.example`. Für Dev: `cp env.example .env`.

**[Zu prüfen]** — vollständige Env-Var-Liste aus `service_impl/src/config.rs`
oder Startup-Code. Typisch sind:

- `DATABASE_URL` — SQLite-Pfad.
- `PORT` — Backend-Port (Default 3000).
- OIDC-Config im `oidc`-Feature — `OIDC_ISSUER`, `OIDC_CLIENT_ID`,
  `OIDC_CLIENT_SECRET`, `OIDC_REDIRECT_URI`.
- Logging-Level (`RUST_LOG`).

## Feature-Flags

Feature-Flags werden bei `cargo build --features "..."` aktiviert.

### Auth-Modi (exklusiv)

- **`mock_auth`** — Development. Mocked Admin-User bei jedem Request.
- **`oidc`** — Production. Externer OpenID Connect Provider.

### Logging-Modi

- **`local_logging`** — Menschen-lesbarer Text.
- **`json_logging`** — Strukturiertes JSON für Log-Aggregation.

## `config.rs`

Die zentrale Config-Struktur liegt in `service_impl/src/config.rs`.
Sie liest Env-Vars beim Startup und stellt sie den Services zur
Verfügung.

## OIDC-Setup

Für Prod-Deployment:

1. IdP registrieren (Client anlegen, Redirect-URI setzen).
2. `OIDC_*`-Env-Vars in `.env` bzw. Systemd-Environment eintragen.
3. Build mit `--features oidc`.
4. Beim ersten Login wird der User in Shifty angelegt (**[Zu prüfen]**
   ob explizite Invitation notwendig, siehe
   [F10](../features/F10-templates-communication.md)).

## Nextcloud / WebDAV

Falls PDF-Export nach Nextcloud aktiv:

- WebDAV-URL, Username, Token in Config.
- Details: [F11 Export](../features/F11-export.md).

## Scheduler-Konfiguration

Der Scheduler läuft aktuell mit fest verdrahteter Cron-Expression
`"0 * * * * *"` (siehe `service_impl/src/scheduler.rs:45`). Konfigurierbar
ist er derzeit nur über Code-Änderung.

## Verwandte Randfälle

- Toggle- und Feature-Flag-Verhalten →
  [`../domain/edge-cases.md#9-feature-toggles--stichtag-rollouts`](../domain/edge-cases.md#9-feature-toggles--stichtag-rollouts).
