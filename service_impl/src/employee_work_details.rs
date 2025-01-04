use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    employee_work_details::{EmployeeWorkDetailsDao, EmployeeWorkDetailsEntity},
    TransactionDao,
};
use service::{
    clock::ClockService,
    employee_work_details::{EmployeeWorkDetails, EmployeeWorkDetailsService},
    permission::{Authentication, HR_PRIVILEGE, SHIFTPLANNER_PRIVILEGE},
    sales_person::SalesPersonService,
    uuid_service::UuidService,
    PermissionService, ServiceError,
};
use tokio::join;
use uuid::Uuid;

use crate::gen_service_impl;

gen_service_impl! {
    struct EmployeeWorkDetailsServiceImpl: service::employee_work_details::EmployeeWorkDetailsService = EmployeeWorkDetailsServiceDeps {
        EmployeeWorkDetailsDao: dao::employee_work_details::EmployeeWorkDetailsDao<Transaction = Self::Transaction> = employee_work_details_dao,
        SalesPersonService: service::sales_person::SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        PermissionService: service::PermissionService<Context = Self::Context> = permission_service,
        ClockService: service::clock::ClockService = clock_service,
        UuidService: service::uuid_service::UuidService = uuid_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

#[async_trait]
impl<Deps: EmployeeWorkDetailsServiceDeps> EmployeeWorkDetailsService
    for EmployeeWorkDetailsServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let working_hours: Arc<[EmployeeWorkDetails]> = self
            .employee_work_details_dao
            .all(tx.clone())
            .await?
            .iter()
            .map(EmployeeWorkDetails::from)
            .collect::<Vec<EmployeeWorkDetails>>()
            .into();

        self.transaction_dao.commit(tx).await?;
        Ok(working_hours)
    }

    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (hr_privilege, user_privilege) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr_privilege.or(user_privilege)?;

        let working_hours: Arc<[EmployeeWorkDetails]> = self
            .employee_work_details_dao
            .find_by_sales_person_id(sales_person_id, tx.clone())
            .await?
            .iter()
            .map(EmployeeWorkDetails::from)
            .collect::<Vec<EmployeeWorkDetails>>()
            .into();
        self.transaction_dao.commit(tx).await?;
        Ok(working_hours)
    }
    async fn find_for_week(
        &self,
        sales_person_id: Uuid,
        calendar_week: u8,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWorkDetails, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (hr_privilege, user_privilege) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr_privilege.or(user_privilege)?;

        let working_hours: EmployeeWorkDetails = self
            .employee_work_details_dao
            .find_by_sales_person_id(sales_person_id, tx.clone())
            .await?
            .iter()
            .find(|wh| {
                (wh.from_year, wh.from_calendar_week) <= (year, calendar_week)
                    && (wh.to_year, wh.to_calendar_week) >= (year, calendar_week)
            })
            .map(EmployeeWorkDetails::from)
            .ok_or(ServiceError::EntityNotFoundGeneric(
                format!(
                    "sales_person_id: {}, year: {}, calendar_week: {}",
                    sales_person_id, year, calendar_week
                )
                .into(),
            ))?;
        self.transaction_dao.commit(tx).await?;
        Ok(working_hours)
    }

    async fn all_for_week(
        &self,
        calendar_week: u8,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let shiftplanner_privilege = self
            .permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
            .await;

        let result = match shiftplanner_privilege {
            Ok(_) => {
                // Shiftplanner can see all working hours
                let working_hours: Arc<[EmployeeWorkDetails]> = self
                    .employee_work_details_dao
                    .find_for_week(calendar_week, year, tx.clone())
                    .await?
                    .iter()
                    .map(EmployeeWorkDetails::from)
                    .collect::<Vec<EmployeeWorkDetails>>()
                    .into();
                Ok(working_hours)
            }
            Err(_) => {
                // Only load the user's working hours
                let Some(sales_person) = self
                    .sales_person_service
                    .get_sales_person_current_user(context, tx.clone().into())
                    .await?
                else {
                    return Ok(Arc::new([]));
                };
                let working_hours: Arc<[EmployeeWorkDetails]> = self
                    .employee_work_details_dao
                    .find_for_week(calendar_week, year, tx.clone())
                    .await?
                    .iter()
                    .filter(|wh| wh.sales_person_id == sales_person.id)
                    .map(EmployeeWorkDetails::from)
                    .collect::<Vec<EmployeeWorkDetails>>()
                    .into();
                Ok(working_hours)
            }
        };
        self.transaction_dao.commit(tx).await?;
        result
    }

    async fn create(
        &self,
        working_hours: &EmployeeWorkDetails,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWorkDetails, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let mut working_hours = working_hours.to_owned();
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        working_hours.created = Some(self.clock_service.date_time_now());
        let mut entity: EmployeeWorkDetailsEntity = (&working_hours).try_into()?;

        if !entity.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !entity.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }
        entity.id = self
            .uuid_service
            .new_uuid("working-hours-service::create id");
        entity.version = self
            .uuid_service
            .new_uuid("working-hours-service::create version");
        self.employee_work_details_dao
            .create(&entity, "working-hours-service::create", tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(EmployeeWorkDetails::from(&entity))
    }

    async fn update(
        &self,
        employee_work_details: &EmployeeWorkDetails,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWorkDetails, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let mut entity = self
            .employee_work_details_dao
            .find_by_id(employee_work_details.id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(employee_work_details.id))?;
        if entity.version != employee_work_details.version {
            return Err(ServiceError::EntityConflicts(
                entity.id,
                entity.version,
                employee_work_details.version,
            ));
        }

        entity.to_calendar_week = employee_work_details.to_calendar_week;
        entity.to_day_of_week = employee_work_details.to_day_of_week.into();
        entity.to_year = employee_work_details.to_year;
        entity.vacation_days = employee_work_details.vacation_days;
        entity.workdays_per_week = employee_work_details.workdays_per_week;

        entity.version = self
            .uuid_service
            .new_uuid("working-hours-service::update version");

        self.employee_work_details_dao
            .update(&entity, "working-hours-service::update", tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(EmployeeWorkDetails::from(&entity))
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<EmployeeWorkDetails, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let entity = self
            .employee_work_details_dao
            .find_by_id(id, tx.clone())
            .await?;
        let ret = if let Some(mut entity) = entity {
            entity.deleted = Some(self.clock_service.date_time_now());
            self.employee_work_details_dao
                .update(&entity, "working-hours-service::delete", tx.clone())
                .await?;
            Ok(EmployeeWorkDetails::from(&entity))
        } else {
            return Err(ServiceError::EntityNotFound(id));
        };

        self.transaction_dao.commit(tx).await?;
        ret
    }
}
