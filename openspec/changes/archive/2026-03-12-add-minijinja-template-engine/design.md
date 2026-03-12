## Context

The text template system currently uses Tera as its sole rendering engine. Templates are stored in the `text_template` database table and rendered at request time in `BillingPeriodReportServiceImpl` and `BlockReportServiceImpl`. The rendering logic directly creates a `tera::Context` and calls `tera.render()`.

Template authors must hardcode UUIDs for employee lookups because Tera lacks dictionary literals. MiniJinja supports `{% set m = {"key": "val"} %}` and `m[variable]` access, which would make templates significantly simpler.

## Goals / Non-Goals

**Goals:**
- Add MiniJinja as a second template engine alongside Tera
- Allow per-template engine selection via a new `template_engine` field
- Default to Tera for full backward compatibility
- Keep the same template context data available to both engines

**Non-Goals:**
- Migrating existing templates to MiniJinja
- Removing Tera
- Adding custom Tera filters or functions
- Enriching the template context with additional data (e.g., sales person names) — that's a separate concern

## Decisions

### 1. Engine selection stored per template in the database

**Decision**: Add a `template_engine TEXT NOT NULL DEFAULT 'tera'` column to the `text_template` table. Valid values: `"tera"`, `"minijinja"`.

**Rationale**: Per-template selection allows gradual migration. Storing it in the database (not as a request parameter) means the engine choice is a property of the template itself, not of the caller.

**Alternative considered**: Feature flag to switch all templates at once — rejected because it's all-or-nothing and prevents gradual migration.

### 2. Template engine as enum per layer

**Decision**: Model `template_engine` as a dedicated enum in each architectural layer, following the project's existing pattern of layer-specific types:
- `TemplateEngineEntity` in `dao` (with `TryFrom<&str>` for DB conversion)
- `TemplateEngine` in `service` (with `From<TemplateEngineEntity>`)
- `TemplateEngineTO` in `rest-types` (with `From<TemplateEngine>`, Serialize/Deserialize, ToSchema)

Each enum has two variants: `Tera` and `MiniJinja`.

**Rationale**: Enums make illegal states unrepresentable — invalid engine values are caught at deserialization time (from DB or from REST request) rather than at render time. This eliminates the need for runtime string validation in the service layer. The `match` on the enum also ensures the compiler flags all dispatch points when a new engine is added in the future.

**Alternative considered**: A single shared `Arc<str>` with runtime validation in the service layer — rejected because it pushes error detection to runtime, requires explicit validation code, and doesn't leverage the type system for exhaustive matching.

### 3. Dispatch at render time in service_impl

**Decision**: The render logic in `billing_period_report.rs` and `block_report.rs` matches on `template.template_engine` and dispatches to the appropriate engine.

**Rationale**: Keeps the change localized to the two files that already do rendering. No need for a new abstraction or trait — a simple `match` is sufficient for two enum variants.

**Alternative considered**: A `TemplateRenderer` trait with Tera/MiniJinja implementations — rejected as over-engineering for two variants with identical context preparation.

### 3. Shared context as serde_json::Value

**Decision**: Build the template context as `serde_json::Value` (which the code already does via `json!()` macros), then convert to engine-specific context types at dispatch time.

**Rationale**: Both Tera (`Context::from_serialize`) and MiniJinja (`minijinja::context!` / `render_str` accepting Serialize) can consume serde-serializable data. The current code already builds JSON values, so this requires minimal change.

### 4. MiniJinja dependency only in service_impl

**Decision**: Add `minijinja` as a dependency only to `service_impl/Cargo.toml`, alongside the existing `tera` dependency.

**Rationale**: Rendering is a service implementation detail. No other crate needs to know about the engine.

## Risks / Trade-offs

- **Filter incompatibility** — Tera and MiniJinja have different filter syntax in some cases (e.g., `split(pat="-")` vs `split("-")`). Templates are not portable between engines without adjustment. → Mitigation: This is expected and acceptable. Each template declares its engine, and authors know which syntax to use.

- **Two dependencies for one concern** — Carrying both `tera` and `minijinja` increases compile time and binary size slightly. → Mitigation: Acceptable during the transition period. Tera can be removed once all templates are migrated.

- **Invalid engine value** — A typo in `template_engine` (e.g., `"minjinja"`) in the database would fail when reading the entity. → Mitigation: The `TryFrom<&str>` conversion on `TemplateEngineEntity` returns an error for unknown values. For REST input, serde deserialization of `TemplateEngineTO` rejects invalid values before the request reaches the service layer.
