use async_trait::async_trait;
use service::{
    booking::{Booking, BookingService},
    permission::Authentication,
    ServiceError, ValidationFailureItem,
};
use std::sync::Arc;
use tokio::join;
use uuid::Uuid;

const BOOKING_SERVICE_PROCESS: &str = "booking-service";

pub struct BookingServiceImpl<
    BookingDao,
    PermissionService,
    ClockService,
    UuidService,
    SalesPersonService,
    SlotService,
> where
    BookingDao: dao::booking::BookingDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    SlotService: service::slot::SlotService + Send + Sync,
{
    pub booking_dao: Arc<BookingDao>,
    pub permission_service: Arc<PermissionService>,
    pub clock_service: Arc<ClockService>,
    pub uuid_service: Arc<UuidService>,
    pub sales_person_service: Arc<SalesPersonService>,
    pub slot_service: Arc<SlotService>,
}
impl<BookingDao, PermissionService, ClockService, UuidService, SalesPersonService, SlotService>
    BookingServiceImpl<
        BookingDao,
        PermissionService,
        ClockService,
        UuidService,
        SalesPersonService,
        SlotService,
    >
where
    BookingDao: dao::booking::BookingDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    SlotService: service::slot::SlotService + Send + Sync,
{
    pub fn new(
        booking_dao: Arc<BookingDao>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
        sales_person_service: Arc<SalesPersonService>,
        slot_service: Arc<SlotService>,
    ) -> Self {
        Self {
            booking_dao,
            permission_service,
            clock_service,
            uuid_service,
            sales_person_service,
            slot_service,
        }
    }
}

#[async_trait]
impl<BookingDao, PermissionService, ClockService, UuidService, SalesPersonService, SlotService>
    BookingService
    for BookingServiceImpl<
        BookingDao,
        PermissionService,
        ClockService,
        UuidService,
        SalesPersonService,
        SlotService,
    >
where
    BookingDao: dao::booking::BookingDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService<Context = PermissionService::Context>
        + Send
        + Sync,
    SlotService: service::slot::SlotService<Context = PermissionService::Context> + Send + Sync,
{
    type Context = PermissionService::Context;

    async fn get_all(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[Booking]>, ServiceError> {
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

    async fn get(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<Booking, ServiceError> {
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

    async fn get_for_week(
        &self,
        calendar_week: u8,
        year: u32,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[Booking]>, ServiceError> {
        let (hr_permission, sales_permission) = join!(
            self.permission_service
                .check_permission("hr", context.clone()),
            self.permission_service.check_permission("sales", context),
        );
        hr_permission.or(sales_permission)?;

        Ok(self
            .booking_dao
            .find_by_week(calendar_week, year)
            .await?
            .iter()
            .map(Booking::from)
            .collect())
    }

    async fn create(
        &self,
        booking: &Booking,
        context: Authentication<Self::Context>,
    ) -> Result<Booking, ServiceError> {
        self.permission_service
            .check_permission("hr", context.clone())
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
        if !self
            .sales_person_service
            .exists(booking.sales_person_id, context.clone())
            .await?
        {
            validation.push(ValidationFailureItem::IdDoesNotExist(
                "sales_person_id".into(),
                booking.sales_person_id,
            ));
        }
        if !self
            .slot_service
            .exists(booking.slot_id, context.clone())
            .await?
        {
            validation.push(ValidationFailureItem::IdDoesNotExist(
                "slot_id".into(),
                booking.slot_id,
            ));
        }
        if self
            .booking_dao
            .find_by_booking_data(
                booking.sales_person_id,
                booking.slot_id,
                booking.calendar_week,
                booking.year,
            )
            .await?
            .is_some()
        {
            validation.push(ValidationFailureItem::Duplicate);
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

    async fn copy_week(
        &self,
        from_calendar_week: u8,
        from_year: u32,
        to_calendar_week: u8,
        to_year: u32,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        self.permission_service
            .check_permission("hr", context.clone())
            .await?;
        let from_week = self
            .get_for_week(from_calendar_week, from_year, Authentication::Full)
            .await?;
        let to_week = self
            .get_for_week(to_calendar_week, to_year, Authentication::Full)
            .await?;

        // Remove entries which are already in the destination week
        let to_week_ids: Arc<[(Uuid, Uuid)]> = to_week
            .iter()
            .map(|b| (b.sales_person_id, b.slot_id))
            .collect();
        let from_week: Arc<[Booking]> = from_week
            .iter()
            .filter(|b| !to_week_ids.contains(&(b.sales_person_id, b.slot_id)))
            .map(|b| {
                let mut new_booking = b.clone();
                new_booking.id = Uuid::nil();
                new_booking.calendar_week = to_calendar_week as i32;
                new_booking.year = to_year;
                new_booking.created = None;
                new_booking.version = Uuid::nil();
                new_booking
            })
            .collect();

        for booking in from_week.into_iter() {
            self.create(booking, Authentication::Full).await?;
        }
        Ok(())
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
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
