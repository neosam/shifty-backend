# Frontend-Architektur

Das Frontend liegt in `shifty-backend/shifty-dioxus/` und ist ein
eigener Cargo-Workspace, der zu WebAssembly kompiliert wird.

## Tech-Stack

- **Framework:** [Dioxus](https://dioxuslabs.com/) 0.6.x.
- **Sprache:** Rust → WASM.
- **CSS:** Tailwind, kompiliert nach `assets/tailwind.css`.
- **Dev-Server:** `dx serve` mit Hot-Reload.
- **Deploy:** Statisches Bundle in `dist/`, ausgeliefert vom Backend
  oder einem Reverse-Proxy.

## Verzeichnisstruktur

```
shifty-dioxus/src/
├── main.rs            # Entry Point
├── app.rs             # Root-Component + Router-Setup
├── router.rs          # Route-Definitions
├── auth.rs            # Login/Logout/Session-State
├── api.rs             # HTTP-Client gegen Backend
├── loader.rs          # Async-Data-Loading-Muster
├── base_types.rs      # Kern-Typen (z.B. Wrapper um rest-types)
├── error.rs           # Frontend-Fehler-Behandlung
├── js.rs              # JS-Interop
├── page/              # Ein Rust-File pro Route
├── component/         # Wiederverwendbare UI-Bausteine
├── service/           # Frontend-Services (Wrapper um API)
├── state/             # Signal-basierter globaler State
├── i18n/              # Übersetzungen (En, De, Cs)
└── tests/
```

## Fat Backend, Thin Client

**Kernprinzip:** Sämtliche Business-Logik lebt im Backend. Das Frontend:

- rendert Ergebnisse,
- validiert Input **nur** für UX-Feedback (nicht als Autorität),
- sendet Requests, hört Responses,
- zeigt Fehler an.

Es rechnet **keine** Balance, keine Konflikte, keine Snapshot-Werte.

Warum: Ein Zweit-Client (Mobile-App, CLI, Automation-Skript) soll
niemals eine Domain-Regel duplizieren müssen. Alles was zählt, kommt
vom Backend über REST.

## API-Kopplung: `rest-types`

DTOs werden aus dem Crate `rest-types` konsumiert — dieselbe
Definitions-Quelle wie das Backend. Feld-Umbenennungen im Backend
zwingen den Frontend-Build zum Fehler (was gewollt ist).

## Dev-Proxy — `Dioxus.toml`

Im Dev-Modus läuft `dx serve` auf Port 8080, während das Backend auf
Port 3000 hört. Die HTTP-Proxy-Konfiguration in `shifty-dioxus/Dioxus.toml`
leitet API-Pfade an das Backend weiter:

```toml
[[web.proxy]]
backend = "http://localhost:3000/api/..."
```

**Randfall (2× real passiert):** Wenn du einen neuen Backend-Endpoint
anlegst, den das Frontend ansprechen soll, MUSST du auch einen
`[[web.proxy]]`-Eintrag hinzufügen. Ohne den bekommst du im Dev-Modus
einen 404 vom `dx serve`, obwohl das Backend läuft. Prod funktioniert
trotzdem (statisches Bundle geht durch den Reverse-Proxy).

Betroffene Phases: 28 und 49 haben's beide vergessen.

## dx-CLI-Version-Pin

**Wichtig:** shifty-dioxus verlangt dx-CLI **0.6.x** (das Crate `dioxus`
ist auf 0.6.3 gepinnt). Wenn nixpkgs auf 0.7.x rollt, startet die App
nicht und das Design ist gestrippt. Der Pin ist in `flake.nix`
festgezurrt.

Wenn du die App lokal startest und weißes Layout siehst:

- `dx --version` prüfen.
- Sicherstellen, dass `Dioxus.toml` `style = "/assets/tailwind.css"`
  setzt.
- Tailwind-Watcher läuft und schreibt tatsächlich in `assets/tailwind.css`.

## State-Management

Signal-basiert (`dioxus::signals`). Globaler State in `state/`:
Auth-State, aktueller User, ausgewählte Sales Person, aktuelle Kalender-Woche.

## Loader-Pattern

Async-Data-Loading nutzt einen Loader-Trait, der `Loading` / `Loaded` /
`Error` liefert. Pages zeigen entsprechenden UI-State an.

**Randfall:** Bei Programmatik-Datepickern (`<input type=date>`)
triggern JS-Injects die Dioxus-Signale nicht — Submit-Buttons bleiben
inaktiv. Für Automation-Tests lieber Werte-Verifikation per cargo-Test
machen, nicht per Browser-Automation. Siehe Memory
`reference_dioxus_browser_test_date_inputs`.

## i18n

Drei Sprachen: **En, De, Cs**. Jeder neue Text braucht Einträge in
allen drei Locales.

Detail: [`08-i18n.md`](./08-i18n.md).

## Testing im Frontend

- WASM-Build-Gate: `cargo build --target wasm32-unknown-unknown` im
  `shifty-dioxus/`-Verzeichnis.
- **Clippy:** Aus dem Backend-CI-Clippy ausgeschlossen (198 pre-existing
  Lints). Muss manuell laufen, und aus dem Backend-Shell — dioxus-Shell
  ist mit E0514 kaputt.

## Backend-Roundtrip verifizieren

**Konvention:** Frontend-Phasen mit "Backend existiert bereits"-Annahme
MÜSSEN im Browser e2e verifiziert werden — Create-Pfad ist nicht
Edit-Pfad. Phase 23 hat gelernt, dass `modify_slot` `max_paid_employees`
fallen ließ, obwohl `create_slot` es korrekt setzte. Nur Browser-Test
hat das gefunden.

## Verwandte Randfälle

Siehe [`../domain/edge-cases.md#11-frontend-backend-kopplung`](../domain/edge-cases.md#11-frontend-backend-kopplung).
