//! `RebookingSuggestionModal` — Phase 55 (F5, HR-ALERT-02/03).
//!
//! Zeigt die backend-berechnete IST/DANN-Vorschau (`RebookingSuggestion`) in
//! einem Drei-Spalten-Grid (`grid-cols-[1fr_auto_auto]`) + Approve/Reject-Footer.
//! Reuset die zentrale `Dialog`-Shell (kein eigenes Backdrop/Chrome); Typo-/Farb-
//! Konventionen von `voluntary_stats_row.rs` (`font-mono tabular-nums`,
//! `text-warn`/`text-good`).
//!
//! **Fat Backend, Thin Client (bindend):** ALLE IST/DANN-Werte + `proposed_hours`
//! kommen backend-fertig aus `RebookingSuggestion` — KEIN FE-Rechnen. Die
//! DANN-Faerbung liest nur das Vorzeichen der bereits berechneten Werte.
//!
//! **55-03-Feld-Drift (dokumentiert):** Die Domain-/DTO-Struktur liefert real nur
//! `balance`/`balance_after` + `voluntary_ist`/`voluntary_ist_after` (+
//! `proposed_hours`). Die im UI-SPEC zusaetzlich skizzierten `voluntary_soll`/
//! `voluntary_delta`-Zeilen existieren im Backend NICHT (55-03 Deviation 1); sie
//! werden NICHT gerendert, um die Prohibition "keine Fake-/FE-gerechneten Werte"
//! zu wahren. Die zugehoerigen i18n-Keys bleiben im Copywriting-Contract erhalten.
//!
//! **Interaktions-Contract (HR-ALERT-03, D-55-07 / Pitfall 12):**
//! - Approve/Reject waehrend in-flight `disabled` (FE-Politur; die harte Autoritaet
//!   ist das state-conditional UPDATE im Backend).
//! - Bei 409/Fehler inline `RebookingErrorGeneric`.
//! - Empty-State (Vorschlag beim Oeffnen weg): `RebookingSuggestionEmpty`, nur Close.

use dioxus::prelude::*;

use crate::base_types::{format_hours, ImStr};
use crate::component::atoms::{Btn, BtnVariant};
use crate::component::dialog::{Dialog, DialogVariant};
use crate::i18n::Key;
use crate::service::i18n::I18N;
use crate::state::rebooking::RebookingSuggestion;

#[derive(Props, Clone, PartialEq)]
pub struct RebookingSuggestionModalProps {
    /// Steuert die Sichtbarkeit (Dialog-Shell early-return bei `false`).
    pub open: bool,
    /// Anzeigename der Person (Dialog-Subtitle).
    pub person_name: ImStr,
    /// Backend-berechnete IST/DANN-Vorschau. `None` → Empty-State (Race).
    #[props(default = None)]
    pub suggestion: Option<RebookingSuggestion>,
    /// Wenn `true`: ein Approve/Reject-Request laeuft — beide Buttons disabled.
    #[props(default = false)]
    pub in_flight: bool,
    /// Wenn `true`: der letzte Approve/Reject lief in einen Fehler/409 →
    /// inline `RebookingErrorGeneric`.
    #[props(default = false)]
    pub had_error: bool,
    /// Approve → schreibt das Pair mit `proposed_hours` (Seite dispatcht).
    pub on_approve: EventHandler<RebookingSuggestion>,
    /// Reject → transitioniert den Batch auf `Rejected` (Seite dispatcht).
    pub on_reject: EventHandler<RebookingSuggestion>,
    /// Schliesst das Modal (Close / Backdrop / ESC / ×).
    pub on_close: EventHandler<()>,
}

/// DANN-Balance-Faerbung: `text-good` wenn >= 0 (Ausgleich erreicht), sonst
/// `text-warn`. Rein signum-basiert auf dem backend-berechneten Wert.
fn dann_balance_class(balance_after: f32) -> &'static str {
    if balance_after >= 0.0 {
        "font-mono tabular-nums text-right text-good"
    } else {
        "font-mono tabular-nums text-right text-warn"
    }
}

#[component]
pub fn RebookingSuggestionModal(props: RebookingSuggestionModalProps) -> Element {
    let i18n = I18N.read().clone();

    let title = i18n.t(Key::RebookingSuggestionModalTitle);
    let subtitle = props.person_name.clone();
    let approve_label = i18n.t(Key::RebookingApprove).to_string();
    let reject_label = i18n.t(Key::RebookingReject).to_string();
    let empty_text = i18n.t(Key::RebookingSuggestionEmpty).to_string();
    let generic_error = i18n.t(Key::RebookingErrorGeneric).to_string();

    let on_close = props.on_close;
    let on_approve = props.on_approve;
    let on_reject = props.on_reject;
    let in_flight = props.in_flight;
    let had_error = props.had_error;
    let suggestion = props.suggestion;

    // Footer: Empty-State → nur Close; sonst Reject (Danger) + Approve (Primary).
    let footer = match suggestion {
        None => rsx! {
            Btn {
                variant: BtnVariant::Secondary,
                on_click: move |_| on_close.call(()),
                "{i18n.t(Key::RebookingCancel)}"
            }
        },
        Some(s) => rsx! {
            Btn {
                variant: BtnVariant::Danger,
                disabled: in_flight,
                on_click: move |_| {
                    if !in_flight {
                        on_reject.call(s);
                    }
                },
                "{reject_label}"
            }
            Btn {
                variant: BtnVariant::Primary,
                disabled: in_flight,
                on_click: move |_| {
                    if !in_flight {
                        on_approve.call(s);
                    }
                },
                "{approve_label}"
            }
        },
    };

    rsx! {
        Dialog {
            open: props.open,
            on_close: move |_| on_close.call(()),
            title: ImStr::from(title.as_ref()),
            subtitle: Some(subtitle),
            variant: DialogVariant::Center,
            width: 460,
            footer: Some(footer),

            if let Some(s) = suggestion {
                RebookingSuggestionGrid { suggestion: s }
                if had_error {
                    div { class: "text-bad text-small font-normal mt-1", "{generic_error}" }
                }
            } else {
                div { class: "text-small text-ink-muted", "{empty_text}" }
            }
        }
    }
}

/// Reines Anzeige-Grid der IST/DANN-Werte — liest nur `RebookingSuggestion`,
/// rechnet nichts.
#[component]
fn RebookingSuggestionGrid(suggestion: RebookingSuggestion) -> Element {
    let i18n = I18N.read().clone();
    let hours: ImStr = ImStr::from(i18n.t(Key::Hours).as_ref());

    let col_ist = i18n.t(Key::RebookingColIst);
    let col_dann = i18n.t(Key::RebookingColDann);
    let row_balance = i18n.t(Key::RebookingRowBalance);
    let row_vol_ist = i18n.t(Key::RebookingRowVoluntaryIst);
    let proposed_label = i18n
        .t(Key::RebookingProposedHours)
        .as_ref()
        .replace("{hours}", &format_hours(suggestion.proposed_hours, 2));

    // Backend-berechnete Werte, nur formatiert.
    let balance_ist = format_hours(suggestion.balance, 2);
    let balance_dann = format_hours(suggestion.balance_after, 2);
    let vol_ist = format_hours(suggestion.voluntary_ist, 2);
    let vol_dann = format_hours(suggestion.voluntary_ist_after, 2);
    let dann_bal_class = dann_balance_class(suggestion.balance_after);

    rsx! {
        div { class: "grid grid-cols-[1fr_auto_auto] gap-x-4 gap-y-1.5 items-baseline",
            // Header-Zeile: leer / IST / DANN.
            div {}
            div { class: "text-micro font-semibold uppercase text-ink-muted text-right", "{col_ist}" }
            div { class: "text-micro font-semibold uppercase text-ink-muted text-right", "{col_dann}" }

            // Stundenkonto (Balance) — DANN good/warn.
            div { class: "text-small text-ink-soft", "{row_balance}" }
            div { class: "font-mono tabular-nums text-right", "{balance_ist} {hours}" }
            div { class: "{dann_bal_class}", "{balance_dann} {hours}" }

            // Freiwillig Ist — neutral.
            div { class: "text-small text-ink-soft", "{row_vol_ist}" }
            div { class: "font-mono tabular-nums text-right", "{vol_ist} {hours}" }
            div { class: "font-mono tabular-nums text-right", "{vol_dann} {hours}" }
        }
        // Hervorgehobene Vorschlags-Zeile.
        div {
            class: "text-body font-semibold text-ink pt-2 mt-2 border-t border-border",
            "{proposed_label}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

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

    fn sample() -> RebookingSuggestion {
        RebookingSuggestion {
            batch_id: Uuid::from_u128(1),
            sales_person_id: Uuid::from_u128(2),
            proposed_hours: 2.0,
            balance: -2.0,
            balance_after: 0.0,
            voluntary_ist: 5.0,
            voluntary_ist_after: 3.0,
        }
    }

    #[test]
    fn dann_balance_class_good_when_non_negative() {
        assert!(dann_balance_class(0.0).contains("text-good"));
        assert!(dann_balance_class(1.5).contains("text-good"));
    }

    #[test]
    fn dann_balance_class_warn_when_negative() {
        assert!(dann_balance_class(-0.5).contains("text-warn"));
    }

    #[test]
    fn closed_modal_renders_nothing() {
        fn app() -> Element {
            pin_de();
            rsx! {
                RebookingSuggestionModal {
                    open: false,
                    person_name: ImStr::from("Maria"),
                    suggestion: Some(sample()),
                    on_approve: |_| {},
                    on_reject: |_| {},
                    on_close: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("shifty-dialog-title"),
            "closed modal must render no dialog: {html}"
        );
    }

    #[test]
    fn grid_shows_backend_ist_dann_values_only() {
        fn app() -> Element {
            pin_de();
            rsx! {
                RebookingSuggestionModal {
                    open: true,
                    person_name: ImStr::from("Maria"),
                    suggestion: Some(sample()),
                    on_approve: |_| {},
                    on_reject: |_| {},
                    on_close: |_| {},
                }
            }
        }
        let html = render(app);
        // Person + drei-Spalten-Grid + backend-Werte.
        assert!(html.contains("Maria"), "subtitle missing: {html}");
        assert!(
            html.contains("grid-cols-[1fr_auto_auto]"),
            "IST/DANN grid missing: {html}"
        );
        assert!(html.contains("-2.00"), "balance IST missing: {html}");
        assert!(html.contains("5.00"), "voluntary IST missing: {html}");
        assert!(html.contains("3.00"), "voluntary DANN missing: {html}");
        // proposed_hours-Zeile hervorgehoben.
        assert!(
            html.contains("border-t border-border"),
            "proposed row highlight missing: {html}"
        );
        // Approve + Reject Buttons.
        assert!(html.contains("Best"), "approve button missing: {html}"); // "Bestätigen"
        assert!(html.contains("Ablehnen"), "reject button missing: {html}");
    }

    #[test]
    fn dann_balance_zero_gets_good_class_in_html() {
        // balance_after = 0.0 → text-good (Ausgleich erreicht).
        fn app() -> Element {
            pin_de();
            rsx! {
                RebookingSuggestionModal {
                    open: true,
                    person_name: ImStr::from("Maria"),
                    suggestion: Some(sample()),
                    on_approve: |_| {},
                    on_reject: |_| {},
                    on_close: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("text-good"), "DANN balance >=0 must be good: {html}");
    }

    #[test]
    fn in_flight_disables_both_buttons() {
        fn app() -> Element {
            pin_de();
            rsx! {
                RebookingSuggestionModal {
                    open: true,
                    person_name: ImStr::from("Maria"),
                    suggestion: Some(sample()),
                    in_flight: true,
                    on_approve: |_| {},
                    on_reject: |_| {},
                    on_close: |_| {},
                }
            }
        }
        let html = render(app);
        // Beide Footer-Buttons disabled (mindestens 2× opacity-50).
        assert!(
            html.matches("cursor-not-allowed").count() >= 2,
            "both buttons must be disabled in-flight: {html}"
        );
    }

    #[test]
    fn empty_state_shows_copy_and_only_close() {
        fn app() -> Element {
            pin_de();
            rsx! {
                RebookingSuggestionModal {
                    open: true,
                    person_name: ImStr::from("Maria"),
                    suggestion: None,
                    on_approve: |_| {},
                    on_reject: |_| {},
                    on_close: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Kein offener Vorschlag"),
            "empty-state copy missing: {html}"
        );
        // Kein Approve/Reject im Empty-State.
        assert!(
            !html.contains("Ablehnen"),
            "empty-state must not show reject: {html}"
        );
    }

    #[test]
    fn had_error_renders_generic_inline() {
        fn app() -> Element {
            pin_de();
            rsx! {
                RebookingSuggestionModal {
                    open: true,
                    person_name: ImStr::from("Maria"),
                    suggestion: Some(sample()),
                    had_error: true,
                    on_approve: |_| {},
                    on_reject: |_| {},
                    on_close: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("text-bad"), "generic error must use text-bad: {html}");
    }
}
