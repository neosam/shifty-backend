use std::rc::Rc;

use crate::{
    component::week_view::{WeekCellSlot, WeekViewButtonTypes, SCALING},
    service::i18n::I18N,
    state::{dropdown::DropdownEntry, DayAggregate, Slot, Weekday},
};
use dioxus::prelude::*;
use uuid::Uuid;

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

#[component]
pub fn DayAggregateView(props: DayAggregateViewProps) -> Element {
    let day_start = props.day_aggregate.min_hour();
    let day_end = props.day_aggregate.max_hour();

    if props.day_aggregate.plans.is_empty() || day_start >= day_end {
        return rsx! {
            div { class: "m-4 text-ink-muted italic", "No plans for this day." }
        };
    }

    let n_plans = props.day_aggregate.plans.len();
    let body_height = (day_end - day_start) * SCALING;
    let grid_template_columns = format!("76px repeat({}, minmax(160px, 1fr))", n_plans);
    let grid_min_width = (76 + n_plans as u32 * 160).max(620);
    let grid_style = format!(
        "display: grid; grid-template-columns: {}; min-width: {}px;",
        grid_template_columns, grid_min_width,
    );
    let hour_start = day_start.ceil() as u8;
    let hour_end = day_end.ceil() as u8;
    let time_col_style = format!(
        "position: sticky; left: 0; z-index: 2; height: {}px;",
        body_height,
    );
    let day_col_style = format!("height: {}px;", body_height);

    rsx! {
        div { class: "bg-surface border border-border rounded-lg overflow-auto print:overflow-visible",
            div {
                style: "{grid_style}",
                // Header row: corner cell + plan headers
                div {
                    class: "bg-surface-alt border-b border-r border-border",
                    style: "position: sticky; top: 0; left: 0; z-index: 3;",
                }
                for plan in props.day_aggregate.plans.iter() {
                    {
                        let plan_name = plan.shiftplan_name.to_string();
                        rsx! {
                            div {
                                class: "bg-surface-alt border-b border-r border-border px-[10px] py-2 select-none",
                                style: "position: sticky; top: 0; z-index: 1;",
                                div { class: "text-[12px] font-bold text-ink truncate",
                                    "{plan_name}"
                                }
                            }
                        }
                    }
                }

                // Body row: time column + plan columns
                div {
                    class: "bg-surface border-r border-border relative",
                    style: "{time_col_style}",
                    for h in hour_start..hour_end {
                        {
                            let label = format!("{:02}:00–{:02}:00", h, h + 1);
                            let top_px = (h as f32 - day_start) * SCALING;
                            let style = format!(
                                "top: {}px; height: {}px; padding-top: 4px;",
                                top_px, SCALING,
                            );
                            rsx! {
                                div {
                                    class: "absolute left-0 right-0 border-t border-border px-2 font-mono text-[11px] text-ink-muted text-right",
                                    style: "{style}",
                                    "{label}"
                                }
                            }
                        }
                    }
                }
                for plan in props.day_aggregate.plans.iter() {
                    {
                        let plan_slots = plan.slots.clone();
                        rsx! {
                            div {
                                class: "relative border-r border-border",
                                style: "{day_col_style}",
                                for h in hour_start..hour_end {
                                    div {
                                        class: "absolute left-0 right-0 border-t border-border pointer-events-none",
                                        style: format!(
                                            "top: {}px; height: 0;",
                                            (h as f32 - day_start) * SCALING,
                                        ),
                                    }
                                }
                                for slot in plan_slots.iter() {
                                    WeekCellSlot {
                                        key: "{slot.id}",
                                        slot: slot.clone(),
                                        day_start,
                                        highlight_item_id: props.highlight_item_id,
                                        add_event: props.add_event,
                                        remove_event: props.remove_event,
                                        item_clicked: props.item_clicked,
                                        discourage: false,
                                        button_types: props.button_types.clone(),
                                        dropdown_entries: props.dropdown_entries.clone(),
                                        is_shiftplanner: props.is_shiftplanner,
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
        let agg = make_day_aggregate(vec![make_slot(Weekday::Monday), make_slot(Weekday::Sunday)]);
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

    let nav_class = "w-7 h-7 inline-flex items-center justify-center border border-border-strong rounded-md font-mono text-ink-soft bg-surface hover:bg-surface-alt print:hidden";
    let day_active_class =
        "px-3 py-1 text-[13px] font-medium rounded-[4px] bg-surface text-ink shadow-sm border border-border-strong";
    let day_inactive_class =
        "px-3 py-1 text-[13px] font-medium rounded-[4px] text-ink-muted hover:text-ink hover:bg-surface-alt border border-transparent";

    rsx! {
        div { class: "flex flex-row items-center justify-center gap-1 mb-4",
            button {
                class: nav_class,
                "aria-label": "Vorheriger Tag",
                onclick: move |_| props.on_prev_day.call(()),
                "‹"
            }
            for day in days.iter() {
                button {
                    class: if *day == props.selected_day { day_active_class } else { day_inactive_class },
                    onclick: {
                        let day = *day;
                        move |_| props.on_select_day.call(day)
                    },
                    {day.i18n_short_string(&i18n)}
                }
            }
            button {
                class: nav_class,
                "aria-label": "Nächster Tag",
                onclick: move |_| props.on_next_day.call(()),
                "›"
            }
        }
    }
}
