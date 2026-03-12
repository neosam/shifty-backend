## Why

The current text template system only supports Tera as its rendering engine. Tera lacks native support for dictionary literals in templates, forcing template authors to hardcode UUIDs or use verbose `{% if %}` chains for name mappings. MiniJinja supports dict literals (`{% set names = {"key": "value"} %}`), enabling much simpler and more readable templates — especially for billing period reports where employee names need to be mapped.

Adding MiniJinja as a second engine alongside Tera allows a gradual migration: existing templates continue to work unchanged with Tera (the default), while new templates can opt into MiniJinja for its richer syntax.

## What Changes

- Add `template_engine` column to the `text_template` database table, defaulting to `"tera"`
- Extend `TextTemplateEntity`, `TextTemplate`, and `TextTemplateTO` with a `template_engine` field
- Add MiniJinja as a dependency to `service_impl`
- Update report rendering logic (billing period reports and block reports) to dispatch to the correct engine based on the template's `template_engine` field
- Expose `template_engine` in REST API create/update endpoints (optional, defaults to `"tera"`)

## Capabilities

### New Capabilities
- `template-engine-selection`: Ability to choose between Tera and MiniJinja as the rendering engine for text templates, stored per template and respected during report generation.

### Modified Capabilities

_(none — no existing spec-level requirements change)_

## Impact

- **Database**: New migration adding `template_engine TEXT NOT NULL DEFAULT 'tera'` to `text_template`
- **Crates affected**: `dao`, `dao_impl_sqlite`, `service`, `service_impl`, `rest`, `rest-types`
- **Dependencies**: New `minijinja` crate added to `service_impl/Cargo.toml`
- **API**: `TextTemplateTO`, create/update request TOs gain an optional `template_engine` field
- **Backward compatibility**: Fully backward compatible — default engine is Tera, existing templates and API consumers unaffected
