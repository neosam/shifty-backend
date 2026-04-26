use crate::permission::Authentication;
use crate::ServiceError;
use async_trait::async_trait;
use dao::billing_period::BillingPeriodEntity;
use dao::billing_period_sales_person::BillingPeriodSalesPersonEntity;
use mockall::automock;
use shifty_utils::ShiftyDate;
use std::{collections::BTreeMap, sync::Arc};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BillingPeriodValueTypeParseError {
    #[error("Invalid billing period value type: {0}")]
    InvalidValueType(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BillingPeriod {
    pub id: uuid::Uuid,
    pub start_date: ShiftyDate,
    pub end_date: ShiftyDate,
    pub snapshot_schema_version: u32,

    pub sales_persons: Arc<[BillingPeriodSalesPerson]>,

    pub created_at: time::PrimitiveDateTime,
    pub created_by: Arc<str>,
    pub deleted_at: Option<time::PrimitiveDateTime>,
    pub deleted_by: Option<Arc<str>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BillingPeriodValueType {
    Balance,
    Overall,
    ExpectedHours,
    ExtraWork,
    VacationHours,
    SickLeave,
    Holiday,
    Volunteer,
    CustomExtraHours(Arc<str>),
    VacationDays,
    VacationEntitlement,
}
impl BillingPeriodValueType {
    pub fn as_str(&self) -> Arc<str> {
        match self {
            BillingPeriodValueType::Balance => "balance".into(),
            BillingPeriodValueType::Overall => "overall".into(),
            BillingPeriodValueType::ExpectedHours => "expected_hours".into(),
            BillingPeriodValueType::ExtraWork => "extra_work".into(),
            BillingPeriodValueType::VacationHours => "vacation_hours".into(),
            BillingPeriodValueType::SickLeave => "sick_leave".into(),
            BillingPeriodValueType::Holiday => "holiday".into(),
            BillingPeriodValueType::Volunteer => "volunteer".into(),
            BillingPeriodValueType::CustomExtraHours(s) => {
                format!("custom_extra_hours:{}", s).into()
            }
            BillingPeriodValueType::VacationDays => "vacation_days".into(),
            BillingPeriodValueType::VacationEntitlement => "vacation_entitlement".into(),
        }
    }
}
impl FromStr for BillingPeriodValueType {
    type Err = BillingPeriodValueTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        match s {
            "overall" => Ok(BillingPeriodValueType::Overall),
            "expected_hours" => Ok(BillingPeriodValueType::ExpectedHours),
            "balance" => Ok(BillingPeriodValueType::Balance),
            "extra_work" => Ok(BillingPeriodValueType::ExtraWork),
            "vacation_hours" => Ok(BillingPeriodValueType::VacationHours),
            "sick_leave" => Ok(BillingPeriodValueType::SickLeave),
            "holiday" => Ok(BillingPeriodValueType::Holiday),
            "volunteer" => Ok(BillingPeriodValueType::Volunteer),
            "vacation_days" => Ok(BillingPeriodValueType::VacationDays),
            "vacation_entitlement" => Ok(BillingPeriodValueType::VacationEntitlement),
            _ if s.starts_with("custom_extra_hours:") => {
                Ok(BillingPeriodValueType::CustomExtraHours(Arc::from(
                    s.trim_start_matches("custom_extra_hours:"),
                )))
            }
            _ => Err(BillingPeriodValueTypeParseError::InvalidValueType(s.into())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BillingPeriodValue {
    pub value_delta: f32,
    pub value_ytd_from: f32,
    pub value_ytd_to: f32,
    pub value_full_year: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BillingPeriodSalesPerson {
    pub id: uuid::Uuid,
    pub sales_person_id: uuid::Uuid,

    pub values: BTreeMap<BillingPeriodValueType, BillingPeriodValue>,

    pub created_at: time::PrimitiveDateTime,
    pub created_by: Arc<str>,
    pub deleted_at: Option<time::PrimitiveDateTime>,
    pub deleted_by: Option<Arc<str>>,
}

impl BillingPeriod {
    pub fn from_billing_period_entity(
        entity: &BillingPeriodEntity,
        sales_persons: Arc<[BillingPeriodSalesPerson]>,
    ) -> Self {
        Self {
            id: entity.id,
            start_date: ShiftyDate::from_date(entity.start_date),
            end_date: ShiftyDate::from_date(entity.end_date),
            snapshot_schema_version: entity.snapshot_schema_version,
            sales_persons,
            created_at: entity.created_at,
            created_by: Arc::clone(&entity.created_by),
            deleted_at: entity.deleted_at,
            deleted_by: entity.deleted_by.clone(),
        }
    }
}

impl BillingPeriodSalesPerson {
    pub fn from_entities(entity: &[BillingPeriodSalesPersonEntity]) -> Option<Self> {
        let mut values = BTreeMap::new();
        for sp in entity.iter() {
            if let Ok(value_type) = BillingPeriodValueType::from_str(&sp.value_type) {
                values.insert(
                    value_type,
                    BillingPeriodValue {
                        value_delta: sp.value_delta,
                        value_ytd_from: sp.value_ytd_from,
                        value_ytd_to: sp.value_ytd_to,
                        value_full_year: sp.value_full_year,
                    },
                );
            }
        }

        if entity.is_empty() {
            return None;
        }
        Some(Self {
            id: entity[0].id,
            sales_person_id: entity[0].sales_person_id,
            values,
            created_at: entity[0].created_at,
            created_by: Arc::clone(&entity[0].created_by),
            deleted_at: entity[0].deleted_at,
            deleted_by: entity[0].deleted_by.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dao::billing_period_sales_person::BillingPeriodSalesPersonEntity;
    use time::macros::datetime;

    #[test]
    fn volunteer_value_type_round_trips_through_as_str_and_from_str() {
        let original = BillingPeriodValueType::Volunteer;
        let s = original.as_str();
        assert_eq!(&*s, "volunteer");
        let parsed = BillingPeriodValueType::from_str(&s).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn volunteer_row_round_trips_through_from_entities() {
        // Spec volunteer-work-hours Req 7 Scenario "Persisted volunteer rows
        // round-trip through service-layer load": a row with value_type =
        // "volunteer" must surface as BillingPeriodValueType::Volunteer with
        // the expected value_delta after going through from_entities.
        let id = uuid::Uuid::new_v4();
        let sales_person_id = uuid::Uuid::new_v4();
        let now = datetime!(2024-01-01 10:00:00);
        let entity = BillingPeriodSalesPersonEntity {
            id,
            billing_period_id: uuid::Uuid::new_v4(),
            sales_person_id,
            value_type: "volunteer".into(),
            value_delta: 8.0,
            value_ytd_from: 0.0,
            value_ytd_to: 8.0,
            value_full_year: 8.0,
            created_at: now,
            created_by: "test".into(),
            deleted_at: None,
            deleted_by: None,
        };

        let result = BillingPeriodSalesPerson::from_entities(&[entity]).unwrap();
        let value = result
            .values
            .get(&BillingPeriodValueType::Volunteer)
            .expect("volunteer row must round-trip without being silently dropped");
        assert!((value.value_delta - 8.0).abs() < 0.01);
    }
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait BillingPeriodService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Returns all not deleted `BillingPeriod`s but no sales person data.
    async fn get_billing_period_overview(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[BillingPeriod]>, ServiceError>;

    /// Returns the `BillingPeriod` with all sales person data for the given ID.
    async fn get_billing_period_by_id(
        &self,
        id: uuid::Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BillingPeriod, ServiceError>;

    /// Creates a new `BillingPeriod` with the given sales person data.
    async fn create_billing_period(
        &self,
        entity: &BillingPeriod,
        process: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BillingPeriod, ServiceError>;

    /// Get latest `BillingPeriod` end date.
    async fn get_latest_billing_period_end_date(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<ShiftyDate>, ServiceError>;

    /// Soft-delete a single billing period by ID. Only the latest (most recent) billing period can be deleted.
    async fn delete_billing_period(
        &self,
        id: uuid::Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    /// Clear all billing periods (soft delete).
    async fn clear_all_billing_periods(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
