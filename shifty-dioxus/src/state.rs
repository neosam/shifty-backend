use std::rc::Rc;

use crate::{
    i18n::{self, I18n, Key, Locale},
    AuthInfo,
};
use rest_types::{DayOfWeekTO, SlotTO};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub backend: Rc<str>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub config: Config,
    pub i18n: Rc<i18n::I18n<i18n::Key, i18n::Locale>>,
    pub auth_info: AuthInfo,
}

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
impl From<DayOfWeekTO> for Weekday {
    fn from(day_of_week: DayOfWeekTO) -> Self {
        match day_of_week {
            rest_types::DayOfWeekTO::Monday => Weekday::Monday,
            rest_types::DayOfWeekTO::Tuesday => Weekday::Tuesday,
            rest_types::DayOfWeekTO::Wednesday => Weekday::Wednesday,
            rest_types::DayOfWeekTO::Thursday => Weekday::Thursday,
            rest_types::DayOfWeekTO::Friday => Weekday::Friday,
            rest_types::DayOfWeekTO::Saturday => Weekday::Saturday,
            rest_types::DayOfWeekTO::Sunday => Weekday::Sunday,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: Uuid,
    pub day_of_week: Weekday,
    pub from: time::Time,
    pub to: time::Time,
}
impl Slot {
    pub fn from_hour(&self) -> f32 {
        self.from.hour() as f32 + self.from.minute() as f32 / 60.0
    }

    pub fn to_hour(&self) -> f32 {
        self.to.hour() as f32 + self.to.minute() as f32 / 60.0
    }
}
impl From<&SlotTO> for Slot {
    fn from(slot: &SlotTO) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shiftplan {
    pub week: u8,
    pub year: u16,
    pub slots: Rc<[Slot]>,
}

impl Shiftplan {
    pub fn slots_by_weekday(&self, weekday: Weekday) -> Rc<[Slot]> {
        self.slots
            .iter()
            .filter(|slot| slot.day_of_week == weekday)
            .cloned()
            .collect()
    }
}
