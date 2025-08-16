use crate::ServiceError;
use crate::{billing_period::BillingPeriod, permission::Authentication};
use async_trait::async_trait;
use mockall::automock;
use shifty_utils::ShiftyDate;
use uuid::Uuid;

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait BillingPeriodReportService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Generate new billing period based on new end date
    ///
    /// The period goes one day ofter the last end date until the new end date.
    /// If the new end date is before the last end date, an error is returned.
    /// If it is the first billing period, it will be set to 2020-01-01.
    ///
    /// Only HR is allowed to create a new billing period.
    async fn build_new_billing_period(
        &self,
        end_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BillingPeriod, ServiceError>;

    /// Build and persist a new billing period report based on the end date
    ///
    /// Returns the new billing period report ID.
    ///
    /// Only HR is allowed to build a new billing period report.
    async fn build_and_persist_billing_period_report(
        &self,
        end_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Uuid, ServiceError>;

    /// Generate a custom report using a text template
    ///
    /// Loads the text template with the given ID, loads the billing report
    /// with the given ID, and uses Tera to render the template with the
    /// billing report data.
    ///
    /// Only HR is allowed to generate custom reports.
    async fn generate_custom_report(
        &self,
        template_id: Uuid,
        billing_period_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<std::sync::Arc<str>, ServiceError>;
}
