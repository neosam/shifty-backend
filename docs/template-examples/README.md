# Billing Period Template Examples

This directory contains example templates for generating custom billing period reports using the Shifty Backend API.

## Overview

The custom report generation feature allows you to create Tera templates that extract specific data from billing periods. This is useful when you need:

- Specific metrics from particular employees
- Cross-employee comparisons
- Custom formatted reports
- Data extraction for external systems

## API Endpoint

```
POST /billing-period/{billing_period_id}/custom-report/{template_id}
```

**Requirements:**
- HR permission level
- Valid billing period ID
- Valid text template ID

## Template Examples

### 1. `selective-sales-report.html`
**Use Case:** Extract specific metrics from targeted employees

**Features:**
- Define specific employee IDs to report on
- Show different metric types for different employees
- Visual styling with CSS
- Error handling for missing employees
- Summary section

**Example Usage:**
- Get overall hours for Employee A
- Get custom hours for Employee B  
- Get balance analysis for Employee C

### 2. `cross-employee-comparison.html`
**Use Case:** Compare multiple employees side-by-side

**Features:**
- Tabular comparison format
- Efficiency calculations
- Color-coded status indicators
- Statistical summaries
- Custom hours breakdown

**Example Usage:**
- Compare productivity across team members
- Identify employees with overtime/deficits
- Analyze custom hours usage patterns

### 3. `simple-text-extract.txt`
**Use Case:** Plain text extraction for simple reporting or data export

**Features:**
- Plain text format
- CSV data section for external tools
- Combined calculations
- Conditional alerts
- Easy parsing format

**Example Usage:**
- Extract data for spreadsheet import
- Generate alerts for management
- Create simple summary reports

## Template Data Structure

Templates have access to the following data:

```json
{
  "billing_period": {
    "id": "uuid-string",
    "start_date": "2024-01-01", 
    "end_date": "2024-12-31",
    "created_at": "2024-01-01T00:00:00",
    "created_by": "username",
    "sales_persons": [
      {
        "id": "uuid-string",
        "sales_person_id": "uuid-string", 
        "values": [
          {
            "type": "overall|balance|expected_hours|custom_extra_hours:name|vacation_hours|sick_leave|holiday|vacation_days|vacation_entitlement",
            "value_delta": 123.45,
            "value_ytd_from": 234.56,
            "value_ytd_to": 345.67,
            "value_full_year": 456.78
          }
        ],
        "created_at": "2024-01-01T00:00:00",
        "created_by": "username"
      }
    ]
  },
  "template": {
    "id": "uuid-string",
    "template_type": "report_type",
    "created_at": "2024-01-01T00:00:00", 
    "created_by": "username"
  }
}
```

## Value Types

The `type` field in the values array can be:

- `"overall"` - Total hours worked
- `"balance"` - Hours worked vs expected (surplus/deficit)
- `"expected_hours"` - Expected hours based on contract
- `"extra_work"` - Extra work hours
- `"vacation_hours"` - Vacation time taken
- `"sick_leave"` - Sick leave hours
- `"holiday"` - Holiday hours
- `"vacation_days"` - Vacation days taken
- `"vacation_entitlement"` - Total vacation entitlement
- `"custom_extra_hours:*"` - Custom hour types (e.g., "custom_extra_hours:overtime")

## Key Tera Template Features

### Variables and Filters
```html
{{ billing_period.start_date }}
{{ value.value_delta | round(precision=2) }}
{{ "now" | date(format="%Y-%m-%d") }}
```

### Loops and Conditionals
```html
{% for person in billing_period.sales_persons %}
  {% if person.sales_person_id == "target-id" %}
    {% for value in person.values %}
      {% if value.type == "overall" %}
        Hours: {{ value.value_delta }}h
      {% endif %}
    {% endfor %}
  {% endif %}
{% endfor %}
```

### Variable Assignment
```html
{% set total = 0 %}
{% set_global total = total + value.value_delta %}
```

### String Operations
```html
{{ value.type | title | replace(from="_", to=" ") }}
{{ value.type starts_with "custom_extra_hours:" }}
```

## How to Use These Templates

1. **Create a text template** in the database using the text template API:
   ```
   POST /text-templates
   {
     "template_type": "selective_report",
     "template_text": "... (content from one of these files) ..."
   }
   ```

2. **Get the template ID** from the response

3. **Find a billing period ID** using:
   ```
   GET /billing-period
   ```

4. **Generate the custom report**:
   ```
   POST /billing-period/{billing_period_id}/custom-report/{template_id}
   ```

5. **Customize the employee IDs** in the template to match your actual sales person IDs

## Customization Tips

1. **Replace employee IDs**: Update the UUID strings in the templates with actual sales person IDs from your system

2. **Modify metrics**: Change which value types you want to extract (overall, balance, custom hours, etc.)

3. **Adjust styling**: Modify CSS in HTML templates for your branding

4. **Add calculations**: Use Tera's math operations to compute totals, averages, percentages

5. **Error handling**: Add checks for missing data or employees not found

6. **Output format**: Choose HTML for rich display, text for simple reports, or create CSV sections for data export

## Security Notes

- Only users with HR privileges can generate custom reports
- Templates have access to all employee data in the billing period
- Be mindful of sensitive information when creating templates
- Templates are stored in the database and can be versioned/audited