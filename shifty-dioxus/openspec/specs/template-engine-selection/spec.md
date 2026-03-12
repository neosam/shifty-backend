## ADDED Requirements

### Requirement: REST types include template engine enum
The `rest-types` crate SHALL define a `TemplateEngineTO` enum with variants `Tera` and `MiniJinja`, serialized as `"tera"` and `"minijinja"` respectively.

#### Scenario: Deserialize Tera engine from JSON
- **WHEN** a JSON response contains `"template_engine": "tera"`
- **THEN** it SHALL deserialize to `TemplateEngineTO::Tera`

#### Scenario: Deserialize MiniJinja engine from JSON
- **WHEN** a JSON response contains `"template_engine": "minijinja"`
- **THEN** it SHALL deserialize to `TemplateEngineTO::MiniJinja`

#### Scenario: Serialize engine to JSON
- **WHEN** `TemplateEngineTO::MiniJinja` is serialized
- **THEN** the JSON output SHALL contain `"minijinja"`

### Requirement: TextTemplateTO includes template_engine field
`TextTemplateTO`, `CreateTextTemplateRequestTO`, and `UpdateTextTemplateRequestTO` SHALL each contain a `template_engine` field of type `TemplateEngineTO`.

#### Scenario: Full template round-trip
- **WHEN** a `TextTemplateTO` with `template_engine: MiniJinja` is serialized and deserialized
- **THEN** the `template_engine` field SHALL equal `TemplateEngineTO::MiniJinja`

### Requirement: Frontend state includes template engine
The `TextTemplate` state struct SHALL contain a `template_engine` field of type `TemplateEngine`. Conversions between `TextTemplateTO` and `TextTemplate` SHALL preserve the engine value.

#### Scenario: Convert TextTemplateTO to TextTemplate
- **WHEN** a `TextTemplateTO` with `template_engine: Tera` is converted to `TextTemplate`
- **THEN** the resulting `TextTemplate` SHALL have `template_engine: TemplateEngine::Tera`

#### Scenario: Create request includes engine
- **WHEN** a `TextTemplate` with `template_engine: MiniJinja` calls `to_create_request()`
- **THEN** the resulting `CreateTextTemplateRequestTO` SHALL have `template_engine: TemplateEngineTO::MiniJinja`

#### Scenario: Update request includes engine
- **WHEN** a `TextTemplate` with `template_engine: Tera` calls `to_update_request()`
- **THEN** the resulting `UpdateTextTemplateRequestTO` SHALL have `template_engine: TemplateEngineTO::Tera`

### Requirement: Template engine selection in edit form
The template management page SHALL display a dropdown to select the template engine (Tera or MiniJinja) in the create and edit form. The default selection SHALL be Tera.

#### Scenario: New template defaults to Tera
- **WHEN** the user opens the create template form
- **THEN** the engine dropdown SHALL default to `Tera`

#### Scenario: Edit existing MiniJinja template
- **WHEN** the user edits a template that uses MiniJinja
- **THEN** the engine dropdown SHALL show `MiniJinja` as the selected value

#### Scenario: User selects MiniJinja and saves
- **WHEN** the user selects MiniJinja in the dropdown and saves the template
- **THEN** the create/update request SHALL include `template_engine: "minijinja"`

### Requirement: i18n support for template engine labels
Translation keys SHALL exist for the template engine label and both engine names in all three locales (En, De, Cs).

#### Scenario: German locale shows engine label
- **WHEN** the UI renders the engine dropdown label in German locale
- **THEN** it SHALL display the translated label for "Template Engine"

#### Scenario: All locales have engine name translations
- **WHEN** any of the three locales (En, De, Cs) is active
- **THEN** display names for both Tera and MiniJinja SHALL be available
