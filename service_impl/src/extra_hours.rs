use std::sync::Arc;

use async_trait::async_trait;
use dao::extra_hours;
use service::{
    extra_hours::ExtraHours,
    permission::{Authentication, HR_PRIVILEGE},
    ServiceError,
};
use uuid::Uuid;

pub struct ExtraHoursServiceImpl<
    ExtraHoursDao: dao::extra_hours::ExtraHoursDao,
    PermissionService: service::PermissionService,
    ClockService: service::clock::ClockService,
    UuidService: service::uuid_service::UuidService,
> {
    extra_hours_dao: Arc<ExtraHoursDao>,
    permission_service: Arc<PermissionService>,
    clock_service: Arc<ClockService>,
    uuid_service: Arc<UuidService>,
}

impl<ExtraHoursDao, PermissionService, ClockService, UuidService>
    ExtraHoursServiceImpl<ExtraHoursDao, PermissionService, ClockService, UuidService>
where
    ExtraHoursDao: dao::extra_hours::ExtraHoursDao + Sync + Send,
    PermissionService: service::PermissionService + Sync + Send,
    ClockService: service::clock::ClockService + Sync + Send,
    UuidService: service::uuid_service::UuidService + Sync + Send,
{
    pub fn new(
        extra_hours_dao: Arc<ExtraHoursDao>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            extra_hours_dao,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

#[async_trait]
impl<
        ExtraHoursDao: dao::extra_hours::ExtraHoursDao + Sync + Send,
        PermissionService: service::PermissionService + Sync + Send,
        ClockService: service::clock::ClockService + Sync + Send,
        UuidService: service::uuid_service::UuidService + Sync + Send,
    > service::extra_hours::ExtraHoursService
    for ExtraHoursServiceImpl<ExtraHoursDao, PermissionService, ClockService, UuidService>
{
    type Context = PermissionService::Context;

    async fn find_by_sales_person_id_and_year(
        &self,
        _sales_person_id: Uuid,
        _year: u32,
        _until_week: u8,
        _context: Authentication<Self::Context>,
    ) -> Result<Arc<[ExtraHours]>, ServiceError> {
        unimplemented!()
    }
    async fn create(
        &self,
        extra_hours: &ExtraHours,
        context: Authentication<Self::Context>,
    ) -> Result<ExtraHours, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

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
        _id: Uuid,
        _context: Authentication<Self::Context>,
    ) -> Result<ExtraHours, ServiceError> {
        unimplemented!()
    }
}
