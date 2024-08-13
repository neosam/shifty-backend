use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    base_types::ImStr,
    component::base_components::*,
    error,
    i18n::Key,
    js, loader,
    service::{CONFIG, I18N},
    state::week::Week,
};

#[allow(dead_code)]
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

#[allow(dead_code)]
#[component]
pub fn AddExtraDaysForm(props: AddExtraDaysFormProps) -> Element {
    let i18n = I18N.read().clone();
    let config = CONFIG.read().clone();

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
        Header { {title_str} }

        Form {
            FormPair { label: week_str,
                SimpleSelect {
                    selected_key: current_week,
                    options: weeks
                        .read()
                        .iter()
                        .map(|week| SimpleOption {
                            key: format!("{}", week.week).into(),
                            text: i18n.format_week(&week),
                        })
                        .collect::<Vec<_>>()
                        .into()
                }
            }
            FormItem {
                Checkbox {
                    value: *whole_week.read(),
                    on_change: move |value| {
                        *whole_week.write() = value;
                    },
                    {full_week_str}
                }
            }
            if !*whole_week.read() {
                FormItem {
                    Checkbox {
                        value: *monday.read(),
                        on_change: move |value| {
                            *monday.write() = value;
                        },
                        "Monday"
                    }
                }
                FormItem {
                    Checkbox {
                        value: *tuesday.read(),
                        on_change: move |value| {
                            *tuesday.write() = value;
                        },
                        "Tuesday"
                    }
                }
                FormItem {
                    Checkbox {
                        value: *wednesday.read(),
                        on_change: move |value| {
                            *wednesday.write() = value;
                        },
                        "Wednesday"
                    }
                }
                FormItem {
                    Checkbox {
                        value: *thursday.read(),
                        on_change: move |value| {
                            *thursday.write() = value;
                        },
                        "Thursday"
                    }
                }
                FormItem {
                    Checkbox {
                        value: *friday.read(),
                        on_change: move |value| {
                            *friday.write() = value;
                        },
                        "Friday"
                    }
                }
                FormItem {
                    Checkbox {
                        value: *saturday.read(),
                        on_change: move |value| {
                            *saturday.write() = value;
                        },
                        "Saturday"
                    }
                }
                FormItem {
                    Checkbox {
                        value: *sunday.read(),
                        on_change: move |value| {
                            *sunday.write() = value;
                        },
                        "Sunday"
                    }
                }
            }
            FormGroup {
                Button { "Abort" }
                Button { "Submit" }
            }
        }
    }
}
