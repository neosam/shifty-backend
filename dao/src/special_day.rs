use crate::DaoError;
use async_trait::async_trait;
use mockall::automock;
use shifty_utils::DayOfWeek;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecialDayTypeEntity {
    Holiday,
    ShortDay,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecialDayEntity {
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub day_of_week: DayOfWeek,
    pub day_type: SpecialDayTypeEntity,
    pub time_of_day: Option<time::Time>,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock]
#[async_trait]
pub trait SpecialDayDao {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SpecialDayEntity>, DaoError>;
    async fn find_by_week(
        &self,
        year: u32,
        calendar_week: u8,
    ) -> Result<Arc<[SpecialDayEntity]>, DaoError>;
    async fn find_by_year(&self, year: u32) -> Result<Arc<[SpecialDayEntity]>, DaoError>;
    /// Phase 52 Follow-up #3 — ISO-Wochenjahr-Batch.
    ///
    /// **Wichtig:** die DB-Spalte `year` speichert bereits das ISO-Wochenjahr
    /// (siehe `create`-Pfad: `ShiftyDate::from_date(...)`). D.h. das `WHERE
    /// year = ?`-Filter matched semantisch das ISO-Wochenjahr — der einzige
    /// Unterschied zu `find_by_year` ist der Vertrag ans Aufruferseite: ein
    /// Konsument, der ISO-basierte Wochen-Buckets baut, muss diese Variante
    /// nutzen, um Boundary-Rows (Feiertag am 2027-01-01 = ISO-2026-W53-Fri)
    /// nicht zu verpassen.
    async fn find_by_iso_year(&self, year: u32) -> Result<Arc<[SpecialDayEntity]>, DaoError>;
    async fn create(&self, entity: &SpecialDayEntity, process: &str) -> Result<(), DaoError>;
    async fn update(&self, entity: &SpecialDayEntity, process: &str) -> Result<(), DaoError>;
}
