use std::sync::Arc;

use async_trait::async_trait;
use dao::employee_work_details::EmployeeWorkDetailsEntity;
use service::{
    employee_work_details::EmployeeWorkDetails,
    permission::{Authentication, HR_PRIVILEGE, SHIFTPLANNER_PRIVILEGE},
    ServiceError,
};
use tokio::join;
use uuid::Uuid;

pub struct EmployeeWorkDetailsServiceImpl<
    EmployeeWorkDetailsDao: dao::employee_work_details::EmployeeWorkDetailsDao,
    SalesPersonService: service::sales_person::SalesPersonService,
    PermissionService: service::PermissionService,
    ClockService: service::clock::ClockService,
    UuidService: service::uuid_service::UuidService,
> {
    working_hours_dao: Arc<EmployeeWorkDetailsDao>,
    sales_person_service: Arc<SalesPersonService>,
    permission_service: Arc<PermissionService>,
    clock_service: Arc<ClockService>,
    uuid_service: Arc<UuidService>,
}

impl<EmployeeWorkDetailsDao, SalesPersonService, PermissionService, ClockService, UuidService>
    EmployeeWorkDetailsServiceImpl<
        EmployeeWorkDetailsDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
where
    EmployeeWorkDetailsDao: dao::employee_work_details::EmployeeWorkDetailsDao + Sync + Send,
    SalesPersonService: service::sales_person::SalesPersonService + Sync + Send,
    PermissionService: service::PermissionService + Sync + Send,
    ClockService: service::clock::ClockService + Sync + Send,
    UuidService: service::uuid_service::UuidService + Sync + Send,
{
    pub fn new(
        working_hours_dao: Arc<EmployeeWorkDetailsDao>,
        sales_person_service: Arc<SalesPersonService>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            working_hours_dao,
            sales_person_service,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

#[async_trait]
impl<
        EmployeeWorkDetailsDao: dao::employee_work_details::EmployeeWorkDetailsDao + Sync + Send,
        SalesPersonService: service::sales_person::SalesPersonService<Context = PermissionService::Context>
            + Sync
            + Send,
        PermissionService: service::PermissionService + Sync + Send,
        ClockService: service::clock::ClockService + Sync + Send,
        UuidService: service::uuid_service::UuidService + Sync + Send,
    > service::employee_work_details::EmployeeWorkDetailsService
    for EmployeeWorkDetailsServiceImpl<
        EmployeeWorkDetailsDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
{
    type Context = PermissionService::Context;

    async fn all(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let working_hours: Arc<[EmployeeWorkDetails]> = self
            .working_hours_dao
            .all()
            .await?
            .iter()
            .map(EmployeeWorkDetails::from)
            .collect::<Vec<EmployeeWorkDetails>>()
            .into();
        Ok(working_hours)
    }
    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError> {
        let (hr_privilege, user_privilege) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(sales_person_id, context),
        );
        hr_privilege.or(user_privilege)?;

        let working_hours: Arc<[EmployeeWorkDetails]> = self
            .working_hours_dao
            .find_by_sales_person_id(sales_person_id)
            .await?
            .iter()
            .map(EmployeeWorkDetails::from)
            .collect::<Vec<EmployeeWorkDetails>>()
            .into();
        Ok(working_hours)
    }
    async fn find_for_week(
        &self,
        sales_person_id: Uuid,
        calendar_week: u8,
        year: u32,
        context: Authentication<Self::Context>,
    ) -> Result<EmployeeWorkDetails, ServiceError> {
        let (hr_privilege, user_privilege) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(sales_person_id, context),
        );
        hr_privilege.or(user_privilege)?;

        let working_hours: EmployeeWorkDetails = self
            .working_hours_dao
            .find_by_sales_person_id(sales_person_id)
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
        Ok(working_hours)
    }

    async fn all_for_week(
        &self,
        calendar_week: u8,
        year: u32,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[EmployeeWorkDetails]>, ServiceError> {
        let shiftplanner_privilege = self
            .permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
            .await;

        match shiftplanner_privilege {
            Ok(_) => {
                // Shiftplanner can see all working hours
                let working_hours: Arc<[EmployeeWorkDetails]> = self
                    .working_hours_dao
                    .find_for_week(calendar_week, year)
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
                    .get_sales_person_current_user(context)
                    .await?
                else {
                    return Ok(Arc::new([]));
                };
                let working_hours: Arc<[EmployeeWorkDetails]> = self
                    .working_hours_dao
                    .find_for_week(calendar_week, year)
                    .await?
                    .iter()
                    .filter(|wh| wh.sales_person_id == sales_person.id)
                    .map(EmployeeWorkDetails::from)
                    .collect::<Vec<EmployeeWorkDetails>>()
                    .into();
                Ok(working_hours)
            }
        }
    }

    async fn create(
        &self,
        working_hours: &EmployeeWorkDetails,
        context: Authentication<Self::Context>,
    ) -> Result<EmployeeWorkDetails, ServiceError> {
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
        self.working_hours_dao
            .create(&entity, "working-hours-service::create")
            .await?;

        Ok(EmployeeWorkDetails::from(&entity))
    }
    async fn update(
        &self,
        employee_work_details: &EmployeeWorkDetails,
        context: Authentication<Self::Context>,
    ) -> Result<EmployeeWorkDetails, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let mut entity = self
            .working_hours_dao
            .find_by_id(employee_work_details.id)
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

        entity.version = self
            .uuid_service
            .new_uuid("working-hours-service::update version");

        self.working_hours_dao
            .update(&entity, "working-hours-service::update")
            .await?;
        Ok(EmployeeWorkDetails::from(&entity))
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<EmployeeWorkDetails, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let entity = self.working_hours_dao.find_by_id(id).await?;
        if let Some(mut entity) = entity {
            entity.deleted = Some(self.clock_service.date_time_now());
            self.working_hours_dao
                .update(&entity, "working-hours-service::delete")
                .await?;
            Ok(EmployeeWorkDetails::from(&entity))
        } else {
            return Err(ServiceError::EntityNotFound(id));
        }
    }
}
