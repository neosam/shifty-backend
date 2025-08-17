# Block Report Templates

This directory contains sample Tera templates for the Block Report Service. These templates can be used to generate various types of reports based on shift block data.

## Available Templates

### 1. `unsufficiently_booked_blocks.tera`
**Purpose**: Comprehensive report showing all unsufficiently booked blocks for the next three weeks

**Features**:
- Summary statistics for each week
- Detailed block listings organized by week
- Visual statistics showing distribution by day of week
- Action items and recommendations
- Uses emojis for better visual presentation

**Use Case**: HR managers who need a detailed overview of staffing gaps

### 2. `email_notification.tera`
**Purpose**: Email-friendly format for automated notifications about staffing gaps

**Features**:
- Subject line included
- Professional email format
- Prioritizes current week urgent items
- Clear action items
- Suitable for automated email systems

**Use Case**: Automated daily/weekly emails to management about staffing status

### 3. `simple_text_report.tera`
**Purpose**: Minimal, compact text report for quick viewing

**Features**:
- Very concise format
- No decorations or extra formatting
- Good for SMS, chat messages, or terminals
- Shows only essential information

**Use Case**: Quick status checks, mobile notifications, or integration with other systems

### 4. `csv_export.tera`
**Purpose**: CSV format for data export and analysis

**Features**:
- Standard CSV format with headers
- Can be imported into Excel or other spreadsheet applications
- Suitable for further data analysis
- Machine-readable format

**Use Case**: Data export for analysis, integration with other HR systems

## Template Variables Available

All templates have access to the following variables:

### Week Information
- `current_week`: Current week number (1-53)
- `current_year`: Current year
- `next_week`: Next week number
- `next_year`: Year for next week
- `week_after_next_week`: Week after next number
- `week_after_next_year`: Year for week after next

### Block Arrays
- `current_week_blocks`: Array of unsufficiently booked blocks for current week
- `next_week_blocks`: Array of unsufficiently booked blocks for next week
- `week_after_next_blocks`: Array of unsufficiently booked blocks for week after next
- `unsufficiently_booked_blocks`: Combined array of all blocks across three weeks

### Block Object Properties
Each block object in the arrays contains:
- `year`: Year of the block
- `week`: Week number of the block
- `sales_person_name`: Name of assigned employee (may be null/empty)
- `day_of_week`: Day name (e.g., "Monday", "Tuesday")
- `from`: Start time as string (e.g., "9:00:00.0")
- `to`: End time as string (e.g., "17:00:00.0")
- `date`: Full date as string (e.g., "2024-08-19")

## How to Use These Templates

1. **Upload Template to System**:
   ```bash
   # Use the text template API to create a new template
   POST /text-template
   {
     "name": "Weekly Staffing Report",
     "template_type": "block_report",
     "template_text": "<contents of template file>"
   }
   ```

2. **Generate Report**:
   ```bash
   # Use the block report API with the template ID
   POST /block-report/{template_id}
   ```

3. **Customize Templates**:
   - Copy any template as a starting point
   - Modify the Tera syntax to fit your needs
   - Test with sample data before deploying

## Tera Template Syntax Quick Reference

- `{{ variable }}`: Output a variable
- `{% if condition %}...{% endif %}`: Conditional blocks
- `{% for item in array %}...{% endfor %}`: Loops
- `| length`: Filter to get array length
- `| filter(attribute="field", value="value")`: Filter arrays
- `| date(format="%Y-%m-%d")`: Format dates
- `"text" | repeat(times=number)`: Repeat text

## Current Limitations

**Note**: The current implementation focuses on unsufficiently booked blocks (gaps in scheduling). It does not provide:
- Individual employee schedules
- Current user's personal blocks
- Fully booked blocks
- Historical data

To add support for current user blocks, the BlockReportService would need to be enhanced to:
1. Identify the current user from the authentication context
2. Fetch blocks for that specific user
3. Add them to the template context

## Examples of Custom Templates

### Manager Dashboard View
```tera
STAFFING DASHBOARD - {{ "now" | date(format="%B %d, %Y") }}
================================================
{% set total_gaps = unsufficiently_booked_blocks | length %}
{% set urgent_gaps = current_week_blocks | length %}

ALERTS: {% if urgent_gaps > 0 %}🔴 {{ urgent_gaps }} URGENT{% else %}🟢 ALL CLEAR{% endif %}

This Week: {{ current_week_blocks | length }} gaps
Next Week: {{ next_week_blocks | length }} gaps  
Following Week: {{ week_after_next_blocks | length }} gaps
```

### Slack/Teams Notification
```tera
{% if current_week_blocks | length > 0 %}:warning: *Urgent Staffing Alert*
We have {{ current_week_blocks | length }} unfilled shifts this week:
{% for block in current_week_blocks %}• {{ block.day_of_week }} {{ block.from }}-{{ block.to }}
{% endfor %}
Please check the scheduling system.{% else %}:white_check_mark: All shifts covered this week!{% endif %}
```

## Tips for Template Development

1. **Test with Edge Cases**: Empty arrays, single items, many items
2. **Consider Output Format**: HTML needs escaping, CSV needs proper quoting
3. **Keep It Maintainable**: Use clear variable names and comments
4. **Performance**: Avoid complex nested loops for large datasets
5. **Internationalization**: Consider using date/time formats appropriate for your locale

## Support

For questions about template development or the Block Report Service, please contact your system administrator or refer to the main Shifty Backend documentation.