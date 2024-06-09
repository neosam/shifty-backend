use std::rc::Rc;

use crate::state::{Slot, State, Weekday};
use dioxus::prelude::*;

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
    pub slots: Rc<[ColumnViewItem]>,
}

#[component]
pub fn DayView(props: DayViewProps) -> Element {
    let state = use_context::<Signal<State>>();
    let i18n = state.read().i18n.clone();
    rsx! {
        ColumnView {
            height: 2400.0 * 3.0,
            scale: 30.0,
            offset: 30.0,
            slots: props.slots,
            title: Some(props.weekday.i18n_string(&i18n)),
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct WeekViewProps {
    pub slots: Rc<[Slot]>,
}

#[component]
pub fn WeekView() -> Element {
    rsx! {
        div {
            class: "flex flex-row",
            TimeView {}
            DayView { weekday: Weekday::Monday, slots: vec![
                ColumnViewItem {
                    start: 0.0,
                    end: 1.0,
                    title: "Test".into(),
                }
            ].into()}
            DayView { weekday: Weekday::Tuesday, slots: vec![].into()}
            DayView { weekday: Weekday::Wednesday, slots: vec![].into()}
            DayView { weekday: Weekday::Thursday, slots: vec![].into()}
            DayView { weekday: Weekday::Friday, slots: vec![].into()}
            DayView { weekday: Weekday::Saturday, slots: vec![].into()}
            DayView { weekday: Weekday::Sunday, slots: vec![].into()}
        }
    }
}
