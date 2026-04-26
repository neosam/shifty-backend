use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use service::text_template::TemplateEngine;
use tera::{Context, Tera};
use dao::TransactionDao;
use service::billing_period::{
    BillingPeriod, BillingPeriodSalesPerson, BillingPeriodService, BillingPeriodValue,
    BillingPeriodValueType,
};
use service::billing_period_report::BillingPeriodReportService;
use service::clock::ClockService;
use service::employee_work_details::EmployeeWorkDetailsService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::reporting::ReportingService;
use service::sales_person::{SalesPerson, SalesPersonService};
use service::text_template::TextTemplateService;
use service::uuid_service::UuidService;
use service::PermissionService;
use service::ServiceError;
use shifty_utils::ShiftyDate;
use time::macros::datetime;
use uuid::Uuid;

use crate::gen_service_impl;

const BILLING_PERIOD_REPORT_SERVICE: &str = "BillingPeriodReportService";

/// Schema version stamped on every newly persisted `billing_period` snapshot.
///
/// Bump by one whenever you add, remove, rename, or change the computation of
/// any persisted `value_type` on `billing_period_sales_person`. See
/// `CLAUDE.md` → "Billing Period Snapshot Schema Versioning" for the rationale
/// and the full list of trigger conditions.
pub const CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 2;

gen_service_impl! {
    struct BillingPeriodReportServiceImpl: BillingPeriodReportService = BillingPeriodReportServiceDeps {
        BillingPeriodService: BillingPeriodService<Context = Self::Context, Transaction = Self::Transaction> = billing_period_service,
        ReportingService: ReportingService<Context = Self::Context, Transaction = Self::Transaction> = reporting_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Context = Self::Context, Transaction = Self::Transaction> = employee_work_details_service,
        TextTemplateService: TextTemplateService<Context = Self::Context, Transaction = Self::Transaction> = text_template_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
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
        billing_period_values.insert(
            BillingPeriodValueType::ExtraWork,
            BillingPeriodValue {
                value_delta: report_delta.extra_work_hours,
                value_ytd_from: report_start.extra_work_hours,
                value_ytd_to: report_end.extra_work_hours,
                value_full_year: report_end_of_year.extra_work_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::VacationHours,
            BillingPeriodValue {
                value_delta: report_delta.vacation_hours,
                value_ytd_from: report_start.vacation_hours,
                value_ytd_to: report_end.vacation_hours,
                value_full_year: report_end_of_year.vacation_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::SickLeave,
            BillingPeriodValue {
                value_delta: report_delta.sick_leave_hours,
                value_ytd_from: report_start.sick_leave_hours,
                value_ytd_to: report_end.sick_leave_hours,
                value_full_year: report_end_of_year.sick_leave_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::Holiday,
            BillingPeriodValue {
                value_delta: report_delta.holiday_hours,
                value_ytd_from: report_start.holiday_hours,
                value_ytd_to: report_end.holiday_hours,
                value_full_year: report_end_of_year.holiday_hours,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::VacationDays,
            BillingPeriodValue {
                value_delta: report_delta.vacation_days,
                value_ytd_from: report_start.vacation_days,
                value_ytd_to: report_end.vacation_days,
                value_full_year: report_end_of_year.vacation_days,
            },
        );
        billing_period_values.insert(
            BillingPeriodValueType::VacationEntitlement,
            BillingPeriodValue {
                value_delta: report_delta.vacation_entitlement,
                value_ytd_from: report_start.vacation_entitlement,
                value_ytd_to: report_end.vacation_entitlement,
                value_full_year: report_end_of_year.vacation_entitlement,
            },
        );
        if report_delta.volunteer_hours != 0.0 {
            billing_period_values.insert(
                BillingPeriodValueType::Volunteer,
                BillingPeriodValue {
                    value_delta: report_delta.volunteer_hours,
                    value_ytd_from: report_start.volunteer_hours,
                    value_ytd_to: report_end.volunteer_hours,
                    value_full_year: report_end_of_year.volunteer_hours,
                },
            );
        }
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
            snapshot_schema_version: CURRENT_SNAPSHOT_SCHEMA_VERSION,
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

    async fn generate_custom_report(
        &self,
        template_id: Uuid,
        billing_period_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<str>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Check HR permission
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        // Load the text template
        let text_template = self
            .text_template_service
            .get_by_id(template_id, context.clone(), tx.clone().into())
            .await?;

        // Load the billing period
        let billing_period = self
            .billing_period_service
            .get_billing_period_by_id(billing_period_id, context.clone(), tx.clone().into())
            .await?;

        // Load all sales persons and employee work details for enrichment
        let all_sales_persons = self
            .sales_person_service
            .get_all(context.clone(), tx.clone().into())
            .await?;
        let all_work_details = self
            .employee_work_details_service
            .all(context.clone(), tx.clone().into())
            .await?;

        // Build template context data as JSON
        let context_data = json!({
            "billing_period": {
                "id": billing_period.id.to_string(),
                "start_date": billing_period.start_date.to_date().to_string(),
                "end_date": billing_period.end_date.to_date().to_string(),
                "created_at": billing_period.created_at.to_string(),
                "created_by": billing_period.created_by.as_ref(),
                "sales_persons": billing_period.sales_persons.iter().map(|sp| {
                    let sales_person = all_sales_persons.iter().find(|s| s.id == sp.sales_person_id);
                    let name = sales_person.map(|s| s.name.as_ref()).unwrap_or("");
                    let is_paid = sales_person.and_then(|s| s.is_paid).unwrap_or(false);
                    let is_dynamic = all_work_details.iter()
                        .filter(|wd| wd.sales_person_id == sp.sales_person_id)
                        .any(|wd| wd.is_dynamic);
                    let sanitize = |v: f32| -> f64 {
                        let v = v as f64;
                        if v.is_nan() || v.is_infinite() { 0.0 } else { v }
                    };
                    let values_map: serde_json::Map<String, serde_json::Value> = sp.values.iter().map(|(key, value)| {
                        (key.as_str().to_string(), json!({
                            "delta": sanitize(value.value_delta),
                            "ytd_from": sanitize(value.value_ytd_from),
                            "ytd_to": sanitize(value.value_ytd_to),
                            "full_year": sanitize(value.value_full_year),
                        }))
                    }).collect();
                    json!({
                        "id": sp.id.to_string(),
                        "sales_person_id": sp.sales_person_id.to_string(),
                        "name": name,
                        "is_paid": is_paid,
                        "is_dynamic": is_dynamic,
                        "values": sp.values.iter().map(|(key, value)| {
                            json!({
                                "type": key.as_str().as_ref(),
                                "value_delta": sanitize(value.value_delta),
                                "value_ytd_from": sanitize(value.value_ytd_from),
                                "value_ytd_to": sanitize(value.value_ytd_to),
                                "value_full_year": sanitize(value.value_full_year),
                            })
                        }).collect::<Vec<_>>(),
                        "values_map": values_map,
                        "created_at": sp.created_at.to_string(),
                        "created_by": sp.created_by.as_ref(),
                    })
                }).collect::<Vec<_>>(),
            },
            "template": {
                "id": text_template.id.to_string(),
                "template_type": text_template.template_type.as_ref(),
                "created_at": text_template.created_at.map(|dt| dt.to_string()),
                "created_by": text_template.created_by.as_ref().map(|s| s.as_ref()),
            }
        });

        // Render using the appropriate engine
        let rendered = match text_template.template_engine {
            TemplateEngine::Tera => {
                let mut tera = Tera::default();
                tera.add_raw_template("custom_report", &text_template.template_text)
                    .map_err(|e| {
                        tracing::error!("Failed to parse Tera template: {}", e);
                        ServiceError::InternalError
                    })?;
                let template_context = Context::from_serialize(&context_data).map_err(|e| {
                    tracing::error!("Failed to serialize template context: {}", e);
                    ServiceError::InternalError
                })?;
                tera.render("custom_report", &template_context)
                    .map_err(|e| {
                        tracing::error!("Failed to render Tera template: {}", e);
                        ServiceError::InternalError
                    })?
            }
            TemplateEngine::MiniJinja => {
                let env = minijinja::Environment::new();
                env.render_str(&text_template.template_text, context_data)
                    .map_err(|e| {
                        tracing::error!("Failed to render MiniJinja template: {e:#}");
                        ServiceError::InternalError
                    })?
            }
        };

        self.transaction_dao.commit(tx).await?;
        Ok(rendered.into())
    }
}
