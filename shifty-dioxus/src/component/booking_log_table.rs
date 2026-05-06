use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    component::atoms::btn::{Btn, BtnVariant},
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

const FORM_INPUT_CLASSES: &str =
    "h-[34px] w-full px-[10px] border border-border-strong rounded-md bg-surface text-ink text-body form-input";

const HEADER_CLASSES: &str = "px-3 py-2 text-left text-micro text-ink-muted uppercase";

#[component]
pub fn BookingLogTable(props: BookingLogTableProps) -> Element {
    let i18n = I18N.read().clone();

    let creators: Vec<String> = {
        let mut creators: Vec<String> = props
            .bookings
            .iter()
            .filter_map(|b| b.created_by.as_ref().map(|s| s.to_string()))
            .collect();
        creators.sort();
        creators.dedup();
        creators
    };

    let filtered_bookings: Vec<BookingLog> = props
        .bookings
        .iter()
        .filter(|b| {
            if !props.name_filter.is_empty()
                && !b
                    .sales_person_name
                    .to_lowercase()
                    .contains(&props.name_filter.to_lowercase())
            {
                return false;
            }

            if let Some(day) = props.day_filter {
                if b.day_of_week != day {
                    return false;
                }
            }

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
                _ => {}
            }

            if !props.created_by_filter.is_empty() && props.created_by_filter != "all" {
                let creator = b.created_by.as_deref().unwrap_or("");
                if creator != props.created_by_filter {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect();

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
            div { class: "bg-surface border border-border rounded-md p-3",
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-3",
                    div {
                        label { class: "block text-micro text-ink-soft uppercase mb-1",
                            {i18n.t(Key::BookingLogFilterName)}
                        }
                        input {
                            class: FORM_INPUT_CLASSES,
                            r#type: "text",
                            value: "{props.name_filter}",
                            placeholder: i18n.t(Key::SearchPlaceholder).as_ref(),
                            oninput: move |event| props.on_name_filter_change.call(event.value()),
                        }
                    }

                    div {
                        label { class: "block text-micro text-ink-soft uppercase mb-1",
                            {i18n.t(Key::BookingLogFilterDay)}
                        }
                        select {
                            class: FORM_INPUT_CLASSES,
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

                    div {
                        label { class: "block text-micro text-ink-soft uppercase mb-1",
                            {i18n.t(Key::BookingLogFilterStatus)}
                        }
                        select {
                            class: FORM_INPUT_CLASSES,
                            value: "{props.status_filter}",
                            onchange: move |event| props.on_status_filter_change.call(event.value()),
                            option { value: "all", {i18n.t(Key::BookingLogFilterAll)} }
                            option { value: "active", {i18n.t(Key::BookingLogFilterActiveOnly)} }
                            option { value: "deleted", {i18n.t(Key::BookingLogFilterDeletedOnly)} }
                        }
                    }

                    div {
                        label { class: "block text-micro text-ink-soft uppercase mb-1",
                            {i18n.t(Key::BookingLogFilterCreatedBy)}
                        }
                        select {
                            class: FORM_INPUT_CLASSES,
                            value: "{props.created_by_filter}",
                            onchange: move |event| props.on_created_by_filter_change.call(event.value()),
                            option { value: "all", {i18n.t(Key::BookingLogFilterAll)} }
                            for creator in creators.iter() {
                                option { value: "{creator}", "{creator}" }
                            }
                        }
                    }

                    div { class: "flex items-end",
                        div { class: "w-full",
                            Btn {
                                variant: BtnVariant::Secondary,
                                on_click: move |_| props.on_clear_filters.call(()),
                                {i18n.t(Key::BookingLogFilterClear)}
                            }
                        }
                    }
                }
            }

            // Table section
            div { class: "bg-surface border border-border rounded-lg overflow-hidden",
                div { class: "overflow-x-auto",
                    table { class: "w-full text-small font-normal border-collapse",
                        thead {
                            tr { class: "bg-surface-alt text-left",
                                th { class: HEADER_CLASSES, {i18n.t(Key::BookingLogDay)} }
                                th { class: HEADER_CLASSES, {i18n.t(Key::BookingLogName)} }
                                th { class: HEADER_CLASSES, {i18n.t(Key::BookingLogTime)} }
                                th { class: HEADER_CLASSES, {i18n.t(Key::BookingLogCreated)} }
                                th { class: HEADER_CLASSES, {i18n.t(Key::BookingLogCreatedBy)} }
                                th { class: HEADER_CLASSES, {i18n.t(Key::BookingLogDeleted)} }
                                th { class: HEADER_CLASSES, {i18n.t(Key::BookingLogDeletedBy)} }
                            }
                        }
                        tbody {
                            for booking in bookings.iter() {
                                {
                                    let is_deleted = booking.deleted.is_some();
                                    let day_str = booking.day_of_week.i18n_string(&i18n);
                                    let time_str = format!("{} - {}", booking.time_from, booking.time_to);
                                    let created_str = format!(
                                        "{} {}",
                                        i18n.format_date(&booking.created.date()),
                                        booking.created.time()
                                    );
                                    let deleted_str = booking
                                        .deleted
                                        .as_ref()
                                        .map(|dt| {
                                            format!("{} {}", i18n.format_date(&dt.date()), dt.time())
                                        })
                                        .unwrap_or_else(|| "—".to_string());
                                    let deleted_by_str = booking
                                        .deleted_by
                                        .as_ref()
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| "—".to_string());
                                    let created_by_str = booking
                                        .created_by
                                        .as_ref()
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| "—".to_string());
                                    let created_by_cell_class = if booking.created_by.is_some() {
                                        "px-3 py-2 text-ink"
                                    } else {
                                        "px-3 py-2 text-ink-muted"
                                    };
                                    let row_class = if is_deleted {
                                        "border-t border-border opacity-50"
                                    } else {
                                        "border-t border-border"
                                    };
                                    let deleted_cell_class = if is_deleted {
                                        "px-3 py-2 text-bad whitespace-nowrap"
                                    } else {
                                        "px-3 py-2 text-ink-muted whitespace-nowrap"
                                    };
                                    let deleted_by_cell_class = if is_deleted {
                                        "px-3 py-2 text-bad"
                                    } else {
                                        "px-3 py-2 text-ink-muted"
                                    };

                                    rsx! {
                                        tr { class: row_class,
                                            td { class: "px-3 py-2 font-mono", {day_str} }
                                            td { class: "px-3 py-2 text-ink", {&booking.sales_person_name} }
                                            td { class: "px-3 py-2 font-mono whitespace-nowrap", {time_str} }
                                            td { class: "px-3 py-2 text-ink-muted whitespace-nowrap", {created_str} }
                                            td { class: created_by_cell_class, {created_by_str} }
                                            td { class: deleted_cell_class, {deleted_str} }
                                            td { class: deleted_by_cell_class, {deleted_by_str} }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::booking_log::BookingLog;
    use time::macros::{date, time};

    fn deleted_booking() -> BookingLog {
        BookingLog {
            year: 2026,
            calendar_week: 17,
            sales_person_name: "Lena".into(),
            day_of_week: Weekday::Monday,
            time_from: time!(09:00),
            time_to: time!(13:00),
            created: time::PrimitiveDateTime::new(date!(2026 - 04 - 20), time!(09:00)),
            created_by: Some("admin".into()),
            deleted: Some(time::PrimitiveDateTime::new(
                date!(2026 - 04 - 21),
                time!(10:00),
            )),
            deleted_by: Some("admin".into()),
        }
    }

    fn active_booking() -> BookingLog {
        BookingLog {
            year: 2026,
            calendar_week: 17,
            sales_person_name: "Tobias".into(),
            day_of_week: Weekday::Tuesday,
            time_from: time!(09:00),
            time_to: time!(13:00),
            created: time::PrimitiveDateTime::new(date!(2026 - 04 - 20), time!(09:00)),
            created_by: Some("admin".into()),
            deleted: None,
            deleted_by: None,
        }
    }

    fn legacy_booking_without_creator() -> BookingLog {
        BookingLog {
            year: 2026,
            calendar_week: 17,
            sales_person_name: "Mira".into(),
            day_of_week: Weekday::Wednesday,
            time_from: time!(09:00),
            time_to: time!(13:00),
            created: time::PrimitiveDateTime::new(date!(2026 - 04 - 20), time!(09:00)),
            created_by: None,
            deleted: None,
            deleted_by: None,
        }
    }

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn booking_log_deleted_row_carries_opacity_50() {
        fn app() -> Element {
            rsx! {
                BookingLogTable {
                    bookings: Rc::from([deleted_booking()].to_vec()),
                    name_filter: String::new(),
                    on_name_filter_change: |_| {},
                    day_filter: None,
                    on_day_filter_change: |_| {},
                    status_filter: "all".to_string(),
                    on_status_filter_change: |_| {},
                    created_by_filter: "all".to_string(),
                    on_created_by_filter_change: |_| {},
                    on_clear_filters: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("opacity-50"), "missing opacity-50: {html}");
    }

    #[test]
    fn booking_log_deleted_row_no_line_through() {
        fn app() -> Element {
            rsx! {
                BookingLogTable {
                    bookings: Rc::from([deleted_booking(), active_booking()].to_vec()),
                    name_filter: String::new(),
                    on_name_filter_change: |_| {},
                    day_filter: None,
                    on_day_filter_change: |_| {},
                    status_filter: "all".to_string(),
                    on_status_filter_change: |_| {},
                    created_by_filter: "all".to_string(),
                    on_created_by_filter_change: |_| {},
                    on_clear_filters: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("line-through"),
            "unexpected line-through: {html}"
        );
    }

    #[test]
    fn booking_log_deleted_cell_uses_text_bad() {
        fn app() -> Element {
            rsx! {
                BookingLogTable {
                    bookings: Rc::from([deleted_booking()].to_vec()),
                    name_filter: String::new(),
                    on_name_filter_change: |_| {},
                    day_filter: None,
                    on_day_filter_change: |_| {},
                    status_filter: "all".to_string(),
                    on_status_filter_change: |_| {},
                    created_by_filter: "all".to_string(),
                    on_created_by_filter_change: |_| {},
                    on_clear_filters: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("text-bad"), "missing text-bad: {html}");
    }

    #[test]
    fn booking_log_active_cell_uses_ink_muted_em_dash() {
        fn app() -> Element {
            rsx! {
                BookingLogTable {
                    bookings: Rc::from([active_booking()].to_vec()),
                    name_filter: String::new(),
                    on_name_filter_change: |_| {},
                    day_filter: None,
                    on_day_filter_change: |_| {},
                    status_filter: "all".to_string(),
                    on_status_filter_change: |_| {},
                    created_by_filter: "all".to_string(),
                    on_created_by_filter_change: |_| {},
                    on_clear_filters: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("—"), "missing em-dash: {html}");
        assert!(
            html.contains("text-ink-muted"),
            "missing text-ink-muted: {html}"
        );
    }

    #[test]
    fn booking_log_no_bad_soft_badge() {
        fn app() -> Element {
            rsx! {
                BookingLogTable {
                    bookings: Rc::from([deleted_booking()].to_vec()),
                    name_filter: String::new(),
                    on_name_filter_change: |_| {},
                    day_filter: None,
                    on_day_filter_change: |_| {},
                    status_filter: "all".to_string(),
                    on_status_filter_change: |_| {},
                    created_by_filter: "all".to_string(),
                    on_created_by_filter_change: |_| {},
                    on_clear_filters: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(!html.contains("bg-bad-soft"), "no badge expected: {html}");
    }

    #[test]
    fn booking_log_filter_section_uses_token_surface() {
        fn app() -> Element {
            rsx! {
                BookingLogTable {
                    bookings: Rc::from([active_booking()].to_vec()),
                    name_filter: String::new(),
                    on_name_filter_change: |_| {},
                    day_filter: None,
                    on_day_filter_change: |_| {},
                    status_filter: "all".to_string(),
                    on_status_filter_change: |_| {},
                    created_by_filter: "all".to_string(),
                    on_created_by_filter_change: |_| {},
                    on_clear_filters: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(html.contains("bg-surface"), "missing bg-surface: {html}");
        assert!(
            html.contains("border-border"),
            "missing border-border: {html}"
        );
    }

    #[test]
    fn booking_log_renders_em_dash_for_null_created_by() {
        fn app() -> Element {
            rsx! {
                BookingLogTable {
                    bookings: Rc::from([legacy_booking_without_creator()].to_vec()),
                    name_filter: String::new(),
                    on_name_filter_change: |_| {},
                    day_filter: None,
                    on_day_filter_change: |_| {},
                    status_filter: "all".to_string(),
                    on_status_filter_change: |_| {},
                    created_by_filter: "all".to_string(),
                    on_created_by_filter_change: |_| {},
                    on_clear_filters: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("Mira"),
            "expected booking row to render: {html}"
        );
        assert!(
            html.contains("—"),
            "expected em-dash placeholder for null created_by: {html}"
        );
    }

    #[test]
    fn booking_log_creator_filter_excludes_null_creators() {
        fn app() -> Element {
            rsx! {
                BookingLogTable {
                    bookings: Rc::from(
                        [active_booking(), legacy_booking_without_creator()].to_vec(),
                    ),
                    name_filter: String::new(),
                    on_name_filter_change: |_| {},
                    day_filter: None,
                    on_day_filter_change: |_| {},
                    status_filter: "all".to_string(),
                    on_status_filter_change: |_| {},
                    created_by_filter: "all".to_string(),
                    on_created_by_filter_change: |_| {},
                    on_clear_filters: |_| {},
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("value=\"admin\""),
            "expected admin option in creator dropdown: {html}"
        );
        // Ensure the empty creator did not produce a stray empty <option value="">
        assert!(
            !html.contains("<option value=\"\""),
            "did not expect empty creator option: {html}"
        );
    }

    #[test]
    fn booking_log_table_no_legacy_classes_in_source() {
        let source = include_str!("booking_log_table.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for forbidden in [
            "bg-gray-",
            "bg-white",
            "text-gray-",
            "text-blue-",
            "text-red-",
            "text-green-",
            "bg-blue-",
            "bg-green-",
            "bg-red-",
            "border-gray-",
            "border-black",
        ] {
            assert!(
                !production.contains(forbidden),
                "non-test source contains legacy class `{}`",
                forbidden
            );
        }
    }
}
