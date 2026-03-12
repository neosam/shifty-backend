## ADDED Requirements

### Requirement: Template engine field on text templates
Each text template SHALL have a `template_engine` field that specifies which rendering engine to use. Valid values are `"tera"` and `"minijinja"`. The default value SHALL be `"tera"`.

#### Scenario: New template without engine specified
- **WHEN** a text template is created without specifying `template_engine`
- **THEN** the `template_engine` field SHALL default to `"tera"`

#### Scenario: New template with MiniJinja engine
- **WHEN** a text template is created with `template_engine` set to `"minijinja"`
- **THEN** the template SHALL be stored with `template_engine` = `"minijinja"`

#### Scenario: Invalid engine value rejected
- **WHEN** a text template is created or updated with an unsupported `template_engine` value
- **THEN** the system SHALL return a validation error

#### Scenario: Existing templates unaffected by migration
- **WHEN** the database migration runs on an existing database
- **THEN** all existing text templates SHALL have `template_engine` set to `"tera"`

### Requirement: Engine dispatch during report rendering
The system SHALL use the template's `template_engine` field to select the rendering engine when generating reports. Both billing period reports and block reports MUST respect this field.

#### Scenario: Rendering with Tera engine
- **WHEN** a report is generated using a template with `template_engine` = `"tera"`
- **THEN** the system SHALL render the template using the Tera engine

#### Scenario: Rendering with MiniJinja engine
- **WHEN** a report is generated using a template with `template_engine` = `"minijinja"`
- **THEN** the system SHALL render the template using the MiniJinja engine

#### Scenario: Same context data for both engines
- **WHEN** a report is generated
- **THEN** the same template context data (billing period, sales persons, values, template metadata) SHALL be available regardless of which engine is used

### Requirement: Template engine exposed via REST API
The `template_engine` field SHALL be included in the text template REST API responses and accepted in create/update requests.

#### Scenario: Template engine in GET response
- **WHEN** a text template is retrieved via the REST API
- **THEN** the response SHALL include the `template_engine` field

#### Scenario: Template engine in create request
- **WHEN** a text template is created via the REST API with `template_engine` specified
- **THEN** the template SHALL be stored with the specified engine

#### Scenario: Template engine optional in create request
- **WHEN** a text template is created via the REST API without `template_engine`
- **THEN** the template SHALL default to `"tera"`
