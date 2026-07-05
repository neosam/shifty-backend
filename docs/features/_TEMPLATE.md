# Feature: {{Name}}

> **Short form:** One sentence stating what the feature delivers and for whom.

**Cluster ID:** F??
**Status:** production / experimental / deprecated
**First introduced:** milestone / date
**Responsible crates:** `service::…`, `service_impl::…`, `dao::…`, `rest::…`

---

## 1. What is it? (Business context)

Prose text, understandable for non-technical stakeholders. Who uses the
feature in the UI, which purpose does it serve in the business process, what
data object is produced?

**Example workflow from a user's perspective:**

1. …
2. …
3. …

## 2. Business rules

All business rules as a bullet list. Each rule is mapped to code in
chapters 4/5.

- Rule A: …
- Rule B: …
- Invariant: …

## 3. Data model

### Tables

| Table | Purpose | Important columns |
| --- | --- | --- |
| `foo` | … | `id`, `deleted`, … |

### Migrations

Chronological list of migrations that built up the feature:

- `2024xxxxxxxx_...sql` — base table
- `2025xxxxxxxx_...sql` — extension by column X

### Relationships

Short text or excerpt from the ER diagram.

## 4. Service API

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

### Auth gates

Which permissions may call which method.

### TX behavior

- Opens TX itself when `tx=None`.
- Composite op X-Y-Z runs atomically.
- Rollback behavior when Y fails.

### Dependencies

- DAO(s): `FooDao`, `PermissionDao`
- Other services: (only if Business-Logic-Tier)

## 5. REST endpoints

| Method | Path | Description | DTO In | DTO Out | Important errors |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/foo` | List | — | `Vec<FooTO>` | 401 |
| `POST` | `/foo` | Create | `FooCreateTO` | `FooTO` | 400, 403, 409 |

DTOs see `rest-types::foo`.

## 6. Frontend integration

- **Pages:** `shifty-dioxus/src/page/…`
- **Services:** `shifty-dioxus/src/service/…`
- **State:** `shifty-dioxus/src/state/…`
- **i18n keys:** …
- **Proxy:** `Dioxus.toml` — which paths must be mapped?

## 7. Edge cases

Feature-specific edges. For the central edge-case reference see
[`../domain/edge-cases.md`](../domain/edge-cases.md), section "…".

- Edge case A: …
- Edge case B: …

## 8. Tests

- **Unit:** `service_impl/src/test/foo/*.rs` — which scenarios are covered.
- **Integration:** `service_impl/src/test/…` — in-memory SQLite roundtrip.
- **Known gaps:** …

## 9. History & context

- Milestone reference, why this feature looks the way it does.
- Cutover history (if superseded).
- References to `.planning/phases/…` for context reads.

---

*Last verified against code:* see git blame of this file.
