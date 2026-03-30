use std::rc::Rc;

use crate::{
    component::week_view::{ColumnView, ColumnViewItem, TimeView, WeekViewButtonTypes},
    service::i18n::I18N,
    state::{dropdown::DropdownEntry, DayAggregate, Slot, Weekday},
};
use dioxus::prelude::*;
use uuid::Uuid;

const SCALING: f32 = 75.0;

fn slot_to_column_view_item(
    slot: Slot,
    is_shiftplanner: bool,
    i18n: &crate::i18n::I18n<crate::i18n::Key, crate::i18n::Locale>,
) -> ColumnViewItem<Slot> {
    use crate::component::week_view::{ColumnViewContent, ColumnViewContentItem};

    let mut bookings: Vec<ColumnViewContentItem> = slot
        .bookings
        .iter()
        .map(|booking| {
            let tooltip = if is_shiftplanner {
                match (&booking.created, &booking.created_by) {
                    (Some(created), Some(created_by)) => {
                        let date_str = i18n.format_date(&created.date());
                        let time_str = created.time();
                        Some(
                            format!(
                                "{} {} {} {}",
                                i18n.t(crate::i18n::Key::BookingLogCreatedBy),
                                created_by,
                                date_str,
                                time_str
                            )
                            .into(),
                        )
                    }
                    _ => Some(i18n.t(crate::i18n::Key::BookingNoInfo)),
                }
            } else {
                None
            };

            ColumnViewContentItem {
                id: booking.sales_person_id,
                title: if booking.self_added {
                    format!("{}*", booking.label).into()
                } else {
                    booking.label.clone()
                },
                background_color: booking.background_color.clone(),
                tooltip,
            }
        })
        .collect();

    bookings.insert(
        0,
        ColumnViewContentItem {
            id: uuid::Uuid::nil(),
            title: format!("{}/{}", slot.bookings.len(), slot.min_resources).into(),
            background_color: if slot.bookings.len() != slot.min_resources as usize {
                "#ffcccc".into()
            } else {
                "#fff".into()
            },
            tooltip: None,
        },
    );

    ColumnViewItem {
        start: slot.from_hour(),
        end: slot.to_hour(),
        show_add: true,
        show_remove: true,
        title: ColumnViewContent::Items(bookings.into()),
        warning: if slot.evaluation().is_faulty() {
            Some("Too few resources".into())
        } else {
            None
        },
        dropdown_entries: None,
        custom_data: slot,
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct DayAggregateViewProps {
    pub day_aggregate: DayAggregate,
    pub highlight_item_id: Option<Uuid>,
    pub add_event: Option<EventHandler<Slot>>,
    pub remove_event: Option<EventHandler<Slot>>,
    pub item_clicked: Option<EventHandler<Uuid>>,
    pub button_types: WeekViewButtonTypes,
    pub dropdown_entries: Option<Rc<[DropdownEntry]>>,

    #[props(default = false)]
    pub is_shiftplanner: bool,
}

enum Zoom {
    Full,
    Half,
    Quarter,
}

#[component]
pub fn DayAggregateView(props: DayAggregateViewProps) -> Element {
    let i18n = I18N.read().clone();
    let day_start = props.day_aggregate.min_hour();
    let day_end = props.day_aggregate.max_hour();
    let mut zoom = use_signal(|| Zoom::Full);
    let zoom_class = match *zoom.read() {
        Zoom::Full => "scale-down-100",
        Zoom::Half => "scale-down-50",
        Zoom::Quarter => "scale-down-75",
    };

    if props.day_aggregate.plans.is_empty() || day_start >= day_end {
        return rsx! {
            div { class: "m-4 text-gray-500 italic", "No plans for this day." }
        };
    }

    rsx! {
        div {
            class: "overflow-y-scroll overflow-visible no-scrollbar print:width-full print:overflow-visible",
            style: format!("height: {}px", (day_end - day_start) as f32 * SCALING + SCALING),
            div { class: "fixed bottom-4 left-4 z-50 border bg-white p-2 rounded-md shadow-lg 2xl:hidden print:hidden",
                label { "Zoom: " }
                select {
                    onchange: move |event| {
                        let value = event.data.value();
                        match value.as_str() {
                            "full" => *zoom.write() = Zoom::Full,
                            "half" => *zoom.write() = Zoom::Half,
                            "quarter" => *zoom.write() = Zoom::Quarter,
                            _ => {}
                        }
                    },
                    option { value: "full", "100%" }
                    option { value: "quarter", "75%" }
                    option { value: "half", "50%" }
                }
            }
            div { class: format!("flex flex-row {}", zoom_class),
                div { class: "flex-shrink-0 sticky left-0 z-10 bg-white border-r border-gray-200",
                    TimeView {
                        start: day_start.ceil() as u8,
                        end: day_end.ceil() as u8,
                    }
                }
                div { class: "flex flex-row overflow-x-auto flex-grow",
                    for plan in props.day_aggregate.plans.iter() {
                        {
                            let slots: Rc<[ColumnViewItem<Slot>]> = plan
                                .slots
                                .iter()
                                .map(|slot| {
                                    let mut item = slot_to_column_view_item(slot.clone(), props.is_shiftplanner, &i18n);
                                    item.start -= day_start;
                                    item.end -= day_start;
                                    item.dropdown_entries = props.dropdown_entries.clone();
                                    item
                                })
                                .collect();
                            rsx! {
                                ColumnView::<Slot> {
                                    height: (day_end - day_start) as f32 * SCALING + SCALING / 2.0,
                                    scale: SCALING,
                                    offset: SCALING / 2.0,
                                    slots: slots,
                                    title: Some(plan.shiftplan_name.clone().into()),
                                    highlight_item_id: props.highlight_item_id,
                                    add_event: props.add_event.clone(),
                                    remove_event: props.remove_event.clone(),
                                    item_clicked: props.item_clicked.clone(),
                                    button_types: props.button_types.clone(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Determine the default day to select when toggling to day view.
/// If viewing the current ISO week, select today's weekday.
/// Otherwise, select Monday.
pub fn default_day_for_week(year: u32, week: u8) -> Weekday {
    let now = crate::js::current_datetime();
    let current_year = now.to_iso_week_date().0 as u32;
    let current_week = now.iso_week();
    if year == current_year && week == current_week {
        match now.weekday() {
            time::Weekday::Monday => Weekday::Monday,
            time::Weekday::Tuesday => Weekday::Tuesday,
            time::Weekday::Wednesday => Weekday::Wednesday,
            time::Weekday::Thursday => Weekday::Thursday,
            time::Weekday::Friday => Weekday::Friday,
            time::Weekday::Saturday => Weekday::Saturday,
            time::Weekday::Sunday => Weekday::Sunday,
        }
    } else {
        Weekday::Monday
    }
}

/// Navigate to the next day, wrapping to next week's Monday if at the end.
/// Returns (new_weekday, week_changed) where week_changed is +1 if wrapped forward.
pub fn next_day(current: Weekday, has_sunday: bool) -> (Weekday, i8) {
    let last_day = if has_sunday {
        Weekday::Sunday
    } else {
        Weekday::Saturday
    };
    if current == last_day {
        (Weekday::Monday, 1)
    } else {
        let num = current.num_from_monday();
        (Weekday::from_num_from_monday(num + 1), 0)
    }
}

/// Navigate to the previous day, wrapping to previous week's last day if at Monday.
/// Returns (new_weekday, week_changed) where week_changed is -1 if wrapped backward.
pub fn prev_day(current: Weekday, has_sunday: bool) -> (Weekday, i8) {
    if current == Weekday::Monday {
        let last_day = if has_sunday {
            Weekday::Sunday
        } else {
            Weekday::Saturday
        };
        (last_day, -1)
    } else {
        let num = current.num_from_monday();
        (Weekday::from_num_from_monday(num - 1), 0)
    }
}

/// Check if any plan in the catalog has Sunday slots.
/// This is a simple heuristic - for day aggregate, we check the loaded data.
pub fn has_sunday_slots(day_aggregate: &Option<DayAggregate>) -> bool {
    day_aggregate
        .as_ref()
        .map(|agg| {
            agg.plans
                .iter()
                .any(|p| p.slots.iter().any(|s| s.day_of_week == Weekday::Sunday))
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::shiftplan::{DayPlanColumn, Slot};
    use std::rc::Rc;

    fn make_slot(day: Weekday) -> Slot {
        Slot {
            id: uuid::Uuid::nil(),
            day_of_week: day,
            from: time::Time::from_hms(8, 0, 0).unwrap(),
            to: time::Time::from_hms(16, 0, 0).unwrap(),
            bookings: Rc::new([]),
            min_resources: 1,
        }
    }

    fn make_day_aggregate(slots: Vec<Slot>) -> DayAggregate {
        DayAggregate {
            year: 2026,
            week: 14,
            day_of_week: Weekday::Monday,
            plans: vec![DayPlanColumn {
                shiftplan_name: "Test".into(),
                shiftplan_id: uuid::Uuid::nil(),
                slots: slots.into(),
            }]
            .into(),
        }
    }

    // --- next_day tests ---

    #[test]
    fn next_day_from_monday() {
        let (day, week_change) = next_day(Weekday::Monday, false);
        assert_eq!(day, Weekday::Tuesday);
        assert_eq!(week_change, 0);
    }

    #[test]
    fn next_day_from_friday() {
        let (day, week_change) = next_day(Weekday::Friday, false);
        assert_eq!(day, Weekday::Saturday);
        assert_eq!(week_change, 0);
    }

    #[test]
    fn next_day_from_saturday_no_sunday() {
        let (day, week_change) = next_day(Weekday::Saturday, false);
        assert_eq!(day, Weekday::Monday);
        assert_eq!(week_change, 1);
    }

    #[test]
    fn next_day_from_saturday_with_sunday() {
        let (day, week_change) = next_day(Weekday::Saturday, true);
        assert_eq!(day, Weekday::Sunday);
        assert_eq!(week_change, 0);
    }

    #[test]
    fn next_day_from_sunday_with_sunday() {
        let (day, week_change) = next_day(Weekday::Sunday, true);
        assert_eq!(day, Weekday::Monday);
        assert_eq!(week_change, 1);
    }

    // --- prev_day tests ---

    #[test]
    fn prev_day_from_tuesday() {
        let (day, week_change) = prev_day(Weekday::Tuesday, false);
        assert_eq!(day, Weekday::Monday);
        assert_eq!(week_change, 0);
    }

    #[test]
    fn prev_day_from_saturday() {
        let (day, week_change) = prev_day(Weekday::Saturday, false);
        assert_eq!(day, Weekday::Friday);
        assert_eq!(week_change, 0);
    }

    #[test]
    fn prev_day_from_monday_no_sunday() {
        let (day, week_change) = prev_day(Weekday::Monday, false);
        assert_eq!(day, Weekday::Saturday);
        assert_eq!(week_change, -1);
    }

    #[test]
    fn prev_day_from_monday_with_sunday() {
        let (day, week_change) = prev_day(Weekday::Monday, true);
        assert_eq!(day, Weekday::Sunday);
        assert_eq!(week_change, -1);
    }

    // --- has_sunday_slots tests ---

    #[test]
    fn has_sunday_slots_none() {
        assert!(!has_sunday_slots(&None));
    }

    #[test]
    fn has_sunday_slots_no_sunday() {
        let agg = make_day_aggregate(vec![make_slot(Weekday::Monday)]);
        assert!(!has_sunday_slots(&Some(agg)));
    }

    #[test]
    fn has_sunday_slots_with_sunday() {
        let agg = make_day_aggregate(vec![
            make_slot(Weekday::Monday),
            make_slot(Weekday::Sunday),
        ]);
        assert!(has_sunday_slots(&Some(agg)));
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct DayButtonBarProps {
    pub selected_day: Weekday,
    pub on_select_day: EventHandler<Weekday>,
    pub on_prev_day: EventHandler<()>,
    pub on_next_day: EventHandler<()>,
    pub show_sunday: bool,
}

#[component]
pub fn DayButtonBar(props: DayButtonBarProps) -> Element {
    let i18n = I18N.read().clone();
    let days = if props.show_sunday {
        vec![
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
            Weekday::Sunday,
        ]
    } else {
        vec![
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
        ]
    };

    rsx! {
        div { class: "flex flex-row items-center justify-center gap-1 mb-4",
            button {
                class: "border-2 border-solid border-black pt-2 pb-2 pl-4 pr-4 text-xl font-bold print:hidden",
                onclick: move |_| props.on_prev_day.call(()),
                "<"
            }
            for day in days.iter() {
                button {
                    class: format!(
                        "pt-2 pb-2 pl-3 pr-3 rounded-md font-medium {}",
                        if *day == props.selected_day {
                            "bg-blue-500 text-white"
                        } else {
                            "bg-gray-200 hover:bg-gray-300"
                        },
                    ),
                    onclick: {
                        let day = *day;
                        move |_| props.on_select_day.call(day)
                    },
                    {day.i18n_short_string(&i18n)}
                }
            }
            button {
                class: "border-2 border-solid border-black pt-2 pb-2 pl-4 pr-4 text-xl font-bold print:hidden",
                onclick: move |_| props.on_next_day.call(()),
                ">"
            }
        }
    }
}
