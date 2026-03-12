## Why

The backend now supports two templating engines (Tera and MiniJinja) per text template, stored as a `template_engine` field. The frontend's REST types, state, and UI do not yet include this field, so users cannot select or see which engine a template uses when creating or editing templates.

## What Changes

- Add `TemplateEngineTO` enum (`Tera`, `MiniJinja`) to the frontend's `rest-types` crate
- Add `template_engine` field to `TextTemplateTO`, `CreateTextTemplateRequestTO`, and `UpdateTextTemplateRequestTO`
- Add `TemplateEngine` enum and `template_engine` field to the frontend state (`TextTemplate`)
- Add engine selection dropdown to the template edit/create form in the management page
- Add i18n keys for the engine labels in all three locales (En, De, Cs)

## Capabilities

### New Capabilities
- `template-engine-selection`: Frontend support for selecting and displaying the template engine (Tera or MiniJinja) when creating and editing text templates

### Modified Capabilities

## Impact

- `rest-types/src/lib.rs`: New enum, updated structs
- `src/state/text_template.rs`: New enum, updated state struct and conversions
- `src/page/text_template_management.rs`: Engine dropdown in edit/create form
- `src/i18n/{mod,en,de,cs}.rs`: New translation keys
