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

    /// Clear all billing periods (soft delete).
    async fn clear_all_billing_periods(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
