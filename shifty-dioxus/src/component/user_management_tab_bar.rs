//! `UserManagementTabBar` — flat underline tab bar for the User Management page.
//!
//! Two fixed tabs: `SalesPersons` and `Users`. Active tab carries
//! `border-accent text-accent`; inactive tabs carry
//! `border-transparent text-ink-soft`. The container provides a faint
//! underline strip via `border-b border-border`.

use dioxus::prelude::*;

use crate::{i18n::Key, service::i18n::I18N};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UserMgmtTab {
    SalesPersons,
    Users,
}

const ACTIVE_CLASSES: &str = "px-4 py-2 text-body font-medium border-b-2 border-accent text-accent";
const INACTIVE_CLASSES: &str = "px-4 py-2 text-body font-medium border-b-2 border-transparent text-ink-soft hover:text-ink hover:border-border-strong";

pub(crate) fn tab_class(active: bool) -> &'static str {
    if active {
        ACTIVE_CLASSES
    } else {
        INACTIVE_CLASSES
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct UserManagementTabBarProps {
    pub active: UserMgmtTab,
    pub on_select: EventHandler<UserMgmtTab>,
}

#[component]
pub fn UserManagementTabBar(props: UserManagementTabBarProps) -> Element {
    let i18n = I18N.read().clone();
    let active = props.active;
    let on_select = props.on_select.clone();
    let on_select2 = on_select.clone();

    rsx! {
        div { class: "flex border-b border-border mb-4 items-center",
            button {
                class: tab_class(active == UserMgmtTab::SalesPersons),
                onclick: move |_| on_select.call(UserMgmtTab::SalesPersons),
                "{i18n.t(Key::SalesPersons)}"
            }
            button {
                class: tab_class(active == UserMgmtTab::Users),
                onclick: move |_| on_select2.call(UserMgmtTab::Users),
                "{i18n.t(Key::Users)}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_class_contains_accent_tokens() {
        let s = tab_class(true);
        assert!(
            s.contains("border-accent"),
            "active missing border-accent: {s}"
        );
        assert!(s.contains("text-accent"), "active missing text-accent: {s}");
    }

    #[test]
    fn inactive_class_contains_transparent_and_inksoft() {
        let s = tab_class(false);
        assert!(
            s.contains("border-transparent"),
            "inactive missing border-transparent: {s}"
        );
        assert!(
            s.contains("text-ink-soft"),
            "inactive missing text-ink-soft: {s}"
        );
    }

    #[test]
    fn both_classes_share_layout() {
        for active in [true, false] {
            let s = tab_class(active);
            assert!(s.contains("px-4"));
            assert!(s.contains("py-2"));
            assert!(s.contains("border-b-2"));
            assert!(s.contains("font-medium"));
        }
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn renders_two_buttons_with_active_sales_persons() {
        fn app() -> Element {
            rsx! {
                UserManagementTabBar {
                    active: UserMgmtTab::SalesPersons,
                    on_select: |_| {},
                }
            }
        }
        let html = render(app);
        // Two buttons present.
        assert_eq!(html.matches("<button").count(), 2);
        // Active one carries accent tokens.
        assert!(html.contains("border-accent"));
        assert!(html.contains("text-accent"));
        // Inactive one carries transparent + ink-soft tokens.
        assert!(html.contains("border-transparent"));
        assert!(html.contains("text-ink-soft"));
    }

    #[test]
    fn user_management_tab_bar_no_legacy_classes() {
        let src = include_str!("user_management_tab_bar.rs");
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

    #[test]
    fn renders_two_buttons_with_active_users() {
        fn app() -> Element {
            rsx! {
                UserManagementTabBar {
                    active: UserMgmtTab::Users,
                    on_select: |_| {},
                }
            }
        }
        let html = render(app);
        assert_eq!(html.matches("<button").count(), 2);
        assert!(html.contains("border-accent"));
        assert!(html.contains("text-accent"));
        assert!(html.contains("border-transparent"));
        assert!(html.contains("text-ink-soft"));
    }
}
