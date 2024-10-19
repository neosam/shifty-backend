use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::{
    component::TopBar,
    js,
    service::{WeeklySummaryAction, WEEKLY_SUMMARY_STORE},
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
            h1 { class: "text-2xl font-bold", "Weekly Overview" }
            p { class: "mt-8 mb-8", "This is a page that shows the weekly overview." }
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
                table { class: "table-fixed w-full md:w-1/2",
                    thead { class: "text-left",
                        tr {
                            th { class: "pl-2 pr-2", "Week" }
                            th { class: "pl-2 pr-2", "Available / Required hours" }
                            th { class: "pl-2 pr-2", "Missing hours" }
                        }
                    }
                    tbody {
                        for week in weekly_summary.iter() {
                            tr { class: "content-center",
                                td { "{week.year} / {week.week}" }
                                td { "{week.available_hours} / {week.required_hours}" }
                                td { "{week.required_hours - week.available_hours}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
