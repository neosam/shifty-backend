# top-bar Specification

## Purpose
TBD - created by archiving change redesign-03-topbar-layout. Update Purpose after archive.
## Requirements
### Requirement: Sticky 56 px theme-aware header
The TopBar SHALL be a sticky header with height 56 px (h-14), positioned at the top of the viewport, using design tokens for surface and ink colors so it adapts to the active theme.

#### Scenario: Header sticks to top while scrolling
- **WHEN** the user scrolls the page down
- **THEN** the TopBar SHALL remain visible at the top of the viewport

#### Scenario: Header adapts to theme
- **WHEN** the active theme is `dark`
- **THEN** the TopBar SHALL render with `background: var(--surface)` resolving to the dark surface color and `color: var(--ink)` resolving to the dark ink color

#### Scenario: Header hidden in print
- **WHEN** the page is printed
- **THEN** the TopBar SHALL NOT appear in the printed output

### Requirement: Brand wordmark with accent period
The TopBar SHALL display the brand `Shifty.` where only the period uses the accent token color.

#### Scenario: Brand period uses accent color
- **WHEN** the TopBar renders the brand
- **THEN** the period character SHALL have `color: var(--accent)` while the word `Shifty` retains the default ink color

#### Scenario: Non-production environment indicator
- **WHEN** `config.is_prod` is `false`
- **THEN** the TopBar SHALL display the `env_short_description` text adjacent to the brand

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

### Requirement: Theme toggle button cycles modes
The TopBar SHALL include a theme-toggle button on the right side that cycles `Light → Dark → System → Light` on click. The button glyph SHALL reflect the current theme mode (`☀` Light, `☾` Dark, `⌬` System).

#### Scenario: Toggle from Light to Dark
- **WHEN** the user clicks the theme toggle while `THEME_MODE` is `Light`
- **THEN** `THEME_MODE` SHALL become `Dark`, the persisted `localStorage.shifty-theme` SHALL be `"dark"`, the toggle glyph SHALL become `☾`, and `<html data-theme>` SHALL become `"dark"`

#### Scenario: Toggle from System completes the cycle
- **WHEN** the user clicks the theme toggle while `THEME_MODE` is `System`
- **THEN** `THEME_MODE` SHALL become `Light`

#### Scenario: Toggle has accessible label
- **WHEN** a screen reader queries the theme toggle button
- **THEN** an `aria-label` describing the current mode SHALL be available

### Requirement: Identity pill with logout dropdown
When the user is authenticated, the TopBar SHALL display a text-only identity pill on the right side reading `[label] [name]` (e.g. `Du bist Lena`), with no avatar circle and no initials. Clicking the pill SHALL open a dropdown containing a `Logout` entry. When unauthenticated, only a `Login` link SHALL appear on the right side.

#### Scenario: Authenticated user sees identity pill
- **WHEN** `AUTH.auth_info` is `Some(info)`
- **THEN** the TopBar SHALL render a pill containing the localized "you are" label followed by `info.user`, AND the pill SHALL NOT contain any image, avatar circle, or initials

#### Scenario: Pill opens account dropdown
- **WHEN** the user clicks the identity pill
- **THEN** a dropdown SHALL open containing at minimum a `Logout` entry

#### Scenario: Logout entry navigates to logout URL
- **WHEN** the user clicks the `Logout` entry in the dropdown
- **THEN** the browser SHALL navigate to `{backend_url}/logout`

#### Scenario: Unauthenticated user sees Login link only
- **WHEN** `AUTH.auth_info` is `None`
- **THEN** the right side of the TopBar SHALL contain only a `Login` link, no identity pill, no dropdown

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

### Requirement: Non-production warning banner preserved
A warning banner SHALL appear immediately below the TopBar when the application is not running in a production environment, using the warn token for styling. The banner SHALL be hidden in print output.

#### Scenario: Non-prod banner visible
- **WHEN** `config.is_prod` is `false`
- **THEN** a banner SHALL render below the TopBar with classes resolving to `background: var(--warn-soft)` and `color: var(--warn)` containing the localized non-prod warning text

#### Scenario: Banner has detail tooltip
- **WHEN** the user hovers the non-prod banner
- **THEN** a tooltip SHALL show the detailed non-prod warning text

#### Scenario: Banner hidden in print
- **WHEN** the page is printed
- **THEN** the non-prod banner SHALL NOT appear in the printed output

#### Scenario: Production hides banner
- **WHEN** `config.is_prod` is `true`
- **THEN** no non-prod banner SHALL render

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

