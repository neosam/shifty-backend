# Deployment

## Prod-Deploy via `shifty-nix`

Shifty wird als NixOS-Modul deployed. Das Nachbar-Repo `shifty-nix`
enthält:

- Die Systemd-Service-Definition für das Backend.
- Die statische Ausliefung des Dioxus-WASM-Bundles.
- Die Nix-Pin auf die zu deployende Backend-Commit-Version.

**Deploy selbst ist manuell.** `/release-version` bereitet den Release
vor (Tag, Nix-Pin), das eigentliche `nixos-rebuild switch` auf dem
Server macht der User selbst.

## Release-Fluss (`/release-version`)

Der Skill:

1. Leitet die SemVer-Version aus dem GSD-Milestone
   (`.planning/STATE.md`) und existierenden Git-Tags ab.
2. Fragt Bestätigung.
3. Generiert Release Notes aus den Commits seit dem letzten Tag.
4. Ruft `./update_versions.sh` mit den Notes als Annotated-Tag-Message.
5. Updated und taggt den Deploy-Pin in `../shifty-nix`.

**Was NICHT passiert:** Push auf den Server. Das machst du.

## Versionierung — SemVer ab v2.0

Historie:

- **v1.x — v2.x:** SemVer. MAJOR.MINOR aus GSD-Milestone, PATCH aus
  Git-Tags.
- **CalVer `v2026.x`:** Eingefrorene Insel — nicht umbenennen, weil
  `shifty-nix` diese Tag-Namen pinnt.
- **GSD-Auto-Tag:** Deaktiviert (`git.create_tag = false`). Releases
  ausschließlich via `/release-version`.

## Feature-Flags im Build

- **`mock_auth`:** Development. Fest verdrahteter Admin-User.
- **`oidc`:** Production. OpenID Connect gegen externen IdP.

Genau eine der beiden Auth-Flags ist beim Build aktiv.

- **`local_logging`:** Text-Logging, für Dev/Local.
- **`json_logging`:** Strukturiertes JSON-Logging, für Prod.

## Prod-Startup-Prüfliste

Wenn ein neues Deployment ausgerollt wird:

1. **Migrations gelaufen?** — `sqlx migrate run` gegen die Prod-DB.
2. **`.sqlx/`-Cache passend?** — Wenn CI grün war, ja.
3. **OIDC-Config aktuell?** — Client-ID / Secret / Redirect-URI.
4. **Feature-Flags korrekt?** — `oidc`, `json_logging`.
5. **Backup vorher gemacht?** — SQLite-DB-Datei sichern.
6. **Nach Deploy:** Login-Flow testen, ein Report öffnen.

## Rollback

Bei kaputtem Deploy:

1. `../shifty-nix`-Pin zurück auf den letzten funktionierenden Commit.
2. `nixos-rebuild switch` auf dem Server.
3. Datenbank-Rollback: **Nur wenn Migration rückwärts-kompatibel.**
   Sonst DB aus Backup wiederherstellen.

**Wichtig:** SQLx-Migrations haben in Shifty kein down-Skript. Rückwärts
funktioniert nur bei rein additiven Änderungen. Bei Schema-Änderungen,
die alte Software nicht mehr lesen kann: aus Backup.

## Monitoring

**[Zu prüfen]** — was ist in Prod eingerichtet? Log-Aggregation,
Health-Check-Endpoint, Alerting?

## Verwandte Randfälle

Siehe [`../domain/edge-cases.md#10-migrations--sqlx-offline-cache`](../domain/edge-cases.md#10-migrations--sqlx-offline-cache)
für Deploy-relevante Kanten.
