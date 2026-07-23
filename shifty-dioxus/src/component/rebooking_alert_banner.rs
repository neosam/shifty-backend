//! `RebookingAlertBanner` — Phase 55 (F5, HR-ALERT-01, D-55-01/07).
//!
//! Dauerhafte, klickbare Warn-Zeile im HR-Employee-Overview (mirror
//! `warning_list.rs` Container + `impersonation_banner.rs` klickbare Zeile).
//!
//! **Fat Backend, Thin Client (bindend):** Die Sichtbarkeit haengt AUSSCHLIESSLICH
//! am backend-berechneten `has_pending_rebooking`-Flag (`ShortEmployeeReportTO`,
//! 55-03). Kein FE-Rollen-Check, keine FE-Arithmetik — der Predicate
//! (`balance <= -0.5h ∧ voluntary_ist > 0 ∧ cap_active`, D-55-01) ist backend-computed.
//!
//! **Zero-State (D-55-07):** Wenn `has_pending_rebooking == false` rendert die
//! Komponente NICHTS (unsichtbar, wie die Overage-Sektion in Phase 24).
//!
//! **Interaktion:** Die gesamte Zeile ist klickbar (`role="button"`,
//! `tabindex="0"`, Enter/Space) und oeffnet das Suggestion-Modal fuer diese
//! Person via `on_open`-Callback (traegt die `pending_rebooking_id`, falls gesetzt).

use uuid::Uuid;

use dioxus::prelude::*;

use crate::base_types::{format_hours, ImStr};
use crate::i18n::Key;
use crate::service::i18n::I18N;

#[derive(Props, Clone, PartialEq)]
pub struct RebookingAlertBannerProps {
    /// Backend-berechnetes Flag — steuert die Sichtbarkeit (Zero-State-Guard).
    pub has_pending_rebooking: bool,
    /// Anzeigename der betroffenen Person (font-semibold im Banner).
    pub person_name: ImStr,
    /// IST-Stundenkonto-Balance (backend-formatiert im Banner-Text).
    pub balance: f32,
    /// Verfuegbare Freiwilligenstunden (backend-formatiert im Banner-Text).
    pub voluntary: f32,
    /// pending-Batch-id (falls das Backend sie mitliefert) — wird an `on_open`
    /// weitergereicht, damit die Seite das Suggestion-Modal oeffnen kann.
    #[props(default = None)]
    pub pending_rebooking_id: Option<Uuid>,
    /// Klick / Enter / Space → oeffnet das Suggestion-Modal fuer diese Person.
    pub on_open: EventHandler<Option<Uuid>>,
}

#[component]
pub fn RebookingAlertBanner(props: RebookingAlertBannerProps) -> Element {
    // Zero-State-Guard (D-55-07): nur bei backend-gesetztem Flag rendern.
    if !props.has_pending_rebooking {
        return rsx! {};
    }

    let i18n = I18N.read().clone();

    // Zahlen font-mono tabular-nums (Praezedenz voluntary_stats_row).
    let balance_str = format_hours(props.balance, 2);
    let voluntary_str = format_hours(props.voluntary, 2);
    let body_text = i18n
        .t(Key::RebookingAlertBannerText)
        .as_ref()
        .replace("{balance}", &balance_str)
        .replace("{voluntary}", &voluntary_str);
    let action_text = i18n.t(Key::RebookingAlertBannerAction).to_string();

    let person = props.person_name.as_str().to_string();
    let aria = format!("{person}: {action_text}");
    let pending_id = props.pending_rebooking_id;
    let on_open = props.on_open;

    rsx! {
        // Container exakt per UI-SPEC (mirror WarningList, klickbar).
        div {
            class: "border-l-[3px] border-warn bg-warn-soft rounded-md px-4 py-3 flex items-center justify-between gap-3 cursor-pointer hover:bg-surface-alt",
            role: "button",
            tabindex: "0",
            "aria-label": "{aria}",
            onclick: move |_| on_open.call(pending_id),
            onkeydown: move |ev| {
                // A11y: Enter/Space oeffnet das Modal (nicht nur Maus-Klick).
                // Idiom wie page/absences.rs: ev.key().to_string() vergleichen.
                // `Key::Character(" ")` (Leertaste) serialisiert zu " ".
                let key = ev.key().to_string();
                if key == "Enter" || key == " " {
                    ev.prevent_default();
                    on_open.call(pending_id);
                }
            },
            span { class: "text-body text-ink",
                "\u{26A0}\u{FE0F} "
                span { class: "font-semibold", "{person}" }
                ": "
                span { class: "font-mono tabular-nums", "{body_text}" }
            }
            span { class: "text-small text-warn font-semibold", "{action_text}" }
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

    fn pin_de() {
        use crate::i18n::{generate, Locale};
        use_hook(|| {
            *I18N.write() = generate(Locale::De);
        });
    }

    /// Zero-State (D-55-07): has_pending_rebooking == false → nichts rendern.
    #[test]
    fn banner_hidden_when_no_pending_rebooking() {
        fn app() -> Element {
            pin_de();
            rsx! {
                RebookingAlertBanner {
                    has_pending_rebooking: false,
                    person_name: ImStr::from("Maria"),
                    balance: -3.0,
                    voluntary: 5.0,
                    on_open: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("border-l-[3px]"),
            "zero-state banner must render nothing, got: {html}"
        );
        assert!(
            !html.contains("Maria"),
            "zero-state banner must not show the person, got: {html}"
        );
    }

    /// Sichtbar bei Flag: Container + Person + a11y role/tabindex.
    #[test]
    fn banner_visible_with_pending_flag_and_a11y() {
        fn app() -> Element {
            pin_de();
            rsx! {
                RebookingAlertBanner {
                    has_pending_rebooking: true,
                    person_name: ImStr::from("Maria"),
                    balance: -3.0,
                    voluntary: 5.0,
                    on_open: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("border-l-[3px]") && html.contains("bg-warn-soft"),
            "visible banner must use the warn container, got: {html}"
        );
        assert!(html.contains("Maria"), "person name missing: {html}");
        // A11y: role=button + tabindex=0 + Enter/Space (keydown vorhanden).
        assert!(
            html.contains(r#"role="button""#),
            "missing role=button: {html}"
        );
        assert!(
            html.contains(r#"tabindex="0""#),
            "missing tabindex=0: {html}"
        );
    }

    /// Zahlen backend-formatiert (font-mono) + Balance-Wert im Text.
    #[test]
    fn banner_renders_backend_formatted_numbers() {
        fn app() -> Element {
            pin_de();
            rsx! {
                RebookingAlertBanner {
                    has_pending_rebooking: true,
                    person_name: ImStr::from("Alex"),
                    balance: -3.0,
                    voluntary: 5.0,
                    on_open: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("font-mono"), "numbers must be mono: {html}");
        assert!(html.contains("-3.00"), "balance value missing: {html}");
        assert!(html.contains("5.00"), "voluntary value missing: {html}");
    }
}
