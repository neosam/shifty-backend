# Feature: {{Name}}

> **Kurzform:** Ein Satz, was das Feature liefert und für wen.

**Cluster-ID:** F??
**Status:** produktiv / experimentell / deprecated
**Erstmalig eingeführt:** Milestone / Datum
**Zuständige Crates:** `service::…`, `service_impl::…`, `dao::…`, `rest::…`

---

## 1. Was ist das? (Fachlich)

Fließtext, verständlich für nicht-technische Stakeholder. Wer benutzt das
Feature im UI, welchen Zweck erfüllt es im Geschäftsprozess, was für ein
Datenobjekt entsteht dabei?

**Beispiel-Workflow aus User-Sicht:**

1. …
2. …
3. …

## 2. Fachliche Regeln

Alle Business-Regeln als Bullet-Liste. Jede Regel wird in Kapitel 4/5 in Code
gemappt.

- Regel A: …
- Regel B: …
- Invariante: …

## 3. Datenmodell

### Tabellen

| Tabelle | Zweck | Wichtige Spalten |
| --- | --- | --- |
| `foo` | … | `id`, `deleted`, … |

### Migrations

Chronologische Liste der Migrations, die das Feature aufgebaut haben:

- `2024xxxxxxxx_...sql` — Basistabelle
- `2025xxxxxxxx_...sql` — Erweiterung um Spalte X

### Beziehungen

Kurzer Text oder Ausschnitt aus dem ER-Diagramm.

## 4. Service-API

### Trait

`service::foo::FooService`

```rust
#[async_trait]
pub trait FooService {
    type Context: …;
    type Transaction: …;

    async fn create(&self, dto: FooCreate, ctx: Authentication<Self::Context>, tx: Option<Self::Transaction>) -> Result<Foo, ServiceError>;
    // ...
}
```

### Auth-Gates

Welche Permissions welche Methode aufrufen darf.

### TX-Verhalten

- Öffnet TX selbst wenn `tx=None`.
- Composite-Op X-Y-Z läuft atomar.
- Rollback-Verhalten wenn Y fehlschlägt.

### Dependencies

- DAO(s): `FooDao`, `PermissionDao`
- Andere Services: (nur wenn Business-Logic-Tier)

## 5. REST-Endpoints

| Methode | Pfad | Beschreibung | DTO In | DTO Out | Wichtige Fehler |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/foo` | Liste | — | `Vec<FooTO>` | 401 |
| `POST` | `/foo` | Anlegen | `FooCreateTO` | `FooTO` | 400, 403, 409 |

DTOs siehe `rest-types::foo`.

## 6. Frontend-Integration

- **Pages:** `shifty-dioxus/src/page/…`
- **Services:** `shifty-dioxus/src/service/…`
- **State:** `shifty-dioxus/src/state/…`
- **i18n-Keys:** …
- **Proxy:** `Dioxus.toml` — welche Pfade müssen gemappt sein?

## 7. Randfälle

Feature-spezifische Kanten. Für die zentrale Randfall-Referenz siehe
[`../domain/edge-cases.md`](../domain/edge-cases.md), Sektion "…".

- Randfall A: …
- Randfall B: …

## 8. Tests

- **Unit:** `service_impl/src/test/foo/*.rs` — welche Szenarien abgedeckt.
- **Integration:** `service_impl/src/test/…` — In-Mem-SQLite Roundtrip.
- **Bekannte Lücken:** …

## 9. Historie & Kontext

- Milestone-Verweis, warum dieses Feature so aussieht.
- Cutover-Historie (wenn abgelöst).
- Verweise auf `.planning/phases/…` für Kontext-Reads.

---

*Letzte Verifikation gegen Code:* siehe git blame dieser Datei.
