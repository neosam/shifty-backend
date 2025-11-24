use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    i18n::Key,
    service::i18n::I18N,
    state::booking_log::BookingLog,
};

#[derive(PartialEq, Clone, Props)]
pub struct BookingLogTableProps {
    pub bookings: Rc<[BookingLog]>,
}

#[component]
pub fn BookingLogTable(props: BookingLogTableProps) -> Element {
    let i18n = I18N.read().clone();

    // Sort bookings by day of week, then by time
    let mut bookings: Vec<BookingLog> = props.bookings.iter().cloned().collect();
    bookings.sort_by(|a, b| {
        let day_cmp = a.day_of_week.num_from_monday().cmp(&b.day_of_week.num_from_monday());
        if day_cmp != std::cmp::Ordering::Equal {
            day_cmp
        } else {
            a.time_from.cmp(&b.time_from)
        }
    });

    rsx! {
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
