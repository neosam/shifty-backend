use std::collections::BTreeMap;
use std::sync::Arc;

use dioxus::prelude::*;
use rest_types::BlockTO;

use crate::{
    component::TopBar,
    i18n::Key,
    js, loader,
    service::{config::CONFIG, i18n::I18N},
    state::{shiftplan::Weekday, week::Week},
};

#[component]
pub fn MyShifts() -> Element {
    let config = CONFIG.read().clone();
    let i18n = I18N.read().clone();

    // Calculate current week and 10 weeks ahead
    let from_year = js::get_current_year();
    let from_week = js::get_current_week();

    // Calculate to_year and to_week (10 weeks from now)
    let (to_year, to_week) = {
        let weeks_ahead = 10;
        let mut year = from_year;
        let mut week = from_week + weeks_ahead;

        // Handle year overflow (assuming 52 weeks per year for simplicity)
        while week > 52 {
            week -= 52;
            year += 1;
        }

        (year, week)
    };

    // Fetch blocks using use_resource
    let blocks = use_resource(move || {
        let config = config.clone();
        async move { loader::load_blocks(config, from_year, from_week, to_year, to_week).await }
    });

    // State for expanded weeks (all expanded by default)
    let mut expanded_weeks: Signal<Vec<(u32, u8)>> = use_signal(Vec::new);
    let mut initialized: Signal<bool> = use_signal(|| false);

    rsx! {
        TopBar {}
        div { class: "px-4 py-4 md:px-6",
            h1 { class: "text-2xl font-bold mb-4",
                "{i18n.t(Key::MyShifts)}"
            }

            match &*blocks.read() {
                Some(Ok(block_list)) => {
                    // Group blocks by (year, week)
                    let grouped: BTreeMap<(u32, u8), Vec<BlockTO>> = {
                        let mut grouped: BTreeMap<(u32, u8), Vec<BlockTO>> = BTreeMap::new();
                        for block in block_list.iter() {
                            grouped
                                .entry((block.year, block.week))
                                .or_default()
                                .push(block.clone());
                        }
                        grouped
                    };

                    // Initialize expanded_weeks on first load (expand all weeks)
                    if !*initialized.read() && !grouped.is_empty() {
                        let keys: Vec<(u32, u8)> = grouped.keys().cloned().collect();
                        expanded_weeks.set(keys);
                        initialized.set(true);
                    }

                    if grouped.is_empty() {
                        rsx! {
                            div { class: "text-gray-500 p-4",
                                "{i18n.t(Key::NoShiftsFound)}"
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "space-y-4",
                                for (year, week) in grouped.keys() {
                                    {
                                        let year = *year;
                                        let week = *week;
                                        let is_expanded = expanded_weeks.read().contains(&(year, week));
                                        let week_blocks = grouped.get(&(year, week)).cloned().unwrap_or_default();

                                        // Calculate week date range
                                        let week_info = Week { year, week };
                                        let monday_date = week_info.monday().ok();
                                        let sunday_date = week_info.sunday().ok();

                                        let date_range = match (monday_date, sunday_date) {
                                            (Some(mon), Some(sun)) => {
                                                format!("{} - {}", i18n.format_date(&mon), i18n.format_date(&sun))
                                            }
                                            _ => String::new(),
                                        };

                                        rsx! {
                                            div { class: "border rounded-lg overflow-hidden",
                                                // Week header (clickable)
                                                button {
                                                    class: "w-full px-4 py-3 bg-gray-100 hover:bg-gray-200 flex items-center justify-between text-left font-semibold",
                                                    onclick: move |_| {
                                                        let key = (year, week);
                                                        let mut current = expanded_weeks.read().clone();
                                                        if current.contains(&key) {
                                                            current.retain(|k| *k != key);
                                                        } else {
                                                            current.push(key);
                                                        }
                                                        expanded_weeks.set(current);
                                                    },
                                                    span {
                                                        "{i18n.t(Key::WeekLabel)} {week} ({year}): {date_range}"
                                                    }
                                                    span { class: "text-lg",
                                                        if is_expanded { "▼" } else { "►" }
                                                    }
                                                }

                                                // Week content (collapsible)
                                                if is_expanded {
                                                    div { class: "p-4 space-y-2",
                                                        for block in week_blocks.iter() {
                                                            {
                                                                let weekday: Weekday = block.day_of_week.into();
                                                                let weekday_name = weekday.i18n_string(&i18n);

                                                                // Calculate the specific date for this block
                                                                let block_date = monday_date
                                                                    .and_then(|mon| mon.checked_add(time::Duration::days(weekday.num_from_monday() as i64)));
                                                                let date_str = block_date
                                                                    .map(|d| i18n.format_date(&d))
                                                                    .unwrap_or_default();

                                                                let time_from = format!("{:02}:{:02}", block.from.hour(), block.from.minute());
                                                                let time_to = format!("{:02}:{:02}", block.to.hour(), block.to.minute());
                                                                let sales_person_name: Arc<str> = block
                                                                    .sales_person
                                                                    .as_ref()
                                                                    .map(|sp| sp.name.clone())
                                                                    .unwrap_or_else(|| "-".into());

                                                                rsx! {
                                                                    div { class: "flex items-center space-x-4 p-2 bg-white border rounded",
                                                                        span { class: "font-medium w-40",
                                                                            "{weekday_name}, {date_str}"
                                                                        }
                                                                        span { class: "text-gray-600",
                                                                            "{time_from} - {time_to}"
                                                                        }
                                                                        span { class: "text-gray-800 ml-auto",
                                                                            "{sales_person_name}"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(err)) => {
                    rsx! {
                        div { class: "text-red-600 p-4",
                            "Error loading shifts: {err}"
                        }
                    }
                }
                None => {
                    rsx! {
                        div { class: "text-gray-500 p-4",
                            "Loading..."
                        }
                    }
                }
            }
        }
    }
}
