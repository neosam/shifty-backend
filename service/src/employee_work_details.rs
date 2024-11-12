use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use time::{PrimitiveDateTime, Weekday};
use uuid::Uuid;

use crate::permission::Authentication;
use crate::slot::DayOfWeek;
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
            from_day_of_week: working_hours.from_day_of_week.into(),
            from_calendar_week: working_hours.from_calendar_week,
            from_year: working_hours.from_year,
            to_day_of_week: working_hours.to_day_of_week.into(),
            to_calendar_week: working_hours.to_calendar_week,
            to_year: working_hours.to_year,
            workdays_per_week: working_hours.workdays_per_week,

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

    pub fn vacation_days_for_year(&self, year: u32) -> f32 {
        let mut days = self.vacation_days as f32;
        if self.from_year == year {
            if let Ok(from_date) = time::Date::from_iso_week_date(
                year as i32,
                self.from_calendar_week,
                self.from_day_of_week.into(),
            ) {
                let relation =
                    from_date.ordinal() as f32 / time::util::days_in_year(year as i32) as f32;
                days -= self.vacation_days as f32 * relation as f32;
                //let month: u8 = from_date.month().into();
                //days -= self.vacation_days as f32 / 12.0 * (month - 1) as f32;
            }
        }
        if self.to_year == year {
            if let Ok(to_date) = time::Date::from_iso_week_date(
                year as i32,
                self.to_calendar_week,
                self.to_day_of_week.into(),
            ) {
                let relation =
                    1.0 - to_date.ordinal() as f32 / time::util::days_in_year(year as i32) as f32;
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
            from_day_of_week: working_hours.from_day_of_week.into(),
            from_calendar_week: working_hours.from_calendar_week,
            from_year: working_hours.from_year,
            to_day_of_week: working_hours.to_day_of_week.into(),
            to_calendar_week: working_hours.to_calendar_week,
            to_year: working_hours.to_year,
            workdays_per_week: working_hours.workdays_per_week,

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

#[automock(type Context=();)]
#[async_trait]
pub trait EmployeeWorkDetailsService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    async fn all(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError>;
    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError>;
    async fn find_for_week(
        &self,
        sales_person_id: Uuid,
        calendar_week: u8,
        year: u32,
        context: Authentication<Self::Context>,
    ) -> Result<EmployeeWorkDetails, ServiceError>;
    async fn all_for_week(
        &self,
        calendar_week: u8,
        year: u32,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError>;
    async fn create(
        &self,
        entity: &EmployeeWorkDetails,
        context: Authentication<Self::Context>,
    ) -> Result<EmployeeWorkDetails, ServiceError>;
    async fn update(
        &self,
        entity: &EmployeeWorkDetails,
        context: Authentication<Self::Context>,
    ) -> Result<EmployeeWorkDetails, ServiceError>;
    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<EmployeeWorkDetails, ServiceError>;
}
