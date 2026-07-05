# Shifty — Technische Dokumentation

Willkommen in der technischen Referenz von **Shifty**, dem Schichtplanungs- und
HR-Management-System für kleine bis mittlere Teams.

Diese Dokumentation ist nach Zielgruppen organisiert. Wähle deinen Einstieg:

## Einstiegspunkte

| Wenn du … | … dann starte hier |
| --- | --- |
| **neu im Projekt** bist und Code schreiben willst | [`onboarding/`](./onboarding/README.md) |
| das System **betreiben / deployen** willst | [`ops/`](./ops/README.md) |
| einen **eigenen Client** (Mobile, CLI, …) gegen das Backend baust | [`api/`](./api/README.md) |
| die **fachlichen Regeln** verstehen willst (Balance, Absence, Billing) | [`domain/`](./domain/README.md) |
| **wie das System intern aufgebaut ist** wissen willst | [`architecture/`](./architecture/README.md) |
| ein einzelnes **Feature** komplett verstehen willst | [`features/`](./features/README.md) |

## Struktur dieser Dokumentation

```
docs/
├── onboarding/     # Dev-Onboarding (Setup, erste Woche, Konventionen)
├── ops/            # Betrieb (Nix-Deploy, Migrations, Konfiguration, Release)
├── api/            # REST-API-Referenz für Zweit-Client-Entwickler
├── domain/         # Fachliche Referenz (Modell, Balance, Absence, Billing)
│   └── edge-cases.md  ← Zentrale Randfall-Referenz
├── architecture/   # Technische Referenz (Layer, Services, DB, Auth, Tests)
│   └── diagrams/   # Mermaid-Diagramme (Service-Graph, ER, Sequence)
└── features/       # Ein Dokument pro Feature-Domäne
```

## Leitprinzipien

Shifty folgt ein paar Grundregeln, die man kennen sollte, bevor man Code schreibt:

1. **Fat Backend, Thin Client.** Sämtliche Business-Logik liegt im Backend. Der
   Frontend-Client ist ein reiner View-Layer. Zweitclients (Mobile, Skripte)
   sollen keine Domain-Regel duplizieren müssen.
2. **Alles ist ein Trait.** Services und DAOs sind Trait-Definitionen mit
   austauschbaren Implementierungen. Tests mocken auf Trait-Ebene.
3. **`Option<Transaction>` überall.** Jede Service-Methode kann in einer
   existierenden Transaktion mitfahren oder eine eigene öffnen. Composite-Ops
   laufen atomar.
4. **Soft-Delete statt Hard-Delete.** Alle Reader-Queries filtern
   `WHERE deleted IS NULL`.
5. **Snapshot-basiertes Reporting.** Billing-Perioden werden mit einer
   `snapshot_schema_version` eingefroren. Nachträgliche Regeländerungen
   invalidieren alte Snapshots nicht.
6. **Clippy ist ein hartes Gate.** `nix build` erzwingt
   `cargo clippy -- --deny warnings`. `cargo test` alleine reicht nicht.

## Aktualität dieser Dokumentation

Diese Dokumentation wurde mit dem `gsd-docs-update`-Verfahren erzeugt und wird
gegen die Codebase verifiziert. Wenn du eine Diskrepanz findest, ist der Code
maßgeblich — bitte melde das Dokument als "veraltet" und korrigiere es im
selben PR wie die Code-Änderung.

Zuletzt vollständig aktualisiert: siehe git log dieses Verzeichnisses.
