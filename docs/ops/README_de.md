# Operations — Betrieb, Deployment, Konfiguration

Diese Sektion richtet sich an alle, die Shifty **betreiben**, deployen,
migrieren oder konfigurieren.

## Kapitel

- **[deployment.md](./deployment.md)** — NixOS-Deployment über
  `shifty-nix`, Release-Prozess (`/release-version`), Version-Pinning.
- **[database.md](./database.md)** — SQLx-Migrations, `sqlx migrate run` vs
  `sqlx database reset`, `.sqlx/`-Cache für Offline-Builds, Backup-Strategie.
- **[configuration.md](./configuration.md)** — Environment-Variablen,
  Feature-Flags (`mock_auth`, `oidc`, Logging-Modi), OIDC-Setup.

## Kurzüberblick

### Zwei Deploy-Formen

1. **Development.** Backend läuft lokal auf Port 3000, Frontend via
   `dx serve` auf Port 8080. Frontend proxied REST-Requests laut
   `shifty-dioxus/Dioxus.toml`.
2. **Production.** NixOS-Modul aus dem Nachbar-Repo `shifty-nix` deployed
   das Backend als Systemd-Service. Das Frontend wird als statisches
   WASM-Bundle ausgeliefert.

### Release-Fluss

Der `/release-version`-Skill:

1. Leitet die neue Version aus dem GSD-Milestone (`.planning/STATE.md`) und
   den existierenden Git-Tags nach SemVer ab.
2. Fragt Bestätigung, erzeugt Release Notes aus den Commits seit dem letzten
   Tag.
3. Ruft `./update_versions.sh` mit den Notes als Annotated-Tag-Message auf.
4. Aktualisiert die Deploy-Pin in `../shifty-nix` und taggt sie.

Deploy selbst ist **manuell** — Push auf den Server erfolgt außerhalb dieser
Automatik.

### Kritische Ops-Randfälle

Siehe [`../domain/edge-cases.md`](../domain/edge-cases.md), speziell die
Sektionen "Migrations & Schema" und "Snapshot-Versioning".

Kurzfassung der schmerzhaftesten:

- **`sqlx database reset` ist destruktiv.** Nur additive `sqlx migrate run`
  benutzen, außer wenn explizit die Dev-DB wegsoll.
- **Nach neuer `query!/query_as!`-Query** muss `cargo sqlx prepare --workspace`
  laufen und der `.sqlx/`-Cache mitcommittet werden — sonst failt CI.
- **`nix build`** erzwingt `cargo clippy -- --deny warnings`. `cargo test`
  alleine ist nicht ausreichend.
