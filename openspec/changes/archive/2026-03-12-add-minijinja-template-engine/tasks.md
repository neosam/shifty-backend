## 1. Scaffolding (make it compile)

- [x] 1.1 Add `TemplateEngineEntity` enum (`Tera`, `MiniJinja`) with `TryFrom<&str>` and `Display` in `dao/src/text_template.rs`, add `template_engine: TemplateEngineEntity` field to `TextTemplateEntity`
- [x] 1.2 Add `TemplateEngine` enum (`Tera`, `MiniJinja`) with `From<TemplateEngineEntity>` in `service/src/text_template.rs`, add `template_engine: TemplateEngine` field to `TextTemplate`
- [x] 1.3 Add `TemplateEngineTO` enum (`Tera`, `MiniJinja`) with `Serialize`, `Deserialize`, `ToSchema`, and `From<TemplateEngine>` in `rest-types/src/lib.rs`, add `template_engine` field to `TextTemplateTO`, `CreateTextTemplateRequestTO`, and `UpdateTextTemplateRequestTO`
- [x] 1.4 Add `minijinja` dependency to `service_impl/Cargo.toml`
- [x] 1.5 Add stub/`todo!()` placeholders in `dao_impl_sqlite`, `rest`, and `service_impl` where needed to make the project compile with the new field
- [x] 1.6 Verify the project compiles with `cargo build`

## 2. Write Tests (red phase)

- [x] 2.1 Add test for engine default: template created without engine gets `TemplateEngine::Tera`
- [x] 2.2 Add test for Tera rendering: existing Tera template still renders correctly (regression)
- [x] 2.3 Add test for MiniJinja rendering: template with `TemplateEngine::MiniJinja` renders correctly
- [x] 2.4 Add test for MiniJinja dict literal: template using `{% set m = {"key": "val"} %}` and `m[variable]` renders correctly
- [x] 2.5 Add test for same context: render a template accessing `billing_period.sales_persons` with both Tera and MiniJinja, verify both produce the same output
- [x] 2.6 Verify tests compile but fail with `cargo test`

## 3. Implementation (green phase)

- [x] 3.1 Create SQL migration adding `template_engine TEXT NOT NULL DEFAULT 'tera'` to `text_template` table
- [x] 3.2 Update `dao_impl_sqlite/src/text_template.rs` to read/write the `template_engine` column in all SQL queries, converting via `TemplateEngineEntity`
- [x] 3.3 Update `rest/src/text_template.rs` handlers to pass `template_engine` through on create/update
- [x] 3.4 Implement the render dispatch: match on `TemplateEngine::Tera` / `TemplateEngine::MiniJinja` to select the rendering engine
- [x] 3.5 Update `billing_period_report.rs` `generate_custom_report` to use the render dispatch
- [x] 3.6 Update `block_report.rs` report generation to use the render dispatch
- [x] 3.7 Run `cargo test` and verify all tests pass
- [x] 3.8 Run `cargo build` and `cargo run` to verify everything works end-to-end
