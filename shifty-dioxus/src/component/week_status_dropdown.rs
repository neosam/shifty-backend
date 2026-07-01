//! `WeekStatusDropdown` — shiftplaner action element to set the calendar-week
//! status. Built on the existing [`DropdownTrigger`]; **not** a controlled
//! `<select>` (D-39-06).
//!
//! The trigger mirrors the current status: at [`WeekStatus::Unset`] it shows a
//! neutral "Kein" pill (`bg-surface-alt border-border text-ink-muted`), otherwise
//! it reuses the badge color token (D-39-08) plus `gap-1` for the caret. All four
//! entries — including "Kein" (Unset) so a shiftplaner can reset (D-39-07) — call
//! `on_change` with the chosen [`WeekStatus`]; the store re-fetches after the
//! mutation (fresh-fetch, Wave 4).

use std::rc::Rc;

use dioxus::prelude::*;

use crate::component::atoms::week_status_badge::week_status_label_key;
use crate::component::dropdown_base::DropdownTrigger;
use crate::i18n::Key;
use crate::service::i18n::I18N;
use crate::state::dropdown::DropdownEntry;
use crate::state::week_status::WeekStatus;

/// Static trigger class per status. Set states reuse the badge token + `gap-1`
/// for the caret; `Unset` is the neutral trigger (never a badge, D-39-08).
fn week_status_trigger_class(status: &WeekStatus) -> &'static str {
    match status {
        WeekStatus::Unset => {
            "inline-flex items-center gap-1 px-2 py-0.5 rounded-sm text-small font-medium bg-surface-alt border border-border text-ink-muted"
        }
        WeekStatus::Locked => {
            "inline-flex items-center gap-1 px-2 py-0.5 rounded-sm text-small font-medium bg-bad-soft border border-bad text-bad"
        }
        WeekStatus::Planned => {
            "inline-flex items-center gap-1 px-2 py-0.5 rounded-sm text-small font-medium bg-good-soft border border-good text-good"
        }
        WeekStatus::InPlanning => {
            "inline-flex items-center gap-1 px-2 py-0.5 rounded-sm text-small font-medium bg-warn-soft border border-warn text-warn"
        }
    }
}

/// i18n label key for the trigger. Unlike the badge, `Unset` has a label ("Kein").
fn trigger_label_key(status: &WeekStatus) -> Key {
    match status {
        WeekStatus::Unset => Key::WeekStatusUnset,
        _ => week_status_label_key(status),
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct WeekStatusDropdownProps {
    pub current_status: WeekStatus,
    pub year: u32,
    pub week: u8,
    pub on_change: EventHandler<WeekStatus>,
}

#[component]
pub fn WeekStatusDropdown(props: WeekStatusDropdownProps) -> Element {
    let i18n = I18N.read();
    let on_change = props.on_change;

    // Entries: Kein → In Planung → Geplant → Gesperrt (ascending commitment,
    // incl. Unset to reset — D-39-07). Each entry forwards the chosen status.
    let entries: Rc<[DropdownEntry]> = [
        WeekStatus::Unset,
        WeekStatus::InPlanning,
        WeekStatus::Planned,
        WeekStatus::Locked,
    ]
    .into_iter()
    .map(|status| {
        let text = i18n.t(trigger_label_key(&status));
        let handler_status = status.clone();
        DropdownEntry::from((
            crate::base_types::ImStr::from(text.as_ref()),
            move |_: Option<Rc<str>>| on_change.call(handler_status.clone()),
        ))
    })
    .collect();

    let trigger_class = week_status_trigger_class(&props.current_status);
    let trigger_label = i18n.t(trigger_label_key(&props.current_status));
    let aria_label = i18n.t(Key::WeekStatusChangeAriaLabel);

    rsx! {
        DropdownTrigger {
            entries,
            context: None,
            button {
                r#type: "button",
                class: trigger_class,
                "aria-label": "{aria_label}",
                "{trigger_label}"
                span { class: "font-mono", "▾" }
            }
        }
    }
}
