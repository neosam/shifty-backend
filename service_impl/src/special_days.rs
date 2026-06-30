use std::sync::Arc;

use async_trait::async_trait;
use dao::special_day::SpecialDayEntity;
use service::{
    permission::{Authentication, SHIFTPLANNER_PRIVILEGE},
    special_days::{SpecialDay, SpecialDayType},
    ServiceError, ValidationFailureItem,
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
    async fn get_by_year(
        &self,
        year: u32,
        _context: Authentication<Self::Context>,
    ) -> Result<Arc<[SpecialDay]>, ServiceError> {
        Ok(self
            .special_day_dao
            .find_by_year(year)
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

        // Server-side input validation (D-33-06 / D-33-07). The backend is the
        // real trust boundary for this shiftplanner-gated mutation; the type/time
        // coupling and calendar_week bounds are otherwise only enforced in the
        // two front-ends (WR-03).
        let mut validation: Vec<ValidationFailureItem> = Vec::new();
        match special_day.day_type {
            // A ShortDay requires a time_of_day.
            SpecialDayType::ShortDay if special_day.time_of_day.is_none() => {
                validation.push(ValidationFailureItem::InvalidValue(
                    "time_of_day is required for a ShortDay".into(),
                ));
            }
            _ => {}
        }
        let max_week = time::util::weeks_in_year(special_day.year as i32);
        if special_day.calendar_week < 1 || special_day.calendar_week > max_week {
            validation.push(ValidationFailureItem::InvalidValue(
                format!(
                    "calendar_week {} out of range 1..={} for year {}",
                    special_day.calendar_week, max_week, special_day.year
                )
                .into(),
            ));
        }
        if !validation.is_empty() {
            return Err(ServiceError::ValidationError(validation.into()));
        }

        let mut special_day = special_day.clone();
        // A Holiday never carries a time_of_day — normalize so the persisted row
        // matches the type/time invariant regardless of what the client sent.
        if special_day.day_type == SpecialDayType::Holiday {
            special_day.time_of_day = None;
        }
        special_day.created = Some(self.clock_service.date_time_now());
        let mut entity: SpecialDayEntity = (&special_day).try_into()?;

        if !entity.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !entity.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }

        // Duplicate guard (D-33-07): reject a second special day for the same
        // (year, calendar_week, day_of_week). Reporting is keyed by date so a
        // duplicate would not double-credit hours, but it cannot be cleared in
        // one action from the Shiftplan UI and leaves a stale indicator (WR-02).
        let existing = self
            .special_day_dao
            .find_by_week(entity.year, entity.calendar_week)
            .await?;
        if existing
            .iter()
            .any(|e| e.day_of_week == entity.day_of_week)
        {
            return Err(ServiceError::ValidationError(Arc::from([
                ValidationFailureItem::Duplicate,
            ])));
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
