use std::rc::Rc;

use crate::state::shiftplan::Identifiable;
use crate::{
    base_types::ImStr,
    component::dropdown_base::DropdownTrigger,
    service::i18n::I18N,
    state::{self, dropdown::DropdownEntry, Slot, Weekday},
};
use dioxus::prelude::*;
use tracing::info;
use uuid::Uuid;

const SCALING: f32 = 75.0;

#[derive(PartialEq, Clone)]
pub struct ColumnViewContentItem {
    pub id: Uuid,
    pub title: Rc<str>,
    pub background_color: Rc<str>,
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
                "w-full select-none absolute border-solid border-black border truncate flex text-ellipsis touch-auto {}",
                if props.discourage {
                    "cursor-not-allowed bg-blockedColor print:bg-white"
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
                    if props.item_data.warning.is_some() { "bg-missingColor" } else { "" },
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
                            rsx! {
                                div { class: "flex flex-row overflow-scroll no-scrollbar flex-wrap gap-1 m-1",
                                    for item in items.iter() {
                                        {
                                            let item_id = item.id;
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
            div { class: "flex flex-col flex-grow overflow-scroll no-scrollbar bg-white",
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
}

impl From<Slot> for ColumnViewItem<Slot> {
    fn from(slot: Slot) -> Self {
        ColumnViewItem {
            start: slot.from_hour(),
            end: slot.to_hour(),
            show_add: true,
            show_remove: true,
            title: ColumnViewContent::Items(
                slot.bookings
                    .iter()
                    .map(|booking| ColumnViewContentItem {
                        id: booking.sales_person_id,
                        title: booking.label.clone(),
                        background_color: booking.background_color.clone(),
                    })
                    .collect::<Rc<[ColumnViewContentItem]>>(),
            ),
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

#[component]
pub fn ColumnView<CustomData>(props: ColumnViewProps<CustomData>) -> Element
where
    CustomData: Identifiable + PartialEq + Clone + 'static,
{
    rsx! {
        div {
            class: "relative min-w-48 flex-grow print:min-w-0",
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
                .map(|slot| ColumnViewItem::from(slot.clone()))
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
}

enum Zoom {
    Full,
    Half,
    Quarter,
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
    rsx! {
        div {
            class: "overflow-y-scroll overflow-visible no-scrollbar print:width-full print:overflow-visible",
            style: format!("height: {}px", (day_end - day_start) as f32 * SCALING + SCALING),
            div { class: "fixed bottom-4 left-4 z-10 border bg-white p-2 rounded-md shadow-md 2xl:hidden print:hidden",
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
                TimeView { start: day_start.ceil() as u8, end: day_end.ceil() as u8 }
                for weekday in [
                    Weekday::Monday,
                    Weekday::Tuesday,
                    Weekday::Wednesday,
                    Weekday::Thursday,
                    Weekday::Friday,
                    Weekday::Saturday,
                    Weekday::Sunday,
                ]
                    .iter()
                {
                    if !(*weekday == Weekday::Sunday && !has_sunday) {
                        DayView {
                            weekday: weekday.clone(),
                            date: props.date_of_monday.map(|date| date + time::Duration::days(weekday.clone() as i64)),
                            slots: props.shiftplan_data.slots_by_weekday(weekday.clone()),
                            day_start,
                            day_end,
                            highlight_item_id: props.highlight_item_id,
                            add_event: props.add_event.clone(),
                            remove_event: props.remove_event.clone(),
                            item_clicked: props.item_clicked.clone(),
                            title_double_clicked: props.title_double_clicked.clone(),
                            discourage: props.discourage_weekdays.contains(weekday),
                            button_types: props.button_types.clone(),
                            dropdown_entries: props.dropdown_entries.clone(),
                            header: props
                                .weekday_headers
                                .iter()
                                .find(|(day, _)| day == weekday)
                                .map(|(_, text)| text.clone()),
                        }
                    }
                }
            }
        }
    }
}
