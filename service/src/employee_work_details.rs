use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use shifty_utils::{DayOfWeek, ShiftyDate, ShiftyDateUtilsError};
use time::{PrimitiveDateTime, Weekday};
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

#[derive(Clone, Debug, PartialEq)]
pub struct EmployeeWorkDetails {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub expected_hours: f32,
    pub from_day_of_week: DayOfWeek,
    pub from_calendar_week: u8,
    pub from_year: u32,
    pub to_day_of_week: DayOfWeek,
    pub to_calendar_week: u8,
    pub to_year: u32,
    pub workdays_per_week: u8,
    pub is_dynamic: bool,
    pub cap_planned_hours_to_expected: bool,
    pub committed_voluntary: f32,

    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,

    pub vacation_days: u8,

    pub created: Option<PrimitiveDateTime>,
    pub deleted: Option<PrimitiveDateTime>,
    pub version: Uuid,
}
impl From<&dao::employee_work_details::EmployeeWorkDetailsEntity> for EmployeeWorkDetails {
    fn from(working_hours: &dao::employee_work_details::EmployeeWorkDetailsEntity) -> Self {
        Self {
            id: working_hours.id,
            sales_person_id: working_hours.sales_person_id,
            expected_hours: working_hours.expected_hours,
            from_day_of_week: working_hours.from_day_of_week,
            from_calendar_week: working_hours.from_calendar_week,
            from_year: working_hours.from_year,
            to_day_of_week: working_hours.to_day_of_week,
            to_calendar_week: working_hours.to_calendar_week,
            to_year: working_hours.to_year,
            workdays_per_week: working_hours.workdays_per_week,
            is_dynamic: working_hours.is_dynamic,
            cap_planned_hours_to_expected: working_hours.cap_planned_hours_to_expected,
            committed_voluntary: working_hours.committed_voluntary,

            monday: working_hours.monday,
            tuesday: working_hours.tuesday,
            wednesday: working_hours.wednesday,
            thursday: working_hours.thursday,
            friday: working_hours.friday,
            saturday: working_hours.saturday,
            sunday: working_hours.sunday,

            vacation_days: working_hours.vacation_days,

            created: Some(working_hours.created),
            deleted: working_hours.deleted,
            version: working_hours.version,
        }
    }
}

impl EmployeeWorkDetails {
    pub fn potential_weekday_list(&self) -> Arc<[Weekday]> {
        let mut list = Vec::new();
        if self.monday {
            list.push(Weekday::Monday);
        }
        if self.tuesday {
            list.push(Weekday::Tuesday);
        }
        if self.wednesday {
            list.push(Weekday::Wednesday);
        }
        if self.thursday {
            list.push(Weekday::Thursday);
        }
        if self.friday {
            list.push(Weekday::Friday);
        }
        if self.saturday {
            list.push(Weekday::Saturday);
        }
        if self.sunday {
            list.push(Weekday::Sunday);
        }
        list.into()
    }

    pub fn potential_days_per_week(&self) -> u8 {
        self.potential_weekday_list().len() as u8
    }

    pub fn hours_per_day(&self) -> f32 {
        self.expected_hours / self.workdays_per_week as f32
    }

    pub fn holiday_hours(&self) -> f32 {
        self.expected_hours / self.potential_days_per_week() as f32
    }

    pub fn from_date(&self) -> Result<ShiftyDate, ShiftyDateUtilsError> {
        ShiftyDate::new(
            self.from_year,
            self.from_calendar_week,
            self.from_day_of_week,
        )
    }

    pub fn to_date(&self) -> Result<ShiftyDate, ShiftyDateUtilsError> {
        ShiftyDate::new(self.to_year, self.to_calendar_week, self.to_day_of_week)
    }

    pub fn with_from_date(&self, date: ShiftyDate) -> Self {
        Self {
            from_year: date.year(),
            from_calendar_week: date.week(),
            from_day_of_week: date.day_of_week(),
            ..self.clone()
        }
    }

    pub fn with_to_date(&self, date: ShiftyDate) -> Self {
        Self {
            to_year: date.year(),
            to_calendar_week: date.week(),
            to_day_of_week: date.day_of_week(),
            ..self.clone()
        }
    }

    pub fn has_day_of_week(&self, weekday: Weekday) -> bool {
        match weekday {
            Weekday::Monday => self.monday,
            Weekday::Tuesday => self.tuesday,
            Weekday::Wednesday => self.wednesday,
            Weekday::Thursday => self.thursday,
            Weekday::Friday => self.friday,
            Weekday::Saturday => self.saturday,
            Weekday::Sunday => self.sunday,
        }
    }

    pub fn vacation_days_for_year(&self, year: u32) -> f32 {
        let mut days = self.vacation_days as f32;
        let from_year = self
            .from_date()
            .map(|date| date.calendar_year())
            .unwrap_or(u32::MAX);
        let to_year = self
            .to_date()
            .map(|date| date.calendar_year())
            .unwrap_or(u32::MIN);
        if year < from_year || year > to_year {
            return 0.0;
        }
        if from_year == year {
            if let Ok(from_date) = self.from_date() {
                // Phase 28 (VAC-OFFSET-01 / D-28-04): subtract the fraction of
                // days STRICTLY before the contract start. A 1.1. start has
                // `ordinal() == 1` → 0 prior days → 0 subtraction. The previous
                // `ordinal()/days_in_year` over-subtracted ~1/365 of the annual
                // entitlement, occasionally tipping `.round()` from 18 to 17.
                let relation = (from_date.to_date().ordinal() as f32 - 1.0)
                    / time::util::days_in_year(year as i32) as f32;
                days -= self.vacation_days as f32 * relation;
            }
        }
        if to_year == year {
            if let Ok(to_date) = self.to_date() {
                let relation = 1.0
                    - to_date.to_date().ordinal() as f32
                        / time::util::days_in_year(year as i32) as f32;
                days -= self.vacation_days as f32 * relation as f32;
                //let month: u8 = to_date.month().into();
                //days -= self.vacation_days as f32 / 12.0 * (12 - month) as f32;
            }
        }
        days
    }
}

impl TryFrom<&EmployeeWorkDetails> for dao::employee_work_details::EmployeeWorkDetailsEntity {
    type Error = ServiceError;
    fn try_from(working_hours: &EmployeeWorkDetails) -> Result<Self, Self::Error> {
        Ok(Self {
            id: working_hours.id,
            sales_person_id: working_hours.sales_person_id,
            expected_hours: working_hours.expected_hours,
            from_day_of_week: working_hours.from_day_of_week,
            from_calendar_week: working_hours.from_calendar_week,
            from_year: working_hours.from_year,
            to_day_of_week: working_hours.to_day_of_week,
            to_calendar_week: working_hours.to_calendar_week,
            to_year: working_hours.to_year,
            workdays_per_week: working_hours.workdays_per_week,
            is_dynamic: working_hours.is_dynamic,
            cap_planned_hours_to_expected: working_hours.cap_planned_hours_to_expected,
            committed_voluntary: working_hours.committed_voluntary,

            monday: working_hours.monday,
            tuesday: working_hours.tuesday,
            wednesday: working_hours.wednesday,
            thursday: working_hours.thursday,
            friday: working_hours.friday,
            saturday: working_hours.saturday,
            sunday: working_hours.sunday,

            vacation_days: working_hours.vacation_days,

            created: working_hours
                .created
                .ok_or_else(|| ServiceError::InternalError)?,
            deleted: working_hours.deleted,
            version: working_hours.version,
        })
    }
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait EmployeeWorkDetailsService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError>;
    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError>;
    async fn find_for_week(
        &self,
        sales_person_id: Uuid,
        calendar_week: u8,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWorkDetails, ServiceError>;
    async fn all_for_week(
        &self,
        calendar_week: u8,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError>;
    async fn create(
        &self,
        entity: &EmployeeWorkDetails,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWorkDetails, ServiceError>;
    async fn update(
        &self,
        entity: &EmployeeWorkDetails,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWorkDetails, ServiceError>;
    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWorkDetails, ServiceError>;
}

#[cfg(test)]
mod vacation_days_for_year_tests {
    //! Phase 28 (VAC-OFFSET-01 / D-28-04) regression tests for
    //! [`EmployeeWorkDetails::vacation_days_for_year`].
    //!
    //! The year-START proration must subtract the fraction of days STRICTLY
    //! before the contract start. A 1.1. start therefore subtracts 0 (no prior
    //! days) — the old code subtracted `ordinal()/days_in_year` (~1/365 of the
    //! annual entitlement) which occasionally tipped `.round()` from 18 to 17.
    //! The year-END branch (`1.0 - ordinal/days_in_year`) is already correct and
    //! is pinned here so a future "symmetry" refactor cannot re-introduce a bug.
    use super::*;
    use shifty_utils::ShiftyDate;

    /// Build an [`EmployeeWorkDetails`] fixture whose contract spans the explicit
    /// `from`/`to` dates so the proration is fully deterministic.
    fn contract_with_dates(from: ShiftyDate, to: ShiftyDate, vacation_days: u8) -> EmployeeWorkDetails {
        EmployeeWorkDetails {
            id: Uuid::nil(),
            sales_person_id: Uuid::nil(),
            expected_hours: 40.0,
            from_day_of_week: from.day_of_week(),
            from_calendar_week: from.week(),
            from_year: from.year(),
            to_day_of_week: to.day_of_week(),
            to_calendar_week: to.week(),
            to_year: to.year(),
            workdays_per_week: 5,
            is_dynamic: false,
            cap_planned_hours_to_expected: false,
            committed_voluntary: 0.0,
            monday: true,
            tuesday: true,
            wednesday: true,
            thursday: true,
            friday: true,
            saturday: false,
            sunday: false,
            vacation_days,
            created: None,
            deleted: None,
            version: Uuid::nil(),
        }
    }

    /// Full-year contract (1.1.–31.12.) must return the full entitlement with NO
    /// proration. This is the core off-by-one regression (D-28-04): before the
    /// fix a 1.1. start over-subtracted ~1/365 of `vacation_days`.
    #[test]
    fn vacation_days_for_year_full_year_contract_no_proration() {
        let wd = contract_with_dates(
            ShiftyDate::first_day_in_year(2025),
            ShiftyDate::last_day_in_year(2025),
            18,
        );
        assert_eq!(
            wd.vacation_days_for_year(2025),
            18.0,
            "full-year contract must not be prorated (Phase 28 off-by-one fix)"
        );
    }

    /// A mid-year start (1.7.2025) subtracts the fraction of days STRICTLY before
    /// the start: `vacation_days * (ordinal_of_start - 1) / days_in_year`.
    #[test]
    fn vacation_days_for_year_mid_year_start_subtracts_prior_days() {
        let from = ShiftyDate::from_ymd(2025, 7, 1).unwrap();
        let wd = contract_with_dates(from, ShiftyDate::last_day_in_year(2025), 18);
        let ordinal = from.to_date().ordinal(); // 182 in 2025 (non-leap)
        let days_in_year = time::util::days_in_year(2025) as f32;
        let expected = 18.0 - 18.0 * (ordinal as f32 - 1.0) / days_in_year;
        let actual = wd.vacation_days_for_year(2025);
        assert!(
            (actual - expected).abs() < 1e-4,
            "mid-year start must subtract days strictly before start: expected {expected}, got {actual}"
        );
    }

    /// A 31.12. end subtracts 0 — the year-END branch is already correct and must
    /// stay untouched (Pitfall 4: do NOT "symmetrize" it).
    #[test]
    fn vacation_days_for_year_year_end_on_dec_31_subtracts_zero() {
        let wd = contract_with_dates(
            ShiftyDate::first_day_in_year(2025),
            ShiftyDate::from_ymd(2025, 12, 31).unwrap(),
            18,
        );
        assert_eq!(
            wd.vacation_days_for_year(2025),
            18.0,
            "a 31.12. end must subtract nothing"
        );
    }

    /// A year fully outside `[from_year, to_year]` returns 0.0 (unchanged).
    #[test]
    fn vacation_days_for_year_out_of_range_returns_zero() {
        let wd = contract_with_dates(
            ShiftyDate::first_day_in_year(2025),
            ShiftyDate::last_day_in_year(2025),
            18,
        );
        assert_eq!(wd.vacation_days_for_year(2024), 0.0);
        assert_eq!(wd.vacation_days_for_year(2026), 0.0);
    }

    /// Mid-year start AND mid-year end in the same year both prorate correctly
    /// (start subtracts prior days, end subtracts trailing days).
    #[test]
    fn vacation_days_for_year_single_year_both_bounds_prorate() {
        let from = ShiftyDate::from_ymd(2025, 4, 1).unwrap();
        let to = ShiftyDate::from_ymd(2025, 9, 30).unwrap();
        let wd = contract_with_dates(from, to, 24);
        let days_in_year = time::util::days_in_year(2025) as f32;
        let start_cut = 24.0 * (from.to_date().ordinal() as f32 - 1.0) / days_in_year;
        let end_cut = 24.0 * (1.0 - to.to_date().ordinal() as f32 / days_in_year);
        let expected = 24.0 - start_cut - end_cut;
        let actual = wd.vacation_days_for_year(2025);
        assert!(
            (actual - expected).abs() < 1e-4,
            "both-bounds proration mismatch: expected {expected}, got {actual}"
        );
    }
}
