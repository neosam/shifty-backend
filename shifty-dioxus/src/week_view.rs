use std::{rc::Rc, sync::Arc};

use crate::{
    i18n::{I18n, Key, Locale},
    state::State,
};
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl Weekday {
    pub fn i18n_string(&self, i18n: &I18n<Key, Locale>) -> Rc<str> {
        match self {
            Weekday::Monday => i18n.t(Key::Monday),
            Weekday::Tuesday => i18n.t(Key::Tuesday),
            Weekday::Wednesday => i18n.t(Key::Wednesday),
            Weekday::Thursday => i18n.t(Key::Thursday),
            Weekday::Friday => i18n.t(Key::Friday),
            Weekday::Saturday => i18n.t(Key::Saturday),
            Weekday::Sunday => i18n.t(Key::Sunday),
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct Slot {
    pub start: f32,
    pub end: f32,
    pub title: Rc<str>,
}

pub fn ColumnViewSlot(props: Slot) -> Element {
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
    pub slots: Rc<[Slot]>,
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
    let slots: Vec<Slot> = (0..24)
        .map(|i| Slot {
            start: i as f32,
            end: i as f32 + 1.0,
            title: format!("{:02}:00-{:02}:00", i, i + 1).into(),
        })
        .collect();
    let slots: Rc<[Slot]> = slots.into();

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
    pub slots: Rc<[Slot]>,
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

#[component]
pub fn WeekView() -> Element {
    rsx! {
        div {
            class: "flex flex-row",
            TimeView {}
            DayView { weekday: Weekday::Monday, slots: vec![
                Slot {
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
