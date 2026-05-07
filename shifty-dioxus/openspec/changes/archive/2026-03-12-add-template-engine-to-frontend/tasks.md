## 1. Tests und Stubs (TDD Phase 1)

- [x] 1.1 Add `TemplateEngineTO` enum with `Tera` and `MiniJinja` variants to `rest-types/src/lib.rs` (stub with `unimplemented!()` Display if needed) and add serde round-trip tests
- [x] 1.2 Add `template_engine` field to `TextTemplateTO`, `CreateTextTemplateRequestTO`, `UpdateTextTemplateRequestTO` in rest-types and add serialization tests
- [x] 1.3 Add `TemplateEngine` enum to `src/state/text_template.rs` (stub) and add conversion tests (`From<TemplateEngineTO>` / `Into<TemplateEngineTO>`)
- [x] 1.4 Add `template_engine` field to `TextTemplate` state struct with stub conversions and add tests for `From<TextTemplateTO>`, `to_create_request()`, `to_update_request()`

## 2. Implementierung (TDD Phase 2)

- [x] 2.1 Implement `TemplateEngineTO` with proper serde rename attributes — verify tests pass
- [x] 2.2 Implement `TemplateEngine` state enum and all `From` conversions — verify tests pass
- [x] 2.3 Add i18n keys (`TemplateEngine`, `TemplateEngineTera`, `TemplateEngineMiniJinja`) to `src/i18n/mod.rs` and translations to `en.rs`, `de.rs`, `cs.rs`
- [x] 2.4 Add engine selection dropdown to the create/edit form in `src/page/text_template_management.rs`
