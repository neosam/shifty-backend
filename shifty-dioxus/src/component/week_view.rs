use std::rc::Rc;

use crate::{
    i18n, state,
    state::{Slot, Weekday},
};
use dioxus::prelude::*;
use tracing::info;
use uuid::Uuid;

#[derive(PartialEq, Clone)]
pub struct ColumnViewContentItem {
    pub id: Uuid,
    pub title: Rc<str>,
    pub background_color: Rc<str>,
}

#[derive(PartialEq, Clone)]
pub enum ColumnViewContent {
    Title(Rc<str>),
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
    CustomData: PartialEq + Clone + 'static,
{
    pub start: f32,
    pub end: f32,
    pub title: ColumnViewContent,
    pub show_add: bool,
    pub show_remove: bool,
    pub custom_data: CustomData,
}

#[derive(PartialEq, Clone, Props)]
pub struct ColumnViewSlotProps<CustomData = ()>
where
    CustomData: PartialEq + Clone + 'static,
{
    pub item_data: ColumnViewItem<CustomData>,
    pub add_event: Option<EventHandler<CustomData>>,
    pub remove_event: Option<EventHandler<CustomData>>,
    pub item_clicked: Option<EventHandler<Uuid>>,
}

pub fn ColumnViewSlot<CustomData>(props: ColumnViewSlotProps<CustomData>) -> Element
where
    CustomData: PartialEq + Clone + 'static,
{
    let custom_data_add = props.item_data.custom_data.clone();
    let custom_data_remove = props.item_data.custom_data.clone();
    rsx! {
        div {
            class: "w-full absolute border-solid border-black border truncate flex",
            style: {
                format!("top: {}px; height: {}px;", props.item_data.start, props.item_data.end - props.item_data.start)
            },
            div {
                class: "text-center truncate flex-grow flex-shrink w-full",
                {
                    match props.item_data.title {
                        ColumnViewContent::Title(title) => rsx! { p { "{title}" } },
                        ColumnViewContent::Items(items) => {
                            let mut items: Vec<ColumnViewContentItem> = items.iter().map(|item| item.clone()).collect();
                            let item_clicked = props.item_clicked.clone();
                            items.sort_by_key(|item| item.title.clone());
                            rsx! { div {
                                class: "flex flex-row overflow-scroll flex-wrap gap-1 m-1",
                                for item in items.iter() {
                                    {
                                        let item_id = item.id;
                                        rsx! { p {
                                            class: "pl-1 pr-1 rounded-md",
                                            onclick: move |_| {
                                                let id = item_id;
                                                if let Some(item_clicked) = item_clicked {
                                                    info!("Found event handler and call it");
                                                    item_clicked.call(id);
                                                };
                                                info!("Item clicked");
                                                ()
                                            },
                                            style: format!("background-color: {}", item.background_color),
                                            "{item.title.clone()}"
                                        } }
                                    }
                                }
                            } }
                        }
                    }
                }
            }
            div {
                class: "flex flex-col flex-grow overflow-scroll",
                if props.item_data.show_add {
                    button {
                        class: "border w-8",
                        onclick: move |_| {
                            if let Some(add_event) = props.add_event {
                                info!("Found event handler and call it");
                                add_event.call(custom_data_add.to_owned());
                            };
                            info!("Add event");
                            ()
                        },
                        "+"
                    }
                }
                if props.item_data.show_remove {
                    button {
                        class: "border w-8",
                        onclick: move |_| {
                            if let Some(remove_event) = props.remove_event {
                                info!("Found event handler and call it");
                                remove_event.call(custom_data_remove.to_owned());
                            };
                            info!("Remove event");
                            ()
                        },
                        "-"
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ColumnViewProps<CustomData = ()>
where
    CustomData: PartialEq + Clone + 'static,
{
    pub height: f32,
    pub scale: f32,
    pub offset: f32,
    pub slots: Rc<[ColumnViewItem<CustomData>]>,
    pub title: Option<Rc<str>>,
    pub add_event: Option<EventHandler<CustomData>>,
    pub remove_event: Option<EventHandler<CustomData>>,
    pub item_clicked: Option<EventHandler<Uuid>>,
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
            custom_data: slot,
        }
    }
}

#[component]
pub fn ColumnView<CustomData>(props: ColumnViewProps<CustomData>) -> Element
where
    CustomData: PartialEq + Clone + 'static,
{
    rsx! {
        div {
            class: "relative min-w-44 flex-grow",
            style: {
                format!("height: {}px;", props.height)
            },
            ColumnViewSlot::<()> {
                item_data: ColumnViewItem {
                    start: 0.0,
                    end: props.offset,
                    title: ColumnViewContent::Title(props.title.unwrap_or_else(|| "".into()).clone()),
                    show_add: false,
                    show_remove: false,
                    custom_data: (),
                },
            }
            for slot in props.slots.iter() {
                ColumnViewSlot::<CustomData> {
                    item_data: ColumnViewItem {
                        start: slot.start * props.scale + props.offset,
                        end: slot.end * props.scale + props.offset,
                        title: slot.title.clone(),
                        show_add: slot.show_add,
                        show_remove: slot.show_remove,
                        custom_data: slot.custom_data.clone(),
                    },
                    add_event: props.add_event,
                    remove_event: props.remove_event,
                    item_clicked: props.item_clicked.clone(),
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
        })
        .collect();
    let slots: Rc<[ColumnViewItem]> = slots.into();

    rsx! {
        ColumnView::<()> {
            height: (props.end - props.start) as f32 * 60.0,
            scale: 60.0,
            offset: 30.0,
            slots: slots,
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct DayViewProps {
    pub weekday: Weekday,
    pub slots: Rc<[state::Slot]>,
    pub day_start: f32,
    pub day_end: f32,
    pub add_event: Option<EventHandler<Slot>>,
    pub remove_event: Option<EventHandler<Slot>>,
    pub item_clicked: Option<EventHandler<Uuid>>,
}

#[component]
pub fn DayView(props: DayViewProps) -> Element {
    let i18n = use_context::<i18n::I18nType>();
    rsx! {
        ColumnView::<Slot> {
            height: (props.day_end - props.day_start) as f32 * 60.0 + 30.0,
            scale: 60.0,
            offset: 30.0,
            slots: props.slots.iter()
                .map(|slot| ColumnViewItem::from(slot.clone()))
                .map(|column| ColumnViewItem {
                    start: column.start - props.day_start,
                    end: column.end - props.day_start,
                    title: column.title,
                    show_add: true,
                    show_remove: true,
                    custom_data: column.custom_data,
                })
                .collect(),
            title: Some(props.weekday.i18n_string(&i18n)),
            add_event: props.add_event.clone(),
            remove_event: props.remove_event.clone(),
            item_clicked: props.item_clicked.clone(),
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct WeekViewProps {
    pub shiftplan_data: state::Shiftplan,
    pub add_event: Option<EventHandler<Slot>>,
    pub remove_event: Option<EventHandler<Slot>>,
    pub item_clicked: Option<EventHandler<Uuid>>,
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
    rsx! {
        div {
            class: "overflow-y-scroll overflow-visible",
            style: format!("height: {}px", (day_end - day_start) as f32 * 60.0 + 60.0),
            div {
                class: "flex flex-row",
                TimeView {start: day_start.ceil() as u8, end: day_end.ceil() as u8}
                for weekday in [Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday, Weekday::Thursday, Weekday::Friday, Weekday::Saturday, Weekday::Sunday].iter() {
                    if !(*weekday == Weekday::Sunday && !has_sunday) {
                        DayView {
                            weekday: weekday.clone(),
                            slots: props.shiftplan_data.slots_by_weekday(weekday.clone()),
                            day_start, day_end,
                            add_event: props.add_event.clone(),
                            remove_event: props.remove_event.clone(),
                            item_clicked: props.item_clicked.clone(),
                        }
                    }
                }
                /*DayView {
                    weekday: Weekday::Monday,
                    slots: props.shiftplan_data.slots_by_weekday(Weekday::Monday),
                    day_start, day_end,
                    add_event: props.add_event,
                    remove_event: props.remove_event,
                    item_clicked: props.item_clicked.clone(),
                }
                DayView {
                    weekday: Weekday::Tuesday,
                    slots: props.shiftplan_data.slots_by_weekday(Weekday::Tuesday),
                    day_start, day_end,
                    add_event: props.add_event,
                    remove_event: props.remove_event
                }
                DayView { weekday: Weekday::Wednesday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Wednesday), day_start, day_end, add_event: props.add_event, remove_event: props.remove_event}
                DayView { weekday: Weekday::Thursday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Thursday), day_start, day_end, add_event: props.add_event, remove_event: props.remove_event}
                DayView { weekday: Weekday::Friday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Friday), day_start, day_end, add_event: props.add_event, remove_event: props.remove_event}
                DayView { weekday: Weekday::Saturday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Saturday), day_start, day_end, add_event: props.add_event, remove_event: props.remove_event}
                if has_sunday {
                    DayView { weekday: Weekday::Sunday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Sunday), day_start, day_end, add_event: props.add_event, remove_event: props.remove_event}
                } */
            }
        }
    }
}
