use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::DaoError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    pub fn from_number(number: u8) -> Option<Self> {
        match number {
            1 => Some(DayOfWeek::Monday),
            2 => Some(DayOfWeek::Tuesday),
            3 => Some(DayOfWeek::Wednesday),
            4 => Some(DayOfWeek::Thursday),
            5 => Some(DayOfWeek::Friday),
            6 => Some(DayOfWeek::Saturday),
            7 => Some(DayOfWeek::Sunday),
            _ => None,
        }
    }
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

#[derive(Debug, PartialEq, Eq)]
pub struct SlotEntity {
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

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait SlotDao {
    type Transaction: crate::Transaction;

    async fn get_slots(&self, tx: Self::Transaction) -> Result<Arc<[SlotEntity]>, DaoError>;
    async fn get_slot(
        &self,
        id: &Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<SlotEntity>, DaoError>;
    async fn get_slots_for_week(
        &self,
        year: u32,
        week: u8,
        tx: Self::Transaction,
    ) -> Result<Arc<[SlotEntity]>, DaoError>;
    async fn create_slot(
        &self,
        slot: &SlotEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    //async fn delete_slot(&self, id: &Uuid, process: &str) -> Result<(), DaoError>;
    async fn update_slot(
        &self,
        slot: &SlotEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
