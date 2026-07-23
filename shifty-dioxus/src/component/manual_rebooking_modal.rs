//! `ManualRebookingModal` — Phase 55 (F3, REB-MANUAL-01/02/03, D-55-05/06).
//!
//! Minimalistisches HR-Modal zum Anlegen einer manuellen Umbuchung fuer den
//! UNIQUE-Slot `(sales_person, iso_year, iso_week)`. Reuset die zentrale
//! `Dialog`-Shell (kein eigenes Backdrop/Chrome) + spiegelt die Feld-/Fehler-/
//! Submit-Defense-Muster aus `absence_convert_modal.rs`.
//!
//! **Fat Backend, Thin Client:** Das Modal rechnet KEINE DANN-Werte (die
//! DANN-Tabelle ist dem F5-Suggestion-Modal vorbehalten). Es sammelt nur die
//! Eingaben + zeigt eine Text-Vorschau (REB-MANUAL-03).
//!
//! **Interaktions-Contract (D-55-05):**
//! - Submit-Defense: Menge > 0, sonst inline `RebookingErrorAmount`, kein Dispatch.
//! - Bei 409 (belegte Woche) setzt die aufrufende Seite `slot_taken_error = true`;
//!   das Modal bleibt offen und zeigt `RebookingErrorSlotTaken` inline unter der
//!   Woche-Auswahl. Kein Auto-Overwrite — HR kann die Woche aendern.
//! - Submit-Button waehrend Request `disabled` (`in_flight`, Doppelklick-Schutz).
//! - KEIN week-Picker-Signal (D-25-06-Risiko) — getippte Jahr/KW-Inputs.

use dioxus::prelude::*;

use crate::base_types::{format_hours, ImStr};
use crate::component::atoms::{Btn, BtnVariant};
use crate::component::dialog::{Dialog, DialogVariant};
use crate::i18n::Key;
use crate::service::i18n::I18N;
use crate::state::rebooking::RebookingDirection;

#[derive(Props, Clone, PartialEq)]
pub struct ManualRebookingModalProps {
    /// Steuert die Sichtbarkeit (Dialog-Shell early-return bei `false`).
    pub open: bool,
    /// Anzeigename der Person (Dialog-Subtitle).
    pub person_name: ImStr,
    /// Default-Jahr (aktuelles ISO-Jahr, von der Seite berechnet).
    pub default_year: u32,
    /// Default-KW (aktuelle ISO-Woche, von der Seite berechnet).
    pub default_week: u8,
    /// Wenn `true`: der letzte Submit lief in ein 409 (belegte Woche). Das
    /// Modal bleibt offen und zeigt `RebookingErrorSlotTaken` inline (D-55-05).
    #[props(default = false)]
    pub slot_taken_error: bool,
    /// Wenn `true`: ein Submit-Request laeuft — Submit-Button disabled.
    #[props(default = false)]
    pub in_flight: bool,
    /// Submit → `(direction, hours, iso_year, iso_week)`. Nur bei Menge > 0.
    pub on_submit: EventHandler<(RebookingDirection, f32, u32, u8)>,
    /// Schliesst das Modal (Cancel / Backdrop / ESC / ×).
    pub on_cancel: EventHandler<()>,
}

#[component]
pub fn ManualRebookingModal(props: ManualRebookingModalProps) -> Element {
    let i18n = I18N.read().clone();

    // Eingabe-Signale (getippte Inputs, KEIN week-Picker — D-25-06).
    let mut direction = use_signal(RebookingDirection::default);
    let mut amount_str = use_signal(String::new);
    let default_year = props.default_year;
    let default_week = props.default_week;
    let mut year_str = use_signal(move || default_year.to_string());
    let mut week_str = use_signal(move || default_week.to_string());
    // Submit-Defense-Fehler (Menge <= 0 / Parse) — inline, kein Dispatch.
    let mut amount_error = use_signal(|| false);

    let title = i18n.t(Key::ManualRebookingModalTitle);
    let subtitle = props.person_name.clone();
    let direction_label = i18n.t(Key::RebookingDirectionLabel);
    let dir_vol_to_extra = i18n.t(Key::RebookingDirectionVolunteerToExtra);
    let dir_extra_to_vol = i18n.t(Key::RebookingDirectionExtraToVolunteer);
    let amount_label = i18n.t(Key::RebookingAmountLabel);
    let week_label = i18n.t(Key::RebookingWeekLabel);
    let year_label = i18n.t(Key::RebookingYearLabel);
    let submit_label = i18n.t(Key::RebookingManualSubmit).to_string();
    let cancel_label = i18n.t(Key::RebookingCancel).to_string();
    let amount_err_text = i18n.t(Key::RebookingErrorAmount).to_string();
    let slot_taken_text = i18n.t(Key::RebookingErrorSlotTaken).to_string();

    // Vorschau-Zeile (REB-MANUAL-03): reine Text-Zusammenfassung, keine Rechnung.
    let parsed_amount = amount_str.read().trim().parse::<f32>().ok();
    let dir_text = match *direction.read() {
        RebookingDirection::VolunteerToExtra => dir_vol_to_extra.as_ref().to_string(),
        RebookingDirection::ExtraToVolunteer => dir_extra_to_vol.as_ref().to_string(),
    };
    let preview_hours = parsed_amount.map(|a| format_hours(a, 2)).unwrap_or_else(|| "–".to_string());
    let preview_text = i18n
        .t(Key::RebookingPreview)
        .as_ref()
        .replace("{direction}", &dir_text)
        .replace("{hours}", &preview_hours)
        .replace("{week}", week_str.read().as_str())
        .replace("{year}", year_str.read().as_str());

    let on_submit = props.on_submit;
    let on_cancel = props.on_cancel;
    let in_flight = props.in_flight;
    let slot_taken = props.slot_taken_error;

    let footer = rsx! {
        Btn {
            variant: BtnVariant::Secondary,
            on_click: move |_| on_cancel.call(()),
            "{cancel_label}"
        }
        Btn {
            variant: BtnVariant::Primary,
            disabled: in_flight,
            on_click: move |_| {
                if in_flight {
                    return;
                }
                // Submit-Defense: Menge > 0 (+ parsebar), sonst inline, kein Dispatch.
                let amount = amount_str.read().trim().parse::<f32>().ok();
                let year = year_str.read().trim().parse::<u32>().ok();
                let week = week_str.read().trim().parse::<u8>().ok();
                match (amount, year, week) {
                    (Some(a), Some(y), Some(w)) if a > 0.0 => {
                        amount_error.set(false);
                        on_submit.call((*direction.read(), a, y, w));
                    }
                    _ => {
                        amount_error.set(true);
                    }
                }
            },
            "{submit_label}"
        }
    };

    rsx! {
        Dialog {
            open: props.open,
            on_close: move |_| on_cancel.call(()),
            title: ImStr::from(title.as_ref()),
            subtitle: Some(subtitle),
            variant: DialogVariant::Center,
            width: 460,
            footer: Some(footer),

            div { class: "flex flex-col gap-4",
                // Richtung (D-55-06) — select, Default VolunteerToExtra.
                label { class: "flex flex-col gap-1",
                    span { class: "text-small text-ink-muted", "{direction_label}" }
                    select {
                        class: "border border-border rounded-md p-2",
                        onchange: move |ev| {
                            let next = match ev.value().as_str() {
                                "ExtraToVolunteer" => RebookingDirection::ExtraToVolunteer,
                                _ => RebookingDirection::VolunteerToExtra,
                            };
                            direction.set(next);
                        },
                        option {
                            value: "VolunteerToExtra",
                            selected: *direction.read() == RebookingDirection::VolunteerToExtra,
                            "{dir_vol_to_extra}"
                        }
                        option {
                            value: "ExtraToVolunteer",
                            selected: *direction.read() == RebookingDirection::ExtraToVolunteer,
                            "{dir_extra_to_vol}"
                        }
                    }
                }
                // Menge (Stunden) — input number, min 0, step 0.5.
                label { class: "flex flex-col gap-1",
                    span { class: "text-small text-ink-muted", "{amount_label}" }
                    input {
                        r#type: "number",
                        min: "0",
                        step: "0.5",
                        class: "border border-border rounded-md p-2",
                        value: "{amount_str}",
                        oninput: move |ev| { amount_str.set(ev.value()); },
                    }
                    // Submit-Defense inline (Menge <= 0).
                    if *amount_error.read() {
                        span { class: "text-bad text-small font-normal mt-1", "{amount_err_text}" }
                    }
                }
                // Woche (D-55-05) — getippte Jahr/KW-Inputs (KEIN week-Picker, D-25-06).
                div { class: "flex gap-3",
                    label { class: "flex flex-col gap-1 flex-1",
                        span { class: "text-small text-ink-muted", "{year_label}" }
                        input {
                            r#type: "number",
                            class: "border border-border rounded-md p-2",
                            value: "{year_str}",
                            oninput: move |ev| { year_str.set(ev.value()); },
                        }
                    }
                    label { class: "flex flex-col gap-1 flex-1",
                        span { class: "text-small text-ink-muted", "{week_label}" }
                        input {
                            r#type: "number",
                            min: "1",
                            max: "53",
                            class: "border border-border rounded-md p-2",
                            value: "{week_str}",
                            oninput: move |ev| { week_str.set(ev.value()); },
                        }
                    }
                }
                // Inline 409-Fehler unter der Woche-Auswahl (D-55-05).
                if slot_taken {
                    span { class: "text-bad text-small font-normal mt-1", "{slot_taken_text}" }
                }
                // Vorschau-Zeile (REB-MANUAL-03).
                div { class: "text-small text-ink-muted", "{preview_text}" }
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

    fn pin_de() {
        use crate::i18n::{generate, Locale};
        use_hook(|| {
            *I18N.write() = generate(Locale::De);
        });
    }

    #[test]
    fn closed_modal_renders_nothing() {
        fn app() -> Element {
            pin_de();
            rsx! {
                ManualRebookingModal {
                    open: false,
                    person_name: ImStr::from("Maria"),
                    default_year: 2026,
                    default_week: 30,
                    on_submit: |_| {},
                    on_cancel: |_| {},
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
    fn open_modal_renders_fields_and_defaults() {
        fn app() -> Element {
            pin_de();
            rsx! {
                ManualRebookingModal {
                    open: true,
                    person_name: ImStr::from("Maria"),
                    default_year: 2026,
                    default_week: 30,
                    on_submit: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(app);
        // Dialog-Shell + Person-Subtitle.
        assert!(html.contains("Maria"), "subtitle missing: {html}");
        // Richtung-Select mit beiden Optionen (statische Arms).
        assert!(html.contains("VolunteerToExtra"), "direction option missing: {html}");
        assert!(html.contains("ExtraToVolunteer"), "direction option missing: {html}");
        // Default-Jahr/KW in den Inputs.
        assert!(html.contains(r#"value="2026""#), "default year missing: {html}");
        assert!(html.contains(r#"value="30""#), "default week missing: {html}");
        // Zwei number-Inputs (Jahr/KW) + ein Menge-Input = 3 number-Inputs.
        assert_eq!(
            html.matches(r#"type="number""#).count(),
            3,
            "expected 3 number inputs (amount/year/week), got: {html}"
        );
    }

    #[test]
    fn slot_taken_error_renders_inline() {
        fn app() -> Element {
            pin_de();
            rsx! {
                ManualRebookingModal {
                    open: true,
                    person_name: ImStr::from("Maria"),
                    default_year: 2026,
                    default_week: 30,
                    slot_taken_error: true,
                    on_submit: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(app);
        // 409-Fehler-Copy inline + text-bad-Styling.
        assert!(html.contains("text-bad"), "409 error must use text-bad: {html}");
        assert!(
            html.contains("andere Woche"),
            "slot-taken copy missing: {html}"
        );
    }

    #[test]
    fn in_flight_disables_submit() {
        fn app() -> Element {
            pin_de();
            rsx! {
                ManualRebookingModal {
                    open: true,
                    person_name: ImStr::from("Maria"),
                    default_year: 2026,
                    default_week: 30,
                    in_flight: true,
                    on_submit: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("opacity-50") && html.contains("cursor-not-allowed"),
            "in-flight submit must be disabled: {html}"
        );
    }
}
