//! This module provides functionality for managing extra hours worked by employees.
//!
//! It handles various categories of extra hours including:
//! - Extra work (overtime)
//! - Vacation time
//! - Sick leave
//! - Holidays
//! - Unavailability
//!
//! The module defines data structures and service interfaces for creating,
//! retrieving, updating, and deleting extra hours records, as well as
//! determining how these hours affect reporting and employee availability.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use shifty_utils::LazyLoad;
use shifty_utils::ShiftyDate;
use uuid::Uuid;

use crate::{custom_extra_hours::CustomExtraHours, permission::Authentication, ServiceError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReportType {
    WorkingHours,
    AbsenceHours,
    Documented,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Availability {
    Available,
    Unavailable,
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExtraHoursCategory {
    ExtraWork,
    Vacation,
    SickLeave,
    Holiday,
    Unavailable,
    UnpaidLeave,
    VolunteerWork,
    CustomExtraHours(LazyLoad<Uuid, CustomExtraHours>),
}

/// Phase 54 (D-54-DM-02): Rebooking-Marker auf jeder `ExtraHours`-Row.
/// F1-Ist- + F2-Soll-Aggregatoren MUESSEN `Rebooking`-Rows herausfiltern
/// (Pitfall 1: Doppel-Zaehlung), sonst zaehlen +N/-N-Paare bei Rebooking
/// die freiwilligen Stunden doppelt in Reports / Balance.
///
/// Bestandsrows landen per SQL-DEFAULT auf `Manual`; Rebooking-Schreiber
/// folgen ab Phase 55.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtraHoursSource {
    /// Row wurde manuell (UI-Add-Extra-Hours, HR-CRUD, Absence-Convert)
    /// erzeugt und zaehlt normal in Balance / F1-Ist / F2-Nutz.
    Manual,
    /// Row wurde durch Rebooking (F3/F4/F5, ab Phase 55) erzeugt und
    /// MUSS von F1-Ist- + F2-Soll-Aggregatoren gefiltert werden
    /// (D-54-DM-02, Pitfall 1).
    Rebooking,
}

impl ExtraHoursSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Rebooking => "rebooking",
        }
    }
}

impl TryFrom<&str> for ExtraHoursSource {
    type Error = ServiceError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "manual" => Ok(Self::Manual),
            "rebooking" => Ok(Self::Rebooking),
            _ => Err(ServiceError::InternalError),
        }
    }
}

impl Default for ExtraHoursSource {
    fn default() -> Self {
        Self::Manual
    }
}
impl ExtraHoursCategory {
    pub fn as_report_type(&self) -> ReportType {
        match self {
            Self::ExtraWork => ReportType::WorkingHours,
            Self::Vacation => ReportType::AbsenceHours,
            Self::SickLeave => ReportType::AbsenceHours,
            Self::Holiday => ReportType::AbsenceHours,
            Self::UnpaidLeave => ReportType::AbsenceHours,
            Self::Unavailable => ReportType::None,
            Self::VolunteerWork => ReportType::Documented,
            Self::CustomExtraHours(custom_extra_hours) => {
                if let Some(custom_extra_hours) = custom_extra_hours.get() {
                    if custom_extra_hours.modifies_balance {
                        ReportType::WorkingHours
                    } else {
                        ReportType::None
                    }
                } else {
                    ReportType::None
                }
            }
        }
    }

    pub fn availability(&self) -> Availability {
        match self {
            Self::ExtraWork => Availability::Available,
            Self::Vacation => Availability::Unavailable,
            Self::SickLeave => Availability::Unavailable,
            Self::Holiday => Availability::Unavailable,
            Self::UnpaidLeave => Availability::Unavailable,
            Self::Unavailable => Availability::Unavailable,
            Self::VolunteerWork => Availability::Available,
            Self::CustomExtraHours(hours) => {
                if let Some(hours) = hours.get() {
                    if hours.modifies_balance {
                        Availability::Available
                    } else {
                        Availability::None
                    }
                } else {
                    Availability::None
                }
            }
        }
    }
}

impl From<&dao::extra_hours::ExtraHoursCategoryEntity> for ExtraHoursCategory {
    fn from(category: &dao::extra_hours::ExtraHoursCategoryEntity) -> Self {
        match category {
            dao::extra_hours::ExtraHoursCategoryEntity::ExtraWork => Self::ExtraWork,
            dao::extra_hours::ExtraHoursCategoryEntity::Vacation => Self::Vacation,
            dao::extra_hours::ExtraHoursCategoryEntity::SickLeave => Self::SickLeave,
            dao::extra_hours::ExtraHoursCategoryEntity::Holiday => Self::Holiday,
            dao::extra_hours::ExtraHoursCategoryEntity::Unavailable => Self::Unavailable,
            dao::extra_hours::ExtraHoursCategoryEntity::UnpaidLeave => Self::UnpaidLeave,
            dao::extra_hours::ExtraHoursCategoryEntity::VolunteerWork => Self::VolunteerWork,
            dao::extra_hours::ExtraHoursCategoryEntity::Custom(id) => {
                Self::CustomExtraHours(LazyLoad::new(*id))
            }
        }
    }
}
impl From<&ExtraHoursCategory> for dao::extra_hours::ExtraHoursCategoryEntity {
    fn from(category: &ExtraHoursCategory) -> Self {
        match category {
            ExtraHoursCategory::ExtraWork => Self::ExtraWork,
            ExtraHoursCategory::Vacation => Self::Vacation,
            ExtraHoursCategory::SickLeave => Self::SickLeave,
            ExtraHoursCategory::Holiday => Self::Holiday,
            ExtraHoursCategory::Unavailable => Self::Unavailable,
            ExtraHoursCategory::UnpaidLeave => Self::UnpaidLeave,
            ExtraHoursCategory::VolunteerWork => Self::VolunteerWork,
            ExtraHoursCategory::CustomExtraHours(lazy) => Self::Custom(*lazy.key()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtraHours {
    /// Externally-stable id. Maps to the `logical_id` column on the persistence layer
    /// (which equals the physical row id for the first version of the entry).
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub amount: f32,
    pub category: ExtraHoursCategory,
    pub description: Arc<str>,
    pub date_time: time::PrimitiveDateTime,
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
    /// Phase 54 (D-54-DM-02): Rebooking-Marker. F1/F2-Aggregatoren filtern
    /// `Rebooking`-Rows raus. Neue Rows (manuell, HR-CRUD, Absence-Convert)
    /// muessen `Manual` setzen; Rebooking-Schreiber (F3/F4/F5 ab Phase 55)
    /// setzen `Rebooking`.
    pub source: ExtraHoursSource,
}
impl From<&dao::extra_hours::ExtraHoursEntity> for ExtraHours {
    fn from(extra_hours: &dao::extra_hours::ExtraHoursEntity) -> Self {
        Self {
            id: extra_hours.logical_id,
            sales_person_id: extra_hours.sales_person_id,
            amount: extra_hours.amount,
            category: (&extra_hours.category).into(),
            description: extra_hours.description.clone(),
            date_time: extra_hours.date_time,
            created: Some(extra_hours.created),
            deleted: extra_hours.deleted,
            version: extra_hours.version,
            source: ExtraHoursSource::try_from(extra_hours.source.as_str())
                .unwrap_or(ExtraHoursSource::Manual),
        }
    }
}
impl TryFrom<&ExtraHours> for dao::extra_hours::ExtraHoursEntity {
    type Error = ServiceError;
    fn try_from(extra_hours: &ExtraHours) -> Result<Self, Self::Error> {
        Ok(Self {
            id: extra_hours.id,
            logical_id: extra_hours.id,
            sales_person_id: extra_hours.sales_person_id,
            amount: extra_hours.amount,
            category: (&extra_hours.category).into(),
            description: extra_hours.description.clone(),
            date_time: extra_hours.date_time,
            created: extra_hours
                .created
                .ok_or_else(|| ServiceError::InternalError)?,
            deleted: extra_hours.deleted,
            version: extra_hours.version,
            source: extra_hours.source.as_str().to_string(),
        })
    }
}

impl ExtraHours {
    pub fn to_date(&self) -> ShiftyDate {
        ShiftyDate::from(self.date_time)
    }
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait ExtraHoursService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ExtraHours]>, ServiceError>;

    async fn find_by_sales_person_id_and_year_range(
        &self,
        sales_person_id: Uuid,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ExtraHours]>, ServiceError>;

    async fn find_by_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ExtraHours]>, ServiceError>;

    /// Phase 52 Follow-up #3 — ISO-Wochenjahr-Batch analog zu
    /// [`Self::find_by_week`].
    ///
    /// Liefert alle nicht-gelöschten `ExtraHours` deren `date_time` in einer
    /// der ISO-Wochen von `year` liegt (`[ISO-Mo(y,1), ISO-Su(y,weeks_in_year(y))+1d)`).
    /// Symmetrisch zum `find_by_week`-Pattern, gedacht als Bulk-Load-Fundament
    /// für Wave 4 (`get_weekly_summary` / `reporting.get_year`), wo heute
    /// 55×`find_by_week` sequenziell iteriert werden.
    ///
    /// Ersetzt das alte `find_by_year` (Kalender-Jahr) aus Phase 52 Wave 3:
    /// dessen Kalender-Range verschluckte an KW 1 / KW 53 Rows, deren
    /// Kalender-Datum ≠ ISO-Wochenjahr ist.
    ///
    /// Auth: identisch zu `find_by_week` — `check_only_full_authentication`
    /// (Cross-Service-Konsumenten mit `Authentication::Full`).
    async fn find_by_iso_year(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ExtraHours]>, ServiceError>;

    async fn create(
        &self,
        entity: &ExtraHours,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ExtraHours, ServiceError>;
    async fn update(
        &self,
        entity: &ExtraHours,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ExtraHours, ServiceError>;
    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    /// Bulk soft-delete (Phase 4 cutover, C-Phase4-04). Marks every id as
    /// `deleted = NOW()` with a caller-provided `update_process` tag for audit.
    /// Bypasses per-row permission checks: caller MUST hold `cutover_admin` and
    /// pass the cutover-tx as `Some(tx)`. ANY id not present is silently
    /// ignored (idempotent for re-runs).
    async fn soft_delete_bulk(
        &self,
        ids: Arc<[Uuid]>,
        update_process: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unpaid_leave_report_type_is_absence_hours() {
        assert_eq!(
            ExtraHoursCategory::UnpaidLeave.as_report_type(),
            ReportType::AbsenceHours
        );
    }

    #[test]
    fn test_unpaid_leave_availability_is_unavailable() {
        assert_eq!(
            ExtraHoursCategory::UnpaidLeave.availability(),
            Availability::Unavailable
        );
    }
}
