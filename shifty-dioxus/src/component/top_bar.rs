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
        "px-3 py-1.5 rounded-md bg-accent-soft text-accent font-semibold"
    } else {
        "px-3 py-1.5 rounded-md text-ink-soft hover:bg-surface-alt"
    }
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

#[component]
pub fn TopBar() -> Element {
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

    let burger_glyph_str = burger_glyph(*visible.read());
    let mobile_panel_visible = *visible.read();

    rsx! {
        header {
            class: "sticky top-0 h-14 max-md:h-[52px] bg-surface text-ink border-b border-border z-40 print:hidden flex items-center px-[18px] max-md:px-[10px] gap-1",

            button {
                class: "md:hidden inline-flex items-center justify-center w-[34px] h-[34px] rounded-md border border-border bg-transparent text-ink-soft text-base flex-shrink-0",
                "aria-label": "Toggle navigation",
                onclick: move |_| {
                    let v = *visible.read();
                    visible.set(!v);
                },
                "{burger_glyph_str}"
            }

            span { class: "text-lg font-bold tracking-[-0.01em] ml-1 mr-3 max-md:text-base",
                "Shifty"
                span { class: "text-accent", "." }
                if !config.is_prod {
                    span { class: "ml-2 text-xs text-ink-muted font-normal",
                        "{config.env_short_description}"
                    }
                }
            }

            nav { class: "hidden md:flex items-center gap-0.5 flex-1 min-w-0",
                for (target, target_route, label) in nav_items.iter().cloned() {
                    Link {
                        to: target_route,
                        class: nav_item_class(is_active_for(target, &route)),
                        "{label}"
                    }
                }
            }

            div { class: "ml-auto flex items-center gap-2 flex-shrink-0",
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
                    span { class: "text-xs text-ink-muted ml-1 max-md:hidden", "{you_are_label}" }
                    DropdownTrigger {
                        entries: logout_entries.clone(),
                        button {
                            class: "flex items-center gap-2 px-3 py-1 rounded-full bg-surface-alt text-[13px] text-ink font-medium cursor-pointer",
                            "{auth_info.user}"
                        }
                    }
                } else {
                    a {
                        href: "/authenticate",
                        class: "px-3 py-1.5 rounded-md text-ink-soft hover:bg-surface-alt text-[13px]",
                        "{login_label}"
                    }
                }
            }

            if mobile_panel_visible {
                div { class: "md:hidden absolute top-[52px] left-2 right-2 bg-surface border border-border rounded-md shadow-md z-50 p-2 flex flex-col gap-1",
                    for (target, target_route, label) in nav_items.iter().cloned() {
                        Link {
                            to: target_route,
                            class: nav_item_class(is_active_for(target, &route)),
                            "{label}"
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
}
