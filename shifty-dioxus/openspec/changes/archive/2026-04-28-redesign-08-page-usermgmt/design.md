## Context

`src/page/user_management.rs` is the entry point of the Benutzerverwaltung area (route `Route::UserManagementPage`). The current implementation:

- Wraps the page in a centered container with a single `<h1>` heading and renders **two parallel sections** under one heading: a list of `User` entities (with an inline "add user" form and trash-can deletion buttons) and a list of `SalesPerson` entities (with a single "Create new sales person" link). Both sections use legacy Tailwind classes (`bg-white`, `bg-gray-50`, `text-gray-800`, `text-blue-600`, `bg-red-100 text-red-800`, `bg-green-100 text-green-800`).
- Loads data via two coroutine actions: `UserManagementAction::LoadAllUsers` populates `USER_MANAGEMENT_STORE.users: Rc<[User]>`; `UserManagementAction::LoadAllSalesPersons` populates `USER_MANAGEMENT_STORE.sales_persons: Rc<[SalesPerson]>`.
- Each user row is a `<Link to=Route::UserDetails { user_id }>` followed by a delete trash icon. Each sales-person row is a `<Link to=Route::SalesPersonDetails { sales_person_id }>` rendering color dot + name + a `💰` emoji (paid) and an `Inactive` pill.
- There is no notion of a linked user on the sales-person row, no roles column, no status column, no per-section search, and no tabs.

The reference design (`design_handoff_shifty/README.md` § 6) reframes the page as a **two-tab** layout:

1. **`SalesPerson` tab** — a table with one row per sales person and four columns:
   - Color dot (10 px circle, `background-color = sales_person.background_color`) + name in `text-ink`.
   - Type pill — `Bezahlt` for `is_paid = true`, `Freiwillig` for `is_paid = false`. Renders via `PersonChip` with the soft accent/warn hex resolved from tokens.
   - Linked user (`Verknüpfter Benutzer`) — the user login linked to this sales person, in `font-mono`, or `—` when unlinked.
   - Edit button — `Btn` Secondary that navigates to `Route::SalesPersonDetails { sales_person_id }`.
2. **`Benutzer` tab** — a table with one row per user and three columns (revised from the proposal — see decision 5):
   - User login in `font-mono text-ink`.
   - Linked sales person — the linked sales person's color dot + name, or `—` when unlinked.
   - Roles — accent-soft pill chips, one per assigned role (e.g. `admin`, `shiftplanner`, `sales`, `hr`).
   - Edit button — `Btn` Secondary that navigates to `Route::UserDetails { user_id }`.

Tabs use the same flat underline styling as the redesigned shiftplan tab bar in change 09: `border-b-2 border-accent text-accent` for the active tab; `border-transparent text-ink-soft hover:text-ink` for inactive tabs. The underline strip (`border-b border-border`) sits beneath the row.

The proposal explicitly notes: "Reuse existing `UserManagementAction` service — no logic change". Existing action semantics SHALL NOT change. New read-only preload actions are introduced for the new columns (decision 4); they fan out existing per-entity loader functions and only populate new store fields. No existing action is renamed, removed, or has its body altered.

### Data shape constraints

- `User` (in `state::user_management`) carries a single `username: ImStr` field. There is no `roles`, no `status`, no `is_active`, no `created_at`. The current store also exposes `user_invitations: Rc<[InvitationResponse]>` per-user (loaded via `LoadUserInvitations`), but invitations are loaded one user at a time and are not part of the table view.
- `SalesPerson` carries `id: Uuid`, `name: ImStr`, `background_color: ImStr`, `is_paid: bool`, `inactive: bool`, `version: Uuid`. No linked-user field on the type itself.
- `USER_MANAGEMENT_STORE.users: Rc<[User]>` and `USER_MANAGEMENT_STORE.sales_persons: Rc<[SalesPerson]>` are the existing master lists.
- The user↔sales-person link is fetchable per-entity: `loader::load_user_for_sales_person(sales_person_id) -> Option<ImStr>` and `loader::load_sales_person_by_user(user) -> Option<SalesPerson>`.
- Roles per user are fetchable via `loader::load_roles_from_user(user) -> Rc<[ImStr]>`.

The store carries no bulk maps for any of these. Building the new tables requires either fan-out per row (N requests when the tab opens) or a backend bulk endpoint. Backend changes are out of scope (proposal). The decision below uses fan-out with parallel `futures::future::join_all`.

### Existing atoms and components

- `Btn` / `BtnVariant` (Primary, Secondary, Ghost, Danger) — Edit buttons (Secondary), action buttons (Primary).
- `PersonChip` — Bezahlt/Freiwillig type pill on the SalesPerson tab; linked-sales-person cell on the Benutzer tab.
- `Dialog` — used for the optional add-user / delete-user / add-sales-person flows if they remain inline (see decision 6). The add-user inline form moves into a `Dialog`.
- `Field` + `FormTextInput` — used inside the add-user dialog.

### Reference HTML

The Benutzerverwaltung screen is implemented in plain HTML inside `design_handoff_shifty/Shifty Preview.html`. Use this file for the visual ground truth on table padding, row hover, tab underline cadence, and pill styling.

## Goals / Non-Goals

**Goals:**
- Rewrite `src/page/user_management.rs` as a two-tab page. Tab state is local (`use_signal::<UserMgmtTab>`), defaulting to `UserMgmtTab::SalesPersons` on first paint.
- Introduce a `UserManagementTabBar` sub-component that renders the two tab buttons and emits an `on_select` event. Reuses the same flat-underline pattern as change 09's shiftplan tab bar but lives as its own small component because the shiftplan tab bar has irrelevant CRUD semantics.
- On the `SalesPerson` tab, render a `<table>` with rows for each non-filtered sales person. Columns: color dot + name, type pill (Bezahlt/Freiwillig), linked user (mono login or `—`), Edit button (`Btn` Secondary navigating to `Route::SalesPersonDetails`).
- On the `Benutzer` tab, render a `<table>` with rows for each user. Columns: user login (mono), linked sales person (color dot + name or `—`), roles (accent-soft pill chips), Edit button (`Btn` Secondary navigating to `Route::UserDetails`).
- Add a search input above each table (per-tab). Filter the SalesPerson tab by case-insensitive substring on `sales_person.name`; filter the Benutzer tab by case-insensitive substring on `user.username`.
- Lift the legacy "Add new user" inline form into a `Dialog`. The existing `Btn` Primary above the Benutzer-tab table opens the dialog. The dialog body uses `Field` + `FormTextInput`; the footer uses `Btn` Secondary Cancel + `Btn` Primary Create.
- Lift the legacy delete-user trash button into a confirm `Dialog` (variant `Auto`, width 420). Trash button on each row opens it; footer carries Cancel + Danger Delete.
- Keep the existing "Create new sales person" entry as a `Btn` Primary above the SalesPerson-tab table that navigates to `Route::SalesPersonDetails { sales_person_id: "" }` (existing flow).
- Preload the linked-user, linked-sales-person, and roles maps when each tab is first activated (or on page mount). Use new `UserManagementAction` actions: `LoadAllSalesPersonUserLinks`, `LoadAllUserSalesPersonLinks`, `LoadAllUserRoles`. These actions populate three new store fields (`sales_person_user_links`, `user_sales_person_links`, `user_role_assignments`); they do not change existing-action semantics.
- All new and rewritten code SHALL use design tokens. The non-test source of `src/page/user_management.rs` SHALL NOT contain any of `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `bg-red-`, `border-gray-`, `border-black` after the rewrite.
- Add new i18n keys: `Key::UserManagementTabSalesPersons`, `Key::UserManagementTabUsers`, `Key::ColumnLinkedUser`, `Key::ColumnLinkedSalesPerson`, `Key::ColumnRoles`, `Key::ColumnType`, `Key::Unlinked`, `Key::DeleteUserConfirmTitle`, `Key::DeleteUserConfirmBody`. Reuse existing `Users`, `SalesPersons`, `Edit`, `Inactive`, `Active`, `Paid`, `Volunteer`, `SearchPlaceholder`, `CreateUser`, `AddNewUser`, `DeleteUser`, `Cancel`, `CreateNewSalesPerson`.

**Non-Goals:**
- No backend changes. No new endpoints, no bulk-fetch APIs. The preload actions are pure fan-out over existing endpoints.
- **No `Status` column on the Benutzer tab** (deviation from proposal — see decision 5). The current data model does not carry a user status field; deriving status from invitations is out of scope. The proposal lists `Status (good/muted dot + label)`; this design omits it and leaves a future-enhancement marker.
- No edit-in-place of the linked user on the SalesPerson tab. Linking/unlinking still happens inside `SalesPersonDetails`.
- No edit-in-place of roles on the Benutzer tab. Role assignments still happen inside `UserDetails`.
- No restyling of `src/page/sales_person_details.rs` or `src/page/user_details.rs` — they keep their current layout. A token-only sweep on those pages happens in the cleanup change (`99`).
- No removal of legacy `Modal`, `Button`, `TextInput` from `base_components.rs` — cleanup change handles deletion.
- No pagination of the tables. Both lists are typically <100 rows; native browser scrolling is fine.
- No inactive-user / inactive-sales-person filter toggle (the existing page also lacks it). Inactive sales persons render with the existing `Inactive` pill in the type column area.
- No bulk role-edit flows.
- No drag-and-drop, no inline rename.

## Decisions

### 1. Tab state lives on the page, not in the URL

Tab state could be expressed via the route (`/user_management/users` vs `/user_management/sales_persons`) or via a `use_signal` local to the page.

**Chosen: local signal.** Reasons: (a) the existing route is `/user_management/`, no tab segment; adding a route segment ripples into the router and introduces deep-link surface that no other consumer uses; (b) the tabs are inspection-only — there is no detail page reachable from a tab that needs the URL to remember which tab the user came from (Edit buttons go to existing routes); (c) the proposal does not require URL persistence.

Implementation: `let mut active_tab = use_signal(|| UserMgmtTab::SalesPersons);` at the top of `UserManagementPage`. Tab buttons call `active_tab.set(...)`. The default is `SalesPersons` because the existing page renders that section first and the redesign preserves the default landing context.

### 2. Tab styling matches the shiftplan tab bar

Both tabs are buttons (not links) inside a `<div class="flex border-b border-border mb-4 items-center">`. Each tab's `<button>` carries:

- Active: `px-4 py-2 text-sm font-medium border-b-2 border-accent text-accent`
- Inactive: `px-4 py-2 text-sm font-medium border-b-2 border-transparent text-ink-soft hover:text-ink hover:border-border-strong`

The underline lives on the tab buttons (each carries `border-b-2`). The container's `border-b border-border` provides the full underline strip beneath any non-rendered area. This mirrors the shiftplan tab bar in change 09 — the only difference is the absence of CRUD affordances next to the tabs.

The tab bar is its own component (`src/component/user_management_tab_bar.rs`) for testability — SSR tests can render just the tab bar with both states. Two reasons not to reuse `ShiftplanTabBar` directly: (a) `ShiftplanTabBar` carries dialog-mode state and CRUD affordances irrelevant to user-management; (b) the user-management tabs are a fixed set of two, not a dynamic list keyed by `Uuid`.

### 3. Table layout — `<table>` element with token classes

Two layout options were considered:

| Option | UX |
|---|---|
| CSS Grid (`display: grid; grid-template-columns: ...`) | Tighter control over column widths; no row-table semantics |
| Native `<table>`, `<thead>`, `<tbody>`, `<tr>`, `<td>` | Better screen-reader semantics; standard column alignment |

**Chosen: native `<table>`.** Reasons: (a) the rows are tabular data (entity per row, attribute per column), which is the canonical use case; (b) screen readers announce headers correctly; (c) Tailwind tokens map cleanly to `<th>`/`<td>` via class attributes.

Layout classes:

```html
<table class="w-full text-left">
  <thead class="border-b border-border text-xs uppercase tracking-wide text-ink-muted">
    <tr>
      <th class="py-2 px-3 font-semibold">{Key::SalesPersons | Users}</th>
      <th class="py-2 px-3 font-semibold">{Key::ColumnType | ColumnLinkedSalesPerson}</th>
      ...
      <th class="py-2 px-3 font-semibold w-[1%]"></th>  <!-- Edit column header is empty; w-[1%] shrinks -->
    </tr>
  </thead>
  <tbody>
    <tr class="border-b border-border hover:bg-surface-alt">
      <td class="py-2 px-3">...</td>
      ...
    </tr>
  </tbody>
</table>
```

The Edit-button column header is empty; its `w-[1%]` keeps the column tight without a fixed pixel width.

Inactive sales persons render with an opacity-60 row and a small `Inactive` pill next to the type pill. This preserves the existing visual cue for inactive sales persons in the data.

### 4. Preload strategy for derived columns

**Three new actions** added to `UserManagementAction`:

- `LoadAllSalesPersonUserLinks` — populates `sales_person_user_links: HashMap<Uuid, Option<ImStr>>`.
- `LoadAllUserSalesPersonLinks` — populates `user_sales_person_links: HashMap<ImStr, Option<SalesPerson>>`.
- `LoadAllUserRoles` — populates `user_role_assignments: HashMap<ImStr, Rc<[ImStr]>>`.

Each action body fans out the existing per-entity loader using `futures::future::join_all`, writes the resulting map into the store, and (on per-entity error) skips that row's entry rather than failing the entire batch. Per-entity errors still log via `ERROR_STORE` for visibility.

Sequence:

1. On `use_effect` mount, dispatch `LoadAllUsers` and `LoadAllSalesPersons` (existing).
2. After both lists arrive (signaled by both store fields becoming non-empty or by chaining inside the action handler), dispatch the three preload actions.
3. The page renders rows immediately with whatever map data is present; entries missing from the maps render as `—` until the preload completes. This keeps the first-paint fast and the eventual-paint complete.

**Sub-decision**: chain the preloads inside the load action bodies (i.e. `LoadAllUsers` triggers `LoadAllUserSalesPersonLinks` and `LoadAllUserRoles` on success; `LoadAllSalesPersons` triggers `LoadAllSalesPersonUserLinks` on success). This avoids the page having to track readiness manually. The chain lives inside the `user_management_service` match arms.

This is the reason the preload actions are "additive" rather than "no logic change": the existing `LoadAllUsers` and `LoadAllSalesPersons` action bodies grow a tail-call into the new actions. Existing action **outputs** (the populated `users` and `sales_persons` store fields) are unchanged; the new fields are written after the existing fields. Documented as a deviation from "no logic change".

**Alternative considered**: trigger preloads only on tab activation (the user clicks the Benutzer tab → fan out roles). Rejected because (a) tab switches feel laggy if the data doesn't load until activation; (b) the page is HR-only and the row counts are small; the cost of always-preloading is acceptable.

**Alternative considered**: add a backend bulk endpoint. Rejected — backend changes are explicitly out of scope.

### 5. Drop the Status column

The proposal lists `Status (good/muted dot + label)` for the Benutzer tab. The current `User` data shape only carries `username`. There is no status field, no `is_active`, no `last_login`, and the only proxy (open invitations via `load_user_invitations`) is a per-user lookup that doesn't yield a binary status.

**Decision**: omit the Status column. Document in the proposal-deviation section. If status becomes a real concept later (backend exposes a flag, or invitations gain a "pending vs accepted" classifier), add the column then.

This shrinks the Benutzer tab to **three** content columns + one Edit column. The proposal listed four content columns; this design ships three. The decision is recorded under "Decisions" so the reviewer doesn't expect a Status column.

### 6. Lift add-user form into a Dialog

The current page mounts an inline `<TextInput>` + `<Button>` row directly above the user list for adding users. The redesign moves this into a dedicated `Dialog` (variant `Auto`, width 420):

- A `Btn` Primary labeled `i18n.t(Key::AddNewUser)` sits above the table on the Benutzer tab.
- Click opens the dialog. Body: a `Field` containing a `FormTextInput` for the username. Footer: `Btn` Secondary Cancel + `Btn` Primary Create.
- Submit dispatches the existing `UserManagementAction::AddUser(name)` (no logic change).

Reasons: (a) inline form clutters the table-above-table layout with an unrelated control; (b) the dialog gives the form room for client-side validation messages later; (c) consistent with how other CRUD lives inside dialogs in this redesign series.

### 7. Lift delete-user trash into a confirm Dialog

Same rationale as decision 6. The trash button on a user row opens a confirm `Dialog` (variant `Auto`, width 420):

- Title: `i18n.t(Key::DeleteUserConfirmTitle)`.
- Body: a paragraph using `i18n.t(Key::DeleteUserConfirmBody)` with the user's username interpolated.
- Footer: `Btn` Secondary Cancel + `Btn` Danger Delete. Delete dispatches `UserManagementAction::DeleteUser(name)` (existing).

Without confirmation, the existing flow can delete a user with a single click, which is risky for HR. Adding confirmation is a small UX improvement that fits the dialog migration.

### 8. Linked-user / linked-sales-person cell rendering

**SalesPerson tab — Linked User cell:**
- Read from `sales_person_user_links.get(&sales_person.id)`.
- If the entry is `Some(Some(login))`, render `<span class="font-mono text-ink">{login}</span>`.
- If the entry is `Some(None)`, render `<span class="text-ink-muted">—</span>`.
- If the entry is missing (preload not yet complete), also render `—`. The cell becomes the login on next render once the map populates. Loading skeletons are unnecessary at this row count.

**Benutzer tab — Linked SalesPerson cell:**
- Read from `user_sales_person_links.get(&user.username)`.
- If `Some(Some(sp))`, render `<div class="flex items-center gap-2">` containing a 10 px color dot (inline `background-color: {sp.background_color}`) + `<span class="text-ink">{sp.name}</span>`. **No initials, no avatar.** This mirrors the list-row dot in change 07.
- If `Some(None)` or missing, render `—`.

**Benutzer tab — Roles cell:**
- Read from `user_role_assignments.get(&user.username)`.
- For each role string, render `<span class="inline-flex px-2 py-0.5 rounded-sm text-xs font-medium bg-accent-soft text-accent">{role}</span>` with `gap-1` between siblings.
- If the entry is missing or empty, render `<span class="text-ink-muted">—</span>`.

These three cells share a common shape (read from a map keyed by id/username, fall back to `—`) so a small helper `or_dash<T>(opt: Option<T>) -> Element` keeps the RSX terse. The helper lives in the page file (private).

### 9. Per-tab search

A `use_signal(|| String::new())` lives at the page top for each tab (`sales_persons_search`, `users_search`). The search input above each table reads/writes its respective signal.

Filter logic on the SalesPerson tab:

```rust
sales_persons.iter().filter(|sp| {
    let q = sales_persons_search.read().to_lowercase();
    q.is_empty() || sp.name.to_lowercase().contains(&q)
})
```

Filter logic on the Benutzer tab uses `user.username.to_lowercase().contains(&q)`.

The search inputs use the existing `form-input`-equivalent token classes (matching the search field in change 07's employees list).

Empty search → all rows. No debounce. No URL persistence.

### 10. i18n: minimal new keys

Nine new keys cover the redesign:

- `Key::UserManagementTabSalesPersons` — "Sales Persons" / "Verkäufer:innen" / "Prodejci"
- `Key::UserManagementTabUsers` — "Users" / "Benutzer" / "Uživatelé"

  *(Or reuse the existing `Key::SalesPersons` and `Key::Users` for the tab labels — see below.)*

- `Key::ColumnLinkedUser` — "Linked user" / "Verknüpfter Benutzer" / "Propojený uživatel"
- `Key::ColumnLinkedSalesPerson` — "Linked sales person" / "Verknüpfter Verkäufer" / "Propojený prodejce"
- `Key::ColumnRoles` — "Roles" / "Rollen" / "Role"
- `Key::ColumnType` — "Type" / "Typ" / "Typ"
- `Key::Unlinked` — "—" (literal em-dash; same in all locales) — used as the empty-cell glyph. Keeping it as an i18n key (rather than a hard-coded string) lets localizers swap to a culturally appropriate placeholder if needed.
- `Key::DeleteUserConfirmTitle` — "Delete user" / "Benutzer löschen" / "Smazat uživatele"
- `Key::DeleteUserConfirmBody` — "Are you sure you want to delete user {username}? This cannot be undone." / German + Czech equivalents

**Sub-decision**: reuse `Key::SalesPersons` and `Key::Users` for the tab labels rather than introducing tab-specific keys. The two existing keys already render with the correct copy in all three locales. The "tab-specific" key prefix (`UserManagementTab*`) was considered for stylistic separation but the savings are marginal. Rejected; reuse the existing keys.

Final new-key count: seven (`ColumnLinkedUser`, `ColumnLinkedSalesPerson`, `ColumnRoles`, `ColumnType`, `Unlinked`, `DeleteUserConfirmTitle`, `DeleteUserConfirmBody`).

### 11. PersonChip color resolution for the type pill

The Bezahlt/Freiwillig pill uses `PersonChip` with the soft-color hex resolved at the call site:

- Paid (`is_paid = true`): `PersonChip { name: i18n.t(Key::Paid), color: Some(ImStr::from("#eaecfb")), ... }` — the `--accent-soft` light-mode hex.
- Volunteer (`is_paid = false`): `PersonChip { name: i18n.t(Key::Volunteer), color: Some(ImStr::from("#fef0d6")), ... }` — the `--warn-soft` light-mode hex.

This mirrors the type-pill resolution in change 07's employee detail header. The pinned hex values are accepted for forward-compat; theme rework would move them once. Documented.

### 12. Inactive sales persons

The existing page hides `is_paid` rendering for inactive sales persons (effectively de-emphasizing them) and shows an `Inactive` pill. The redesign:

- Renders an `Inactive` pill inline next to the type pill on the SalesPerson tab when `sales_person.inactive == true`. The pill uses `bg-bad-soft text-bad` tokens, matching the destructive-action color family. Keeps the row visible (don't hide inactive rows), so they can be reactivated.
- Optionally fades the entire row to `opacity-60` for inactive sales persons.

The existing data is sufficient; no new field needed.

### 13. File-level plan

**New files:**
- `src/component/user_management_tab_bar.rs` — the two-tab underline tab bar component.

**Modified files:**
- `src/page/user_management.rs` — full rewrite (proposal scope).
- `src/state/user_management.rs` (in `service::user_management` next to `UserManagementStore`) — add three new map fields. The struct additions are tiny.
- `src/service/user_management.rs` — add three new action variants and their handler arms; add a new helper function for each preload (mirroring the existing single-entity helpers).
- `src/component/mod.rs` — re-export `UserManagementTabBar`.
- `src/i18n/mod.rs`, `src/i18n/en.rs`, `src/i18n/de.rs`, `src/i18n/cs.rs` — seven new keys.

Estimated diff: ~600 LOC change, ~250 LOC net addition.

### 14. Test strategy

Three tiers, mirroring change 07:

**Unit tests (pure functions):**
- `filter_case_insensitive_substring` — exercise the search helper with mixed-case inputs.
- `or_dash_returns_dash_for_none` — the helper renders `—` for absent map entries.
- `tab_button_classes_active` — active-tab class string contains `border-accent` and `text-accent`.
- `tab_button_classes_inactive` — inactive class string contains `border-transparent` and `text-ink-soft`.

**SSR tests (rendered HTML assertions):**
- Tab bar: renders two `<button>` elements; the active one carries `border-accent text-accent`; the inactive one carries `border-transparent text-ink-soft`.
- SalesPerson tab table: 4 columns rendered (color-dot + name, type pill, linked user, edit); type pill uses `person-pill`; linked-user cell is `font-mono` when present, `—` when missing; edit button is a `Btn` Secondary linking to `Route::SalesPersonDetails`.
- Benutzer tab table: 4 columns rendered (login, linked sales person, roles, edit); login is `font-mono`; linked-sales-person cell carries the dot inline style and the name; roles cell carries one `bg-accent-soft text-accent` chip per role; edit button is a `Btn` Secondary linking to `Route::UserDetails`.
- Search filter SSR: rendering the page with a search term filters rows by case-insensitive substring; both tab tables respect their respective signal.
- Empty search: all rows render.
- Add-user dialog: `Btn` Primary above the Benutzer table opens a `Dialog`; dialog body contains a `Field` with a `FormTextInput`; footer contains Cancel + Create buttons.
- Delete-user dialog: trash button on a user row opens a `Dialog`; footer contains Cancel + a Danger-variant Delete `Btn`.
- Tab switch: rendering with `active_tab=SalesPersons` does not contain the Benutzer table headers; vice versa.
- Inactive sales person: row carries `opacity-60` and an `Inactive` pill with `bg-bad-soft text-bad`.
- Token sweep: the non-test source of `src/page/user_management.rs` and `src/component/user_management_tab_bar.rs` SHALL NOT contain `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `bg-red-`, `border-gray-`, `border-black`.

Tests live next to their source under `#[cfg(test)] mod tests`.

## Risks / Trade-offs

**[N+1 fan-out for the linked-user / linked-sales-person / roles preload]** → For 50 users with 4 role lookups each = 50 requests; for 50 sales persons = 50 requests. Backend is HR-internal, latencies are typically <50 ms per request, and `join_all` runs them in parallel. Mitigation: parallelize via `futures::future::join_all`. If row counts grow into the hundreds and observed latency becomes a problem, add a backend bulk endpoint in a follow-up change. Acceptable for the current scale.

**[Status column dropped from Benutzer tab]** → Proposal lists Status; data model does not support it. Mitigation: explicitly documented under decision 5; future enhancement when backend adds a status field. Reviewer must accept the deviation.

**[Tab state is not URL-persistent]** → Reloading the page lands on the SalesPerson tab regardless of the user's last selection. Mitigation: acceptable trade-off — see decision 1. If feedback shows users repeatedly navigate to the Benutzer tab, persist via `sessionStorage` (no URL pollution) in a follow-up.

**[Edit button on each row navigates to existing detail pages, which have legacy styling]** → The `SalesPersonDetails` and `UserDetails` pages will still render in the legacy look until the cleanup change `99` sweeps them. Mitigation: documented; visually inconsistent for one release cycle, but the tables themselves are tokenized and the navigation flow is preserved.

**[Pinning the soft-color hex values in two places]** → `#eaecfb` and `#fef0d6` already pinned in change 07's employee detail header; this design pins them again. If the design tokens move, both pins drift in lockstep. Mitigation: tolerate; both pins follow the same theme-rework risk as the original.

**[New preload actions chain inside existing actions]** → `LoadAllUsers` and `LoadAllSalesPersons` now have tail-calls into the new actions. A consumer of the existing actions that **only** needed the master list (e.g. another page that briefly renders the user list) pays the cost of also fetching roles and links it doesn't need. Mitigation: the only consumer of `UserManagementAction::LoadAllUsers` outside this page today is none (verified by `grep`); same for `LoadAllSalesPersons` (one or two other pages may dispatch it, but none of them consume the new map fields, so the extra fetches are harmless overhead). If a new consumer wants the lists without the maps, factor the chain into a separate orchestration action.

**[Roles render as `accent-soft` chips, but role semantics differ (`admin` is privileged, `sales` is not)]** → All four standard roles render with the same chip color. Mitigation: acceptable — the chip carries the role name in plain text; visual distinction is unnecessary in a small table. If role-criticality coloring becomes desirable, derive a per-role chip variant later.

**[Inline confirm dialog on user delete adds a friction point]** → Today's UI deletes on a single trash-button click. Adding a confirm dialog is a UX safety improvement but breaks muscle memory for users who delete frequently. Mitigation: low risk — user deletion is an HR-only flow that happens rarely; the confirmation is short (one click of the Danger button) and prevents the more painful accidental deletion.

**[Preload may finish out of order]** → If `LoadAllSalesPersons` finishes before `LoadAllSalesPersonUserLinks`, the SalesPerson table renders without linked-user values for a frame, then re-renders with them. Mitigation: acceptable; the empty cell shows `—` until the map populates, and the re-render is fast. No spinner.
