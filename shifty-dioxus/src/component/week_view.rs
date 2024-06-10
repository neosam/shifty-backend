use std::rc::Rc;

use crate::{
    i18n, state,
    state::{Slot, Weekday},
};
use dioxus::prelude::*;
use dioxus_elements::p;

#[derive(PartialEq, Clone, Props)]
pub struct ColumnViewItem {
    pub start: f32,
    pub end: f32,
    pub title: Rc<str>,
}

pub fn ColumnViewSlot(props: ColumnViewItem) -> Element {
    rsx! {
        div {
            class: "w-full absolute border-solid border-black border truncate",
            style: {
                format!("top: {}px; height: {}px;", props.start, props.end - props.start)
            },
            { props.title }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ColumnViewProps {
    pub height: f32,
    pub scale: f32,
    pub offset: f32,
    pub slots: Rc<[ColumnViewItem]>,
    pub title: Option<Rc<str>>,
}

impl From<Slot> for ColumnViewItem {
    fn from(slot: Slot) -> Self {
        ColumnViewItem {
            start: slot.from_hour(),
            end: slot.to_hour(),
            title: "".into(),
        }
    }
}

#[component]
pub fn ColumnView(props: ColumnViewProps) -> Element {
    rsx! {
        div {
            class: "relative w-full",
            style: {
                format!("height: {}px;", props.height)
            },
            ColumnViewSlot {
                start: 0.0,
                end: props.offset,
                title: props.title.unwrap_or_else(|| "".into()).clone()
            }
            for slot in props.slots.iter() {
                ColumnViewSlot {
                    start: slot.start * props.scale + props.offset,
                    end: slot.end * props.scale + props.offset,
                    title: slot.title.clone()
                }
            }
        }
    }
}

#[component]
pub fn TimeView() -> Element {
    let slots: Vec<ColumnViewItem> = (0..24)
        .map(|i| ColumnViewItem {
            start: i as f32,
            end: i as f32 + 1.0,
            title: format!("{:02}:00-{:02}:00", i, i + 1).into(),
        })
        .collect();
    let slots: Rc<[ColumnViewItem]> = slots.into();

    rsx! {
        ColumnView {
            height: 2400.0 * 3.0,
            scale: 30.0,
            offset: 30.0,
            slots: slots,
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct DayViewProps {
    pub weekday: Weekday,
    pub slots: Rc<[state::Slot]>,
}

#[component]
pub fn DayView(props: DayViewProps) -> Element {
    let i18n = use_context::<i18n::I18nType>();
    rsx! {
        ColumnView {
            height: 2400.0 * 3.0,
            scale: 30.0,
            offset: 30.0,
            slots: props.slots.iter().map(|slot| ColumnViewItem::from(slot.clone())).collect(),
            title: Some(props.weekday.i18n_string(&i18n)),
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct WeekViewProps {
    pub shiftplan_data: state::Shiftplan,
}

#[component]
pub fn WeekView(props: WeekViewProps) -> Element {
    rsx! {
        div {
            class: "flex flex-row",
            TimeView {}
            DayView { weekday: Weekday::Monday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Monday)}
            DayView { weekday: Weekday::Tuesday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Tuesday)}
            DayView { weekday: Weekday::Wednesday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Wednesday)}
            DayView { weekday: Weekday::Thursday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Thursday)}
            DayView { weekday: Weekday::Friday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Friday)}
            DayView { weekday: Weekday::Saturday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Saturday)}
            DayView { weekday: Weekday::Sunday, slots: props.shiftplan_data.slots_by_weekday(Weekday::Sunday)}
        }
    }
}
