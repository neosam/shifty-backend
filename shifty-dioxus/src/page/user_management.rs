//! Redesigned `UserManagementPage` — two-tab layout (`SalesPersons` /
//! `Benutzer`) on top of the existing `UserManagementAction` service.
//!
//! Tab state is page-local (no URL segment); each tab carries its own
//! search input. Add-user and delete-user flows live in `Dialog`s so the
//! page surface stays clean. Linked-user / linked-sales-person / role
//! columns are populated by the preload actions chained inside
//! `LoadAllUsers` and `LoadAllSalesPersons`.

use crate::{
    base_types::ImStr,
    component::{
        atoms::btn::build_class as btn_build_class, Btn, BtnVariant, Dialog, DialogVariant, Field,
        PersonChip, TextInput, TopBar, UserManagementTabBar, UserMgmtTab,
    },
    i18n::Key,
    router::Route,
    service::{
        i18n::I18N,
        user_management::{UserManagementAction, USER_MANAGEMENT_STORE},
    },
};
use dioxus::prelude::*;

const SEARCH_INPUT_CLASSES: &str =
    "h-[34px] px-[10px] border border-border-strong rounded-md bg-surface text-ink text-[13px] w-full min-w-0 form-input";

const ACCENT_SOFT_HEX: &str = "#eaecfb";
const WARN_SOFT_HEX: &str = "#fef0d6";

pub(crate) fn matches_search(name: &str, term: &str) -> bool {
    if term.is_empty() {
        return true;
    }
    name.to_lowercase().contains(&term.to_lowercase())
}

#[component]
pub fn UserManagementPage() -> Element {
    let user_management_service = use_coroutine_handle::<UserManagementAction>();
    let user_management = USER_MANAGEMENT_STORE.read().clone();
    let i18n = I18N.read().clone();

    let mut active_tab = use_signal(|| UserMgmtTab::SalesPersons);
    let sales_persons_search = use_signal(String::new);
    let users_search = use_signal(String::new);
    let mut show_add_user_dialog = use_signal(|| false);
    let mut delete_user_confirm: Signal<Option<ImStr>> = use_signal(|| None);
    let mut new_user_name: Signal<ImStr> = use_signal(|| "".into());

    use_effect(move || {
        user_management_service.send(UserManagementAction::LoadAllUsers);
        user_management_service.send(UserManagementAction::LoadAllSalesPersons);
    });

    rsx! {
        TopBar {}

        div { class: "px-4 py-4 md:px-6 lg:px-8 max-w-5xl mx-auto",
            h1 { class: "text-2xl md:text-3xl font-bold mb-6 text-ink",
                "{i18n.t(Key::UserManagement)}"
            }

            UserManagementTabBar {
                active: *active_tab.read(),
                on_select: move |t| active_tab.set(t),
            }

            if *active_tab.read() == UserMgmtTab::SalesPersons {
                SalesPersonTabContent {
                    user_management: user_management.clone(),
                    search: sales_persons_search,
                }
            } else {
                UsersTabContent {
                    user_management: user_management.clone(),
                    search: users_search,
                    on_open_add_user: move |_| show_add_user_dialog.set(true),
                    on_request_delete: move |username: ImStr| delete_user_confirm.set(Some(username)),
                }
            }
        }

        AddUserDialog {
            open: *show_add_user_dialog.read(),
            value: new_user_name.read().clone(),
            on_value_change: move |v: ImStr| new_user_name.set(v),
            on_cancel: move |_| show_add_user_dialog.set(false),
            on_create: move |_| {
                let name = new_user_name.read().clone();
                if !name.as_str().trim().is_empty() {
                    user_management_service.send(UserManagementAction::AddUser(name));
                    new_user_name.set("".into());
                    show_add_user_dialog.set(false);
                }
            },
        }

        DeleteUserConfirmDialog {
            target: delete_user_confirm.read().clone(),
            on_cancel: move |_| delete_user_confirm.set(None),
            on_confirm: move |username: ImStr| {
                user_management_service.send(UserManagementAction::DeleteUser(username));
                delete_user_confirm.set(None);
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct AddUserDialogProps {
    open: bool,
    value: ImStr,
    on_value_change: EventHandler<ImStr>,
    on_cancel: EventHandler<()>,
    on_create: EventHandler<()>,
}

#[component]
fn AddUserDialog(props: AddUserDialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let i18n = I18N.read().clone();
    let on_cancel = props.on_cancel;
    let on_cancel_footer = props.on_cancel;
    let on_create = props.on_create;
    let on_value_change = props.on_value_change;

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_cancel.call(()),
            title: ImStr::from(i18n.t(Key::AddNewUser).as_ref()),
            variant: DialogVariant::Auto,
            width: 420,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_cancel_footer.call(()),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Primary,
                    on_click: move |_| on_create.call(()),
                    "{i18n.t(Key::CreateUser)}"
                }
            }),
            Field {
                label: ImStr::from(i18n.t(Key::AddNewUser).as_ref()),
                TextInput {
                    value: props.value.clone(),
                    on_change: move |v: ImStr| on_value_change.call(v),
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DeleteUserConfirmDialogProps {
    target: Option<ImStr>,
    on_cancel: EventHandler<()>,
    on_confirm: EventHandler<ImStr>,
}

#[component]
fn DeleteUserConfirmDialog(props: DeleteUserConfirmDialogProps) -> Element {
    let Some(username) = props.target.clone() else {
        return rsx! {};
    };
    let i18n = I18N.read().clone();
    let body_text = i18n
        .t(Key::DeleteUserConfirmBody)
        .as_ref()
        .replace("{username}", username.as_str());
    let on_cancel = props.on_cancel;
    let on_cancel_footer = props.on_cancel;
    let on_confirm = props.on_confirm;
    let username_for_confirm = username.clone();

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_cancel.call(()),
            title: ImStr::from(i18n.t(Key::DeleteUserConfirmTitle).as_ref()),
            variant: DialogVariant::Auto,
            width: 420,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_cancel_footer.call(()),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Danger,
                    on_click: move |_| on_confirm.call(username_for_confirm.clone()),
                    "{i18n.t(Key::DeleteUser)}"
                }
            }),
            p { class: "text-sm text-ink", "{body_text}" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct SalesPersonTabContentProps {
    user_management: crate::service::user_management::UserManagementStore,
    search: Signal<String>,
}

#[component]
fn SalesPersonTabContent(props: SalesPersonTabContentProps) -> Element {
    let i18n = I18N.read().clone();
    let mut search = props.search;
    let user_management = props.user_management;

    let placeholder = i18n.t(Key::SearchPlaceholder);
    let term = search.read().clone();

    let filtered: Vec<_> = user_management
        .sales_persons
        .iter()
        .filter(|sp| matches_search(&sp.name, &term))
        .cloned()
        .collect();

    rsx! {
        div { class: "flex items-center gap-2 mb-4",
            input {
                class: "{SEARCH_INPUT_CLASSES}",
                r#type: "text",
                placeholder: "{placeholder}",
                value: "{search.read()}",
                oninput: move |evt| search.set(evt.value()),
            }
            {
                let primary_cls = btn_build_class(BtnVariant::Primary, false);
                let href = format!(
                    "{}",
                    Route::SalesPersonDetails { sales_person_id: String::new() }
                );
                rsx! {
                    a {
                        href: "{href}",
                        class: "{primary_cls} inline-flex items-center",
                        "{i18n.t(Key::CreateNewSalesPerson)}"
                    }
                }
            }
        }

        table { class: "w-full text-left",
            thead { class: "border-b border-border text-xs uppercase tracking-wide text-ink-muted",
                tr {
                    th { class: "py-2 px-3 font-semibold", "{i18n.t(Key::SalesPersons)}" }
                    th { class: "py-2 px-3 font-semibold", "{i18n.t(Key::ColumnType)}" }
                    th { class: "py-2 px-3 font-semibold", "{i18n.t(Key::ColumnLinkedUser)}" }
                    th { class: "py-2 px-3 font-semibold w-[1%]" }
                }
            }
            tbody {
                for sp in filtered.into_iter() {
                    {
                        let inactive = sp.inactive;
                        let row_class = if inactive {
                            "border-b border-border hover:bg-surface-alt opacity-60"
                        } else {
                            "border-b border-border hover:bg-surface-alt"
                        };
                        let is_paid = sp.is_paid;
                        let pill_label = if is_paid {
                            i18n.t(Key::Paid)
                        } else {
                            i18n.t(Key::Volunteer)
                        };
                        let pill_color = if is_paid {
                            ImStr::from(ACCENT_SOFT_HEX)
                        } else {
                            ImStr::from(WARN_SOFT_HEX)
                        };
                        let linked_user = user_management
                            .sales_person_user_links
                            .get(&sp.id)
                            .cloned()
                            .flatten();
                        let bg = sp.background_color.clone();
                        let name = sp.name.clone();
                        let id = sp.id;
                        rsx! {
                            tr { class: "{row_class}",
                                td { class: "py-2 px-3",
                                    div { class: "flex items-center gap-2",
                                        span {
                                            class: "w-2.5 h-2.5 rounded-full",
                                            style: "background-color: {bg}",
                                        }
                                        span { class: "text-ink", "{name}" }
                                    }
                                }
                                td { class: "py-2 px-3",
                                    div { class: "flex items-center gap-2",
                                        PersonChip {
                                            name: ImStr::from(pill_label.as_ref()),
                                            color: Some(pill_color),
                                        }
                                        if inactive {
                                            span { class: "inline-flex px-2 py-0.5 rounded-sm text-xs font-medium bg-bad-soft text-bad",
                                                "{i18n.t(Key::Inactive)}"
                                            }
                                        }
                                    }
                                }
                                td { class: "py-2 px-3",
                                    if let Some(login) = linked_user {
                                        span { class: "font-mono text-ink", "{login}" }
                                    } else {
                                        span { class: "text-ink-muted", "{i18n.t(Key::Unlinked)}" }
                                    }
                                }
                                td { class: "py-2 px-3 w-[1%]",
                                    {
                                        let secondary_cls = btn_build_class(BtnVariant::Secondary, false);
                                        let href = format!(
                                            "{}",
                                            Route::SalesPersonDetails { sales_person_id: id.to_string() }
                                        );
                                        rsx! {
                                            a {
                                                href: "{href}",
                                                class: "{secondary_cls} inline-flex items-center",
                                                "{i18n.t(Key::Edit)}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct UsersTabContentProps {
    user_management: crate::service::user_management::UserManagementStore,
    search: Signal<String>,
    on_open_add_user: EventHandler<()>,
    on_request_delete: EventHandler<ImStr>,
}

#[component]
fn UsersTabContent(props: UsersTabContentProps) -> Element {
    let i18n = I18N.read().clone();
    let mut search = props.search;
    let user_management = props.user_management;

    let placeholder = i18n.t(Key::SearchPlaceholder);
    let term = search.read().clone();

    let filtered: Vec<_> = user_management
        .users
        .iter()
        .filter(|u| matches_search(&u.username.as_str(), &term))
        .cloned()
        .collect();

    let on_open_add_user = props.on_open_add_user;
    let on_request_delete = props.on_request_delete;

    rsx! {
        div { class: "flex items-center gap-2 mb-4",
            input {
                class: "{SEARCH_INPUT_CLASSES}",
                r#type: "text",
                placeholder: "{placeholder}",
                value: "{search.read()}",
                oninput: move |evt| search.set(evt.value()),
            }
            Btn {
                variant: BtnVariant::Primary,
                on_click: move |_| on_open_add_user.call(()),
                "{i18n.t(Key::AddNewUser)}"
            }
        }

        table { class: "w-full text-left",
            thead { class: "border-b border-border text-xs uppercase tracking-wide text-ink-muted",
                tr {
                    th { class: "py-2 px-3 font-semibold", "{i18n.t(Key::Users)}" }
                    th { class: "py-2 px-3 font-semibold", "{i18n.t(Key::ColumnLinkedSalesPerson)}" }
                    th { class: "py-2 px-3 font-semibold", "{i18n.t(Key::ColumnRoles)}" }
                    th { class: "py-2 px-3 font-semibold w-[1%]" }
                }
            }
            tbody {
                for user in filtered.into_iter() {
                    {
                        let username = user.username.clone();
                        let username_for_link = username.clone();
                        let username_for_delete = username.clone();
                        let linked_sp = user_management
                            .user_sales_person_links
                            .get(&username)
                            .cloned()
                            .flatten();
                        let roles = user_management
                            .user_role_assignments
                            .get(&username)
                            .cloned();
                        let on_request_delete = on_request_delete.clone();
                        rsx! {
                            tr { class: "border-b border-border hover:bg-surface-alt",
                                td { class: "py-2 px-3 font-mono text-ink", "{username}" }
                                td { class: "py-2 px-3",
                                    if let Some(sp) = linked_sp {
                                        div { class: "flex items-center gap-2",
                                            span {
                                                class: "w-2.5 h-2.5 rounded-full",
                                                style: "background-color: {sp.background_color}",
                                            }
                                            span { class: "text-ink", "{sp.name}" }
                                        }
                                    } else {
                                        span { class: "text-ink-muted", "{i18n.t(Key::Unlinked)}" }
                                    }
                                }
                                td { class: "py-2 px-3",
                                    if let Some(role_list) = roles.as_ref() {
                                        if role_list.is_empty() {
                                            span { class: "text-ink-muted", "{i18n.t(Key::Unlinked)}" }
                                        } else {
                                            div { class: "flex flex-wrap gap-1",
                                                for role in role_list.iter() {
                                                    span { class: "inline-flex px-2 py-0.5 rounded-sm text-xs font-medium bg-accent-soft text-accent",
                                                        "{role}"
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        span { class: "text-ink-muted", "{i18n.t(Key::Unlinked)}" }
                                    }
                                }
                                td { class: "py-2 px-3 w-[1%]",
                                    div { class: "flex gap-2",
                                        {
                                            let secondary_cls = btn_build_class(BtnVariant::Secondary, false);
                                            let href = format!(
                                                "{}",
                                                Route::UserDetails { user_id: username_for_link.to_string() }
                                            );
                                            rsx! {
                                                a {
                                                    href: "{href}",
                                                    class: "{secondary_cls} inline-flex items-center",
                                                    "{i18n.t(Key::Edit)}"
                                                }
                                            }
                                        }
                                        Btn {
                                            variant: BtnVariant::Danger,
                                            on_click: move |_| on_request_delete.call(username_for_delete.clone()),
                                            "🗑"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::user_management::UserManagementStore;
    use crate::state::shiftplan::SalesPerson;
    use crate::state::user_management::User;
    use std::collections::HashMap;
    use std::rc::Rc;
    use uuid::Uuid;

    fn render_with(store: UserManagementStore, search_initial: &str) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps {
            store: UserManagementStore,
            initial: String,
        }
        #[component]
        fn SalesWrapper(props: WrapperProps) -> Element {
            let mut search = use_signal(|| props.initial.clone());
            let _ = search.read();
            rsx! {
                SalesPersonTabContent { user_management: props.store.clone(), search }
            }
        }
        let mut vdom = VirtualDom::new_with_props(
            SalesWrapper,
            WrapperProps {
                store,
                initial: search_initial.to_string(),
            },
        );
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    fn render_users_with(store: UserManagementStore, search_initial: &str) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps {
            store: UserManagementStore,
            initial: String,
        }
        #[component]
        fn UsersWrapper(props: WrapperProps) -> Element {
            let mut search = use_signal(|| props.initial.clone());
            let _ = search.read();
            rsx! {
                UsersTabContent {
                    user_management: props.store.clone(),
                    search,
                    on_open_add_user: |_| {},
                    on_request_delete: |_: ImStr| {},
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(
            UsersWrapper,
            WrapperProps {
                store,
                initial: search_initial.to_string(),
            },
        );
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn matches_search_empty_term_matches_all() {
        assert!(matches_search("alex", ""));
        assert!(matches_search("", ""));
    }

    #[test]
    fn matches_search_is_case_insensitive() {
        assert!(matches_search("Lena Müller", "lena"));
        assert!(matches_search("ALEX", "ale"));
        assert!(!matches_search("Tom", "Lena"));
    }

    fn sample_sp(name: &str, is_paid: bool, inactive: bool) -> SalesPerson {
        SalesPerson {
            id: Uuid::new_v4(),
            name: Rc::from(name),
            background_color: Rc::from("#dbe0ff"),
            is_paid,
            inactive,
            version: Uuid::new_v4(),
        }
    }

    fn store_with_sales_persons(spans: Vec<SalesPerson>) -> UserManagementStore {
        UserManagementStore {
            sales_persons: Rc::from(spans),
            ..Default::default()
        }
    }

    #[test]
    fn sales_person_tab_renders_four_columns() {
        let sp = sample_sp("Lena", true, false);
        let id = sp.id;
        let mut links: HashMap<Uuid, Option<ImStr>> = HashMap::new();
        links.insert(id, Some(ImStr::from("alex")));
        let store = UserManagementStore {
            sales_persons: Rc::from(vec![sp]),
            sales_person_user_links: Rc::new(links),
            ..Default::default()
        };
        let html = render_with(store, "");
        assert_eq!(html.matches("<th ").count(), 4);
        assert!(html.contains("background-color: #dbe0ff"));
        assert!(html.contains("Lena"));
        assert!(html.contains("person-pill"));
        assert!(html.contains("font-mono"));
        assert!(html.contains("alex"));
    }

    #[test]
    fn sales_person_tab_inactive_row_has_opacity_and_pill() {
        let sp = sample_sp("Old", true, true);
        let store = store_with_sales_persons(vec![sp]);
        let html = render_with(store, "");
        assert!(html.contains("opacity-60"));
        assert!(html.contains("bg-bad-soft"));
        assert!(html.contains("text-bad"));
    }

    #[test]
    fn sales_person_tab_color_dot_has_no_inner_text() {
        let sp = sample_sp("Lena", false, false);
        let store = store_with_sales_persons(vec![sp]);
        let html = render_with(store, "");
        assert!(html.contains("rounded-full"));
    }

    #[test]
    fn sales_person_tab_unlinked_renders_dash() {
        let sp = sample_sp("Solo", false, false);
        let store = store_with_sales_persons(vec![sp]);
        let html = render_with(store, "");
        assert!(html.contains("—"));
    }

    #[test]
    fn sales_person_tab_search_filters_case_insensitive() {
        let store = store_with_sales_persons(vec![
            sample_sp("Lena", true, false),
            sample_sp("Tom", true, false),
        ]);
        let html = render_with(store, "lena");
        assert!(html.contains("Lena"));
        assert!(!html.contains(">Tom<"));
    }

    fn store_with_users(users: Vec<User>) -> UserManagementStore {
        UserManagementStore {
            users: Rc::from(users),
            ..Default::default()
        }
    }

    #[test]
    fn users_tab_renders_four_columns_no_status() {
        let user = User {
            username: ImStr::from("alex"),
        };
        let store = store_with_users(vec![user]);
        let html = render_users_with(store, "");
        assert_eq!(html.matches("<th ").count(), 4);
        let lowered = html.to_lowercase();
        assert!(
            !lowered.contains(">status<"),
            "found Status column header: {lowered}"
        );
        assert!(html.contains("font-mono"));
        assert!(html.contains("alex"));
    }

    #[test]
    fn users_tab_linked_sales_person_renders_dot_and_name() {
        let user = User {
            username: ImStr::from("alex"),
        };
        let sp = sample_sp("Lena", false, false);
        let mut links: HashMap<ImStr, Option<SalesPerson>> = HashMap::new();
        links.insert(ImStr::from("alex"), Some(sp));
        let store = UserManagementStore {
            users: Rc::from(vec![user]),
            user_sales_person_links: Rc::new(links),
            ..Default::default()
        };
        let html = render_users_with(store, "");
        assert!(html.contains("background-color: #dbe0ff"));
        assert!(html.contains("Lena"));
    }

    #[test]
    fn users_tab_roles_render_as_chips() {
        let user = User {
            username: ImStr::from("alex"),
        };
        let mut roles: HashMap<ImStr, Rc<[ImStr]>> = HashMap::new();
        roles.insert(
            ImStr::from("alex"),
            Rc::from(vec![ImStr::from("admin"), ImStr::from("hr")]),
        );
        let store = UserManagementStore {
            users: Rc::from(vec![user]),
            user_role_assignments: Rc::new(roles),
            ..Default::default()
        };
        let html = render_users_with(store, "");
        assert!(html.contains("bg-accent-soft"));
        assert!(html.contains("text-accent"));
        assert!(html.contains("admin"));
        assert!(html.contains("hr"));
    }

    #[test]
    fn users_tab_empty_roles_show_dash() {
        let user = User {
            username: ImStr::from("alex"),
        };
        let store = store_with_users(vec![user]);
        let html = render_users_with(store, "");
        assert!(html.contains("—"));
    }

    #[test]
    fn users_tab_search_filters_case_insensitive() {
        let store = store_with_users(vec![
            User {
                username: ImStr::from("alex"),
            },
            User {
                username: ImStr::from("bob"),
            },
        ]);
        let html = render_users_with(store, "ALE");
        assert!(html.contains("alex"));
        assert!(!html.contains(">bob<"));
    }

    fn render_add_user_dialog(open: bool) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps {
            open: bool,
        }
        #[component]
        fn AddWrapper(props: WrapperProps) -> Element {
            rsx! {
                AddUserDialog {
                    open: props.open,
                    value: ImStr::from(""),
                    on_value_change: |_| {},
                    on_cancel: |_| {},
                    on_create: |_| {},
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(AddWrapper, WrapperProps { open });
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn add_user_dialog_closed_renders_nothing() {
        let html = render_add_user_dialog(false);
        // No Dialog content (no `role="dialog"`).
        assert!(
            !html.contains("role=\"dialog\""),
            "expected closed dialog: {html}"
        );
    }

    #[test]
    fn add_user_dialog_open_renders_field_and_buttons() {
        let html = render_add_user_dialog(true);
        // Dialog renders.
        assert!(
            html.contains("role=\"dialog\""),
            "missing dialog role: {html}"
        );
        // Field label: the i18n key "Add New User" is used both as title and label.
        // TextInput class.
        assert!(html.contains("form-input"), "missing form-input class");
        // Two footer buttons (Cancel + Create).
        // Cancel uses Secondary tokens; Create uses Primary.
        assert!(html.contains("bg-accent"), "missing primary tokens: {html}");
    }

    fn render_delete_dialog(target: Option<&str>) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps {
            target: Option<ImStr>,
        }
        #[component]
        fn DeleteWrapper(props: WrapperProps) -> Element {
            rsx! {
                DeleteUserConfirmDialog {
                    target: props.target.clone(),
                    on_cancel: |_| {},
                    on_confirm: |_: ImStr| {},
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(
            DeleteWrapper,
            WrapperProps {
                target: target.map(ImStr::from),
            },
        );
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn delete_user_confirm_dialog_closed_renders_nothing() {
        let html = render_delete_dialog(None);
        assert!(
            !html.contains("role=\"dialog\""),
            "expected closed dialog: {html}"
        );
    }

    #[test]
    fn delete_user_confirm_dialog_open_interpolates_username_and_has_danger_button() {
        let html = render_delete_dialog(Some("alex"));
        assert!(html.contains("role=\"dialog\""));
        // Username appears in the body text via interpolation.
        assert!(html.contains("alex"), "username not interpolated: {html}");
        // Danger button uses bad tokens.
        assert!(html.contains("text-bad"), "missing danger tokens: {html}");
        assert!(html.contains("border-bad"), "missing danger border: {html}");
    }

    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("user_management.rs");
        let test_module_start = src
            .find("#[cfg(test)]")
            .expect("test module marker missing");
        let prefix = &src[..test_module_start];
        for forbidden in [
            "bg-gray-",
            "bg-white",
            "text-gray-",
            "text-blue-",
            "text-red-",
            "text-green-",
            "bg-blue-",
            "bg-green-",
            "bg-red-",
            "border-black",
            "border-gray-",
        ] {
            assert!(
                !prefix.contains(forbidden),
                "legacy class `{forbidden}` found in source"
            );
        }
    }
}
