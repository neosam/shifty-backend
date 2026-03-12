## Context

The backend added a `template_engine` field (enum: `tera` | `minijinja`) to text templates. This field is present in all REST responses and expected in create/update requests. The frontend has its own copy of `rest-types` and needs to mirror this change across REST types, state, and UI.

## Goals / Non-Goals

**Goals:**
- Frontend can deserialize `template_engine` from backend responses
- Frontend sends `template_engine` in create/update requests
- Users can select the template engine when creating or editing a template

**Non-Goals:**
- Template preview or syntax highlighting per engine
- Validation of template syntax in the frontend
- Showing the engine in the template list/overview table

## Decisions

### 1. Mirror the backend enum shape exactly

The frontend `TemplateEngineTO` uses the same `#[serde(rename = "tera")]` / `#[serde(rename = "minijinja")]` as the backend. This keeps JSON serialization identical without coupling the crates.

### 2. Separate frontend `TemplateEngine` enum in state

Following the existing pattern (e.g., `TextTemplateTO` → `TextTemplate`), the state layer gets its own `TemplateEngine` enum with `From` conversions. This keeps the state layer independent of REST types.

### 3. Default to Tera in UI

When creating a new template, the engine dropdown defaults to `Tera`, matching the backend default behavior.

### 4. Test-Driven Development approach

Implementation follows TDD: write tests and `unimplemented!()` stubs first, then fill in implementations. This ensures all conversions and state handling are covered by tests before the actual code is written.

## Risks / Trade-offs

- [Enum mismatch] If a third engine is added to the backend later, the frontend will fail to deserialize unknown variants → Acceptable for now; both are deployed together.
- [No `#[serde(default)]`] The `template_engine` field has no default on deserialization since backend always sends it → Low risk given simultaneous deployment.
