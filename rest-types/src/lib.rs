use std::sync::Arc;

use serde::{Deserialize, Serialize};
use service::{booking::Booking, sales_person::SalesPerson};
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTO {
    pub name: String,
}
impl From<&service::User> for UserTO {
    fn from(user: &service::User) -> Self {
        Self {
            name: user.name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleTO {
    pub name: String,
}
impl From<&service::Role> for RoleTO {
    fn from(role: &service::Role) -> Self {
        Self {
            name: role.name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivilegeTO {
    pub name: String,
}
impl From<&service::Privilege> for PrivilegeTO {
    fn from(privilege: &service::Privilege) -> Self {
        Self {
            name: privilege.name.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRole {
    pub user: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RolePrivilege {
    pub role: String,
    pub privilege: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BookingTO {
    #[serde(default)]
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub slot_id: Uuid,
    pub calendar_week: i32,
    pub year: u32,
    #[serde(default)]
    pub created: Option<PrimitiveDateTime>,
    #[serde(default)]
    pub deleted: Option<PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
impl From<&Booking> for BookingTO {
    fn from(booking: &Booking) -> Self {
        Self {
            id: booking.id,
            sales_person_id: booking.sales_person_id,
            slot_id: booking.slot_id,
            calendar_week: booking.calendar_week,
            year: booking.year,
            created: booking.created,
            deleted: booking.deleted,
            version: booking.version,
        }
    }
}
impl From<&BookingTO> for Booking {
    fn from(booking: &BookingTO) -> Self {
        Self {
            id: booking.id,
            sales_person_id: booking.sales_person_id,
            slot_id: booking.slot_id,
            calendar_week: booking.calendar_week,
            year: booking.year,
            created: booking.created,
            deleted: booking.deleted,
            version: booking.version,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SalesPersonTO {
    #[serde(default)]
    pub id: Uuid,
    pub name: Arc<str>,
    pub background_color: Arc<str>,
    #[serde(default)]
    pub inactive: bool,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
impl From<&SalesPerson> for SalesPersonTO {
    fn from(sales_person: &SalesPerson) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.clone(),
            background_color: sales_person.background_color.clone(),
            inactive: sales_person.inactive,
            deleted: sales_person.deleted,
            version: sales_person.version,
        }
    }
}
impl From<&SalesPersonTO> for SalesPerson {
    fn from(sales_person: &SalesPersonTO) -> Self {
        Self {
            id: sales_person.id,
            name: sales_person.name.clone(),
            background_color: sales_person.background_color.clone(),
            inactive: sales_person.inactive,
            deleted: sales_person.deleted,
            version: sales_person.version,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum DayOfWeekTO {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl From<service::slot::DayOfWeek> for DayOfWeekTO {
    fn from(day_of_week: service::slot::DayOfWeek) -> Self {
        match day_of_week {
            service::slot::DayOfWeek::Monday => Self::Monday,
            service::slot::DayOfWeek::Tuesday => Self::Tuesday,
            service::slot::DayOfWeek::Wednesday => Self::Wednesday,
            service::slot::DayOfWeek::Thursday => Self::Thursday,
            service::slot::DayOfWeek::Friday => Self::Friday,
            service::slot::DayOfWeek::Saturday => Self::Saturday,
            service::slot::DayOfWeek::Sunday => Self::Sunday,
        }
    }
}
impl From<DayOfWeekTO> for service::slot::DayOfWeek {
    fn from(day_of_week: DayOfWeekTO) -> Self {
        match day_of_week {
            DayOfWeekTO::Monday => Self::Monday,
            DayOfWeekTO::Tuesday => Self::Tuesday,
            DayOfWeekTO::Wednesday => Self::Wednesday,
            DayOfWeekTO::Thursday => Self::Thursday,
            DayOfWeekTO::Friday => Self::Friday,
            DayOfWeekTO::Saturday => Self::Saturday,
            DayOfWeekTO::Sunday => Self::Sunday,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotTO {
    #[serde(default)]
    pub id: Uuid,
    pub day_of_week: DayOfWeekTO,
    pub from: time::Time,
    pub to: time::Time,
    pub valid_from: time::Date,
    pub valid_to: Option<time::Date>,
    #[serde(default)]
    pub deleted: Option<time::PrimitiveDateTime>,
    #[serde(rename = "$version")]
    #[serde(default)]
    pub version: Uuid,
}
impl From<&service::slot::Slot> for SlotTO {
    fn from(slot: &service::slot::Slot) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
        }
    }
}
impl From<&SlotTO> for service::slot::Slot {
    fn from(slot: &SlotTO) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
        }
    }
}
