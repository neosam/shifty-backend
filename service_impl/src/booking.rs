use async_trait::async_trait;
use service::{
    booking::{Booking, BookingService},
    ServiceError, ValidationFailureItem,
};
use std::sync::Arc;
use uuid::Uuid;

const BOOKING_SERVICE_PROCESS: &str = "booking-service";

pub struct BookingServiceImpl<BookingDao, PermissionService, ClockService, UuidService>
where
    BookingDao: dao::booking::BookingDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
{
    pub booking_dao: Arc<BookingDao>,
    pub permission_service: Arc<PermissionService>,
    pub clock_service: Arc<ClockService>,
    pub uuid_service: Arc<UuidService>,
}
impl<BookingDao, PermissionService, ClockService, UuidService>
    BookingServiceImpl<BookingDao, PermissionService, ClockService, UuidService>
where
    BookingDao: dao::booking::BookingDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
{
    pub fn new(
        booking_dao: Arc<BookingDao>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            booking_dao,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

#[async_trait]
impl<BookingDao, PermissionService, ClockService, UuidService> BookingService
    for BookingServiceImpl<BookingDao, PermissionService, ClockService, UuidService>
where
    BookingDao: dao::booking::BookingDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
{
    type Context = PermissionService::Context;

    async fn get_all(&self, context: Self::Context) -> Result<Arc<[Booking]>, ServiceError> {
        self.permission_service
            .check_permission("hr", context)
            .await?;
        Ok(self
            .booking_dao
            .all()
            .await?
            .iter()
            .map(Booking::from)
            .collect())
    }

    async fn get(&self, id: Uuid, context: Self::Context) -> Result<Booking, ServiceError> {
        self.permission_service
            .check_permission("hr", context)
            .await?;
        let booking_entity = self.booking_dao.find_by_id(id).await?;
        let booking = booking_entity
            .as_ref()
            .map(Booking::from)
            .ok_or_else(move || ServiceError::EntityNotFound(id))?;
        Ok(booking)
    }

    async fn create(
        &self,
        booking: &Booking,
        context: Self::Context,
    ) -> Result<Booking, ServiceError> {
        self.permission_service
            .check_permission("hr", context)
            .await?;

        if booking.id != Uuid::nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if booking.version != Uuid::nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }

        let mut validation = Vec::with_capacity(8);
        if booking.created.is_some() {
            validation.push(ValidationFailureItem::InvalidValue("created".into()));
        }
        if booking.sales_person_id == Uuid::nil() {
            validation.push(ValidationFailureItem::InvalidValue(
                "sales_person_id".into(),
            ));
        }
        if booking.slot_id == Uuid::nil() {
            validation.push(ValidationFailureItem::InvalidValue("slot_id".into()));
        }
        if booking.calendar_week <= 0 {
            validation.push(ValidationFailureItem::InvalidValue("calendar_week".into()));
        }
        if booking.calendar_week > 53 {
            validation.push(ValidationFailureItem::InvalidValue("calendar_week".into()));
        }

        if !validation.is_empty() {
            return Err(ServiceError::ValidationError(validation.into()));
        }

        let new_id = self.uuid_service.new_uuid("booking-id");
        let new_version = self.uuid_service.new_uuid("booking-version");
        let new_booking = Booking {
            id: new_id,
            version: new_version,
            created: Some(self.clock_service.date_time_now()),
            ..booking.clone()
        };

        self.booking_dao
            .create(&(&new_booking).try_into()?, BOOKING_SERVICE_PROCESS)
            .await?;

        Ok(new_booking)
    }

    async fn delete(&self, id: Uuid, context: Self::Context) -> Result<(), ServiceError> {
        self.permission_service
            .check_permission("hr", context)
            .await?;

        let mut booking_entity = self
            .booking_dao
            .find_by_id(id)
            .await?
            .ok_or_else(move || ServiceError::EntityNotFound(id))?;

        booking_entity.deleted = Some(self.clock_service.date_time_now());
        booking_entity.version = self.uuid_service.new_uuid("booking-version");
        self.booking_dao
            .update(&booking_entity, BOOKING_SERVICE_PROCESS)
            .await?;
        Ok(())
    }
}
