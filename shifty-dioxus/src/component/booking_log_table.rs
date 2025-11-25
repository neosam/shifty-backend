use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    base_types::ImStr,
    i18n::Key,
    service::i18n::I18N,
    state::{booking_log::BookingLog, Weekday},
};

#[derive(PartialEq, Clone, Props)]
pub struct BookingLogTableProps {
    pub bookings: Rc<[BookingLog]>,
    pub name_filter: String,
    pub on_name_filter_change: EventHandler<String>,
    pub day_filter: Option<Weekday>,
    pub on_day_filter_change: EventHandler<Option<Weekday>>,
    pub status_filter: String,
    pub on_status_filter_change: EventHandler<String>,
    pub created_by_filter: String,
    pub on_created_by_filter_change: EventHandler<String>,
    pub on_clear_filters: EventHandler<()>,
}

#[component]
pub fn BookingLogTable(props: BookingLogTableProps) -> Element {
    let i18n = I18N.read().clone();

    // Get unique list of creators for the dropdown
    let creators: Vec<String> = {
        let mut creators: Vec<String> = props
            .bookings
            .iter()
            .map(|b| b.created_by.to_string())
            .collect();
        creators.sort();
        creators.dedup();
        creators
    };

    // Apply filters
    let filtered_bookings: Vec<BookingLog> = props
        .bookings
        .iter()
        .filter(|b| {
            // Name filter (case-insensitive)
            if !props.name_filter.is_empty() {
                if !b
                    .sales_person_name
                    .to_lowercase()
                    .contains(&props.name_filter.to_lowercase())
                {
                    return false;
                }
            }

            // Day filter
            if let Some(day) = props.day_filter {
                if b.day_of_week != day {
                    return false;
                }
            }

            // Status filter
            match props.status_filter.as_str() {
                "active" => {
                    if b.deleted.is_some() {
                        return false;
                    }
                }
                "deleted" => {
                    if b.deleted.is_none() {
                        return false;
                    }
                }
                _ => {} // "all" - no filtering
            }

            // Created by filter
            if !props.created_by_filter.is_empty() && props.created_by_filter != "all" {
                if b.created_by.as_ref() != props.created_by_filter {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect();

    // Sort filtered bookings by day of week, then by time
    let mut bookings = filtered_bookings;
    bookings.sort_by(|a, b| {
        let day_cmp = a
            .day_of_week
            .num_from_monday()
            .cmp(&b.day_of_week.num_from_monday());
        if day_cmp != std::cmp::Ordering::Equal {
            day_cmp
        } else {
            a.time_from.cmp(&b.time_from)
        }
    });

    rsx! {
        div { class: "space-y-4",
            // Filter section
            div { class: "bg-gray-50 p-4 rounded-lg border border-gray-200",
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4",
                    // Name filter (search input)
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            {i18n.t(Key::BookingLogFilterName)}
                        }
                        input {
                            class: "w-full p-2 border border-gray-300 rounded-md",
                            r#type: "text",
                            value: "{props.name_filter}",
                            placeholder: "Search...",
                            oninput: move |event| props.on_name_filter_change.call(event.value())
                        }
                    }

                    // Day filter (dropdown)
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            {i18n.t(Key::BookingLogFilterDay)}
                        }
                        select {
                            class: "w-full p-2 border border-gray-300 rounded-md",
                            value: match props.day_filter {
                                Some(Weekday::Monday) => "Monday",
                                Some(Weekday::Tuesday) => "Tuesday",
                                Some(Weekday::Wednesday) => "Wednesday",
                                Some(Weekday::Thursday) => "Thursday",
                                Some(Weekday::Friday) => "Friday",
                                Some(Weekday::Saturday) => "Saturday",
                                Some(Weekday::Sunday) => "Sunday",
                                None => "all",
                            },
                            onchange: move |event| {
                                let day = match event.value().as_str() {
                                    "Monday" => Some(Weekday::Monday),
                                    "Tuesday" => Some(Weekday::Tuesday),
                                    "Wednesday" => Some(Weekday::Wednesday),
                                    "Thursday" => Some(Weekday::Thursday),
                                    "Friday" => Some(Weekday::Friday),
                                    "Saturday" => Some(Weekday::Saturday),
                                    "Sunday" => Some(Weekday::Sunday),
                                    _ => None,
                                };
                                props.on_day_filter_change.call(day);
                            },
                            option { value: "all", {i18n.t(Key::BookingLogFilterAll)} }
                            option { value: "Monday", {i18n.t(Key::Monday)} }
                            option { value: "Tuesday", {i18n.t(Key::Tuesday)} }
                            option { value: "Wednesday", {i18n.t(Key::Wednesday)} }
                            option { value: "Thursday", {i18n.t(Key::Thursday)} }
                            option { value: "Friday", {i18n.t(Key::Friday)} }
                            option { value: "Saturday", {i18n.t(Key::Saturday)} }
                            option { value: "Sunday", {i18n.t(Key::Sunday)} }
                        }
                    }

                    // Status filter (dropdown)
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            {i18n.t(Key::BookingLogFilterStatus)}
                        }
                        select {
                            class: "w-full p-2 border border-gray-300 rounded-md",
                            value: "{props.status_filter}",
                            onchange: move |event| props.on_status_filter_change.call(event.value()),
                            option { value: "all", {i18n.t(Key::BookingLogFilterAll)} }
                            option { value: "active", {i18n.t(Key::BookingLogFilterActiveOnly)} }
                            option { value: "deleted", {i18n.t(Key::BookingLogFilterDeletedOnly)} }
                        }
                    }

                    // Created by filter (dropdown)
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            {i18n.t(Key::BookingLogFilterCreatedBy)}
                        }
                        select {
                            class: "w-full p-2 border border-gray-300 rounded-md",
                            value: "{props.created_by_filter}",
                            onchange: move |event| props.on_created_by_filter_change.call(event.value()),
                            option { value: "all", {i18n.t(Key::BookingLogFilterAll)} }
                            for creator in creators.iter() {
                                option { value: "{creator}", "{creator}" }
                            }
                        }
                    }

                    // Clear filters button
                    div { class: "flex items-end",
                        button {
                            class: "w-full bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded",
                            onclick: move |_| props.on_clear_filters.call(()),
                            {i18n.t(Key::BookingLogFilterClear)}
                        }
                    }
                }
            }

            // Table section
            div { class: "overflow-x-auto",
                table { class: "min-w-full border-collapse border border-gray-300",
                    thead {
                        tr { class: "bg-gray-100",
                            th { class: "border border-gray-300 px-4 py-2 text-left font-semibold", {i18n.t(Key::BookingLogDay)} }
                            th { class: "border border-gray-300 px-4 py-2 text-left font-semibold", {i18n.t(Key::BookingLogName)} }
                            th { class: "border border-gray-300 px-4 py-2 text-left font-semibold", {i18n.t(Key::BookingLogTime)} }
                            th { class: "border border-gray-300 px-4 py-2 text-left font-semibold", {i18n.t(Key::BookingLogCreated)} }
                            th { class: "border border-gray-300 px-4 py-2 text-left font-semibold", {i18n.t(Key::BookingLogCreatedBy)} }
                            th { class: "border border-gray-300 px-4 py-2 text-left font-semibold", {i18n.t(Key::BookingLogDeleted)} }
                            th { class: "border border-gray-300 px-4 py-2 text-left font-semibold", {i18n.t(Key::BookingLogDeletedBy)} }
                        }
                    }
                    tbody {
                        for booking in bookings.iter() {
                            {
                                let is_deleted = booking.deleted.is_some();
                                let day_str = booking.day_of_week.i18n_string(&i18n);
                                let time_str = format!("{} - {}", booking.time_from, booking.time_to);
                                let created_str = format!("{} {}",
                                    i18n.format_date(&booking.created.date()),
                                    booking.created.time()
                                );
                                let deleted_str = booking.deleted
                                    .as_ref()
                                    .map(|dt| format!("{} {}", i18n.format_date(&dt.date()), dt.time()))
                                    .unwrap_or_else(|| "".to_string());
                                let deleted_by_str = booking.deleted_by.as_ref()
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| "".to_string());

                                rsx! {
                                    tr {
                                        class: format!(
                                            "hover:bg-gray-50 {}",
                                            if is_deleted { "line-through text-gray-500" } else { "" }
                                        ),
                                        td { class: "border border-gray-200 px-4 py-2", {day_str} }
                                        td { class: "border border-gray-200 px-4 py-2", {&booking.sales_person_name} }
                                        td { class: "border border-gray-200 px-4 py-2 whitespace-nowrap", {time_str} }
                                        td { class: "border border-gray-200 px-4 py-2 text-sm whitespace-nowrap", {created_str} }
                                        td { class: "border border-gray-200 px-4 py-2", {&booking.created_by} }
                                        td { class: "border border-gray-200 px-4 py-2 text-sm whitespace-nowrap", {deleted_str} }
                                        td { class: "border border-gray-200 px-4 py-2", {deleted_by_str} }
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
