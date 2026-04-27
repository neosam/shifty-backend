## MODIFIED Requirements

### Requirement: Privilege-gated navigation with active-route highlighting
The TopBar SHALL render navigation links gated by user privileges. The link corresponding to the currently active route SHALL be visually highlighted using the accent-soft background. Administrative items (`Mitarbeiter`, `Abrechnungszeiträume`, `Benutzerverwaltung`, `Textvorlagen`) SHALL NOT render directly in the desktop nav bar; instead they render inside an "Administration" group dropdown (see "Administration group dropdown" requirement). Non-administrative items (`Schichtplan`, `Meine Schichten`, `Meine Zeit`, `Jahresübersicht`) SHALL render as direct top-level entries in the desktop nav bar.

#### Scenario: User with `sales` privilege sees Schichtplan as top-level
- **WHEN** the user has the `sales` privilege
- **THEN** the `Schichtplan` link SHALL be rendered directly in the desktop nav bar

#### Scenario: User without any privilege sees no nav links
- **WHEN** the user has no privileges
- **THEN** no privilege-gated nav link SHALL be rendered in the desktop nav bar AND no Administration group trigger SHALL be rendered

#### Scenario: Active top-level route highlighted
- **WHEN** the current route is `Route::ShiftPlan`
- **THEN** the `Schichtplan` nav link SHALL render with classes resolving to `background: var(--accent-soft)` and `color: var(--accent)` and font-weight 600

#### Scenario: Parameterised top-level route highlights parent
- **WHEN** the current route is a parameterised top-level route such as `Route::ShiftPlanDeep { … }`
- **THEN** the corresponding parent nav link SHALL render in the active highlighted state

#### Scenario: Administrative items not rendered as top-level entries
- **WHEN** the user has the `hr` privilege
- **THEN** the `Mitarbeiter` link SHALL NOT appear directly in the desktop nav bar AND it SHALL appear inside the Administration group dropdown

#### Scenario: Administrative parameterised route highlights group trigger, not a direct nav entry
- **WHEN** the current route is `Route::EmployeeDetails { … }`
- **THEN** no direct nav-bar entry SHALL render the active highlighted state AND the Administration group trigger SHALL render in the active highlighted state (see "Administration group dropdown active state")

#### Scenario: Privilege rules unchanged
- **WHEN** comparing privilege gating before and after this change
- **THEN** the visibility rules for each of the eight nav items SHALL be identical (no removals, no relaxations, no additions); only the rendering placement changes

### Requirement: Mobile burger menu collapses navigation
At viewports below 720 px, the TopBar SHALL collapse the navigation into a dropdown panel toggled by a burger button. The burger glyph SHALL switch between `☰` (closed) and `✕` (open). The panel SHALL auto-close when the route changes. Inside the panel, top-level items SHALL appear first; if any administrative items are visible to the current user, an `Administration` section header SHALL precede the administrative items rendered as flat entries (no nested dropdown on mobile).

#### Scenario: Mobile viewport collapses nav
- **WHEN** the viewport width is below 720 px
- **THEN** the navigation links SHALL NOT be visible in the bar; the burger button SHALL be visible

#### Scenario: Burger toggles dropdown
- **WHEN** the user taps the burger button
- **THEN** the dropdown panel SHALL appear below the bar containing all visible nav links (top-level then administrative) AND the burger glyph SHALL change from `☰` to `✕`

#### Scenario: Route change closes dropdown
- **WHEN** the dropdown is open and the user navigates to a different route
- **THEN** the dropdown SHALL close automatically

#### Scenario: Mobile administration section header
- **WHEN** the mobile dropdown is open AND at least one administrative item is visible to the current user
- **THEN** a section header reading the localized `Administration` label SHALL render between the top-level items and the administrative items, styled with `font-size: 11px`, `font-weight: 700`, uppercase, `letter-spacing: 0.06em`, color `var(--ink-muted)`, and a top border separator using `var(--border)`

#### Scenario: Mobile admin section omitted when no admin items visible
- **WHEN** the mobile dropdown is open AND no administrative item is visible to the current user
- **THEN** the `Administration` section header SHALL NOT render AND no flat administrative entries SHALL render below the top-level items

## ADDED Requirements

### Requirement: Administration group dropdown
The desktop TopBar SHALL render administrative nav items inside a single group dropdown opened by an "Administration" trigger button. The trigger button SHALL appear at the end of the desktop nav, after the top-level items. The trigger button SHALL be hidden entirely when no administrative item is visible to the current user. The trigger button SHALL display a localized "Administration" label and a `▾` chevron at the end of the label. Clicking the trigger SHALL open a dropdown panel listing the administrative items visible to the current user, in declaration order: `Mitarbeiter`, `Abrechnungszeiträume`, `Benutzerverwaltung`, `Textvorlagen`. The panel SHALL be positioned `position: fixed` with its top edge 4 px below the trigger's bounding rectangle and its left edge aligned to the trigger's left edge. The panel SHALL have minimum width 220 px, surface background, a 1 px border using `var(--border)`, the medium border-radius token, and a soft drop shadow.

#### Scenario: Trigger hidden when no admin item is visible
- **WHEN** the user has only the `sales` privilege (no `hr`, no `admin`)
- **THEN** the Administration group trigger SHALL NOT render in the desktop nav

#### Scenario: Trigger renders when at least one admin item is visible
- **WHEN** the user has the `hr` privilege but no `admin` privilege
- **THEN** the Administration group trigger SHALL render in the desktop nav AND the dropdown panel (when opened) SHALL list only the admin items visible to that user (i.e. `Mitarbeiter` and `Abrechnungszeiträume`)

#### Scenario: Default trigger label
- **WHEN** the trigger renders AND no administrative route is currently active
- **THEN** the trigger button SHALL display the localized "Administration" label followed by a `▾` chevron

#### Scenario: Trigger label substitution when admin route active
- **WHEN** the current route is `Route::Employees` (or its parameterised variant `Route::EmployeeDetails { … }`)
- **THEN** the trigger button SHALL display the localized `Mitarbeiter` label (replacing the "Administration" label) followed by the `▾` chevron

#### Scenario: Dropdown opens on click
- **WHEN** the user clicks the Administration group trigger
- **THEN** the dropdown panel SHALL open below the trigger AND list the visible administrative items

#### Scenario: Dropdown closes on outside click
- **WHEN** the dropdown panel is open AND the user clicks outside both the panel and the trigger
- **THEN** the dropdown panel SHALL close

#### Scenario: Dropdown closes on Escape key
- **WHEN** the dropdown panel is open AND the user presses the `Escape` key
- **THEN** the dropdown panel SHALL close

#### Scenario: Reopening immediately is not blocked by the opening click
- **WHEN** the user clicks the trigger to open the dropdown
- **THEN** the dropdown SHALL remain open after the click event tick (the document-level outside-click listener SHALL NOT close the panel as part of the same click that opened it)

### Requirement: Administration group dropdown active state
When any administrative item is the current active route, the Administration group trigger SHALL render in the active visual state. When clicked open, the active item inside the panel SHALL also render in the active visual state.

#### Scenario: Trigger adopts active pill when admin route active
- **WHEN** the current route is `Route::Employees`
- **THEN** the Administration group trigger SHALL render with classes resolving to `background: var(--accent-soft)`, `color: var(--accent)`, and font-weight 600

#### Scenario: Active item highlighted inside panel
- **WHEN** the dropdown panel is open AND the current route is `Route::Employees`
- **THEN** the `Mitarbeiter` entry inside the panel SHALL render with classes resolving to `background: var(--accent-soft)` and `color: var(--accent)`

#### Scenario: Trigger uses inactive style when no admin route active
- **WHEN** no administrative route is currently active
- **THEN** the Administration group trigger SHALL render with the inactive nav classes (`text-ink-soft` with hover `bg-surface-alt`)
