use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use std::fmt::Debug;
use std::sync::Arc;
use time::Weekday;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl DayOfWeek {
    pub fn to_number(&self) -> u8 {
        match self {
            DayOfWeek::Monday => 1,
            DayOfWeek::Tuesday => 2,
            DayOfWeek::Wednesday => 3,
            DayOfWeek::Thursday => 4,
            DayOfWeek::Friday => 5,
            DayOfWeek::Saturday => 6,
            DayOfWeek::Sunday => 7,
        }
    }
}

impl From<dao::slot::DayOfWeek> for DayOfWeek {
    fn from(day_of_week: dao::slot::DayOfWeek) -> Self {
        match day_of_week {
            dao::slot::DayOfWeek::Monday => Self::Monday,
            dao::slot::DayOfWeek::Tuesday => Self::Tuesday,
            dao::slot::DayOfWeek::Wednesday => Self::Wednesday,
            dao::slot::DayOfWeek::Thursday => Self::Thursday,
            dao::slot::DayOfWeek::Friday => Self::Friday,
            dao::slot::DayOfWeek::Saturday => Self::Saturday,
            dao::slot::DayOfWeek::Sunday => Self::Sunday,
        }
    }
}
impl From<DayOfWeek> for dao::slot::DayOfWeek {
    fn from(day_of_week: DayOfWeek) -> Self {
        match day_of_week {
            DayOfWeek::Monday => Self::Monday,
            DayOfWeek::Tuesday => Self::Tuesday,
            DayOfWeek::Wednesday => Self::Wednesday,
            DayOfWeek::Thursday => Self::Thursday,
            DayOfWeek::Friday => Self::Friday,
            DayOfWeek::Saturday => Self::Saturday,
            DayOfWeek::Sunday => Self::Sunday,
        }
    }
}
impl From<Weekday> for DayOfWeek {
    fn from(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Monday => Self::Monday,
            Weekday::Tuesday => Self::Tuesday,
            Weekday::Wednesday => Self::Wednesday,
            Weekday::Thursday => Self::Thursday,
            Weekday::Friday => Self::Friday,
            Weekday::Saturday => Self::Saturday,
            Weekday::Sunday => Self::Sunday,
        }
    }
}
impl From<DayOfWeek> for Weekday {
    fn from(day_of_week: DayOfWeek) -> Self {
        match day_of_week {
            DayOfWeek::Monday => Self::Monday,
            DayOfWeek::Tuesday => Self::Tuesday,
            DayOfWeek::Wednesday => Self::Wednesday,
            DayOfWeek::Thursday => Self::Thursday,
            DayOfWeek::Friday => Self::Friday,
            DayOfWeek::Saturday => Self::Saturday,
            DayOfWeek::Sunday => Self::Sunday,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    pub id: Uuid,
    pub day_of_week: DayOfWeek,
    pub from: time::Time,
    pub to: time::Time,
    pub min_resources: u8,
    pub valid_from: time::Date,
    pub valid_to: Option<time::Date>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}
impl From<&dao::slot::SlotEntity> for Slot {
    fn from(slot: &dao::slot::SlotEntity) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
            min_resources: slot.min_resources,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
        }
    }
}
impl From<&Slot> for dao::slot::SlotEntity {
    fn from(slot: &Slot) -> Self {
        Self {
            id: slot.id,
            day_of_week: slot.day_of_week.into(),
            from: slot.from,
            to: slot.to,
            min_resources: slot.min_resources,
            valid_from: slot.valid_from,
            valid_to: slot.valid_to,
            deleted: slot.deleted,
            version: slot.version,
        }
    }
}
impl PartialOrd for Slot {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.day_of_week < other.day_of_week {
            return Some(std::cmp::Ordering::Less);
        }
        if self.day_of_week > other.day_of_week {
            return Some(std::cmp::Ordering::Greater);
        }
        if self.from < other.from {
            return Some(std::cmp::Ordering::Less);
        }
        if self.to > other.to {
            return Some(std::cmp::Ordering::Greater);
        }
        return Some(std::cmp::Ordering::Equal);
    }
}
impl Ord for Slot {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait SlotService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_slots(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Slot]>, ServiceError>;
    async fn get_slot(
        &self,
        id: &Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Slot, ServiceError>;
    async fn get_slots_for_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Slot]>, ServiceError>;
    async fn exists(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError>;
    async fn create_slot(
        &self,
        slot: &Slot,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Slot, ServiceError>;
    async fn delete_slot(
        &self,
        id: &Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
    async fn update_slot(
        &self,
        slot: &Slot,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
