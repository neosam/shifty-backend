use std::sync::Arc;

use async_trait::async_trait;
use dao::extra_hours;
use service::{
    extra_hours::ExtraHours,
    permission::{Authentication, HR_PRIVILEGE, SALES_PRIVILEGE},
    ServiceError,
};
use tokio::join;
use uuid::Uuid;

pub struct ExtraHoursServiceImpl<
    ExtraHoursDao: dao::extra_hours::ExtraHoursDao,
    PermissionService: service::PermissionService,
    SalesPersonService: service::sales_person::SalesPersonService,
    ClockService: service::clock::ClockService,
    UuidService: service::uuid_service::UuidService,
> {
    extra_hours_dao: Arc<ExtraHoursDao>,
    permission_service: Arc<PermissionService>,
    sales_person_service: Arc<SalesPersonService>,
    clock_service: Arc<ClockService>,
    uuid_service: Arc<UuidService>,
}

impl<ExtraHoursDao, PermissionService, SalesPersonService, ClockService, UuidService>
    ExtraHoursServiceImpl<
        ExtraHoursDao,
        PermissionService,
        SalesPersonService,
        ClockService,
        UuidService,
    >
where
    ExtraHoursDao: dao::extra_hours::ExtraHoursDao + Sync + Send,
    PermissionService: service::PermissionService + Sync + Send,
    SalesPersonService: service::sales_person::SalesPersonService + Sync + Send,
    ClockService: service::clock::ClockService + Sync + Send,
    UuidService: service::uuid_service::UuidService + Sync + Send,
{
    pub fn new(
        extra_hours_dao: Arc<ExtraHoursDao>,
        permission_service: Arc<PermissionService>,
        sales_person_service: Arc<SalesPersonService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            extra_hours_dao,
            permission_service,
            sales_person_service,
            clock_service,
            uuid_service,
        }
    }
}

#[async_trait]
impl<
        ExtraHoursDao: dao::extra_hours::ExtraHoursDao + Sync + Send,
        PermissionService: service::PermissionService + Sync + Send,
        SalesPersonService: service::sales_person::SalesPersonService<Context = PermissionService::Context>
            + Sync
            + Send,
        ClockService: service::clock::ClockService + Sync + Send,
        UuidService: service::uuid_service::UuidService + Sync + Send,
    > service::extra_hours::ExtraHoursService
    for ExtraHoursServiceImpl<
        ExtraHoursDao,
        PermissionService,
        SalesPersonService,
        ClockService,
        UuidService,
    >
{
    type Context = PermissionService::Context;

    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[ExtraHours]>, ServiceError> {
        let (hr_permission, sales_person_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(sales_person_id, context),
        );
        hr_permission.or(sales_person_permission)?;

        let extra_hours_entities = self
            .extra_hours_dao
            .find_by_sales_person_id_and_year(sales_person_id, year)
            .await?;
        let extra_hours = extra_hours_entities
            .iter()
            .filter(|extra_hours| extra_hours.date_time.iso_week() <= until_week)
            .map(ExtraHours::from)
            .collect::<Vec<ExtraHours>>();
        Ok(extra_hours.into())
    }

    async fn create(
        &self,
        extra_hours: &ExtraHours,
        context: Authentication<Self::Context>,
    ) -> Result<ExtraHours, ServiceError> {
        let (hr_permission, sales_person_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(extra_hours.sales_person_id, context),
        );
        hr_permission.or(sales_person_permission)?;

        let mut extra_hours = extra_hours.to_owned();
        if !extra_hours.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !extra_hours.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }

        extra_hours.id = self.uuid_service.new_uuid("extra_hours_service::create id");
        extra_hours.version = self
            .uuid_service
            .new_uuid("extra_hours_service::create version");
        extra_hours.created = Some(self.clock_service.date_time_now());

        let extra_hours_entity = extra_hours::ExtraHoursEntity::try_from(&extra_hours)?;
        self.extra_hours_dao
            .create(&extra_hours_entity, "extra_hours_service::create")
            .await?;

        Ok(extra_hours.into())
    }
    async fn update(
        &self,
        _entity: &ExtraHours,
        _context: Authentication<Self::Context>,
    ) -> Result<ExtraHours, ServiceError> {
        unimplemented!()
    }

    async fn delete(
        &self,
        extra_hours_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        let (hr_permission, sales_person_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context.clone()),
        );
        hr_permission.or(sales_person_permission)?;

        let mut extra_hours_entity = self
            .extra_hours_dao
            .find_by_id(extra_hours_id)
            .await?
            .ok_or(ServiceError::EntityNotFound(extra_hours_id))?;

        let (hr_permission, user_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(extra_hours_entity.sales_person_id, context),
        );
        hr_permission.or(user_permission)?;

        extra_hours_entity.deleted = Some(self.clock_service.date_time_now());

        self.extra_hours_dao
            .update(&extra_hours_entity, "extra_hours_service::delete")
            .await?;

        Ok(())
    }
}
