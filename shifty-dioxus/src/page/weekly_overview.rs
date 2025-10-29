use crate::i18n::Key;
use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{
    component::TopBar,
    js,
    service::{
        i18n::I18N, weekly_summary::WeeklySummaryAction, weekly_summary::WEEKLY_SUMMARY_STORE,
    },
};

pub enum WeeklyOverviewPageAction {
    NextYear,
    PreviousYear,
}

#[component]
pub fn WeeklyOverview() -> Element {
    let year = use_signal(|| js::get_current_year());
    let weekly_overview_service = use_coroutine_handle::<WeeklySummaryAction>();
    let weekly_summary = WEEKLY_SUMMARY_STORE.read().clone();
    let i18n = I18N.read().clone();

    let title = i18n.t(Key::WeeklyOverviewTitle);
    let week_label = i18n.t(Key::WeekLabel);
    let available_required_hours = i18n.t(Key::AvailableRequiredHours);
    let missing_hours = i18n.t(Key::MissingHours);

    let cr = use_coroutine({
        to_owned![year];

        let load_data = move || async move {
            weekly_overview_service.send(WeeklySummaryAction::LoadYear(*year.read()))
        };

        move |mut rx: UnboundedReceiver<WeeklyOverviewPageAction>| async move {
            load_data().await;
            while let Some(action) = rx.next().await {
                match action {
                    WeeklyOverviewPageAction::NextYear => {
                        *year.write() += 1;
                        load_data().await;
                    }
                    WeeklyOverviewPageAction::PreviousYear => {
                        *year.write() -= 1;
                        load_data().await;
                    }
                }
            }
        }
    });

    rsx! {
        TopBar {}
        div { class: "m-4",
            h1 { class: "text-2xl font-bold mb-6", "{title}" }
            if weekly_summary.data_loaded {
                div {
                    button {
                        onclick: move |_| cr.send(WeeklyOverviewPageAction::PreviousYear),
                        class: "border-2 border-solid border-black mr-2 pt-2 pb-2 pl-4 pr-4 text-xl font-bold print:hidden",
                        "<"
                    }
                    span { class: "mx-4 mr-6", "{year.read()}" }
                    button {
                        onclick: move |_| cr.send(WeeklyOverviewPageAction::NextYear),
                        class: "border-2 border-solid border-black mr-2 pt-2 pb-2 pl-4 pr-4 text-xl font-bold print:hidden",
                        ">"
                    }
                }
                div {
                    table { class: "table-auto w-full mt-4",
                        thead { class: "text-left",
                            tr { class: "border-b border-black",
                                th { class: "pl-2 pr-2", "{week_label}" }
                                th { class: "pl-2 pr-2", "Paid/Volunteer" }
                                th { class: "pl-2 pr-2", "{available_required_hours}" }
                                th { class: "pl-2 pr-2", "{missing_hours}" }
                            }
                        }
                        tbody {
                            for week in weekly_summary.weekly_summary.iter() {
                                tr { class: "content-center border-b",
                                    td { class: "pb-2 pt-2 underline",
                                        Link { to: format!("/shiftplan/{}/{}", week.year, week.week),
                                            div { class: "font-bold", "{week.year} / {week.week}" }
                                            div {
                                                "{i18n.format_date(&week.monday_date())} - {i18n.format_date(&week.sunday_date())}"
                                            }
                                        }
                                    }
                                    td { "💰{week.paid_hours:.2} | 🤝{(week.volunteer_hours+0.0):.2}" }
                                    td { "{week.available_hours:.2} / {week.required_hours:.2}" }
                                    if week.required_hours - week.available_hours < 0.0 {
                                        td { class: "text-green-500",
                                            "+ {week.available_hours - week.required_hours:.2}"
                                        }
                                    } else if week.required_hours - week.available_hours > 20.0 {
                                        td { class: "text-red-500",
                                            "- {week.required_hours - week.available_hours:.2}"
                                        }
                                    } else if week.required_hours - week.available_hours > 7.0 {
                                        td { class: "text-yellow-700",
                                            "- {week.required_hours - week.available_hours:.2}"
                                        }
                                    } else {
                                        td { "- {week.required_hours - week.available_hours:.2}" }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "text-center", "Loading data..." }
            }
        }
    }
}
