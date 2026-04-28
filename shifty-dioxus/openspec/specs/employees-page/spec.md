# employees-page Specification

## Purpose
TBD - created by archiving change redesign-07-page-employees. Update Purpose after archive.
## Requirements
### Requirement: Master/detail layout with searchable employee list
The Employees page SHALL render its content as a master/detail layout. On viewports above the mobile breakpoint (`>720 px`), the layout SHALL render two columns: a left list (between 280 and 360 px wide) and a right detail area (flex-grow). Below the breakpoint, the page SHALL render either the list or the detail (mutually exclusive), with a back button on the detail returning to the list. The list SHALL be reachable at the existing route `Route::Employees`; the detail at `Route::EmployeeDetails { employee_id }`. The shared shell SHALL be implemented as a reusable composition (`EmployeesShell`) used by both page components so each route renders both panes on desktop.

#### Scenario: Desktop renders list and detail side-by-side
- **WHEN** the page is rendered on a viewport wider than 720 px and the route is `Route::EmployeeDetails { employee_id }`
- **THEN** the page SHALL render two siblings under a flex row: a list pane on the left with classes resolving to a width clamp around 280–360 px, AND a detail pane on the right with `flex-1` (or equivalent flex-grow class)

#### Scenario: Mobile shows list only on Employees route
- **WHEN** the page is rendered on a viewport at or below 720 px and the route is `Route::Employees` (no employee id)
- **THEN** the page SHALL render the list only and SHALL NOT render any detail content

#### Scenario: Mobile shows detail only on EmployeeDetails route
- **WHEN** the page is rendered on a viewport at or below 720 px and the route is `Route::EmployeeDetails { employee_id }`
- **THEN** the page SHALL render the detail only, prefixed with a back button labeled by `Key::BackToList` whose click navigates to `Route::Employees`

### Requirement: List header with search input
Above the employee list rows, the page SHALL render a section heading using `Key::Employees` and a search input below the heading. The search input SHALL filter the visible rows by case-insensitive substring match against `sales_person.name`. The input SHALL use the form-input token classes (`h-[34px]`, `bg-surface`, `text-ink`, `border-border-strong`, `rounded-md`) and SHALL display the placeholder text returned by `Key::SearchPlaceholder`. The filter SHALL apply on every keystroke without requiring a submit action.

#### Scenario: Search filters rows by case-insensitive substring
- **WHEN** the list contains employees named `Lena`, `lena Müller`, and `Tom`, and the user types `lena`
- **THEN** the rendered list SHALL contain rows for `Lena` and `lena Müller` and SHALL NOT contain a row for `Tom`

#### Scenario: Empty search shows all rows
- **WHEN** the search input value is empty
- **THEN** the list SHALL render all non-inactive employees in the loaded data

#### Scenario: Search uses the placeholder translation
- **WHEN** the page renders in any locale
- **THEN** the search input's `placeholder` attribute SHALL equal `i18n.t(Key::SearchPlaceholder)`

### Requirement: List rows with color dot, name, and hours/target
Each employee row SHALL render as a single-line item with three slots: a 10 px circular color dot whose `background-color` is the employee's `sales_person.background_color`, the employee's `sales_person.name` (truncated when too long), and the employee's `balance` and target hours rendered as `<balance>/<target>` in monospace tabular-numbers format. The row SHALL NOT render any text inside the color dot, no avatar circle, no two-letter initials, and no abbreviation. The row SHALL be wrapped in a `<Link>` whose target is `Route::EmployeeDetails { employee_id }`.

#### Scenario: Row renders only color, name, and hours
- **WHEN** any list row is rendered
- **THEN** the row's HTML SHALL contain a `<span>` with an inline `background-color: <hex>` style and `width`/`height` resolving to a 10 px circle, AND the row's text content SHALL contain the employee's name and a hours string of the form `<n>/<n>`

#### Scenario: Color dot has no inner text
- **WHEN** any list row's color dot is rendered
- **THEN** the dot element SHALL have no text content (i.e. no initials, no abbreviation)

#### Scenario: Hours uses mono tabular-numbers
- **WHEN** a row's hours slot is rendered
- **THEN** the hours element SHALL include classes resolving to `font-mono` and `tabular-nums`

### Requirement: Active row highlighted with accent-soft background and accent border
When the list is rendered with the route `Route::EmployeeDetails { employee_id }`, the row whose `sales_person.id == employee_id` SHALL include classes resolving to `bg-accent-soft` and a 3 px solid `border-accent` left border. All other rows SHALL render with no background tint and a 3 px transparent left border (so the layout does not jump when active state changes).

#### Scenario: Active row tinted and bordered
- **WHEN** the list is rendered for `Route::EmployeeDetails { employee_id: "abc" }` and a row exists with `sales_person.id` equal to `abc`
- **THEN** that row's outer element SHALL include `bg-accent-soft` and a class resolving to a 3 px accent-colored left border

#### Scenario: Inactive rows reserve the same border space
- **WHEN** any non-active list row is rendered
- **THEN** the row's outer element SHALL include a 3 px transparent left border (i.e. `border-l-[3px]` and `border-transparent` or equivalent)

#### Scenario: No active state on Employees route
- **WHEN** the list is rendered for `Route::Employees` (no employee id)
- **THEN** no row SHALL include `bg-accent-soft`

### Requirement: Billing-period CRUD removed from the Employees page
The redesigned Employees page SHALL NOT render the billing-period CRUD section (heading, list of billing periods, "create" button, "delete" button, and the create/delete dialogs). All billing-period UI moves to a new `BillingPeriods` page (capability `billing-periods-page`). The Employees page SHALL include a single navigation entry pointing to the billing-periods page (rendered as a button or top-bar entry; the placement is informative, not normative).

#### Scenario: Billing-period content absent from Employees page
- **WHEN** the Employees page is rendered with billing periods present in `BILLING_PERIOD_STORE`
- **THEN** the rendered HTML SHALL NOT contain `Key::BillingPeriods` heading text, SHALL NOT contain the "create new billing period" button, and SHALL NOT contain billing-period list rows

### Requirement: Design tokens replace legacy classes in employees page sources
The non-test sources of `src/page/employees.rs`, `src/component/employee_short.rs`, and any new shell file SHALL NOT contain any of these legacy Tailwind classes: `bg-gray-100`, `bg-gray-200`, `bg-white`, `text-gray-500`, `text-gray-600`, `text-gray-700`, `text-gray-900`, `text-blue-600`, `text-blue-800`, `text-red-500`, `text-red-600`, `text-green-500`, `bg-blue-600`, `bg-green-600`, `bg-red-600`, `border-black`, `border-gray-200`, `border-gray-300`. All surface, ink, border, and accent colors SHALL use design-token classes (`bg-surface`, `bg-accent-soft`, `text-ink`, `text-ink-muted`, `border-border`, `border-accent`, etc.).

#### Scenario: No legacy classes in employees page source
- **WHEN** the non-test source of `src/page/employees.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

#### Scenario: No legacy classes in employee-short component source
- **WHEN** the non-test source of `src/component/employee_short.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

