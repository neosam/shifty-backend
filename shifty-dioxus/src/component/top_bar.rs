use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    base_types::ImStr,
    component::dropdown_base::DropdownTrigger,
    i18n::Key,
    loader,
    router::Route,
    service::{
        auth::AUTH,
        config::CONFIG,
        i18n::I18N,
        theme::{cycle_theme, ThemeAction, ThemeMode, THEME_MODE},
    },
    state::{dropdown::DropdownEntry, AuthInfo},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NavVisibility {
    pub shiftplan: bool,
    pub my_shifts: bool,
    pub my_time: bool,
    pub year_overview: bool,
    pub employees: bool,
    pub billing_periods: bool,
    pub user_management: bool,
    pub templates: bool,
}

pub(crate) fn nav_visibility(auth_info: Option<&AuthInfo>, is_paid: bool) -> NavVisibility {
    let has = |p: &str| auth_info.map(|a| a.has_privilege(p)).unwrap_or(false);
    let show_reports = has("hr");
    NavVisibility {
        shiftplan: has("sales") || has("shiftplanner"),
        my_shifts: has("sales"),
        my_time: is_paid && !show_reports,
        year_overview: has("shiftplanner") || has("sales"),
        employees: show_reports,
        billing_periods: show_reports,
        user_management: has("admin"),
        templates: has("admin"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NavTarget {
    Shiftplan,
    MyShifts,
    MyTime,
    YearOverview,
    Employees,
    BillingPeriods,
    UserManagement,
    Templates,
}

pub(crate) fn is_active_for(target: NavTarget, route: &Route) -> bool {
    match target {
        NavTarget::Shiftplan => {
            matches!(route, Route::ShiftPlan {} | Route::ShiftPlanDeep { .. })
        }
        NavTarget::MyShifts => matches!(route, Route::MyShifts {}),
        NavTarget::MyTime => matches!(route, Route::MyEmployeeDetails {}),
        NavTarget::YearOverview => matches!(route, Route::WeeklyOverview {}),
        NavTarget::Employees => {
            matches!(route, Route::Employees {} | Route::EmployeeDetails { .. })
        }
        NavTarget::BillingPeriods => matches!(
            route,
            Route::BillingPeriods {} | Route::BillingPeriodDetails { .. }
        ),
        NavTarget::UserManagement => matches!(
            route,
            Route::UserManagementPage {}
                | Route::UserDetails { .. }
                | Route::SalesPersonDetails { .. }
        ),
        NavTarget::Templates => matches!(route, Route::TextTemplateManagement {}),
    }
}

pub(crate) fn nav_item_class(active: bool) -> &'static str {
    if active {
        "px-3 py-1.5 rounded-md bg-accent-soft text-accent text-body font-semibold"
    } else {
        "px-3 py-1.5 rounded-md text-ink-soft text-body font-medium hover:bg-surface-alt"
    }
}

pub(crate) fn admin_panel_item_class(active: bool) -> &'static str {
    if active {
        "block w-full text-left px-2.5 py-2 rounded-md bg-accent-soft text-accent text-body font-semibold"
    } else {
        "block w-full text-left px-2.5 py-2 rounded-md text-ink text-body hover:bg-surface-alt"
    }
}

pub(crate) const MOBILE_ADMIN_SECTION_HEADER_CLASS: &str =
    "mt-1 pt-3 px-3.5 pb-1 text-micro font-bold uppercase text-ink-muted border-t border-border";

pub(crate) const ADMIN_PANEL_CONTAINER_CLASS: &str =
    "min-w-[220px] bg-surface border border-border rounded-md shadow-md p-1 z-50";

pub(crate) fn is_admin_target(target: NavTarget) -> bool {
    matches!(
        target,
        NavTarget::Employees
            | NavTarget::BillingPeriods
            | NavTarget::UserManagement
            | NavTarget::Templates
    )
}

pub(crate) fn partition_nav_items<T: Clone>(
    items: &[(NavTarget, T, String)],
) -> (Vec<(NavTarget, T, String)>, Vec<(NavTarget, T, String)>) {
    let mut top_level = Vec::new();
    let mut admin = Vec::new();
    for entry in items {
        if is_admin_target(entry.0) {
            admin.push(entry.clone());
        } else {
            top_level.push(entry.clone());
        }
    }
    (top_level, admin)
}

pub(crate) fn active_admin_label<'a, T>(
    admin_items: &'a [(NavTarget, T, String)],
    route: &Route,
) -> Option<&'a str> {
    admin_items
        .iter()
        .find(|(target, _, _)| is_active_for(*target, route))
        .map(|(_, _, label)| label.as_str())
}

pub(crate) fn theme_glyph(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "☀",
        ThemeMode::Dark => "☾",
        ThemeMode::System => "⌬",
    }
}

pub(crate) fn theme_aria_label(mode: ThemeMode) -> String {
    format!("Theme: {}", mode.as_str())
}

pub(crate) fn burger_glyph(visible: bool) -> &'static str {
    if visible {
        "✕"
    } else {
        "☰"
    }
}

pub(crate) fn logout_url(backend_url: &str) -> String {
    format!("{}/logout", backend_url)
}

const ADMIN_TRIGGER_ID: &str = "top-bar-admin-trigger";
const ADMIN_PANEL_ID: &str = "top-bar-admin-panel";

#[cfg(target_arch = "wasm32")]
struct AdminDropdownGuard {
    mousedown_closure: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MouseEvent)>,
    keydown_closure: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::KeyboardEvent)>,
}

#[cfg(target_arch = "wasm32")]
impl Drop for AdminDropdownGuard {
    fn drop(&mut self) {
        use wasm_bindgen::JsCast;
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            let _ = document.remove_event_listener_with_callback(
                "mousedown",
                self.mousedown_closure.as_ref().unchecked_ref(),
            );
            let _ = document.remove_event_listener_with_callback(
                "keydown",
                self.keydown_closure.as_ref().unchecked_ref(),
            );
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn install_admin_dropdown_listeners(
    mut admin_open: Signal<bool>,
) -> Option<Rc<AdminDropdownGuard>> {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let document = web_sys::window()?.document()?;

    let mousedown_closure: Closure<dyn FnMut(web_sys::MouseEvent)> =
        Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if !*admin_open.read() {
                return;
            }
            let target_node = event
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Node>().ok());
            if let (Some(document), Some(target)) =
                (web_sys::window().and_then(|w| w.document()), target_node)
            {
                let trigger_hits = document
                    .get_element_by_id(ADMIN_TRIGGER_ID)
                    .map(|el| el.contains(Some(&target)))
                    .unwrap_or(false);
                let panel_hits = document
                    .get_element_by_id(ADMIN_PANEL_ID)
                    .map(|el| el.contains(Some(&target)))
                    .unwrap_or(false);
                if trigger_hits || panel_hits {
                    return;
                }
            }
            admin_open.set(false);
        }));

    let keydown_closure: Closure<dyn FnMut(web_sys::KeyboardEvent)> =
        Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            if !*admin_open.read() {
                return;
            }
            if event.key() == "Escape" {
                admin_open.set(false);
            }
        }));

    let _ = document
        .add_event_listener_with_callback("mousedown", mousedown_closure.as_ref().unchecked_ref());
    let _ = document
        .add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref());

    Some(Rc::new(AdminDropdownGuard {
        mousedown_closure,
        keydown_closure,
    }))
}

#[cfg(not(target_arch = "wasm32"))]
fn install_admin_dropdown_listeners(_admin_open: Signal<bool>) -> Option<Rc<()>> {
    None
}

#[cfg(target_arch = "wasm32")]
fn read_trigger_anchor() -> Option<(f64, f64)> {
    let document = web_sys::window()?.document()?;
    let element = document.get_element_by_id(ADMIN_TRIGGER_ID)?;
    let rect = element.get_bounding_client_rect();
    Some((rect.bottom() + 4.0, rect.left()))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_trigger_anchor() -> Option<(f64, f64)> {
    None
}

#[component]
pub fn TopBar() -> Element {
    let auth_info = AUTH.read().auth_info.clone();
    if auth_info.is_some() {
        rsx! { TopBarRouted {} }
    } else {
        rsx! { TopBarLanding {} }
    }
}

#[component]
fn TopBarRouted() -> Element {
    let i18n = I18N.read().clone();
    let auth_info = AUTH.read().auth_info.clone();
    let config = CONFIG.read().clone();
    let backend_url: Rc<str> = config.backend.clone();
    let non_production_warning_str = i18n.t(Key::NonProdWarning);
    let non_production_warning_detail_str = i18n.t(Key::NonProdWarningDetails);
    let you_are_label = i18n.t(Key::TopBarYouAreLabel);
    let logout_label = i18n.t(Key::Logout);
    let login_label = i18n.t(Key::Login);

    let employee = {
        let config = config.clone();
        use_resource(move || loader::load_current_sales_person(config.to_owned()))
    };
    let is_paid = if let Some(Ok(Some(employee))) = &*employee.read_unchecked() {
        employee.is_paid
    } else {
        false
    };

    let visibility = nav_visibility(auth_info.as_ref(), is_paid);
    let route = use_route::<Route>();

    let mut visible = use_signal(|| false);

    let theme_service = use_coroutine_handle::<ThemeAction>();
    let theme_mode = *THEME_MODE.read();
    let theme_glyph_str: ImStr = ImStr::from(theme_glyph(theme_mode));
    let theme_aria: ImStr = ImStr::from(theme_aria_label(theme_mode).as_str());

    use_effect(move || {
        let _ = use_route::<Route>();
        visible.set(false);
    });

    let nav_items: Vec<(NavTarget, Route, String)> = {
        let mut items = Vec::new();
        if visibility.shiftplan {
            items.push((
                NavTarget::Shiftplan,
                Route::ShiftPlan {},
                i18n.t(Key::Shiftplan).to_string(),
            ));
        }
        if visibility.my_shifts {
            items.push((
                NavTarget::MyShifts,
                Route::MyShifts {},
                i18n.t(Key::MyShifts).to_string(),
            ));
        }
        if visibility.my_time {
            items.push((
                NavTarget::MyTime,
                Route::MyEmployeeDetails {},
                i18n.t(Key::MyTime).to_string(),
            ));
        }
        if visibility.year_overview {
            items.push((
                NavTarget::YearOverview,
                Route::WeeklyOverview {},
                i18n.t(Key::YearOverview).to_string(),
            ));
        }
        if visibility.employees {
            items.push((
                NavTarget::Employees,
                Route::Employees {},
                i18n.t(Key::Employees).to_string(),
            ));
        }
        if visibility.billing_periods {
            items.push((
                NavTarget::BillingPeriods,
                Route::BillingPeriods {},
                i18n.t(Key::BillingPeriods).to_string(),
            ));
        }
        if visibility.user_management {
            items.push((
                NavTarget::UserManagement,
                Route::UserManagementPage {},
                i18n.t(Key::UserManagement).to_string(),
            ));
        }
        if visibility.templates {
            items.push((
                NavTarget::Templates,
                Route::TextTemplateManagement {},
                i18n.t(Key::TextTemplateManagement).to_string(),
            ));
        }
        items
    };

    let logout_entries: Rc<[DropdownEntry]> = {
        let backend_url = backend_url.clone();
        let logout_label_owned: ImStr = ImStr::from(logout_label.as_ref());
        Rc::from([DropdownEntry::from((
            logout_label_owned,
            move |_ctx: Option<Rc<str>>| {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href(&logout_url(&backend_url));
                }
            },
        ))])
    };

    let (top_level_items, admin_items) = partition_nav_items(&nav_items);
    let admin_visible = !admin_items.is_empty();
    let active_admin_label_owned: Option<String> =
        active_admin_label(&admin_items, &route).map(|s| s.to_string());
    let admin_active = active_admin_label_owned.is_some();
    let admin_group_default_label = i18n.t(Key::TopBarAdminGroupLabel).to_string();
    let admin_trigger_label = active_admin_label_owned
        .clone()
        .unwrap_or_else(|| admin_group_default_label.clone());

    let mut admin_open = use_signal(|| false);
    let mut admin_anchor = use_signal::<Option<(f64, f64)>>(|| None);
    let _admin_listener_guard = use_hook(|| install_admin_dropdown_listeners(admin_open));

    use_effect(move || {
        let _ = use_route::<Route>();
        admin_open.set(false);
    });

    let burger_glyph_str = burger_glyph(*visible.read());
    let mobile_panel_visible = *visible.read();
    let admin_panel_visible = *admin_open.read();
    let admin_anchor_value = *admin_anchor.read();

    rsx! {
        header {
            class: "sticky top-0 h-14 max-md:h-[52px] bg-surface text-ink border-b border-border z-40 print:hidden flex items-center px-[18px] max-md:px-[10px] gap-1",

            button {
                class: "md:hidden inline-flex items-center justify-center w-[34px] h-[34px] rounded-md border border-border bg-transparent text-ink-soft text-lg flex-shrink-0",
                "aria-label": "Toggle navigation",
                onclick: move |_| {
                    let v = *visible.read();
                    visible.set(!v);
                },
                "{burger_glyph_str}"
            }

            span { class: "text-h2 font-bold tracking-[-0.01em] ml-1 mr-3 max-md:text-lg",
                "Shifty"
                span { class: "text-accent", "." }
                if !config.is_prod {
                    span { class: "ml-2 text-small font-normal text-ink-muted",
                        "{config.env_short_description}"
                    }
                }
            }

            nav { class: "hidden md:flex items-center gap-0.5 flex-1 min-w-0",
                for (target, target_route, label) in top_level_items.iter().cloned() {
                    Link {
                        to: target_route,
                        class: nav_item_class(is_active_for(target, &route)),
                        "{label}"
                    }
                }
                if admin_visible {
                    span { class: "relative inline-block",
                        button {
                            id: ADMIN_TRIGGER_ID,
                            r#type: "button",
                            class: "{nav_item_class(admin_active)} inline-flex items-center gap-1",
                            "aria-haspopup": "menu",
                            "aria-expanded": admin_panel_visible,
                            onclick: move |_| {
                                let was_open = *admin_open.read();
                                if !was_open {
                                    admin_anchor.set(read_trigger_anchor());
                                }
                                admin_open.set(!was_open);
                            },
                            "{admin_trigger_label}"
                            span { class: "text-micro opacity-70 ml-0.5", "▾" }
                        }
                        if admin_panel_visible {
                            if let Some((top, left)) = admin_anchor_value {
                                div {
                                    id: ADMIN_PANEL_ID,
                                    role: "menu",
                                    class: ADMIN_PANEL_CONTAINER_CLASS,
                                    style: "position: fixed; top: {top}px; left: {left}px;",
                                    for (target, target_route, label) in admin_items.iter().cloned() {
                                        Link {
                                            to: target_route,
                                            role: "menuitem",
                                            class: admin_panel_item_class(is_active_for(target, &route)),
                                            onclick: move |_| {
                                                admin_open.set(false);
                                            },
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "ml-auto flex items-center gap-2 flex-shrink-0",
                // Theme button glyph: 15 px is the design's specific icon-glyph size
                // (Shifty Preview.html line 322). It sits between body (14 px) and
                // lg (16 px) — kept as an arbitrary value because it is a glyph
                // size, not a typography role. See
                // openspec/changes/redesign-typography-bump/specs/typography/spec.md.
                button {
                    class: "inline-flex items-center justify-center w-[30px] h-[30px] rounded-md border border-border bg-transparent text-ink-soft text-[15px] flex-shrink-0",
                    "aria-label": theme_aria.as_str(),
                    title: format!("{} (klicken zum Wechseln)", theme_aria_label(theme_mode)),
                    onclick: {
                        let theme_service = theme_service.clone();
                        move |evt: Event<MouseData>| {
                            evt.prevent_default();
                            let next = cycle_theme(*THEME_MODE.read());
                            theme_service.send(ThemeAction::SetMode(next));
                        }
                    },
                    "{theme_glyph_str}"
                }

                if let Some(auth_info) = auth_info.as_ref() {
                    span { class: "text-small font-normal text-ink-muted ml-1 max-md:hidden", "{you_are_label}" }
                    DropdownTrigger {
                        entries: logout_entries.clone(),
                        button {
                            class: "flex items-center gap-2 px-3 py-1 rounded-full bg-surface-alt text-body font-medium text-ink cursor-pointer",
                            "{auth_info.user}"
                        }
                    }
                } else {
                    a {
                        href: "/authenticate",
                        class: "px-3 py-1.5 rounded-md text-ink-soft hover:bg-surface-alt text-body",
                        "{login_label}"
                    }
                }
            }

            if mobile_panel_visible {
                div { class: "md:hidden absolute top-[52px] left-2 right-2 bg-surface border border-border rounded-md shadow-md z-50 p-2 flex flex-col gap-1",
                    for (target, target_route, label) in top_level_items.iter().cloned() {
                        Link {
                            to: target_route,
                            class: nav_item_class(is_active_for(target, &route)),
                            "{label}"
                        }
                    }
                    if admin_visible {
                        div { class: MOBILE_ADMIN_SECTION_HEADER_CLASS,
                            "{admin_group_default_label}"
                        }
                        for (target, target_route, label) in admin_items.iter().cloned() {
                            Link {
                                to: target_route,
                                class: nav_item_class(is_active_for(target, &route)),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }

        if !config.is_prod {
            div { class: "bg-warn-soft text-warn pl-4 p-1 print:hidden",
                div {
                    class: "font-bold",
                    title: "{non_production_warning_detail_str}",
                    "{non_production_warning_str}"
                }
            }
        }
    }
}

/// Header rendered when no user is logged in. Same surface chrome as
/// [`TopBarRouted`] (logo, env-tag, theme toggle), but without the nav
/// items / mobile burger / logout dropdown — and crucially without
/// [`use_route`], so it can be mounted outside of a [`Router`].
#[component]
fn TopBarLanding() -> Element {
    let i18n = I18N.read().clone();
    let config = CONFIG.read().clone();
    let non_production_warning_str = i18n.t(Key::NonProdWarning);
    let non_production_warning_detail_str = i18n.t(Key::NonProdWarningDetails);
    let login_label = i18n.t(Key::Login);

    let theme_service = use_coroutine_handle::<ThemeAction>();
    let theme_mode = *THEME_MODE.read();
    let theme_glyph_str: ImStr = ImStr::from(theme_glyph(theme_mode));
    let theme_aria: ImStr = ImStr::from(theme_aria_label(theme_mode).as_str());
    let theme_title = format!("{} (klicken zum Wechseln)", theme_aria_label(theme_mode));

    rsx! {
        header {
            class: "sticky top-0 h-14 max-md:h-[52px] bg-surface text-ink border-b border-border z-40 print:hidden flex items-center px-[18px] max-md:px-[10px] gap-1",

            span { class: "text-h2 font-bold tracking-[-0.01em] ml-1 mr-3 max-md:text-lg",
                "Shifty"
                span { class: "text-accent", "." }
                if !config.is_prod {
                    span { class: "ml-2 text-small font-normal text-ink-muted",
                        "{config.env_short_description}"
                    }
                }
            }

            div { class: "ml-auto flex items-center gap-2 flex-shrink-0",
                // 15 px theme glyph — same justification as TopBarRouted above.
                button {
                    class: "inline-flex items-center justify-center w-[30px] h-[30px] rounded-md border border-border bg-transparent text-ink-soft text-[15px] flex-shrink-0",
                    "aria-label": theme_aria.as_str(),
                    title: theme_title,
                    onclick: {
                        let theme_service = theme_service.clone();
                        move |evt: Event<MouseData>| {
                            evt.prevent_default();
                            let next = cycle_theme(*THEME_MODE.read());
                            theme_service.send(ThemeAction::SetMode(next));
                        }
                    },
                    "{theme_glyph_str}"
                }

                a {
                    href: "/authenticate",
                    class: "px-3 py-1.5 rounded-md text-ink-soft hover:bg-surface-alt text-body",
                    "{login_label}"
                }
            }
        }

        if !config.is_prod {
            div { class: "bg-warn-soft text-warn pl-4 p-1 print:hidden",
                div {
                    class: "font-bold",
                    title: "{non_production_warning_detail_str}",
                    "{non_production_warning_str}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    fn auth_with(privileges: &[&str]) -> AuthInfo {
        AuthInfo {
            user: "user".into(),
            privileges: privileges
                .iter()
                .map(|p| Rc::<str>::from(*p))
                .collect::<Vec<_>>()
                .into(),
            authenticated: true,
        }
    }

    #[test]
    fn nav_visibility_no_auth_hides_everything() {
        let v = nav_visibility(None, false);
        assert!(!v.shiftplan);
        assert!(!v.my_shifts);
        assert!(!v.my_time);
        assert!(!v.year_overview);
        assert!(!v.employees);
        assert!(!v.billing_periods);
        assert!(!v.user_management);
        assert!(!v.templates);
    }

    #[test]
    fn nav_visibility_sales_shows_shiftplan_my_shifts_year_overview() {
        let auth = auth_with(&["sales"]);
        let v = nav_visibility(Some(&auth), false);
        assert!(v.shiftplan);
        assert!(v.my_shifts);
        assert!(v.year_overview);
        assert!(!v.my_time);
        assert!(!v.employees);
        assert!(!v.user_management);
        assert!(!v.templates);
    }

    #[test]
    fn nav_visibility_shiftplanner_shows_shiftplan_year_overview() {
        let auth = auth_with(&["shiftplanner"]);
        let v = nav_visibility(Some(&auth), false);
        assert!(v.shiftplan);
        assert!(!v.my_shifts);
        assert!(v.year_overview);
        assert!(!v.my_time);
    }

    #[test]
    fn nav_visibility_hr_shows_employees_and_billing_periods() {
        let auth = auth_with(&["hr"]);
        let v = nav_visibility(Some(&auth), false);
        assert!(v.employees);
        assert!(v.billing_periods);
        assert!(!v.shiftplan);
        assert!(!v.my_shifts);
        assert!(!v.year_overview);
        assert!(!v.my_time);
        assert!(!v.user_management);
        assert!(!v.templates);
    }

    #[test]
    fn nav_visibility_admin_shows_user_management_and_templates() {
        let auth = auth_with(&["admin"]);
        let v = nav_visibility(Some(&auth), false);
        assert!(v.user_management);
        assert!(v.templates);
        assert!(!v.shiftplan);
        assert!(!v.employees);
    }

    #[test]
    fn nav_visibility_my_time_requires_paid_and_not_hr() {
        let no_priv = AuthInfo::default();
        let v = nav_visibility(Some(&no_priv), true);
        assert!(v.my_time, "is_paid && !hr should show my_time");

        let v = nav_visibility(Some(&no_priv), false);
        assert!(!v.my_time, "without is_paid, my_time hidden");

        let auth_hr_paid = auth_with(&["hr"]);
        let v = nav_visibility(Some(&auth_hr_paid), true);
        assert!(!v.my_time, "hr suppresses my_time even when paid");
    }

    #[test]
    fn nav_visibility_combined_privileges_union() {
        let auth = auth_with(&["sales", "hr", "admin"]);
        let v = nav_visibility(Some(&auth), true);
        assert!(v.shiftplan);
        assert!(v.my_shifts);
        assert!(!v.my_time, "hr suppresses my_time");
        assert!(v.year_overview);
        assert!(v.employees);
        assert!(v.user_management);
        assert!(v.templates);
    }

    #[test]
    fn is_active_for_shiftplan_matches_both_variants() {
        assert!(is_active_for(NavTarget::Shiftplan, &Route::ShiftPlan {}));
        assert!(is_active_for(
            NavTarget::Shiftplan,
            &Route::ShiftPlanDeep {
                year: 2026,
                week: 17
            }
        ));
        assert!(!is_active_for(NavTarget::Shiftplan, &Route::Home {}));
    }

    #[test]
    fn is_active_for_employees_matches_details() {
        assert!(is_active_for(NavTarget::Employees, &Route::Employees {}));
        assert!(is_active_for(
            NavTarget::Employees,
            &Route::EmployeeDetails {
                employee_id: "abc".to_string()
            }
        ));
        assert!(!is_active_for(NavTarget::Employees, &Route::ShiftPlan {}));
    }

    #[test]
    fn is_active_for_user_management_matches_all_three_variants() {
        assert!(is_active_for(
            NavTarget::UserManagement,
            &Route::UserManagementPage {}
        ));
        assert!(is_active_for(
            NavTarget::UserManagement,
            &Route::UserDetails {
                user_id: "u".into()
            }
        ));
        assert!(is_active_for(
            NavTarget::UserManagement,
            &Route::SalesPersonDetails {
                sales_person_id: "s".into()
            }
        ));
    }

    #[test]
    fn is_active_for_my_shifts_my_time_year_overview_templates() {
        assert!(is_active_for(NavTarget::MyShifts, &Route::MyShifts {}));
        assert!(is_active_for(
            NavTarget::MyTime,
            &Route::MyEmployeeDetails {}
        ));
        assert!(is_active_for(
            NavTarget::YearOverview,
            &Route::WeeklyOverview {}
        ));
        assert!(is_active_for(
            NavTarget::Templates,
            &Route::TextTemplateManagement {}
        ));
    }

    #[test]
    fn is_active_for_unrelated_routes_return_false() {
        assert!(!is_active_for(NavTarget::Shiftplan, &Route::Home {}));
        assert!(!is_active_for(NavTarget::Employees, &Route::MyShifts {}));
    }

    #[test]
    fn nav_item_class_active_uses_accent_soft_and_semibold() {
        let c = nav_item_class(true);
        assert!(c.contains("bg-accent-soft"), "missing accent-soft: {c}");
        assert!(c.contains("text-accent"), "missing text-accent: {c}");
        assert!(c.contains("font-semibold"), "missing font-semibold: {c}");
        assert!(c.contains("rounded-md"));
    }

    #[test]
    fn nav_item_class_inactive_uses_ink_soft_and_hover() {
        let c = nav_item_class(false);
        assert!(c.contains("text-ink-soft"));
        assert!(c.contains("hover:bg-surface-alt"));
        assert!(!c.contains("font-semibold"));
        assert!(!c.contains("bg-accent-soft"));
    }

    #[test]
    fn theme_glyph_matches_mode() {
        assert_eq!(theme_glyph(ThemeMode::Light), "☀");
        assert_eq!(theme_glyph(ThemeMode::Dark), "☾");
        assert_eq!(theme_glyph(ThemeMode::System), "⌬");
    }

    #[test]
    fn theme_aria_label_describes_mode() {
        assert_eq!(theme_aria_label(ThemeMode::Light), "Theme: light");
        assert_eq!(theme_aria_label(ThemeMode::Dark), "Theme: dark");
        assert_eq!(theme_aria_label(ThemeMode::System), "Theme: system");
    }

    #[test]
    fn burger_glyph_swaps_with_visibility() {
        assert_eq!(burger_glyph(false), "☰");
        assert_eq!(burger_glyph(true), "✕");
    }

    #[test]
    fn logout_entry_label_uses_provided_label() {
        let label = ImStr::from("Logout");
        let entry: DropdownEntry = (label.clone(), |_: Option<Rc<str>>| {}).into();
        assert_eq!(entry.text, label);
    }

    #[test]
    fn logout_url_appends_logout_path_to_backend() {
        assert_eq!(
            logout_url("https://api.example.org"),
            "https://api.example.org/logout"
        );
        assert_eq!(logout_url(""), "/logout");
    }

    #[test]
    fn is_admin_target_classifies_each_variant() {
        assert!(!is_admin_target(NavTarget::Shiftplan));
        assert!(!is_admin_target(NavTarget::MyShifts));
        assert!(!is_admin_target(NavTarget::MyTime));
        assert!(!is_admin_target(NavTarget::YearOverview));
        assert!(is_admin_target(NavTarget::Employees));
        assert!(is_admin_target(NavTarget::BillingPeriods));
        assert!(is_admin_target(NavTarget::UserManagement));
        assert!(is_admin_target(NavTarget::Templates));
    }

    fn nav_entry(target: NavTarget, label: &str) -> (NavTarget, Route, String) {
        let route = match target {
            NavTarget::Shiftplan => Route::ShiftPlan {},
            NavTarget::MyShifts => Route::MyShifts {},
            NavTarget::MyTime => Route::MyEmployeeDetails {},
            NavTarget::YearOverview => Route::WeeklyOverview {},
            NavTarget::Employees => Route::Employees {},
            NavTarget::BillingPeriods => Route::BillingPeriods {},
            NavTarget::UserManagement => Route::UserManagementPage {},
            NavTarget::Templates => Route::TextTemplateManagement {},
        };
        (target, route, label.to_string())
    }

    #[test]
    fn partition_nav_items_splits_admin_and_top_level_preserving_order() {
        let items = vec![
            nav_entry(NavTarget::Shiftplan, "Schichtplan"),
            nav_entry(NavTarget::MyShifts, "Meine Schichten"),
            nav_entry(NavTarget::MyTime, "Meine Zeit"),
            nav_entry(NavTarget::YearOverview, "Jahresübersicht"),
            nav_entry(NavTarget::Employees, "Mitarbeiter"),
            nav_entry(NavTarget::BillingPeriods, "Abrechnungszeiträume"),
            nav_entry(NavTarget::UserManagement, "Benutzerverwaltung"),
            nav_entry(NavTarget::Templates, "Textvorlagen"),
        ];
        let (top_level, admin) = partition_nav_items(&items);

        let top_level_targets: Vec<NavTarget> = top_level.iter().map(|e| e.0).collect();
        assert_eq!(
            top_level_targets,
            vec![
                NavTarget::Shiftplan,
                NavTarget::MyShifts,
                NavTarget::MyTime,
                NavTarget::YearOverview,
            ]
        );

        let admin_targets: Vec<NavTarget> = admin.iter().map(|e| e.0).collect();
        assert_eq!(
            admin_targets,
            vec![
                NavTarget::Employees,
                NavTarget::BillingPeriods,
                NavTarget::UserManagement,
                NavTarget::Templates,
            ]
        );
    }

    #[test]
    fn partition_nav_items_no_admin_only_top_level() {
        let items = vec![
            nav_entry(NavTarget::Shiftplan, "Schichtplan"),
            nav_entry(NavTarget::MyShifts, "Meine Schichten"),
        ];
        let (top_level, admin) = partition_nav_items(&items);
        assert_eq!(top_level.len(), 2);
        assert!(admin.is_empty());
    }

    #[test]
    fn partition_nav_items_only_admin() {
        let items = vec![
            nav_entry(NavTarget::Employees, "Mitarbeiter"),
            nav_entry(NavTarget::Templates, "Textvorlagen"),
        ];
        let (top_level, admin) = partition_nav_items(&items);
        assert!(top_level.is_empty());
        assert_eq!(admin.len(), 2);
    }

    #[test]
    fn partition_nav_items_empty_input() {
        let items: Vec<(NavTarget, Route, String)> = Vec::new();
        let (top_level, admin) = partition_nav_items(&items);
        assert!(top_level.is_empty());
        assert!(admin.is_empty());
    }

    #[test]
    fn active_admin_label_returns_active_item_label() {
        let admin = vec![
            nav_entry(NavTarget::Employees, "Mitarbeiter"),
            nav_entry(NavTarget::BillingPeriods, "Abrechnungszeiträume"),
        ];
        let label = active_admin_label(&admin, &Route::Employees {});
        assert_eq!(label, Some("Mitarbeiter"));
    }

    #[test]
    fn active_admin_label_none_when_top_level_route_active() {
        let admin = vec![
            nav_entry(NavTarget::Employees, "Mitarbeiter"),
            nav_entry(NavTarget::Templates, "Textvorlagen"),
        ];
        let label = active_admin_label(&admin, &Route::ShiftPlan {});
        assert_eq!(label, None);
    }

    #[test]
    fn active_admin_label_handles_parameterised_admin_route() {
        let admin = vec![nav_entry(NavTarget::Employees, "Mitarbeiter")];
        let label = active_admin_label(
            &admin,
            &Route::EmployeeDetails {
                employee_id: "abc".to_string(),
            },
        );
        assert_eq!(label, Some("Mitarbeiter"));
    }

    #[test]
    fn admin_panel_item_class_active_uses_accent() {
        let c = admin_panel_item_class(true);
        assert!(c.contains("bg-accent-soft"));
        assert!(c.contains("text-accent"));
        assert!(c.contains("font-semibold"));
    }

    #[test]
    fn admin_panel_item_class_inactive_uses_ink_and_hover() {
        let c = admin_panel_item_class(false);
        assert!(c.contains("text-ink"));
        assert!(c.contains("hover:bg-surface-alt"));
        assert!(!c.contains("font-semibold"));
    }

    fn nav_items_for_visibility(v: NavVisibility) -> Vec<(NavTarget, Route, String)> {
        let mut items = Vec::new();
        if v.shiftplan {
            items.push(nav_entry(NavTarget::Shiftplan, "Schichtplan"));
        }
        if v.my_shifts {
            items.push(nav_entry(NavTarget::MyShifts, "Meine Schichten"));
        }
        if v.my_time {
            items.push(nav_entry(NavTarget::MyTime, "Meine Zeit"));
        }
        if v.year_overview {
            items.push(nav_entry(NavTarget::YearOverview, "Jahresübersicht"));
        }
        if v.employees {
            items.push(nav_entry(NavTarget::Employees, "Mitarbeiter"));
        }
        if v.billing_periods {
            items.push(nav_entry(NavTarget::BillingPeriods, "Abrechnungszeiträume"));
        }
        if v.user_management {
            items.push(nav_entry(NavTarget::UserManagement, "Benutzerverwaltung"));
        }
        if v.templates {
            items.push(nav_entry(NavTarget::Templates, "Textvorlagen"));
        }
        items
    }

    #[test]
    fn sales_only_user_yields_no_admin_group() {
        let auth = auth_with(&["sales"]);
        let v = nav_visibility(Some(&auth), false);
        let items = nav_items_for_visibility(v);
        let (top_level, admin) = partition_nav_items(&items);
        assert!(admin.is_empty(), "sales-only should have no admin items");
        let labels: Vec<&str> = top_level.iter().map(|e| e.2.as_str()).collect();
        assert_eq!(
            labels,
            vec!["Schichtplan", "Meine Schichten", "Jahresübersicht"]
        );
    }

    #[test]
    fn hr_admin_user_partitions_into_top_level_and_full_admin_group() {
        let auth = auth_with(&["sales", "hr", "admin"]);
        let v = nav_visibility(Some(&auth), true);
        let items = nav_items_for_visibility(v);
        let (top_level, admin) = partition_nav_items(&items);

        let top_labels: Vec<&str> = top_level.iter().map(|e| e.2.as_str()).collect();
        assert_eq!(
            top_labels,
            vec!["Schichtplan", "Meine Schichten", "Jahresübersicht"],
            "hr suppresses my_time, top-level shows the rest in declaration order"
        );

        let admin_labels: Vec<&str> = admin.iter().map(|e| e.2.as_str()).collect();
        assert_eq!(
            admin_labels,
            vec![
                "Mitarbeiter",
                "Abrechnungszeiträume",
                "Benutzerverwaltung",
                "Textvorlagen"
            ],
            "all four admin items grouped, in declaration order"
        );
    }

    #[test]
    fn top_level_partition_excludes_admin_items() {
        let auth = auth_with(&["sales", "hr", "admin"]);
        let v = nav_visibility(Some(&auth), false);
        let items = nav_items_for_visibility(v);
        let (top_level, _admin) = partition_nav_items(&items);
        for entry in top_level.iter() {
            assert!(
                !is_admin_target(entry.0),
                "top-level slice must not contain admin target {:?}",
                entry.0
            );
        }
    }

    #[test]
    fn hr_user_admin_group_active_label_is_employees_label_for_employee_route() {
        let auth = auth_with(&["hr"]);
        let v = nav_visibility(Some(&auth), false);
        let items = nav_items_for_visibility(v);
        let (_top_level, admin) = partition_nav_items(&items);
        let label = active_admin_label(&admin, &Route::Employees {});
        assert_eq!(label, Some("Mitarbeiter"));
    }

    #[test]
    fn hr_user_admin_group_default_label_when_top_level_route_active() {
        let auth = auth_with(&["sales", "hr"]);
        let v = nav_visibility(Some(&auth), false);
        let items = nav_items_for_visibility(v);
        let (_top_level, admin) = partition_nav_items(&items);
        let label = active_admin_label(&admin, &Route::ShiftPlan {});
        assert_eq!(label, None);
    }

    #[test]
    fn hr_user_admin_group_active_label_for_employee_details_parameterised_route() {
        let auth = auth_with(&["hr"]);
        let v = nav_visibility(Some(&auth), false);
        let items = nav_items_for_visibility(v);
        let (_top_level, admin) = partition_nav_items(&items);
        let label = active_admin_label(
            &admin,
            &Route::EmployeeDetails {
                employee_id: "abc".to_string(),
            },
        );
        assert_eq!(label, Some("Mitarbeiter"));
    }

    #[test]
    fn mobile_admin_section_header_class_matches_design_typography() {
        let c = MOBILE_ADMIN_SECTION_HEADER_CLASS;
        // text-micro encodes the design's 11 px / 0.06em uppercase eyebrow
        // (see openspec/changes/redesign-typography-bump).
        assert!(c.contains("text-micro"), "missing text-micro: {c}");
        assert!(c.contains("font-bold"), "missing font-bold: {c}");
        assert!(c.contains("uppercase"), "missing uppercase: {c}");
        assert!(c.contains("text-ink-muted"), "missing text-ink-muted: {c}");
        assert!(c.contains("border-t"), "missing border-t: {c}");
        assert!(c.contains("border-border"), "missing border-border: {c}");
    }

    #[test]
    fn admin_panel_container_class_matches_design_panel() {
        let c = ADMIN_PANEL_CONTAINER_CLASS;
        assert!(c.contains("min-w-[220px]"), "missing min-w-[220px]: {c}");
        assert!(c.contains("bg-surface"), "missing bg-surface: {c}");
        assert!(c.contains("border"), "missing border: {c}");
        assert!(c.contains("border-border"), "missing border-border: {c}");
        assert!(c.contains("rounded-md"), "missing rounded-md: {c}");
        assert!(c.contains("shadow-md"), "missing shadow-md: {c}");
        assert!(c.contains("z-50"), "missing z-50: {c}");
    }

    #[test]
    fn admin_trigger_active_class_when_admin_route_active_full_user() {
        let auth = auth_with(&["sales", "hr", "admin"]);
        let v = nav_visibility(Some(&auth), false);
        let items = nav_items_for_visibility(v);
        let (_top_level, admin) = partition_nav_items(&items);
        let admin_active = active_admin_label(&admin, &Route::Employees {}).is_some();
        let class = nav_item_class(admin_active);
        assert!(class.contains("bg-accent-soft"));
        assert!(class.contains("text-accent"));
        assert!(class.contains("font-semibold"));
    }

    #[test]
    fn admin_trigger_inactive_class_when_top_level_route_active() {
        let auth = auth_with(&["sales", "hr", "admin"]);
        let v = nav_visibility(Some(&auth), false);
        let items = nav_items_for_visibility(v);
        let (_top_level, admin) = partition_nav_items(&items);
        let admin_active = active_admin_label(&admin, &Route::ShiftPlan {}).is_some();
        let class = nav_item_class(admin_active);
        assert!(class.contains("text-ink-soft"));
        assert!(!class.contains("bg-accent-soft"));
    }
}
