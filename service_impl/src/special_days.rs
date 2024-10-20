use std::sync::Arc;

use async_trait::async_trait;
use dao::special_day::SpecialDayEntity;
use service::{
    permission::{Authentication, SHIFTPLANNER_PRIVILEGE},
    special_days::SpecialDay,
    ServiceError,
};
use uuid::Uuid;

pub struct SpecialDayServiceImpl<
    SpecialDayDao: dao::special_day::SpecialDayDao,
    PermissionService: service::PermissionService,
    ClockService: service::clock::ClockService,
    UuidService: service::uuid_service::UuidService,
> {
    special_day_dao: Arc<SpecialDayDao>,
    permission_service: Arc<PermissionService>,
    clock_service: Arc<ClockService>,
    uuid_service: Arc<UuidService>,
}

impl<SpecialDayDao, PermissionService, ClockService, UuidService>
    SpecialDayServiceImpl<SpecialDayDao, PermissionService, ClockService, UuidService>
where
    SpecialDayDao: dao::special_day::SpecialDayDao + Sync + Send,
    PermissionService: service::PermissionService + Sync + Send,
    ClockService: service::clock::ClockService + Sync + Send,
    UuidService: service::uuid_service::UuidService + Sync + Send,
{
    pub fn new(
        special_day_dao: Arc<SpecialDayDao>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            special_day_dao,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

#[async_trait]
impl<
        SpecialDayDao: dao::special_day::SpecialDayDao + Sync + Send,
        PermissionService: service::PermissionService + Sync + Send,
        ClockService: service::clock::ClockService + Sync + Send,
        UuidService: service::uuid_service::UuidService + Sync + Send,
    > service::special_days::SpecialDayService
    for SpecialDayServiceImpl<SpecialDayDao, PermissionService, ClockService, UuidService>
{
    type Context = PermissionService::Context;

    async fn get_by_week(
        &self,
        year: u32,
        calendar_week: u8,
        _context: Authentication<Self::Context>,
    ) -> Result<Arc<[SpecialDay]>, ServiceError> {
        Ok(self
            .special_day_dao
            .find_by_week(year, calendar_week)
            .await?
            .iter()
            .map(SpecialDay::from)
            .collect())
    }
    async fn create(
        &self,
        special_day: &SpecialDay,
        context: Authentication<Self::Context>,
    ) -> Result<SpecialDay, ServiceError> {
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;
        let mut special_day = special_day.clone();
        special_day.created = Some(self.clock_service.date_time_now());
        let mut entity: SpecialDayEntity = (&special_day).try_into()?;

        if !entity.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !entity.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }
        entity.id = self.uuid_service.new_uuid("special-day-service::create id");
        entity.version = self
            .uuid_service
            .new_uuid("special-day-service::create version");

        self.special_day_dao
            .create(&entity, "special-days-service::create")
            .await?;
        Ok(SpecialDay::from(&entity))
    }
    async fn delete(
        &self,
        special_day_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<SpecialDay, ServiceError> {
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        let mut entity = self
            .special_day_dao
            .find_by_id(special_day_id)
            .await?
            .ok_or_else(|| ServiceError::EntityNotFound(special_day_id))?;

        if entity.deleted.is_some() {
            return Err(ServiceError::EntityNotFound(special_day_id));
        }

        entity.deleted = Some(self.clock_service.date_time_now());
        entity.version = self.uuid_service.new_uuid("booking-version");

        self.special_day_dao
            .update(&entity, "special-days-service::delete")
            .await?;

        Ok(SpecialDay::from(&entity))
    }
}
