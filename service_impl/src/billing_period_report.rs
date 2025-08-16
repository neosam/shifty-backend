use std::collections::BTreeMap;

use async_trait::async_trait;
use dao::TransactionDao;
use service::billing_period::{
    BillingPeriod, BillingPeriodSalesPerson, BillingPeriodService, BillingPeriodValue,
    BillingPeriodValueType,
};
use service::billing_period_report::BillingPeriodReportService;
use service::clock::ClockService;
use service::permission::Authentication;
use service::reporting::ReportingService;
use service::sales_person::{SalesPerson, SalesPersonService};
use service::uuid_service::UuidService;
use service::ServiceError;
use shifty_utils::ShiftyDate;
use time::macros::datetime;
use uuid::Uuid;

use crate::gen_service_impl;

const BILLING_PERIOD_REPORT_SERVICE: &str = "BillingPeriodReportService";

gen_service_impl! {
    struct BillingPeriodReportServiceImpl: BillingPeriodReportService = BillingPeriodReportServiceDeps {
        BillingPeriodService: BillingPeriodService<Context = Self::Context, Transaction = Self::Transaction> = billing_period_service,
        ReportingService: ReportingService<Context = Self::Context, Transaction = Self::Transaction> = reporting_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        UuidService: UuidService = uuid_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

impl<Deps: BillingPeriodReportServiceDeps> BillingPeriodReportServiceImpl<Deps> {
    pub async fn build_billing_period_report_for_sales_person(
        &self,
        sales_person: SalesPerson,
        start_date: ShiftyDate,
        end_date: ShiftyDate,
        context: Authentication<Deps::Context>,
        tx: Option<Deps::Transaction>,
    ) -> Result<BillingPeriodSalesPerson, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let report_start = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person.id,
                ShiftyDate::first_day_in_year(start_date.calendar_year()),
                start_date.previous_day(),
                true,
                context.clone(),
                tx.clone().into(),
            )
            .await?;
        let report_end = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person.id,
                ShiftyDate::first_day_in_year(end_date.calendar_year()),
                end_date,
                true,
                context.clone(),
                tx.clone().into(),
            )
            .await?;
        let report_end_of_year = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person.id,
                ShiftyDate::first_day_in_year(end_date.calendar_year()),
                ShiftyDate::last_day_in_year(end_date.calendar_year()),
                true,
                context.clone(),
                tx.clone().into(),
            )
            .await?;
        let report_delta = self
            .reporting_service
            .get_report_for_employee_range(
                &sales_person.id,
                start_date,
                end_date,
                false,
                context.clone(),
                tx.clone().into(),
            )
            .await?;

        let mut billing_period_values = BTreeMap::new();
        billing_period_values.insert(
            BillingPeriodValueType::Overall,
            BillingPeriodValue {
                value_delta: report_delta.overall_hours,
                value_ytd_from: report_start.overall_hours,
                value_ytd_to: report_end.overall_hours,
                value_full_year: report_end_of_year.overall_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::Balance,
            BillingPeriodValue {
                value_delta: report_delta.balance_hours,
                value_ytd_from: report_start.balance_hours,
                value_ytd_to: report_end.balance_hours,
                value_full_year: report_end_of_year.balance_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::ExpectedHours,
            BillingPeriodValue {
                value_delta: report_delta.expected_hours,
                value_ytd_from: report_start.expected_hours,
                value_ytd_to: report_end.expected_hours,
                value_full_year: report_end_of_year.expected_hours,
            },
        );
        for custom_hours in report_delta.custom_extra_hours.iter() {
            let ytd_from_value = report_start
                .custom_extra_hours
                .iter()
                .find(|ch| ch.name == custom_hours.name)
                .map_or(0.0, |ch| ch.hours);
            let ytd_to_value = report_end
                .custom_extra_hours
                .iter()
                .find(|ch| ch.name == custom_hours.name)
                .map_or(0.0, |ch| ch.hours);
            let full_year_value = report_end_of_year
                .custom_extra_hours
                .iter()
                .find(|ch| ch.name == custom_hours.name)
                .map_or(0.0, |ch| ch.hours);

            billing_period_values.insert(
                BillingPeriodValueType::CustomExtraHours(custom_hours.name.clone()),
                BillingPeriodValue {
                    value_delta: custom_hours.hours,
                    value_ytd_from: ytd_from_value,
                    value_ytd_to: ytd_to_value,
                    value_full_year: full_year_value,
                },
            );
        }

        Ok(BillingPeriodSalesPerson {
            id: Uuid::nil(),
            sales_person_id: sales_person.id,
            values: billing_period_values,
            created_at: datetime!(1970-01-01 00:00:00),
            created_by: "".into(),
            deleted_at: None,
            deleted_by: None,
        })
    }
}

#[async_trait]
impl<Deps: BillingPeriodReportServiceDeps> BillingPeriodReportService
    for BillingPeriodReportServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn build_new_billing_period(
        &self,
        end_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BillingPeriod, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let start_date = self
            .billing_period_service
            .get_latest_billing_period_end_date(context.clone(), tx.clone().into())
            .await?
            .unwrap_or(ShiftyDate::from_date(
                time::OffsetDateTime::UNIX_EPOCH.date(),
            ))
            .next_day();

        let sales_persons = self
            .sales_person_service
            .get_all(context.clone(), tx.clone().into())
            .await?;

        let mut sales_person_reports = Vec::new();
        for sales_person in sales_persons.iter() {
            let sales_person_report = self
                .build_billing_period_report_for_sales_person(
                    sales_person.clone(),
                    start_date,
                    end_date,
                    context.clone(),
                    tx.clone().into(),
                )
                .await?;
            sales_person_reports.push(sales_person_report);
        }

        let billing_period = BillingPeriod {
            id: Uuid::nil(),
            start_date,
            end_date,
            sales_persons: sales_person_reports.into(),
            created_at: datetime!(1970-01-01 00:00:00),
            created_by: "".into(),
            deleted_at: None,
            deleted_by: None,
        };

        Ok(billing_period)
    }

    async fn build_and_persist_billing_period_report(
        &self,
        end_date: ShiftyDate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Uuid, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let billing_period = self
            .build_new_billing_period(end_date, context.clone(), tx.clone().into())
            .await?;
        let billing_period_id = billing_period.id;

        self.billing_period_service
            .create_billing_period(
                &billing_period,
                BILLING_PERIOD_REPORT_SERVICE,
                context,
                tx.clone().into(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(billing_period_id)
    }
}
