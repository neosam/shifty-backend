use dioxus::prelude::*;

use crate::i18n::Key;
use crate::service::i18n::I18N;
use crate::service::ui_prefs::WorkingHoursLayout;

#[derive(PartialEq, Clone, Props)]
pub struct WorkingHoursOverviewLayoutToggleProps {
    pub active: WorkingHoursLayout,
    pub on_change: EventHandler<WorkingHoursLayout>,
}

const PILL_CONTAINER: &str = "inline-flex bg-surface-alt rounded-md p-[2px]";
const ACTIVE_BTN: &str =
    "px-3 py-1 rounded-sm text-small font-semibold bg-surface text-ink shadow-sm cursor-pointer";
const INACTIVE_BTN: &str =
    "px-3 py-1 rounded-sm text-small font-semibold bg-transparent text-ink-muted cursor-pointer hover:text-ink";

pub(crate) fn pill_button_class(active: bool) -> &'static str {
    if active {
        ACTIVE_BTN
    } else {
        INACTIVE_BTN
    }
}

#[component]
pub fn WorkingHoursOverviewLayoutToggle(props: WorkingHoursOverviewLayoutToggleProps) -> Element {
    let i18n = I18N.read().clone();
    let active = props.active;
    let on_change = props.on_change;
    let on_change_table = on_change;

    rsx! {
        div { class: "{PILL_CONTAINER}",
            button {
                r#type: "button",
                class: pill_button_class(active == WorkingHoursLayout::Cards),
                onclick: move |_| on_change.call(WorkingHoursLayout::Cards),
                "{i18n.t(Key::WorkingHoursLayoutCards)}"
            }
            button {
                r#type: "button",
                class: pill_button_class(active == WorkingHoursLayout::Table),
                onclick: move |_| on_change_table.call(WorkingHoursLayout::Table),
                "{i18n.t(Key::WorkingHoursLayoutTable)}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_class_uses_surface_token() {
        let s = pill_button_class(true);
        assert!(s.contains("bg-surface"), "active missing bg-surface: {s}");
        assert!(s.contains("text-ink"), "active missing text-ink: {s}");
    }

    #[test]
    fn inactive_class_uses_muted_token() {
        let s = pill_button_class(false);
        assert!(
            s.contains("text-ink-muted"),
            "inactive missing text-ink-muted: {s}"
        );
        assert!(
            s.contains("bg-transparent"),
            "inactive missing bg-transparent: {s}"
        );
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn toggle_active_cards_marks_cards_button_active() {
        fn app() -> Element {
            rsx! {
                WorkingHoursOverviewLayoutToggle {
                    active: WorkingHoursLayout::Cards,
                    on_change: |_| {},
                }
            }
        }
        let html = render(app);
        // Cards button must carry the active class set; Table button the inactive set.
        let cards_idx = html.find("Cards").expect("Cards label missing");
        let table_idx = html.find("Table").expect("Table label missing");
        assert!(
            cards_idx < table_idx,
            "Cards should appear before Table: {html}"
        );
        // The active button uses `bg-surface`; the inactive uses `bg-transparent`.
        // Both substrings must be present, and `text-ink-muted` must appear once
        // (on the inactive Table button).
        assert!(
            html.contains("bg-surface"),
            "active styling missing: {html}"
        );
        assert!(
            html.contains("bg-transparent"),
            "inactive styling missing: {html}"
        );
        let muted_count = html.matches("text-ink-muted").count();
        assert_eq!(muted_count, 1, "expected one inactive button: {html}");
    }

    #[test]
    fn toggle_active_table_marks_table_button_active() {
        fn app() -> Element {
            rsx! {
                WorkingHoursOverviewLayoutToggle {
                    active: WorkingHoursLayout::Table,
                    on_change: |_| {},
                }
            }
        }
        let html = render(app);
        // Find the position of the active token block (`bg-surface text-ink shadow-sm`)
        // and the inactive (`bg-transparent text-ink-muted`). The active block
        // must come AFTER the Cards label (i.e. on the Table button).
        let cards_pos = html.find("Cards").expect("Cards label missing");
        let active_marker = html
            .find("bg-surface text-ink")
            .expect("active marker missing");
        assert!(
            active_marker > cards_pos,
            "active marker should be on the Table button: {html}"
        );
    }

    #[test]
    fn toggle_buttons_are_type_button() {
        fn app() -> Element {
            rsx! {
                WorkingHoursOverviewLayoutToggle {
                    active: WorkingHoursLayout::Cards,
                    on_change: |_| {},
                }
            }
        }
        let html = render(app);
        let count = html.matches("type=\"button\"").count();
        assert_eq!(
            count, 2,
            "expected two `type=\"button\"` attributes: {html}"
        );
    }
}
