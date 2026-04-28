## MODIFIED Requirements

### Requirement: Same context data for both engines
The system SHALL use the template's `template_engine` field to select the rendering engine when generating reports. Both billing period reports and block reports MUST respect this field.

#### Scenario: Same context data for both engines
- **WHEN** a report is generated
- **THEN** the same template context data (billing period, sales persons, values, values_map, employee metadata, template metadata) SHALL be available regardless of which engine is used
