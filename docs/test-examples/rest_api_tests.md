# REST API Test Examples for Custom Reports

This document provides comprehensive test examples for the custom report generation API endpoints.

## Prerequisites

1. **Authentication**: You need HR-level access
2. **Running Server**: Shifty backend must be running
3. **Database**: SQLite database with proper migrations
4. **Test Data**: Sales persons and billing periods must exist

## Test Environment Setup

```bash
# Start the server with mock authentication
cargo run --features mock_auth

# Or use environment variables for real auth
export APP_URL="http://localhost:3000"
export ISSUER="your-oidc-issuer"
export CLIENT_ID="your-client-id"
cargo run --features oidc
```

## 1. Creating Test Data

### Step 1: Create Sales Persons

```bash
# Create Natalie
curl -X POST http://localhost:3000/sales-person \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "name": "Natalie",
    "inactive": false,
    "is_paid": true
  }'
# Response: {"id": "natalie-uuid", ...}

# Create Dany
curl -X POST http://localhost:3000/sales-person \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "name": "Dany", 
    "inactive": false,
    "is_paid": true
  }'
# Response: {"id": "dany-uuid", ...}
```

### Step 2: Create Billing Period

```bash
curl -X POST http://localhost:3000/billing-period \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "end_date": "2025-08-14"
  }'
# Response: {"id": "billing-period-uuid", ...}
```

## 2. Text Template Tests

### Test 1: Create German Hours Report Template

```bash
curl -X POST http://localhost:3000/text-templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "template_type": "german_hours_report",
    "template_text": "Hallo Frau Saur,\n\nhiermit sende ich Ihnen die Stunden für den Abrechnungszeitraum vom {{ billing_period.start_date }} bis {{ billing_period.end_date }}.\n{% for person in billing_period.sales_persons %}{% if person.sales_person_id == \"natalie-uuid\" %}{% for value in person.values %}{% if value.type == \"overall\" %}Natalie: {{ value.value_delta | round(precision=0) }} Stunden\n{% endif %}{% endfor %}{% elif person.sales_person_id == \"dany-uuid\" %}{% for value in person.values %}{% if value.type == \"overall\" %}Dany: {{ value.value_delta | round(precision=0) }} Stunden\n{% endif %}{% endfor %}{% endif %}{% endfor %}\nViele Grüße,"
  }'
```

**Expected Response:**
```json
{
  "id": "template-uuid",
  "template_type": "german_hours_report",
  "template_text": "Hallo Frau Saur...",
  "created_at": "2024-01-01T10:00:00",
  "created_by": "test_user"
}
```

### Test 2: Create Custom Hours Extraction Template

```bash
curl -X POST http://localhost:3000/text-templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "template_type": "custom_hours_extract",
    "template_text": "Custom Hours Report:\n{% for person in billing_period.sales_persons %}{% if person.sales_person_id == \"employee-uuid\" %}{% for value in person.values %}{% if value.type starts_with \"custom_extra_hours:\" %}{{ value.type | replace(from=\"custom_extra_hours:\", to=\"\") | title }}: {{ value.value_delta }}h\n{% endif %}{% endfor %}{% endif %}{% endfor %}"
  }'
```

### Test 3: Create Cross-Employee Comparison Template

```bash
curl -X POST http://localhost:3000/text-templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "template_type": "team_comparison",
    "template_text": "Team Report:\n{% set total = 0 %}{% for person in billing_period.sales_persons %}{% for value in person.values %}{% if value.type == \"overall\" %}Employee {{ person.sales_person_id }}: {{ value.value_delta }}h\n{% set_global total = total + value.value_delta %}{% endif %}{% endfor %}{% endfor %}Total: {{ total }}h"
  }'
```

## 3. Custom Report Generation Tests

### Test 1: Generate German Hours Report

```bash
curl -X POST http://localhost:3000/billing-period/{billing-period-uuid}/custom-report/{template-uuid} \
  -H "Authorization: Bearer your-token"
```

**Expected Response:**
```
Hallo Frau Saur,

hiermit sende ich Ihnen die Stunden für den Abrechnungszeitraum vom 2025-07-15 bis 2025-08-14.
Natalie: 4 Stunden
Dany: 24 Stunden

Viele Grüße,
```

### Test 2: Generate Custom Hours Report

```bash
curl -X POST http://localhost:3000/billing-period/{billing-period-uuid}/custom-report/{custom-hours-template-uuid} \
  -H "Authorization: Bearer your-token"
```

**Expected Response:**
```
Custom Hours Report:
Overtime: 8h
Bonus: 4h
Training: 2h
```

### Test 3: Generate Team Comparison Report

```bash
curl -X POST http://localhost:3000/billing-period/{billing-period-uuid}/custom-report/{team-template-uuid} \
  -H "Authorization: Bearer your-token"
```

**Expected Response:**
```
Team Report:
Employee 12345678-1234-1234-1234-123456789012: 160h
Employee 87654321-4321-4321-4321-210987654321: 140h
Employee 11111111-2222-3333-4444-555555555555: 180h
Total: 480h
```

## 4. Error Case Tests

### Test 1: Unauthorized Access (Non-HR User)

```bash
curl -X POST http://localhost:3000/billing-period/{billing-period-uuid}/custom-report/{template-uuid} \
  -H "Authorization: Bearer non-hr-token"
```

**Expected Response:**
```
HTTP 403 Forbidden
```

### Test 2: Template Not Found

```bash
curl -X POST http://localhost:3000/billing-period/{billing-period-uuid}/custom-report/non-existent-uuid \
  -H "Authorization: Bearer your-token"
```

**Expected Response:**
```
HTTP 404 Not Found
TextTemplate not found
```

### Test 3: Billing Period Not Found

```bash
curl -X POST http://localhost:3000/billing-period/non-existent-uuid/custom-report/{template-uuid} \
  -H "Authorization: Bearer your-token"
```

**Expected Response:**
```
HTTP 404 Not Found
```

### Test 4: Invalid Template Syntax

```bash
curl -X POST http://localhost:3000/text-templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "template_type": "invalid_template",
    "template_text": "{{ unclosed_tag"
  }'
```

When generating report with this template:

**Expected Response:**
```
HTTP 500 Internal Server Error
```

## 5. Advanced Test Scenarios

### Test 1: Template with Missing Employee

```bash
# Create template looking for non-existent employee
curl -X POST http://localhost:3000/text-templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "template_type": "missing_employee_test",
    "template_text": "{% set found = false %}{% for person in billing_period.sales_persons %}{% if person.sales_person_id == \"non-existent-uuid\" %}{% set_global found = true %}Found!{% endif %}{% endfor %}{% if not found %}Employee not found{% endif %}"
  }'

# Generate report
curl -X POST http://localhost:3000/billing-period/{billing-period-uuid}/custom-report/{template-uuid} \
  -H "Authorization: Bearer your-token"
```

**Expected Response:**
```
Employee not found
```

### Test 2: Complex Calculation Template

```bash
curl -X POST http://localhost:3000/text-templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "template_type": "complex_calculations",
    "template_text": "{% set total_hours = 0 %}{% set overtime_threshold = 160 %}{% for person in billing_period.sales_persons %}{% for value in person.values %}{% if value.type == \"overall\" %}{% set_global total_hours = total_hours + value.value_delta %}{% endif %}{% endfor %}{% endfor %}Total: {{ total_hours }}h\nAverage: {{ (total_hours / billing_period.sales_persons | length) | round(precision=1) }}h\n{% if total_hours > overtime_threshold %}⚠️ Team overtime detected!{% endif %}"
  }'
```

### Test 3: CSV Export Template

```bash
curl -X POST http://localhost:3000/text-templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "template_type": "csv_export",
    "template_text": "Employee_ID,Overall_Hours,Balance_Hours\n{% for person in billing_period.sales_persons %}{% set overall = 0 %}{% set balance = 0 %}{% for value in person.values %}{% if value.type == \"overall\" %}{% set overall = value.value_delta %}{% elif value.type == \"balance\" %}{% set balance = value.value_delta %}{% endif %}{% endfor %}{{ person.sales_person_id }},{{ overall }},{{ balance }}\n{% endfor %}"
  }'
```

**Expected Response:**
```
Employee_ID,Overall_Hours,Balance_Hours
12345678-1234-1234-1234-123456789012,160,10
87654321-4321-4321-4321-210987654321,140,-10
```

## 6. Performance Tests

### Test 1: Large Billing Period

```bash
# Create billing period with many employees (simulate via multiple API calls)
# Then test report generation time

time curl -X POST http://localhost:3000/billing-period/{large-billing-period-uuid}/custom-report/{template-uuid} \
  -H "Authorization: Bearer your-token"
```

### Test 2: Complex Template

```bash
# Create template with nested loops and calculations
# Measure generation time for performance baseline

time curl -X POST http://localhost:3000/billing-period/{billing-period-uuid}/custom-report/{complex-template-uuid} \
  -H "Authorization: Bearer your-token"
```

## 7. Integration Test Script

Create a bash script to run all tests:

```bash
#!/bin/bash
# test_custom_reports.sh

BASE_URL="http://localhost:3000"
TOKEN="your-hr-token"

echo "Testing Custom Report Generation API..."

# Test 1: Create template
echo "1. Creating template..."
TEMPLATE_RESPONSE=$(curl -s -X POST $BASE_URL/text-templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"template_type": "test", "template_text": "Test: {{ billing_period.id }}"}')

TEMPLATE_ID=$(echo $TEMPLATE_RESPONSE | jq -r '.id')
echo "Template created: $TEMPLATE_ID"

# Test 2: Create billing period
echo "2. Creating billing period..."
BP_RESPONSE=$(curl -s -X POST $BASE_URL/billing-period \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"end_date": "2025-08-14"}')

BP_ID=$(echo $BP_RESPONSE | jq -r '.id')
echo "Billing period created: $BP_ID"

# Test 3: Generate report
echo "3. Generating custom report..."
REPORT=$(curl -s -X POST $BASE_URL/billing-period/$BP_ID/custom-report/$TEMPLATE_ID \
  -H "Authorization: Bearer $TOKEN")

echo "Generated report:"
echo "$REPORT"

# Test 4: Error cases
echo "4. Testing error cases..."

# Non-existent template
curl -s -w "Status: %{http_code}\n" -X POST $BASE_URL/billing-period/$BP_ID/custom-report/non-existent \
  -H "Authorization: Bearer $TOKEN" > /dev/null

# Non-existent billing period  
curl -s -w "Status: %{http_code}\n" -X POST $BASE_URL/billing-period/non-existent/custom-report/$TEMPLATE_ID \
  -H "Authorization: Bearer $TOKEN" > /dev/null

echo "All tests completed!"
```

## 8. Automated Testing with curl and jq

```bash
# Test successful report generation
test_success() {
  local response=$(curl -s -w "%{http_code}" -X POST \
    "$BASE_URL/billing-period/$BP_ID/custom-report/$TEMPLATE_ID" \
    -H "Authorization: Bearer $TOKEN")
  
  local status_code="${response: -3}"
  local body="${response%???}"
  
  if [ "$status_code" = "200" ]; then
    echo "✅ Success test passed"
    echo "Report content: $body"
  else
    echo "❌ Success test failed. Status: $status_code"
  fi
}

# Test permission error
test_permission_error() {
  local status_code=$(curl -s -w "%{http_code}" -o /dev/null -X POST \
    "$BASE_URL/billing-period/$BP_ID/custom-report/$TEMPLATE_ID" \
    -H "Authorization: Bearer invalid-token")
  
  if [ "$status_code" = "403" ]; then
    echo "✅ Permission test passed"
  else
    echo "❌ Permission test failed. Expected 403, got: $status_code"
  fi
}

# Run all tests
test_success
test_permission_error
```

These test examples cover all aspects of the custom report generation feature:

- ✅ Template creation and management
- ✅ Report generation with real data
- ✅ Error handling and edge cases  
- ✅ Security and permission testing
- ✅ Performance testing
- ✅ Complex template scenarios
- ✅ Integration testing scripts

Use these tests to verify that your custom report feature works correctly across all scenarios!