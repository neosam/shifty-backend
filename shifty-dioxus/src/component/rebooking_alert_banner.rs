//! `RebookingAlertBanner` — Phase 55 HR-ALERT-01.
//!
//! Inline-Banner-Komponente, KEIN Dialog (MEMORY
//! `feedback_warnings_inline_not_dialog`). Sichtbarkeits-Gate ist im
//! Aufrufer: das Banner wird nur gerendert, wenn der Backend-Flag
//! `has_pending_rebooking = true` ist (Fat-Backend, kein FE-Rollen-
//! Check).
//!
//! Klick auf das Banner triggert den `on_click`-Handler mit dem
//! `pending_rebooking_id` — der Parent oeffnet dann das
//! `RebookingSuggestionModal`.

use dioxus::prelude::*;
use uuid::Uuid;

use crate::i18n::Key;
use crate::service::i18n::I18N;

#[derive(Props, Clone, PartialEq)]
pub struct RebookingAlertBannerProps {
    /// SalesPerson des Reports (nur fuer Kontext-Log, nicht im UI).
    pub sales_person_id: Uuid,
    /// ID der Pending-Suggestion — wird an `on_click` durchgereicht.
    pub pending_rebooking_id: Uuid,
    /// Handler: Parent oeffnet Suggestion-Modal mit dieser Batch-ID.
    pub on_click: EventHandler<Uuid>,
}

#[component]
pub fn RebookingAlertBanner(props: RebookingAlertBannerProps) -> Element {
    let i18n = I18N.read().clone();
    let title = i18n.t(Key::RebookingBannerTitle);
    let body = i18n.t(Key::RebookingBannerBody);
    let pending_id = props.pending_rebooking_id;
    let _sp = props.sales_person_id; // reserved for future logging/analytics
    rsx! {
        button {
            r#type: "button",
            class: "w-full text-left p-3 rounded border-l-4 bg-warn-soft border-warn hover:bg-warn-soft/80 focus:outline-none focus-visible:ring-2 focus-visible:ring-warn",
            onclick: move |_| props.on_click.call(pending_id),
            div { class: "font-semibold text-warn", "{title}" }
            div { class: "text-small text-ink-muted", "{body}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn banner_renders_as_button_not_dialog() {
        fn app() -> Element {
            rsx! {
                RebookingAlertBanner {
                    sales_person_id: Uuid::from_u128(1),
                    pending_rebooking_id: Uuid::from_u128(2),
                    on_click: |_| {},
                }
            }
        }
        let html = render(app);
        // Kein Dialog-Rolle — Banner MUSS ein button sein (Inline-Pattern).
        assert!(
            html.contains("<button"),
            "banner must render as a button, got: {html}"
        );
        assert!(
            !html.contains(r#"role="dialog""#),
            "banner must NOT be a dialog (MEMORY feedback_warnings_inline_not_dialog): {html}"
        );
    }

    #[test]
    fn banner_has_warn_semantics() {
        fn app() -> Element {
            rsx! {
                RebookingAlertBanner {
                    sales_person_id: Uuid::from_u128(1),
                    pending_rebooking_id: Uuid::from_u128(2),
                    on_click: |_| {},
                }
            }
        }
        let html = render(app);
        // Muss visuell als warn-Semantik erkennbar sein.
        assert!(
            html.contains("warn"),
            "banner missing warn styling: {html}"
        );
    }
}
