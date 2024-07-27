use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    base_types::ImStr,
    component::base_components::*,
    error,
    i18n::{self, Key},
    js, loader,
    state::{week::Week, Config},
};

#[derive(Clone, PartialEq)]
pub enum AddExtraDaysType {
    Vacation,
    SickLeave,
    Holiday,
}

#[derive(Props, Clone, PartialEq)]
pub struct AddExtraDaysFormProps {
    pub extra_hours_type: AddExtraDaysType,
}

#[component]
pub fn AddExtraDaysForm(props: AddExtraDaysFormProps) -> Element {
    let i18n = use_context::<i18n::I18n<Key, i18n::Locale>>();
    let config = use_context::<Config>();

    let title_str = match props.extra_hours_type {
        AddExtraDaysType::Vacation => i18n.t(Key::AddVacationTitle),
        AddExtraDaysType::SickLeave => i18n.t(Key::AddSickLeaveTitle),
        AddExtraDaysType::Holiday => i18n.t(Key::AddHolidaysTitle),
    };
    let week_str = i18n.t(Key::WeekLabel);
    let full_week_str = i18n.t(Key::FullWeekLabel);
    let weeks: Signal<Rc<[Week]>> = use_signal(|| [].into());
    let mut whole_week: Signal<bool> = use_signal(|| true);
    let mut monday: Signal<bool> = use_signal(|| false);
    let mut tuesday: Signal<bool> = use_signal(|| false);
    let mut wednesday: Signal<bool> = use_signal(|| false);
    let mut thursday: Signal<bool> = use_signal(|| false);
    let mut friday: Signal<bool> = use_signal(|| false);
    let mut saturday: Signal<bool> = use_signal(|| false);
    let mut sunday: Signal<bool> = use_signal(|| false);
    let current_week: ImStr = format!("{}", js::get_current_week()).into();

    let _cr = use_coroutine(|_rx: UnboundedReceiver<()>| async move {
        to_owned![weeks];
        if let Some(loaded_weeks) = error::result_handler(loader::load_weeks(config, 2024).await) {
            *weeks.write() = loaded_weeks;
        }
    });

    rsx! {
        Header {
            {title_str}
        }

        Form {
            FormPair {
                label: week_str,
                SimpleSelect {
                    selected_key: current_week,
                    options: weeks.read().iter().map(|week| SimpleOption {
                        key: format!("{}", week.week).into(),
                        text: i18n.format_week(&week),
                    }).collect::<Vec<_>>().into(),
                }
            }
            FormItem {
                Checkbox {
                    value: *whole_week.read(),
                    {full_week_str},
                    on_change: move |value| {
                        *whole_week.write() = value;
                    }
                }
            }
            if !*whole_week.read() {
                FormItem {
                    Checkbox {
                        value: *monday.read(),
                        "Monday",
                        on_change: move |value| {
                            *monday.write() = value;
                        }
                    }
                }
                FormItem {
                    Checkbox {
                        value: *tuesday.read(),
                        "Tuesday",
                        on_change: move |value| {
                            *tuesday.write() = value;
                        }
                    }
                }
                FormItem {
                    Checkbox {
                        value: *wednesday.read(),
                        "Wednesday",
                        on_change: move |value| {
                            *wednesday.write() = value;
                        }
                    }
                }
                FormItem {
                    Checkbox {
                        value: *thursday.read(),
                        "Thursday",
                        on_change: move |value| {
                            *thursday.write() = value;
                        }
                    }
                }
                FormItem {
                    Checkbox {
                        value: *friday.read(),
                        "Friday",
                        on_change: move |value| {
                            *friday.write() = value;
                        }
                    }
                }
                FormItem {
                    Checkbox {
                        value: *saturday.read(),
                        "Saturday",
                        on_change: move |value| {
                            *saturday.write() = value;
                        }
                    }
                }
                FormItem {
                    Checkbox {
                        value: *sunday.read(),
                        "Sunday",
                        on_change: move |value| {
                            *sunday.write() = value;
                        }
                    }
                }
            }
            FormGroup {
                Button {
                    "Abort"
                }
                Button {
                    "Submit"
                }
            }
        }
    }
}
