//! `RebookingSuggestionModal` — Phase 55 HR-ALERT-02/03.
//!
//! Zeigt die vom Backend berechneten IST/DANN-Werte (Balance,
//! Voluntary-Ist, Voluntary-Soll, Voluntary-Delta) fuer eine
//! Pending-Suggestion und bietet Approve/Reject-Buttons an.
//!
//! **Kein FE-Rechnen** (D-55-03, Fat-Backend): Alle Delta-Felder
//! kommen als `voluntary_delta_before`/`voluntary_delta_after`
//! direkt vom Backend. Keine Subtraktion im Component.
//!
//! **Kein Bestaetigungs-Dialog** (MEMORY
//! `feedback_warnings_inline_not_dialog`): Klick auf Approve/Reject
//! ist bereits die Aktion. Bei HTTP 409 rendert das Modal eine
//! inline Warn-Section und bleibt offen.
//!
//! Basiert auf der zentralen `Dialog`-Shell fuer Backdrop/ESC.

use dioxus::prelude::*;

use crate::base_types::{format_hours, ImStr};
use crate::component::dialog::{Dialog, DialogVariant};
use crate::component::{Btn, BtnVariant};
use crate::i18n::Key;
use crate::loader::{approve_rebooking_suggestion, reject_rebooking_suggestion};
use crate::service::config::CONFIG;
use crate::service::i18n::I18N;
use crate::state::rebooking::{RebookingSubmitError, RebookingSuggestion};

#[derive(Props, Clone, PartialEq)]
pub struct RebookingSuggestionModalProps {
    pub suggestion: RebookingSuggestion,
    /// Handler: schliesst das Modal ohne Aktion.
    pub on_close: EventHandler<()>,
    /// Handler: Parent laedt Report neu nachdem Approve/Reject
    /// erfolgreich war.
    pub on_after_action: EventHandler<()>,
}

#[component]
pub fn RebookingSuggestionModal(props: RebookingSuggestionModalProps) -> Element {
    let i18n = I18N.read().clone();
    let title: ImStr = ImStr::from(i18n.t(Key::RebookingModalTitleSuggestion).as_ref());
    let approve_label = i18n.t(Key::RebookingApprove);
    let reject_label = i18n.t(Key::RebookingReject);
    let ist_label = i18n.t(Key::RebookingIstColumn);
    let dann_label = i18n.t(Key::RebookingDannColumn);
    let row_balance = i18n.t(Key::RebookingRowBalance);
    let row_ist = i18n.t(Key::RebookingRowVoluntaryIst);
    let row_soll = i18n.t(Key::RebookingRowVoluntarySoll);
    let row_delta = i18n.t(Key::RebookingRowVoluntaryDelta);

    // Inline-Warn-State fuer 409-Fehler.
    let mut error_key: Signal<Option<Key>> = use_signal(|| None);
    let mut busy = use_signal(|| false);

    let suggestion = props.suggestion.clone();
    let batch_id = suggestion.batch_id;

    let on_approve = {
        let on_close = props.on_close;
        let on_after = props.on_after_action;
        move |_| {
            error_key.set(None);
            busy.set(true);
            spawn(async move {
                let config = CONFIG.read().clone();
                match approve_rebooking_suggestion(config, batch_id).await {
                    Ok(_) => {
                        busy.set(false);
                        on_after.call(());
                        on_close.call(());
                    }
                    Err(RebookingSubmitError::AlreadyResolved) => {
                        busy.set(false);
                        error_key.set(Some(Key::RebookingErrorAlreadyResolved));
                    }
                    Err(_) => {
                        busy.set(false);
                        error_key.set(Some(Key::RebookingErrorAlreadyResolved));
                    }
                }
            });
        }
    };

    let on_reject = {
        let on_close = props.on_close;
        let on_after = props.on_after_action;
        move |_| {
            error_key.set(None);
            busy.set(true);
            spawn(async move {
                let config = CONFIG.read().clone();
                match reject_rebooking_suggestion(config, batch_id).await {
                    Ok(_) => {
                        busy.set(false);
                        on_after.call(());
                        on_close.call(());
                    }
                    Err(RebookingSubmitError::AlreadyResolved) => {
                        busy.set(false);
                        error_key.set(Some(Key::RebookingErrorAlreadyResolved));
                    }
                    Err(_) => {
                        busy.set(false);
                        error_key.set(Some(Key::RebookingErrorAlreadyResolved));
                    }
                }
            });
        }
    };

    let footer_on_close = props.on_close;
    let footer = rsx! {
        Btn {
            variant: BtnVariant::Secondary,
            disabled: *busy.read(),
            on_click: move |_| footer_on_close.call(()),
            "\u{2716}"
        }
        Btn {
            variant: BtnVariant::Danger,
            disabled: *busy.read(),
            on_click: on_reject,
            "{reject_label}"
        }
        Btn {
            variant: BtnVariant::Primary,
            disabled: *busy.read(),
            on_click: on_approve,
            "{approve_label}"
        }
    };

    // Format-Helper — no FE-arithmetic; alle Werte direkt aus dem Struct.
    let s = &suggestion;
    let bal_ist = format_hours(s.balance_before, 2);
    let bal_dann = format_hours(s.balance_after, 2);
    let ist_ist = format_hours(s.voluntary_ist_before, 2);
    let ist_dann = format_hours(s.voluntary_ist_after, 2);
    let soll_ist = format_hours(s.voluntary_soll_before, 2);
    let soll_dann = format_hours(s.voluntary_soll_after, 2);
    let delta_ist = format!("{:+.2}", s.voluntary_delta_before);
    let delta_dann = format!("{:+.2}", s.voluntary_delta_after);

    let error_rendered: Option<String> =
        (*error_key.read()).map(|k| i18n.t(k).to_string());

    rsx! {
        Dialog {
            open: true,
            on_close: props.on_close,
            title,
            variant: DialogVariant::Center,
            width: 520,
            footer: Some(footer),

            // Header-Row der IST/DANN-Spalten (kein <table>, sondern TupleRow
            // wie in VoluntaryStatsRow — konsistent mit Employee-Detail-View).
            div { class: "grid grid-cols-3 gap-2 mb-2 pb-2 border-b border-border text-small font-semibold text-ink-muted",
                div {}
                div { class: "text-right", "{ist_label}" }
                div { class: "text-right", "{dann_label}" }
            }

            // Balance-Row
            div { class: "grid grid-cols-3 gap-2 py-1",
                div { class: "text-small text-ink", "{row_balance}" }
                div { class: "text-right font-mono tabular-nums", "{bal_ist}" }
                div { class: "text-right font-mono tabular-nums", "{bal_dann}" }
            }
            // Voluntary-Ist
            div { class: "grid grid-cols-3 gap-2 py-1",
                div { class: "text-small text-ink", "{row_ist}" }
                div { class: "text-right font-mono tabular-nums", "{ist_ist}" }
                div { class: "text-right font-mono tabular-nums", "{ist_dann}" }
            }
            // Voluntary-Soll
            div { class: "grid grid-cols-3 gap-2 py-1",
                div { class: "text-small text-ink", "{row_soll}" }
                div { class: "text-right font-mono tabular-nums", "{soll_ist}" }
                div { class: "text-right font-mono tabular-nums", "{soll_dann}" }
            }
            // Voluntary-Delta (Backend-computed, kein FE-Minus)
            div { class: "grid grid-cols-3 gap-2 py-1",
                div { class: "text-small text-ink", "{row_delta}" }
                div { class: "text-right font-mono tabular-nums", "{delta_ist}" }
                div { class: "text-right font-mono tabular-nums", "{delta_dann}" }
            }

            // Inline-Warn-Section fuer 409 (kein Bestaetigungs-Dialog!).
            if let Some(msg) = error_rendered {
                div {
                    class: "mt-3 p-2 rounded border-l-4 bg-warn-soft border-warn text-small text-ink",
                    role: "alert",
                    "{msg}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_suggestion() -> RebookingSuggestion {
        RebookingSuggestion {
            batch_id: Uuid::from_u128(1),
            sales_person_id: Uuid::from_u128(2),
            iso_year: 2026,
            iso_week: 27,
            proposed_hours: 3.0,
            balance_before: -3.0,
            voluntary_ist_before: 5.0,
            voluntary_soll_before: 2.0,
            voluntary_delta_before: 3.0,
            balance_after: 0.0,
            voluntary_ist_after: 2.0,
            voluntary_soll_after: 2.0,
            voluntary_delta_after: 0.0,
        }
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn suggestion_modal_renders_backend_computed_delta_values_verbatim() {
        fn app() -> Element {
            rsx! {
                RebookingSuggestionModal {
                    suggestion: make_suggestion(),
                    on_close: |_| {},
                    on_after_action: |_| {},
                }
            }
        }
        let html = render(app);
        // Vor-Delta = +3.00, Nach-Delta = +0.00 → beides MUSS im DOM sein.
        assert!(
            html.contains("+3.00"),
            "voluntary_delta_before nicht gerendert: {html}"
        );
        assert!(
            html.contains("+0.00"),
            "voluntary_delta_after nicht gerendert: {html}"
        );
    }

    #[test]
    fn suggestion_modal_does_not_contain_minus_operator_on_ist_soll() {
        // T-55-07 Property-Kontrolle: der Component-Quelltext (Non-Test-Bereich)
        // darf keine Delta-Arithmetik ueber Ist/Soll-Felder machen.
        // Wir schneiden den `#[cfg(test)]`-Block ab, damit das Assertion-
        // Muster in diesem Test nicht als Selbstreferenz zaehlt.
        let full_src = include_str!("rebooking_suggestion_modal.rs");
        let cutoff = full_src.find("#[cfg(test)]").unwrap_or(full_src.len());
        let production_src = &full_src[..cutoff];
        // Verbotene Delta-Formeln — normalisiert auf ein Kernwort-Paar.
        for pattern in [
            "voluntary_ist_before - voluntary_soll_before",
            "voluntary_ist_after - voluntary_soll_after",
            "voluntary_ist_before-voluntary_soll_before",
            "voluntary_ist_after-voluntary_soll_after",
        ] {
            assert!(
                !production_src.contains(pattern),
                "FE-Delta-Arithmetik im Production-Bereich gefunden ({pattern}) \u{2014} D-55-03 verletzt"
            );
        }
    }
}
