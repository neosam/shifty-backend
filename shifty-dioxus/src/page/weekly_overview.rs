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
                button { onclick: move |_| cr.send(WeeklyOverviewPageAction::PreviousYear),
                    "<"
                }
                span { class: "mx-4", "{year.read()}" }
                button { onclick: move |_| cr.send(WeeklyOverviewPageAction::NextYear),
                    ">"
                }
            }
            div {
                h2 { class: "text-xl font-bold", "Weeks" }
                for week in weekly_summary.iter() {
                    div {
                        h3 { class: "text-lg font-bold", "{week.year} / {week.week}" }
                        p { class: "mt-2",
                            "Available hours: {week.available_hours} / {week.required_hours}."
                        }
                    }
                }
            }
        }
    }
}
