# Testing — Konventionen & Gates

## Zwei Test-Ebenen

- **Unit-Tests** — Mocken die Trait-Grenzen. Kein Datenbank-, kein
  HTTP-Zugriff. Test-Runtime in Millisekunden. Fokus: einzelne Business-Regel.
- **Integration-Tests** — In-Memory-SQLite, echte DAOs, echte Services.
  Testen ganze Aggregat-Flows: Booking anlegen → Log geschrieben →
  Report zeigt es korrekt.

## Unit-Tests mit `mockall`

Jeder Trait in `service/` und `dao/` hat ein `#[cfg_attr(test, automock)]`.
Das erzeugt `MockFooService` etc.

**Muster:**

```rust
#[tokio::test]
async fn create_booking_denies_without_role() {
    let mut mock_perm = MockPermissionService::new();
    mock_perm.expect_require_role()
        .returning(|_, _| Err(ServiceError::Forbidden));

    let service = BookingServiceImpl::new(Deps {
        permission_service: mock_perm,
        // ...
    });

    let result = service.create(dto, auth, None).await;
    assert!(matches!(result, Err(ServiceError::Forbidden)));
}
```

Test-Dateien liegen unter `service_impl/src/test/` — ein Modul pro
Domäne (`test/booking.rs`, `test/absence.rs`, …).

## Integration-Tests

**In-Memory-SQLite** wird über SQLx-Setup mit `sqlite::memory:` gestartet.
Migrations laufen beim Test-Start. Danach: echte Services über echte
DAOs.

**Helper-Traits** wie `NoneTypeExt` bauen `Authentication`-Kontexte für
Tests kompakt zusammen.

## Test-Coverage-Erwartung

- **Neue Business-Regel:** Immer ein Unit-Test, der die Regel isoliert
  prüft (auch die Deny-Fälle).
- **Neuer REST-Endpoint:** OpenAPI-Annotation + mindestens ein
  Integration-Test für Happy-Path + einen Fehlerfall.
- **Neue Migration mit Semantik-Änderung:** Regression-Test, der beweist,
  dass alte Daten nach Migration weiter funktionieren.
- **Re-Point-Op (Slot-Split, Booking-Migration):** Test gegen
  Doppelzählung im Report. **Verpflichtend**, siehe
  [`../domain/edge-cases.md#7-transaktionen--atomarität`](../domain/edge-cases.md#7-transaktionen--atomarität).
- **Snapshot-Erzeugung mit neuem `value_type`:** Bump von
  `CURRENT_SNAPSHOT_SCHEMA_VERSION` und Test, dass Alt-Snapshot noch
  gelesen wird.

## Test-Gates (was CI und `nix build` erzwingen)

Die Pipeline besteht aus mehreren Stufen. **Nur die letzte ist
verbindlich:**

| Stufe | Was läuft | Genug? |
| --- | --- | --- |
| `cargo build` | Kompiliert | **Nein** |
| `cargo test` | Führt Unit + Integration aus | **Nein** |
| `cargo clippy --workspace -- -D warnings` | Lint-Check | **Nein** allein |
| `SQLX_OFFLINE=true cargo test` | Test mit Offline-Cache | **Nein** allein |
| **`nix build`** | Alle Stufen + Reproducibility-Check | **Ja** |

**Wichtig:** `cargo test` alleine reicht **nicht**. `nix build` erzwingt
`cargo clippy -- --deny warnings`. Jedes Phase-Gate (auch autonome
Phase-Execution) MUSS `cargo clippy --workspace -- -D warnings`
mitfahren, sonst schlägt der finale Build fehl.

Siehe `.github/workflows/rust.yml` für die CI-Definition.

## sqlx-Offline-Cache

CI läuft mit `SQLX_OFFLINE=true`. SQLx greift dann auf den `.sqlx/`-Cache
zurück statt auf eine echte Datenbank.

**Regel:** Nach jeder neuen `query!`/`query_as!`-Verwendung muss
`cargo sqlx prepare --workspace` laufen und der `.sqlx/`-Cache
mitcommittet werden.

Wenn du das vergisst:

- Inkrementeller Build kann grün sein (Cache noch da).
- Clean-Build (CI) failt.
- `cargo test --doc` failt (nutzt anderes Target).
- Phase 33 hat das mit "wieso ist CI rot obwohl alles grün" gefunden.

## Toolchain-Split (Backend vs Frontend)

Der Backend-Workspace nutzt eine andere Rust-Toolchain und einen
anderen Clippy-Level als `shifty-dioxus/`:

- Backend: Strict, `clippy -D warnings`.
- Frontend: Aus dem Backend-CI-Clippy **ausgeschlossen** — enthält
  ~198 pre-existing Lints, die als Backlog geführt werden.
- Clippy im dioxus-Shell ist zusätzlich funktional kaputt (E0514) und
  muss aus dem Backend-Shell heraus laufen, wenn man ihn braucht.

**Konsequenz:** Neue Frontend-Lints driften unbemerkt. Wenn du im
Frontend arbeitest, führe Clippy manuell durch — es fährt kein Gate.

## Test-Isolation

- **Nicht parallelisieren, wenn DB-Fixtures geteilt.** SQLite-In-Memory-DBs
  sind pro Test isoliert; nur wenn ein Test-Fixture explizit geteilt wird
  (was in Shifty nicht der Fall ist), können Race-Condition-Tests entstehen.
- **Time-Sensitive-Tests:** Verwenden `MockClockService`, um "heute"
  deterministisch zu setzen.

## Was NICHT getestet ist

- **RBAC-Deny-Pfade in Dev.** Weil `mock_auth` immer Admin ist, laufen
  Deny-Wege in E2E-Dev nie durch. **Konvention:** Explizite Unit-Tests
  für "kein Admin, nur Rolle X" sind Pflicht.
- **UI-E2E.** Es gibt keine automatische Browser-Test-Suite. Frontend-Änderungen
  werden manuell verifiziert; für kritische Flows im Zweifel Browser-Automation
  einsetzen (siehe [`06-frontend.md`](./06-frontend.md)).
- **Load / Concurrency.** Kein systematischer Load-Test.
