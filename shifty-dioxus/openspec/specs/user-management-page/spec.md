# user-management-page Specification

## Purpose
TBD - created by archiving change redesign-08-page-usermgmt. Update Purpose after archive.
## Requirements
### Requirement: Two-tab layout with SalesPerson and Benutzer tabs
The User Management page SHALL render its content as two tabs: `SalesPerson` and `Benutzer`. Tab state SHALL be page-local (not URL-persistent) and default to `SalesPerson` on first paint. Only the active tab's table content SHALL render at any given time. The two tabs SHALL be reachable from the existing route `Route::UserManagementPage` (no route segment is added).

#### Scenario: Default tab is SalesPerson
- **WHEN** the page is rendered for the first time
- **THEN** the SalesPerson tab table SHALL be visible and the Benutzer tab table SHALL NOT be present in the rendered HTML

#### Scenario: Switching to Benutzer tab swaps the visible table
- **WHEN** the user activates the Benutzer tab
- **THEN** the rendered HTML SHALL contain the Benutzer table headers and SHALL NOT contain the SalesPerson table headers

#### Scenario: Tab buttons render as flat underline tabs
- **WHEN** the tab bar is rendered
- **THEN** the active tab's `<button>` SHALL include classes resolving to `border-b-2`, `border-accent`, and `text-accent`, AND each inactive tab's `<button>` SHALL include classes resolving to `border-b-2`, `border-transparent`, and `text-ink-soft`

### Requirement: SalesPerson tab table structure
The SalesPerson tab SHALL render a `<table>` with one row per sales person. Each row SHALL contain four cells (in this order): a name cell with a 10 px circular color dot styled with the sales person's `background_color` followed by the sales person's name, a type cell containing a `PersonChip` rendering `Bezahlt` (when `is_paid`) or `Freiwillig` (otherwise), a linked-user cell rendering the linked user's login in `font-mono` (or `—` when unlinked or not yet loaded), and an Edit cell containing a `Btn` Secondary that navigates to `Route::SalesPersonDetails { sales_person_id }`. The color dot SHALL contain no text and SHALL NOT render any avatar circle, abbreviation, or two-letter initials.

#### Scenario: Color dot has no inner text
- **WHEN** any SalesPerson tab row is rendered
- **THEN** the color-dot element SHALL have no text content (no initials, no abbreviation)

#### Scenario: Type pill renders Paid for paid sales persons
- **WHEN** a sales person with `is_paid = true` is rendered in the SalesPerson tab
- **THEN** the row's type cell SHALL contain a chip whose text equals `i18n.t(Key::Paid)` and whose class list SHALL include `person-pill`

#### Scenario: Type pill renders Volunteer for non-paid sales persons
- **WHEN** a sales person with `is_paid = false` is rendered in the SalesPerson tab
- **THEN** the row's type cell SHALL contain a chip whose text equals `i18n.t(Key::Volunteer)` and whose class list SHALL include `person-pill`

#### Scenario: Linked user cell shows login when present
- **WHEN** a sales person with id `X` is rendered AND `sales_person_user_links[X] = Some(Some("alex"))`
- **THEN** the row's linked-user cell SHALL contain a `<span>` with class `font-mono` and text `alex`

#### Scenario: Linked user cell shows dash when unlinked
- **WHEN** a sales person with id `X` is rendered AND `sales_person_user_links[X] = Some(None)`
- **THEN** the row's linked-user cell SHALL render `i18n.t(Key::Unlinked)` (the em-dash glyph)

#### Scenario: Linked user cell shows dash when not yet loaded
- **WHEN** a sales person with id `X` is rendered AND `sales_person_user_links` does not contain key `X`
- **THEN** the row's linked-user cell SHALL render `i18n.t(Key::Unlinked)` and SHALL NOT crash or render `undefined`

#### Scenario: Edit button navigates to sales-person details
- **WHEN** the Edit button on a sales-person row is rendered
- **THEN** it SHALL be a `Btn` Secondary wrapped in a navigation that targets `Route::SalesPersonDetails { sales_person_id }` for that row's sales person

### Requirement: Benutzer tab table structure
The Benutzer tab SHALL render a `<table>` with one row per user. Each row SHALL contain four cells (in this order): a login cell rendering the user's `username` in `font-mono`, a linked-sales-person cell rendering a 10 px circular color dot (using the linked sales person's `background_color`) followed by the linked sales person's name (or `—` when unlinked), a roles cell rendering one accent-soft pill chip per assigned role (or `—` when no roles), and an Edit cell containing a `Btn` Secondary that navigates to `Route::UserDetails { user_id }`. The Benutzer tab SHALL NOT render a Status column.

#### Scenario: Login renders in monospace
- **WHEN** any Benutzer tab row is rendered
- **THEN** the login cell SHALL include a class resolving to `font-mono`

#### Scenario: Linked sales person cell renders dot and name
- **WHEN** a user `alex` is rendered AND `user_sales_person_links["alex"] = Some(Some(SalesPerson { name: "Lena", background_color: "#dbe0ff", ... }))`
- **THEN** the row's linked-sales-person cell SHALL contain an inline-styled `<span>` with `background-color: #dbe0ff` AND a name `<span>` containing `Lena`

#### Scenario: Linked sales person cell shows dash when unlinked
- **WHEN** a user `alex` is rendered AND `user_sales_person_links["alex"] = Some(None)`
- **THEN** the row's linked-sales-person cell SHALL render `i18n.t(Key::Unlinked)` (the em-dash glyph)

#### Scenario: Roles cell renders one chip per role
- **WHEN** a user `alex` is rendered AND `user_role_assignments["alex"]` equals `["admin", "hr"]`
- **THEN** the row's roles cell SHALL contain exactly two `<span>` chips, each with classes resolving to `bg-accent-soft` and `text-accent`, with text `admin` and `hr` respectively

#### Scenario: Roles cell shows dash when empty
- **WHEN** a user `alex` is rendered AND `user_role_assignments["alex"]` is empty or absent
- **THEN** the row's roles cell SHALL render `i18n.t(Key::Unlinked)`

#### Scenario: No Status column
- **WHEN** the Benutzer tab table is rendered
- **THEN** the rendered HTML SHALL NOT contain a column header with text matching `Status` (case-insensitive) and SHALL NOT contain row cells whose only purpose is a status dot

#### Scenario: Edit button navigates to user details
- **WHEN** the Edit button on a Benutzer row is rendered
- **THEN** it SHALL be a `Btn` Secondary wrapped in a navigation that targets `Route::UserDetails { user_id }` for that row's user

### Requirement: Per-tab search filter
Each tab SHALL render a search input above its table. The SalesPerson tab's input SHALL filter rows by case-insensitive substring match against `sales_person.name`. The Benutzer tab's input SHALL filter rows by case-insensitive substring match against `user.username`. The filter SHALL apply on every keystroke without requiring submit. The input SHALL display the placeholder text returned by `Key::SearchPlaceholder` and SHALL use the form-input token classes (resolving to `bg-surface`, `text-ink`, `border-border-strong`, `rounded-md`).

#### Scenario: SalesPerson search filters rows
- **WHEN** the SalesPerson tab is active, the table contains entries `Lena`, `Lena Müller`, and `Tom`, and the user types `lena`
- **THEN** the rendered table SHALL contain rows for `Lena` and `Lena Müller` and SHALL NOT contain a row for `Tom`

#### Scenario: Benutzer search filters rows
- **WHEN** the Benutzer tab is active, the table contains usernames `alex`, `Alex2`, and `bob`, and the user types `ALE`
- **THEN** the rendered table SHALL contain rows for `alex` and `Alex2` and SHALL NOT contain a row for `bob`

#### Scenario: Empty search shows all rows
- **WHEN** the search input value is empty
- **THEN** the active tab's table SHALL render all rows in the loaded data

#### Scenario: Search uses the placeholder translation
- **WHEN** the page renders in any locale
- **THEN** each search input's `placeholder` attribute SHALL equal `i18n.t(Key::SearchPlaceholder)`

### Requirement: Add-user dialog replaces inline form
The Benutzer tab SHALL render a `Btn` Primary above its table whose label equals `i18n.t(Key::AddNewUser)`. Clicking the button SHALL open a `Dialog` (variant `Auto`, width 420). The dialog body SHALL contain a `Field` wrapping a `FormTextInput` for the new username. The dialog footer SHALL contain a `Btn` Secondary Cancel and a `Btn` Primary whose label equals `i18n.t(Key::CreateUser)`. Submitting the form SHALL dispatch the existing `UserManagementAction::AddUser(name)` action and close the dialog. The page SHALL NOT render an inline username `<input>` outside this dialog.

#### Scenario: Click on Add-User button opens dialog
- **WHEN** the user clicks the `Btn` Primary labeled `i18n.t(Key::AddNewUser)` while the Benutzer tab is active
- **THEN** a `Dialog` SHALL render with title text matching the add-user title key, body containing a `Field` with a `FormTextInput`, and footer containing both `Btn` Secondary Cancel and `Btn` Primary Create

#### Scenario: Submit dispatches AddUser action
- **WHEN** the user enters `alex` into the dialog input and clicks the Create button
- **THEN** the page SHALL dispatch `UserManagementAction::AddUser("alex".into())` and SHALL close the dialog

#### Scenario: No inline add-user input outside dialog
- **WHEN** the page is rendered with the dialog closed
- **THEN** the rendered HTML SHALL NOT contain a free-standing `<input>` for a new username outside the dialog body

### Requirement: Delete-user confirm dialog replaces direct trash click
A trash button on each Benutzer row SHALL open a confirm `Dialog` (variant `Auto`, width 420) before deletion. The dialog title SHALL render `i18n.t(Key::DeleteUserConfirmTitle)`. The dialog body SHALL render `i18n.t(Key::DeleteUserConfirmBody)` with the user's username interpolated. The dialog footer SHALL contain a `Btn` Secondary Cancel and a `Btn` Danger whose label equals `i18n.t(Key::DeleteUser)`. Clicking the Danger button SHALL dispatch `UserManagementAction::DeleteUser(name)` and close the dialog.

#### Scenario: Trash button opens confirm dialog
- **WHEN** the user clicks the trash button on a row for user `alex`
- **THEN** a `Dialog` SHALL render with body text containing `alex` and a footer with both Cancel and Danger Delete buttons

#### Scenario: Confirm Delete dispatches DeleteUser
- **WHEN** the user clicks the Danger Delete button inside the confirm dialog for user `alex`
- **THEN** the page SHALL dispatch `UserManagementAction::DeleteUser("alex".into())` and SHALL close the dialog

#### Scenario: Cancel closes dialog without dispatching
- **WHEN** the user clicks Cancel inside the confirm dialog
- **THEN** the dialog SHALL close AND no `UserManagementAction::DeleteUser` action SHALL be dispatched

### Requirement: Inactive sales-person rows visually de-emphasized
On the SalesPerson tab, rows whose `sales_person.inactive == true` SHALL render with a class resolving to `opacity-60` and SHALL include an `Inactive` pill near the type pill. The inactive pill SHALL include classes resolving to `bg-bad-soft` and `text-bad`. Inactive rows SHALL still appear in the table (not hidden) so they remain reachable for reactivation.

#### Scenario: Inactive row carries opacity-60
- **WHEN** a sales person with `inactive = true` is rendered
- **THEN** the row's outer element SHALL include a class resolving to `opacity-60`

#### Scenario: Inactive pill uses bad tokens
- **WHEN** an inactive sales person row is rendered
- **THEN** the row SHALL contain an `Inactive` pill with classes resolving to `bg-bad-soft` and `text-bad`

### Requirement: Preload actions populate derived store fields
The page SHALL preload the following derived data after the master lists arrive: a `sales_person_user_links` map keyed by `Uuid`, a `user_sales_person_links` map keyed by username, and a `user_role_assignments` map keyed by username. Three new `UserManagementAction` variants SHALL exist (`LoadAllSalesPersonUserLinks`, `LoadAllUserSalesPersonLinks`, `LoadAllUserRoles`). Each action SHALL fan out the existing per-entity loader function for every loaded entity and write the aggregated map into the corresponding store field. Existing action semantics SHALL NOT change for the `users` and `sales_persons` master lists themselves; the new fields SHALL be populated additively. Per-entity errors during fan-out SHALL skip the affected entry rather than abort the entire batch.

#### Scenario: LoadAllSalesPersons triggers link preload on success
- **WHEN** `LoadAllSalesPersons` completes successfully
- **THEN** the action handler SHALL chain `LoadAllSalesPersonUserLinks` so the linked-user map is populated for the visible rows

#### Scenario: LoadAllUsers triggers role and link preloads on success
- **WHEN** `LoadAllUsers` completes successfully
- **THEN** the action handler SHALL chain both `LoadAllUserRoles` and `LoadAllUserSalesPersonLinks`

#### Scenario: Per-entity error skips that entry
- **WHEN** `LoadAllSalesPersonUserLinks` is dispatched and one sales-person fetch fails
- **THEN** the resulting `sales_person_user_links` map SHALL contain entries for the successful fetches and SHALL NOT contain an entry for the failing one, AND the action SHALL still mark the store field as populated rather than leaving it unchanged

#### Scenario: Existing action variants are not removed or renamed
- **WHEN** the diff is inspected after the change
- **THEN** all existing variants of `UserManagementAction` (e.g. `LoadAllUsers`, `LoadAllSalesPersons`, `AddUser`, `DeleteUser`, `LoadSalesPerson`, `SaveSalesPerson`, etc.) SHALL still exist with identical names

### Requirement: Design tokens replace legacy classes in user-management sources
The non-test sources of `src/page/user_management.rs` and `src/component/user_management_tab_bar.rs` SHALL NOT contain any of these legacy Tailwind class substrings: `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `bg-red-`, `border-gray-`, `border-black`. All surface, ink, border, and accent colors SHALL use design-token classes (`bg-surface`, `bg-surface-alt`, `bg-accent-soft`, `text-ink`, `text-ink-muted`, `text-ink-soft`, `border-border`, `border-border-strong`, `border-accent`, `text-accent`, etc.).

#### Scenario: No legacy classes in user-management page source
- **WHEN** the non-test source of `src/page/user_management.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

#### Scenario: No legacy classes in tab-bar component source
- **WHEN** the non-test source of `src/component/user_management_tab_bar.rs` is inspected
- **THEN** it SHALL NOT contain any of the substrings listed in the requirement

### Requirement: New i18n keys for tab content
The `Key` enum SHALL include new variants for the redesigned page: `ColumnLinkedUser`, `ColumnLinkedSalesPerson`, `ColumnRoles`, `ColumnType`, `Unlinked`, `DeleteUserConfirmTitle`, and `DeleteUserConfirmBody`. Each new key SHALL have non-empty translations in all three locales (`en`, `de`, `cs`). The `Unlinked` translation in every locale SHALL be the em-dash character `—` (or its locale-appropriate placeholder if the localizer chooses one).

#### Scenario: All new keys present in all locales
- **WHEN** the i18n locale files are inspected
- **THEN** each of the seven new keys SHALL return a non-empty string in `Locale::En`, `Locale::De`, and `Locale::Cs`

#### Scenario: Unlinked default is em-dash
- **WHEN** `i18n.t(Key::Unlinked)` is called in any locale
- **THEN** the returned string SHALL be `—`

### Requirement: Existing Edit-route flows preserved
Clicking the Edit button on a SalesPerson row SHALL navigate to the existing `Route::SalesPersonDetails { sales_person_id }` route without modifying the destination page. Clicking the Edit button on a Benutzer row SHALL navigate to the existing `Route::UserDetails { user_id }` route without modifying the destination page. Neither destination page is restyled in this change.

#### Scenario: SalesPerson edit links to existing route
- **WHEN** any sales-person row's Edit button is rendered
- **THEN** the rendered HTML SHALL contain an `href` (or equivalent navigation target) resolving to `/sales_person/{id}` (or whatever the existing route format is) for that sales person

#### Scenario: User edit links to existing route
- **WHEN** any user row's Edit button is rendered
- **THEN** the rendered HTML SHALL contain an `href` (or equivalent navigation target) resolving to `/user/{username}` (or whatever the existing route format is) for that user

