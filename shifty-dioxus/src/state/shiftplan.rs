use std::rc::Rc;

use crate::{
    base_types::ImStr,
    i18n::{I18n, Key, Locale},
};
use rest_types::{BookingConflictTO, BookingTO, DayOfWeekTO, SalesPersonTO, SlotTO};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

    pub fn num_from_monday(&self) -> u8 {
        match self {
            Weekday::Monday => 0,
            Weekday::Tuesday => 1,
            Weekday::Wednesday => 2,
            Weekday::Thursday => 3,
            Weekday::Friday => 4,
            Weekday::Saturday => 5,
            Weekday::Sunday => 6,
        }
    }

    pub fn from_num_from_monday(num: u8) -> Self {
        match num {
            0 => Weekday::Monday,
            1 => Weekday::Tuesday,
            2 => Weekday::Wednesday,
            3 => Weekday::Thursday,
            4 => Weekday::Friday,
            5 => Weekday::Saturday,
            6 => Weekday::Sunday,
            _ => panic!("Invalid weekday number: {}", num),
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
impl From<&Weekday> for DayOfWeekTO {
    fn from(weekday: &Weekday) -> Self {
        match weekday {
            Weekday::Monday => DayOfWeekTO::Monday,
            Weekday::Tuesday => DayOfWeekTO::Tuesday,
            Weekday::Wednesday => DayOfWeekTO::Wednesday,
            Weekday::Thursday => DayOfWeekTO::Thursday,
            Weekday::Friday => DayOfWeekTO::Friday,
            Weekday::Saturday => DayOfWeekTO::Saturday,
            Weekday::Sunday => DayOfWeekTO::Sunday,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Booking {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub slot_id: Uuid,
    pub week: u8,
    pub year: u32,
    pub label: Rc<str>,
    pub background_color: Rc<str>,
    pub self_added: bool,
    pub created: Option<time::PrimitiveDateTime>,
    pub created_by: Option<Rc<str>>,
}
impl From<&BookingTO> for Booking {
    fn from(booking: &BookingTO) -> Self {
        Self {
            id: booking.id,
            sales_person_id: booking.sales_person_id,
            slot_id: booking.slot_id,
            week: booking.calendar_week as u8,
            year: booking.year,
            label: "value".into(),
            background_color: "#FFF".into(),
            self_added: false,
            created: None,
            created_by: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SalesPerson {
    pub id: Uuid,
    pub name: Rc<str>,
    pub background_color: Rc<str>,
    pub is_paid: bool,
    pub inactive: bool,
    pub version: Uuid,
}
impl From<&SalesPersonTO> for SalesPerson {
    fn from(sales_person: &SalesPersonTO) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.as_ref().into(),
            background_color: sales_person.background_color.as_ref().into(),
            is_paid: sales_person.is_paid.unwrap_or(false),
            inactive: sales_person.inactive,
            version: sales_person.version,
        }
    }
}
impl From<&SalesPerson> for SalesPersonTO {
    fn from(sales_person: &SalesPerson) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.to_string().into(),
            background_color: sales_person.background_color.to_string().into(),
            is_paid: Some(sales_person.is_paid),
            inactive: sales_person.inactive,
            deleted: None,
            version: sales_person.version,
        }
    }
}

pub trait Identifiable {
    fn id(&self) -> Rc<str>;
}
impl Identifiable for () {
    fn id(&self) -> Rc<str> {
        "".into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub id: Uuid,
    pub day_of_week: Weekday,
    pub from: time::Time,
    pub to: time::Time,
    pub bookings: Rc<[Booking]>,
    pub min_resources: u8,
}
impl Identifiable for Slot {
    fn id(&self) -> Rc<str> {
        self.id.to_string().into()
    }
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
            bookings: [].into(),
            min_resources: slot.min_resources,
        }
    }
}

impl Slot {
    pub fn evaluation(&self) -> SlotEvaluation {
        SlotEvaluation {
            too_less_resources: self.bookings.len() < self.min_resources as usize,
        }
    }
}

pub struct SlotEvaluation {
    pub too_less_resources: bool,
}

impl SlotEvaluation {
    pub fn is_faulty(&self) -> bool {
        self.too_less_resources
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shiftplan {
    pub week: u8,
    pub year: u32,
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

    pub fn min_hour(&self) -> f32 {
        self.slots
            .iter()
            .map(|slot| slot.from_hour())
            .fold(f32::INFINITY, f32::min)
    }

    pub fn max_hour(&self) -> f32 {
        self.slots
            .iter()
            .map(|slot| slot.to_hour())
            .fold(f32::NEG_INFINITY, f32::max)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookingConflict {
    pub booking_id: Uuid,
    pub slot_id: Uuid,
    pub sales_person_id: Uuid,
    pub sales_person_name: ImStr,
    pub day_of_week: Weekday,
}

impl From<&BookingConflictTO> for BookingConflict {
    fn from(to: &BookingConflictTO) -> BookingConflict {
        BookingConflict {
            booking_id: to.booking.id,
            slot_id: to.slot.id,
            sales_person_id: to.sales_person.id,
            sales_person_name: to.sales_person.name.to_string().into(),
            day_of_week: to.slot.day_of_week.into(),
        }
    }
}
