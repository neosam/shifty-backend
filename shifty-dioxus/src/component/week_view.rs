use std::rc::Rc;

use crate::component::atoms::PersonChip;
use crate::i18n::Key;
use crate::service::weekly_summary::WEEKLY_SUMMARY_STORE;
use crate::state::shiftplan::Identifiable;
use crate::{
    base_types::ImStr,
    component::dropdown_base::DropdownTrigger,
    service::i18n::I18N,
    service::tooltip::TooltipAction,
    state::{self, dropdown::DropdownEntry, Slot, Weekday},
};
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use tracing::info;
use uuid::Uuid;

pub const SCALING: f32 = 75.0;

#[derive(PartialEq, Clone)]
pub struct ColumnViewContentItem {
    pub id: Uuid,
    pub title: Rc<str>,
    pub background_color: Rc<str>,
    pub tooltip: Option<Rc<str>>,
}

#[derive(PartialEq, Clone)]
pub enum ColumnViewContent {
    Title(ImStr),
    Items(Rc<[ColumnViewContentItem]>),
}
impl From<String> for ColumnViewContent {
    fn from(title: String) -> Self {
        ColumnViewContent::Title(title.into())
    }
}

#[derive(PartialEq, Clone)]
pub struct ColumnViewItem<CustomData = ()>
where
    CustomData: Identifiable + PartialEq + Clone + 'static,
{
    pub start: f32,
    pub end: f32,
    pub title: ColumnViewContent,
    pub show_add: bool,
    pub show_remove: bool,
    pub custom_data: CustomData,
    pub warning: Option<ImStr>,
    pub dropdown_entries: Option<Rc<[DropdownEntry]>>,
}

#[derive(PartialEq, Clone, Props)]
pub struct ColumnViewSlotProps<CustomData: Identifiable + PartialEq + Clone + 'static = ()>
where
    CustomData: Identifiable + PartialEq + Clone + 'static,
{
    pub highlight_item_id: Option<Uuid>,
    pub item_data: ColumnViewItem<CustomData>,
    pub add_event: Option<EventHandler<CustomData>>,
    pub remove_event: Option<EventHandler<CustomData>>,
    pub double_clicked: Option<EventHandler<()>>,
    pub item_clicked: Option<EventHandler<Uuid>>,

    #[props(!optional)]
    pub warning: Option<ImStr>,

    #[props(default = false)]
    pub discourage: bool,

    #[props(default = false)]
    pub show_dropdown: bool,
}

pub fn ColumnViewSlot<CustomData>(props: ColumnViewSlotProps<CustomData>) -> Element
where
    CustomData: Identifiable + PartialEq + Clone + 'static,
{
    let custom_data_add = props.item_data.custom_data.clone();
    let custom_data_remove = props.item_data.custom_data.clone();
    rsx! {
        div {
            class: format!(
                "w-full select-none absolute border border-border-strong truncate flex text-ellipsis touch-auto {}",
                if props.discourage {
                    "cursor-not-allowed bg-warn-soft print:bg-surface"
                } else {
                    ""
                },
            ),
            ondoubleclick: move |_| {
                if let Some(double_clicked) = &props.double_clicked {
                    double_clicked.call(());
                }
            },
            style: {
                format!(
                    "top: {}px; height: {}px;",
                    props.item_data.start,
                    props.item_data.end - props.item_data.start,
                )
            },
            div {
                class: format!(
                    "text-center flex-grow flex-shrink w-full overflow-auto no-scrollbar {}",
                    if props.item_data.warning.is_some() { "bg-warn-soft" } else { "" },
                ),
                {
                    match props.item_data.title {
                        ColumnViewContent::Title(title) => rsx! {
                            p { "{title}" }
                        },
                        ColumnViewContent::Items(items) => {
                            let mut items: Vec<ColumnViewContentItem> = items
                                .iter()
                                .map(|item| item.clone())
                                .collect();
                            let item_clicked = props.item_clicked.clone();
                            items.sort_by_key(|item| item.title.clone());
                            let tooltip_service = use_coroutine_handle::<TooltipAction>();
                            rsx! {
                                div { class: "flex flex-row overflow-scroll no-scrollbar flex-wrap gap-1 m-1",
                                    for item in items.iter() {
                                        {
                                            let item_id = item.id;
                                            let item_tooltip = item.tooltip.clone();
                                            let mut timeout_task = use_signal(|| None::<Task>);
                                            rsx! {
                                                p {
                                                    class: format!(
                                                        "select-none pl-1 pr-1 rounded-md {}",
                                                        if Some(item_id) == props.highlight_item_id { "font-bold" } else { "" },
                                                    ),
                                                    ondoubleclick: move |_| {
                                                        let id = item_id;
                                                        if let Some(item_clicked) = item_clicked {
                                                            info!("Found event handler and call it");
                                                            item_clicked.call(id);
                                                        }
                                                        info!("Item clicked");
                                                        ()
                                                    },
                                                    onmousedown: {
                                                        let tooltip = item_tooltip.clone();
                                                        move |e: Event<MouseData>| {
                                                            // Cancel any existing timeout
                                                            if let Some(task) = timeout_task.read().as_ref() {
                                                                task.cancel();
                                                            }

                                                            if tooltip.is_some() {
                                                                let coords = e.data().page_coordinates();
                                                                let tooltip_content = tooltip.clone().unwrap();

                                                                // Spawn task to show tooltip after 500ms
                                                                let task = spawn(async move {
                                                                    TimeoutFuture::new(500).await;
                                                                    tooltip_service.send(TooltipAction::ShowTooltip(
                                                                        coords.x,
                                                                        coords.y + 20.0,
                                                                        tooltip_content,
                                                                    ));
                                                                });
                                                                *timeout_task.write() = Some(task);
                                                            }
                                                        }
                                                    },
                                                    onmouseup: move |_| {
                                                        // Cancel timeout if still pending, but keep tooltip if shown
                                                        if let Some(task) = timeout_task.read().as_ref() {
                                                            task.cancel();
                                                        }
                                                        *timeout_task.write() = None;
                                                    },
                                                    onmouseleave: move |_| {
                                                        // Cancel timeout to prevent tooltip from showing
                                                        if let Some(task) = timeout_task.read().as_ref() {
                                                            task.cancel();
                                                        }
                                                        *timeout_task.write() = None;
                                                    },
                                                    ontouchstart: {
                                                        let tooltip = item_tooltip.clone();
                                                        move |_| {
                                                            // Cancel any existing timeout
                                                            if let Some(task) = timeout_task.read().as_ref() {
                                                                task.cancel();
                                                            }

                                                            if tooltip.is_some() {
                                                                let tooltip_content = tooltip.clone().unwrap();

                                                                // Spawn task to show tooltip after 500ms (centered for touch)
                                                                let task = spawn(async move {
                                                                    TimeoutFuture::new(500).await;
                                                                    let window = web_sys::window().unwrap();
                                                                    let width = window.inner_width().unwrap().as_f64().unwrap();
                                                                    let height = window.inner_height().unwrap().as_f64().unwrap();
                                                                    tooltip_service.send(TooltipAction::ShowTooltip(
                                                                        width / 2.0 - 100.0,
                                                                        height / 2.0 - 50.0,
                                                                        tooltip_content,
                                                                    ));
                                                                });
                                                                *timeout_task.write() = Some(task);
                                                            }
                                                        }
                                                    },
                                                    ontouchend: move |_| {
                                                        // Cancel timeout if still pending, but keep tooltip if shown
                                                        if let Some(task) = timeout_task.read().as_ref() {
                                                            task.cancel();
                                                        }
                                                        *timeout_task.write() = None;
                                                    },
                                                    ontouchcancel: move |_| {
                                                        // Cancel timeout to prevent tooltip from showing
                                                        if let Some(task) = timeout_task.read().as_ref() {
                                                            task.cancel();
                                                        }
                                                        *timeout_task.write() = None;
                                                    },
                                                    style: format!("background-color: {}", item.background_color),
                                                    "{item.title.clone()}"
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
            div { class: "flex flex-col flex-grow overflow-scroll no-scrollbar bg-surface",
                if props.item_data.show_add {
                    button {
                        class: "border w-8 print:hidden",
                        onclick: move |_| {
                            if let Some(add_event) = props.add_event {
                                info!("Found event handler and call it");
                                add_event.call(custom_data_add.to_owned());
                            }
                            info!("Add event");
                            ()
                        },
                        "+"
                    }
                }
                if props.item_data.show_remove {
                    button {
                        class: "border w-8 print:hidden",
                        onclick: move |_| {
                            if let Some(remove_event) = props.remove_event {
                                info!("Found event handler and call it");
                                remove_event.call(custom_data_remove.to_owned());
                            }
                            info!("Remove event");
                            ()
                        },
                        "-"
                    }
                }
                if let Some(dropdown_entries) = &props.item_data.dropdown_entries {
                    {
                        rsx! {
                            DropdownTrigger {
                                entries: dropdown_entries.clone(),
                                context: props.item_data.custom_data.id(),
                                button { class: "border w-8 print:hidden", "..." }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ColumnViewProps<CustomData: Identifiable + PartialEq + Clone + 'static = ()>
where
    CustomData: Identifiable + PartialEq + Clone + 'static,
{
    pub height: f32,
    pub scale: f32,
    pub offset: f32,
    pub slots: Rc<[ColumnViewItem<CustomData>]>,
    pub title: Option<ImStr>,
    pub highlight_item_id: Option<Uuid>,
    pub add_event: Option<EventHandler<CustomData>>,
    pub remove_event: Option<EventHandler<CustomData>>,
    pub item_clicked: Option<EventHandler<Uuid>>,
    pub title_double_clicked: Option<EventHandler<()>>,
    #[props(default = false)]
    pub discourage: bool,
    pub button_types: WeekViewButtonTypes,
    pub dropdown_entries: Option<Rc<[DropdownEntry]>>,
    #[props(default = "relative min-w-48 flex-grow print:min-w-0".into())]
    pub custom_class: ImStr,
}

impl From<Slot> for ColumnViewItem<Slot> {
    fn from(slot: Slot) -> Self {
        let mut bookings: Vec<ColumnViewContentItem> = slot
            .bookings
            .iter()
            .map(|booking| ColumnViewContentItem {
                id: booking.sales_person_id,
                title: if booking.self_added {
                    format!("{}*", booking.label).into()
                } else {
                    booking.label.clone()
                },
                background_color: booking.background_color.clone(),
                tooltip: None, // Tooltip generation requires i18n context
            })
            .collect();

        // Add min_resources indicator as the first item
        bookings.insert(
            0,
            ColumnViewContentItem {
                id: uuid::Uuid::nil(), // Use nil UUID for the indicator
                title: format!("{}/{}", slot.bookings.len(), slot.min_resources).into(),
                background_color: if slot.bookings.len() != slot.min_resources as usize {
                    "#ffcccc".into() // Light red background when understaffed
                } else {
                    "#fff".into() // Light green background when adequately staffed
                },
                tooltip: None, // Min resources indicator doesn't need a tooltip
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
}

/// Convert a Slot to a ColumnViewItem with tooltip support for shiftplanners
fn slot_to_column_view_item_with_tooltips(
    slot: Slot,
    is_shiftplanner: bool,
    i18n: &crate::i18n::I18n<crate::i18n::Key, crate::i18n::Locale>,
) -> ColumnViewItem<Slot> {
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

    // Add min_resources indicator as the first item
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

#[component]
pub fn ColumnView<CustomData>(props: ColumnViewProps<CustomData>) -> Element
where
    CustomData: Identifiable + PartialEq + Clone + 'static,
{
    rsx! {
        div {
            class: props.custom_class.as_str(),
            style: format!("height: {}px;", props.height),
            ColumnViewSlot::<()> {
                item_data: ColumnViewItem {
                    start: 0.0,
                    end: props.offset,
                    title: ColumnViewContent::Title(props.title.unwrap_or_else(|| "".into()).clone())
                        .into(),
                    show_add: false,
                    show_remove: false,
                    custom_data: (),
                    warning: None,
                    dropdown_entries: None,
                },
                show_dropdown: true,
                discourage: props.discourage,
                double_clicked: props.title_double_clicked.clone(),
                warning: None,
            }
            for slot in props.slots.iter() {
                ColumnViewSlot::<CustomData> {
                    highlight_item_id: props.highlight_item_id,
                    item_data: ColumnViewItem {
                        start: slot.start * props.scale + props.offset,
                        end: slot.end * props.scale + props.offset,
                        title: slot.title.clone(),
                        show_add: slot.show_add && props.button_types == WeekViewButtonTypes::AddRemove,
                        show_remove: slot.show_remove
                            && props.button_types == WeekViewButtonTypes::AddRemove,
                        custom_data: slot.custom_data.clone(),
                        warning: slot.warning.clone(),
                        dropdown_entries: if props.button_types == WeekViewButtonTypes::Dropdown {
                            slot.dropdown_entries.clone()
                        } else {
                            None
                        },
                    },
                    add_event: props.add_event,
                    remove_event: props.remove_event,
                    item_clicked: props.item_clicked.clone(),
                    discourage: props.discourage,
                    warning: slot.warning.clone(),
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct TimeViewProps {
    pub start: u8,
    pub end: u8,
}

#[component]
pub fn TimeView(props: TimeViewProps) -> Element {
    let slots: Vec<ColumnViewItem> = (props.start..props.end)
        .map(|i| ColumnViewItem {
            start: (i - props.start) as f32,
            end: (i - props.start) as f32 + 1.0,
            title: format!("{:02}:00-{:02}:00", i, i + 1).into(),
            show_add: false,
            show_remove: false,
            custom_data: (),
            warning: None,
            dropdown_entries: None,
        })
        .collect();
    let slots: Rc<[ColumnViewItem]> = slots.into();

    rsx! {
        ColumnView::<()> {
            height: (props.end - props.start) as f32 * SCALING,
            scale: SCALING,
            offset: SCALING / 2.0,
            button_types: WeekViewButtonTypes::None,
            slots,
            custom_class: "relative min-w-28 flex-shrink-0 print:min-w-0".into(),
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct DayViewProps {
    pub weekday: Weekday,
    pub slots: Rc<[state::Slot]>,
    pub day_start: f32,
    pub day_end: f32,
    pub highlight_item_id: Option<Uuid>,
    pub add_event: Option<EventHandler<Slot>>,
    pub remove_event: Option<EventHandler<Slot>>,
    pub item_clicked: Option<EventHandler<Uuid>>,
    pub title_double_clicked: Option<EventHandler<Weekday>>,
    pub date: Option<time::Date>,
    pub button_types: WeekViewButtonTypes,
    pub dropdown_entries: Option<Rc<[DropdownEntry]>>,
    pub header: Option<Rc<str>>,

    pub discourage: bool,

    #[props(default = false)]
    pub is_shiftplanner: bool,
}

#[component]
pub fn DayView(props: DayViewProps) -> Element {
    let i18n = I18N.read().clone();
    let mut title = format!("{}", props.weekday.i18n_string(&i18n));
    if let Some(date) = props.date {
        title.push_str(&format!(", {}", i18n.format_date(&date)));
    }
    if let Some(header) = &props.header {
        title.push_str(&format!(" | {}", header));
    }
    rsx! {
        ColumnView::<Slot> {
            height: (props.day_end - props.day_start) as f32 * SCALING + SCALING / 2.0,
            scale: SCALING,
            offset: SCALING / 2.0,
            slots: props
                .slots
                .iter()
                .map(|slot| slot_to_column_view_item_with_tooltips(slot.clone(), props.is_shiftplanner, &i18n))
                .map(|column| ColumnViewItem {
                    start: column.start - props.day_start,
                    end: column.end - props.day_start,
                    title: column.title,
                    show_add: true,
                    show_remove: true,
                    custom_data: column.custom_data,
                    warning: column.warning,
                    dropdown_entries: props.dropdown_entries.clone(),
                })
                .collect(),
            title: Some(title.into()),
            highlight_item_id: props.highlight_item_id,
            add_event: props.add_event.clone(),
            remove_event: props.remove_event.clone(),
            item_clicked: props.item_clicked.clone(),
            title_double_clicked: move |_| {
                if let Some(title_double_clicked) = &props.title_double_clicked {
                    title_double_clicked.call(props.weekday.clone());
                }
            },
            discourage: props.discourage,
            button_types: props.button_types.clone(),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum WeekViewButtonTypes {
    AddRemove,
    Dropdown,
    None,
}

/// Resolved kind of action button to render in a single week-grid cell.
///
/// The redesigned grid renders at most one absolute-positioned button per cell.
/// Branching depends on the editing person and the cell's bookings, plus the
/// global `WeekViewButtonTypes` mode.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CellButton {
    /// Editing person is not in the cell's bookings — render `+` (Ghost-style).
    Add,
    /// Editing person is in the cell's bookings — render `−` (Danger-style).
    Remove,
    /// Structure mode — render the `…` dropdown trigger.
    Dropdown,
    /// No button at all (read-only or no editing person selected).
    None,
}

/// Pure helper that resolves which (if any) button a single cell renders.
pub fn resolve_cell_button(
    button_types: &WeekViewButtonTypes,
    editing_person_id: Option<Uuid>,
    booking_sales_person_ids: &[Uuid],
) -> CellButton {
    match button_types {
        WeekViewButtonTypes::None => CellButton::None,
        WeekViewButtonTypes::Dropdown => CellButton::Dropdown,
        WeekViewButtonTypes::AddRemove => match editing_person_id {
            None => CellButton::None,
            Some(id) => {
                if booking_sales_person_ids.iter().any(|sp| *sp == id) {
                    CellButton::Remove
                } else {
                    CellButton::Add
                }
            }
        },
    }
}

#[cfg(test)]
mod cell_button_tests {
    use super::*;

    #[test]
    fn cell_button_dropdown_in_dropdown_mode() {
        let result = resolve_cell_button(&WeekViewButtonTypes::Dropdown, None, &[]);
        assert_eq!(result, CellButton::Dropdown);
    }

    #[test]
    fn cell_button_none_in_none_mode() {
        let result = resolve_cell_button(
            &WeekViewButtonTypes::None,
            Some(Uuid::from_u128(1)),
            &[Uuid::from_u128(1)],
        );
        assert_eq!(result, CellButton::None);
    }

    #[test]
    fn cell_button_none_when_no_editing_person() {
        let result = resolve_cell_button(&WeekViewButtonTypes::AddRemove, None, &[]);
        assert_eq!(result, CellButton::None);
    }

    #[test]
    fn cell_button_remove_when_editing_person_in_cell() {
        let editing = Uuid::from_u128(7);
        let bookings = vec![Uuid::from_u128(1), editing, Uuid::from_u128(2)];
        let result = resolve_cell_button(&WeekViewButtonTypes::AddRemove, Some(editing), &bookings);
        assert_eq!(result, CellButton::Remove);
    }

    #[test]
    fn cell_button_add_when_editing_person_not_in_cell() {
        let editing = Uuid::from_u128(7);
        let bookings = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
        let result = resolve_cell_button(&WeekViewButtonTypes::AddRemove, Some(editing), &bookings);
        assert_eq!(result, CellButton::Add);
    }

    #[test]
    fn cell_button_add_when_cell_empty() {
        let result = resolve_cell_button(
            &WeekViewButtonTypes::AddRemove,
            Some(Uuid::from_u128(1)),
            &[],
        );
        assert_eq!(result, CellButton::Add);
    }

    #[test]
    fn cell_background_class_understaffed_is_warn_soft() {
        assert_eq!(cell_background_class(true, false), "bg-warn-soft");
    }

    #[test]
    fn cell_background_class_fully_staffed_is_empty() {
        assert_eq!(cell_background_class(false, false), "");
    }

    #[test]
    fn cell_background_class_discourage_is_bad_soft() {
        assert_eq!(cell_background_class(false, true), "bg-bad-soft");
    }

    #[test]
    fn cell_background_class_discourage_overrides_missing() {
        // Both flags set → discourage wins (bad takes priority over warn)
        assert_eq!(cell_background_class(true, true), "bg-bad-soft");
    }

    #[test]
    fn min_resources_class_understaffed_is_warn() {
        assert_eq!(min_resources_class(true), "text-warn");
    }

    #[test]
    fn min_resources_class_fully_staffed_is_ink_muted() {
        assert_eq!(min_resources_class(false), "text-ink-muted");
    }

    #[test]
    fn cell_button_classes_add_uses_surface_and_strong_border() {
        let c = cell_button_classes(CellButton::Add);
        assert!(c.contains("bg-surface"), "missing bg-surface: {c}");
        assert!(
            c.contains("border-border-strong"),
            "missing border-border-strong: {c}"
        );
        assert!(c.contains("text-ink-soft"), "missing text-ink-soft: {c}");
        assert!(c.contains("absolute"));
        assert!(c.contains("rounded-[3px]"));
    }

    #[test]
    fn cell_button_classes_remove_uses_bad_tokens() {
        let c = cell_button_classes(CellButton::Remove);
        assert!(c.contains("bg-bad-soft"), "missing bg-bad-soft: {c}");
        assert!(c.contains("text-bad"), "missing text-bad: {c}");
        assert!(c.contains("border-bad"), "missing border-bad: {c}");
    }

    #[test]
    fn cell_button_classes_none_is_empty() {
        assert_eq!(cell_button_classes(CellButton::None), "");
    }

    #[test]
    fn cell_button_classes_no_legacy_color_tokens() {
        for variant in [CellButton::Add, CellButton::Remove, CellButton::Dropdown] {
            let c = cell_button_classes(variant);
            for forbidden in ["bg-blue-", "text-blue-", "bg-red-", "text-red-", "bg-gray-"] {
                assert!(
                    !c.contains(forbidden),
                    "legacy class `{}` in cell-button class `{:?}`",
                    forbidden,
                    variant
                );
            }
        }
    }

    #[test]
    fn week_view_no_legacy_classes_in_source() {
        let source = include_str!("week_view.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for forbidden in [
            "bg-gray-",
            "bg-white",
            "text-gray-",
            "text-blue-",
            "text-red-",
            "text-green-",
            "text-orange-",
            "bg-blue-",
            "bg-green-",
            "bg-red-",
            "bg-slate-",
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

#[derive(PartialEq, Clone, Props)]
pub struct WeekViewProps {
    pub shiftplan_data: state::Shiftplan,
    pub highlight_item_id: Option<Uuid>,
    pub add_event: Option<EventHandler<Slot>>,
    pub remove_event: Option<EventHandler<Slot>>,
    pub item_clicked: Option<EventHandler<Uuid>>,
    pub date_of_monday: Option<time::Date>,
    pub title_double_clicked: Option<EventHandler<Weekday>>,
    pub button_types: WeekViewButtonTypes,
    pub dropdown_entries: Option<Rc<[DropdownEntry]>>,

    #[props(default = Rc::new([]))]
    pub discourage_weekdays: Rc<[Weekday]>,

    #[props(default = Vec::new())]
    pub weekday_headers: Vec<(Weekday, Rc<str>)>,

    #[props(default = false)]
    pub is_shiftplanner: bool,
}

enum Zoom {
    Full,
    Half,
    Quarter,
}

/// Returns the day-total label string (e.g. `"5.0h"`) for a weekday using
/// the loaded weekly summary, or an empty string if the summary has not
/// loaded yet. Pure helper so the lookup is testable.
pub(crate) fn day_total_label(weekday: Weekday) -> String {
    let store = WEEKLY_SUMMARY_STORE.read();
    if !store.data_loaded || store.weekly_summary.is_empty() {
        return String::new();
    }
    let row = &store.weekly_summary[0];
    let hours = match weekday {
        Weekday::Monday => row.monday_available_hours,
        Weekday::Tuesday => row.tuesday_available_hours,
        Weekday::Wednesday => row.wednesday_available_hours,
        Weekday::Thursday => row.thursday_available_hours,
        Weekday::Friday => row.friday_available_hours,
        Weekday::Saturday => row.saturday_available_hours,
        Weekday::Sunday => row.sunday_available_hours,
    };
    format!("{:.1}h", hours)
}

/// Renders one cell-internal person chip with the existing 500-ms-delay
/// tooltip wiring. Wraps a [`PersonChip`] atom in a `<div>` that holds the
/// mouse and touch listeners.
#[derive(PartialEq, Clone, Props)]
struct WeekCellChipProps {
    label: ImStr,
    color: ImStr,
    bold: bool,
    tooltip: Option<ImStr>,
    sales_person_id: Uuid,
    item_clicked: Option<EventHandler<Uuid>>,
}

#[component]
fn WeekCellChip(props: WeekCellChipProps) -> Element {
    let tooltip_service = use_coroutine_handle::<TooltipAction>();
    let mut timeout_task = use_signal(|| None::<Task>);
    let id = props.sales_person_id;
    let item_clicked = props.item_clicked;
    let tooltip = props.tooltip.clone();
    let tooltip_for_touch = props.tooltip.clone();
    rsx! {
        div {
            ondoubleclick: move |_| {
                if let Some(handler) = item_clicked {
                    handler.call(id);
                }
            },
            onmousedown: {
                let tooltip = tooltip.clone();
                move |e: Event<MouseData>| {
                    if let Some(task) = timeout_task.read().as_ref() {
                        task.cancel();
                    }
                    if let Some(tooltip_content) = tooltip.clone() {
                        let coords = e.data().page_coordinates();
                        let task = spawn(async move {
                            TimeoutFuture::new(500).await;
                            tooltip_service.send(TooltipAction::ShowTooltip(
                                coords.x,
                                coords.y + 20.0,
                                tooltip_content.as_rc(),
                            ));
                        });
                        *timeout_task.write() = Some(task);
                    }
                }
            },
            onmouseup: move |_| {
                if let Some(task) = timeout_task.read().as_ref() {
                    task.cancel();
                }
                *timeout_task.write() = None;
            },
            onmouseleave: move |_| {
                if let Some(task) = timeout_task.read().as_ref() {
                    task.cancel();
                }
                *timeout_task.write() = None;
            },
            ontouchstart: {
                let tooltip = tooltip_for_touch.clone();
                move |_| {
                    if let Some(task) = timeout_task.read().as_ref() {
                        task.cancel();
                    }
                    if let Some(tooltip_content) = tooltip.clone() {
                        let task = spawn(async move {
                            TimeoutFuture::new(500).await;
                            if let Some(window) = web_sys::window() {
                                if let (Ok(w), Ok(h)) =
                                    (window.inner_width(), window.inner_height())
                                {
                                    if let (Some(width), Some(height)) =
                                        (w.as_f64(), h.as_f64())
                                    {
                                        tooltip_service.send(TooltipAction::ShowTooltip(
                                            width / 2.0 - 100.0,
                                            height / 2.0 - 50.0,
                                            tooltip_content.as_rc(),
                                        ));
                                    }
                                }
                            }
                        });
                        *timeout_task.write() = Some(task);
                    }
                }
            },
            ontouchend: move |_| {
                if let Some(task) = timeout_task.read().as_ref() {
                    task.cancel();
                }
                *timeout_task.write() = None;
            },
            ontouchcancel: move |_| {
                if let Some(task) = timeout_task.read().as_ref() {
                    task.cancel();
                }
                *timeout_task.write() = None;
            },
            PersonChip {
                name: props.label.clone(),
                color: Some(props.color.clone()),
                bold: props.bold,
            }
        }
    }
}

/// Returns the cell-button class string for the given resolved [`CellButton`].
/// Empty string means no button is rendered.
pub(crate) fn cell_button_classes(button: CellButton) -> &'static str {
    match button {
        CellButton::Add => "absolute top-[6px] right-[6px] w-5 h-5 inline-flex items-center justify-center rounded-[3px] border border-border-strong bg-surface text-ink-soft text-sm font-bold leading-none hover:bg-surface-alt",
        CellButton::Remove => "absolute top-[6px] right-[6px] w-5 h-5 inline-flex items-center justify-center rounded-[3px] border border-bad bg-bad-soft text-bad text-sm font-bold leading-none",
        CellButton::Dropdown => "absolute top-[6px] right-[6px] w-5 h-5 inline-flex items-center justify-center rounded-[3px] border border-border-strong bg-surface text-ink-soft text-sm font-bold leading-none hover:bg-surface-alt",
        CellButton::None => "",
    }
}

/// Returns the per-cell state-background class.
///
/// `discourage` (the editing person is unavailable on this day) takes priority
/// over `missing` and tints the cell `bad`. Missing-staff cells without
/// discourage tint `warn`.
pub(crate) fn cell_background_class(missing: bool, discourage: bool) -> &'static str {
    if discourage {
        "bg-bad-soft"
    } else if missing {
        "bg-warn-soft"
    } else {
        ""
    }
}

/// Returns the min-resources `filled/need` text-color class.
pub(crate) fn min_resources_class(missing: bool) -> &'static str {
    if missing {
        "text-warn"
    } else {
        "text-ink-muted"
    }
}

#[derive(PartialEq, Clone, Props)]
struct WeekDayHeaderProps {
    weekday: Weekday,
    date: Option<time::Date>,
    title_double_clicked: Option<EventHandler<Weekday>>,
}

#[component]
fn WeekDayHeader(props: WeekDayHeaderProps) -> Element {
    let i18n = I18N.read().clone();
    let weekday = props.weekday;
    let weekday_label = weekday.i18n_string(&i18n);
    let date_label = props
        .date
        .map(|d| i18n.format_date(&d).to_string())
        .unwrap_or_default();
    rsx! {
        div {
            class: "bg-surface-alt border-b border-border px-[10px] py-2 select-none cursor-pointer",
            style: "position: sticky; top: 0; z-index: 1;",
            ondoubleclick: move |_| {
                if let Some(handler) = &props.title_double_clicked {
                    handler.call(weekday);
                }
            },
            div { class: "text-[12px] font-bold text-ink",
                "{weekday_label}"
                if !date_label.is_empty() {
                    span { class: "font-normal text-ink-soft", ", {date_label}" }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct WeekCellSlotProps {
    pub slot: Slot,
    pub day_start: f32,
    pub highlight_item_id: Option<Uuid>,
    pub add_event: Option<EventHandler<Slot>>,
    pub remove_event: Option<EventHandler<Slot>>,
    pub item_clicked: Option<EventHandler<Uuid>>,
    pub discourage: bool,
    pub button_types: WeekViewButtonTypes,
    pub dropdown_entries: Option<Rc<[DropdownEntry]>>,
    pub is_shiftplanner: bool,
}

#[component]
pub fn WeekCellSlot(props: WeekCellSlotProps) -> Element {
    let i18n = I18N.read().clone();
    let slot = &props.slot;
    let top = (slot.from_hour() - props.day_start) * SCALING;
    let height = (slot.to_hour() - slot.from_hour()) * SCALING;
    let filled = slot.bookings.len();
    let need = slot.min_resources as usize;
    let missing = filled < need;
    let bg_class = cell_background_class(missing, props.discourage);
    let mr_class = min_resources_class(missing);

    let booking_ids: Vec<Uuid> = slot.bookings.iter().map(|b| b.sales_person_id).collect();
    let cell_button =
        resolve_cell_button(&props.button_types, props.highlight_item_id, &booking_ids);
    let btn_class = cell_button_classes(cell_button);

    let filled_str = i18n.t_m_rc(
        Key::ShiftplanFilledOfNeed,
        [
            ("filled", ImStr::from(filled.to_string())),
            ("need", ImStr::from(need.to_string())),
        ]
        .into(),
    );

    let add_label: ImStr = i18n.t(Key::ShiftplanCellAddTitle).as_ref().into();
    let remove_label: ImStr = i18n.t(Key::ShiftplanCellRemoveTitle).as_ref().into();

    let slot_for_add = slot.clone();
    let slot_for_remove = slot.clone();
    let slot_id_str: ImStr = slot.id.to_string().into();

    rsx! {
        div {
            class: format!(
                "absolute left-0 right-0 border-t border-border {} {}",
                bg_class,
                if props.discourage { "cursor-not-allowed" } else { "" },
            ),
            style: format!(
                "top: {}px; height: {}px; padding: 6px 32px 6px 8px;",
                top, height,
            ),
            div { class: "flex flex-wrap items-start gap-1",
                span {
                    class: format!("font-mono text-[10px] font-bold {}", mr_class),
                    style: "line-height: 18px;",
                    "{filled_str}"
                }
                for booking in slot.bookings.iter() {
                    {
                        let label: ImStr = if booking.self_added {
                            format!("{}*", booking.label).into()
                        } else {
                            booking.label.to_string().into()
                        };
                        let color: ImStr = booking.background_color.to_string().into();
                        let bold = Some(booking.sales_person_id) == props.highlight_item_id;
                        let tooltip = if props.is_shiftplanner {
                            match (&booking.created, &booking.created_by) {
                                (Some(created), Some(created_by)) => {
                                    let date_str = i18n.format_date(&created.date());
                                    let time_str = created.time();
                                    Some(ImStr::from(format!(
                                        "{} {} {} {}",
                                        i18n.t(Key::BookingLogCreatedBy),
                                        created_by,
                                        date_str,
                                        time_str,
                                    )))
                                }
                                _ => Some(i18n.t(Key::BookingNoInfo).as_ref().into()),
                            }
                        } else {
                            None
                        };
                        rsx! {
                            WeekCellChip {
                                label,
                                color,
                                bold,
                                tooltip,
                                sales_person_id: booking.sales_person_id,
                                item_clicked: props.item_clicked,
                            }
                        }
                    }
                }
            }
            match cell_button {
                CellButton::Add => rsx! {
                    button {
                        class: btn_class,
                        "aria-label": add_label.as_str(),
                        onclick: move |evt| {
                            evt.stop_propagation();
                            if let Some(handler) = &props.add_event {
                                handler.call(slot_for_add.clone());
                            }
                            info!("Add event");
                        },
                        "+"
                    }
                },
                CellButton::Remove => rsx! {
                    button {
                        class: btn_class,
                        "aria-label": remove_label.as_str(),
                        onclick: move |evt| {
                            evt.stop_propagation();
                            if let Some(handler) = &props.remove_event {
                                handler.call(slot_for_remove.clone());
                            }
                            info!("Remove event");
                        },
                        "−"
                    }
                },
                CellButton::Dropdown => {
                    if let Some(entries) = props.dropdown_entries.clone() {
                        let ctx: Rc<str> = slot_id_str.as_rc();
                        rsx! {
                            div {
                                class: "absolute top-[6px] right-[6px]",
                                DropdownTrigger {
                                    entries,
                                    context: ctx,
                                    button { class: btn_class, "…" }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                },
                CellButton::None => rsx! {},
            }
        }
    }
}

#[component]
pub fn WeekView(props: WeekViewProps) -> Element {
    let day_start = props.shiftplan_data.min_hour();
    let day_end = props.shiftplan_data.max_hour();
    let has_sunday = props
        .shiftplan_data
        .slots
        .iter()
        .any(|slot| slot.day_of_week == Weekday::Sunday);
    let mut zoom = use_signal(|| Zoom::Full);
    let zoom_class = match *zoom.read() {
        Zoom::Full => "scale-down-100",
        Zoom::Half => "scale-down-50",
        Zoom::Quarter => "scale-down-75",
    };
    let visible_days: Vec<Weekday> = if has_sunday {
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
    let n_days = visible_days.len();
    let body_height = (day_end - day_start) * SCALING;
    let grid_template_columns = format!("76px repeat({}, minmax(140px, 1fr))", n_days);
    let grid_min_width = if n_days >= 7 { 1060 } else { 920 };
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
            div { class: "fixed bottom-4 left-4 z-50 border border-border bg-surface p-2 rounded-md shadow-lg 2xl:hidden print:hidden",
                label { class: "text-[12px] text-ink-muted", "Zoom: " }
                select {
                    class: "ml-2 bg-surface text-ink border border-border-strong rounded-md text-[12px]",
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
            div { class: "{zoom_class}",
                div {
                    style: "{grid_style}",
                    // Header row: corner cell + day headers
                    div {
                        class: "bg-surface-alt border-b border-r border-border",
                        style: "position: sticky; top: 0; left: 0; z-index: 3;",
                    }
                    for weekday in visible_days.iter() {
                        WeekDayHeader {
                            weekday: *weekday,
                            date: props.date_of_monday.map(|date| date + time::Duration::days(*weekday as i64)),
                            title_double_clicked: props.title_double_clicked,
                        }
                    }

                    // Body row: time column (sticky) + day columns
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
                    for weekday in visible_days.iter() {
                        {
                            let day_slots = props.shiftplan_data.slots_by_weekday(*weekday);
                            let discourage = props.discourage_weekdays.contains(weekday);
                            let day = *weekday;
                            rsx! {
                                div {
                                    class: format!(
                                        "relative border-r border-border {}",
                                        if discourage { "bg-bad-soft" } else { "" },
                                    ),
                                    style: "{day_col_style}",
                                    // Hour grid lines (visual separators aligned with time column)
                                    for h in hour_start..hour_end {
                                        div {
                                            class: "absolute left-0 right-0 border-t border-border pointer-events-none",
                                            style: format!(
                                                "top: {}px; height: 0;",
                                                (h as f32 - day_start) * SCALING,
                                            ),
                                        }
                                    }
                                    for slot in day_slots.iter() {
                                        WeekCellSlot {
                                            key: "{slot.id}-{day:?}",
                                            slot: slot.clone(),
                                            day_start,
                                            highlight_item_id: props.highlight_item_id,
                                            add_event: props.add_event,
                                            remove_event: props.remove_event,
                                            item_clicked: props.item_clicked,
                                            discourage,
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
}
