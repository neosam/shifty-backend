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

## Routen & Seiten

Die Routen-Definitionen liegen in `shifty-dioxus/src/router.rs`; zu jeder
Route gehört genau eine Datei unter `page/`.

| Route | Seite | Anmerkung |
| --- | --- | --- |
| `/` | `home.rs` | |
| `/shiftplan/`, `/shiftplan/:year/:week` | `shiftplan.rs` | |
| `/weekly_overview/` | `weekly_overview.rs` | |
| `/employees/`, `/employees/:employee_id/` | `employees.rs`, `employee_details.rs` | Master/Detail |
| `/my_employee_details/` | `my_employee_details.rs` | Self-Service |
| `/my-shifts/` | `my_shifts.rs` | |
| `/absences/`, `/absences/:employee_id/` | `absences.rs` | |
| `/billing-periods/`, `/billing_period/:billing_period_id/` | `billing_periods.rs`, `billing_period_details.rs` | |
| `/user_management/` | `user_management.rs` | seit v2.7 Phase 62 das eine Personen-Verzeichnis |
| `/person/:person_id/` | `person_detail.rs` | **die** Personen-Detailroute seit v2.7 Phase 63 |
| `/custom_extra_hours/` | `custom_extra_hours_management.rs` | |
| `/text_templates/` | `text_template_management.rs` | |
| `/settings/` | `settings.rs` | |

**Eine Detailroute für alle drei Ausprägungen (v2.7 Phase 63, D-63-04).** Die
beiden früheren, ausprägungs-spezifischen Detailrouten und ihre Seiten sind in
derselben Phase **ersatzlos** entfallen — eine verknüpfte Person, ein reines
Benutzerkonto und ein reiner Verkäufer landen alle auf
`/person/:person_id/`, das die jeweils zutreffenden Abschnitte rendert. Eine
Weiterleitung von den alten Routen gibt es bewusst **nicht**: sie würde zwei
URL-Formen am Leben halten, die keine Bedeutung mehr tragen, und die einzigen
Verweise darauf lagen in diesem Frontend.

**Die `person_id` in der URL kann wechseln**, wenn einem reinen
Benutzer-Eintrag eine Sales Person zugeschaltet wird [D-61-03]. Die Seite
navigiert deshalb auf die `person_id`, die das Backend *zurückgegeben* hat,
nie auf die gesendete. Siehe
[F01 §4.7](../features/F01-employee-management_de.md).

## Entscheidungslogik gehört in reine Funktionen

**[Konvention, etabliert in v2.7 Phase 63]** Was eine Seite *entscheidet* —
welche Abschnitte erscheinen, welcher Bestätigungstext gilt, ob eine Aktion
die letzte ihrer Art ist — gehört in reine Funktionen mit direkten
Unit-Tests, nicht in den `rsx!`-Rumpf. `page/person_detail.rs` führt drei
davon (`visible_sections`, `is_last_facet`, `delete_confirmation_copy`); der
SSR-Rendertest prüft danach ein *zweites* Mal, dass das Ergebnis auch im
Markup ankommt.

Der Grund ist die Umgebung, nicht der Geschmack: DOM-Rendering ist in diesem
Frontend-Stack headless nur eingeschränkt prüfbar (ein Beispiel dafür steht
unten beim Loader-Pattern). Eine reine Funktion ist vollständig prüfbar, ein
`rsx!`-Zweig nicht. Alles, was die reinen Funktionen nicht belegen können,
wird deshalb von vornherein als **manueller Durchgang** eingeplant statt am
Ende als Lücke entdeckt.

Ein reiner Helfer bleibt frei von Autorisierungs-Entscheidungen — er
entscheidet über *Anzeige*, nie über *Berechtigung*. Das Gate liegt im
Backend-Service; eine Frontend-Funktion, die auf Privilegien verzweigt, wäre
eine zweite Wahrheit, und die schwächere davon wäre die Lücke.

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

### Aufrufe der Personenverwaltung (v2.7, Phase 63)

`api.rs` hat acht Aufrufe für die Personen-Detailansicht bekommen, jeder mit
einem gleichnamigen Wrapper in `loader.rs`:

| `api.rs` | Endpoint |
| --- | --- |
| `get_person_detail` | `GET /person/{id}` |
| `get_person_by_sales_person` | `GET /person/by-sales-person/{id}` |
| `post_person_entry` | `POST /person` |
| `post_person_facet` | `POST /person/{id}/facet` |
| `delete_person_facet` | `DELETE /person/{id}/facet/{facet}` |
| `get_unlinked_sales_persons` | `GET /person/candidates/sales-persons` |
| `get_unlinked_users` | `GET /person/candidates/users` |
| `delete_sales_person` | `DELETE /sales-person/{id}` |

Der letzte ist **kein** neuer Endpoint. `DELETE /sales-person/{id}` existiert
im Backend seit Langem und ist HR-gegated; bis Phase 63 hatte das Frontend
nur keinen Aufrufer dafür [D-63-06]. Das Löschen passiert jetzt dort, wo der
Benutzer den Verkäufer sieht, statt als Nebenwirkung an anderer Stelle.

**Drei unterscheidbare Antworten — weil die Seite drei verschiedene Dinge
rendert.** Die Übersetzung liegt in `api.rs`, wo die Response noch offen und
ein 422-Body lesbar ist:

- **404 ist ein *Zustand*, kein Fehler.** `load_person_detail` bildet ihn auf
  `Ok(None)` ab; die Seite zeigt ihren Nicht-gefunden-Zustand, statt das
  zentrale Fehler-Overlay auszulösen. Der Store hält „nicht gefunden" und
  „noch nicht geladen" deshalb in getrennten Feldern.
- **409 ist das besetzte Ziel** eines Rennens — die Kandidatenliste hat es
  ausgeblendet, der Server lehnt es trotzdem ab [D-61-04]. Die Seite bietet
  ein Neuladen an statt eines Fehlers.
- **422 ist ein Validierungsfehler**, dessen Body das Feld benennt.

Eine Ausnahme aus Notwendigkeit: `save_user_for_sales_person` bekommt seine
Übersetzung im `loader.rs` (`classify_link_error`), weil das darunterliegende
`api::post_user_to_sales_person` ein unveränderter Phase-61-Pfad mit
`reqwest::Error`-Signatur und mehreren Bestandsaufrufern ist.

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

Betroffene Phases: 28 und 49 haben's beide vergessen; bis v2.7 steht der
Zähler der ausgelieferten Dev-Server-404s durch einen fehlenden Eintrag bei
drei.

**Die Reihenfolge zählt, wenn ein Präfix ein anderes enthält.** Der Proxy
matcht präfix-literal. `/person` (Phase 63) ist ein Präfix von
`/person-directory` (Phase 62), der generische Eintrag muss deshalb **nach**
dem spezifischeren stehen; davor würde er die Verzeichnis-Anfragen mit
abfangen.

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
