## 1. i18n keys

- [x] 1.1 Add `Key::ColumnLinkedUser`, `Key::ColumnLinkedSalesPerson`, `Key::ColumnRoles`, `Key::ColumnType`, `Key::Unlinked`, `Key::DeleteUserConfirmTitle`, and `Key::DeleteUserConfirmBody` variants to the `Key` enum in `src/i18n/mod.rs` (placed in a new `// User management page` block near the existing user-management keys)
- [x] 1.2 Add English translations in `src/i18n/en.rs`: `ColumnLinkedUser = "Linked user"`, `ColumnLinkedSalesPerson = "Linked sales person"`, `ColumnRoles = "Roles"`, `ColumnType = "Type"`, `Unlinked = "—"`, `DeleteUserConfirmTitle = "Delete user"`, `DeleteUserConfirmBody = "Are you sure you want to delete user {username}? This cannot be undone."`
- [x] 1.3 Add German translations in `src/i18n/de.rs`: `ColumnLinkedUser = "Verknüpfter Benutzer"`, `ColumnLinkedSalesPerson = "Verknüpfter Verkäufer"`, `ColumnRoles = "Rollen"`, `ColumnType = "Typ"`, `Unlinked = "—"`, `DeleteUserConfirmTitle = "Benutzer löschen"`, `DeleteUserConfirmBody = "Soll Benutzer {username} wirklich gelöscht werden? Dieser Vorgang kann nicht rückgängig gemacht werden."`
- [x] 1.4 Add Czech translations in `src/i18n/cs.rs`: `ColumnLinkedUser = "Propojený uživatel"`, `ColumnLinkedSalesPerson = "Propojený prodejce"`, `ColumnRoles = "Role"`, `ColumnType = "Typ"`, `Unlinked = "—"`, `DeleteUserConfirmTitle = "Smazat uživatele"`, `DeleteUserConfirmBody = "Opravdu chcete smazat uživatele {username}? Tuto akci nelze vrátit zpět."`
- [x] 1.5 Add unit test `i18n_user_management_keys_present_in_all_locales` asserting all seven new keys return non-empty strings in En/De/Cs

## 2. Store and service preload actions

- [x] 2.1 In `src/service/user_management.rs`, add three new fields to `UserManagementStore`: `sales_person_user_links: Rc<HashMap<Uuid, Option<ImStr>>>`, `user_sales_person_links: Rc<HashMap<ImStr, Option<SalesPerson>>>`, `user_role_assignments: Rc<HashMap<ImStr, Rc<[ImStr]>>>` (default to empty maps)
- [x] 2.2 Add three new variants to `UserManagementAction`: `LoadAllSalesPersonUserLinks`, `LoadAllUserSalesPersonLinks`, `LoadAllUserRoles`
- [x] 2.3 Implement helper `load_all_sales_person_user_links()` that iterates `USER_MANAGEMENT_STORE.read().sales_persons`, calls `loader::load_user_for_sales_person` for each id via `futures::future::join_all`, and writes the resulting map into `sales_person_user_links` (per-entity errors log via `ERROR_STORE` but do not abort the batch)
- [x] 2.4 Implement helper `load_all_user_sales_person_links()` that iterates `USER_MANAGEMENT_STORE.read().users`, calls `loader::load_sales_person_by_user` for each username via `futures::future::join_all`, and writes the resulting map into `user_sales_person_links`
- [x] 2.5 Implement helper `load_all_user_roles()` that iterates `USER_MANAGEMENT_STORE.read().users`, calls `loader::load_roles_from_user` for each username via `futures::future::join_all`, and writes the resulting map into `user_role_assignments`
- [x] 2.6 Add a match arm for each of the three new actions calling the corresponding helper and propagating errors via `Err(_)` (handled by the existing `Err(err) => *ERROR_STORE.write() = ...` block)
- [x] 2.7 Modify the existing `LoadAllUsers` arm to chain `load_all_user_sales_person_links()` and `load_all_user_roles()` after `load_all_users()` succeeds (sequential within the arm to keep the chain visible)
- [x] 2.8 Modify the existing `LoadAllSalesPersons` arm to chain `load_all_sales_person_user_links()` after `load_all_sales_persons()` succeeds
- [x] 2.9 Add unit tests for the helpers: with mocked loaders, an empty store yields empty maps; a per-entity loader error skips that entry but populates the rest; the resulting map keys match the input ids/usernames

## 3. UserManagementTabBar component

- [x] 3.1 Create `src/component/user_management_tab_bar.rs` exporting a `UserMgmtTab` enum (`SalesPersons`, `Users`) and a `UserManagementTabBar` component with props `active: UserMgmtTab` and `on_select: EventHandler<UserMgmtTab>`
- [x] 3.2 Render the bar as `<div class="flex border-b border-border mb-4 items-center">` containing two `<button>` elements
- [x] 3.3 Each button uses the active class string `"px-4 py-2 text-sm font-medium border-b-2 border-accent text-accent"` when its tab matches `active`, otherwise `"px-4 py-2 text-sm font-medium border-b-2 border-transparent text-ink-soft hover:text-ink hover:border-border-strong"`
- [x] 3.4 The `SalesPersons` button label uses `i18n.t(Key::SalesPersons)`; the `Users` button label uses `i18n.t(Key::Users)`
- [x] 3.5 Each button's `onclick` calls `on_select.call(<that tab>)`
- [x] 3.6 Add SSR tests: rendering with `active=SalesPersons` yields one `border-accent text-accent` button (the SalesPersons one) and one `border-transparent text-ink-soft` button (the Users one); vice versa for `active=Users`
- [x] 3.7 Add unit test for active/inactive class strings: the active variant SHALL contain `border-accent` and `text-accent`; the inactive variant SHALL contain `border-transparent` and `text-ink-soft`
- [x] 3.8 Re-export `UserManagementTabBar` and `UserMgmtTab` from `src/component/mod.rs`

## 4. Rewrite UserManagementPage

- [x] 4.1 In `src/page/user_management.rs`, replace the existing body with a fresh `UserManagementPage` component
- [x] 4.2 Add page-local state: `let mut active_tab = use_signal(|| UserMgmtTab::SalesPersons);`, `let mut sales_persons_search = use_signal(|| String::new());`, `let mut users_search = use_signal(|| String::new());`, `let mut show_add_user_dialog = use_signal(|| false);`, `let mut delete_user_confirm: Signal<Option<ImStr>> = use_signal(|| None);`
- [x] 4.3 Use `use_effect` to dispatch `UserManagementAction::LoadAllUsers` and `UserManagementAction::LoadAllSalesPersons` (existing pattern). The chained preloads land via the service-arm modifications from §2.7/§2.8
- [x] 4.4 Render `<TopBar />` followed by a centered container (`<div class="px-4 py-4 md:px-6 lg:px-8 max-w-5xl mx-auto">`)
- [x] 4.5 Render the page heading `<h1 class="text-2xl md:text-3xl font-bold mb-6 text-ink">{Key::UserManagement}</h1>`
- [x] 4.6 Render `<UserManagementTabBar active={*active_tab.read()} on_select=move |t| active_tab.set(t) />`
- [x] 4.7 Conditionally render the SalesPerson tab content when `*active_tab.read() == UserMgmtTab::SalesPersons`, otherwise the Benutzer tab content
- [x] 4.8 Add a small helper `or_dash(label_opt: Option<ImStr>) -> Element` private to the file that returns the `Unlinked` translation when `None` and the label as `<span class="font-mono text-ink">` when `Some`

## 5. SalesPerson tab table

- [x] 5.1 Above the SalesPerson table, render a flex row with the search `<input>` (form-input token classes, `placeholder = i18n.t(Key::SearchPlaceholder)`, `oninput` setting `sales_persons_search`) and a `Btn` Primary labeled `i18n.t(Key::CreateNewSalesPerson)` whose click navigates via `<Link>` wrapper to `Route::SalesPersonDetails { sales_person_id: "".into() }`
- [x] 5.2 Render `<table class="w-full text-left">` with `<thead class="border-b border-border text-xs uppercase tracking-wide text-ink-muted">` containing four `<th class="py-2 px-3 font-semibold">` cells: `Key::SalesPersons`, `Key::ColumnType`, `Key::ColumnLinkedUser`, and an empty header cell `<th class="w-[1%]"></th>` for the Edit column
- [x] 5.3 Filter the loaded `sales_persons` by case-insensitive substring match against `sales_person.name` using `sales_persons_search`
- [x] 5.4 For each remaining sales person, render `<tr class="border-b border-border hover:bg-surface-alt">` (add `opacity-60` to the class when `sales_person.inactive`)
- [x] 5.5 First cell — color dot + name: `<td class="py-2 px-3"><div class="flex items-center gap-2"><span class="w-2.5 h-2.5 rounded-full" style="background-color: {sp.background_color}"/><span class="text-ink">{sp.name}</span></div></td>` (no inner text in the dot)
- [x] 5.6 Second cell — type pill (and inactive pill when applicable): `<td class="py-2 px-3"><div class="flex items-center gap-2"><PersonChip name={paid_or_volunteer_label} color={Some(soft_hex)} />{ if sp.inactive { rsx!{ <span class="inline-flex px-2 py-0.5 rounded-sm text-xs font-medium bg-bad-soft text-bad">{Key::Inactive}</span> } } }</div></td>` where `paid_or_volunteer_label` is `i18n.t(Key::Paid)` for `sp.is_paid` and `i18n.t(Key::Volunteer)` otherwise; `soft_hex` is `ImStr::from("#eaecfb")` for paid and `ImStr::from("#fef0d6")` for volunteer
- [x] 5.7 Third cell — linked user: `<td class="py-2 px-3">{ user_management.sales_person_user_links.get(&sp.id).cloned().flatten().map(|u| rsx!{ <span class="font-mono text-ink">{u}</span> }).unwrap_or_else(|| rsx!{ <span class="text-ink-muted">{Key::Unlinked}</span> }) }</td>`
- [x] 5.8 Fourth cell — Edit button: `<td class="py-2 px-3 w-[1%]"><Link to=Route::SalesPersonDetails { sales_person_id: sp.id.to_string() }><Btn variant=BtnVariant::Secondary>{Key::Edit}</Btn></Link></td>`
- [x] 5.9 Add SSR tests covering: 4-column row structure (count `<td>` per row); color dot has no text content; type pill renders `Bezahlt` or `Freiwillig` with `person-pill` class; linked-user cell shows login when present and `—` when absent; inactive row carries `opacity-60` and the bad-soft inactive pill; edit button links to the correct route

## 6. Benutzer tab table

- [x] 6.1 Above the Benutzer table, render a flex row with the search `<input>` (form-input token classes, `placeholder = i18n.t(Key::SearchPlaceholder)`, `oninput` setting `users_search`) and a `Btn` Primary labeled `i18n.t(Key::AddNewUser)` whose click sets `show_add_user_dialog` to `true`
- [x] 6.2 Render `<table class="w-full text-left">` with `<thead class="border-b border-border text-xs uppercase tracking-wide text-ink-muted">` containing four `<th class="py-2 px-3 font-semibold">` cells: `Key::Users`, `Key::ColumnLinkedSalesPerson`, `Key::ColumnRoles`, and an empty header cell `<th class="w-[1%]"></th>` for the Edit column. **Do NOT render a Status column header**
- [x] 6.3 Filter the loaded `users` by case-insensitive substring match against `user.username` using `users_search`
- [x] 6.4 For each remaining user, render `<tr class="border-b border-border hover:bg-surface-alt">`
- [x] 6.5 First cell — login: `<td class="py-2 px-3 font-mono text-ink">{user.username}</td>`
- [x] 6.6 Second cell — linked sales person: read `user_management.user_sales_person_links.get(&user.username).cloned().flatten()`. When present, render `<td><div class="flex items-center gap-2"><span class="w-2.5 h-2.5 rounded-full" style="background-color: {sp.background_color}"/><span class="text-ink">{sp.name}</span></div></td>` (no inner text in the dot). When absent or `None`, render `<td><span class="text-ink-muted">{Key::Unlinked}</span></td>`
- [x] 6.7 Third cell — roles: read `user_management.user_role_assignments.get(&user.username)`. When present and non-empty, render `<td class="py-2 px-3"><div class="flex flex-wrap gap-1">` followed by one `<span class="inline-flex px-2 py-0.5 rounded-sm text-xs font-medium bg-accent-soft text-accent">{role}</span>` per role. When empty/absent, render `<td><span class="text-ink-muted">{Key::Unlinked}</span></td>`
- [x] 6.8 Fourth cell — actions row: `<td class="py-2 px-3 w-[1%]"><div class="flex gap-2"><Link to=Route::UserDetails { user_id: user.username.to_string() }><Btn variant=BtnVariant::Secondary>{Key::Edit}</Btn></Link><Btn variant=BtnVariant::Danger on_click=move |_| delete_user_confirm.set(Some(user.username.clone()))>🗑</Btn></div></td>`
- [x] 6.9 Add SSR tests covering: 4-column row structure (no Status column header); login rendered with `font-mono`; linked-sales-person cell renders dot inline-style and name when linked, em-dash otherwise; roles cell renders one `bg-accent-soft text-accent` chip per role; edit button targets `Route::UserDetails`; trash button is a `Btn` Danger
- [x] 6.10 Add SSR test asserting the rendered HTML contains no `<th>` whose text matches `Status` (case-insensitive)

## 7. Add-user dialog

- [x] 7.1 In `src/page/user_management.rs`, add a sibling render: `if *show_add_user_dialog.read() { Dialog { open: true, on_close: move |_| show_add_user_dialog.set(false), title: ImStr::from(i18n.t(Key::AddNewUser).as_ref()), variant: DialogVariant::Auto, width: 420, footer: Some(rsx!{ Btn { variant: BtnVariant::Secondary, on_click: move |_| show_add_user_dialog.set(false), {Key::Cancel} } Btn { variant: BtnVariant::Primary, on_click: <submit handler>, {Key::CreateUser} } }), AddUserDialogBody { ... } } }`
- [x] 7.2 The dialog body uses `Field { label: i18n.t(Key::AddNewUser), input: rsx!{ FormTextInput { value: ..., on_change: ... } } }` (single field for the new username)
- [x] 7.3 The submit handler reads the field value, dispatches `UserManagementAction::AddUser(value)`, clears the field, and sets `show_add_user_dialog` to `false`
- [x] 7.4 Add SSR tests: with `show_add_user_dialog=true`, rendering contains a Dialog title, a `Field` with a `FormTextInput`, and footer buttons Cancel + Create; with the signal `false`, no Dialog renders
- [x] 7.5 Add SSR test asserting the rendered HTML when the dialog is closed does not contain a free-standing `<input>` for a new username outside any Dialog component

## 8. Delete-user confirm dialog

- [x] 8.1 In `src/page/user_management.rs`, add a sibling render: `if let Some(username) = delete_user_confirm.read().clone() { Dialog { open: true, on_close: move |_| delete_user_confirm.set(None), title: ImStr::from(i18n.t(Key::DeleteUserConfirmTitle).as_ref()), variant: DialogVariant::Auto, width: 420, footer: Some(rsx!{ Btn { variant: BtnVariant::Secondary, on_click: move |_| delete_user_confirm.set(None), {Key::Cancel} } Btn { variant: BtnVariant::Danger, on_click: <delete handler>, {Key::DeleteUser} } }), <body>{i18n.t(Key::DeleteUserConfirmBody).replace("{username}", &username)}</body> } }`
- [x] 8.2 The delete handler dispatches `UserManagementAction::DeleteUser(username.clone())` and sets `delete_user_confirm` to `None`
- [x] 8.3 Add SSR tests: with `delete_user_confirm=Some("alex")`, rendering contains a Dialog whose body interpolates `alex` and a footer with Cancel + Danger Delete buttons; with `delete_user_confirm=None`, no Dialog renders
- [x] 8.4 Add SSR test asserting the rendered HTML when the trash button is clicked (signal set) shows the confirm Dialog before any DeleteUser dispatch happens

## 9. Token sweep tests

- [x] 9.1 Add SSR test `user_management_page_no_legacy_classes`: read the non-test source of `src/page/user_management.rs` and assert it contains none of `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `bg-red-`, `border-black`, `border-gray-`
- [x] 9.2 Add SSR test `user_management_tab_bar_no_legacy_classes`: same assertion against `src/component/user_management_tab_bar.rs`

## 10. Verification

- [x] 10.1 `cargo check --package shifty-dioxus` passes
- [x] 10.2 `cargo test --package shifty-dioxus` passes; all new tests in §1, §2, §3, §5, §6, §7, §8, §9 are green
- [x] 10.3 `cargo clippy --no-deps --package shifty-dioxus` produces no new warnings in any of the new or modified files
- [x] 10.4 `cargo fmt -- --check` passes
- [ ] 10.5 Manual smoke (Tailwind watcher + `dx serve`) on `/user_management/`: SalesPerson tab is the default; tab switch swaps the visible table; per-tab search filters live; Edit buttons navigate to existing detail pages; the legacy single-h1 layout is gone
- [ ] 10.6 Manual smoke on Add-User dialog: Btn Primary opens it; Cancel closes without a dispatch; Create dispatches `AddUser` and closes
- [ ] 10.7 Manual smoke on Delete-User confirm dialog: trash button opens it; Cancel closes without a dispatch; Danger Delete dispatches `DeleteUser` and closes
- [ ] 10.8 Manual smoke on linked columns: SalesPerson tab shows linked logins (or `—`); Benutzer tab shows linked sales-person dots+names (or `—`) and role chips
- [x] 10.9 `openspec validate "redesign-08-page-usermgmt" --strict` passes
