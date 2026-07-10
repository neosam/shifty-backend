//! `ManualRebookingModal` — Phase 55 F3 (REB-MANUAL-01/02/03).
//!
//! HR waehlt Jahr + Kalenderwoche + Richtung + Stundenmenge und
//! bucht ueber `POST /rebooking/manual`. Kein Datepicker (MEMORY
//! `reference_dioxus_browser_test_date_inputs` warnt vor
//! `<input type=date>`-Signal-Problemen — hier zwei
//! `<input type=number>` fuer Year+Week).
//!
//! Preview-Sektion zeigt die aktuelle Auswahl kompakt an
//! ("Umbuchung: {hours} h — {direction} — KW {week}/{year}");
//! Submit-Button ist der Bestaetigungs-Trigger — KEIN separater
//! Confirm-Dialog (MEMORY `feedback_warnings_inline_not_dialog`).
//!
//! Bei HTTP 409 `RebookingErrorSlotTaken` rendert das Modal eine
//! inline Warn-Section und bleibt offen, sodass der Nutzer die
//! Woche wechseln kann.

use dioxus::prelude::*;
use uuid::Uuid;

use crate::base_types::ImStr;
use crate::component::dialog::{Dialog, DialogVariant};
use crate::component::{Btn, BtnVariant};
use crate::i18n::Key;
use crate::loader::submit_manual_rebooking;
use crate::service::config::CONFIG;
use crate::service::i18n::I18N;
use crate::state::rebooking::{
    ManualRebookingRequest, RebookingDirection, RebookingSubmitError,
};

#[derive(Props, Clone, PartialEq)]
pub struct ManualRebookingModalProps {
    pub sales_person_id: Uuid,
    /// Aktuelle Kalenderwoche als Default fuer das Wochen-Feld
    /// (D-55-05: Default = aktuelle KW).
    pub current_iso_year: u32,
    pub current_iso_week: u8,
    pub on_close: EventHandler<()>,
    /// Handler: Parent laedt Report neu, wenn Buchung erfolgreich war.
    pub on_success: EventHandler<()>,
}

#[component]
pub fn ManualRebookingModal(props: ManualRebookingModalProps) -> Element {
    let i18n = I18N.read().clone();
    let title: ImStr = ImStr::from(i18n.t(Key::RebookingModalTitleManual).as_ref());
    let submit_label = i18n.t(Key::RebookingSubmit);
    let year_label = i18n.t(Key::RebookingYearLabel);
    let week_label = i18n.t(Key::RebookingWeekLabel);
    let hours_label = i18n.t(Key::RebookingHoursLabel);
    let preview_label = i18n.t(Key::RebookingPreviewLabel);
    let dir_v2e_label = i18n.t(Key::RebookingDirectionVolunteerToExtra);
    let dir_e2v_label = i18n.t(Key::RebookingDirectionExtraToVolunteer);

    // Signals
    let default_year = props.current_iso_year;
    let default_week = props.current_iso_week;
    let mut year_signal = use_signal(move || default_year);
    let mut week_signal = use_signal(move || default_week);
    // D-55-06: Default = VolunteerToExtra (haeufigster Fall).
    let mut direction = use_signal(|| RebookingDirection::VolunteerToExtra);
    let mut hours = use_signal(|| 0.0f32);
    let mut error_key: Signal<Option<Key>> = use_signal(|| None);
    let mut busy = use_signal(|| false);

    let sales_person_id = props.sales_person_id;

    let submit_disabled = *hours.read() <= 0.0 || *busy.read();

    let on_submit = {
        let on_close = props.on_close;
        let on_success = props.on_success;
        move |_| {
            let request = ManualRebookingRequest {
                sales_person_id,
                iso_year: *year_signal.read(),
                iso_week: *week_signal.read(),
                direction: *direction.read(),
                hours: *hours.read(),
            };
            error_key.set(None);
            busy.set(true);
            spawn(async move {
                let config = CONFIG.read().clone();
                match submit_manual_rebooking(config, request).await {
                    Ok(_) => {
                        busy.set(false);
                        on_success.call(());
                        on_close.call(());
                    }
                    Err(RebookingSubmitError::SlotTaken) => {
                        busy.set(false);
                        error_key.set(Some(Key::RebookingErrorSlotTaken));
                    }
                    Err(RebookingSubmitError::AlreadyResolved) => {
                        busy.set(false);
                        error_key.set(Some(Key::RebookingErrorAlreadyResolved));
                    }
                    Err(_) => {
                        busy.set(false);
                        error_key.set(Some(Key::RebookingErrorSlotTaken));
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
            variant: BtnVariant::Primary,
            disabled: submit_disabled,
            on_click: on_submit,
            "{submit_label}"
        }
    };

    // Preview-Text: "Umbuchung: {hours} h — {direction} — KW {week}/{year}"
    let direction_label_for_preview = match *direction.read() {
        RebookingDirection::VolunteerToExtra => dir_v2e_label.clone(),
        RebookingDirection::ExtraToVolunteer => dir_e2v_label.clone(),
    };
    let preview_text = format!(
        "{:.2} h \u{2014} {} \u{2014} KW {}/{}",
        *hours.read(),
        direction_label_for_preview,
        *week_signal.read(),
        *year_signal.read()
    );

    let error_rendered: Option<String> =
        (*error_key.read()).map(|k| i18n.t(k).to_string());

    let current_year = *year_signal.read();
    let current_week = *week_signal.read();
    let current_hours = *hours.read();
    let is_v2e = matches!(*direction.read(), RebookingDirection::VolunteerToExtra);
    let is_e2v = matches!(*direction.read(), RebookingDirection::ExtraToVolunteer);

    rsx! {
        Dialog {
            open: true,
            on_close: props.on_close,
            title,
            variant: DialogVariant::Center,
            width: 460,
            footer: Some(footer),

            // Year + Week — number inputs (KEIN Datepicker!)
            div { class: "grid grid-cols-2 gap-3 mb-3",
                label { class: "flex flex-col gap-1",
                    span { class: "text-small text-ink-muted", "{year_label}" }
                    input {
                        r#type: "number",
                        min: "2000",
                        max: "2100",
                        value: "{current_year}",
                        class: "px-2 py-1 rounded border border-border bg-surface text-ink",
                        oninput: move |ev| {
                            if let Ok(v) = ev.value().parse::<u32>() {
                                year_signal.set(v);
                            }
                        }
                    }
                }
                label { class: "flex flex-col gap-1",
                    span { class: "text-small text-ink-muted", "{week_label}" }
                    input {
                        r#type: "number",
                        min: "1",
                        max: "53",
                        value: "{current_week}",
                        class: "px-2 py-1 rounded border border-border bg-surface text-ink",
                        oninput: move |ev| {
                            if let Ok(v) = ev.value().parse::<u8>() {
                                week_signal.set(v);
                            }
                        }
                    }
                }
            }

            // Direction — Radio-Group (D-55-06)
            div { class: "flex flex-col gap-1 mb-3",
                label { class: "flex items-center gap-2 text-small text-ink",
                    input {
                        r#type: "radio",
                        name: "rebooking-direction",
                        value: "v2e",
                        checked: is_v2e,
                        onchange: move |_| direction.set(RebookingDirection::VolunteerToExtra),
                    }
                    "{dir_v2e_label}"
                }
                label { class: "flex items-center gap-2 text-small text-ink",
                    input {
                        r#type: "radio",
                        name: "rebooking-direction",
                        value: "e2v",
                        checked: is_e2v,
                        onchange: move |_| direction.set(RebookingDirection::ExtraToVolunteer),
                    }
                    "{dir_e2v_label}"
                }
            }

            // Hours
            label { class: "flex flex-col gap-1 mb-3",
                span { class: "text-small text-ink-muted", "{hours_label}" }
                input {
                    r#type: "number",
                    min: "0.0",
                    step: "0.25",
                    value: "{current_hours}",
                    class: "px-2 py-1 rounded border border-border bg-surface text-ink",
                    oninput: move |ev| {
                        if let Ok(v) = ev.value().parse::<f32>() {
                            hours.set(v);
                        }
                    }
                }
            }

            // Preview-Sektion (REB-MANUAL-03): kompakter Vorschau-Satz.
            div { class: "mb-3 p-2 rounded bg-surface-alt border border-border",
                div { class: "text-small font-semibold text-ink-muted", "{preview_label}" }
                div { class: "font-mono tabular-nums text-ink", "{preview_text}" }
            }

            // Inline-Warn-Section fuer 409 (kein Bestaetigungs-Dialog!).
            if let Some(msg) = error_rendered {
                div {
                    class: "p-2 rounded border-l-4 bg-warn-soft border-warn text-small text-ink",
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

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn manual_modal_renders_year_week_hours_inputs() {
        fn app() -> Element {
            rsx! {
                ManualRebookingModal {
                    sales_person_id: Uuid::from_u128(1),
                    current_iso_year: 2026,
                    current_iso_week: 27,
                    on_close: |_| {},
                    on_success: |_| {},
                }
            }
        }
        let html = render(app);
        // Year default in DOM
        assert!(html.contains("2026"), "year default missing: {html}");
        // Week default in DOM
        assert!(html.contains("27"), "week default missing: {html}");
        // Type=number for year/week (no datepicker)
        assert!(
            html.contains(r#"type="number""#),
            "no number input found (must be number, not date): {html}"
        );
    }

    #[test]
    fn manual_modal_renders_direction_radio_group() {
        fn app() -> Element {
            rsx! {
                ManualRebookingModal {
                    sales_person_id: Uuid::from_u128(1),
                    current_iso_year: 2026,
                    current_iso_week: 27,
                    on_close: |_| {},
                    on_success: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains(r#"type="radio""#),
            "direction must be radio group (D-55-06): {html}"
        );
        assert!(
            html.contains(r#"name="rebooking-direction""#),
            "radio group missing name: {html}"
        );
    }

    #[test]
    fn manual_modal_renders_preview_section() {
        fn app() -> Element {
            rsx! {
                ManualRebookingModal {
                    sales_person_id: Uuid::from_u128(1),
                    current_iso_year: 2026,
                    current_iso_week: 27,
                    on_close: |_| {},
                    on_success: |_| {},
                }
            }
        }
        let html = render(app);
        // REB-MANUAL-03: Vorschau muss KW-Zeile + Stunden zeigen
        assert!(
            html.contains("KW 27/2026"),
            "preview line missing week/year: {html}"
        );
    }
}
